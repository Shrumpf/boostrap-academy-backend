{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    crate2nix.url = "github:nix-community/crate2nix";
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    crate2nix,
    devenv,
    ...
  } @ inputs: let
    inherit (nixpkgs) lib;

    eachDefaultSystem = lib.genAttrs [
      "x86_64-linux"
      "aarch64-linux"
    ];

    importNixpkgs = system: import nixpkgs {inherit system;};
  in {
    packages = eachDefaultSystem (system: let
      pkgs = importNixpkgs system;
    in
      (pkgs.callPackages ./nix/packages.nix {inherit crate2nix fenix self;})
      // {
        tests = pkgs.callPackages ./nix/tests {inherit self;};
        devShell = self.devShells.${system}.default;
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

    formatter = eachDefaultSystem (system: (importNixpkgs system).alejandra);
  };

  nixConfig = {
    extra-substituters = "https://academy-backend.cachix.org";
    extra-trusted-public-keys = "academy-backend.cachix.org-1:MxmjN6hjaiGdi42M6evdALWj5hHOyUAQTEgKvm+J0Ow=";
  };
}
