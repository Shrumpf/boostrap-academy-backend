{
  testers,
  defaultModule,
}:
testers.runNixOSTest {
  name = "academy-config";

  nodes.default = {
    imports = [defaultModule];
  };
  nodes.captcha = {
    imports = [defaultModule];
    services.academy.backend.settings = {
      recaptcha = {
        sitekey = "test-sitekey";
        secret = "test-secret";
        min_score = 0.5;
      };
    };
  };

  testScript = ''
    import json

    start_all()

    default.wait_for_unit("academy-backend.service")
    default.wait_for_open_port(8000)
    assert json.loads(default.succeed("curl -s http://127.0.0.1:8000/auth/recaptcha")) is None

    captcha.wait_for_unit("academy-backend.service")
    captcha.wait_for_open_port(8000)
    assert json.loads(captcha.succeed("curl -s http://127.0.0.1:8000/auth/recaptcha")) == "test-sitekey"
  '';
}
