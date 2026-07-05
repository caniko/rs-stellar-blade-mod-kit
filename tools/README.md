# Tools

The committed tooling currently lives in the `modkit-tools` workspace crate
under `crates/modkit-tools/src/bin` and is exposed through flake apps:

- `offline-recon`: local package and loose-file inventory.
- `analyze-candidates`: grouped combat candidate reports.
- `analyze-mod-example`: disabled mod metadata report.
- `probe-pak`: minimal Unreal Pak listing and guarded local extraction.
- `probe-uasset`: conservative cooked `SkillTable.uasset/.uexp` probe.
- `list-iostore`: list-only `.utoc` directory-index report through pinned
  `retoc`.
- `extract-iostore`: allowlist-based targeted `.utoc` chunk extraction
  through pinned `retoc`.
- `scan-asset-strings`: visible string evidence report for targeted
  extracted cooked asset blobs.
- `compare-skilltable`: joined SkillTable evidence report.

`probe-pak` is deliberately scoped to legacy `.pak` files. Use
`list-iostore` for base-game `.utoc` listing. Use `extract-iostore` only
for specific entries already present in the generated `iostore-list.jsonl`; it
does not perform broad package extraction.

Generated reports, extracted cooked assets, SDK dumps, package files, mapping
files, and SQLite databases must stay under ignored local paths such as
`local/`. Do not commit proprietary game data or scripts that require
redistributed game assets, SDK dumps, encryption keys, or extracted content.
