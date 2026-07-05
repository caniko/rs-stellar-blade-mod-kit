#[derive(Debug)]
struct CombatCandidate {
    evidence_kind: String,
    source_path: PathBuf,
    source_package: Option<String>,
    source_kind: Option<String>,
    byte_offset: Option<u64>,
    raw_value: String,
    matched_terms: Vec<String>,
}

fn collect_combat_candidates(
    visible_strings: &[VisibleStringRecord],
    loose_files: &[LooseFile],
    terms: &[String],
) -> Vec<CombatCandidate> {
    let mut candidates = Vec::new();
    for record in visible_strings {
        if record.matched_terms.is_empty() {
            continue;
        }
        candidates.push(CombatCandidate {
            evidence_kind: "visible_string_candidate".to_owned(),
            source_path: record.source_path.clone(),
            source_package: Some(record.source_package.clone()),
            source_kind: Some(record.source_kind.clone()),
            byte_offset: Some(record.byte_offset),
            raw_value: record.raw_string.clone(),
            matched_terms: record.matched_terms.clone(),
        });
    }
    for file in loose_files {
        let haystack = format!("{} {}", file.root_label, file.relative_path);
        let matched_terms = matched_terms(&haystack, terms);
        if matched_terms.is_empty() {
            continue;
        }
        candidates.push(CombatCandidate {
            evidence_kind: "loose_file_candidate".to_owned(),
            source_path: file.path.clone(),
            source_package: None,
            source_kind: Some(file.root_label.clone()),
            byte_offset: None,
            raw_value: file.relative_path.clone(),
            matched_terms,
        });
    }
    candidates
}

fn matched_terms(value: &str, terms: &[String]) -> Vec<String> {
    let lowered = value.to_ascii_lowercase();
    terms
        .iter()
        .filter(|term| lowered.contains(term.as_str()))
        .cloned()
        .collect()
}
