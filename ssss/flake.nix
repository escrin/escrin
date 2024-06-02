{
  description = "SSSS";

  inputs = {
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix.url = "github:nix-community/crate2nix";

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
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
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

      perSystem =
        { system
        , pkgs
        , lib
        , inputs'
        , ...
        }:
        let
          generatedCargoNix = inputs.crate2nix.tools.${system}.appliedCargoNix {
            name = "rustnix";
            src = ./.;
          };
          darwinFrameworks = attrs:
            let
              isDarwin = lib.hasSuffix "darwin" system;
            in
            {
              buildInputs =
                if isDarwin
                then [
                  pkgs.darwin.apple_sdk.frameworks.Foundation
                  pkgs.darwin.apple_sdk.frameworks.Security
                  pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
                ]
                else [ ];
            };
          cargoOverride = attrs: {
            CARGO = "${pkgs.rust-toolchain}/bin/cargo";
          };
          cargoNix = generatedCargoNix.override {
            defaultCrateOverrides =
              pkgs.defaultCrateOverrides
              // {
                # scale-info is a proc macro that generates code requiring cargo in the env
                scale-info = cargoOverride;
                ethbloom = cargoOverride;
                primitive-types = cargoOverride;
                ethereum-types = cargoOverride;
                ssss = attrs:
                  let
                    abis = builtins.path { path = ../evm/abi; };
                  in
                  (darwinFrameworks attrs) // {
                    prePatch = ''
                      export ABI_DIR="$TMPDIR/abi"
                      mkdir -p $ABI_DIR
                      cp -R ${abis}/* $ABI_DIR
                    '';
                  };
                s4 = darwinFrameworks;
              };
          };
        in
        rec {
          checks = {
            rustnix = pkgs.symlinkJoin {
              name = "all-workspace-members-test";
              paths =
                let
                  members = builtins.attrValues cargoNix.workspaceMembers;
                  neAttestationDoc = builtins.path {
                    path = ../evm/test/identity/v1/permitters/att_doc_sample.bin;
                    name = "att_doc_sample.bin";
                  };
                in
                builtins.map
                  (m: m.build.override {
                    runTests = true;
                    testCrateFlags = [ "--skip" "aws" "--skip" "azure" "--skip" "gcp" ];
                    testPreRun = "export NIX_NE_ATT_DOC=\"${neAttestationDoc}\"";
                  })
                  members;
            };
          };

          packages = {
            rustnix = cargoNix.allWorkspaceMembers;
            default = packages.rustnix;

            rust-toolchain-versions = pkgs.writeScriptBin "rust-toolchain-versions" ''
              ${pkgs.rust-toolchain}/bin/cargo --version
              ${pkgs.rust-toolchain}/bin/rustc --version
            '';
          };

          formatter = nixpkgs.legacyPackages.${system}.nixpkgs-fmt;
        };
    };
}
