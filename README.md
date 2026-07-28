# Stellar Blade Bouldy Modkit

<!-- simit:badges:start -->

![CI](https://img.shields.io/badge/CI-drift-2088ff) [![crates.io](https://img.shields.io/badge/crates.io-ready-f46623)](https://crates.io/crates/modkit-tools)

<!-- simit:badges:end -->

This repository contains a Bouldy-based reconnaissance modkit for the PC release of Stellar Blade. The first version discovers combat-related Unreal symbols through a generic Bouldy discovery API and exports candidate records for later analysis.

It does not patch gameplay, register Stellar Blade-specific hooks, ship generated SDKs, include extracted game assets, bypass anti-cheat, or support online-play modification.

## Assumptions

- Target game: Stellar Blade PC.
- Engine assumption: Unreal Engine 4.26 or 4.26.2.
- Packaging assumption: Io Store-style cooked content (`.utoc`/`.ucas`) with pak-side metadata, based on current public PC modding practice.
- Loader assumption: UE4SS-compatible Bouldy V3 shim deployed in the game's Win64 binary directory.
- Runtime scope: reconnaissance only. All concrete hook targets, offsets, and object paths must come from captured discovery output.

## Build

After cloning, initialize the vendored Bouldy submodule:

```bash
git submodule update --init --recursive
```

Native validation:

```bash
nix develop
cargo test
cargo clippy --all-targets -- -D warnings
```

Windows Rust DLL build:

```bash
nix develop .#windows
cargo build -p stellar-blade-bouldy-mod --release --target x86_64-pc-windows-gnu
```

The DLL is produced at:

```text
target/x86_64-pc-windows-gnu/release/stellar_blade_bouldy_mod.dll
```

Runtime binary export for the local Stellar Blade install:

```bash
nix run .#install-ue4ss -- \
  --game-root /data/nvme0/can/games/steamapps/steamapps/common/StellarBlade

nix run .#export-runtime-binaries -- \
  --game-root /data/nvme0/can/games/steamapps/steamapps/common/StellarBlade
```

On this machine, the same command is available through `just`:

```bash
just install-ue4ss
just export-runtime
```

To install/export to another install:

```bash
just install-ue4ss-to /path/to/StellarBlade
just export-runtime-to /path/to/StellarBlade
```

## Deployment Sketch

Expected UE4SS-style layout:

```text
StellarBlade/Binaries/Win64/
  UE4SS.dll
  Mods/
    StellarBladeBouldyRecon/
      dlls/
        main.dll
        stellar_blade_bouldy_mod.dll
```

`main.dll` is the C++ UE4SS adapter. `stellar_blade_bouldy_mod.dll` is the
Rust Bouldy runtime module loaded by the adapter. The export command builds and
copies both DLLs, copies the required MinGW runtime DLLs, and also places
`stellar_blade_bouldy_mod.dll` beside the game executable because the current
adapter uses a bare `LoadLibraryW`.

Enable it with:

```text
StellarBladeBouldyRecon : 1
```

Place the line before UE4SS's built-in keybind section in
`SB/Binaries/Win64/Mods/mods.txt` if load order matters. The UE4SS installer and
runtime exporter preserve an existing `Mods/mods.txt` and add the line only when
it is missing. The runtime exporter also writes a top-level `mods.txt` as legacy
compatibility for older local experiments.

UE4SS is pinned through the vendored Bouldy flake. The default installer uses
the stable UE4SS v3.0.1 archive; Bouldy also pins the zDEV archive for later
debugging and reflection work.

## Reconnaissance Output

At initialization, the mod requests discovery candidates for combat-related terms such as parry, dodge, evade, guard, damage, hit reaction, stagger, lock-on, enemy combatants, input windows, and attack intent. Matching candidates are exported on the `stellar_blade.combat_recon` channel as stable JSON-style records:

```json
{"schema_version":2,"kind":4,"name":"ExampleParryWindow","path":"/Script/Example","owner":"ExampleOwner","flags":0,"chunk_index":12,"object_index":3456}
```

This output is evidence for future hook design. It is not a statement that the named example exists in Stellar Blade.

The C++ adapter persists raw scan candidates and Rust-filtered exports to:

```text
Mods/StellarBladeBouldyRecon/data/recon.sqlite3
```

Tables:

- `scan_runs`: one row per mod startup scan session.
- `candidates`: every raw metadata-only candidate seen by the C++ scan.
- `exports`: Rust-filtered combat records sent through `export_record`.

Local C++ validation:

```bash
nix develop
cmake -S loader/ue4ss-cpp -B target/ue4ss-cpp -G Ninja
cmake --build target/ue4ss-cpp
ctest --test-dir target/ue4ss-cpp --output-on-failure
```

## Local Stellar Blade Recon

Treat the local game root as:

```text
/data/nvme0/can/games/steamapps/steamapps/common/StellarBlade
```

The current install uses Io Store-style content under `SB/Content/Paks`.
Keep package inventories, extracted assets, dumps, SDKs, `.usmap` files, and
all generated game data local and untracked.

The flake apps are local-workspace wrappers around `nix develop --command cargo
run`, so run them from the repository checkout where `vendor/bouldy` is present.
Reproduce the current offline first pass with:

```bash
nix run .#offline-recon -- \
  --game-root /data/nvme0/can/games/steamapps/steamapps/common/StellarBlade \
  --out local/stellarblade/recon/first-pass

nix run .#analyze-candidates -- \
  --input local/stellarblade/recon/first-pass/combat_candidates.jsonl \
  --out local/stellarblade/recon/first-pass/analysis

nix run .#analyze-mod-example -- \
  --input local/stellarblade/recon/first-pass/visible_strings.jsonl \
  --out local/stellarblade/recon/first-pass/analysis/mod-example.md

nix run .#probe-pak -- \
  --pak /data/nvme0/can/games/steamapps/steamapps/common/StellarBlade/SB/Content/Paks/~mods-off/SB_ImprovedPerfectDefense_Extended_P.pak \
  --out local/stellarblade/recon/first-pass/analysis/pak-probe \
  --extract-to local/stellarblade/extracted/SB_ImprovedPerfectDefense_Extended_P

nix run .#probe-uasset -- \
  --uasset local/stellarblade/extracted/SB_ImprovedPerfectDefense_Extended_P/SB/Content/Local/Data/SkillTable.uasset \
  --uexp local/stellarblade/extracted/SB_ImprovedPerfectDefense_Extended_P/SB/Content/Local/Data/SkillTable.uexp \
  --out local/stellarblade/recon/first-pass/analysis/skilltable-uasset

nix run .#list-iostore -- \
  --utoc /data/nvme0/can/games/steamapps/steamapps/common/StellarBlade/SB/Content/Paks/pakchunk0-WindowsNoEditor.utoc \
  --out local/stellarblade/recon/first-pass/analysis/iostore-pakchunk0

nix run .#extract-iostore -- \
  --utoc /data/nvme0/can/games/steamapps/steamapps/common/StellarBlade/SB/Content/Paks/pakchunk0-WindowsNoEditor.utoc \
  --list local/stellarblade/recon/first-pass/analysis/iostore-pakchunk0/iostore-list.jsonl \
  --extract-to local/stellarblade/extracted/base-pakchunk0 \
  --out local/stellarblade/recon/first-pass/analysis/iostore-extract \
  --entry SB/Content/Local/Data/SkillTable.uasset

nix run .#scan-asset-strings -- \
  --input local/stellarblade/extracted/base-pakchunk0/SB/Content/Local/Data/SkillTable.uasset \
  --out local/stellarblade/recon/first-pass/analysis/base-skilltable-strings

nix run .#compare-skilltable -- \
  --visible-strings local/stellarblade/recon/first-pass/visible_strings.jsonl \
  --pak-entries local/stellarblade/recon/first-pass/analysis/pak-probe/entries.jsonl \
  --row-candidates local/stellarblade/recon/first-pass/analysis/skilltable-uasset/row-candidates.jsonl \
  --iostore-list local/stellarblade/recon/first-pass/analysis/iostore-pakchunk0/iostore-list.jsonl \
  --out local/stellarblade/recon/first-pass/analysis/skilltable-compare.md
```

The pak probe defaults to listing only. Extraction happens only when
`--extract-to` is supplied, and currently requires uncompressed, unencrypted
entries from the parsed pak index. The cooked asset probe validates the
`SkillTable.uasset/.uexp` package summary, name map, imports, exports, and row
name candidates; it does not decode arbitrary DataTable property values yet.

Package listing has an explicit backend boundary:

- `.pak`: supported by `probe-pak` for the current disabled local mod example.
- `.utoc`/`.ucas`: listed through the pinned `retoc` app with
  `list-iostore`; targeted single-entry extraction is available through
  `extract-iostore`.

Do not substitute visible-string hits for authoritative base-game package
listings. Use the generated `iostore-list.jsonl` before claiming ownership of
base-game packages such as `/Game/Local/Data/SkillTable`. `extract-iostore`
extracts only requested paths that are already present in that allowlist,
verifies the extracted byte count, and writes a local manifest. Io Store
`ExportBundleData` chunks are not treated as legacy `.uasset/.uexp` packages;
use `scan-asset-strings` for conservative string evidence until a validated
Zen package/DataTable parser is added.

Runtime symbol capture requires a UE4SS-compatible Bouldy V3 shim deployed in:

```text
SB/Binaries/Win64/
```

After a game run, validate that reconnaissance data was produced with:

```bash
sqlite3 '/data/nvme0/can/games/steamapps/steamapps/common/StellarBlade/SB/Binaries/Win64/Mods/StellarBladeBouldyRecon/data/recon.sqlite3' \
  'select count(*) from scan_runs; select count(*) from candidates; select count(*) from exports;'
```

## Safety

Keep all generated dumps, extracted assets, SDKs, `.usmap`, `.pak`, `.utoc`, and `.ucas` files local. Do not commit proprietary game data. Do not use this modkit to bypass anti-cheat or alter online play.
