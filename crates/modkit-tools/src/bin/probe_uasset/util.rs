fn writer(path: &Path) -> Result<BufWriter<File>, UassetError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| UassetError::Io {
            path: parent.to_path_buf(),
            action: "create parent directory",
            source,
        })?;
    }
    let file = File::create(path).map_err(|source| UassetError::Io {
        path: path.to_path_buf(),
        action: "create output file",
        source,
    })?;
    Ok(BufWriter::new(file))
}

fn json_array_strings(values: Vec<&str>) -> String {
    format!(
        "[{}]",
        values
            .into_iter()
            .map(json_string)
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
