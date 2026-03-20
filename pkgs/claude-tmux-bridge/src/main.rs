use serde::Deserialize;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::{exit, Command, Stdio};

fn log_debug(msg: &str) {
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/tmux-bridge.log")
    {
        let _ = writeln!(f, "{msg}");
    }
}

fn cmux(args: &[&str]) -> Option<String> {
    log_debug(&format!("  cmux {}", args.join(" ")));
    let out = Command::new("cmux")
        .args(args)
        .stderr(Stdio::piped())
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if out.status.success() {
        if !stdout.is_empty() {
            log_debug(&format!("  -> ok: {}", &stdout[..stdout.len().min(200)]));
        } else {
            log_debug("  -> ok (empty)");
        }
        Some(stdout)
    } else {
        log_debug(&format!("  -> FAIL: {stderr}"));
        None
    }
}

fn cmux_json<T: serde::de::DeserializeOwned>(args: &[&str]) -> Option<T> {
    let mut full_args = vec!["--json"];
    full_args.extend(args);
    cmux(&full_args).and_then(|s| serde_json::from_str(&s).ok())
}

fn cmux_ok(args: &[&str]) -> bool {
    log_debug(&format!("  cmux_ok {}", args.join(" ")));
    let result = Command::new("cmux")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    log_debug(&format!("  -> {result}"));
    result
}

#[derive(Debug, Deserialize, Default)]
struct SurfaceInfo {
    surface_ref: Option<String>,
    window_ref: Option<String>,
    workspace_ref: Option<String>,
    pane_ref: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct IdentifyResponse {
    caller: Option<SurfaceInfo>,
    focused: Option<SurfaceInfo>,
}

#[derive(Debug, Deserialize)]
struct ListPanesResponse {
    panes: Vec<PaneInfo>,
    workspace_ref: String,
}

#[derive(Debug, Deserialize)]
struct PaneInfo {
    #[serde(rename = "ref")]
    pane_ref: String,
    selected_surface_ref: Option<String>,
    surface_refs: Option<Vec<String>>,
    surface_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ResizePaneResponse {
    old_divider_position: f64,
    new_divider_position: f64,
}

fn extract_ref_from_identify(json: &str, field: &str) -> String {
    let resp: IdentifyResponse = serde_json::from_str(json).unwrap_or_default();
    let pick = |info: &SurfaceInfo| match field {
        "surface_ref" => info.surface_ref.clone(),
        "window_ref" => info.window_ref.clone(),
        "workspace_ref" => info.workspace_ref.clone(),
        _ => None,
    };
    resp.caller
        .as_ref()
        .and_then(pick)
        .or_else(|| resp.focused.as_ref().and_then(pick))
        .unwrap_or_default()
}

fn identify_json() -> String {
    cmux(&["identify", "--json"]).unwrap_or_default()
}

fn surface_ref() -> String {
    extract_ref_from_identify(&identify_json(), "surface_ref")
}

fn window_ref() -> String {
    extract_ref_from_identify(&identify_json(), "window_ref")
}

fn workspace_ref() -> String {
    extract_ref_from_identify(&identify_json(), "workspace_ref")
}

fn strip_pane_prefix(s: &str) -> String {
    s.strip_prefix('%').unwrap_or(s).to_string()
}

fn extract_workspace_ref(target: &str) -> Option<String> {
    target
        .split(':')
        .collect::<Vec<_>>()
        .windows(2)
        .find(|pair| pair[0] == "workspace")
        .map(|pair| format!("workspace:{}", pair[1]))
}

fn parse_flag_value<'a, I>(iter: &mut I) -> Option<&'a str>
where
    I: Iterator<Item = &'a str>,
{
    iter.next()
}

fn pane_ref_for_surface(surface: &str) -> Option<String> {
    let json = cmux(&["--json", "identify", "--surface", surface])?;
    let resp: IdentifyResponse = serde_json::from_str(&json).ok()?;
    resp.caller
        .as_ref()
        .and_then(|i| i.pane_ref.clone())
        .or_else(|| resp.focused.as_ref().and_then(|i| i.pane_ref.clone()))
}

/// Move a divider between two panes to a target normalized position (0.0–1.0).
///
/// `pane_fwd`/`dir_fwd`: pane + direction that pushes the divider toward higher positions.
/// `pane_bwd`/`dir_bwd`: pane + direction that pushes the divider toward lower positions.
///
/// For a horizontal split (left|right), fwd = left pane + "-R", bwd = right pane + "-L".
/// For a vertical split (top/bottom), fwd = top pane + "-D", bwd = bottom pane + "-U".
fn set_divider_position(
    pane_fwd: &str, dir_fwd: &str,
    pane_bwd: &str, dir_bwd: &str,
    target: f64,
) {
    let probe: ResizePaneResponse = match cmux_json(&[
        "resize-pane", "--pane", pane_fwd, dir_fwd, "--amount", "1",
    ]) {
        Some(p) => p,
        None => return,
    };
    let diff = (probe.new_divider_position - probe.old_divider_position).abs();
    if diff < 1e-9 {
        return;
    }
    let px_per_unit = 1.0 / diff;

    let delta = target - probe.old_divider_position;
    let fwd_is_positive = probe.new_divider_position > probe.old_divider_position;
    let need_fwd = (delta > 0.0) == fwd_is_positive;

    let total_px = (delta.abs() * px_per_unit).round() as i64;

    if need_fwd {
        // Probe already went 1px forward; subtract it
        let remaining = (total_px - 1).max(0) as u32;
        if remaining > 0 {
            cmux(&[
                "resize-pane", "--pane", pane_fwd, dir_fwd,
                "--amount", &remaining.to_string(),
            ]);
        }
    } else {
        // Need backward: undo the 1px probe + go delta backward
        let needed = (total_px + 1).max(0) as u32;
        if needed > 0 {
            cmux(&[
                "resize-pane", "--pane", pane_bwd, dir_bwd,
                "--amount", &needed.to_string(),
            ]);
        }
    }
}

fn parse_percentage(s: &str) -> Option<u32> {
    s.strip_suffix('%').and_then(|v| v.parse().ok())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    log_debug(&format!("tmux {}", args.join(" ")));

    if args.is_empty() {
        println!("tmux-cmux-bridge active (cmux backend)");
        return;
    }

    let mut iter = args.iter().map(|s| s.as_str()).peekable();

    // Handle global flags before the subcommand
    while let Some(&flag) = iter.peek() {
        match flag {
            "-V" => {
                println!("tmux-cmux-bridge 1.0 (cmux backend)");
                return;
            }
            "-L" => {
                iter.next();
                iter.next(); // skip socket name
            }
            _ => break,
        }
    }

    let cmd = match iter.next() {
        Some(c) => c,
        None => {
            println!("tmux-cmux-bridge active (cmux backend)");
            return;
        }
    };

    let rest: Vec<String> = iter.map(|s| s.to_string()).collect();

    match cmd {
        "split-window" => cmd_split_window(&rest),
        "send-keys" => cmd_send_keys(&rest),
        "select-pane" => cmd_select_pane(&rest),
        "list-panes" => cmd_list_panes(&rest),
        "has-session" | "-has-session" => cmd_has_session(&rest),
        "kill-pane" => cmd_kill_pane(&rest),
        "kill-session" => cmd_kill_session(&rest),
        "ls" | "list-sessions" => cmd_list_sessions(&rest),
        "new-window" => cmd_new_window(&rest),
        "new-session" => cmd_new_session(&rest),
        "display-message" => cmd_display_message(&rest),
        "capture-pane" => cmd_capture_pane(&rest),
        "resize-pane" => cmd_resize_pane(&rest),
        "set-option" | "set" => {}
        "select-layout" => cmd_select_layout(&rest),
        "break-pane" => {}
        "join-pane" => {}
        "kill-window" => cmd_kill_window(&rest),
        _ => {
            let str_rest: Vec<&str> = rest.iter().map(|s| s.as_str()).collect();
            let mut all = vec![cmd];
            all.extend(str_rest);
            if cmux(&all).is_none() {
                eprintln!("tmux-bridge: unsupported command: {cmd}");
                exit(1);
            }
        }
    }
}

fn parse_surface_from_response(resp: &str) -> Option<String> {
    resp.split_whitespace()
        .find(|token| token.starts_with("surface:"))
        .map(|s| s.to_string())
}

fn cmd_split_window(args: &[String]) {
    let mut direction = "right";
    let mut print_id = false;
    let mut target = String::new();
    let mut size_spec = String::new();
    let mut command_args: Vec<String> = Vec::new();
    let mut iter = args.iter().map(|s| s.as_str()).peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "-h" => direction = "right",
            "-v" => direction = "down",
            "-d" => {}
            "-P" => print_id = true,
            "-F" => { parse_flag_value(&mut iter); }
            "-t" => {
                if let Some(t) = parse_flag_value(&mut iter) {
                    target = strip_pane_prefix(t);
                }
            }
            "-l" | "-p" => {
                if let Some(v) = parse_flag_value(&mut iter) {
                    size_spec = v.to_string();
                }
            }
            "--" => {
                command_args = iter.map(|s| s.to_string()).collect();
                break;
            }
            s if s.starts_with('-') => {}
            _ => {
                command_args = std::iter::once(arg)
                    .chain(iter.by_ref())
                    .map(|s| s.to_string())
                    .collect();
                break;
            }
        }
    }

    let source_surface = if !target.is_empty() {
        target.clone()
    } else {
        surface_ref()
    };

    let resp = if !target.is_empty() {
        cmux(&["new-split", direction, "--surface", &target])
    } else {
        cmux(&["new-split", direction])
    };

    let new_surface = resp
        .as_deref()
        .and_then(parse_surface_from_response)
        .unwrap_or_default();

    log_debug(&format!("  split-window: new_surface={new_surface}"));

    if let Some(pct) = parse_percentage(&size_spec) {
        if let (Some(ref src_pane), Some(ref new_pane)) = (
            pane_ref_for_surface(&source_surface),
            pane_ref_for_surface(&new_surface),
        ) {
            let target_pos = (100 - pct) as f64 / 100.0;
            if direction == "right" {
                set_divider_position(src_pane, "-R", new_pane, "-L", target_pos);
            } else {
                set_divider_position(src_pane, "-D", new_pane, "-U", target_pos);
            }
        }
    }

    if !command_args.is_empty() && !new_surface.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(400));
        let joined = command_args.join(" ");
        cmux(&["send", "--surface", &new_surface, &joined]);
        cmux(&["send-key", "--surface", &new_surface, "enter"]);
    }

    if print_id && !new_surface.is_empty() {
        println!("%{new_surface}");
    }
}

fn cmd_send_keys(args: &[String]) {
    let mut target = String::new();
    let mut key_args: Vec<String> = Vec::new();
    let mut iter = args.iter().map(|s| s.as_str()).peekable();

    while let Some(arg) = iter.next() {
        if arg == "-t" {
            if let Some(t) = parse_flag_value(&mut iter) {
                target = strip_pane_prefix(t);
            }
        } else {
            key_args.push(arg.to_string());
        }
    }

    let mut target_args: Vec<&str> = Vec::new();
    if !target.is_empty() {
        target_args.push("--surface");
        target_args.push(&target);
    }

    for arg in &key_args {
        match arg.as_str() {
            "Enter" | "enter" => {
                let mut a: Vec<&str> = vec!["send-key"];
                a.extend(&target_args);
                a.push("enter");
                cmux(&a);
            }
            "Escape" | "escape" | "C-c" => {
                let mut a: Vec<&str> = vec!["send-key"];
                a.extend(&target_args);
                a.push("escape");
                cmux(&a);
            }
            "C-d" => {
                let mut a: Vec<&str> = vec!["send"];
                a.extend(&target_args);
                a.push("\x04");
                cmux(&a);
            }
            "Space" => {
                let mut a: Vec<&str> = vec!["send"];
                a.extend(&target_args);
                a.push(" ");
                cmux(&a);
            }
            "Tab" => {
                let mut a: Vec<&str> = vec!["send-key"];
                a.extend(&target_args);
                a.push("tab");
                cmux(&a);
            }
            "BSpace" => {
                let mut a: Vec<&str> = vec!["send-key"];
                a.extend(&target_args);
                a.push("backspace");
                cmux(&a);
            }
            text => {
                let patched: String;
                let send_text = if text.contains(".claude-unwrapped") {
                    let mut cl = vec!["send"];
                    cl.extend(&target_args);
                    cl.push("\x05\x15"); // Ctrl+E Ctrl+U: clear any stray input
                    cmux(&cl);

                    patched = text.replace(
                        "env CLAUDECODE=1",
                        "env DISABLE_INSTALLATION_CHECKS=1 CLAUDECODE=1",
                    );
                    &patched
                } else {
                    text
                };
                let mut a: Vec<&str> = vec!["send"];
                a.extend(&target_args);
                a.push(send_text);
                cmux(&a);
            }
        }
    }
}

fn cmd_select_pane(args: &[String]) {
    let mut target = String::new();
    let mut has_style = false;
    let mut title = String::new();
    let mut iter = args.iter().map(|s| s.as_str()).peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "-t" => {
                if let Some(t) = parse_flag_value(&mut iter) {
                    target = strip_pane_prefix(t);
                }
            }
            "-P" => {
                has_style = true;
                parse_flag_value(&mut iter); // skip style value
            }
            "-T" => {
                if let Some(t) = parse_flag_value(&mut iter) {
                    title = t.to_string();
                }
            }
            _ => {}
        }
    }

    if !title.is_empty() && !target.is_empty() {
        cmux(&["rename-tab", "--surface", &target, &title]);
    }

    let _ = has_style;
}

fn cmd_list_panes(args: &[String]) {
    let mut target = String::new();
    let mut iter = args.iter().map(|s| s.as_str()).peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "-t" => {
                if let Some(t) = parse_flag_value(&mut iter) {
                    target = t.to_string();
                }
            }
            "-F" => { parse_flag_value(&mut iter); }
            _ => {}
        }
    }

    let mut lp_args: Vec<&str> = vec!["list-panes"];
    let ws_ref_str = if !target.is_empty() {
        extract_workspace_ref(&target).unwrap_or_default()
    } else {
        String::new()
    };
    if !ws_ref_str.is_empty() {
        lp_args.push("--workspace");
        lp_args.push(&ws_ref_str);
    }

    let resp: Option<ListPanesResponse> = cmux_json(&lp_args);
    if let Some(resp) = resp {
        for pane in &resp.panes {
            if let Some(ref surface) = pane.selected_surface_ref {
                println!("%{surface}");
            }
        }
    }
}

fn cmd_has_session(args: &[String]) {
    let mut iter = args.iter().map(|s| s.as_str()).peekable();
    while let Some(arg) = iter.next() {
        if arg == "-t" {
            parse_flag_value(&mut iter);
        }
    }
    if !cmux_ok(&["ping"]) {
        exit(1);
    }
}

fn cmd_kill_pane(args: &[String]) {
    let target = parse_target_flag(args);
    if target.is_empty() {
        cmux(&["close-surface"]);
    } else {
        cmux(&["close-surface", "--surface", &target]);
    }
}

fn cmd_kill_session(args: &[String]) {
    let target = parse_target_flag_raw(args);
    if !target.is_empty() {
        cmux(&["close-workspace", "--workspace", &target]);
    }
}

fn cmd_kill_window(args: &[String]) {
    let target = parse_target_flag_raw(args);
    if !target.is_empty() {
        cmux(&["close-workspace", "--workspace", &target]);
    }
}

// Extract -t value with strip_pane_prefix
fn parse_target_flag(args: &[String]) -> String {
    let mut iter = args.iter().map(|s| s.as_str()).peekable();
    while let Some(arg) = iter.next() {
        if arg == "-t" {
            if let Some(t) = parse_flag_value(&mut iter) {
                return strip_pane_prefix(t);
            }
        }
    }
    String::new()
}

// Extract -t value without stripping prefix
fn parse_target_flag_raw(args: &[String]) -> String {
    let mut iter = args.iter().map(|s| s.as_str()).peekable();
    while let Some(arg) = iter.next() {
        if arg == "-t" {
            if let Some(t) = parse_flag_value(&mut iter) {
                return t.to_string();
            }
        }
    }
    String::new()
}

fn cmd_list_sessions(args: &[String]) {
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut a = vec!["list-workspaces"];
    a.extend(str_args);
    if let Some(out) = cmux(&a) {
        print!("{out}");
        if !out.ends_with('\n') {
            println!();
        }
    }
}

fn cmd_new_window(args: &[String]) {
    let mut name = String::new();
    let mut print_id = false;
    let mut command_args: Vec<String> = Vec::new();
    let mut iter = args.iter().map(|s| s.as_str()).peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "-n" => {
                if let Some(n) = parse_flag_value(&mut iter) {
                    name = n.to_string();
                }
            }
            "-t" | "-F" => { parse_flag_value(&mut iter); }
            "-P" => print_id = true,
            "--" => {
                command_args = iter.map(|s| s.to_string()).collect();
                break;
            }
            s if s.starts_with('-') => {}
            _ => {
                command_args = std::iter::once(arg)
                    .chain(iter.by_ref())
                    .map(|s| s.to_string())
                    .collect();
                break;
            }
        }
    }

    cmux(&["new-workspace"]);

    if !name.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(300));
        cmux(&["rename-workspace", &name]);
    }

    if !command_args.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(400));
        let joined = command_args.join(" ");
        cmux(&["send", &joined]);
        cmux(&["send-key", "enter"]);
    }

    if print_id {
        let s = surface_ref();
        if !s.is_empty() {
            println!("%{s}");
        }
    }
}

fn cmd_new_session(args: &[String]) {
    let mut session_name = String::new();
    let mut window_name = String::new();
    let mut print_id = false;
    let mut iter = args.iter().map(|s| s.as_str()).peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "-d" | "--die-with-parent" => {}
            "-s" => {
                if let Some(s) = parse_flag_value(&mut iter) {
                    session_name = s.to_string();
                }
            }
            "-n" => {
                if let Some(n) = parse_flag_value(&mut iter) {
                    window_name = n.to_string();
                }
            }
            "-P" => print_id = true,
            "-F" => { parse_flag_value(&mut iter); }
            _ => {}
        }
    }

    cmux(&["new-workspace"]);

    let ws_name = if !window_name.is_empty() {
        &window_name
    } else if !session_name.is_empty() {
        &session_name
    } else {
        ""
    };

    if !ws_name.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(300));
        cmux(&["rename-workspace", ws_name]);
    }

    if print_id {
        let s = surface_ref();
        if !s.is_empty() {
            println!("%{s}");
        }
    }
}

fn cmd_display_message(args: &[String]) {
    let mut print_mode = false;
    let mut format_str = String::new();
    let mut iter = args.iter().map(|s| s.as_str()).peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "-p" => print_mode = true,
            "-t" => { parse_flag_value(&mut iter); }
            _ => format_str = arg.to_string(),
        }
    }

    if print_mode && !format_str.is_empty() {
        if format_str.contains("session_name") && format_str.contains("window_index") {
            let win = window_ref();
            let ws = workspace_ref();
            println!("{win}:{ws}");
        } else if format_str.contains("pane_id") {
            let s = surface_ref();
            println!("%{s}");
        } else {
            match cmux(&["display-message", "-p", &format_str]) {
                Some(out) => println!("{out}"),
                None => println!("{format_str}"),
            }
        }
    }
}

fn cmd_capture_pane(args: &[String]) {
    let target = parse_target_flag(args);

    let out = if target.is_empty() {
        cmux(&["read-screen"])
    } else {
        cmux(&["read-screen", "--surface", &target])
    };

    if let Some(text) = out {
        print!("{text}");
        if !text.ends_with('\n') {
            println!();
        }
    }
}

fn cmd_resize_pane(args: &[String]) {
    let mut target = String::new();
    let mut direction = "";
    let mut amount = String::new();
    let mut x_val = String::new();
    let mut y_val = String::new();
    let mut iter = args.iter().map(|s| s.as_str()).peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "-t" => {
                if let Some(t) = parse_flag_value(&mut iter) {
                    target = strip_pane_prefix(t);
                }
            }
            "-L" => direction = "-L",
            "-R" => direction = "-R",
            "-U" => direction = "-U",
            "-D" => direction = "-D",
            "-x" => {
                if let Some(v) = parse_flag_value(&mut iter) {
                    x_val = v.to_string();
                }
            }
            "-y" => {
                if let Some(v) = parse_flag_value(&mut iter) {
                    y_val = v.to_string();
                }
            }
            s if !s.starts_with('-') => amount = s.to_string(),
            _ => {}
        }
    }

    if !x_val.is_empty() || !y_val.is_empty() {
        let target_pane = if !target.is_empty() {
            pane_ref_for_surface(&target)
        } else {
            pane_ref_for_surface(&surface_ref())
        };
        if let Some(ref tpane) = target_pane {
            let panes_resp: Option<ListPanesResponse> = cmux_json(&["list-panes"]);
            if let Some(ref resp) = panes_resp {
                let idx = resp.panes.iter().position(|p| p.pane_ref == **tpane);
                if let Some(pct) = parse_percentage(&x_val) {
                    // -x: horizontal divider. Neighbor is the next pane.
                    if let Some(i) = idx {
                        if i + 1 < resp.panes.len() {
                            set_divider_position(
                                tpane, "-R",
                                &resp.panes[i + 1].pane_ref, "-L",
                                pct as f64 / 100.0,
                            );
                        }
                    }
                }
                if let Some(pct) = parse_percentage(&y_val) {
                    // -y: vertical divider. Neighbor is the next pane.
                    if let Some(i) = idx {
                        if i + 1 < resp.panes.len() {
                            set_divider_position(
                                tpane, "-D",
                                &resp.panes[i + 1].pane_ref, "-U",
                                pct as f64 / 100.0,
                            );
                        }
                    }
                }
            }
        }
        return;
    }

    if direction.is_empty() {
        return;
    }

    let mut a: Vec<&str> = vec!["resize-pane"];
    if !target.is_empty() {
        a.push("--pane");
        a.push(&target);
    }
    a.push(direction);
    if !amount.is_empty() {
        a.push("--amount");
        a.push(&amount);
    }
    cmux(&a);
}

fn cmd_select_layout(args: &[String]) {
    let mut layout = String::new();
    let mut iter = args.iter().map(|s| s.as_str()).peekable();
    while let Some(arg) = iter.next() {
        match arg {
            "-t" => { parse_flag_value(&mut iter); }
            s if !s.starts_with('-') => layout = s.to_string(),
            _ => {}
        }
    }
    log_debug(&format!("  select-layout: {layout}"));
    if layout == "main-vertical" {
        apply_main_vertical_layout();
    }
}

fn apply_main_vertical_layout() {
    // Use --id-format both to get UUIDs (needed for drag-surface-to-split V1 API)
    // Global flags like --id-format must come before the subcommand.
    let resp: ListPanesResponse = match cmux_json(&["--id-format", "both", "list-panes"]) {
        Some(r) => r,
        None => return,
    };
    if resp.panes.len() <= 2 {
        return;
    }

    let teammates: Vec<&PaneInfo> = resp.panes.iter().skip(1).collect();
    let n = teammates.len();
    if n <= 1 {
        return;
    }

    // Collect surface refs, sorted by numeric ID to preserve spawn order.
    // list-panes returns tree order which gets scrambled by intermediate splits;
    // surface IDs increase monotonically with creation time.
    let mut surface_refs: Vec<String> = teammates
        .iter()
        .filter_map(|p| p.selected_surface_ref.clone())
        .collect();
    if surface_refs.len() != n {
        log_debug("  main-vertical: surface_refs count mismatch, aborting");
        return;
    }
    surface_refs.sort_by_key(|s| {
        s.strip_prefix("surface:").and_then(|n| n.parse::<u64>().ok()).unwrap_or(0)
    });

    // Build ref→UUID mapping from the initial pane data
    let mut ref_to_uuid: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for pane in &resp.panes {
        if let (Some(refs), Some(ids)) = (&pane.surface_refs, &pane.surface_ids) {
            for (r, id) in refs.iter().zip(ids.iter()) {
                ref_to_uuid.insert(r.clone(), id.clone());
            }
        }
    }

    // Phase 1: CONSOLIDATE — move all teammate surfaces into the first teammate's pane
    let target_pane = &teammates[0].pane_ref;
    for surface in &surface_refs[1..] {
        cmux(&[
            "move-surface", "--surface", surface, "--pane", target_pane, "--no-focus",
        ]);
    }

    cmux(&["select-workspace", "--workspace", &resp.workspace_ref]);

    // Phase 2: SEPARATE — drag each surface out as a vertical split
    // drag-surface-to-split is a V1 API that requires UUIDs (not surface:N refs).
    for i in (1..n).rev() {
        let uuid = match ref_to_uuid.get(&surface_refs[i]) {
            Some(u) => u.clone(),
            None => {
                log_debug(&format!("  main-vertical: no UUID for {}", surface_refs[i]));
                continue;
            }
        };
        cmux(&["drag-surface-to-split", "--surface", &uuid, "down"]);
    }

    // Phase 3: EQUALIZE — set divider positions for equal heights
    // Left-leaning tree: vsplit(vsplit(vsplit(S1, S2), S3), S4)
    // Split k is between tm_panes[k-1] (bottom of upper group) and tm_panes[k] (below).
    let fresh: ListPanesResponse = match cmux_json(&["list-panes"]) {
        Some(r) => r,
        None => return,
    };
    let tm_panes: Vec<&PaneInfo> = fresh.panes.iter().skip(1).collect();
    for k in 2..n {
        if k < tm_panes.len() {
            let target_pos = k as f64 / (k + 1) as f64;
            set_divider_position(
                &tm_panes[k - 1].pane_ref, "-D",
                &tm_panes[k].pane_ref, "-U",
                target_pos,
            );
        }
    }

    // Return focus to the leader pane
    if let Some(leader) = fresh.panes.first() {
        cmux(&["focus-pane", "--pane", &leader.pane_ref]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_caller_priority() {
        let json = r#"{
            "caller": {"surface_ref": "surface:1", "window_ref": "window:10", "workspace_ref": "workspace:100"},
            "focused": {"surface_ref": "surface:2", "window_ref": "window:20", "workspace_ref": "workspace:200"}
        }"#;
        assert_eq!(extract_ref_from_identify(json, "surface_ref"), "surface:1");
        assert_eq!(extract_ref_from_identify(json, "window_ref"), "window:10");
        assert_eq!(
            extract_ref_from_identify(json, "workspace_ref"),
            "workspace:100"
        );
    }

    #[test]
    fn test_identify_focused_fallback() {
        let json = r#"{
            "caller": null,
            "focused": {"surface_ref": "surface:5", "window_ref": "window:50", "workspace_ref": "workspace:500"}
        }"#;
        assert_eq!(extract_ref_from_identify(json, "surface_ref"), "surface:5");
        assert_eq!(extract_ref_from_identify(json, "window_ref"), "window:50");
        assert_eq!(
            extract_ref_from_identify(json, "workspace_ref"),
            "workspace:500"
        );
    }

    #[test]
    fn test_identify_missing_fields() {
        let json = r#"{"caller": {"surface_ref": null}, "focused": null}"#;
        assert_eq!(extract_ref_from_identify(json, "surface_ref"), "");
        assert_eq!(extract_ref_from_identify(json, "window_ref"), "");

        assert_eq!(extract_ref_from_identify("{}", "surface_ref"), "");
        assert_eq!(extract_ref_from_identify("", "surface_ref"), "");
        assert_eq!(extract_ref_from_identify("not json", "surface_ref"), "");
    }

    #[test]
    fn test_parse_surface_from_response() {
        assert_eq!(
            parse_surface_from_response("OK surface:11 workspace:2"),
            Some("surface:11".to_string())
        );
        assert_eq!(
            parse_surface_from_response("surface:99"),
            Some("surface:99".to_string())
        );
        assert_eq!(parse_surface_from_response("OK"), None);
        assert_eq!(parse_surface_from_response(""), None);
    }

    #[test]
    fn test_extract_workspace_ref() {
        assert_eq!(
            extract_workspace_ref("window:1:workspace:2"),
            Some("workspace:2".to_string())
        );
        assert_eq!(
            extract_workspace_ref("workspace:5"),
            Some("workspace:5".to_string())
        );
        assert_eq!(extract_workspace_ref("window:1"), None);
        assert_eq!(extract_workspace_ref(""), None);
    }

    #[test]
    fn test_strip_pane_prefix() {
        assert_eq!(strip_pane_prefix("%surface:5"), "surface:5");
        assert_eq!(strip_pane_prefix("surface:5"), "surface:5");
        assert_eq!(strip_pane_prefix("%"), "");
        assert_eq!(strip_pane_prefix(""), "");
    }

    #[test]
    fn test_parse_list_panes_json() {
        let json = r#"{
            "panes": [
                {"ref": "pane:1", "selected_surface_ref": "surface:1", "surface_refs": ["surface:1"]},
                {"ref": "pane:2", "selected_surface_ref": "surface:2", "surface_refs": ["surface:2"]},
                {"ref": "pane:3", "selected_surface_ref": "surface:3", "surface_refs": ["surface:3"]}
            ],
            "workspace_ref": "workspace:1"
        }"#;
        let resp: ListPanesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.panes.len(), 3);
        assert_eq!(resp.panes[0].pane_ref, "pane:1");
        assert_eq!(
            resp.panes[1].selected_surface_ref,
            Some("surface:2".into())
        );
        assert_eq!(resp.workspace_ref, "workspace:1");
    }

    #[test]
    fn test_parse_percentage() {
        assert_eq!(parse_percentage("30%"), Some(30));
        assert_eq!(parse_percentage("50"), None);
        assert_eq!(parse_percentage("70%"), Some(70));
        assert_eq!(parse_percentage(""), None);
        assert_eq!(parse_percentage("%"), None);
    }

    #[test]
    fn test_parse_resize_response() {
        let json = r#"{"old_divider_position": 0.3, "new_divider_position": 0.31}"#;
        let resp: ResizePaneResponse = serde_json::from_str(json).unwrap();
        assert!((resp.old_divider_position - 0.3).abs() < 1e-9);
        assert!((resp.new_divider_position - 0.31).abs() < 1e-9);
    }

    #[test]
    fn test_equalization_targets() {
        let n = 3;
        let targets: Vec<f64> = (2..n).map(|k| k as f64 / (k + 1) as f64).collect();
        assert_eq!(targets.len(), 1);
        assert!((targets[0] - 2.0 / 3.0).abs() < 1e-9);

        let n = 4;
        let targets: Vec<f64> = (2..n).map(|k| k as f64 / (k + 1) as f64).collect();
        assert_eq!(targets.len(), 2);
        assert!((targets[0] - 2.0 / 3.0).abs() < 1e-9);
        assert!((targets[1] - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_consolidation_order() {
        let json = r#"{
            "panes": [
                {"ref": "pane:1", "selected_surface_ref": "surface:10", "surface_refs": ["surface:10"]},
                {"ref": "pane:2", "selected_surface_ref": "surface:20", "surface_refs": ["surface:20"]},
                {"ref": "pane:3", "selected_surface_ref": "surface:30", "surface_refs": ["surface:30"]},
                {"ref": "pane:4", "selected_surface_ref": "surface:40", "surface_refs": ["surface:40"]}
            ],
            "workspace_ref": "workspace:1"
        }"#;
        let resp: ListPanesResponse = serde_json::from_str(json).unwrap();
        let teammates: Vec<&PaneInfo> = resp.panes.iter().skip(1).collect();
        let surface_refs: Vec<String> = teammates
            .iter()
            .filter_map(|p| p.selected_surface_ref.clone())
            .collect();
        assert_eq!(
            surface_refs,
            vec!["surface:20", "surface:30", "surface:40"]
        );
        assert_eq!(teammates[0].pane_ref, "pane:2");
    }
}
