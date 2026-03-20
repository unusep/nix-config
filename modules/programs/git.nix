{ ... }: {
  programs.git = {
    enable = true;
    settings.user.name = "Deshun Cai";
    settings.user.email = "unusep@gmail.com";
    ignores = [ "**/.claude/settings.local.json" ];
  };
}
