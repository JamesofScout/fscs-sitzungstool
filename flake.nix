{
  description = "FSCS Dioxus frontend development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rustfmt" "clippy" "rust-src" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        frontendStart = pkgs.writeShellScriptBin "frontend-start" ''
          export PATH="${pkgs.lib.makeBinPath [
            rustToolchain
            pkgs.trunk
            pkgs.wasm-bindgen-cli
            pkgs.binaryen
            pkgs.gcc
          ]}:$PATH"
          site_url="''${1:-http://localhost:8080}"
          export FSCS_SITE_URL="$site_url"
          echo "Starting frontend for $FSCS_SITE_URL..."
          exec trunk serve --port 8040 --address 0.0.0.0
        '';
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            trunk
            rustup
            wasm-bindgen-cli
            binaryen
            pkg-config
            openssl
            curl
          ];

          shellHook = ''
            export PATH="${rustToolchain}/bin:$PATH"
            if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
              rustup target add wasm32-unknown-unknown
            fi
            echo "FSCS frontend shell ready."
            echo "Set Backend via: \`export FSCS_SITE_URL=<url>\`"
            echo "Run: trunk serve --open"
          '';
        };

        apps.frontend = {
          type = "app";
          program = "${frontendStart}/bin/frontend-start";
        };

        packages.docker = pkgs.dockerTools.buildImage {
          name = "my-app";
          tag = "latest";
          contents = [ frontendStart ];
          config = {
            Cmd = [ "${frontendStart}/bin/frontend-start" ];
            ExposedPorts = { "8080/tcp" = {}; };
          };
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "sitzungstool";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [ pkg-config gcc ];
          buildInputs = with pkgs; [ openssl ];
        };
      }
    );
}
