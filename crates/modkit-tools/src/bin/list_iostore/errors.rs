#[derive(Debug)]
enum IoStoreError {
    Usage(String),
    MissingRequired {
        path: PathBuf,
        why_required: String,
        upstream_producer: String,
        regenerate_command: String,
        validation_command: String,
    },
    RetocFailed {
        command: String,
        status_code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    Parse(String),
    Io {
        path: PathBuf,
        action: &'static str,
        source: io::Error,
    },
}

impl std::fmt::Display for IoStoreError {
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
            Self::RetocFailed {
                command,
                status_code,
                stdout,
                stderr,
            } => write!(
                formatter,
                "retoc command failed: {command}\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
                status_code, stdout, stderr
            ),
            Self::Parse(message) => write!(formatter, "failed to parse retoc output: {message}"),
            Self::Io {
                path,
                action,
                source,
            } => write!(formatter, "failed to {action} `{}`: {source}", path.display()),
        }
    }
}

impl std::error::Error for IoStoreError {}

impl From<io::Error> for IoStoreError {
    fn from(source: io::Error) -> Self {
        Self::Io {
            path: PathBuf::from("<unknown>"),
            action: "perform I/O",
            source,
        }
    }
}
