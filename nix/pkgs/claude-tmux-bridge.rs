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

/// Extract a string field from JSON like `"field_ref": "value"` or `"field_ref" : null`
/// Searches for the key and returns the string value, or None if null/missing.
fn json_str<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{}\"", key);
    let idx = json.find(&needle)? + needle.len();
    let rest = &json[idx..];
    // Skip whitespace and colon
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim_start();
    if rest.starts_with("null") {
        return None;
    }
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// Extract a ref from the identify JSON, trying caller first then focused.
fn extract_ref(json: &str, field: &str) -> String {
    // Try to find the field in the "caller" object first
    if let Some(caller_start) = json.find("\"caller\"") {
        let caller_rest = &json[caller_start..];
        if let Some(obj_start) = caller_rest.find('{') {
            // Find matching close brace (simple nesting)
            let obj = &caller_rest[obj_start..];
            if let Some(end) = find_matching_brace(obj) {
                let caller_obj = &obj[..=end];
                if let Some(v) = json_str(caller_obj, field) {
                    return v.to_string();
                }
            }
        }
    }
    // Fallback to "focused" object
    if let Some(focused_start) = json.find("\"focused\"") {
        let focused_rest = &json[focused_start..];
        if let Some(obj_start) = focused_rest.find('{') {
            let obj = &focused_rest[obj_start..];
            if let Some(end) = find_matching_brace(obj) {
                let focused_obj = &obj[..=end];
                if let Some(v) = json_str(focused_obj, field) {
                    return v.to_string();
                }
            }
        }
    }
    String::new()
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn identify_json() -> String {
    cmux(&["identify", "--json"]).unwrap_or_default()
}

fn surface_ref() -> String {
    extract_ref(&identify_json(), "surface_ref")
}

fn window_ref() -> String {
    extract_ref(&identify_json(), "window_ref")
}

fn workspace_ref() -> String {
    extract_ref(&identify_json(), "workspace_ref")
}

fn strip_pane_prefix(s: &str) -> String {
    s.strip_prefix('%').unwrap_or(s).to_string()
}

/// Extract "workspace:N" from a compound target like "window:1:workspace:2"
fn extract_workspace_ref(target: &str) -> Option<String> {
    let parts: Vec<&str> = target.split(':').collect();
    for i in 0..parts.len().saturating_sub(1) {
        if parts[i] == "workspace" {
            return Some(format!("workspace:{}", parts[i + 1]));
        }
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    log_debug(&format!("tmux {}", args.join(" ")));

    if args.is_empty() {
        println!("tmux-cmux-bridge active (cmux backend)");
        return;
    }

    let mut i = 0;

    // Handle global flags before the subcommand
    while i < args.len() {
        match args[i].as_str() {
            "-V" => {
                println!("tmux-cmux-bridge 1.0 (cmux backend)");
                return;
            }
            "-L" => {
                // Named socket — ignored in cmux (single instance)
                i += 2;
            }
            _ => break,
        }
    }

    if i >= args.len() {
        println!("tmux-cmux-bridge active (cmux backend)");
        return;
    }

    let cmd = &args[i];
    let rest: Vec<String> = args[i + 1..].to_vec();

    match cmd.as_str() {
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
        "select-layout" => {}
        "break-pane" => {}       // hide — no-op, cmux doesn't have session-move
        "join-pane" => {}        // show — no-op
        "kill-window" => cmd_kill_window(&rest),
        _ => {
            let str_rest: Vec<&str> = rest.iter().map(|s| s.as_str()).collect();
            let mut all = vec![cmd.as_str()];
            all.extend(str_rest);
            if cmux(&all).is_none() {
                eprintln!("tmux-bridge: unsupported command: {cmd}");
                exit(1);
            }
        }
    }
}

/// Parse "OK surface:11 workspace:2" → Some("surface:11")
fn parse_surface_from_response(resp: &str) -> Option<String> {
    for token in resp.split_whitespace() {
        if token.starts_with("surface:") {
            return Some(token.to_string());
        }
    }
    None
}

fn cmd_split_window(args: &[String]) {
    let mut direction = "right";
    let mut print_id = false;
    let mut target = String::new();
    let mut command_args: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" => direction = "right",
            "-v" => direction = "down",
            "-d" | "-P" => {
                if args[i] == "-P" { print_id = true; }
            }
            "-F" => { i += 1; }
            "-t" => {
                if let Some(t) = args.get(i + 1) {
                    target = strip_pane_prefix(t);
                }
                i += 1;
            }
            "-l" | "-p" => { i += 1; }
            "--" => {
                command_args = args[i + 1..].to_vec();
                break;
            }
            s if s.starts_with('-') => {}
            _ => {
                command_args = args[i..].to_vec();
                break;
            }
        }
        i += 1;
    }

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
    let mut i = 0;

    while i < args.len() {
        if args[i] == "-t" {
            if let Some(t) = args.get(i + 1) {
                target = strip_pane_prefix(t);
                i += 2;
                continue;
            }
        }
        key_args.push(args[i].clone());
        i += 1;
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
                let mut a: Vec<&str> = vec!["send"];
                a.extend(&target_args);
                a.push(text);
                cmux(&a);
            }
        }
    }
}

fn cmd_select_pane(args: &[String]) {
    let mut target = String::new();
    let mut has_style = false;
    let mut title = String::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-t" => {
                if let Some(t) = args.get(i + 1) {
                    target = strip_pane_prefix(t);
                    i += 2;
                    continue;
                }
            }
            "-P" => {
                has_style = true;
                i += 2;
                continue;
            }
            "-T" => {
                if let Some(t) = args.get(i + 1) {
                    title = t.clone();
                    i += 2;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if !title.is_empty() && !target.is_empty() {
        cmux(&["rename-tab", "--surface", &target, &title]);
    }

    // Style setting (-P) is cosmetic — no-op
    // Pure focus without style/title: also no-op since cmux doesn't have focus-surface
    let _ = has_style;
}

fn cmd_list_panes(args: &[String]) {
    let mut target = String::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-t" => {
                if let Some(t) = args.get(i + 1) {
                    target = t.clone();
                    i += 2;
                    continue;
                }
            }
            "-F" => { i += 1; }
            _ => {}
        }
        i += 1;
    }

    let mut ws_args: Vec<&str> = vec!["list-pane-surfaces"];
    let ws_ref_str = if !target.is_empty() {
        extract_workspace_ref(&target).unwrap_or_default()
    } else {
        String::new()
    };
    if !ws_ref_str.is_empty() {
        ws_args.push("--workspace");
        ws_args.push(&ws_ref_str);
    }

    if let Some(raw) = cmux(&ws_args) {
        for line in raw.lines() {
            if let Some(surface) = extract_surface_ref(line) {
                println!("%{surface}");
            }
        }
    }
}

fn extract_surface_ref(line: &str) -> Option<&str> {
    let idx = line.find("surface:")?;
    let rest = &line[idx..];
    // surface:NN — find end of the ref
    let end = rest
        .find(|c: char| c.is_whitespace() || c == ',' || c == '}')
        .unwrap_or(rest.len());
    let surface = &rest[..end];
    if surface.len() > "surface:".len() {
        Some(surface)
    } else {
        None
    }
}

fn cmd_has_session(args: &[String]) {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-t" { i += 2; continue; }
        i += 1;
    }
    if !cmux_ok(&["ping"]) {
        exit(1);
    }
}

fn cmd_kill_pane(args: &[String]) {
    let mut target = String::new();
    let mut i = 0;

    while i < args.len() {
        if args[i] == "-t" {
            if let Some(t) = args.get(i + 1) {
                target = strip_pane_prefix(t);
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    if target.is_empty() {
        cmux(&["close-surface"]);
    } else {
        cmux(&["close-surface", "--surface", &target]);
    }
}

fn cmd_kill_session(args: &[String]) {
    let mut target = String::new();
    let mut i = 0;

    while i < args.len() {
        if args[i] == "-t" {
            if let Some(t) = args.get(i + 1) {
                target = t.clone();
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    if !target.is_empty() {
        cmux(&["close-workspace", "--workspace", &target]);
    }
}

fn cmd_kill_window(args: &[String]) {
    let mut target = String::new();
    let mut i = 0;

    while i < args.len() {
        if args[i] == "-t" {
            if let Some(t) = args.get(i + 1) {
                target = t.clone();
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    if !target.is_empty() {
        cmux(&["close-workspace", "--workspace", &target]);
    }
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
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                if let Some(n) = args.get(i + 1) {
                    name = n.clone();
                    i += 2;
                    continue;
                }
            }
            "-t" | "-F" => { i += 2; continue; }
            "-P" => print_id = true,
            "--" => {
                command_args = args[i + 1..].to_vec();
                break;
            }
            s if s.starts_with('-') => {}
            _ => {
                command_args = args[i..].to_vec();
                break;
            }
        }
        i += 1;
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
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-d" => {}
            "-s" => {
                if let Some(s) = args.get(i + 1) {
                    session_name = s.clone();
                    i += 2;
                    continue;
                }
            }
            "-n" => {
                if let Some(n) = args.get(i + 1) {
                    window_name = n.clone();
                    i += 2;
                    continue;
                }
            }
            "-P" => print_id = true,
            "-F" => { i += 2; continue; }
            "--die-with-parent" => {}
            _ => {}
        }
        i += 1;
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
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-p" => print_mode = true,
            "-t" => { i += 2; continue; }
            _ => format_str = args[i].clone(),
        }
        i += 1;
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
    let mut target = String::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-t" => {
                if let Some(t) = args.get(i + 1) {
                    target = strip_pane_prefix(t);
                    i += 2;
                    continue;
                }
            }
            "-p" => {}
            _ => {}
        }
        i += 1;
    }

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
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-t" => {
                if let Some(t) = args.get(i + 1) {
                    target = strip_pane_prefix(t);
                    i += 2;
                    continue;
                }
            }
            "-L" => direction = "-L",
            "-R" => direction = "-R",
            "-U" => direction = "-U",
            "-D" => direction = "-D",
            "-x" | "-y" => {
                // Absolute sizing not supported by cmux — skip
                i += 2;
                continue;
            }
            s if !s.starts_with('-') => {
                amount = s.to_string();
            }
            _ => {}
        }
        i += 1;
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
