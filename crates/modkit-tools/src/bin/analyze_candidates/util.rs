fn writer(path: &Path) -> Result<BufWriter<File>, AnalyzeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| AnalyzeError::Io {
            path: parent.to_path_buf(),
            action: "create parent directory",
            source,
        })?;
    }
    let file = File::create(path).map_err(|source| AnalyzeError::Io {
        path: path.to_path_buf(),
        action: "create report",
        source,
    })?;
    Ok(BufWriter::new(file))
}

fn dedup_key(candidate: &Candidate) -> String {
    format!(
        "{}\u{1f}{}\u{1f}{}\u{1f}{}",
        candidate.evidence_kind,
        candidate.source_package.as_deref().unwrap_or("<loose>"),
        candidate.source_kind.as_deref().unwrap_or("<unknown>"),
        candidate.raw_value
    )
}

fn source_label(candidate: &Candidate) -> String {
    match candidate.byte_offset {
        Some(offset) => format!("{}:{offset}", candidate.source_path),
        None => candidate.source_path.clone(),
    }
}

fn candidate_prefix(raw_value: &str) -> String {
    let name = raw_value.rsplit('/').next().unwrap_or(raw_value);
    let name = name.split('.').next().unwrap_or(name);
    let prefix = name
        .split('_')
        .find(|part| !part.is_empty())
        .unwrap_or(name)
        .trim();
    if prefix.is_empty() {
        "<empty>".to_owned()
    } else {
        prefix.to_owned()
    }
}

fn require_file(
    path: &Path,
    why_required: &str,
    upstream_producer: &str,
    validation_command: &str,
) -> Result<(), AnalyzeError> {
    if path.is_file() {
        return Ok(());
    }
    Err(AnalyzeError::MissingRequired {
        path: path.to_path_buf(),
        why_required: why_required.to_owned(),
        upstream_producer: upstream_producer.to_owned(),
        regenerate_command: "nix develop --command cargo run --bin offline_recon -- --game-root /data/nvme0/can/games/steamapps/steamapps/common/StellarBlade --out local/stellarblade/recon/first-pass".to_owned(),
        validation_command: validation_command.to_owned(),
    })
}
