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
          embeddedStuffFilter = path: _type: builtins.match ".*\\.x$" path != null;
          embeddedOrCargoStuff = path: type:
            (embeddedStuffFilter path type) || (craneLib.filterCargoSources path type);
          bootloader = craneLib.buildPackage {
            cargoExtraArgs = "--target thumbv6m-none-eabi";
            src = lib.cleanSourceWith {
              src = craneLib.path ./bootloader;
              filter = embeddedOrCargoStuff;
            };
            doCheck = false;
            buildInputs = [
              # Add additional build inputs here
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];
          };
          firmware = profile: craneLib.buildPackage {
            cargoExtraArgs = "--target thumbv6m-none-eabi";
            CARGO_PROFILE = profile;
            src = lib.cleanSourceWith {
              src = craneLib.path ./firmware;
              filter = embeddedOrCargoStuff;
            };
            doCheck = false;
            buildInputs = [
              # Add additional build inputs here
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];
          };
        in
        rec
        {
          devShells.default = pkgs.mkShell {
            inputsFrom = [ (firmware "dev") ];
            nativeBuildInputs = with pkgs; [ fenix.packages.${system}.rust-analyzer cargo-binutils picotool ];
          };
          packages.default = firmware "release";
          packages.dev = firmware "dev";
          packages.bootloader = bootloader;
          packages.binary = pkgs.runCommandLocal "kb.bin" { } ''
            mkdir -p $out
            cp ${packages.default}/bin/rusty-dialemma $out/rusty-dialemma.elf
          '';
          packages.bootloader-binary = pkgs.runCommandLocal "kb.bin" { } ''
            mkdir -p $out
            cp ${packages.bootloader}/bin/rusty-dialemma-boot $out/rusty-dialemma-boot.elf
          '';
        };
    };
}
