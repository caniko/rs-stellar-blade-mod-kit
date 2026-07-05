#[derive(Debug)]
enum AnalyzeError {
    Usage(String),
    MissingRequired {
        path: PathBuf,
        why_required: String,
        upstream_producer: String,
        regenerate_command: String,
        validation_command: String,
    },
    Io {
        path: PathBuf,
        action: &'static str,
        source: io::Error,
    },
    Parse {
        path: PathBuf,
        line: usize,
        message: String,
    },
}

impl std::fmt::Display for AnalyzeError {
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
            Self::Io {
                path,
                action,
                source,
            } => write!(formatter, "failed to {action} `{}`: {source}", path.display()),
            Self::Parse {
                path,
                line,
                message,
            } => write!(
                formatter,
                "failed to parse `{}` at line {}: {}",
                path.display(),
                line,
                message
            ),
        }
    }
}

impl std::error::Error for AnalyzeError {}

impl From<io::Error> for AnalyzeError {
    fn from(source: io::Error) -> Self {
        Self::Io {
            path: PathBuf::from("<unknown>"),
            action: "perform I/O",
            source,
        }
    }
}
