{
  description = "RSS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          frontend = pkgs.stdenv.mkDerivation (finalAttrs: {
            pname = "rss-frontend";
            version = "0.0.1";
            src = ./frontend;

            pnpmDeps = pkgs.fetchPnpmDeps {
              inherit (finalAttrs) pname version src;
              fetcherVersion = 3;
              hash = "sha256-Dr+SzjvtPvccky7btZhWd/lOXYNzqSrImTgeIEfQkHQ=";
            };

            nativeBuildInputs = with pkgs; [
              nodejs_24
              pnpm_10
              pnpmConfigHook
            ];

            buildPhase = ''
              runHook preBuild
              pnpm build
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              cp -r dist $out
              runHook postInstall
            '';
          });

          backend = pkgs.rustPlatform.buildRustPackage {
            pname = "rss-backend";
            version = "0.0.1";
            src = ./backend;

            checkFlags = [
              # skip tests that require a postgres instance
              "--skip=db::tests::pg"
            ];
            cargoLock = {
              lockFile = ./backend/Cargo.lock;
            };

            nativeCheckInputs = with pkgs; [
              postgresql_18
            ];

            env = {
              SQLX_OFFLINE = "true";
            };
          };
        in {
          inherit frontend backend;

          default = pkgs.runCommand "rss" {
            nativeBuildInputs = [ pkgs.makeWrapper ];
          } ''
            mkdir -p $out/bin
            makeWrapper ${backend}/bin/backend $out/bin/rss \
              --set FRONTEND_DIR ${frontend}
          '';
        }
      );

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              nodejs_24
              pnpm_10
              rustup
              postgresql_18
            ];

            postgresConf = pkgs.writeText "postgresql.conf" ''
              log_min_messages = warning
              log_min_error_statement = error
              log_min_duration_statement = 100
              log_connections = on
              log_disconnections = on
              log_duration = on
              log_timezone = 'UTC'
              log_statement = 'all'
              log_directory = 'pg_log'
              log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
              logging_collector = on
            '';

            shellHook = ''
              if [ -f .env ]; then
                set -a
                source .env
                set +a
              fi

              free_port() {
                local port="$1"
                shift
                while (echo >/dev/tcp/localhost/"$port") 2>/dev/null || [[ " $* " == *" $port "* ]]; do
                  port=$((port + 1))
                done
                echo "$port"
              }

              export PGDATA="$PWD/.pg"
              export HOST="''${HOST:-0.0.0.0:8000}"

              if pg_ctl -D "$PGDATA" status >/dev/null 2>&1; then
                export PGPORT="$(awk 'NR == 4 { print; exit }' "$PGDATA/postmaster.pid")"
              else
                echo "Setting up ${pkgs.postgresql_18.name}"
                export PGPORT="$(free_port "''${PGPORT:-5555}" 8000 3000)"

                if [ ! -f "$PGDATA/PG_VERSION" ]; then
                  echo "Initializing database..."
                  initdb -D "$PGDATA" -U postgres
                  cat "$postgresConf" >> "$PGDATA/postgresql.conf"
                fi

                pg_ctl -D "$PGDATA" -o "-k $PGDATA" start
              fi

              export PGHOST="$PGDATA"
              export DATABASE_URL="postgres://postgres:postgres@localhost:$PGPORT/postgres"

              echo "Ports: backend 8000, frontend 3000, Postgres $PGPORT"

              alias fin="pg_ctl -D $PGDATA stop && exit"
              alias pg="psql -U postgres -d postgres"
            '';
          };
        }
      );

      nixosModules.default = { pkgs, ... }@args:
        let
          rssPkg = self.packages.${pkgs.system}.default;
        in
        import ./module.nix { inherit rssPkg; } args;
    };
}
