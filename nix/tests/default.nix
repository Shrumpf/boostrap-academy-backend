{
  callPackage,
  lib,
  linkFarm,
  python3,
  self,
  testers,
  writeShellScriptBin,
  writeTextDir,
}: let
  tests = lib.pipe ./. [
    builtins.readDir
    (lib.filterAttrs (name: type: type == "regular" && isTest name))
    (lib.mapAttrs' (name: _: {
      name = removeSuffix name;
      value = mkTest name;
    }))
  ];

  isTest = name: builtins.any (f: f name) [isPythonTest isNixosTest] && ! builtins.elem name ignored;
  isPythonTest = lib.hasSuffix ".py";
  isNixosTest = lib.hasSuffix ".nix";
  ignored = ["default.nix" "utils.py"];
  removeSuffix = lib.flip lib.pipe [
    (lib.removeSuffix ".py")
    (lib.removeSuffix ".nix")
  ];

  mkTest = name:
    if isPythonTest name
    then mkPythonTest name
    else mkNixosTest name;

  mkPythonTest = name: let
    python = python3.withPackages (p: with p; [httpx pyotp]);
    run-test = writeShellScriptBin "run-test" ''
      export PYTHONPATH=${writeTextDir "utils.py" (builtins.readFile ./utils.py)}
      ${python}/bin/python ${./${name}}
    '';
  in
    testers.runNixOSTest {
      name = "academy-${removeSuffix name}";

      nodes.machine = {pkgs, ...}: {
        imports = [self.nixosModules.default];

        services.academy.backend = {
          enable = true;
          logLevel = "debug,academy=trace";
          settings = {
            http = {
              host = "127.0.0.1";
              port = 8000;
            };
            database.acquire_timeout = "2s";
            cache.acquire_timeout = "2s";
            email = {
              smtp_url = "smtp://127.0.0.1:25";
              from = "test@bootstrap.academy";
            };
            jwt.secret = "changeme";
            health.cache_ttl = "2s";
            contact.email = "contact@academy";
            session.refresh_token_ttl = lib.mkIf (name == "prune-database.py") "10m";
          };
        };

        environment.systemPackages = [run-test];

        services.postfix = {
          enable = true;
          virtual = "/.*/ root";
          virtualMapType = "pcre";
        };
      };

      testScript = ''
        machine.start()
        machine.wait_for_unit("academy-backend.service")
        machine.wait_for_open_port(8000)
        machine.succeed("run-test")
      '';
    };

  mkNixosTest = name: callPackage ./${name} {inherit self;};

  composite = linkFarm "academy-tests-composite" (builtins.mapAttrs (_: toString) tests);
in
  tests // {inherit composite tests;}
