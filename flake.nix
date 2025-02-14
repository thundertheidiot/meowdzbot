{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    crane,
    ...
  }: let
    inherit (nixpkgs) lib;

    forAllSystems = fn:
      lib.genAttrs lib.systems.flakeExposed
      (system:
        fn
        (import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        }));
  in {
    packages = forAllSystems (pkgs: let
      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in {
      default = craneLib.buildPackage rec {
        inherit (cargoToml.package) version;
        pname = cargoToml.package.name;
        src = ./.;

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src pname version;
        };
      };
    });
  };
}
