//! Read-only offline inventory collector for a local Stellar Blade install.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_GAME_ROOT: &str = "/data/nvme0/can/games/steamapps/steamapps/common/StellarBlade";
const DEFAULT_TERMS: &[&str] = &[
    "combat", "skill", "parry", "guard", "evade", "attack", "shield", "tachy", "burst", "range",
];
const SMALL_PAK_METADATA_LIMIT: u64 = 64 * 1024 * 1024;
const MIN_VISIBLE_STRING_LEN: usize = 4;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ReconError> {
    let config = Config::parse(env::args_os().skip(1))?;
    let install = ValidatedInstall::validate(&config.game_root)?;
    let run_started_unix_seconds = unix_now()?;

    fs::create_dir_all(&config.out).map_err(|source| ReconError::Io {
        path: config.out.clone(),
        action: "create output directory",
        source,
    })?;

    let packages = collect_packages(&install.paks_dir)?;
    let loose_roots = collect_loose_roots(&install.game_root)?;
    let loose_files = collect_loose_files(&loose_roots)?;
    let metadata_sources = metadata_sources(&packages);
    let visible_strings = collect_visible_strings(&metadata_sources, &config.terms)?;
    let combat_candidates =
        collect_combat_candidates(&visible_strings, &loose_files, &config.terms);

    write_inventory(
        &config.out.join("inventory.json"),
        &install,
        &packages,
        &loose_roots,
        &loose_files,
        &config.terms,
        run_started_unix_seconds,
    )?;
    write_visible_strings(&config.out.join("visible_strings.jsonl"), &visible_strings)?;
    write_combat_candidates(
        &config.out.join("combat_candidates.jsonl"),
        &combat_candidates,
    )?;
    write_summary(
        &config.out.join("summary.md"),
        &install,
        &packages,
        &loose_files,
        &visible_strings,
        &combat_candidates,
        &metadata_sources,
    )?;

    println!(
        "wrote offline reconnaissance data to {}",
        config.out.display()
    );
    Ok(())
}

#[derive(Debug)]
struct Config {
    game_root: PathBuf,
    out: PathBuf,
    terms: Vec<String>,
}

impl Config {
    fn parse<I>(args: I) -> Result<Self, ReconError>
    where
        I: IntoIterator<Item = std::ffi::OsString>,
    {
        let mut game_root = PathBuf::from(DEFAULT_GAME_ROOT);
        let mut out = None;
        let mut terms = DEFAULT_TERMS
            .iter()
            .map(|term| term.to_ascii_lowercase())
            .collect::<Vec<_>>();

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--game-root" => {
                    let value = args.next().ok_or(ReconError::Usage(
                        "--game-root requires a path argument".to_owned(),
                    ))?;
                    game_root = PathBuf::from(value);
                }
                "--out" => {
                    let value = args.next().ok_or(ReconError::Usage(
                        "--out requires a path argument".to_owned(),
                    ))?;
                    out = Some(PathBuf::from(value));
                }
                "--terms" => {
                    let value = args.next().ok_or(ReconError::Usage(
                        "--terms requires a comma-separated argument".to_owned(),
                    ))?;
                    terms = parse_terms(&value.to_string_lossy())?;
                }
                "-h" | "--help" => return Err(ReconError::Usage(usage())),
                other => {
                    return Err(ReconError::Usage(format!(
                        "unknown argument: {other}\n\n{}",
                        usage()
                    )));
                }
            }
        }

        let out = out.ok_or(ReconError::Usage(format!(
            "--out is required\n\n{}",
            usage()
        )))?;

        Ok(Self {
            game_root,
            out,
            terms,
        })
    }
}

fn usage() -> String {
    format!(
        "usage: offline_recon --out local/stellarblade/recon/first-pass [--game-root {DEFAULT_GAME_ROOT}] [--terms combat,skill,parry]"
    )
}

fn parse_terms(value: &str) -> Result<Vec<String>, ReconError> {
    let mut seen = BTreeSet::new();
    let mut terms = Vec::new();
    for term in value.split(',') {
        let term = term.trim().to_ascii_lowercase();
        if term.is_empty() {
            continue;
        }
        if seen.insert(term.clone()) {
            terms.push(term);
        }
    }
    if terms.is_empty() {
        return Err(ReconError::Usage(
            "--terms must contain at least one non-empty term".to_owned(),
        ));
    }
    Ok(terms)
}

#[path = "../../tools_support/mod.rs"]
mod tools_support;

include!("install.rs");
include!("packages.rs");
include!("strings.rs");
include!("candidates.rs");
include!("reports.rs");
include!("fs.rs");
include!("errors.rs");
include!("tests.rs");
