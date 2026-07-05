fn require_file(
    path: &Path,
    why_required: &str,
    upstream_producer: &str,
    validation_command: &str,
) -> Result<(), UassetError> {
    if path.is_file() {
        return Ok(());
    }
    Err(UassetError::MissingRequired {
        path: path.to_path_buf(),
        why_required: why_required.to_owned(),
        upstream_producer: upstream_producer.to_owned(),
        regenerate_command: "run probe_pak with --extract-to against the disabled mod pak"
            .to_owned(),
        validation_command: validation_command.to_owned(),
    })
}

#[derive(Debug)]
enum UassetError {
    Usage(String),
    MissingRequired {
        path: PathBuf,
        why_required: String,
        upstream_producer: String,
        regenerate_command: String,
        validation_command: String,
    },
    Unsupported(String),
    Io {
        path: PathBuf,
        action: &'static str,
        source: io::Error,
    },
}

impl std::fmt::Display for UassetError {
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
            Self::Unsupported(message) => write!(formatter, "unsupported uasset layout: {message}"),
            Self::Io {
                path,
                action,
                source,
            } => write!(formatter, "failed to {action} `{}`: {source}", path.display()),
        }
    }
}

impl std::error::Error for UassetError {}

impl From<io::Error> for UassetError {
    fn from(source: io::Error) -> Self {
        Self::Io {
            path: PathBuf::from("<unknown>"),
            action: "perform I/O",
            source,
        }
    }
}
