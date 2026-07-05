//! String scanner for targeted extracted cooked asset blobs.

use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DEFAULT_INPUT: &str =
    "local/stellarblade/extracted/base-pakchunk0/SB/Content/Local/Data/SkillTable.uasset";
const DEFAULT_OUT: &str = "local/stellarblade/recon/first-pass/analysis/base-skilltable-strings";
const DEFAULT_TERMS: &[&str] = &[
    "SkillTable",
    "SBSkillTableProperty",
    "JustParry",
    "JustParry1",
    "ComboParry",
    "GuardSkill",
    "DataTable",
    "/Script/SB",
    "ESBSkill",
    "Parry",
    "Guard",
    "Evade",
    "Attack",
    "Burst",
    "Tachy",
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

fn run() -> Result<(), ScanError> {
    let config = Config::parse(env::args_os().skip(1))?;
    require_file(
        &config.input,
        "the extracted cooked asset blob is required for local string evidence",
        "extract_iostore output",
        "test -f local/stellarblade/extracted/base-pakchunk0/SB/Content/Local/Data/SkillTable.uasset",
    )?;
    let bytes = fs::read(&config.input).map_err(|source| ScanError::Io {
        path: config.input.clone(),
        action: "read input blob",
        source,
    })?;
    let strings = visible_strings(&bytes);
    let candidates = candidate_strings(&strings, &config.terms);

    fs::create_dir_all(&config.out).map_err(|source| ScanError::Io {
        path: config.out.clone(),
        action: "create output directory",
        source,
    })?;
    write_strings(&config.out.join("strings.jsonl"), &config, &strings)?;
    write_candidates(&config.out.join("candidates.jsonl"), &config, &candidates)?;
    write_summary(
        &config.out.join("summary.md"),
        &config,
        bytes.len(),
        &strings,
        &candidates,
    )?;

    println!("wrote asset string scan to {}", config.out.display());
    Ok(())
}

#[derive(Debug)]
struct Config {
    input: PathBuf,
    out: PathBuf,
    terms: Vec<String>,
}

impl Config {
    fn parse<I>(args: I) -> Result<Self, ScanError>
    where
        I: IntoIterator<Item = std::ffi::OsString>,
    {
        let mut input = PathBuf::from(DEFAULT_INPUT);
        let mut out = PathBuf::from(DEFAULT_OUT);
        let mut terms = DEFAULT_TERMS
            .iter()
            .map(|term| (*term).to_owned())
            .collect::<Vec<_>>();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--input" => {
                    input = PathBuf::from(args.next().ok_or(ScanError::Usage(
                        "--input requires a path argument".to_owned(),
                    ))?);
                }
                "--out" => {
                    out = PathBuf::from(
                        args.next()
                            .ok_or(ScanError::Usage("--out requires a path".to_owned()))?,
                    );
                }
                "--terms" => {
                    let value = args.next().ok_or(ScanError::Usage(
                        "--terms requires a comma-separated list".to_owned(),
                    ))?;
                    terms = parse_terms(&value.to_string_lossy());
                }
                "-h" | "--help" => return Err(ScanError::Usage(usage())),
                other => {
                    return Err(ScanError::Usage(format!(
                        "unknown argument: {other}\n\n{}",
                        usage()
                    )));
                }
            }
        }
        if terms.is_empty() {
            return Err(ScanError::Usage(
                "--terms must include at least one non-empty term".to_owned(),
            ));
        }
        Ok(Self { input, out, terms })
    }
}

fn usage() -> String {
    format!("usage: scan_asset_strings [--input {DEFAULT_INPUT}] [--out {DEFAULT_OUT}] [--terms SkillTable,SBSkillTableProperty]")
}

#[derive(Clone, Debug)]
struct StringRecord {
    offset: usize,
    value: String,
}

#[derive(Debug)]
struct CandidateRecord {
    offset: usize,
    value: String,
    matched_terms: Vec<String>,
}

fn visible_strings(bytes: &[u8]) -> Vec<StringRecord> {
    let mut records = Vec::new();
    let mut start = None;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if byte.is_ascii_graphic() || byte == b' ' {
            start.get_or_insert(index);
        } else if let Some(start_index) = start.take() {
            push_string(bytes, start_index, index, &mut records);
        }
    }
    if let Some(start_index) = start {
        push_string(bytes, start_index, bytes.len(), &mut records);
    }
    records
}

fn push_string(bytes: &[u8], start: usize, end: usize, records: &mut Vec<StringRecord>) {
    if end.saturating_sub(start) >= 3 {
        records.push(StringRecord {
            offset: start,
            value: String::from_utf8_lossy(&bytes[start..end]).into_owned(),
        });
    }
}

fn candidate_strings(strings: &[StringRecord], terms: &[String]) -> Vec<CandidateRecord> {
    strings
        .iter()
        .filter_map(|record| {
            let haystack = record.value.to_ascii_lowercase();
            let matched_terms = terms
                .iter()
                .filter(|term| haystack.contains(&term.to_ascii_lowercase()))
                .cloned()
                .collect::<Vec<_>>();
            if matched_terms.is_empty() {
                None
            } else {
                Some(CandidateRecord {
                    offset: record.offset,
                    value: record.value.clone(),
                    matched_terms,
                })
            }
        })
        .collect()
}

fn write_strings(path: &Path, config: &Config, records: &[StringRecord]) -> Result<(), ScanError> {
    let mut writer = writer(path, "create strings jsonl")?;
    for record in records {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"evidence_kind\":\"asset_visible_string\",\"source\":{},\"offset\":{},\"value\":{}}}",
            json_string(&config.input.display().to_string()),
            record.offset,
            json_string(&record.value)
        )?;
    }
    Ok(())
}

fn write_candidates(
    path: &Path,
    config: &Config,
    records: &[CandidateRecord],
) -> Result<(), ScanError> {
    let mut writer = writer(path, "create candidates jsonl")?;
    for record in records {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"evidence_kind\":\"asset_string_candidate\",\"source\":{},\"offset\":{},\"matched_terms\":{},\"value\":{}}}",
            json_string(&config.input.display().to_string()),
            record.offset,
            json_string_array(&record.matched_terms),
            json_string(&record.value)
        )?;
    }
    Ok(())
}

fn write_summary(
    path: &Path,
    config: &Config,
    input_size: usize,
    strings: &[StringRecord],
    candidates: &[CandidateRecord],
) -> Result<(), ScanError> {
    let mut groups = BTreeMap::<String, usize>::new();
    for candidate in candidates {
        for term in &candidate.matched_terms {
            *groups.entry(term.clone()).or_default() += 1;
        }
    }

    let mut writer = writer(path, "create summary")?;
    writeln!(writer, "# Asset String Scan")?;
    writeln!(writer)?;
    writeln!(writer, "- Input: `{}`", config.input.display())?;
    writeln!(writer, "- Size: {input_size} bytes")?;
    writeln!(writer, "- Visible strings: {}", strings.len())?;
    writeln!(writer, "- Candidate strings: {}", candidates.len())?;
    writeln!(writer)?;
    writeln!(writer, "## Matched Terms")?;
    for (term, count) in groups {
        writeln!(writer, "- `{term}`: {count}")?;
    }
    writeln!(writer)?;
    writeln!(
        writer,
        "This is string evidence from an extracted cooked blob, not a decoded DataTable property report."
    )?;
    Ok(())
}

fn writer(path: &Path, action: &'static str) -> Result<BufWriter<File>, ScanError> {
    Ok(BufWriter::new(File::create(path).map_err(|source| {
        ScanError::Io {
            path: path.to_path_buf(),
            action,
            source,
        }
    })?))
}

fn parse_terms(value: &str) -> Vec<String> {
    let mut terms = value
        .split(',')
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms
}

fn json_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn json_string(value: &str) -> String {
    let escaped = tools_support::json::escape(value);
    format!("\"{escaped}\"")
}

fn require_file(
    path: &Path,
    why_required: &str,
    upstream_producer: &str,
    validation_command: &str,
) -> Result<(), ScanError> {
    if path.is_file() {
        return Ok(());
    }
    Err(ScanError::MissingRequiredArtifact(Box::new(
        MissingRequiredArtifact {
            path: path.to_path_buf(),
            why_required: why_required.to_owned(),
            upstream_producer: upstream_producer.to_owned(),
            validation_command: validation_command.to_owned(),
        },
    )))
}

#[derive(Debug)]
enum ScanError {
    Usage(String),
    Io {
        path: PathBuf,
        action: &'static str,
        source: io::Error,
    },
    MissingRequiredArtifact(Box<MissingRequiredArtifact>),
}

#[derive(Debug)]
struct MissingRequiredArtifact {
    path: PathBuf,
    why_required: String,
    upstream_producer: String,
    validation_command: String,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}"),
            Self::Io {
                path,
                action,
                source,
            } => write!(formatter, "failed to {action} `{}`: {source}", path.display()),
            Self::MissingRequiredArtifact(issue) => write!(
                formatter,
                "missing required artifact: {}\nwhy required: {}\nupstream producer: {}\nvalidation command: {}",
                issue.path.display(),
                issue.why_required,
                issue.upstream_producer,
                issue.validation_command
            ),
        }
    }
}

impl std::error::Error for ScanError {}

impl From<io::Error> for ScanError {
    fn from(source: io::Error) -> Self {
        Self::Io {
            path: PathBuf::new(),
            action: "write output",
            source,
        }
    }
}

#[path = "../../tools_support/mod.rs"]
mod tools_support;
