//! Analyze offline Stellar Blade combat candidate JSONL into focused reports.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DEFAULT_INPUT: &str = "local/stellarblade/recon/first-pass/combat_candidates.jsonl";
const DEFAULT_OUT: &str = "local/stellarblade/recon/first-pass/analysis";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), AnalyzeError> {
    let config = Config::parse(env::args_os().skip(1))?;
    require_file(
        &config.input,
        "combat candidate JSONL is the required upstream evidence from offline_recon",
        "run offline_recon against the local Stellar Blade install",
        "test -f local/stellarblade/recon/first-pass/combat_candidates.jsonl",
    )?;

    let candidates = read_candidates(&config.input)?;
    let analysis = Analysis::build(candidates);
    fs::create_dir_all(&config.out).map_err(|source| AnalyzeError::Io {
        path: config.out.clone(),
        action: "create output directory",
        source,
    })?;

    write_summary(&config.out.join("summary.md"), &analysis)?;
    write_package_report(&config.out.join("packages.md"), &analysis)?;
    write_prefix_report(&config.out.join("prefixes.md"), &analysis)?;
    let terms_dir = config.out.join("terms");
    fs::create_dir_all(&terms_dir).map_err(|source| AnalyzeError::Io {
        path: terms_dir.clone(),
        action: "create terms output directory",
        source,
    })?;
    for term in analysis.term_counts.keys() {
        write_term_report(&terms_dir.join(format!("{term}.md")), term, &analysis)?;
    }

    println!("wrote candidate analysis to {}", config.out.display());
    Ok(())
}

#[derive(Debug)]
struct Config {
    input: PathBuf,
    out: PathBuf,
}

impl Config {
    fn parse<I>(args: I) -> Result<Self, AnalyzeError>
    where
        I: IntoIterator<Item = std::ffi::OsString>,
    {
        let mut input = PathBuf::from(DEFAULT_INPUT);
        let mut out = PathBuf::from(DEFAULT_OUT);
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--input" => {
                    let value = args.next().ok_or(AnalyzeError::Usage(
                        "--input requires a path argument".to_owned(),
                    ))?;
                    input = PathBuf::from(value);
                }
                "--out" => {
                    let value = args.next().ok_or(AnalyzeError::Usage(
                        "--out requires a path argument".to_owned(),
                    ))?;
                    out = PathBuf::from(value);
                }
                "-h" | "--help" => return Err(AnalyzeError::Usage(usage())),
                other => {
                    return Err(AnalyzeError::Usage(format!(
                        "unknown argument: {other}\n\n{}",
                        usage()
                    )));
                }
            }
        }
        Ok(Self { input, out })
    }
}

fn usage() -> String {
    format!("usage: analyze_candidates [--input {DEFAULT_INPUT}] [--out {DEFAULT_OUT}]")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Candidate {
    evidence_kind: String,
    source_path: String,
    source_package: Option<String>,
    source_kind: Option<String>,
    byte_offset: Option<u64>,
    matched_terms: Vec<String>,
    raw_value: String,
}

#[derive(Debug)]
struct DedupedCandidate {
    candidate: Candidate,
    occurrences: usize,
    all_sources: BTreeSet<String>,
}

#[derive(Debug)]
struct Analysis {
    total_records: usize,
    unique_records: Vec<DedupedCandidate>,
    term_counts: BTreeMap<String, usize>,
    package_counts: BTreeMap<String, usize>,
    prefix_counts: BTreeMap<String, usize>,
    evidence_kind_counts: BTreeMap<String, usize>,
}

impl Analysis {
    fn build(candidates: Vec<Candidate>) -> Self {
        let total_records = candidates.len();
        let mut by_key = BTreeMap::<String, DedupedCandidate>::new();
        let mut term_counts = BTreeMap::new();
        let mut package_counts = BTreeMap::new();
        let mut prefix_counts = BTreeMap::new();
        let mut evidence_kind_counts = BTreeMap::new();

        for candidate in candidates {
            for term in &candidate.matched_terms {
                *term_counts.entry(term.clone()).or_insert(0) += 1;
            }
            let package = candidate
                .source_package
                .as_deref()
                .unwrap_or("<loose>")
                .to_owned();
            *package_counts.entry(package).or_insert(0) += 1;
            *prefix_counts
                .entry(candidate_prefix(&candidate.raw_value))
                .or_insert(0) += 1;
            *evidence_kind_counts
                .entry(candidate.evidence_kind.clone())
                .or_insert(0) += 1;

            let key = dedup_key(&candidate);
            by_key
                .entry(key)
                .and_modify(|existing| {
                    existing.occurrences += 1;
                    existing.all_sources.insert(source_label(&candidate));
                })
                .or_insert_with(|| {
                    let mut all_sources = BTreeSet::new();
                    all_sources.insert(source_label(&candidate));
                    DedupedCandidate {
                        candidate,
                        occurrences: 1,
                        all_sources,
                    }
                });
        }

        let mut unique_records = by_key.into_values().collect::<Vec<_>>();
        unique_records.sort_by(|left, right| {
            right
                .occurrences
                .cmp(&left.occurrences)
                .then_with(|| left.candidate.raw_value.cmp(&right.candidate.raw_value))
        });

        Self {
            total_records,
            unique_records,
            term_counts,
            package_counts,
            prefix_counts,
            evidence_kind_counts,
        }
    }
}

include!("input.rs");
include!("reports.rs");
include!("util.rs");
include!("errors.rs");
include!("tests.rs");
