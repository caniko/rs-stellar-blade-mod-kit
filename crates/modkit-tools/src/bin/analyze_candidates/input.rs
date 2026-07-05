fn read_candidates(path: &Path) -> Result<Vec<Candidate>, AnalyzeError> {
    let file = File::open(path).map_err(|source| AnalyzeError::Io {
        path: path.to_path_buf(),
        action: "open candidate JSONL",
        source,
    })?;
    let reader = BufReader::new(file);
    let mut candidates = Vec::new();
    for (line_index, line) in reader.lines().enumerate() {
        let line = line.map_err(|source| AnalyzeError::Io {
            path: path.to_path_buf(),
            action: "read candidate JSONL",
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        candidates.push(
            parse_candidate_line(&line).map_err(|message| AnalyzeError::Parse {
                path: path.to_path_buf(),
                line: line_index + 1,
                message,
            })?,
        );
    }
    Ok(candidates)
}

fn parse_candidate_line(line: &str) -> Result<Candidate, String> {
    Ok(Candidate {
        evidence_kind: required_string_field(line, "evidence_kind")?,
        source_path: required_string_field(line, "source_path")?,
        source_package: optional_string_field(line, "source_package")?,
        source_kind: optional_string_field(line, "source_kind")?,
        byte_offset: optional_u64_field(line, "byte_offset")?,
        matched_terms: string_array_field(line, "matched_terms")?,
        raw_value: required_string_field(line, "raw_value")?,
    })
}

fn required_string_field(line: &str, key: &str) -> Result<String, String> {
    optional_string_field(line, key)?
        .ok_or_else(|| format!("missing non-null string field `{key}`"))
}

fn optional_string_field(line: &str, key: &str) -> Result<Option<String>, String> {
    let value = field_value(line, key)?;
    if value.starts_with("null") {
        return Ok(None);
    }
    let (parsed, _) = parse_json_string(value)?;
    Ok(Some(parsed))
}

fn optional_u64_field(line: &str, key: &str) -> Result<Option<u64>, String> {
    let value = field_value(line, key)?;
    if value.starts_with("null") {
        return Ok(None);
    }
    let end = value
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(value.len());
    value[..end]
        .parse::<u64>()
        .map(Some)
        .map_err(|error| format!("invalid numeric field `{key}`: {error}"))
}

fn string_array_field(line: &str, key: &str) -> Result<Vec<String>, String> {
    let value = field_value(line, key)?.trim_start();
    let mut rest = value
        .strip_prefix('[')
        .ok_or_else(|| format!("field `{key}` is not an array"))?;
    let mut out = Vec::new();
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            return Ok(out);
        }
        let (parsed, next) = parse_json_string(rest)?;
        out.push(parsed);
        rest = next.trim_start();
        if rest.starts_with(',') {
            rest = &rest[1..];
        } else if rest.starts_with(']') {
            return Ok(out);
        } else {
            return Err(format!("unterminated array field `{key}`"));
        }
    }
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
