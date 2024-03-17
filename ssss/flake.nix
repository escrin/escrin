{
  description = "SSSS";

  inputs = {
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix.url = "github:nix-community/crate2nix";

    # Development

    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs @ { self
    , nixpkgs
    , flake-parts
    , rust-overlay
    , crate2nix
    , ...
    }: flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      imports = [
        ./nix/rust-overlay/flake-module.nix
        ./nix/devshell/flake-module.nix
      ];

      perSystem = { system, pkgs, lib, inputs', ... }:
        let
          generatedCargoNix = inputs.crate2nix.tools.${system}.appliedCargoNix {
            name = "rustnix";
            src = ./.;
          };
          darwinFrameworks = attrs:
            let
              isDarwin = lib.hasSuffix "darwin" system;
            in {
              buildInputs = if isDarwin then [
                pkgs.darwin.apple_sdk.frameworks.Foundation
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ] else [];
            };
          cargoNix = generatedCargoNix.override {
            defaultCrateOverrides = pkgs.defaultCrateOverrides // {
              scale-info = attrs: {
                CARGO = "${pkgs.rust-toolchain}/bin/cargo";
              };
              ssss = darwinFrameworks;
              s4 = darwinFrameworks;
            };
          };
        in
        rec {
          checks = {
            rustnix = cargoNix.workspaceMembers.ssss.build.override {
              runTests = true;
            };
          };

          packages = {
            rustnix = cargoNix.allWorkspaceMembers;
            default = packages.rustnix;

            inherit (pkgs) rust-toolchain;

            rust-toolchain-versions = pkgs.writeScriptBin "rust-toolchain-versions" ''
              ${pkgs.rust-toolchain}/bin/cargo --version
              ${pkgs.rust-toolchain}/bin/rustc --version
            '';
          };
        };
    };
}
