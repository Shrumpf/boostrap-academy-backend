{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    devenv,
    ...
  } @ inputs: let
    inherit (nixpkgs) lib;

    eachDefaultSystem = lib.genAttrs [
      "x86_64-linux"
      "aarch64-linux" # untested
    ];

    importNixpkgs = system: import nixpkgs {inherit system;};

    mkDevShell = {
      system,
      root ? null,
    }:
      devenv.lib.mkShell {
        inputs = inputs // {inherit (self.packages.${system}) testing generate update-swagger-ui;};
        pkgs = importNixpkgs system;
        modules = [
          ./nix/dev.nix
          {devenv.root = lib.mkIf (root != null) root;}
        ];
      };
  in {
    packages = eachDefaultSystem (system: let
      pkgs = importNixpkgs system;
    in
      (pkgs.callPackages ./nix/packages.nix {inherit fenix self;})
      // {
        tests = pkgs.callPackages ./nix/tests {inherit self;};
        devenv-up = self.devShells.${system}.default.config.procfileScript;
        devShell = mkDevShell {
          inherit system;
          root = "/fake-root";
        };
      });

    nixosModules = {
      default = import ./nix/module.nix self;
    };

    devShells = eachDefaultSystem (system: {
      default = mkDevShell {inherit system;};
    });

    formatter = eachDefaultSystem (system: (importNixpkgs system).alejandra);

    checks = builtins.mapAttrs (system: packages: builtins.removeAttrs packages ["tests" "devenv-up"]) self.packages;
  };

  nixConfig = {
    extra-substituters = "https://academy-backend.cachix.org";
    extra-trusted-public-keys = "academy-backend.cachix.org-1:MxmjN6hjaiGdi42M6evdALWj5hHOyUAQTEgKvm+J0Ow=";
  };
}
