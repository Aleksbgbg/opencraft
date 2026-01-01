{
  inputs = {
    self.submodules = true;

    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }: let
    packageName = "opencraft";
  in (
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
        ];
      };
      lib = nixpkgs.lib;

      libraryName = "l${packageName}";
      buildTarget = "wasm32-unknown-unknown";

      rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
        toolchain.minimal.override {
          targets = [buildTarget];
        });

      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };

      wasm-bindgen-cli = pkgs.buildWasmBindgenCli rec {
        src = pkgs.fetchCrate {
          pname = "wasm-bindgen-cli";
          version = "0.2.106";
          hash = "sha256-M6WuGl7EruNopHZbqBpucu4RWz44/MSdv6f0zkYw+44=";
        };

        cargoDeps = rustPlatform.fetchCargoVendor {
          inherit src;
          inherit (src) pname version;
          hash = "sha256-ElDatyOwdKwHg3bNH/1pcxKI7LXkhsotlDPQjiLHBwA=";
        };
      };
    in {
      devShells.default = pkgs.mkShell rec {
        packages = with pkgs; [
          # Rust
          (rust-bin.selectLatestNightlyWith (toolchain:
            toolchain.complete.override {
              targets = [buildTarget];
            }))
          cargo-watch # Continuous rebuild
          cargo-edit # Package updates

          # WASM
          wasm-pack
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

      packages.default = rustPlatform.buildRustPackage {
        name = packageName;
        version = "0.0.0";

        src = lib.cleanSource ./.;
        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = with pkgs; [
          wasm-bindgen-cli
          binaryen
        ];

        buildPhase = ''
          runHook preBuild

          cargo build --lib --release --target ${buildTarget}
          wasm-bindgen target/${buildTarget}/release/${libraryName}.wasm --target web --out-dir dist
          wasm-opt dist/${libraryName}_bg.wasm -O -o dist/${libraryName}_bg.wasm.opt

          runHook postBuild
        '';
        installPhase = ''
          runHook preInstall

          mkdir -p $out/assets

          cp -r assets $out
          cp dist/${libraryName}_bg.wasm.opt $out/${libraryName}_bg.wasm
          cp dist/${libraryName}.js $out
          cp ${packageName}/pkg/index.html $out

          runHook postInstall
        '';
      };
    })
    // {
      nixosModules.default = {
        config,
        lib,
        pkgs,
        ...
      }:
        with lib; let
          description = "browser-runnable video game Opencraft";
          cfg = config.services.opencraft;
          runtimeFilesDir = "/var/run/${packageName}";
          platform = pkgs.stdenv.hostPlatform;
        in {
          options.services.opencraft.enable = mkEnableOption description;

          config = mkIf cfg.enable {
            users.groups."${packageName}" = {};
            users.users."${packageName}" = {
              group = "${packageName}";
              isSystemUser = true;
            };

            systemd.tmpfiles.rules = [
              # Type Path Mode User Group Age Argument
              "d ${runtimeFilesDir} 0755 ${packageName} ${packageName} - -"
              "L+ ${runtimeFilesDir}/frontend - - - - ${self.packages.${platform.system}.default}"
            ];
          };
        };
    }
  );
}
