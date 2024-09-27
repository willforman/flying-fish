{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
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
            lldb_17
            libllvm
            cargo-flamegraph
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
              CoreServices
              SystemConfiguration
          ]);
          RUST_LOG = 1;
        };
      }
    );
}
