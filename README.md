# Bootstrap Academy Backend
The official backend of [Bootstrap Academy](https://bootstrap.academy/).

If you would like to submit a bug report or feature request, or are looking for general information about the project or the publicly available instances, please refer to the [Bootstrap-Academy repository](https://github.com/Bootstrap-Academy/Bootstrap-Academy).

## Development Setup
1. Install [Nix](https://nixos.org/) with [flakes](https://wiki.nixos.org/wiki/Flakes) enabled. If you are on NixOS, just ensure that flakes are enabled. If you are using a different Linux distribution or OS, we recommend using the [Determinate Nix Installer](https://github.com/DeterminateSystems/nix-installer?tab=readme-ov-file#the-determinate-nix-installer). Also make sure to add your user to the [`trusted-users`](https://nix.dev/manual/nix/2.19/command-ref/conf-file#conf-trusted-users) option in `/etc/nix/nix.conf` (run `sudo systemctl restart nix-daemon` after changing this file).
2. [Install direnv](https://github.com/direnv/direnv/blob/master/docs/installation.md) (optional, but recommended). To install it via Nix, run the command `nix profile install nixpkgs#direnv`. Don't forget to install the shell hook (e.g. for bash run `echo 'eval "$(direnv hook bash)"' >> ~/.bashrc`).
3. Clone this repository and `cd` into it.
4. If you installed direnv, run `direnv allow` to automatically load the development environment when entering this repository. Otherwise you need to run `nix develop --no-pure-eval` manually each time to enter the development shell.
5. Run `devenv up` to start a local Postgres database, Valkey cache, SMTP server and some other services needed for testing.
6. In a new terminal, you can now run `cargo run -- serve`. By default, the Swagger UI documentation is now available on http://127.0.0.1:8000/docs

### VSCode Setup
For development in [VSCode](https://code.visualstudio.com/)/[VSCodium](https://vscodium.com/), simply open this repository, install the recommended extensions and confirm restarting the extensions after direnv successfully initialized.

### Useful Commands
- `psql`: Connect to the local Postgres database
- `valkey-cli`: Connect to the local Valkey cache
- `cargo run -- --help`: List all commands provided by the backend CLI
- `just`: List all recipes provided by the `justfile`

### Services
- The web interface of [smtp4dev](https://github.com/rnwood/smtp4dev) is available on http://localhost:5000/
- Various services for mocking external APIs are running on ports starting at 8001. See the logs of the `testing-*` services in `devenv up` for details.

## Tests
This repository contains three different kinds of tests: Unit tests, integration tests and system tests.

Unit tests test only one unit (i.e. function in most cases) at a time and do not rely on any external systems.
To run the unit tests, use the command `just test-unit`.

Integration tests are used to test the integration with other systems such as external APIs or databases.
To run the integration tests, use the corresponding just recipes (e.g. to run the Postgres tests execute `just test-postgres`).

To run all unit and integration tests, use the command `just test`.
It is also possible to generate coverage reports by replacing `test` with `coverage` in any of the previous commands.

System tests are used to test a production build of the backend in an environment that is as realistic as possible (with some exceptions).
For that we use the [NixOS test framework](https://nixos.org/manual/nixos/stable/#sec-nixos-tests) which spawns virtual machines containing a minimal NixOS system running the backend and then runs test scripts written in Python against these VMs.
To run the system tests, use the command `nix build -L .#tests.composite`.
You can also run individual system tests e.g. using `nix build -L .#tests.user`.
