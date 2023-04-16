{
  description = "things";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    parts.url = "github:hercules-ci/flake-parts";
    parts.inputs.nixpkgs-lib.follows = "nixpkgs";
  };

  outputs =
    inputs@{ self
    , nixpkgs
    , crane
    , fenix
    , parts
    , ...
    }:
    parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;
      imports = [
      ];
      perSystem = { config, pkgs, system, lib, ... }:
        let
          toolchain = fenix.packages.${system}.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-/De+QF3/xndFuAqyt7+Nl1EuqaJtRBQqPYRYVXPYC9U=";
          };
          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
          embeddedStuffFilter = path: _type: builtins.match "memory.x$" path != null;
          embeddedOrCargoStuff = path: type:
            (embeddedStuffFilter path type) || (craneLib.filterCargoSources path type);
          crate = craneLib.buildPackage {
            src = craneLib.path ./.;             doCheck = false;
            buildInputs = [
              # Add additional build inputs here
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];
          };
        in
        {
          #          devShells.default = rusty-dialemma.devShell;
          #          # packages.rusty-dialemma = rusty-dialemma.packages.release; 
          packages.default = crate;
          #          packages.binary = pkgs.runCommandLocal "kb.bin" {buildInputs = with pkgs; [llvm];} ''
          #            llvm-objcopy -O binary ${rusty-dialemma.packages.release}/bin/rusty-dialemma $out
          #          '';
        };
    };
}
