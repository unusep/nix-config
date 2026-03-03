{ ... }: {
  home.file.".config/ghostty/config" = {
    force = true;
    text = ''
      font-family = JetBrains Mono
      font-size = 13
    '';
  };
}
