{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs-smtp4dev.url = "github:Rucadi/nixpkgs/smtp4dev";
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    devenv,
    nixpkgs-smtp4dev,
    ...
  } @ inputs: let
    inherit (nixpkgs) lib;

    defaultSystems = [
      "x86_64-linux"
      "aarch64-linux"
    ];
    eachDefaultSystem = lib.genAttrs defaultSystems;

    overlays = [
      (final: prev: {inherit (nixpkgs-smtp4dev.legacyPackages.${final.system}) smtp4dev;})
    ];
    importNixpkgs = system: import nixpkgs {inherit system overlays;};
  in {
    packages = eachDefaultSystem (system: let
      pkgs = importNixpkgs system;
    in
      (pkgs.callPackages ./nix/packages.nix {inherit fenix;})
      // {
        tests = pkgs.callPackages ./nix/tests {inherit self;};
        devenv-up = self.devShells.${system}.default.config.procfileScript;
      });

    nixosModules = {
      default = import ./nix/module.nix self;
    };

    devShells = eachDefaultSystem (system: {
      default = devenv.lib.mkShell {
        inputs = inputs // {inherit (self.packages.${system}) testing;};
        pkgs = importNixpkgs system;
        modules = [./nix/dev.nix];
      };
    });
  };
}
