{ pkgs, ... }:
let
  tmuxBridge = pkgs.writeShellScriptBin "tmux" ''
    set -euo pipefail

    cmd="''${1:-}"
    shift 2>/dev/null || true

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

        current=$(cmux identify --json 2>/dev/null | ${pkgs.jq}/bin/jq -r '.surface_ref // empty' 2>/dev/null || true)

        cmux new-split "$direction" 2>/dev/null

        new_surface=$(cmux identify --json 2>/dev/null | ${pkgs.jq}/bin/jq -r '.surface_ref // empty' 2>/dev/null || true)

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
        cmux list-surfaces "$@"
        ;;

      has-session|-has-session)
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
        command_args=()
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -n) name="$2"; shift 2 ;;
            -t) shift 2 ;;
            --) shift; command_args=("$@"); break ;;
            -*) shift ;;
            *) command_args=("$@"); break ;;
          esac
        done
        cmux new-workspace 2>/dev/null
        if [[ -n "$name" ]]; then
          sleep 0.3
          cmux rename-workspace "$name"
        fi
        if [[ ''${#command_args[@]} -gt 0 ]]; then
          sleep 0.4
          cmux send "''${command_args[*]}"
          cmux send-key enter
        fi
        ;;

      display-message)
        passthrough_args=()
        while [[ $# -gt 0 ]]; do
          case "$1" in
            -p) passthrough_args+=(-p); shift ;;
            *)  passthrough_args+=("$1"); shift ;;
          esac
        done
        cmux display-message "''${passthrough_args[@]}"
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
