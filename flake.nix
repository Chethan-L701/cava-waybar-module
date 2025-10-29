{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        name = "cava-waybar-module";
        version = "0.2.0";
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = name;
          version = version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          buildType = "release";
        };

        packages.debug = pkgs.rustPlatform.buildRustPackage {
          pname = name;
          version = version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;

          buildType = "debug";
          dontStrip = true;
          seperateDebugInfo = false;
        };

        devShells.default = pkgs.mkShell {

          nativeBuildInputs = [
            pkgs.package-version-server
            pkgs.rust-analyzer
            pkgs.rustfmt
            pkgs.cargo
            pkgs.rustc
          ];

          shellHook = ''
            export NIX_DEV_SHELL="${name}:${version}"
          '';

        };
      }
    );
}
