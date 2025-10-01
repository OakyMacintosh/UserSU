{
  description = "UserSU nix flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        pkgsCross = pkgs.pkgsCross;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.gcc
            pkgs.kmod
            pkgs.linux-headers
            pkgs.glibc
            pkgs.rustup

            # Cross-compilers
            pkgsCross.aarch64-multiplatform.buildPackages.gcc
            pkgsCross.armv7l-hf-multiplatform.buildPackages.gcc
          ];

          shellHook = ''
            clear
            rustup default stable
            aarch64-unknown-linux-gnu-gcc -V
            armv7l-unknown-linux-gnueabihf-gcc -V
            gcc -V
            rustc --version
          '';
      };
  });
}