{ pkgs, ... }:
let
  tmuxBridge = pkgs.rustPlatform.buildRustPackage {
    pname = "claude-tmux-bridge";
    version = "0.1.0";
    src = ../../pkgs/claude-tmux-bridge;
    cargoLock.lockFile = ../../pkgs/claude-tmux-bridge/Cargo.lock;
  };

  cct = pkgs.writeShellScriptBin "cct" ''
    export TMUX=cmux-bridge
    export PATH="${tmuxBridge}/bin:$PATH"
    export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1
    exec claude "$@"
  '';
in
{
  home.packages = [ cct tmuxBridge ];

  programs.zsh.shellAliases.claude = "cct";

  programs.claude-code.settings.env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1";
}
