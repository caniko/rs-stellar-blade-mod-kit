//! Analyze the disabled local mod example from offline visible string output.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DEFAULT_INPUT: &str = "local/stellarblade/recon/first-pass/visible_strings.jsonl";
const DEFAULT_OUT: &str = "local/stellarblade/recon/first-pass/analysis/mod-example.md";
const DEFAULT_MOD_PACKAGE: &str = "SB_ImprovedPerfectDefense_Extended_P";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ModAnalyzeError> {
    let config = Config::parse(env::args_os().skip(1))?;
    require_file(
        &config.input,
        "visible string JSONL is required to compare the disabled mod metadata with base packages",
        "run offline_recon against the local Stellar Blade install",
        "test -f local/stellarblade/recon/first-pass/visible_strings.jsonl",
    )?;

    let records = read_records(&config.input)?;
    let analysis = ModExampleAnalysis::build(records, &config.mod_package);
    if analysis.mod_records.is_empty() {
        return Err(ModAnalyzeError::MissingRequired {
            path: config.input,
            why_required: format!(
                "no records were found for source_package `{}`",
                config.mod_package
            ),
            upstream_producer: "SB/Content/Paks/~mods-off disabled mod triplet and offline_recon"
                .to_owned(),
            regenerate_command: "confirm the disabled mod triplet exists, then rerun offline_recon"
                .to_owned(),
            validation_command: format!(
                "rg '{}' local/stellarblade/recon/first-pass/visible_strings.jsonl",
                config.mod_package
            ),
        });
    }

    write_report(&config.out, &analysis)?;
    println!(
        "wrote disabled mod example analysis to {}",
        config.out.display()
    );
    Ok(())
}

#[derive(Debug)]
struct Config {
    input: PathBuf,
    out: PathBuf,
    mod_package: String,
}

impl Config {
    fn parse<I>(args: I) -> Result<Self, ModAnalyzeError>
    where
        I: IntoIterator<Item = std::ffi::OsString>,
    {
        let mut input = PathBuf::from(DEFAULT_INPUT);
        let mut out = PathBuf::from(DEFAULT_OUT);
        let mut mod_package = DEFAULT_MOD_PACKAGE.to_owned();

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--input" => {
                    let value = args.next().ok_or(ModAnalyzeError::Usage(
                        "--input requires a path argument".to_owned(),
                    ))?;
                    input = PathBuf::from(value);
                }
                "--out" => {
                    let value = args.next().ok_or(ModAnalyzeError::Usage(
                        "--out requires a path argument".to_owned(),
                    ))?;
                    out = PathBuf::from(value);
                }
                "--mod-package" => {
                    let value = args.next().ok_or(ModAnalyzeError::Usage(
                        "--mod-package requires a package name".to_owned(),
                    ))?;
                    mod_package = value.to_string_lossy().into_owned();
                }
                "-h" | "--help" => return Err(ModAnalyzeError::Usage(usage())),
                other => {
                    return Err(ModAnalyzeError::Usage(format!(
                        "unknown argument: {other}\n\n{}",
                        usage()
                    )));
                }
            }
        }

        Ok(Self {
            input,
            out,
            mod_package,
        })
    }
}

fn usage() -> String {
    format!(
        "usage: analyze_mod_example [--input {DEFAULT_INPUT}] [--out {DEFAULT_OUT}] [--mod-package {DEFAULT_MOD_PACKAGE}]"
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VisibleStringRecord {
    source_package: String,
    source_kind: String,
    source_path: String,
    byte_offset: u64,
    raw_string: String,
}

#[derive(Debug)]
struct ModExampleAnalysis {
    mod_package: String,
    mod_records: Vec<VisibleStringRecord>,
    unique_mod_strings: BTreeSet<String>,
    base_strings: BTreeSet<String>,
    categories: BTreeMap<String, BTreeSet<String>>,
    base_overlaps: BTreeSet<String>,
    kind_counts: BTreeMap<String, usize>,
}

impl ModExampleAnalysis {
    fn build(records: Vec<VisibleStringRecord>, mod_package: &str) -> Self {
        let mut mod_records = Vec::new();
        let mut unique_mod_strings = BTreeSet::new();
        let mut base_strings = BTreeSet::new();
        let mut categories = BTreeMap::<String, BTreeSet<String>>::new();
        let mut kind_counts = BTreeMap::new();

        for record in records {
            if record.source_package == mod_package {
                *kind_counts.entry(record.source_kind.clone()).or_insert(0) += 1;
                unique_mod_strings.insert(record.raw_string.clone());
                for category in classify_string(&record.raw_string) {
                    categories
                        .entry(category.to_owned())
                        .or_default()
                        .insert(record.raw_string.clone());
                }
                mod_records.push(record);
            } else {
                base_strings.insert(record.raw_string);
            }
        }

        let base_overlaps = unique_mod_strings
            .intersection(&base_strings)
            .cloned()
            .collect::<BTreeSet<_>>();

        Self {
            mod_package: mod_package.to_owned(),
            mod_records,
            unique_mod_strings,
            base_strings,
            categories,
            base_overlaps,
            kind_counts,
        }
    }
}

fn classify_string(value: &str) -> Vec<&'static str> {
    let lowered = value.to_ascii_lowercase();
    let mut categories = Vec::new();
    if value.starts_with("/Game/") {
        categories.push("asset_paths");
    }
    if value.starts_with("/Script/") {
        categories.push("script_paths");
    }
    if lowered.ends_with(".uasset") || lowered.ends_with(".umap") {
        categories.push("asset_names");
    }
    if value.contains("Table") || value == "DataTable" || value.ends_with("Property") {
        categories.push("table_schema");
    }
    if contains_any(
        &lowered,
        &[
            "skill", "parry", "guard", "evade", "attack", "shield", "combat", "combo", "tachy",
            "burst",
        ],
    ) {
        categories.push("combat_terms");
    }
    if value.starts_with('b') && value.chars().nth(1).is_some_and(char::is_uppercase) {
        categories.push("bool_like_fields");
    }
    if value.starts_with("ESB") || value.starts_with("Eve") || value.starts_with("M_") {
        categories.push("game_symbols");
    }
    categories
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

include!("input.rs");
include!("report.rs");
include!("util.rs");
include!("errors.rs");
include!("tests.rs");
