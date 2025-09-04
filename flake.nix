{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        shallow-blue = pkgs.stdenv.mkDerivation {
          pname = "ShallowBlue";
          version = "2.0.0";

          src = pkgs.fetchFromGitHub {
            owner = "GunshipPenguin";
            repo = "shallow-blue";
            rev = "a04fbd9861770c897eb566d83b0d2e3b17aa9fc0";
            hash = "sha256-PgAwByWzDe5Blll62aLhiodvcpKKWwoodDiZc+HbD3U=";
          };

          nativeBuildInputs = with pkgs; [
            gcc
          ];

          postPatch = ''
            sed -i '1i\#include <string>' src/option.h
          '';

          buildPhase = ''
            make
          '';

          installPhase = ''
            mkdir -p $out/bin
            mv shallowblue $out/bin
          '';
        };

        cee-chess = pkgs.stdenv.mkDerivation {
          pname = "CeeChess";
          version = "1.4";

          src = pkgs.fetchFromGitHub {
            owner = "bctboi23";
            repo = "CeeChess";
            rev = "3d53576ae009418eea2da61b54c963d670fb83f1";
            hash = "sha256-twPHChinKFew4Ugsm9oDo7d73P1RyFknPyINvll1rk4=";
          };

          nativeBuildInputs = with pkgs; [
            gcc
          ];

          buildPhase = ''
            make
          '';

          installPhase = ''
            mkdir -p $out/bin
            mv bin/CeeChess-v1.4-linux $out/bin
          '';
        };

        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        });
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rust
            libiconv
            cargo-leptos
            wasm-bindgen-cli
            cargo-generate
            tailwindcss
            lldb_20
            libllvm
            cargo-flamegraph

            stockfish

            shallow-blue
            cee-chess
          ];
          # ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.apple-sdk_15.frameworks; [
          #     CoreServices
          #     SystemConfiguration
          # ]);
          RUST_LOG = 1;
        };
      }
    );
}
