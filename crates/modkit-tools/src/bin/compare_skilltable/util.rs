fn require_file(
    path: &Path,
    why_required: &str,
    validation_command: &str,
) -> Result<(), CompareError> {
    if path.is_file() {
        return Ok(());
    }
    Err(CompareError::MissingRequired {
        path: path.to_path_buf(),
        why_required: why_required.to_owned(),
        upstream_producer: "offline recon, pak probe, and uasset probe outputs".to_owned(),
        regenerate_command: "rerun offline_recon, probe_pak, and probe_uasset".to_owned(),
        validation_command: validation_command.to_owned(),
    })
}
