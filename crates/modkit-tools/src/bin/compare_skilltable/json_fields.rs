fn optional_string_field(line: &str, key: &str) -> Result<Option<String>, CompareError> {
    let value = field_value(line, key)?.trim_start();
    if value.starts_with("null") {
        Ok(None)
    } else {
        let (parsed, _) = parse_json_string(value)?;
        Ok(Some(parsed))
    }
}

fn required_string_field(line: &str, key: &str) -> Result<String, CompareError> {
    let value = field_value(line, key)?;
    let (parsed, _) = parse_json_string(value)?;
    Ok(parsed)
}

fn required_u64_field(line: &str, key: &str) -> Result<u64, CompareError> {
    let value = field_value(line, key)?;
    let end = value
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(value.len());
    value[..end]
        .parse::<u64>()
        .map_err(|error| CompareError::Parse(format!("invalid `{key}`: {error}")))
}

fn required_bool_field(line: &str, key: &str) -> Result<bool, CompareError> {
    let value = field_value(line, key)?.trim_start();
    if value.starts_with("true") {
        Ok(true)
    } else if value.starts_with("false") {
        Ok(false)
    } else {
        Err(CompareError::Parse(format!("invalid bool field `{key}`")))
    }
}

fn field_value<'a>(line: &'a str, key: &str) -> Result<&'a str, CompareError> {
    let needle = format!("\"{key}\":");
    let start = line
        .find(&needle)
        .ok_or_else(|| CompareError::Parse(format!("missing field `{key}`")))?
        + needle.len();
    Ok(&line[start..])
}

fn parse_json_string(input: &str) -> Result<(String, &str), CompareError> {
    let mut chars = input.char_indices();
    match chars.next() {
        Some((_, '"')) => {}
        _ => return Err(CompareError::Parse("expected JSON string".to_owned())),
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
    Err(CompareError::Parse("unterminated JSON string".to_owned()))
}
