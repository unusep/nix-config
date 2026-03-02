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

  homebrew = {
    enable = true;
    onActivation = {
      cleanup = "none";
      upgrade = false;
    };
    casks = [
      "crossover"
      "docker"
      "google-chrome"
      "karabiner-elements"
      "raycast"
      "zoom"
      "obs"
    ];
  };

  launchd.daemons.nix-auto-update = {
    serviceConfig = {
      Label = "org.nix.auto-update";
      EnvironmentVariables = {
        PATH = "/run/current-system/sw/bin:/nix/var/nix/profiles/default/bin:/usr/bin:/bin";
      };
      ProgramArguments = [
        "/bin/sh" "-c"
        ''
          FLAKE_DIR="/Users/${user}/.config/nix"

          if ! curl -s --max-time 5 https://cache.nixos.org > /dev/null 2>&1; then
            echo "No network, skipping update"
            exit 0
          fi

          cd "$FLAKE_DIR"
          nix flake update 2>&1
          darwin-rebuild switch --flake "$FLAKE_DIR" 2>&1
        ''
      ];
      StartCalendarInterval = [{ Hour = 4; Minute = 0; }];
      StandardOutPath = "/tmp/nix-auto-update.log";
      StandardErrorPath = "/tmp/nix-auto-update.log";
    };
  };

  system.configurationRevision = self.rev or self.dirtyRev or null;
  system.stateVersion = 5;
}
