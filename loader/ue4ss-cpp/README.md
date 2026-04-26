# UE4SS C++ Adapter

This directory contains the C++ side of the Stellar Blade Bouldy reconnaissance collector.

Pinned upstream template:

- `UE4SS-RE/UE4SSCPPTemplate`
- commit `515d49e0ac98c9e985f07f27150d53da89c5c56f`

The checked-in code is split into:

- `src/bouldy_adapter.*`: Bouldy V3 ABI structs, filtering, candidate visitation, and DLL lifecycle helpers.
- `src/recon_database.*`: SQLite persistence for scan runs, raw candidates, and Rust export payloads.
- `src/ue4ss_entry.cpp`: UE4SS integration seam. It is intentionally guarded behind `BOULDY_WITH_UE4SS` because UE4SS headers are not vendored in this repository.
- `tests/recon_tests.cpp`: local tests using fake candidates and the vendored SQLite amalgamation.

Deploy the compiled UE4SS mod DLL as:

```text
StellarBlade/Binaries/Win64/Mods/StellarBladeBouldyRecon/dlls/main.dll
```

Place `stellar_blade_bouldy_mod.dll` beside it or adjust the DLL name in the adapter.
