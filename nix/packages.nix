{
  fenix,
  installShellFiles,
  lib,
  makeRustPlatform,
  system,
}: let
  cargoToml = fromTOML (builtins.readFile ../Cargo.toml);
  toolchain = fenix.packages.${system}.stable;
  rustPlatform = makeRustPlatform {
    inherit (toolchain) cargo rustc;
  };

  buildRustPackage = {
    pname,
    subdir,
    mainProgram,
  }:
    rustPlatform.buildRustPackage {
      inherit pname;
      inherit (cargoToml.workspace.package) version;

      src = lib.fileset.toSource {
        root = ../.;
        fileset = lib.fileset.unions ([
            ../Cargo.toml
            ../Cargo.lock
          ]
          ++ (lib.pipe ../. [
            builtins.readDir
            builtins.attrNames
            (builtins.filter (lib.hasPrefix "academy"))
            (map (x: ../${x}))
          ]));
      };
      cargoLock.lockFile = ../Cargo.lock;
      doCheck = false;

      buildAndTestSubdir = subdir;

      nativeBuildInputs = [installShellFiles];
      postInstall = ''
        installShellCompletion --cmd ${mainProgram} \
          --bash <($out/bin/${mainProgram} completion bash) \
          --fish <($out/bin/${mainProgram} completion fish) \
          --zsh <($out/bin/${mainProgram} completion zsh)
      '';

      meta.mainProgram = mainProgram;
    };
in {
  default = buildRustPackage {
    pname = "academy-backend";
    subdir = "academy";
    mainProgram = "academy";
  };

  testing = buildRustPackage {
    pname = "academy-testing";
    subdir = "academy_testing";
    mainProgram = "academy-testing";
  };
}
