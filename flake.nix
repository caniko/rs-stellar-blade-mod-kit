{
  description = "Stellar Blade Bouldy modkit";

  # Advertise the private macOS SDK Attic cache so darwin cross-compiles
  # substitute the realized SDK from the pin instead of rebuilding it.
  nixConfig = {
    extra-substituters = ["https://attic.candee.baby/harbor-macos-sdk"];
    extra-trusted-public-keys = [
      "harbor-macos-sdk:ci7MNMkHDqdeTS4aKwzDNEJ1175AbpVUypTRjCJoHDk="
    ];
  };

  inputs = {
    rs-harbor.url = "git+https://github.com/caniko/harbor-rs.git?ref=trunk&rev=05cc4f162b55fa904b687db1821e2463fa813e50";

    rs-harbor-macos-sdk-pin.url = "git+ssh://git@github.com/caniko/harbor-macos-sdk-pin.git";

    nixpkgs.follows = "rs-harbor/nixpkgs";
    rust-overlay.follows = "rs-harbor/rust-overlay";
    crane.follows = "rs-harbor/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rs-harbor,
    rs-harbor-macos-sdk-pin,
    flake-utils,
    rust-overlay,
    ...
  }: let
    mkOutputs = {
      macosSdkStorePath ? rs-harbor-macos-sdk-pin.storePath,
      osxSdkVersion ? rs-harbor-macos-sdk-pin.sdkVersion,
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

        cross = rs-harbor.lib.mkCross ({
            inherit pkgs system osxSdkVersion;
          }
          // pkgs.lib.optionalAttrs (macosSdkStorePath != null) {
            inherit macosSdkStorePath;
          });

        cargoConfig = rs-harbor.lib.mkCargoConfig {
          inherit pkgs;
          channel = "stable";
          crossTargets = toolchain.crossTargets;
        };

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type: let
            rel = pkgs.lib.removePrefix "${toString ./.}/" (toString path);
          in
            (craneLib.filterCargoSources path type)
            || pkgs.lib.hasPrefix "vendor/bouldy/" rel;
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          nativeBuildInputs = [pkgs.clang pkgs.mold];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        package = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
          });

        retoc = pkgs.rustPlatform.buildRustPackage rec {
          pname = "retoc";
          version = "0.1.5";

          oodleLib = pkgs.fetchurl {
            url = "https://github.com/WorkingRobot/OodleUE/raw/refs/heads/main/Engine/Source/Programs/Shared/EpicGames.Oodle/Sdk/2.9.10/linux/lib/liboo2corelinux64.so.9";
            hash = "sha256-7X6Y9wvhJUqAZE79OuRC/2H4VKL+neuwuXi5UomITpw=";
          };

          src = pkgs.fetchFromGitHub {
            owner = "trumank";
            repo = "retoc";
            rev = "v${version}";
            hash = "sha256-rpDBDViype47oIPfTwlsjRlF9rLDIu3NSGGV46YP8jQ=";
          };

          cargoHash = "sha256-jevIk0XXQbGMA8M94tti8UuXlpcjagpcPCB7PXKewLE=";

          cargoBuildFlags = ["-p" "retoc_cli"];
          cargoTestFlags = ["-p" "retoc_cli"];

          nativeBuildInputs = [pkgs.makeWrapper];

          postInstall = ''
            install -Dm444 ${oodleLib} "$out/bin/liboo2corelinux64.so.9"
            wrapProgram "$out/bin/retoc" \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [pkgs.stdenv.cc.cc.lib]}"
          '';

          meta = {
            description = "Unreal Engine IoStore CLI packing and unpacking tool";
            homepage = "https://github.com/trumank/retoc";
            license = pkgs.lib.licenses.mit;
            mainProgram = "retoc";
          };
        };

        mingw = pkgs.pkgsCross.mingwW64;
        mingwCc = mingw.stdenv.cc;
        mingwGccLib = mingw.stdenv.cc.cc.lib;
        mingwMcfgthreads = mingw.windows.mcfgthreads;
        mingwMcfgthreadsDev = mingw.windows.mcfgthreads.dev;

        mkCargoApp = name: bin: let
          script = pkgs.writeShellApplication {
            inherit name;
            runtimeInputs = [pkgs.nix retoc];
            text = ''
              exec nix develop "$PWD" --command cargo run -p modkit-tools --bin ${bin} -- "$@"
            '';
          };
        in {
          type = "app";
          program = "${script}/bin/${name}";
        };

        exportRuntimeBinaries = let
          script = pkgs.writeShellApplication {
            name = "export-runtime-binaries";
            runtimeInputs = [
              pkgs.coreutils
              pkgs.findutils
              pkgs.git
              pkgs.gnugrep
              pkgs.nix
            ];
            text = ''
              usage() {
                cat <<'USAGE'
              Usage:
                export-runtime-binaries --game-root /path/to/StellarBlade
                export-runtime-binaries --win64-dir /path/to/StellarBlade/SB/Binaries/Win64

              Builds the Windows Rust runtime DLL and UE4SS C++ adapter, then exports
              them into Mods/StellarBladeBouldyRecon under the target Win64 directory.
              USAGE
              }

              game_root=""
              win64_dir=""

              while [ "$#" -gt 0 ]; do
                case "$1" in
                  --game-root)
                    if [ "$#" -lt 2 ]; then
                      echo "error: --game-root requires a path" >&2
                      usage >&2
                      exit 2
                    fi
                    game_root="$2"
                    shift 2
                    ;;
                  --win64-dir)
                    if [ "$#" -lt 2 ]; then
                      echo "error: --win64-dir requires a path" >&2
                      usage >&2
                      exit 2
                    fi
                    win64_dir="$2"
                    shift 2
                    ;;
                  -h|--help)
                    usage
                    exit 0
                    ;;
                  *)
                    echo "error: unknown argument: $1" >&2
                    usage >&2
                    exit 2
                    ;;
                esac
              done

              if [ -z "$game_root" ] && [ -z "$win64_dir" ]; then
                echo "error: expected --game-root or --win64-dir" >&2
                usage >&2
                exit 2
              fi

              if [ -n "$game_root" ] && [ -n "$win64_dir" ]; then
                echo "error: pass only one of --game-root or --win64-dir" >&2
                usage >&2
                exit 2
              fi

              if [ -n "$game_root" ]; then
                if [ ! -d "$game_root" ]; then
                  echo "error: missing game root: $game_root" >&2
                  echo "required because the exporter must resolve SB/Binaries/Win64 from the Stellar Blade install" >&2
                  echo "upstream producer: Steam local game install" >&2
                  echo "regenerate/retry: install Stellar Blade or pass --win64-dir directly" >&2
                  echo "validation: test -d '$game_root/SB/Binaries/Win64'" >&2
                  exit 1
                fi
                win64_dir="$game_root/SB/Binaries/Win64"
              fi

              ensure_bouldy_mod_enabled() {
                mods_file="$1"
                enabled_line="StellarBladeBouldyRecon : 1"
                tmp_file="$mods_file.tmp.$$"
                inserted=0

                if [ -f "$mods_file" ]; then
                  while IFS= read -r line || [ -n "$line" ]; do
                    if printf '%s\n' "$line" | grep -Eq '^[[:space:]]*StellarBladeBouldyRecon[[:space:]]*:'; then
                      continue
                    fi

                    case "$line" in
                      "; Built-in keybinds, do not move up!"*)
                        if [ "$inserted" -eq 0 ]; then
                          printf '%s\n' "$enabled_line"
                          inserted=1
                        fi
                        printf '%s\n' "$line"
                        ;;
                      *)
                        printf '%s\n' "$line"
                        ;;
                    esac
                  done < "$mods_file" > "$tmp_file"

                  if [ "$inserted" -eq 0 ]; then
                    printf '\n%s\n' "$enabled_line" >> "$tmp_file"
                  fi
                else
                  printf '%s\n' "$enabled_line" > "$tmp_file"
                fi

                mv "$tmp_file" "$mods_file"
              }

              if [ ! -d "$win64_dir" ]; then
                echo "error: missing Win64 directory: $win64_dir" >&2
                echo "required because UE4SS mods load from the game's SB/Binaries/Win64 directory" >&2
                echo "upstream producer: Steam local game install" >&2
                echo "regenerate/retry: verify the Stellar Blade install path or pass --win64-dir" >&2
                echo "validation: test -d '$win64_dir'" >&2
                exit 1
              fi

              shipping_exe="$win64_dir/SB-Win64-Shipping.exe"
              if [ ! -f "$shipping_exe" ]; then
                echo "error: missing shipping executable: $shipping_exe" >&2
                echo "required because this confirms the target is Stellar Blade's Win64 runtime directory" >&2
                echo "upstream producer: Steam local game install" >&2
                echo "regenerate/retry: verify game files in Steam or pass the correct --win64-dir" >&2
                echo "validation: test -f '$shipping_exe'" >&2
                exit 1
              fi

              repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
              cd "$repo_root"

              mkdir -p target/mingw-libs target/export-runtime
              ln -sf "${mingwMcfgthreads}/lib/libmcfgthread.a" target/mingw-libs/libpthread.a

              toolchain_file="$repo_root/target/export-runtime/mingw-toolchain.cmake"
              cat > "$toolchain_file" <<'TOOLCHAIN'
              set(CMAKE_SYSTEM_NAME Windows)
              set(CMAKE_SYSTEM_PROCESSOR x86_64)
              set(CMAKE_C_COMPILER x86_64-w64-mingw32-gcc)
              set(CMAKE_CXX_COMPILER x86_64-w64-mingw32-g++)
              set(CMAKE_RC_COMPILER x86_64-w64-mingw32-windres)
              TOOLCHAIN

              build_script="$repo_root/target/export-runtime/build-runtime.sh"
              cat > "$build_script" <<'BUILD'
              set -euo pipefail
              cd "$BOULDY_REPO_ROOT"

              export PATH="$BOULDY_MINGW_BIN:$PATH"
              export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
              export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-L native=$BOULDY_REPO_ROOT/target/mingw-libs -L native=$BOULDY_MCFG_LIB"

              cargo build -p stellar-blade-bouldy-mod --release --target x86_64-pc-windows-gnu

              rm -rf target/ue4ss-cpp-mingw
              cmake -S loader/ue4ss-cpp -B target/ue4ss-cpp-mingw -G Ninja \
                -DCMAKE_TOOLCHAIN_FILE="$BOULDY_TOOLCHAIN_FILE" \
                -DBOULDY_BUILD_UE4SS_ENTRY=ON \
                -DCMAKE_C_FLAGS="-I$BOULDY_MCFG_INCLUDE" \
                -DCMAKE_CXX_FLAGS="-I$BOULDY_MCFG_INCLUDE" \
                -DCMAKE_EXE_LINKER_FLAGS="-L$BOULDY_MCFG_LIB" \
                -DCMAKE_SHARED_LINKER_FLAGS="-L$BOULDY_MCFG_LIB"
              cmake --build target/ue4ss-cpp-mingw
              BUILD
              chmod +x "$build_script"

              export BOULDY_REPO_ROOT="$repo_root"
              export BOULDY_MINGW_BIN="${mingwCc}/bin"
              export BOULDY_MCFG_LIB="${mingwMcfgthreads}/lib"
              export BOULDY_MCFG_INCLUDE="${mingwMcfgthreadsDev}/include"
              export BOULDY_TOOLCHAIN_FILE="$toolchain_file"
              nix develop "$repo_root" --command bash "$build_script"

              rust_dll="$repo_root/target/x86_64-pc-windows-gnu/release/stellar_blade_bouldy_mod.dll"
              adapter_dll="$repo_root/target/ue4ss-cpp-mingw/libStellarBladeBouldyRecon.dll"
              gcc_runtime="${mingwGccLib}/x86_64-w64-mingw32/lib"
              mcfg_runtime="${mingwMcfgthreads}/bin"

              required_sources=(
                "$rust_dll"
                "$adapter_dll"
                "$gcc_runtime/libgcc_s_seh-1.dll"
                "$gcc_runtime/libstdc++-6.dll"
                "$mcfg_runtime/libmcfgthread-2.dll"
              )

              for source in "''${required_sources[@]}"; do
                if [ ! -f "$source" ]; then
                  echo "error: missing build artifact: $source" >&2
                  echo "required because the runtime export must include all DLLs needed by the UE4SS adapter" >&2
                  echo "upstream producer: nix run .#export-runtime-binaries build phase" >&2
                  echo "regenerate/retry: nix run .#export-runtime-binaries -- --win64-dir '$win64_dir'" >&2
                  echo "validation: test -f '$source'" >&2
                  exit 1
                fi
              done

              mod_dir="$win64_dir/Mods/StellarBladeBouldyRecon"
              dll_dir="$mod_dir/dlls"
              mkdir -p "$dll_dir"

              install -m 0644 "$adapter_dll" "$dll_dir/main.dll"
              install -m 0644 "$rust_dll" "$dll_dir/stellar_blade_bouldy_mod.dll"
              install -m 0644 "$rust_dll" "$win64_dir/stellar_blade_bouldy_mod.dll"
              install -m 0644 "$gcc_runtime/libgcc_s_seh-1.dll" "$dll_dir/libgcc_s_seh-1.dll"
              install -m 0644 "$gcc_runtime/libstdc++-6.dll" "$dll_dir/libstdc++-6.dll"
              install -m 0644 "$mcfg_runtime/libmcfgthread-2.dll" "$dll_dir/libmcfgthread-2.dll"

              canonical_mods_file="$win64_dir/Mods/mods.txt"
              mkdir -p "$win64_dir/Mods"
              ensure_bouldy_mod_enabled "$canonical_mods_file"

              legacy_mods_file="$win64_dir/mods.txt"
              if [ -f "$legacy_mods_file" ]; then
                if ! grep -Eq '^[[:space:]]*StellarBladeBouldyRecon[[:space:]]*:[[:space:]]*1[[:space:]]*$' "$legacy_mods_file"; then
                  printf '\nStellarBladeBouldyRecon : 1\n' >> "$legacy_mods_file"
                fi
              else
                printf 'StellarBladeBouldyRecon : 1\n' > "$legacy_mods_file"
              fi

              if ! find "$win64_dir" -maxdepth 1 \
                \( -name 'UE4SS.dll' -o -name 'xinput1_3.dll' -o -name 'dwmapi.dll' -o -name 'dsound.dll' \) \
                -print -quit | grep -q .; then
                echo "warning: no UE4SS loader/proxy DLL was found in $win64_dir" >&2
                echo "warning: exported recon DLLs will not load until UE4SS is installed separately" >&2
              fi

              echo "exported StellarBladeBouldyRecon runtime binaries to: $dll_dir"
              echo "enabled mod entry in: $canonical_mods_file"
              echo "updated legacy compatibility mod entry in: $legacy_mods_file"
            '';
          };
        in {
          type = "app";
          program = "${script}/bin/export-runtime-binaries";
        };

        installUe4ss = let
          script = pkgs.writeShellApplication {
            name = "install-ue4ss";
            runtimeInputs = [
              pkgs.coreutils
              pkgs.git
              pkgs.nix
            ];
            text = ''
              usage() {
                cat <<'USAGE'
              Usage:
                install-ue4ss --game-root /path/to/StellarBlade
                install-ue4ss --win64-dir /path/to/StellarBlade/SB/Binaries/Win64

              Installs the Bouldy-pinned UE4SS release into the target Win64 directory.
              USAGE
              }

              game_root=""
              win64_dir=""

              while [ "$#" -gt 0 ]; do
                case "$1" in
                  --game-root)
                    if [ "$#" -lt 2 ]; then
                      echo "error: --game-root requires a path" >&2
                      usage >&2
                      exit 2
                    fi
                    game_root="$2"
                    shift 2
                    ;;
                  --win64-dir)
                    if [ "$#" -lt 2 ]; then
                      echo "error: --win64-dir requires a path" >&2
                      usage >&2
                      exit 2
                    fi
                    win64_dir="$2"
                    shift 2
                    ;;
                  -h|--help)
                    usage
                    exit 0
                    ;;
                  *)
                    echo "error: unknown argument: $1" >&2
                    usage >&2
                    exit 2
                    ;;
                esac
              done

              if [ -z "$game_root" ] && [ -z "$win64_dir" ]; then
                echo "error: expected --game-root or --win64-dir" >&2
                usage >&2
                exit 2
              fi

              if [ -n "$game_root" ] && [ -n "$win64_dir" ]; then
                echo "error: pass only one of --game-root or --win64-dir" >&2
                usage >&2
                exit 2
              fi

              if [ -n "$game_root" ]; then
                if [ ! -d "$game_root" ]; then
                  echo "error: missing game root: $game_root" >&2
                  echo "required because the installer must resolve SB/Binaries/Win64 from the Stellar Blade install" >&2
                  echo "upstream producer: Steam local game install" >&2
                  echo "regenerate/retry: install Stellar Blade or pass --win64-dir directly" >&2
                  echo "validation: test -d '$game_root/SB/Binaries/Win64'" >&2
                  exit 1
                fi
                win64_dir="$game_root/SB/Binaries/Win64"
              fi

              repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
              exec nix run "$repo_root/vendor/bouldy#install-ue4ss" -- --win64-dir "$win64_dir"
            '';
          };
        in {
          type = "app";
          program = "${script}/bin/install-ue4ss";
        };
      in {
        packages.default = package;
        packages.retoc = retoc;

        apps = {
          default = mkCargoApp "offline-recon" "offline_recon";
          offline-recon = mkCargoApp "offline-recon" "offline_recon";
          analyze-candidates = mkCargoApp "analyze-candidates" "analyze_candidates";
          analyze-mod-example = mkCargoApp "analyze-mod-example" "analyze_mod_example";
          probe-pak = mkCargoApp "probe-pak" "probe_pak";
          probe-uasset = mkCargoApp "probe-uasset" "probe_uasset";
          compare-skilltable = mkCargoApp "compare-skilltable" "compare_skilltable";
          list-iostore = mkCargoApp "list-iostore" "list_iostore";
          extract-iostore = mkCargoApp "extract-iostore" "extract_iostore";
          scan-asset-strings = mkCargoApp "scan-asset-strings" "scan_asset_strings";
          export-runtime-binaries = exportRuntimeBinaries;
          install-ue4ss = installUe4ss;
          retoc = {
            type = "app";
            program = "${retoc}/bin/retoc";
          };
        };

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
            cmake
            cargo-nextest
            gcc
            just
            ninja
            retoc
            rust-analyzer
          ];
          extraShellHook = ''
            echo "Stellar Blade Bouldy modkit"
            echo "Native:  cargo test"
            echo "C++:     cmake -S loader/ue4ss-cpp -B target/ue4ss-cpp -G Ninja && cmake --build target/ue4ss-cpp && ctest --test-dir target/ue4ss-cpp"
            echo "Windows: cargo build -p stellar-blade-bouldy-mod --release --target x86_64-pc-windows-gnu"
          '';
        };
      });
  in
    mkOutputs {}
    // {
      lib = {
        inherit mkOutputs;
      };
    };
}
