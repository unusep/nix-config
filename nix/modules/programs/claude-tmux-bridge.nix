{ pkgs, ... }:
let
  jq = "${pkgs.jq}/bin/jq";

  tmuxBridge = pkgs.writeShellScriptBin "tmux" ''
    set -euo pipefail

    cmd="''${1:-}"
    shift 2>/dev/null || true

    # Helper: get caller's workspace ref from cmux identify
    get_workspace_ref() {
      cmux identify --json 2>/dev/null | ${jq} -r '(.caller.workspace_ref // .focused.workspace_ref) // empty' 2>/dev/null || true
    }

    # Helper: get caller's window ref from cmux identify
    get_window_ref() {
      cmux identify --json 2>/dev/null | ${jq} -r '(.caller.window_ref // .focused.window_ref) // empty' 2>/dev/null || true
    }

    # Helper: get caller's surface ref from cmux identify
    get_surface_ref() {
      cmux identify --json 2>/dev/null | ${jq} -r '(.caller.surface_ref // .focused.surface_ref) // empty' 2>/dev/null || true
    }

    case "$cmd" in
      split-window)
        direction="right"
        detach=false
        print_id=false
        command_args=()

        while [[ $# -gt 0 ]]; do
          case "$1" in
            -h) direction="right"; shift ;;
            -v) direction="down"; shift ;;
            -d) detach=true; shift ;;
            -P) print_id=true; shift ;;
            -F) shift 2 ;;
            -t) shift 2 ;;
            -l|-p) shift 2 ;;
            --) shift; command_args=("$@"); break ;;
            -*) shift ;;
            *) command_args=("$@"); break ;;
          esac
        done

        current=$(get_surface_ref)

        cmux new-split "$direction" 2>/dev/null

        new_surface=$(get_surface_ref)

        if [[ ''${#command_args[@]} -gt 0 ]]; then
          sleep 0.4
          cmux send "''${command_args[*]}"
          cmux send-key enter
        fi

        if $detach && [[ -n "$current" ]]; then
          cmux focus-surface --surface "$current"
        fi

        if $print_id && [[ -n "$new_surface" ]]; then
          echo "%$new_surface"
        fi
        ;;

      send-keys)
        target=""
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -t) target="''${2#\%}"; shift 2 ;;
            *) break ;;
          esac
        done

        target_args=()
        [[ -n "$target" ]] && target_args=(--surface "$target")

        for arg in "$@"; do
          case "$arg" in
            Enter|enter)   cmux send-key "''${target_args[@]}" enter ;;
            Escape|escape) cmux send-key "''${target_args[@]}" escape ;;
            C-c)           cmux send-key "''${target_args[@]}" escape ;;
            C-d)           cmux send "''${target_args[@]}" $'\x04' ;;
            Space)         cmux send "''${target_args[@]}" " " ;;
            Tab)           cmux send-key "''${target_args[@]}" tab ;;
            BSpace)        cmux send-key "''${target_args[@]}" backspace ;;
            *)             cmux send "''${target_args[@]}" "$arg" ;;
          esac
        done
        ;;

      select-pane)
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -t) cmux focus-surface --surface "''${2#\%}"; shift 2 ;;
            -L) cmux focus-surface --surface left 2>/dev/null || true; shift ;;
            -R) cmux focus-surface --surface right 2>/dev/null || true; shift ;;
            -U) cmux focus-surface --surface up 2>/dev/null || true; shift ;;
            -D) cmux focus-surface --surface down 2>/dev/null || true; shift ;;
            *) shift ;;
          esac
        done
        ;;

      list-panes)
        # Claude Code runs: list-panes -t <target> -F "#{pane_id}"
        # We need to return one surface ref per line as %surface:N
        target=""
        format=""
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -t) target="$2"; shift 2 ;;
            -F) format="$2"; shift 2 ;;
            *) shift ;;
          esac
        done

        # Parse workspace from target (format: "window:N:workspace:M" or "workspace:M")
        ws_arg=""
        if [[ -n "$target" ]]; then
          # Extract workspace ref from compound target
          ws_ref=$(echo "$target" | grep -oE 'workspace:[0-9]+' || true)
          if [[ -n "$ws_ref" ]]; then
            ws_arg="--workspace $ws_ref"
          fi
        fi

        # Get list of surfaces in the workspace
        raw=$(cmux list-pane-surfaces $ws_arg 2>/dev/null || true)

        # Parse surface refs from output lines like "* surface:22  ..."
        echo "$raw" | while IFS= read -r line; do
          ref=$(echo "$line" | grep -oE 'surface:[0-9]+' || true)
          if [[ -n "$ref" ]]; then
            echo "%$ref"
          fi
        done
        ;;

      has-session|-has-session)
        # Parse -t target
        target=""
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -t) target="$2"; shift 2 ;;
            *) shift ;;
          esac
        done
        cmux ping >/dev/null 2>&1
        ;;

      kill-pane)
        target=""
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -t) target="''${2#\%}"; shift 2 ;;
            *) shift ;;
          esac
        done
        if [[ -n "$target" ]]; then
          cmux close-surface --surface "$target"
        else
          cmux close-surface
        fi
        ;;

      kill-session)
        target=""
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -t) target="$2"; shift 2 ;;
            *) shift ;;
          esac
        done
        [[ -n "$target" ]] && cmux close-workspace --workspace "$target"
        ;;

      ls|list-sessions)
        cmux list-workspaces "$@"
        ;;

      new-window)
        name=""
        print_id=false
        session_target=""
        command_args=()
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -n) name="$2"; shift 2 ;;
            -t) session_target="$2"; shift 2 ;;
            -P) print_id=true; shift ;;
            -F) shift 2 ;;
            --) shift; command_args=("$@"); break ;;
            -*) shift ;;
            *) command_args=("$@"); break ;;
          esac
        done

        cmux new-workspace 2>/dev/null

        if [[ -n "$name" ]]; then
          sleep 0.3
          cmux rename-workspace "$name" 2>/dev/null || true
        fi

        if [[ ''${#command_args[@]} -gt 0 ]]; then
          sleep 0.4
          cmux send "''${command_args[*]}"
          cmux send-key enter
        fi

        if $print_id; then
          new_surface=$(get_surface_ref)
          [[ -n "$new_surface" ]] && echo "%$new_surface"
        fi
        ;;

      new-session)
        # Claude Code runs: new-session -d -s <name> -n <window_name> -P -F "#{pane_id}"
        # or: new-session -d -s <name>
        session_name=""
        window_name=""
        detach=false
        print_id=false
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -d) detach=true; shift ;;
            -s) session_name="$2"; shift 2 ;;
            -n) window_name="$2"; shift 2 ;;
            -P) print_id=true; shift ;;
            -F) shift 2 ;;
            --die-with-parent) shift ;;
            -*) shift ;;
            *) shift ;;
          esac
        done

        # Create a new workspace (= tmux session+window)
        cmux new-workspace 2>/dev/null

        ws_name="''${window_name:-$session_name}"
        if [[ -n "$ws_name" ]]; then
          sleep 0.3
          cmux rename-workspace "$ws_name" 2>/dev/null || true
        fi

        if $print_id; then
          new_surface=$(get_surface_ref)
          [[ -n "$new_surface" ]] && echo "%$new_surface"
        fi
        ;;

      display-message)
        # Claude Code runs:
        #   display-message -p "#{session_name}:#{window_index}"  → get window target
        #   display-message -p "#{pane_id}"                       → get current pane id
        print_mode=false
        target=""
        format_str=""
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -p) print_mode=true; shift ;;
            -t) target="$2"; shift 2 ;;
            *)  format_str="$1"; shift ;;
          esac
        done

        if $print_mode && [[ -n "$format_str" ]]; then
          case "$format_str" in
            *session_name*window_index*)
              # Return "window_ref:workspace_ref" as the window target
              win_ref=$(get_window_ref)
              ws_ref=$(get_workspace_ref)
              echo "''${win_ref}:''${ws_ref}"
              ;;
            *pane_id*)
              surface_ref=$(get_surface_ref)
              echo "%$surface_ref"
              ;;
            *)
              cmux display-message -p "$format_str" 2>/dev/null || echo "$format_str"
              ;;
          esac
        else
          cmux display-message "''${format_str}" 2>/dev/null || true
        fi
        ;;

      capture-pane)
        target=""
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -t) target="''${2#\%}"; shift 2 ;;
            -p) shift ;;
            *) shift ;;
          esac
        done
        if [[ -n "$target" ]]; then
          cmux read-screen --surface "$target"
        else
          cmux read-screen
        fi
        ;;

      resize-pane)
        cmux resize-pane "$@"
        ;;

      set-option|set)
        # No-op for tmux options like pane-border-status
        # Claude Code sets: set-option -w -t <target> pane-border-status top
        ;;

      "")
        echo "tmux-cmux-bridge active (cmux backend)"
        exit 0
        ;;

      *)
        cmux "$cmd" "$@" 2>/dev/null || {
          echo "tmux-bridge: unsupported command: $cmd" >&2
          exit 1
        }
        ;;
    esac
  '';
in
{
  programs.zsh.shellAliases.cct = builtins.concatStringsSep " " [
    "TMUX=cmux-bridge"
    "PATH=${tmuxBridge}/bin:$PATH"
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1"
    "claude --teammate-mode tmux"
  ];

  programs.claude-code.settings.env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1";
}
