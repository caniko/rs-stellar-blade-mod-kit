//! List Unreal Io Store containers through a pinned retoc backend.

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_UTOC: &str = "/data/nvme0/can/games/steamapps/steamapps/common/StellarBlade/SB/Content/Paks/pakchunk0-WindowsNoEditor.utoc";
const DEFAULT_OUT: &str = "local/stellarblade/recon/first-pass/analysis/iostore-pakchunk0";
const DEFAULT_TERMS: &[&str] = &[
    "SkillTable",
    "SBSkillTableProperty",
    "JustParry1",
    "ComboParry",
    "GuardSkill",
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

fn run() -> Result<(), IoStoreError> {
    let config = Config::parse(env::args_os().skip(1))?;
    require_file(
        &config.utoc,
        "Io Store listing requires the .utoc table-of-contents file",
        "Stellar Blade install under SB/Content/Paks",
        "test -f '<path>.utoc'",
    )?;
    let ucas = config
        .ucas
        .clone()
        .unwrap_or_else(|| config.utoc.with_extension("ucas"));
    require_file(
        &ucas,
        "retoc opens the matching .ucas payload beside the .utoc while building the directory index",
        "Stellar Blade install under SB/Content/Paks",
        "test -f '<path>.ucas'",
    )?;

    fs::create_dir_all(&config.out).map_err(|source| IoStoreError::Io {
        path: config.out.clone(),
        action: "create output directory",
        source,
    })?;

    let info = run_retoc(&config.retoc, ["info".into(), config.utoc.clone().into()])?;
    let list = run_retoc(
        &config.retoc,
        [
            "list".into(),
            "--hash".into(),
            "--package".into(),
            "--size".into(),
            "--path".into(),
            config.utoc.clone().into(),
        ],
    )?;
    let entries = parse_list_output(&list.stdout)?;
    let focus_hits = focus_hits(&entries, &config.terms);

    write_text(&config.out.join("retoc-info.txt"), &info.stdout)?;
    write_text(&config.out.join("retoc-list.txt"), &list.stdout)?;
    write_list_jsonl(&config.out.join("iostore-list.jsonl"), &config, &entries)?;
    write_focus_jsonl(
        &config.out.join("iostore-focus-hits.jsonl"),
        &config,
        &focus_hits,
    )?;
    write_summary(
        &config.out.join("summary.md"),
        &config,
        &ucas,
        &info,
        &list,
        &entries,
        &focus_hits,
    )?;

    println!("wrote Io Store listing to {}", config.out.display());
    Ok(())
}

#[derive(Debug)]
struct Config {
    utoc: PathBuf,
    ucas: Option<PathBuf>,
    out: PathBuf,
    terms: Vec<String>,
    retoc: PathBuf,
}

impl Config {
    fn parse<I>(args: I) -> Result<Self, IoStoreError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut utoc = PathBuf::from(DEFAULT_UTOC);
        let mut ucas = None;
        let mut out = PathBuf::from(DEFAULT_OUT);
        let mut terms = DEFAULT_TERMS
            .iter()
            .map(|term| (*term).to_owned())
            .collect();
        let mut retoc = PathBuf::from("retoc");
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--utoc" => {
                    utoc = PathBuf::from(args.next().ok_or(IoStoreError::Usage(
                        "--utoc requires a path argument".to_owned(),
                    ))?);
                }
                "--ucas" => {
                    ucas = Some(PathBuf::from(args.next().ok_or(IoStoreError::Usage(
                        "--ucas requires a path argument".to_owned(),
                    ))?));
                }
                "--out" => {
                    out = PathBuf::from(
                        args.next()
                            .ok_or(IoStoreError::Usage("--out requires a path".to_owned()))?,
                    );
                }
                "--terms" => {
                    let value = args.next().ok_or(IoStoreError::Usage(
                        "--terms requires a comma-separated list".to_owned(),
                    ))?;
                    terms = parse_terms(&value.to_string_lossy());
                }
                "--retoc" => {
                    retoc = PathBuf::from(args.next().ok_or(IoStoreError::Usage(
                        "--retoc requires a command path".to_owned(),
                    ))?);
                }
                "-h" | "--help" => return Err(IoStoreError::Usage(usage())),
                other => {
                    return Err(IoStoreError::Usage(format!(
                        "unknown argument: {other}\n\n{}",
                        usage()
                    )));
                }
            }
        }
        if terms.is_empty() {
            return Err(IoStoreError::Usage(
                "--terms must include at least one non-empty term".to_owned(),
            ));
        }
        Ok(Self {
            utoc,
            ucas,
            out,
            terms,
            retoc,
        })
    }
}

fn usage() -> String {
    format!(
        "usage: list_iostore [--utoc {DEFAULT_UTOC}] [--ucas matching.ucas] [--out {DEFAULT_OUT}] [--terms SkillTable,SBSkillTableProperty] [--retoc retoc]"
    )
}

#[path = "../../tools_support/mod.rs"]
mod tools_support;

include!("retoc.rs");
include!("reports.rs");
include!("util.rs");
include!("errors.rs");
include!("tests.rs");
