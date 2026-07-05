//! Conservative cooked UE4 asset probe for the extracted Stellar Blade SkillTable.

use std::collections::BTreeSet;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DEFAULT_UASSET: &str = "local/stellarblade/extracted/SB_ImprovedPerfectDefense_Extended_P/SB/Content/Local/Data/SkillTable.uasset";
const DEFAULT_UEXP: &str = "local/stellarblade/extracted/SB_ImprovedPerfectDefense_Extended_P/SB/Content/Local/Data/SkillTable.uexp";
const DEFAULT_OUT: &str = "local/stellarblade/recon/first-pass/analysis/skilltable-uasset";
const PACKAGE_TAG: u32 = 0x9e2a83c1;
const ROW_TERMS: &[&str] = &[
    "Parry", "Guard", "Evade", "Attack", "Skill", "Shield", "Combo", "Tachy", "Burst",
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

fn run() -> Result<(), UassetError> {
    let config = Config::parse(env::args_os().skip(1))?;
    require_file(
        &config.uasset,
        "the extracted .uasset is required for package summary and name/import/export probing",
        "probe_pak extraction from the disabled mod pak",
        "test -f local/stellarblade/extracted/SB_ImprovedPerfectDefense_Extended_P/SB/Content/Local/Data/SkillTable.uasset",
    )?;
    require_file(
        &config.uexp,
        "the paired .uexp is required to validate export serial offsets and scan external payload references",
        "probe_pak extraction from the disabled mod pak",
        "test -f local/stellarblade/extracted/SB_ImprovedPerfectDefense_Extended_P/SB/Content/Local/Data/SkillTable.uexp",
    )?;

    let package = parse_package(&config.uasset, &config.uexp)?;
    fs::create_dir_all(&config.out).map_err(|source| UassetError::Io {
        path: config.out.clone(),
        action: "create output directory",
        source,
    })?;
    write_summary(&config.out.join("summary.md"), &package)?;
    write_names(&config.out.join("names.jsonl"), &package)?;
    write_imports(&config.out.join("imports.jsonl"), &package)?;
    write_exports(&config.out.join("exports.jsonl"), &package)?;
    write_row_candidates(&config.out.join("row-candidates.jsonl"), &package)?;

    println!("wrote uasset probe to {}", config.out.display());
    Ok(())
}

#[derive(Debug)]
struct Config {
    uasset: PathBuf,
    uexp: PathBuf,
    out: PathBuf,
}

impl Config {
    fn parse<I>(args: I) -> Result<Self, UassetError>
    where
        I: IntoIterator<Item = std::ffi::OsString>,
    {
        let mut uasset = PathBuf::from(DEFAULT_UASSET);
        let mut uexp = PathBuf::from(DEFAULT_UEXP);
        let mut out = PathBuf::from(DEFAULT_OUT);
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--uasset" => {
                    let value = args.next().ok_or(UassetError::Usage(
                        "--uasset requires a path argument".to_owned(),
                    ))?;
                    uasset = PathBuf::from(value);
                }
                "--uexp" => {
                    let value = args.next().ok_or(UassetError::Usage(
                        "--uexp requires a path argument".to_owned(),
                    ))?;
                    uexp = PathBuf::from(value);
                }
                "--out" => {
                    let value = args.next().ok_or(UassetError::Usage(
                        "--out requires a path argument".to_owned(),
                    ))?;
                    out = PathBuf::from(value);
                }
                "-h" | "--help" => return Err(UassetError::Usage(usage())),
                other => {
                    return Err(UassetError::Usage(format!(
                        "unknown argument: {other}\n\n{}",
                        usage()
                    )));
                }
            }
        }
        Ok(Self { uasset, uexp, out })
    }
}

fn usage() -> String {
    format!("usage: probe_uasset [--uasset {DEFAULT_UASSET}] [--uexp {DEFAULT_UEXP}] [--out {DEFAULT_OUT}]")
}

#[derive(Debug)]
struct PackageProbe {
    uasset_path: PathBuf,
    uexp_path: PathBuf,
    uasset_size: u64,
    uexp_size: u64,
    summary: PackageSummary,
    names: Vec<NameEntry>,
    imports: Vec<ImportEntry>,
    exports: Vec<ExportEntry>,
    external_payload_strings: Vec<String>,
}

#[derive(Debug)]
struct PackageSummary {
    tag: u32,
    legacy_file_version: i32,
    total_header_size: i32,
    package_name: String,
    package_flags: u32,
    name_count: i32,
    name_offset: i32,
    export_count: i32,
    export_offset: i32,
    import_count: i32,
    import_offset: i32,
    depends_offset: i32,
}

#[derive(Debug)]
struct NameEntry {
    index: usize,
    value: String,
    flags: u32,
}

#[derive(Debug)]
struct ImportEntry {
    index: usize,
    class_package: String,
    class_name: String,
    outer_index: i32,
    object_name: String,
}

#[derive(Debug)]
struct ExportEntry {
    index: usize,
    class_index: i32,
    super_index: i32,
    outer_index: i32,
    object_name: String,
    object_flags: u32,
    serial_size: i64,
    serial_offset: i64,
}

#[path = "../../tools_support/mod.rs"]
mod tools_support;

include!("parser.rs");
include!("reports.rs");
include!("analysis.rs");
include!("cursor.rs");
include!("util.rs");
include!("errors.rs");
include!("tests.rs");
