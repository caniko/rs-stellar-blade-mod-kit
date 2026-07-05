#[derive(Debug)]
struct VisibleStringRecord {
    source_package: String,
    source_kind: String,
    source_path: PathBuf,
    byte_offset: u64,
    raw_string: String,
    matched_terms: Vec<String>,
}

fn collect_visible_strings(
    sources: &[MetadataSource],
    terms: &[String],
) -> Result<Vec<VisibleStringRecord>, ReconError> {
    let mut records = Vec::new();
    for source in sources {
        let mut file = File::open(&source.path).map_err(|source_error| ReconError::Io {
            path: source.path.clone(),
            action: "open metadata source",
            source: source_error,
        })?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|source_error| ReconError::Io {
                path: source.path.clone(),
                action: "read metadata source",
                source: source_error,
            })?;
        extract_visible_strings(&bytes, |byte_offset, raw_string| {
            records.push(VisibleStringRecord {
                source_package: source.package_name.clone(),
                source_kind: source.source_kind.clone(),
                source_path: source.path.clone(),
                byte_offset,
                matched_terms: matched_terms(raw_string, terms),
                raw_string: raw_string.to_owned(),
            });
        });
    }
    Ok(records)
}

fn extract_visible_strings<'a>(bytes: &'a [u8], mut emit: impl FnMut(u64, &'a str)) {
    let mut start = None;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if is_visible_ascii(byte) {
            if start.is_none() {
                start = Some(index);
            }
            continue;
        }

        if let Some(start_index) = start.take() {
            emit_if_visible(bytes, start_index, index, &mut emit);
        }
    }
    if let Some(start_index) = start {
        emit_if_visible(bytes, start_index, bytes.len(), &mut emit);
    }
}

fn emit_if_visible<'a>(
    bytes: &'a [u8],
    start: usize,
    end: usize,
    emit: &mut impl FnMut(u64, &'a str),
) {
    if end.saturating_sub(start) < MIN_VISIBLE_STRING_LEN {
        return;
    }
    if let Ok(value) = std::str::from_utf8(&bytes[start..end]) {
        emit(start as u64, value);
    }
}

fn is_visible_ascii(byte: u8) -> bool {
    matches!(byte, b' '..=b'~')
}
