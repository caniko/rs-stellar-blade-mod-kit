fn extract_entries(
    pak_path: &Path,
    probe: &PakProbe,
    extract_to: &Path,
    filters: &[String],
) -> Result<(), PakProbeError> {
    let selected = select_entries(&probe.entries, filters)?;
    let mut pak = File::open(pak_path).map_err(|source| PakProbeError::Io {
        path: pak_path.to_path_buf(),
        action: "open pak for extraction",
        source,
    })?;
    let mut extracted = Vec::new();

    for entry in selected {
        validate_extractable_entry(probe, entry)?;
        let data_offset = entry.offset + local_header_size(&mut pak, pak_path, entry)?;
        let output_path = safe_output_path(extract_to, &entry.path)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|source| PakProbeError::Io {
                path: parent.to_path_buf(),
                action: "create extraction directory",
                source,
            })?;
        }

        pak.seek(SeekFrom::Start(data_offset))
            .map_err(|source| PakProbeError::Io {
                path: pak_path.to_path_buf(),
                action: "seek to pak entry payload",
                source,
            })?;
        let mut output = File::create(&output_path).map_err(|source| PakProbeError::Io {
            path: output_path.clone(),
            action: "create extracted entry",
            source,
        })?;
        let copied = io::copy(&mut (&mut pak).take(entry.size), &mut output).map_err(|source| {
            PakProbeError::Io {
                path: output_path.clone(),
                action: "write extracted entry",
                source,
            }
        })?;
        if copied != entry.size {
            return Err(PakProbeError::Unsupported(format!(
                "extracted byte count mismatch for `{}`: copied={} expected={}",
                entry.path, copied, entry.size
            )));
        }
        extracted.push(ExtractedEntry {
            path: entry.path.clone(),
            output_path,
            offset: entry.offset,
            data_offset,
            size: entry.size,
            sha1: entry.hash,
        });
    }

    write_extract_manifest(
        &extract_to.join("manifest.json"),
        pak_path,
        probe,
        &extracted,
    )?;
    Ok(())
}

fn select_entries<'a>(
    entries: &'a [PakEntry],
    filters: &[String],
) -> Result<Vec<&'a PakEntry>, PakProbeError> {
    if filters.is_empty() {
        return Ok(entries.iter().collect());
    }

    let mut selected = Vec::new();
    for filter in filters {
        let Some(entry) = entries.iter().find(|entry| entry.path == *filter) else {
            return Err(PakProbeError::MissingRequired {
                path: PathBuf::from(filter),
                why_required: "requested pak entry was not present in the parsed index".to_owned(),
                upstream_producer: "the source .pak index".to_owned(),
                regenerate_command:
                    "rerun probe_pak without --entry to inspect available entries".to_owned(),
                validation_command: format!("rg '{}$' <report-dir>/entries.jsonl", filter),
            });
        };
        selected.push(entry);
    }
    Ok(selected)
}

fn validate_extractable_entry(probe: &PakProbe, entry: &PakEntry) -> Result<(), PakProbeError> {
    if entry.compression_method != 0 {
        return Err(PakProbeError::Unsupported(format!(
            "entry `{}` is compressed with method {}; extraction currently supports only uncompressed entries",
            entry.path, entry.compression_method
        )));
    }
    if entry.encrypted {
        return Err(PakProbeError::Unsupported(format!(
            "entry `{}` is encrypted; extraction requires an unencrypted entry",
            entry.path
        )));
    }
    let data_offset = entry.offset + UNCOMPRESSED_LOCAL_HEADER_SIZE;
    if data_offset >= probe.file_size || data_offset + entry.size > probe.file_size {
        return Err(PakProbeError::Unsupported(format!(
            "entry `{}` payload range is outside pak bounds: data_offset={} size={} file_size={}",
            entry.path, data_offset, entry.size, probe.file_size
        )));
    }
    Ok(())
}

fn local_header_size(
    pak: &mut File,
    pak_path: &Path,
    entry: &PakEntry,
) -> Result<u64, PakProbeError> {
    pak.seek(SeekFrom::Start(entry.offset))
        .map_err(|source| PakProbeError::Io {
            path: pak_path.to_path_buf(),
            action: "seek to local pak entry header",
            source,
        })?;
    let mut header = [0u8; UNCOMPRESSED_LOCAL_HEADER_SIZE as usize];
    pak.read_exact(&mut header)
        .map_err(|source| PakProbeError::Io {
            path: pak_path.to_path_buf(),
            action: "read local pak entry header",
            source,
        })?;
    let mut cursor = ByteCursor::new(&header);
    let local_offset = cursor.read_i64()? as u64;
    let local_size = cursor.read_i64()? as u64;
    let local_uncompressed_size = cursor.read_i64()? as u64;
    let local_compression_method = cursor.read_u32()?;
    let local_hash = cursor.read_array_20()?;
    let local_encrypted = cursor.read_u8()? != 0;
    let _local_block_size = cursor.read_u32()?;

    if !(local_offset == entry.offset || local_offset == 0)
        || local_size != entry.size
        || local_uncompressed_size != entry.uncompressed_size
        || local_compression_method != entry.compression_method
        || local_hash != entry.hash
        || local_encrypted != entry.encrypted
    {
        return Err(PakProbeError::Unsupported(format!(
            "local pak entry header for `{}` did not match index metadata",
            entry.path
        )));
    }
    Ok(UNCOMPRESSED_LOCAL_HEADER_SIZE)
}

fn safe_output_path(root: &Path, entry_path: &str) -> Result<PathBuf, PakProbeError> {
    let mut output = root.to_path_buf();
    for component in Path::new(entry_path).components() {
        match component {
            std::path::Component::Normal(value) => output.push(value),
            _ => {
                return Err(PakProbeError::Unsupported(format!(
                    "refusing to extract unsafe pak entry path `{entry_path}`"
                )));
            }
        }
    }
    Ok(output)
}

fn write_extract_manifest(
    path: &Path,
    pak_path: &Path,
    probe: &PakProbe,
    extracted: &[ExtractedEntry],
) -> Result<(), PakProbeError> {
    let mut writer = writer(path)?;
    writeln!(writer, "{{")?;
    writeln!(writer, "  \"schema_version\": 1,")?;
    writeln!(writer, "  \"created_at_unix_seconds\": {},", unix_now()?)?;
    writeln!(
        writer,
        "  \"source_pak\": {},",
        json_string(&pak_path.to_string_lossy())
    )?;
    writeln!(writer, "  \"pak_version\": {},", probe.footer.version)?;
    writeln!(
        writer,
        "  \"mount_point\": {},",
        json_string(&probe.mount_point)
    )?;
    writeln!(writer, "  \"entries\": [")?;
    for (index, entry) in extracted.iter().enumerate() {
        writeln!(writer, "    {{")?;
        writeln!(writer, "      \"path\": {},", json_string(&entry.path))?;
        writeln!(
            writer,
            "      \"output_path\": {},",
            json_string(&entry.output_path.to_string_lossy())
        )?;
        writeln!(writer, "      \"offset\": {},", entry.offset)?;
        writeln!(writer, "      \"data_offset\": {},", entry.data_offset)?;
        writeln!(writer, "      \"size\": {},", entry.size)?;
        writeln!(writer, "      \"sha1\": {}", json_string(&hex(&entry.sha1)))?;
        writeln!(
            writer,
            "    }}{}",
            if index + 1 == extracted.len() {
                ""
            } else {
                ","
            }
        )?;
    }
    writeln!(writer, "  ]")?;
    writeln!(writer, "}}")?;
    Ok(())
}
