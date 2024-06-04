{
  config,
  lib,
  pkgs,
  ...
}: {
  languages.rust.enable = builtins.getEnv "DEVENV_RUST" != "0";

  packages = with pkgs; [
    just
    cargo-llvm-cov
    lcov
    smtp4dev
    (python3.withPackages (p: with p; [httpx pyotp]))
  ];

  services.postgres = {
    enable = true;
    package = pkgs.postgresql_16;
    listen_addresses = "127.0.0.1";
    initialScript = ''
      CREATE USER academy SUPERUSER;
      CREATE DATABASE academy OWNER academy;
    '';
  };

  services.redis = {
    enable = true;
    package = pkgs.valkey;
  };

  processes.smtp4dev.exec = ''
    ${pkgs.smtp4dev}/bin/smtp4dev --smtpport=2525 --imapport=1143
  '';

  env = {
    RUST_LOG = "debug,backend=trace";

    PGDATABASE = "academy";

    SMTP4DEV_URL = "http://127.0.0.1:5000";

    PYTHONPATH = "${config.env.DEVENV_ROOT}/nix/tests";
  };

  process.implementation = "hivemind";

  scripts = {
    devenv-reset.exec = ''
      rm -rf ${lib.escapeShellArg config.devenv.state}
    '';
  };
}
