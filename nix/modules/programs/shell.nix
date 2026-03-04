{ ... }: {
  programs.zsh = {
    enable = true;
    enableCompletion = true;
    oh-my-zsh = {
      enable = true;
      plugins = [ "git" "sudo" "vi-mode" ];
      theme = "robbyrussell";
    };
    shellAliases = {
      ls = "eza --icons";
      ll = "eza -l --icons --git -a";
      v = "nvim";
      cc = "claude";
      drs = "sudo darwin-rebuild switch --flake ~/.config/nix";
    };
    initContent = ''
      bindkey -v

      eval "$(/opt/homebrew/bin/brew shellenv zsh)"

      eval "$(zoxide init zsh)"

      eval "$(direnv hook zsh)"
    '';
  };
}
