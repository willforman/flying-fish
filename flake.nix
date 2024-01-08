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

    nixpkgs-lldb-fix.url = "github:Itaros/nixpkgs/sideport-lldb-1x";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, nixpkgs-lldb-fix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        lldb-fix-overlay = final: prev: {
          inherit (nixpkgs-lldb-fix.legacyPackages.${prev.system})
            lldb_17;
        };

        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) lldb-fix-overlay ];
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
