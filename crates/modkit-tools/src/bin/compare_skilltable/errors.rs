#[derive(Debug)]
enum CompareError {
    Usage(String),
    MissingRequired {
        path: PathBuf,
        why_required: String,
        upstream_producer: String,
        regenerate_command: String,
        validation_command: String,
    },
    Parse(String),
    Io {
        path: PathBuf,
        action: &'static str,
        source: io::Error,
    },
}

impl std::fmt::Display for CompareError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) | Self::Parse(message) => write!(formatter, "{message}"),
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
            Self::Io {
                path,
                action,
                source,
            } => write!(formatter, "failed to {action} `{}`: {source}", path.display()),
        }
    }
}

impl std::error::Error for CompareError {}

impl From<io::Error> for CompareError {
    fn from(source: io::Error) -> Self {
        Self::Io {
            path: PathBuf::from("<unknown>"),
            action: "perform I/O",
            source,
        }
    }
}
