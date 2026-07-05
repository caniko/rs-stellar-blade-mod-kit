fn require_file(
    path: &Path,
    why_required: &str,
    upstream_producer: &str,
    validation_command: &str,
) -> Result<(), PakProbeError> {
    if path.is_file() {
        return Ok(());
    }
    Err(PakProbeError::MissingRequired {
        path: path.to_path_buf(),
        why_required: why_required.to_owned(),
        upstream_producer: upstream_producer.to_owned(),
        regenerate_command:
            "restore the .pak from Steam or the local disabled-mod source, then rerun the probe"
                .to_owned(),
        validation_command: validation_command.to_owned(),
    })
}

#[derive(Debug)]
enum PakProbeError {
    Usage(String),
    MissingRequired {
        path: PathBuf,
        why_required: String,
        upstream_producer: String,
        regenerate_command: String,
        validation_command: String,
    },
    UnsupportedPackageBackend(Box<PackageBackendIssue>),
    Unsupported(String),
    Io {
        path: PathBuf,
        action: &'static str,
        source: io::Error,
    },
}

#[derive(Debug)]
struct PackageBackendIssue {
    path: PathBuf,
    detected_format: String,
    why_required: String,
    upstream_producer: String,
    regenerate_command: String,
    validation_command: String,
}

impl std::fmt::Display for PakProbeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}"),
            Self::MissingRequired {
                path,
                why_required,
                upstream_producer,
                regenerate_command,
                validation_command,
            } => write!(
                formatter,
                "missing required input: {}\nwhy required: {}\nupstream producer to fix: {}\nregenerate workflow: {}\nvalidation command: {}",
                path.display(),
                why_required,
                upstream_producer,
                regenerate_command,
                validation_command
            ),
            Self::UnsupportedPackageBackend(issue) => write!(
                formatter,
                "unsupported package backend: {}\ndetected format: {}\nwhy required: {}\nupstream producer to fix: {}\nregenerate workflow: {}\nvalidation command: {}",
                issue.path.display(),
                issue.detected_format,
                issue.why_required,
                issue.upstream_producer,
                issue.regenerate_command,
                issue.validation_command
            ),
            Self::Unsupported(message) => write!(formatter, "unsupported pak layout: {message}"),
            Self::Io {
                path,
                action,
                source,
            } => write!(formatter, "failed to {action} `{}`: {source}", path.display()),
        }
    }
}

impl std::error::Error for PakProbeError {}

impl From<io::Error> for PakProbeError {
    fn from(source: io::Error) -> Self {
        Self::Io {
            path: PathBuf::from("<unknown>"),
            action: "perform I/O",
            source,
        }
    }
}
