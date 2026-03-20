{ config, ... }: {
  programs.neovim = {
    enable = true;
    defaultEditor = true;
    viAlias = true;
    vimAlias = true;
  };

  xdg.configFile."nvim".source = config.lib.file.mkOutOfStoreSymlink "/Users/deshuncai/nix-config/nix/nvim";
}
