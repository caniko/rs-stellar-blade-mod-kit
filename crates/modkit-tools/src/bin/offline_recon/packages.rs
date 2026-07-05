#[derive(Debug)]
struct PackageGroup {
    name: String,
    group: String,
    files: BTreeMap<String, FileRecord>,
}

#[derive(Debug, Clone)]
struct FileRecord {
    path: PathBuf,
    size_bytes: u64,
}

fn collect_packages(paks_dir: &Path) -> Result<Vec<PackageGroup>, ReconError> {
    let mut groups = BTreeMap::<(String, String), PackageGroup>::new();
    for entry in read_dir_recursive_sorted(paks_dir)? {
        if !entry.is_file() {
            continue;
        }
        let Some(extension) = entry.extension().and_then(OsStr::to_str) else {
            continue;
        };
        if !matches!(extension, "pak" | "utoc" | "ucas") {
            continue;
        }

        let name = entry
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("<unknown>")
            .to_owned();
        let group = if entry
            .components()
            .any(|component| component.as_os_str() == "~mods-off")
        {
            "existing_mod_example"
        } else {
            "game_package"
        }
        .to_owned();
        let metadata = fs::metadata(&entry).map_err(|source| ReconError::Io {
            path: entry.clone(),
            action: "read metadata",
            source,
        })?;
        let key = (group.clone(), name.clone());
        groups
            .entry(key)
            .or_insert_with(|| PackageGroup {
                name,
                group,
                files: BTreeMap::new(),
            })
            .files
            .insert(
                extension.to_owned(),
                FileRecord {
                    path: entry,
                    size_bytes: metadata.len(),
                },
            );
    }
    Ok(groups.into_values().collect())
}

#[derive(Debug)]
struct LooseRoot {
    label: String,
    path: PathBuf,
    exists: bool,
}

#[derive(Debug)]
struct LooseFile {
    root_label: String,
    path: PathBuf,
    relative_path: String,
    size_bytes: u64,
}

fn collect_loose_roots(game_root: &Path) -> Result<Vec<LooseRoot>, ReconError> {
    let mut roots = Vec::new();
    for label in ["Movies", "Images", "Splash"] {
        let path = game_root.join("SB/Content").join(label);
        roots.push(LooseRoot {
            label: label.to_owned(),
            exists: path.exists(),
            path,
        });
    }
    Ok(roots)
}

fn collect_loose_files(roots: &[LooseRoot]) -> Result<Vec<LooseFile>, ReconError> {
    let mut files = Vec::new();
    for root in roots {
        if !root.exists {
            continue;
        }
        for path in read_dir_recursive_sorted(&root.path)? {
            if !path.is_file() {
                continue;
            }
            let metadata = fs::metadata(&path).map_err(|source| ReconError::Io {
                path: path.clone(),
                action: "read metadata",
                source,
            })?;
            let relative_path = path
                .strip_prefix(&root.path)
                .unwrap_or(path.as_path())
                .to_string_lossy()
                .replace('\\', "/");
            files.push(LooseFile {
                root_label: root.label.clone(),
                path,
                relative_path,
                size_bytes: metadata.len(),
            });
        }
    }
    Ok(files)
}

#[derive(Debug)]
struct MetadataSource {
    package_name: String,
    source_kind: String,
    path: PathBuf,
    size_bytes: u64,
    scan_reason: String,
}

fn metadata_sources(packages: &[PackageGroup]) -> Vec<MetadataSource> {
    let mut sources = Vec::new();
    for package in packages {
        if let Some(utoc) = package.files.get("utoc") {
            sources.push(MetadataSource {
                package_name: package.name.clone(),
                source_kind: "utoc".to_owned(),
                path: utoc.path.clone(),
                size_bytes: utoc.size_bytes,
                scan_reason: "Io Store table-of-contents metadata".to_owned(),
            });
        }
        if let Some(pak) = package.files.get("pak") {
            if pak.size_bytes <= SMALL_PAK_METADATA_LIMIT {
                sources.push(MetadataSource {
                    package_name: package.name.clone(),
                    source_kind: "small_pak".to_owned(),
                    path: pak.path.clone(),
                    size_bytes: pak.size_bytes,
                    scan_reason: format!(
                        "pak metadata file is <= {} bytes",
                        SMALL_PAK_METADATA_LIMIT
                    ),
                });
            }
        }
    }
    sources
}
