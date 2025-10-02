{
  description = "UserSU nix flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable"; # [span_0](start_span)Already set to unstable[span_0](end_span)
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
    
            # pkgs.linuxHeaders
            pkgs.glibc
            pkgs.rustup

            # === NEW ADDITIONS FOR PYTHON/UV ===
            pkgs.python314 # Python 3.14
            pkgs.uv         # uv package manager
            # =================================

            # [span_1](start_span)Cross-compilers[span_1](end_span)
            pkgsCross.aarch64-multiplatform.buildPackages.gcc
            pkgsCross.armv7l-hf-multiplatform.buildPackages.gcc
          ];

          shellHook = ''
            [span_2](start_span)clear[span_2](end_span)
            [span_3](start_span)rustup default stable[span_3](end_span)

            # Optional: Add a setup message for the new tools
            echo "Python 3.14 and uv are available."
          '';
      };
  });
}
