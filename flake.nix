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
