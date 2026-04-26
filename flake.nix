{
  description = "Stellar Blade Bouldy modkit";

  inputs = {
    rs-harbor.url = "git+ssh://git@codeberg.org/caniko/rs-harbor.git";

    nixpkgs.follows = "rs-harbor/nixpkgs";
    rust-overlay.follows = "rs-harbor/rust-overlay";
    crane.follows = "rs-harbor/crane";
    flake-utils.follows = "rs-harbor/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rs-harbor,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      toolchain = rs-harbor.lib.mkToolchain {
        inherit pkgs;
        channel = "stable";
        extensions = ["rust-src" "rustfmt" "clippy"];
        crossTargets = [
          "x86_64-unknown-linux-gnu"
          "x86_64-pc-windows-gnu"
        ];
      };
      inherit (toolchain) craneLib;

      cross = rs-harbor.lib.mkCross {
        inherit pkgs system;
      };

      cargoConfig = rs-harbor.lib.mkCargoConfig {
        inherit pkgs;
        channel = "stable";
        crossTargets = toolchain.crossTargets;
      };

      src = craneLib.cleanCargoSource ./.;

      commonArgs = {
        inherit src;
        strictDeps = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      package = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
        });
    in {
      packages.default = package;

      checks = {
        default = package;

        clippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

        fmt = craneLib.cargoFmt {
          inherit src;
        };
      };

      devShells = rs-harbor.lib.mkDevShells {
        inherit pkgs cross cargoConfig;
        inherit (toolchain) craneLib;
        checks = self.checks.${system};
        packages = with pkgs; [
          cargo-nextest
          rust-analyzer
        ];
        extraShellHook = ''
          echo "Stellar Blade Bouldy modkit"
          echo "Native:  cargo test"
          echo "Windows: cargo build --release --target x86_64-pc-windows-gnu"
        '';
      };
    });
}
