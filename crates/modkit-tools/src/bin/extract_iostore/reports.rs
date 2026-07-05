fn write_manifest(
    path: &Path,
    config: &Config,
    extracted: &[ExtractedEntry],
) -> Result<(), ExtractError> {
    let mut writer = BufWriter::new(File::create(path).map_err(|source| ExtractError::Io {
        path: path.to_path_buf(),
        action: "create manifest",
        source,
    })?);
    writeln!(writer, "{{")?;
    writeln!(writer, "  \"schema_version\": 1,")?;
    writeln!(
        writer,
        "  \"source_utoc\": {},",
        json_string(&config.utoc.display().to_string())
    )?;
    writeln!(
        writer,
        "  \"source_list\": {},",
        json_string(&config.list.display().to_string())
    )?;
    writeln!(
        writer,
        "  \"extract_to\": {},",
        json_string(&config.extract_to.display().to_string())
    )?;
    writeln!(writer, "  \"extracted_at_unix\": {},", unix_timestamp())?;
    writeln!(writer, "  \"entries\": [")?;
    for (index, item) in extracted.iter().enumerate() {
        let comma = if index + 1 == extracted.len() {
            ""
        } else {
            ","
        };
        writeln!(writer, "    {{")?;
        writeln!(writer, "      \"path\": {},", json_string(&item.entry.path))?;
        writeln!(
            writer,
            "      \"chunk_id\": {},",
            json_string(&item.entry.chunk_id)
        )?;
        writeln!(
            writer,
            "      \"content_hash\": {},",
            json_string(&item.entry.content_hash)
        )?;
        writeln!(
            writer,
            "      \"package_id\": {},",
            json_optional_string(item.entry.package_id.as_deref())
        )?;
        writeln!(
            writer,
            "      \"container\": {},",
            json_string(&item.entry.container)
        )?;
        writeln!(
            writer,
            "      \"chunk_type\": {},",
            json_string(&item.entry.chunk_type)
        )?;
        writeln!(writer, "      \"listed_size\": {},", item.entry.size)?;
        writeln!(writer, "      \"byte_count\": {},", item.byte_count)?;
        writeln!(
            writer,
            "      \"output_path\": {},",
            json_string(&item.output_path.display().to_string())
        )?;
        writeln!(writer, "      \"command\": {}", json_string(&item.command))?;
        writeln!(writer, "    }}{comma}")?;
    }
    writeln!(writer, "  ]")?;
    writeln!(writer, "}}")?;
    Ok(())
}

fn write_summary(
    path: &Path,
    config: &Config,
    extracted: &[ExtractedEntry],
) -> Result<(), ExtractError> {
    let mut writer = BufWriter::new(File::create(path).map_err(|source| ExtractError::Io {
        path: path.to_path_buf(),
        action: "create summary",
        source,
    })?);
    writeln!(writer, "# Io Store Targeted Extraction")?;
    writeln!(writer)?;
    writeln!(writer, "- Source `.utoc`: `{}`", config.utoc.display())?;
    writeln!(writer, "- Allowlist: `{}`", config.list.display())?;
    writeln!(
        writer,
        "- Extraction root: `{}`",
        config.extract_to.display()
    )?;
    writeln!(writer, "- Extracted entries: {}", extracted.len())?;
    writeln!(writer)?;
    writeln!(writer, "## Entries")?;
    for item in extracted {
        writeln!(
            writer,
            "- `{}` -> `{}` ({} bytes, chunk `{}`)",
            item.entry.path,
            item.output_path.display(),
            item.byte_count,
            item.entry.chunk_id
        )?;
    }
    writeln!(writer)?;
    writeln!(
        writer,
        "Only allowlisted entries were extracted. No broad base-game extraction was performed."
    )?;
    Ok(())
}
