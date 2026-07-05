#[derive(Debug)]
struct ValidatedInstall {
    game_root: PathBuf,
    executable: PathBuf,
    paks_dir: PathBuf,
}

impl ValidatedInstall {
    fn validate(game_root: &Path) -> Result<Self, ReconError> {
        let executable = game_root.join("SB/Binaries/Win64/SB-Win64-Shipping.exe");
        require_file(
            &executable,
            "the shipping executable identifies the target install and runtime binary",
            "Steam should install or repair Stellar Blade",
            "test -f '<game-root>/SB/Binaries/Win64/SB-Win64-Shipping.exe'",
        )?;

        let paks_dir = game_root.join("SB/Content/Paks");
        require_dir(
            &paks_dir,
            "Io Store package metadata and payloads live under this directory",
            "Steam should install or repair Stellar Blade content files",
            "test -d '<game-root>/SB/Content/Paks'",
        )?;

        let mut has_matching_pair = false;
        for entry in read_dir_sorted(&paks_dir)? {
            if entry.extension() == Some(OsStr::new("utoc")) {
                let matching_ucas = entry.with_extension("ucas");
                if matching_ucas.is_file() {
                    has_matching_pair = true;
                    break;
                }
            }
        }
        if !has_matching_pair {
            return Err(ReconError::MissingRequired {
                path: paks_dir,
                why_required: "at least one .utoc with a matching .ucas is required to confirm readable Io Store package triplets".to_owned(),
                upstream_producer: "Steam install or game content depot repair".to_owned(),
                regenerate_command: "steam validate/repair Stellar Blade, then rerun the collector".to_owned(),
                validation_command: "find '<game-root>/SB/Content/Paks' -maxdepth 1 -name '*.utoc' -exec sh -c 'test -f \"${1%.utoc}.ucas\"' sh {} \\; -print -quit".to_owned(),
            });
        }

        Ok(Self {
            game_root: game_root.to_path_buf(),
            executable,
            paks_dir,
        })
    }
}

fn require_file(
    path: &Path,
    why_required: &str,
    upstream_producer: &str,
    validation_command: &str,
) -> Result<(), ReconError> {
    if path.is_file() {
        return Ok(());
    }
    Err(ReconError::MissingRequired {
        path: path.to_path_buf(),
        why_required: why_required.to_owned(),
        upstream_producer: upstream_producer.to_owned(),
        regenerate_command: "steam validate/repair Stellar Blade, then rerun the collector"
            .to_owned(),
        validation_command: validation_command.to_owned(),
    })
}

fn require_dir(
    path: &Path,
    why_required: &str,
    upstream_producer: &str,
    validation_command: &str,
) -> Result<(), ReconError> {
    if path.is_dir() {
        return Ok(());
    }
    Err(ReconError::MissingRequired {
        path: path.to_path_buf(),
        why_required: why_required.to_owned(),
        upstream_producer: upstream_producer.to_owned(),
        regenerate_command: "steam validate/repair Stellar Blade, then rerun the collector"
            .to_owned(),
        validation_command: validation_command.to_owned(),
    })
}
