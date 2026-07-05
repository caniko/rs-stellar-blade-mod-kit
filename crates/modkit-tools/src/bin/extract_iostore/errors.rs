#[derive(Debug)]
enum ExtractError {
    Usage(String),
    Io {
        path: PathBuf,
        action: &'static str,
        source: io::Error,
    },
    Parse(String),
    Unsupported(String),
    MissingRequiredArtifact(Box<MissingRequiredArtifact>),
    UnsupportedArtifact(Box<UnsupportedArtifact>),
}

#[derive(Debug)]
struct MissingRequiredArtifact {
    path: PathBuf,
    why_required: String,
    upstream_producer: String,
    validation_command: String,
}

#[derive(Debug)]
struct UnsupportedArtifact {
    artifact: String,
    why_required: String,
    upstream_producer: String,
    regenerate_command: String,
    validation_command: String,
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}"),
            Self::Io {
                path,
                action,
                source,
            } => write!(formatter, "failed to {action} `{}`: {source}", path.display()),
            Self::Parse(message) => write!(formatter, "failed to parse Io Store list: {message}"),
            Self::Unsupported(message) => write!(formatter, "unsupported Io Store extraction: {message}"),
            Self::MissingRequiredArtifact(issue) => write!(
                formatter,
                "missing required artifact: {}\nwhy required: {}\nupstream producer: {}\nvalidation command: {}",
                issue.path.display(),
                issue.why_required,
                issue.upstream_producer,
                issue.validation_command
            ),
            Self::UnsupportedArtifact(issue) => write!(
                formatter,
                "unsupported artifact: {}\nwhy required: {}\nupstream producer: {}\nregenerate/retry command: {}\nvalidation command: {}",
                issue.artifact,
                issue.why_required,
                issue.upstream_producer,
                issue.regenerate_command,
                issue.validation_command
            ),
        }
    }
}

impl std::error::Error for ExtractError {}

impl From<io::Error> for ExtractError {
    fn from(source: io::Error) -> Self {
        Self::Io {
            path: PathBuf::new(),
            action: "write output",
            source,
        }
    }
}
