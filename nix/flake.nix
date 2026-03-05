{
  description = "Deshun's MacOS Configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nix-darwin.url = "github:LnL7/nix-darwin";
    nix-darwin.inputs.nixpkgs.follows = "nixpkgs";
    home-manager.url = "github:nix-community/home-manager";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";
    claude-code.url = "github:sadjow/claude-code-nix";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ self, nixpkgs, nix-darwin, home-manager, claude-code, rust-overlay, ... }:
  let
    user = "deshuncai";
    hostname = "Deshuns-MacBook-Pro";
    system = "aarch64-darwin";
  in {
    darwinConfigurations.${hostname} = nix-darwin.lib.darwinSystem {
      inherit system;
      specialArgs = { inherit user hostname self; };
      modules = [
        { nixpkgs.overlays = [ claude-code.overlays.default rust-overlay.overlays.default ]; }
        ./modules/system.nix

        home-manager.darwinModules.home-manager
        {
          home-manager.useGlobalPkgs = true;
          home-manager.useUserPackages = true;
          home-manager.extraSpecialArgs = { inherit user; };
          home-manager.users.${user} = { ... }: {
            imports = [
              ./modules/home-manager.nix
              ./modules/programs/shell.nix
              ./modules/programs/editor.nix
              ./modules/programs/claude-code.nix
              ./modules/programs/claude-tmux-bridge.nix
              ./modules/programs/aerospace.nix
              ./modules/programs/cmux.nix
              ./modules/programs/ghostty.nix
              ./modules/programs/karabiner.nix

              ./modules/programs/hrm.nix
              ./modules/programs/git.nix
              ./modules/programs/gh.nix

            ];
          };
        }
      ];
    };
  };
}
