{
  crate2nix,
  fenix,
  callPackage,
  installShellFiles,
  lib,
  pkgs,
  system,
}: let
  toolchain = fenix.packages.${system}.stable;

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

  generateShellCompletionOverride = bin: attrs: {
    pname = bin;
    name = "${bin}-${attrs.version}";
    CARGO_BIN_NAME = bin;
    nativeBuildInputs = attrs.postInstall or [] ++ [installShellFiles];
    postInstall = ''
      ${attrs.postInstall or ""}
      installShellCompletion --cmd ${bin} \
        --bash <($out/bin/${bin} completion bash) \
        --fish <($out/bin/${bin} completion fish) \
        --zsh <($out/bin/${bin} completion zsh)
    '';
    meta.mainProgram = bin;
  };

  crateOverrides = {
    academy = generateShellCompletionOverride "academy";
    academy_testing = generateShellCompletionOverride "academy-testing";
  };

  generated = crate2nix.tools.${system}.generatedCargoNix {
    name = "academy";
    inherit src;
  };

  cargoNix = callPackage generated {
    pkgs = pkgs.extend (final: prev: {
      inherit (toolchain) cargo;
      # workaround for https://github.com/NixOS/nixpkgs/blob/d80a3129b239f8ffb9015473c59b09ac585b378b/pkgs/build-support/rust/build-rust-crate/default.nix#L19-L23
      rustc = toolchain.rustc // {unwrapped = {configureFlags = ["--target="];};};
    });
    defaultCrateOverrides = pkgs.defaultCrateOverrides // crateOverrides;
  };
in {
  default = cargoNix.workspaceMembers.academy.build;
  testing = cargoNix.workspaceMembers.academy_testing.build;
}
