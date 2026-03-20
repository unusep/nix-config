{ ... }: {
  programs.gh = {
    enable = true;
    gitCredentialHelper.enable = false;
    settings = {
      version = 1;
      git_protocol = "https";
      prompt = "enabled";
      aliases = { co = "pr checkout"; };
    };
  };
}
