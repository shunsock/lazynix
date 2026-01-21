{
  description = "Rust Builder, Runner and DevShell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils }:
    let
      # Release target systems for cross-compilation
      releaseSystems = {
        x86_64-linux = {
          system = "x86_64-linux";
          rustTarget = "x86_64-unknown-linux-gnu";
        };
        aarch64-linux = {
          system = "aarch64-linux";
          rustTarget = "aarch64-unknown-linux-gnu";
        };
        aarch64-darwin = {
          system = "aarch64-darwin";
          rustTarget = "aarch64-apple-darwin";
        };
      };

      # Function to create a package for a specific system
      mkPackageForSystem = { system, rustTarget }: let
        # For aarch64-linux, cross-compile from x86_64-linux
        pkgs = if system == "aarch64-linux"
          then import nixpkgs {
            system = "x86_64-linux";
            crossSystem = {
              config = rustTarget;
            };
          }
          else import nixpkgs {
            inherit system;
          };

        craneLib = crane.mkLib pkgs;

        # Custom source filter to include templates and README files
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type)
            || (builtins.match ".*src/templates/.*" path != null)
            || (builtins.match ".*/README\.md$" path != null);
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          # Explicitly set package name and version
          pname = "lnix";
          version = "2.0.0";

          # Specify target for cross-compilation
          CARGO_BUILD_TARGET = rustTarget;

          # MacOS-specific dependencies
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
        craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          # Skip tests in Nix build (tests require nix command and binary paths)
          # Tests are run separately via CI and local development
          doCheck = false;
        });
    in
    {
      # Release packages for all target platforms
      releasePackages = {
        x86_64-linux = {
          rust_builder = mkPackageForSystem releaseSystems.x86_64-linux;
        };
        aarch64-linux = {
          rust_builder = mkPackageForSystem releaseSystems.aarch64-linux;
        };
        aarch64-darwin = {
          rust_builder = mkPackageForSystem releaseSystems.aarch64-darwin;
        };
      };
    }
    // flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        # Initialize Crane library
        craneLib = crane.mkLib pkgs;

        # Custom source filter to include templates and README files
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type)
            || (builtins.match ".*src/templates/.*" path != null)
            || (builtins.match ".*/README\.md$" path != null);
        };

        # Common arguments for all builds
        commonArgs = {
          inherit src;
          strictDeps = true;

          # Explicitly set package name and version
          pname = "lnix";
          version = "2.0.0";

          # MacOS-specific dependencies
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        # Build dependencies only (for caching)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual package
        rustBuilder = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          # Skip tests in Nix build (tests require nix command and binary paths)
          # Tests are run separately via CI and local development
          doCheck = false;
        });

      in
      {
        # Development shell
        devShells.default = craneLib.devShell {
          # Inherit inputs from checks
          checks = self.checks.${system} or {};

          packages = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
          ];

          shellHook = ''
            echo "Rust Builder Dev Environment"
            rustc --version
            cargo --version
          '';
        };

        # Package outputs
        packages = {
          default = rustBuilder;
          rust_builder = rustBuilder;
        };

        # App for `nix run`
        apps.default = flake-utils.lib.mkApp {
          drv = rustBuilder;
        };

        # Quality checks
        checks = {
          inherit rustBuilder;

          rust-builder-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          rust-builder-fmt = craneLib.cargoFmt {
            inherit (commonArgs) src;
          };
        };
      }
    );
}
