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

  nixConfig = {
    extra-trusted-public-keys = "eigenvalue.cachix.org-1:ykerQDDa55PGxU25CETy9wF6uVDpadGGXYrFNJA3TUs=";
    extra-substituters = "https://eigenvalue.cachix.org";
    allow-import-from-derivation = true;
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
          cargoNix = generatedCargoNix.override {
            defaultCrateOverrides = pkgs.defaultCrateOverrides // {
              scale-info = attrs: {
                CARGO = "${pkgs.rust-toolchain}/bin/cargo";
              };
              ssss = attrs:
                let
                  isDarwin = lib.hasSuffix "darwin" system;
                in {
                  buildInputs = if isDarwin then [
                    pkgs.darwin.apple_sdk.frameworks.Security
                    pkgs.darwin.apple_sdk.frameworks.Foundation
                  ] else [];
              };
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
            rustnix = cargoNix.workspaceMembers.ssss.build;
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
