{
  description = "things";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    nci.url = "github:yusdacra/nix-cargo-integration";
    nci.inputs.nixpkgs.follows = "nixpkgs";

    parts.url = "github:hercules-ci/flake-parts";
    parts.inputs.nixpkgs-lib.follows = "nixpkgs";
  };

  outputs =
    inputs@{ self
    , nixpkgs
    , flake-utils
    , nci
    , parts
    , ...
    }:
    parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;
      imports = [
        nci.flakeModule
      ];
      perSystem = { config, pkgs, system, ... }:
        let
          outputs = config.nci.outputs;
          rusty-dialemma = outputs."rusty-dialemma";
#          requiredOverrides = with pkgs; { extraDeps ? [ ], envVars ? { } }: old: {
#            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
#              pkg-config
#            ];
#            buildInputs = extraDeps ++ (old.buildInputs or [ ]) ++ [
#              libiconv
#            ] ++ lib.optionals stdenv.isDarwin [
#              darwin.apple_sdk.frameworks.Security
#            ];
#          } // envVars;
#          cratesCfg = { extraDeps ? [ ], extraAttrs ? { }, envVars ? { } }: {
#            overrides = {
#              add-inputs.overrideAttrs = requiredOverrides { inherit extraDeps envVars; };
#            };
#          } // extraAttrs;
        in
        {
          nci.projects."rusty-dialemma".relPath = "";
          nci.crates = {
            "rusty-dialemma" = {};
          };

          devShells.default = rusty-dialemma.devShell;
          # packages.rusty-dialemma = rusty-dialemma.packages.release; 
          packages.default = rusty-dialemma.packages.release; 
          packages.binary = pkgs.runCommandLocal "kb.bin" {buildInputs = with pkgs; [llvm];} ''
            llvm-objcopy -O binary ${rusty-dialemma.packages.release}/bin/rusty-dialemma $out
          '';
        };
    };
}
