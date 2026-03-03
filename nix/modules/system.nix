{ pkgs, user, hostname, self, ... }: {
  system.primaryUser = user;

  nix.settings.experimental-features = "nix-command flakes";
  nixpkgs.config.allowUnfree = true;

  users.users.${user} = {
    name = user;
    home = "/Users/${user}";
  };

  environment.systemPackages = with pkgs; [
    vim
    git
    curl
    zellij
    aerospace
    ghostty-bin
    vscode
  ];

  system.defaults = {
    dock = {
      autohide = true;
      show-recents = false;
      orientation = "left";
      persistent-apps = [
        "/Applications/Google Chrome.app"
        "/Applications/Nix Apps/Ghostty.app"
        "/System/Applications/Messages.app"
      ];
    };
    finder = {
      AppleShowAllExtensions = true;
      _FXShowPosixPathInTitle = true;
    };
    NSGlobalDomain = {
      AppleInterfaceStyle = "Dark";
      KeyRepeat = 2;
      "com.apple.trackpad.scaling" = 3.0;
    };
  };

  security.pam.services.sudo_local.touchIdAuth = true;

  environment.etc."sudoers.d/darwin-rebuild".text = ''
    ${user} ALL=(root) NOPASSWD: /run/current-system/sw/bin/darwin-rebuild
  '';

  homebrew = {
    enable = true;
    onActivation = {
      cleanup = "none";
      upgrade = false;
    };
    taps = [
      "wontaeyang/hrm"
    ];
    casks = [
      "claude"
      "crossover"
      "docker"
      "google-chrome"
      "hrm"
      "karabiner-elements"
      "raycast"
      "zoom"
      "obs"
    ];
  };

  system.configurationRevision = self.rev or self.dirtyRev or null;
  system.stateVersion = 5;
}
