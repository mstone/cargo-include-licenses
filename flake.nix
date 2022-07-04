{
  description = "Collects license files from the dependencies of a project into a predefined directory";

  inputs.crane.url = "github:ipetkov/crane";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.flake-utils.follows = "flake-utils";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  outputs = { self, nixpkgs, crane, rust-overlay, flake-utils}:
    flake-utils.lib.simpleFlake {
      inherit self nixpkgs;
      name = "cargo-include-licenses";
      systems = flake-utils.lib.allSystems;
      preOverlays = [ rust-overlay.overlays.default ];
      overlay = final: prev: {
        cargo-include-licenses = rec {
          rust = with final; with pkgs; rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          cranelib = crane.lib."${final.system}".overrideToolchain rust;
          cargo-include-licenses = with final; with pkgs; let
            buildInputs = [
              rust
            ];
          in cranelib.buildPackage {
            pname = "cargo-include-licenses";
            version = "1.0";
            src = self;
            inherit buildInputs;
            cargoArtifacts = cranelib.buildDepsOnly {
              src = self;
              inherit buildInputs;
            };
          };

          defaultPackage = cargo-include-licenses;
        };
      };
    };
}
