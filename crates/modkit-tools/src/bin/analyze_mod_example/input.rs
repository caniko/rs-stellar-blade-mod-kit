fn read_records(path: &Path) -> Result<Vec<VisibleStringRecord>, ModAnalyzeError> {
    let file = File::open(path).map_err(|source| ModAnalyzeError::Io {
        path: path.to_path_buf(),
        action: "open visible string JSONL",
        source,
    })?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for (line_index, line) in reader.lines().enumerate() {
        let line = line.map_err(|source| ModAnalyzeError::Io {
            path: path.to_path_buf(),
            action: "read visible string JSONL",
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(parse_visible_string_line(&line).map_err(|message| {
            ModAnalyzeError::Parse {
                path: path.to_path_buf(),
                line: line_index + 1,
                message,
            }
        })?);
    }
    Ok(records)
}

fn parse_visible_string_line(line: &str) -> Result<VisibleStringRecord, String> {
    Ok(VisibleStringRecord {
        source_package: required_string_field(line, "source_package")?,
        source_kind: required_string_field(line, "source_kind")?,
        source_path: required_string_field(line, "source_path")?,
        byte_offset: required_u64_field(line, "byte_offset")?,
        raw_string: required_string_field(line, "raw_string")?,
    })
}

fn required_string_field(line: &str, key: &str) -> Result<String, String> {
    let value = field_value(line, key)?;
    let (parsed, _) = parse_json_string(value)?;
    Ok(parsed)
}

fn required_u64_field(line: &str, key: &str) -> Result<u64, String> {
    let value = field_value(line, key)?;
    let end = value
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(value.len());
    value[..end]
        .parse::<u64>()
        .map_err(|error| format!("invalid numeric field `{key}`: {error}"))
}

fn field_value<'a>(line: &'a str, key: &str) -> Result<&'a str, String> {
    let needle = format!("\"{key}\":");
    let start = line
        .find(&needle)
        .ok_or_else(|| format!("missing field `{key}`"))?
        + needle.len();
    Ok(&line[start..])
}

fn parse_json_string(input: &str) -> Result<(String, &str), String> {
    let mut chars = input.char_indices();
    match chars.next() {
        Some((_, '"')) => {}
        _ => return Err("expected JSON string".to_owned()),
    }

    let mut out = String::new();
    let mut escaped = false;
    for (index, ch) in chars {
        if escaped {
            match ch {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                other => out.push(other),
            }
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Ok((out, &input[index + 1..])),
            other => out.push(other),
        }
    }
    Err("unterminated JSON string".to_owned())
}
