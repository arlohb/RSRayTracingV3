{
  description = "A simple rust project";
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        name = "rs_ray_tracing_v3";
        version = "0.0.1";
        deps = with pkgs; [
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi

          vulkan-loader

          wasm-pack
          deno
        ];

        package = pkgs.rustPlatform.buildRustPackage {
          inherit version;
          pname = name;

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = deps ++ [
            (pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml)
          ];

          env.SHADERC_LIB_DIR = "${pkgs.shaderc.lib}/lib";
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml)
            # cargo
            # rustc
            rust-analyzer
            # clippy
            # rustfmt
            # cargo-binutils
          ] ++ deps;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps;
          SHADERC_LIB_DIR = "${pkgs.shaderc.lib}/lib";
        };

        packages = rec {
          "${name}" = package;
          default = package;
        };
      }
    );
}
