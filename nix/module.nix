self: {
  config,
  lib,
  pkgs,
  ...
}: let
  package = self.packages.${pkgs.system}.default;
  settingsFormat = pkgs.formats.toml {};
in {
  options.services.academy.backend = {
    enable = lib.mkEnableOption "Bootstrap Academy Backend";

    localDatabase = lib.mkOption {
      type = lib.types.bool;
      default = true;
    };

    localCache = lib.mkOption {
      type = lib.types.bool;
      default = true;
    };

    logLevel = lib.mkOption {
      type = lib.types.str;
      default = "info";
    };

    extraConfigFiles = lib.mkOption {
      type = lib.types.listOf lib.types.path;
      default = [];
    };

    settings = lib.mkOption {
      inherit (settingsFormat) type;
      default = {};
    };

    tasks = lib.genAttrs ["prune-database"] (task: {
      schedule = lib.mkOption {
        type = lib.types.either lib.types.str (lib.types.listOf lib.types.str);
        default = [];
      };
    });
  };

  config = let
    cfg = config.services.academy.backend;

    updateDefaultConfig = settings:
      settings
      // {
        http = builtins.removeAttrs settings.http ["host" "port"];
        database = builtins.removeAttrs settings.database ["url"];
        cache = builtins.removeAttrs settings.cache ["url"];
        email = builtins.removeAttrs settings.email ["smtp_url" "from"];
        jwt = builtins.removeAttrs settings.jwt ["secret"];
        session =
          settings.session
          // {
            access_token_ttl = "5m";
          };
      };

    settings = settingsFormat.generate "config.toml" cfg.settings;
    defaultConfig = lib.pipe ../config.toml [
      builtins.readFile
      builtins.fromTOML
      updateDefaultConfig
      (settingsFormat.generate "config.default.toml")
    ];
    configArgs = lib.pipe ([defaultConfig settings] ++ cfg.extraConfigFiles) [
      (map (path: "--config=${lib.escapeShellArg path}"))
      (builtins.concatStringsSep " ")
    ];

    wrapper = pkgs.stdenvNoCC.mkDerivation {
      inherit (package) pname version;
      src = package;
      nativeBuildInputs = [pkgs.makeWrapper];
      installPhase = ''
        cp -r . $out
        wrapProgram $out/bin/academy --run "[[ \$USER = academy ]] || exec ${pkgs.sudo}/bin/sudo -u academy \"\$0\" \"\$@\"" --add-flags "${configArgs}"
      '';
    };
  in
    lib.mkIf cfg.enable {
      systemd.services = let
        dependencies = ["network-online.target"] ++ (lib.optional cfg.localDatabase "postgresql.service") ++ (lib.optional cfg.localCache "redis-academy.service");
        baseConfig = {
          wants = dependencies;
          after = dependencies;

          serviceConfig = {
            User = "academy";
            Group = "academy";
          };

          environment.RUST_LOG = cfg.logLevel;
        };
      in
        {
          academy-backend =
            baseConfig
            // {
              wantedBy = ["multi-user.target"];
              script = ''
                ${package}/bin/academy ${configArgs} serve
              '';
            };
        }
        // (
          lib.mapAttrs' (task: {schedule}: {
            name = "academy-task-${task}";
            value =
              baseConfig
              // {
                startAt = schedule;
                script = ''
                  ${package}/bin/academy ${configArgs} task ${task}
                '';
              };
          })
          cfg.tasks
        );

      services.postgresql = lib.mkIf cfg.localDatabase {
        enable = true;
        ensureUsers = [
          {
            name = "academy";
            ensureDBOwnership = true;
          }
        ];
        ensureDatabases = ["academy"];
      };

      services.redis = lib.mkIf cfg.localCache {
        package = pkgs.valkey;
        servers.academy = {
          enable = true;
          user = "academy";
          save = [];
        };
      };

      users.users.academy = {
        isSystemUser = true;
        group = "academy";
      };
      users.groups.academy = {};

      services.academy.backend.settings.database.url = lib.mkIf cfg.localDatabase "host=/run/postgresql user=academy";
      services.academy.backend.settings.cache.url = lib.mkIf cfg.localCache "redis+unix://${config.services.redis.servers.academy.unixSocket}";

      environment.systemPackages = [wrapper];
    };
}
