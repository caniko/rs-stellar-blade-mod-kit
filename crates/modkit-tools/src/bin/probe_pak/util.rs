fn unix_now() -> Result<u64, PakProbeError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PakProbeError::Unsupported(format!("system clock is invalid: {error}")))?
        .as_secs())
}

fn writer(path: &Path) -> Result<BufWriter<File>, PakProbeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| PakProbeError::Io {
            path: parent.to_path_buf(),
            action: "create parent directory",
            source,
        })?;
    }
    let file = File::create(path).map_err(|source| PakProbeError::Io {
        path: path.to_path_buf(),
        action: "create output file",
        source,
    })?;
    Ok(BufWriter::new(file))
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
    }
    out
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", json_escape(value))
}

fn json_escape(value: &str) -> String {
    tools_support::json::escape(value)
}
