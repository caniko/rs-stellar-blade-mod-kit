//! Join SkillTable mod evidence, cooked asset metadata, and base-game string candidates.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DEFAULT_VISIBLE_STRINGS: &str = "local/stellarblade/recon/first-pass/visible_strings.jsonl";
const DEFAULT_PAK_ENTRIES: &str =
    "local/stellarblade/recon/first-pass/analysis/pak-probe/entries.jsonl";
const DEFAULT_ROW_CANDIDATES: &str =
    "local/stellarblade/recon/first-pass/analysis/skilltable-uasset/row-candidates.jsonl";
const DEFAULT_IOSTORE_LIST: &str =
    "local/stellarblade/recon/first-pass/analysis/iostore-pakchunk0/iostore-list.jsonl";
const DEFAULT_OUT: &str = "local/stellarblade/recon/first-pass/analysis/skilltable-compare.md";
const MOD_PACKAGE: &str = "SB_ImprovedPerfectDefense_Extended_P";
const FOCUS_TERMS: &[&str] = &[
    "SkillTable",
    "SBSkillTableProperty",
    "JustParry1",
    "ComboParry",
    "GuardSkill",
    "/Game/Local/Data/SkillTable",
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), CompareError> {
    let config = Config::parse(env::args_os().skip(1))?;
    for (path, why, validation) in [
        (
            &config.visible_strings,
            "base-game visible string candidates are required for overlap checks",
            "test -f local/stellarblade/recon/first-pass/visible_strings.jsonl",
        ),
        (
            &config.pak_entries,
            "disabled mod pak entries are required to identify confirmed package contents",
            "test -f local/stellarblade/recon/first-pass/analysis/pak-probe/entries.jsonl",
        ),
        (
            &config.row_candidates,
            "SkillTable row candidates are required from probe_uasset",
            "test -f local/stellarblade/recon/first-pass/analysis/skilltable-uasset/row-candidates.jsonl",
        ),
    ] {
        require_file(path, why, validation)?;
    }

    let pak_entries = read_pak_entries(&config.pak_entries)?;
    let rows = read_values(&config.row_candidates, "value")?;
    let visible = read_visible_strings(&config.visible_strings)?;
    let iostore_entries = match &config.iostore_list {
        Some(path) => {
            require_file(
                path,
                "Io Store list entries are required when --iostore-list is supplied",
                "test -f local/stellarblade/recon/first-pass/analysis/iostore-pakchunk0/iostore-list.jsonl",
            )?;
            read_iostore_entries(path)?
        }
        None => Vec::new(),
    };
    let report = CompareReport::build(pak_entries, rows, visible, iostore_entries);
    write_report(&config.out, &report)?;

    println!("wrote SkillTable comparison to {}", config.out.display());
    Ok(())
}

#[derive(Debug)]
struct Config {
    visible_strings: PathBuf,
    pak_entries: PathBuf,
    row_candidates: PathBuf,
    iostore_list: Option<PathBuf>,
    out: PathBuf,
}

impl Config {
    fn parse<I>(args: I) -> Result<Self, CompareError>
    where
        I: IntoIterator<Item = std::ffi::OsString>,
    {
        let mut visible_strings = PathBuf::from(DEFAULT_VISIBLE_STRINGS);
        let mut pak_entries = PathBuf::from(DEFAULT_PAK_ENTRIES);
        let mut row_candidates = PathBuf::from(DEFAULT_ROW_CANDIDATES);
        let mut iostore_list = None;
        let mut out = PathBuf::from(DEFAULT_OUT);
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--visible-strings" => {
                    visible_strings = PathBuf::from(args.next().ok_or(CompareError::Usage(
                        "--visible-strings requires a path".to_owned(),
                    ))?);
                }
                "--pak-entries" => {
                    pak_entries = PathBuf::from(args.next().ok_or(CompareError::Usage(
                        "--pak-entries requires a path".to_owned(),
                    ))?);
                }
                "--row-candidates" => {
                    row_candidates = PathBuf::from(args.next().ok_or(CompareError::Usage(
                        "--row-candidates requires a path".to_owned(),
                    ))?);
                }
                "--iostore-list" => {
                    iostore_list = Some(PathBuf::from(args.next().ok_or(CompareError::Usage(
                        "--iostore-list requires a path".to_owned(),
                    ))?));
                }
                "--out" => {
                    out = PathBuf::from(
                        args.next()
                            .ok_or(CompareError::Usage("--out requires a path".to_owned()))?,
                    );
                }
                "-h" | "--help" => return Err(CompareError::Usage(usage())),
                other => {
                    return Err(CompareError::Usage(format!(
                        "unknown argument: {other}\n\n{}",
                        usage()
                    )));
                }
            }
        }
        Ok(Self {
            visible_strings,
            pak_entries,
            row_candidates,
            iostore_list,
            out,
        })
    }
}

fn usage() -> String {
    format!(
        "usage: compare_skilltable [--visible-strings {DEFAULT_VISIBLE_STRINGS}] [--pak-entries {DEFAULT_PAK_ENTRIES}] [--row-candidates {DEFAULT_ROW_CANDIDATES}] [--iostore-list {DEFAULT_IOSTORE_LIST}] [--out {DEFAULT_OUT}]"
    )
}

#[derive(Debug)]
struct PakEntry {
    path: String,
    size: u64,
    encrypted: bool,
}

#[derive(Clone, Debug)]
struct VisibleStringRecord {
    source_package: String,
    source_kind: String,
    raw_string: String,
}

#[derive(Clone, Debug)]
struct IoStoreEntry {
    container: String,
    chunk_type: String,
    size: u64,
    path: Option<String>,
}

#[derive(Debug)]
struct CompareReport {
    pak_entries: Vec<PakEntry>,
    row_candidates: BTreeSet<String>,
    focus_hits: BTreeMap<String, Vec<VisibleStringRecord>>,
    iostore_focus_hits: BTreeMap<String, Vec<IoStoreEntry>>,
    base_overlap: BTreeSet<String>,
    unresolved: Vec<String>,
}

impl CompareReport {
    fn build(
        pak_entries: Vec<PakEntry>,
        row_candidates: BTreeSet<String>,
        visible: Vec<VisibleStringRecord>,
        iostore_entries: Vec<IoStoreEntry>,
    ) -> Self {
        let mut focus_hits = BTreeMap::<String, Vec<VisibleStringRecord>>::new();
        let mut base_strings = BTreeSet::new();
        for record in visible {
            if record.source_package != MOD_PACKAGE {
                base_strings.insert(record.raw_string.clone());
                for term in FOCUS_TERMS {
                    if record.raw_string.contains(term) {
                        focus_hits
                            .entry((*term).to_owned())
                            .or_default()
                            .push(record.clone());
                    }
                }
            }
        }

        let base_overlap = row_candidates
            .intersection(&base_strings)
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut iostore_focus_hits = BTreeMap::<String, Vec<IoStoreEntry>>::new();
        for entry in iostore_entries {
            let path = entry.path.as_deref().unwrap_or_default();
            for term in FOCUS_TERMS {
                if path.contains(term) {
                    iostore_focus_hits
                        .entry((*term).to_owned())
                        .or_default()
                        .push(entry.clone());
                }
            }
        }

        let mut unresolved = vec![
            "Exact DataTable row values are still unresolved; current parser validates package metadata, names, imports, exports, and row-name candidates only.".to_owned(),
            "Runtime reflection for SBSkillTableProperty remains blocked until UE4SS headers and collect_unreal_candidates() are implemented.".to_owned(),
        ];
        if iostore_focus_hits.is_empty() {
            unresolved.push(
                "Base-game /Game/Local/Data/SkillTable package ownership still requires an Io Store list hit or a targeted trusted extraction backend."
                    .to_owned(),
            );
        }

        Self {
            pak_entries,
            row_candidates,
            focus_hits,
            iostore_focus_hits,
            base_overlap,
            unresolved,
        }
    }
}

include!("inputs.rs");
include!("report.rs");
include!("json_fields.rs");
include!("util.rs");
include!("errors.rs");
include!("tests.rs");
