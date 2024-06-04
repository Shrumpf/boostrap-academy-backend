{
  lib,
  testers,
  self,
}:
testers.runNixOSTest {
  name = "academy-basics";

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
        jwt.secret = "changeme";
        email = {
          smtp_url = "smtp://127.0.0.1:25";
          from = "test@bootstrap.academy";
        };
      };
    };

    services.postfix.enable = true;
  };

  testScript = ''
    import json
    import time

    machine.start()
    machine.succeed("academy --version")

    machine.wait_for_unit("academy-backend.service")
    machine.wait_for_open_port(8000)

    machine.wait_for_unit("postfix.service")
    machine.wait_for_open_port(25)

    machine.succeed("curl -s http://127.0.0.1:8000/")

    assert machine.succeed("academy migrate list").strip() == ${lib.pipe ../../academy_persistence/postgres/migrations [
      builtins.readDir
      builtins.attrNames
      (map (lib.removeSuffix ".up.sql"))
      (map (lib.removeSuffix ".down.sql"))
      lib.unique
      (lib.sortOn lib.id)
      (map (m: "[applied] ${m}"))
      (builtins.concatStringsSep "\n")
      builtins.toJSON
    ]}, "some migrations are missing or have not been applied"

    machine.succeed("academy email test root@localhost")
    time.sleep(1)
    machine.succeed("grep 'Email deliverability seems to be working!' /var/mail/root/new/*")

    status = json.loads(machine.succeed("curl -s http://127.0.0.1:8000/health"))
    assert status == {
      "http": True,
      "database": True,
      "cache": True,
      "email": True,
    }

    machine.succeed("academy admin user create --admin --verified admin admin@example.com supersecureadminpassword")
    login = json.loads(machine.succeed("curl -s http://127.0.0.1:8000/auth/sessions -X POST -H 'Content-Type: application/json' -d '{\"name_or_email\": \"admin\", \"password\": \"supersecureadminpassword\"}'"))
    assert login["user"]["admin"]
    assert login["user"]["email_verified"]

    login = json.loads(machine.succeed("curl -s http://127.0.0.1:8000/auth/users -X POST -H 'Content-Type: application/json' -d '{\"name\": \"user\", \"display_name\": \"new user 42\", \"email\": \"user@example.com\", \"password\": \"password\"}'"))
    assert login["user"]["name"] == "user"
    assert login["user"]["display_name"] == "new user 42"

    user = json.loads(machine.succeed(f"curl -s http://127.0.0.1:8000/auth/users/self -H 'Authorization: Bearer {login["access_token"]}'"))
    assert user == login["user"]
  '';
}
