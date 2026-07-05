#[derive(Clone, Debug, Eq, PartialEq)]
struct RetocEntry {
    container: String,
    chunk_id: String,
    hash: String,
    package_id: Option<String>,
    chunk_type: String,
    size: u64,
    path: Option<String>,
}

#[derive(Debug)]
struct RetocOutput {
    command: String,
    status_code: Option<i32>,
    stdout: String,
}

fn run_retoc<I>(retoc: &Path, args: I) -> Result<RetocOutput, IoStoreError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    let command = format_command(retoc, &args);
    let output = Command::new(retoc)
        .args(&args)
        .output()
        .map_err(|source| IoStoreError::Io {
            path: retoc.to_path_buf(),
            action: "run retoc",
            source,
        })?;
    if !output.status.success() {
        return Err(IoStoreError::RetocFailed {
            command,
            status_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(RetocOutput {
        command,
        status_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    })
}

fn format_command(retoc: &Path, args: &[OsString]) -> String {
    std::iter::once(retoc.to_string_lossy().into_owned())
        .chain(args.iter().map(|arg| arg.to_string_lossy().into_owned()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_list_output(output: &str) -> Result<Vec<RetocEntry>, IoStoreError> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_list_line)
        .collect()
}

fn parse_list_line(line: &str) -> Result<RetocEntry, IoStoreError> {
    let parts = line.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 7 {
        return Err(IoStoreError::Parse(format!(
            "retoc list line had {} columns; expected at least 7: {line}",
            parts.len()
        )));
    }
    let size = parts[5].parse::<u64>().map_err(|error| {
        IoStoreError::Parse(format!("invalid retoc size `{}`: {error}", parts[5]))
    })?;
    let path = parts[6..].join(" ");
    Ok(RetocEntry {
        container: parts[0].to_owned(),
        chunk_id: parts[1].to_owned(),
        hash: parts[2].to_owned(),
        package_id: if parts[3] == "-" {
            None
        } else {
            Some(parts[3].to_owned())
        },
        chunk_type: parts[4].to_owned(),
        size,
        path: if path == "-" { None } else { Some(path) },
    })
}

fn focus_hits(entries: &[RetocEntry], terms: &[String]) -> Vec<(RetocEntry, Vec<String>)> {
    entries
        .iter()
        .filter_map(|entry| {
            let haystack = entry.path.as_deref().unwrap_or_default();
            let matches = terms
                .iter()
                .filter(|term| contains_case_insensitive(haystack, term))
                .cloned()
                .collect::<Vec<_>>();
            if matches.is_empty() {
                None
            } else {
                Some((entry.clone(), matches))
            }
        })
        .collect()
}

fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn parse_terms(value: &str) -> Vec<String> {
    let mut terms = value
        .split(',')
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms
}
