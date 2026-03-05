{ pkgs, ... }:
let
  tmuxBridge = pkgs.stdenv.mkDerivation {
    name = "claude-tmux-bridge";
    src = ../../pkgs/claude-tmux-bridge.rs;
    nativeBuildInputs = [ pkgs.rustc ];
    dontUnpack = true;
    buildPhase = "rustc -O -o tmux $src";
    installPhase = "mkdir -p $out/bin && cp tmux $out/bin/";
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
