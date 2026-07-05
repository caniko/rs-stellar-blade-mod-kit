fn unix_now() -> Result<u64, IoStoreError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| IoStoreError::Parse(format!("system clock is invalid: {error}")))?
        .as_secs())
}

fn require_file(
    path: &Path,
    why_required: &str,
    upstream_producer: &str,
    validation_command: &str,
) -> Result<(), IoStoreError> {
    if path.is_file() {
        return Ok(());
    }
    Err(IoStoreError::MissingRequired {
        path: path.to_path_buf(),
        why_required: why_required.to_owned(),
        upstream_producer: upstream_producer.to_owned(),
        regenerate_command:
            "restore or verify the local Stellar Blade install, then rerun the Io Store lister"
                .to_owned(),
        validation_command: validation_command.to_owned(),
    })
}

fn json_option(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_owned(), json_string)
}

fn json_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", json_escape(value))
}

fn json_escape(value: &str) -> String {
    tools_support::json::escape(value)
}
