fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn require_file(
    path: &Path,
    why_required: &str,
    upstream_producer: &str,
    validation_command: &str,
) -> Result<(), ExtractError> {
    if path.is_file() {
        return Ok(());
    }
    Err(ExtractError::MissingRequiredArtifact(Box::new(
        MissingRequiredArtifact {
            path: path.to_path_buf(),
            why_required: why_required.to_owned(),
            upstream_producer: upstream_producer.to_owned(),
            validation_command: validation_command.to_owned(),
        },
    )))
}
