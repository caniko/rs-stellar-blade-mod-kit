# Stellar Blade Bouldy Modkit vision

This repository is a Bouldy-based reconnaissance modkit for the PC release of
Stellar Blade, plus the offline tooling that reads the game's cooked content.
Its first job is to discover combat-related Unreal symbols through Bouldy's
generic discovery API and export candidate records that later analysis can work
from. Its users are the people doing that analysis, on one known local game
install.

## Problem

Modding a packaged Unreal game starts from almost no ground truth. Hook targets,
offsets, and object paths have to be found before anything can be patched, and
the packaged content (Io Store `.utoc`/`.ucas`, paks, `.uasset`/`.uexp`) hides
the metadata that would identify them. Guessing targets from memory or from the
public modding scene is not evidence.

## Principles

- All concrete hook targets, offsets, and object paths must come from captured
  discovery output. Nothing about the game is hardcoded ahead of observation.
- Candidate records are metadata only, schema-versioned
  (`schema_version: 2`), and exported on a named channel
  (`stellar_blade.combat_recon`) as stable JSON-style records.
- Every scan is persisted for later analysis: the C++ adapter writes `scan_runs`,
  `candidates`, and Rust-filtered `exports` into a local SQLite database.
- Package inventories, extracted assets, dumps, SDKs, `.usmap` files, and all
  generated game data stay local and untracked.
- The build is strict about unsafe code: `unsafe_op_in_unsafe_fn` and
  `undocumented_unsafe_blocks` are denied workspace lints, and `missing_docs`
  warns.
- Bouldy is a pinned submodule and UE4SS is pinned through the vendored Bouldy
  flake, so the runtime adapter and the framework move together.

## Scope

- A UE4SS C++ adapter (`main.dll`) and a Rust Bouldy runtime module
  (`stellar_blade_bouldy_mod.dll`) that request and filter discovery candidates.
- Runtime installation and export helpers that lay out `Mods/StellarBladeBouldyRecon`
  and preserve an existing `mods.txt`.
- Offline probing tools: `probe-pak`, `probe-uasset`, `list-iostore`,
  `extract-iostore`, `scan-asset-strings`, `compare-skilltable`,
  `analyze-candidates`, and `analyze-mod-example`.
- An `offline-recon` first pass that turns a local game root into candidate and
  string inventories.

## Non-goals

The README states these directly: it does not patch gameplay, register
Stellar Blade-specific hooks, ship generated SDKs, include extracted game
assets, bypass anti-cheat, or support online-play modification. Runtime scope is
reconnaissance only.

## Direction

Recent commits add the SQLite-backed recon adapter and split the offline probe
tools into a separate workspace crate with dedicated error types. There is no
roadmap file; the direction visible in the code is more faithful offline
evidence before any hook work.

## Place in the fleet

No canix runtime consumer was found. Canix tracks the repository in the atlas
`simit-updater` registry.
