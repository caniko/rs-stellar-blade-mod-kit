fn read_iostore_entries(path: &Path) -> Result<Vec<IoStoreEntry>, CompareError> {
    read_lines(path)?
        .into_iter()
        .map(|line| {
            Ok(IoStoreEntry {
                container: required_string_field(&line, "container")?,
                chunk_type: required_string_field(&line, "chunk_type")?,
                size: required_u64_field(&line, "size")?,
                path: optional_string_field(&line, "path")?,
            })
        })
        .collect()
}

fn read_pak_entries(path: &Path) -> Result<Vec<PakEntry>, CompareError> {
    read_lines(path)?
        .into_iter()
        .map(|line| {
            Ok(PakEntry {
                path: required_string_field(&line, "path")?,
                size: required_u64_field(&line, "size")?,
                encrypted: required_bool_field(&line, "encrypted")?,
            })
        })
        .collect()
}

fn read_values(path: &Path, key: &str) -> Result<BTreeSet<String>, CompareError> {
    read_lines(path)?
        .into_iter()
        .map(|line| required_string_field(&line, key))
        .collect::<Result<BTreeSet<_>, _>>()
}

fn read_visible_strings(path: &Path) -> Result<Vec<VisibleStringRecord>, CompareError> {
    read_lines(path)?
        .into_iter()
        .map(|line| {
            Ok(VisibleStringRecord {
                source_package: required_string_field(&line, "source_package")?,
                source_kind: required_string_field(&line, "source_kind")?,
                raw_string: required_string_field(&line, "raw_string")?,
            })
        })
        .collect()
}

fn read_lines(path: &Path) -> Result<Vec<String>, CompareError> {
    let file = File::open(path).map_err(|source| CompareError::Io {
        path: path.to_path_buf(),
        action: "open input",
        source,
    })?;
    BufReader::new(file)
        .lines()
        .map(|line| {
            line.map_err(|source| CompareError::Io {
                path: path.to_path_buf(),
                action: "read input",
                source,
            })
        })
        .filter(|line| line.as_ref().map_or(true, |line| !line.trim().is_empty()))
        .collect()
}
