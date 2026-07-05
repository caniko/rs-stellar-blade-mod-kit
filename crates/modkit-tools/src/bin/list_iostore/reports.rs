fn write_text(path: &Path, text: &str) -> Result<(), IoStoreError> {
    fs::write(path, text).map_err(|source| IoStoreError::Io {
        path: path.to_path_buf(),
        action: "write retoc output",
        source,
    })
}

fn write_list_jsonl(
    path: &Path,
    config: &Config,
    entries: &[RetocEntry],
) -> Result<(), IoStoreError> {
    let mut writer = writer(path)?;
    for entry in entries {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"evidence_kind\":\"iostore_list_entry\",\"source_utoc\":{},\"container\":{},\"chunk_id\":{},\"hash\":{},\"package_id\":{},\"chunk_type\":{},\"size\":{},\"path\":{}}}",
            json_string(&config.utoc.to_string_lossy()),
            json_string(&entry.container),
            json_string(&entry.chunk_id),
            json_string(&entry.hash),
            json_option(entry.package_id.as_deref()),
            json_string(&entry.chunk_type),
            entry.size,
            json_option(entry.path.as_deref()),
        )?;
    }
    Ok(())
}

fn write_focus_jsonl(
    path: &Path,
    config: &Config,
    hits: &[(RetocEntry, Vec<String>)],
) -> Result<(), IoStoreError> {
    let mut writer = writer(path)?;
    for (entry, terms) in hits {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"evidence_kind\":\"iostore_focus_hit\",\"source_utoc\":{},\"container\":{},\"chunk_id\":{},\"package_id\":{},\"chunk_type\":{},\"size\":{},\"matched_terms\":{},\"path\":{}}}",
            json_string(&config.utoc.to_string_lossy()),
            json_string(&entry.container),
            json_string(&entry.chunk_id),
            json_option(entry.package_id.as_deref()),
            json_string(&entry.chunk_type),
            entry.size,
            json_string_array(terms),
            json_option(entry.path.as_deref()),
        )?;
    }
    Ok(())
}

fn write_summary(
    path: &Path,
    config: &Config,
    ucas: &Path,
    info: &RetocOutput,
    list: &RetocOutput,
    entries: &[RetocEntry],
    hits: &[(RetocEntry, Vec<String>)],
) -> Result<(), IoStoreError> {
    let mut by_type = BTreeMap::<String, usize>::new();
    for entry in entries {
        *by_type.entry(entry.chunk_type.clone()).or_insert(0) += 1;
    }

    let mut writer = writer(path)?;
    writeln!(writer, "# Io Store Listing Summary")?;
    writeln!(writer)?;
    writeln!(writer, "- Created at unix seconds: {}", unix_now()?)?;
    writeln!(writer, "- UTOC: `{}`", config.utoc.display())?;
    writeln!(writer, "- UCAS: `{}`", ucas.display())?;
    writeln!(writer, "- retoc info command: `{}`", info.command)?;
    writeln!(writer, "- retoc info status: `{:?}`", info.status_code)?;
    writeln!(writer, "- retoc list command: `{}`", list.command)?;
    writeln!(writer, "- retoc list status: `{:?}`", list.status_code)?;
    writeln!(writer, "- Entries: {}", entries.len())?;
    writeln!(writer, "- Focus hits: {}", hits.len())?;
    writeln!(writer, "- Terms: `{}`", config.terms.join(", "))?;
    writeln!(writer)?;
    writeln!(writer, "## Chunk Types")?;
    for (chunk_type, count) in by_type {
        writeln!(writer, "- `{chunk_type}`: {count}")?;
    }
    writeln!(writer)?;
    writeln!(writer, "## Focus Hits")?;
    if hits.is_empty() {
        writeln!(writer, "- None")?;
    } else {
        for (entry, terms) in hits.iter().take(200) {
            writeln!(
                writer,
                "- `{}` size={} terms=`{}`",
                entry.path.as_deref().unwrap_or("<no path>"),
                entry.size,
                terms.join(", ")
            )?;
        }
    }
    Ok(())
}

fn writer(path: &Path) -> Result<BufWriter<File>, IoStoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| IoStoreError::Io {
            path: parent.to_path_buf(),
            action: "create parent directory",
            source,
        })?;
    }
    let file = File::create(path).map_err(|source| IoStoreError::Io {
        path: path.to_path_buf(),
        action: "create output file",
        source,
    })?;
    Ok(BufWriter::new(file))
}
