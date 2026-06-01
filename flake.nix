{
  description = "DCR — Cargo-like utility for managing C/C++ projects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    treefmt-nix.url = "github:numtide/treefmt-nix";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    treefmt-nix,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };

      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      packageName = cargoToml.package.name;
      packageVersion = cargoToml.package.version;

      rustToolchain = fenix.packages.${system}.stable.toolchain;

      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };

      treefmtEval = treefmt-nix.lib.evalModule pkgs {
        projectRootFile = "flake.nix";

        programs.alejandra.enable = true;
        programs.rustfmt.enable = true;
        programs.taplo.enable = true;
      };

      app = rustPlatform.buildRustPackage {
        pname = packageName;
        version = packageVersion;

        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.cmake
          pkgs.perl
        ];

        buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Tests require external services; run separately.
        doCheck = false;
      };
    in {
      packages = {
        default = app;
        dcr = app;
      };

      apps = let
        defaultApp =
          (flake-utils.lib.mkApp {
            drv = app;
          })
          // {
            meta.description = "Run ${packageName}";
          };
      in {
        default = defaultApp;
        dcr = defaultApp;
      };

      checks = {
        dcr = app;
        formatting = treefmtEval.config.build.check self;
      };

      formatter = treefmtEval.config.build.wrapper;

      devShells.default = pkgs.mkShell {
        inputsFrom = [app];

        packages = [
          rustToolchain
          pkgs.rust-analyzer

          treefmtEval.config.build.wrapper
        ];

        shellHook = ''
          export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
        '';
      };
    });
}
