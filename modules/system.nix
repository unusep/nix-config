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
    aerospace
    vscode
  ];

  system.defaults = {
    controlcenter.Bluetooth = true;
    dock = {
      autohide = true;
      show-recents = false;
      orientation = "left";
      persistent-apps = [
        "/Applications/Arc.app"
        "/Applications/cmux.app"
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
      cleanup = "uninstall";
      upgrade = true;
    };
    taps = [
      "manaflow-ai/cmux"
      "wontaeyang/hrm"
    ];
    casks = [
      "cmux"
      "claude"
      "crossover"
      "docker"
      "arc"
      "hrm"
      "karabiner-elements"
      "raycast"
      "zoom"
      "obs"
      "steam"
      "discord"
    ];
  };

  system.configurationRevision = self.rev or self.dirtyRev or null;
  system.stateVersion = 5;
}
