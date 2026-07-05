fn read_dir_sorted(path: &Path) -> Result<Vec<PathBuf>, ReconError> {
    let mut entries = fs::read_dir(path)
        .map_err(|source| ReconError::Io {
            path: path.to_path_buf(),
            action: "read directory",
            source,
        })?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|source| ReconError::Io {
                    path: path.to_path_buf(),
                    action: "read directory entry",
                    source,
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    Ok(entries)
}

fn read_dir_recursive_sorted(path: &Path) -> Result<Vec<PathBuf>, ReconError> {
    let mut out = Vec::new();
    if !path.exists() {
        return Ok(out);
    }
    read_dir_recursive_into(path, &mut out)?;
    out.sort();
    Ok(out)
}

fn read_dir_recursive_into(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), ReconError> {
    for entry in read_dir_sorted(path)? {
        if entry.is_dir() {
            read_dir_recursive_into(&entry, out)?;
        } else {
            out.push(entry);
        }
    }
    Ok(())
}

fn unix_now() -> Result<u64, ReconError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| ReconError::Usage(format!("system clock is before Unix epoch: {error}")))?
        .as_secs())
}
