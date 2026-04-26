# Stellar Blade Bouldy Modkit

This repository contains a Bouldy-based reconnaissance modkit for the PC release of Stellar Blade. The first version discovers combat-related Unreal symbols through a generic Bouldy discovery API and exports candidate records for later analysis.

It does not patch gameplay, register Stellar Blade-specific hooks, ship generated SDKs, include extracted game assets, bypass anti-cheat, or support online-play modification.

## Assumptions

- Target game: Stellar Blade PC.
- Engine assumption: Unreal Engine 4.26 or 4.26.2.
- Packaging assumption: Io Store-style cooked content (`.utoc`/`.ucas`) with pak-side metadata, based on current public PC modding practice.
- Loader assumption: UE4SS-compatible Bouldy V3 shim deployed in the game's Win64 binary directory.
- Runtime scope: reconnaissance only. All concrete hook targets, offsets, and object paths must come from captured discovery output.

## Build

Native validation:

```bash
nix develop
cargo test
cargo clippy --all-targets -- -D warnings
```

Windows DLL build:

```bash
nix develop .#windows
cargo build --release --target x86_64-pc-windows-gnu
```

The DLL is produced at:

```text
target/x86_64-pc-windows-gnu/release/stellar_blade_bouldy_mod.dll
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
```

Copy or rename `stellar_blade_bouldy_mod.dll` to `Mods/StellarBladeBouldyRecon/dlls/main.dll`. Enable it with:

```text
StellarBladeBouldyRecon : 1
```

Place the line before UE4SS's built-in keybind section in `mods.txt` if load order matters.

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

## Safety

Keep all generated dumps, extracted assets, SDKs, `.usmap`, `.pak`, `.utoc`, and `.ucas` files local. Do not commit proprietary game data. Do not use this modkit to bypass anti-cheat or alter online play.
