{
  description = "UserSU flake (GNU/Linux cross compilers + Android NDK)";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    android-nixpkgs.url = "github:tadfisher/android-nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Use the NDK from androidenv (more portable across nixpkgs versions)
        ndk = pkgs.androidenv.ndkPackages_21d.ndk;

        targets = [
          { name = "aarch64"; triple = "aarch64-linux-android"; api = "21"; }
          { name = "armv7";   triple = "armv7a-linux-androideabi"; api = "21"; }
          { name = "i686";    triple = "i686-linux-android"; api = "21"; }
          { name = "x86_64";  triple = "x86_64-linux-android"; api = "21"; }
        ];

        mkCompilerWrapper = target: let
          prefix = "${ndk}/toolchains/llvm/prebuilt/linux-x86_64/bin";
          clang = "${prefix}/${target.triple}${target.api}-clang";
          clangpp = "${prefix}/${target.triple}${target.api}-clang++";
        in [
          (pkgs.writeShellScriptBin "android-${target.name}-gcc" ''
            exec ${clang} "$@"
          '')
          (pkgs.writeShellScriptBin "android-${target.name}-g++" ''
            exec ${clangpp} "$@"
          '')
        ];

        compilerWrappers = pkgs.lib.concatMap mkCompilerWrapper targets;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.gcc
            pkgs.rustup
            pkgs.just
            pkgs.python314
            pkgs.uv
            pkgs.android-tools
            ndk
          ] ++ compilerWrappers;
        };
      });
}