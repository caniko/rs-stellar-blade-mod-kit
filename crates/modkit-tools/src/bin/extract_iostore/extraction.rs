fn extract_entry(config: &Config, entry: &IoStoreEntry) -> Result<ExtractedEntry, ExtractError> {
    if entry.chunk_type != "ExportBundleData" {
        return Err(ExtractError::UnsupportedArtifact(Box::new(
            UnsupportedArtifact {
                artifact: entry.path.clone(),
                why_required:
                    "this pass only extracts cooked asset ExportBundleData chunks for metadata probing"
                        .to_owned(),
                upstream_producer: "retoc list --hash --package --size --path".to_owned(),
                regenerate_command:
                    "rerun with an --entry path whose chunk_type is ExportBundleData".to_owned(),
                validation_command: format!(
                    "rg --fixed-strings '{}' {}",
                    entry.path,
                    config.list.display()
                ),
            },
        )));
    }

    let relative = safe_relative_path(&entry.path)?;
    let output_path = config.extract_to.join(relative);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|source| ExtractError::Io {
            path: parent.to_path_buf(),
            action: "create extraction parent directory",
            source,
        })?;
    }

    let command = format!(
        "{} get {} {} {}",
        config.retoc.display(),
        config.utoc.display(),
        entry.chunk_id,
        output_path.display()
    );
    let status = Command::new(&config.retoc)
        .arg("get")
        .arg(&config.utoc)
        .arg(&entry.chunk_id)
        .arg(&output_path)
        .status()
        .map_err(|source| ExtractError::Io {
            path: config.retoc.clone(),
            action: "run retoc get",
            source,
        })?;
    if !status.success() {
        return Err(ExtractError::Unsupported(format!(
            "retoc get failed for {} with status {:?}",
            entry.path,
            status.code()
        )));
    }

    let byte_count = fs::metadata(&output_path)
        .map_err(|source| ExtractError::Io {
            path: output_path.clone(),
            action: "stat extracted file",
            source,
        })?
        .len();
    if byte_count != entry.size {
        return Err(ExtractError::UnsupportedArtifact(Box::new(
            UnsupportedArtifact {
                artifact: output_path.display().to_string(),
                why_required: format!(
                    "extracted byte count {byte_count} did not match Io Store list size {}",
                    entry.size
                ),
                upstream_producer: "retoc get and the source .utoc/.ucas pair".to_owned(),
                regenerate_command: command.clone(),
                validation_command: format!("wc -c {}", output_path.display()),
            },
        )));
    }

    Ok(ExtractedEntry {
        entry: entry.clone(),
        output_path,
        byte_count,
        command,
    })
}

fn safe_relative_path(path: &str) -> Result<PathBuf, ExtractError> {
    let mut relative = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir | Component::ParentDir => {}
            Component::RootDir | Component::Prefix(_) => {
                return Err(ExtractError::Unsupported(format!(
                    "Io Store path `{path}` is not a relative package path"
                )));
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err(ExtractError::Unsupported(format!(
            "Io Store path `{path}` produced an empty relative path"
        )));
    }
    Ok(relative)
}
