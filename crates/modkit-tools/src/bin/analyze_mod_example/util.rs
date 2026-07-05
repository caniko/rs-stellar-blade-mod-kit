fn require_file(
    path: &Path,
    why_required: &str,
    upstream_producer: &str,
    validation_command: &str,
) -> Result<(), ModAnalyzeError> {
    if path.is_file() {
        return Ok(());
    }
    Err(ModAnalyzeError::MissingRequired {
        path: path.to_path_buf(),
        why_required: why_required.to_owned(),
        upstream_producer: upstream_producer.to_owned(),
        regenerate_command: "nix develop --command cargo run --bin offline_recon -- --game-root /data/nvme0/can/games/steamapps/steamapps/common/StellarBlade --out local/stellarblade/recon/first-pass".to_owned(),
        validation_command: validation_command.to_owned(),
    })
}
