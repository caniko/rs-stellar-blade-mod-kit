fn required_string_field(line: &str, key: &str) -> Result<String, ExtractError> {
    optional_string_field(line, key)?.ok_or_else(|| {
        ExtractError::Parse(format!(
            "missing required string field `{key}` in line: {line}"
        ))
    })
}

fn optional_string_field(line: &str, key: &str) -> Result<Option<String>, ExtractError> {
    let Some(start) = field_start(line, key) else {
        return Ok(None);
    };
    let value = line[start..].trim_start();
    if value.starts_with("null") {
        return Ok(None);
    }
    if !value.starts_with('"') {
        return Err(ExtractError::Parse(format!(
            "field `{key}` is not a JSON string or null in line: {line}"
        )));
    }
    parse_json_string(value).map(Some)
}

fn required_u64_field(line: &str, key: &str) -> Result<u64, ExtractError> {
    let start = field_start(line, key)
        .ok_or_else(|| ExtractError::Parse(format!("missing field `{key}` in line: {line}")))?;
    let value = line[start..].trim_start();
    let digits = value
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return Err(ExtractError::Parse(format!(
            "field `{key}` is not an unsigned integer in line: {line}"
        )));
    }
    digits
        .parse::<u64>()
        .map_err(|error| ExtractError::Parse(format!("invalid field `{key}`: {error}")))
}

fn field_start(line: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\":");
    line.find(&needle).map(|index| index + needle.len())
}

fn parse_json_string(value: &str) -> Result<String, ExtractError> {
    let mut chars = value.chars();
    if chars.next() != Some('"') {
        return Err(ExtractError::Parse("expected JSON string".to_owned()));
    }
    let mut output = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Ok(output),
            '\\' => {
                let escaped = chars
                    .next()
                    .ok_or_else(|| ExtractError::Parse("unterminated JSON escape".to_owned()))?;
                output.push(match escaped {
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'b' => '\u{0008}',
                    'f' => '\u{000c}',
                    other => other,
                });
            }
            other => output.push(other),
        }
    }
    Err(ExtractError::Parse("unterminated JSON string".to_owned()))
}

fn json_optional_string(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_owned(), json_string)
}

fn json_string(value: &str) -> String {
    let escaped = tools_support::json::escape(value);
    format!("\"{escaped}\"")
}
