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
in {
  default = rustPlatform.buildRustPackage {
    pname = "academy-backend";
    inherit version src cargoLock;
    doCheck = false;

    buildAndTestSubdir = "academy";

    nativeBuildInputs = [installShellFiles];
    postInstall = ''
      installShellCompletion --cmd academy \
        --bash <($out/bin/academy completion bash) \
        --fish <($out/bin/academy completion fish) \
        --zsh <($out/bin/academy completion zsh)
    '';

    meta.mainProgram = "academy";
  };

  testing = rustPlatform.buildRustPackage {
    pname = "academy-testing";
    inherit version src cargoLock;
    doCheck = false;

    buildAndTestSubdir = "academy_testing";

    postInstall = ''
      mv $out/bin/{academy_testing,academy-testing}
    '';

    meta.mainProgram = "academy-testing";
  };
}
