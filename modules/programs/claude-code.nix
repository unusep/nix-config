{ ... }: {
  programs.claude-code = {
    enable = true;
    settings = {
      permissions = {
        allow = [
          "WebSearch"
          "Read(~/.cargo/**)"
          "Read(~/.rustup/**)"
          "Read(**/node_modules/**)"
        ];
        defaultMode = "default";
      };
      enabledPlugins = {
        "typescript-lsp@claude-plugins-official" = true;
        "lua-lsp@claude-plugins-official" = true;
        "pyright-lsp@claude-plugins-official" = true;
        "cmux@local" = true;
      };
      promptSuggestionEnabled = false;
      voiceEnabled = true;
      attribution = {
        commit = "";
        pr = "";
      };
    };
  };
}
