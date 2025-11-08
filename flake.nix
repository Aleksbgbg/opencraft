{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
        ];
      };
      lib = nixpkgs.lib;
    in {
      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          # Rust
          (rust-bin.selectLatestNightlyWith (toolchain: toolchain.complete))
          cargo-watch # Continuous rebuild
          cargo-edit # Package updates
        ];
      };
    });
}
