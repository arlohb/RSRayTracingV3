{
  description = "A simple rust project";
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";

        name = "name";
        version = "0.0.1";
        deps = with pkgs; [
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi

          vulkan-loader
        ];

        package = pkgs.rustPlatform.buildRustPackage {
          inherit version;
          pname = name;

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = deps;
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
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
