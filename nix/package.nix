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
in
  rustPlatform.buildRustPackage {
    inherit (cargoToml.workspace.package) version;
    pname = "academy-backend";

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

    nativeBuildInputs = [installShellFiles];
    postInstall = ''
      installShellCompletion --cmd academy \
        --bash <($out/bin/academy completion bash) \
        --fish <($out/bin/academy completion fish) \
        --zsh <($out/bin/academy completion zsh)
    '';

    meta.mainProgram = "academy";
  }
