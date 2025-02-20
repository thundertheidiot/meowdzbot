{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
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
    in rec {
      default = craneLib.buildPackage rec {
        inherit (cargoToml.package) version;
        pname = cargoToml.package.name;
        src = ./.;

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src pname version;
        };

        nativeBuildInputs = with pkgs; [
          sqlx-cli
        ];

        preBuild = ''
          export DATABASE_URL=sqlite:./db.sqlite3
           sqlx database create
           sqlx migrate run
        '';

        installPhaseCommand = ''
          mkdir -p $out/bin
          cp target/release/${pname} $out/bin/
          cp -r target/release/static $out/static
        '';
      };

      docker = pkgs.dockerTools.buildLayeredImage {
        name = "registry.gitlab.com/thundertheidiot/meowdzbot";
        tag = "latest";

        contents = "${default}";

        config = {
          Env = [
            "DATABASE_URL=sqlite:/meow.db"
          ];
          ExposedPorts = {
            "8080" = {};
          };
          Cmd = "/bin/meowdz-bot";
        };
      };
    });
  };
}
