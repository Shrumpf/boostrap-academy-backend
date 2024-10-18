{
  crate2nix,
  fenix,
  callPackage,
  installShellFiles,
  lib,
  pkgs,
  system,
  self,
}: let
  toolchain = fenix.packages.${system}.stable;

  version = let
    year = builtins.substring 0 4 self.sourceInfo.lastModifiedDate;
    month = builtins.substring 4 2 self.sourceInfo.lastModifiedDate;
    day = builtins.substring 6 2 self.sourceInfo.lastModifiedDate;
    rev = self.sourceInfo.shortRev or self.sourceInfo.dirtyShortRev;
  in "${year}.${month}.${day}+${rev}";

  crateDirs = lib.pipe ../. [
    builtins.readDir
    builtins.attrNames
    (builtins.filter (lib.hasPrefix "academy"))
    (map (x: ../${x}))
  ];
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions ([../Cargo.toml ../Cargo.lock] ++ crateDirs);
  };

  workspaceMembers = lib.pipe crateDirs [
    (map lib.filesystem.listFilesRecursive)
    lib.flatten
    (builtins.filter (x: baseNameOf x == "Cargo.toml"))
    (map (lib.flip lib.pipe [
      builtins.readFile
      fromTOML
      (x: x.package.name)
    ]))
  ];

  cargoToml = fromTOML (builtins.readFile ../Cargo.toml);

  mergeOverrides = a: b: attrs: (a attrs) // (b (attrs // (a attrs)));
  mergeOverrideSets = a: b: a // b // (builtins.mapAttrs (k: _: mergeOverrides a.${k} b.${k}) (lib.intersectAttrs a b));

  defaultOverrides = lib.genAttrs workspaceMembers (crate: attrs: {
    inherit version;
    preBuild = ''
      ${attrs.preBuild or ""}
      export CARGO_PKG_HOMEPAGE=${lib.escapeShellArg cargoToml.workspace.package.homepage}
      export CARGO_PKG_REPOSITORY=${lib.escapeShellArg cargoToml.workspace.package.repository}
    '';
  });

  binOverride = bin: attrs: {
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

  crateOverrides = mergeOverrideSets defaultOverrides {
    academy = binOverride "academy";
    academy_testing = binOverride "academy-testing";
    academy_config = attrs: {
      patchPhase = ''
        sed -i 's|env!("CARGO_MANIFEST_DIR"), "/../config.toml"|"${../config.toml}"|' src/lib.rs
        ${attrs.patchPhase or ""}
      '';
    };
  };

  generated = crate2nix.tools.${system}.generatedCargoNix {
    name = "academy";
    inherit src;
  };

  cargoNix = callPackage generated {
    pkgs = pkgs.extend (final: prev: {
      inherit (toolchain) cargo;
      # workaround for https://github.com/NixOS/nixpkgs/blob/d80a3129b239f8ffb9015473c59b09ac585b378b/pkgs/build-support/rust/build-rust-crate/default.nix#L19-L23
      rustc = toolchain.rustc // {unwrapped.configureFlags = ["--target="];};
    });
    defaultCrateOverrides = mergeOverrideSets pkgs.defaultCrateOverrides crateOverrides;
  };
in {
  default = cargoNix.workspaceMembers.academy.build;
  testing = cargoNix.workspaceMembers.academy_testing.build;
}
