//! Targeted Io Store extraction through a pinned retoc backend.

use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_UTOC: &str = "/data/nvme0/can/games/steamapps/steamapps/common/StellarBlade/SB/Content/Paks/pakchunk0-WindowsNoEditor.utoc";
const DEFAULT_LIST: &str =
    "local/stellarblade/recon/first-pass/analysis/iostore-pakchunk0/iostore-list.jsonl";
const DEFAULT_OUT: &str = "local/stellarblade/recon/first-pass/analysis/iostore-extract";
const DEFAULT_EXTRACT_TO: &str = "local/stellarblade/extracted/base-pakchunk0";
const DEFAULT_ENTRY: &str = "SB/Content/Local/Data/SkillTable.uasset";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ExtractError> {
    let config = Config::parse(env::args_os().skip(1))?;
    require_file(
        &config.utoc,
        "targeted Io Store extraction requires the source .utoc",
        "Stellar Blade install under SB/Content/Paks",
        "test -f '<path>.utoc'",
    )?;
    require_file(
        &config.utoc.with_extension("ucas"),
        "retoc requires the matching .ucas payload beside the .utoc",
        "Stellar Blade install under SB/Content/Paks",
        "test -f '<path>.ucas'",
    )?;
    require_file(
        &config.list,
        "the retoc-generated Io Store list is the extraction allowlist",
        "list_iostore output",
        "nix run .#list-iostore -- --utoc '<path>.utoc' --out local/stellarblade/recon/first-pass/analysis/iostore-pakchunk0",
    )?;

    fs::create_dir_all(&config.out).map_err(|source| ExtractError::Io {
        path: config.out.clone(),
        action: "create report directory",
        source,
    })?;
    fs::create_dir_all(&config.extract_to).map_err(|source| ExtractError::Io {
        path: config.extract_to.clone(),
        action: "create extraction directory",
        source,
    })?;

    let entries = read_iostore_entries(&config.list)?;
    let selected = select_entries(&entries, &config.entries)?;
    let mut extracted = Vec::new();
    for entry in selected {
        extracted.push(extract_entry(&config, entry)?);
    }
    write_manifest(&config.out.join("manifest.json"), &config, &extracted)?;
    write_summary(&config.out.join("summary.md"), &config, &extracted)?;

    println!(
        "wrote Io Store extraction report to {}",
        config.out.display()
    );
    Ok(())
}

#[derive(Debug)]
struct Config {
    utoc: PathBuf,
    list: PathBuf,
    out: PathBuf,
    extract_to: PathBuf,
    entries: Vec<String>,
    retoc: PathBuf,
}

impl Config {
    fn parse<I>(args: I) -> Result<Self, ExtractError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut utoc = PathBuf::from(DEFAULT_UTOC);
        let mut list = PathBuf::from(DEFAULT_LIST);
        let mut out = PathBuf::from(DEFAULT_OUT);
        let mut extract_to = PathBuf::from(DEFAULT_EXTRACT_TO);
        let mut entries = Vec::new();
        let mut retoc = PathBuf::from("retoc");
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--utoc" => {
                    utoc = PathBuf::from(args.next().ok_or(ExtractError::Usage(
                        "--utoc requires a path argument".to_owned(),
                    ))?);
                }
                "--list" => {
                    list = PathBuf::from(args.next().ok_or(ExtractError::Usage(
                        "--list requires a path argument".to_owned(),
                    ))?);
                }
                "--out" => {
                    out = PathBuf::from(
                        args.next()
                            .ok_or(ExtractError::Usage("--out requires a path".to_owned()))?,
                    );
                }
                "--extract-to" => {
                    extract_to = PathBuf::from(args.next().ok_or(ExtractError::Usage(
                        "--extract-to requires a path argument".to_owned(),
                    ))?);
                }
                "--entry" => {
                    let entry = args.next().ok_or(ExtractError::Usage(
                        "--entry requires an Io Store entry path or suffix".to_owned(),
                    ))?;
                    entries.push(entry.to_string_lossy().into_owned());
                }
                "--retoc" => {
                    retoc = PathBuf::from(args.next().ok_or(ExtractError::Usage(
                        "--retoc requires a command path".to_owned(),
                    ))?);
                }
                "-h" | "--help" => return Err(ExtractError::Usage(usage())),
                other => {
                    return Err(ExtractError::Usage(format!(
                        "unknown argument: {other}\n\n{}",
                        usage()
                    )));
                }
            }
        }
        if entries.is_empty() {
            entries.push(DEFAULT_ENTRY.to_owned());
        }
        Ok(Self {
            utoc,
            list,
            out,
            extract_to,
            entries,
            retoc,
        })
    }
}

fn usage() -> String {
    format!(
        "usage: extract_iostore [--utoc {DEFAULT_UTOC}] [--list {DEFAULT_LIST}] [--out {DEFAULT_OUT}] [--extract-to {DEFAULT_EXTRACT_TO}] [--entry {DEFAULT_ENTRY}] [--retoc retoc]"
    )
}

#[derive(Clone, Debug)]
struct IoStoreEntry {
    container: String,
    chunk_id: String,
    content_hash: String,
    package_id: Option<String>,
    chunk_type: String,
    size: u64,
    path: String,
}

#[derive(Debug)]
struct ExtractedEntry {
    entry: IoStoreEntry,
    output_path: PathBuf,
    byte_count: u64,
    command: String,
}

#[path = "../../tools_support/mod.rs"]
mod tools_support;

include!("entries.rs");
include!("extraction.rs");
include!("reports.rs");
include!("json_fields.rs");
include!("util.rs");
include!("errors.rs");
