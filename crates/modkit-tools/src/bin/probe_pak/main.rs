//! Minimal Unreal Pak index probe for local Stellar Blade package examples.

use std::env;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_PAK: &str = "/data/nvme0/can/games/steamapps/steamapps/common/StellarBlade/SB/Content/Paks/~mods-off/SB_ImprovedPerfectDefense_Extended_P.pak";
const DEFAULT_OUT: &str = "local/stellarblade/recon/first-pass/analysis/pak-probe";
const PAK_MAGIC_LE: [u8; 4] = [0xe1, 0x12, 0x6f, 0x5a];
const FOOTER_SEARCH_BYTES: u64 = 512;
const UNCOMPRESSED_LOCAL_HEADER_SIZE: u64 = 53;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), PakProbeError> {
    let config = Config::parse(env::args_os().skip(1))?;
    require_file(
        &config.pak,
        "a local .pak file is required so the probe can read the footer and index",
        "Stellar Blade install or disabled mod package under SB/Content/Paks",
        "test -f '<pak-path>'",
    )?;
    require_supported_package_format(&config.pak)?;

    let probe = probe_pak(&config.pak)?;
    fs::create_dir_all(&config.out).map_err(|source| PakProbeError::Io {
        path: config.out.clone(),
        action: "create output directory",
        source,
    })?;
    write_summary(&config.out.join("summary.md"), &probe)?;
    write_entries_jsonl(&config.out.join("entries.jsonl"), &probe)?;
    if let Some(extract_to) = &config.extract_to {
        extract_entries(&config.pak, &probe, extract_to, &config.entries)?;
    }

    println!("wrote pak probe to {}", config.out.display());
    Ok(())
}

fn require_supported_package_format(path: &Path) -> Result<(), PakProbeError> {
    match package_format(path) {
        PackageFormat::Pak => Ok(()),
        PackageFormat::IoStoreToc | PackageFormat::IoStoreContainer => {
            Err(PakProbeError::UnsupportedPackageBackend(Box::new(
                PackageBackendIssue {
                path: path.to_path_buf(),
                detected_format: package_format(path).label().to_owned(),
                why_required:
                    "authoritative base-game package listings require an Io Store table-of-contents backend, not the legacy Unreal Pak index parser"
                        .to_owned(),
                upstream_producer:
                    "Phase 5 Io Store backend: a validated in-tree .utoc parser or a pinned external lister such as retoc/FModel/UnrealPak-compatible tooling"
                        .to_owned(),
                regenerate_command:
                    "rerun this probe with a .pak path for the current backend; for base-game .utoc/.ucas files, first add and validate the Io Store backend"
                        .to_owned(),
                validation_command:
                    "nix develop --command bash -lc 'command -v retoc || command -v UnrealPak'"
                        .to_owned(),
                },
            )))
        }
        PackageFormat::Unknown => Err(PakProbeError::UnsupportedPackageBackend(Box::new(
            PackageBackendIssue {
            path: path.to_path_buf(),
            detected_format: "unknown".to_owned(),
            why_required: "the current package listing backend only accepts .pak files".to_owned(),
            upstream_producer: "a supported local package file under SB/Content/Paks".to_owned(),
            regenerate_command: "rerun with --pak <path-to-file.pak>".to_owned(),
            validation_command: "test -f '<path-to-file.pak>'".to_owned(),
            },
        ))),
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum PackageFormat {
    Pak,
    IoStoreToc,
    IoStoreContainer,
    Unknown,
}

impl PackageFormat {
    fn label(self) -> &'static str {
        match self {
            Self::Pak => "unreal_pak",
            Self::IoStoreToc => "unreal_io_store_utoc",
            Self::IoStoreContainer => "unreal_io_store_ucas",
            Self::Unknown => "unknown",
        }
    }
}

fn package_format(path: &Path) -> PackageFormat {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("pak") => PackageFormat::Pak,
        Some("utoc") => PackageFormat::IoStoreToc,
        Some("ucas") => PackageFormat::IoStoreContainer,
        _ => PackageFormat::Unknown,
    }
}

#[derive(Debug)]
struct Config {
    pak: PathBuf,
    out: PathBuf,
    extract_to: Option<PathBuf>,
    entries: Vec<String>,
}

impl Config {
    fn parse<I>(args: I) -> Result<Self, PakProbeError>
    where
        I: IntoIterator<Item = std::ffi::OsString>,
    {
        let mut pak = PathBuf::from(DEFAULT_PAK);
        let mut out = PathBuf::from(DEFAULT_OUT);
        let mut extract_to = None;
        let mut entries = Vec::new();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--pak" => {
                    let value = args.next().ok_or(PakProbeError::Usage(
                        "--pak requires a path argument".to_owned(),
                    ))?;
                    pak = PathBuf::from(value);
                }
                "--out" => {
                    let value = args.next().ok_or(PakProbeError::Usage(
                        "--out requires a path argument".to_owned(),
                    ))?;
                    out = PathBuf::from(value);
                }
                "--extract-to" => {
                    let value = args.next().ok_or(PakProbeError::Usage(
                        "--extract-to requires a path argument".to_owned(),
                    ))?;
                    extract_to = Some(PathBuf::from(value));
                }
                "--entry" => {
                    let value = args.next().ok_or(PakProbeError::Usage(
                        "--entry requires an entry path argument".to_owned(),
                    ))?;
                    entries.push(value.to_string_lossy().into_owned());
                }
                "-h" | "--help" => return Err(PakProbeError::Usage(usage())),
                other => {
                    return Err(PakProbeError::Usage(format!(
                        "unknown argument: {other}\n\n{}",
                        usage()
                    )));
                }
            }
        }
        Ok(Self {
            pak,
            out,
            extract_to,
            entries,
        })
    }
}

fn usage() -> String {
    format!(
        "usage: probe_pak [--pak {DEFAULT_PAK}] [--out {DEFAULT_OUT}] [--extract-to local/stellarblade/extracted/pkg] [--entry SB/Content/Local/Data/SkillTable.uasset]"
    )
}

#[path = "../../tools_support/mod.rs"]
mod tools_support;

include!("model.rs");
include!("parser.rs");
include!("reports.rs");
include!("extraction.rs");
include!("util.rs");
include!("errors.rs");
include!("tests.rs");
