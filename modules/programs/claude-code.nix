{ config, lib, ... }: {
  programs.claude-code.enable = true;

  home.activation.claudeSettings = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    $DRY_RUN_CMD rm -f "$HOME/.claude/settings.json"
    $DRY_RUN_CMD ln -sf "$HOME/nix-config/.claude/settings.json" "$HOME/.claude/settings.json"
  '';
}
