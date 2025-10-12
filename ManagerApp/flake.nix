{
  description = "Android development environment for ManagerApp";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.android_sdk.accept_license = true;
        };

        android-sdk = pkgs.android-sdk.composeAndroidPackages {
          buildToolsVersions = [ "34.0.0" ];
          platformVersions = [ "34" ];
          cmdlineToolsVersion = "latest";
          includeEmulator = true;
          includeSystemImages = true;
          systemImageTypes = [ "google_apis" ];
          abiVersions = [ "x86_64" ];
        };

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.jdk11
            android-sdk
          ];

          # Set environment variables for Android development
          ANDROID_HOME = "${android-sdk}/libexec/android-sdk";
          ANDROID_SDK_ROOT = "${android-sdk}/libexec/android-sdk";

          # Add Android SDK tools to PATH
          shellHook = ''
            export PATH=$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH
          '';
        };
      });
}
