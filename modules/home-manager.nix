{ pkgs, user, ... }: {
  home.homeDirectory = "/Users/${user}";
  home.stateVersion = "24.05";

  launchd.agents.nix-flake-update = {
    enable = true;
    config = {
      Label = "org.nix.flake-update";
      EnvironmentVariables = {
        PATH = "/run/current-system/sw/bin:/nix/var/nix/profiles/default/bin:/usr/bin:/bin";
      };
      ProgramArguments = [
        "/bin/sh" "-c"
        ''
          if ! curl -s --max-time 5 https://cache.nixos.org > /dev/null 2>&1; then
            echo "No network, skipping update"
            exit 0
          fi

          cd /Users/${user}/nix-config
          nix flake update 2>&1
        ''
      ];
      StartCalendarInterval = [{ Hour = 4; Minute = 0; }];
      StandardOutPath = "/tmp/nix-flake-update.log";
      StandardErrorPath = "/tmp/nix-flake-update.log";
    };
  };

  home.packages = with pkgs; [
    ripgrep
    fd
    jq
    eza
    bat
    fzf
    zoxide
    direnv
    lazygit

    nodejs
    bun
    python313

    (rust-bin.nightly.latest.default.override {
      extensions = [ "rust-src" "rust-analyzer" ];
    })

    cmake
    rust-script

    statix
    deadnix
    nixfmt-rfc-style
  ];
}
