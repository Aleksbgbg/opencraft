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
      devShells.default = pkgs.mkShell rec {
        packages = with pkgs; [
          # Rust
          (rust-bin.selectLatestNightlyWith (toolchain: toolchain.complete))
          cargo-watch # Continuous rebuild
          cargo-edit # Package updates
        ];

        buildInputs = with pkgs; [
          # WINIT_UNIX_BACKEND=x11
          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi

          # WINIT_UNIX_BACKEND=wayland
          # wayland

          # WGPU_BACKEND=gl
          # libGL

          # WGPU_BACKEND=vulkan
          vulkan-loader
        ];
        LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
      };
    });
}
