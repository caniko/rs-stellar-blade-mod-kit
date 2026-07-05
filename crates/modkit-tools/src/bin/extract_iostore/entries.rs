fn read_iostore_entries(path: &Path) -> Result<Vec<IoStoreEntry>, ExtractError> {
    let file = File::open(path).map_err(|source| ExtractError::Io {
        path: path.to_path_buf(),
        action: "open Io Store list",
        source,
    })?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .enumerate()
        .filter_map(|(index, line)| match line {
            Ok(line) if line.trim().is_empty() => None,
            Ok(line) => parse_iostore_entry(index + 1, &line).transpose(),
            Err(source) => Some(Err(ExtractError::Io {
                path: path.to_path_buf(),
                action: "read Io Store list",
                source,
            })),
        })
        .collect()
}

fn parse_iostore_entry(
    _line_number: usize,
    line: &str,
) -> Result<Option<IoStoreEntry>, ExtractError> {
    let path = optional_string_field(line, "path")?;
    let Some(path) = path else {
        return Ok(None);
    };
    Ok(Some(IoStoreEntry {
        container: required_string_field(line, "container")?,
        chunk_id: required_string_field(line, "chunk_id")?,
        content_hash: required_string_field(line, "hash")?,
        package_id: optional_string_field(line, "package_id")?,
        chunk_type: required_string_field(line, "chunk_type")?,
        size: required_u64_field(line, "size")?,
        path,
    }))
}

fn select_entries<'a>(
    entries: &'a [IoStoreEntry],
    requested: &[String],
) -> Result<Vec<&'a IoStoreEntry>, ExtractError> {
    requested
        .iter()
        .map(|request| {
            let matches = entries
                .iter()
                .filter(|entry| entry.path == *request || entry.path.ends_with(request))
                .collect::<Vec<_>>();
            match matches.as_slice() {
                [entry] => Ok(*entry),
                [] => Err(ExtractError::UnsupportedArtifact(Box::new(
                    UnsupportedArtifact {
                        artifact: request.clone(),
                        why_required:
                            "the requested path must exist in the retoc-generated Io Store allowlist"
                                .to_owned(),
                        upstream_producer: "list_iostore using retoc list --hash --package --size --path"
                            .to_owned(),
                        regenerate_command:
                            "nix run .#list-iostore -- --utoc /data/nvme0/can/games/steamapps/steamapps/common/StellarBlade/SB/Content/Paks/pakchunk0-WindowsNoEditor.utoc --out local/stellarblade/recon/first-pass/analysis/iostore-pakchunk0"
                                .to_owned(),
                        validation_command:
                            format!("rg --fixed-strings '{}' local/stellarblade/recon/first-pass/analysis/iostore-pakchunk0/iostore-list.jsonl", request),
                    },
                ))),
                many => Err(ExtractError::Unsupported(format!(
                    "entry request `{request}` matched {} Io Store entries; use a more exact --entry path",
                    many.len()
                ))),
            }
        })
        .collect()
}
