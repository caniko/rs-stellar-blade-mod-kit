fn write_inventory(
    path: &Path,
    install: &ValidatedInstall,
    packages: &[PackageGroup],
    loose_roots: &[LooseRoot],
    loose_files: &[LooseFile],
    terms: &[String],
    run_started_unix_seconds: u64,
) -> Result<(), ReconError> {
    let mut writer = json_writer(path)?;
    writeln!(writer, "{{")?;
    write_json_field(&mut writer, 1, "schema_version", "1", true)?;
    write_json_field(
        &mut writer,
        1,
        "run_started_unix_seconds",
        &run_started_unix_seconds.to_string(),
        true,
    )?;
    write_json_field(
        &mut writer,
        1,
        "game_root",
        &json_string(&install.game_root.to_string_lossy()),
        true,
    )?;
    write_json_field(
        &mut writer,
        1,
        "executable",
        &json_string(&install.executable.to_string_lossy()),
        true,
    )?;
    write_json_field(
        &mut writer,
        1,
        "terms",
        &json_array_strings(terms.iter().map(String::as_str)),
        true,
    )?;

    writeln!(writer, "  \"package_triplets\": [")?;
    for (index, package) in packages.iter().enumerate() {
        writeln!(writer, "    {{")?;
        write_json_field(&mut writer, 3, "name", &json_string(&package.name), true)?;
        write_json_field(&mut writer, 3, "group", &json_string(&package.group), true)?;
        writeln!(writer, "      \"files\": {{")?;
        for (file_index, (extension, record)) in package.files.iter().enumerate() {
            writeln!(writer, "        \"{}\": {{", json_escape(extension))?;
            write_json_field(
                &mut writer,
                5,
                "path",
                &json_string(&record.path.to_string_lossy()),
                true,
            )?;
            write_json_field(
                &mut writer,
                5,
                "size_bytes",
                &record.size_bytes.to_string(),
                false,
            )?;
            writeln!(
                writer,
                "        }}{}",
                if file_index + 1 == package.files.len() {
                    ""
                } else {
                    ","
                }
            )?;
        }
        writeln!(writer, "      }}")?;
        writeln!(
            writer,
            "    }}{}",
            if index + 1 == packages.len() { "" } else { "," }
        )?;
    }
    writeln!(writer, "  ],")?;

    writeln!(writer, "  \"loose_content_roots\": [")?;
    for (index, root) in loose_roots.iter().enumerate() {
        writeln!(writer, "    {{")?;
        write_json_field(&mut writer, 3, "label", &json_string(&root.label), true)?;
        write_json_field(
            &mut writer,
            3,
            "path",
            &json_string(&root.path.to_string_lossy()),
            true,
        )?;
        write_json_field(
            &mut writer,
            3,
            "exists",
            if root.exists { "true" } else { "false" },
            false,
        )?;
        writeln!(
            writer,
            "    }}{}",
            if index + 1 == loose_roots.len() {
                ""
            } else {
                ","
            }
        )?;
    }
    writeln!(writer, "  ],")?;

    writeln!(writer, "  \"loose_files\": [")?;
    for (index, file) in loose_files.iter().enumerate() {
        writeln!(writer, "    {{")?;
        write_json_field(&mut writer, 3, "root", &json_string(&file.root_label), true)?;
        write_json_field(
            &mut writer,
            3,
            "relative_path",
            &json_string(&file.relative_path),
            true,
        )?;
        write_json_field(
            &mut writer,
            3,
            "path",
            &json_string(&file.path.to_string_lossy()),
            true,
        )?;
        write_json_field(
            &mut writer,
            3,
            "size_bytes",
            &file.size_bytes.to_string(),
            false,
        )?;
        writeln!(
            writer,
            "    }}{}",
            if index + 1 == loose_files.len() {
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

fn write_visible_strings(path: &Path, records: &[VisibleStringRecord]) -> Result<(), ReconError> {
    let mut writer = json_writer(path)?;
    for record in records {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"evidence_kind\":\"visible_string_candidate\",\"source_package\":{},\"source_kind\":{},\"source_path\":{},\"byte_offset\":{},\"matched_terms\":{},\"raw_string\":{}}}",
            json_string(&record.source_package),
            json_string(&record.source_kind),
            json_string(&record.source_path.to_string_lossy()),
            record.byte_offset,
            json_array_strings(record.matched_terms.iter().map(String::as_str)),
            json_string(&record.raw_string),
        )?;
    }
    Ok(())
}

fn write_combat_candidates(path: &Path, records: &[CombatCandidate]) -> Result<(), ReconError> {
    let mut writer = json_writer(path)?;
    for record in records {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"evidence_kind\":{},\"source_path\":{},\"source_package\":{},\"source_kind\":{},\"byte_offset\":{},\"matched_terms\":{},\"raw_value\":{}}}",
            json_string(&record.evidence_kind),
            json_string(&record.source_path.to_string_lossy()),
            json_optional_string(record.source_package.as_deref()),
            json_optional_string(record.source_kind.as_deref()),
            record
                .byte_offset
                .map_or_else(|| "null".to_owned(), |value| value.to_string()),
            json_array_strings(record.matched_terms.iter().map(String::as_str)),
            json_string(&record.raw_value),
        )?;
    }
    Ok(())
}

fn write_summary(
    path: &Path,
    install: &ValidatedInstall,
    packages: &[PackageGroup],
    loose_files: &[LooseFile],
    visible_strings: &[VisibleStringRecord],
    combat_candidates: &[CombatCandidate],
    metadata_sources: &[MetadataSource],
) -> Result<(), ReconError> {
    let mut writer = text_writer(path)?;
    writeln!(writer, "# Stellar Blade Offline Recon Summary")?;
    writeln!(writer)?;
    writeln!(writer, "- Game root: `{}`", install.game_root.display())?;
    writeln!(writer, "- Package groups: {}", packages.len())?;
    writeln!(
        writer,
        "- Metadata sources scanned: {}",
        metadata_sources.len()
    )?;
    writeln!(writer, "- Visible strings: {}", visible_strings.len())?;
    writeln!(writer, "- Loose content files: {}", loose_files.len())?;
    writeln!(writer, "- Combat candidates: {}", combat_candidates.len())?;
    writeln!(writer)?;
    writeln!(writer, "## Runtime Recon Status")?;
    writeln!(writer)?;
    writeln!(
        writer,
        "Runtime Bouldy/UE4SS discovery was not executed in this pass. The current C++ entrypoint still has `collect_unreal_candidates()` as a stub, so UObject/UClass/UFunction/FProperty metadata capture remains blocked until the UE4SS headers and shim integration are wired."
    )?;
    writeln!(writer)?;
    writeln!(writer, "## Package Groups")?;
    writeln!(writer)?;
    for package in packages {
        let extensions = package
            .files
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            writer,
            "- `{}` ({}) [{}]",
            package.name, package.group, extensions
        )?;
    }
    writeln!(writer)?;
    writeln!(writer, "## Metadata Sources")?;
    writeln!(writer)?;
    for source in metadata_sources {
        writeln!(
            writer,
            "- `{}` ({}, {} bytes): {}",
            source.path.display(),
            source.source_kind,
            source.size_bytes,
            source.scan_reason
        )?;
    }
    writeln!(writer)?;
    writeln!(writer, "## Top Candidate Groups")?;
    writeln!(writer)?;
    let mut grouped = BTreeMap::<String, usize>::new();
    for candidate in combat_candidates {
        *grouped.entry(candidate.evidence_kind.clone()).or_default() += 1;
    }
    for (kind, count) in grouped {
        writeln!(writer, "- `{kind}`: {count}")?;
    }
    writeln!(writer)?;
    writeln!(writer, "## Sample Combat Candidates")?;
    writeln!(writer)?;
    for candidate in combat_candidates.iter().take(40) {
        writeln!(
            writer,
            "- `{}` from `{}` matched `{}`",
            candidate.raw_value,
            candidate.source_path.display(),
            candidate.matched_terms.join(", ")
        )?;
    }
    Ok(())
}

fn write_json_field(
    writer: &mut BufWriter<File>,
    indent_level: usize,
    key: &str,
    value: &str,
    comma: bool,
) -> Result<(), ReconError> {
    writeln!(
        writer,
        "{}\"{}\": {}{}",
        "  ".repeat(indent_level),
        json_escape(key),
        value,
        if comma { "," } else { "" }
    )?;
    Ok(())
}

fn json_writer(path: &Path) -> Result<BufWriter<File>, ReconError> {
    text_writer(path)
}

fn text_writer(path: &Path) -> Result<BufWriter<File>, ReconError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ReconError::Io {
            path: parent.to_path_buf(),
            action: "create parent directory",
            source,
        })?;
    }
    let file = File::create(path).map_err(|source| ReconError::Io {
        path: path.to_path_buf(),
        action: "create output file",
        source,
    })?;
    Ok(BufWriter::new(file))
}

fn json_array_strings<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    let values = values
        .into_iter()
        .map(json_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn json_optional_string(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_owned(), json_string)
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", json_escape(value))
}

fn json_escape(value: &str) -> String {
    tools_support::json::escape(value)
}
