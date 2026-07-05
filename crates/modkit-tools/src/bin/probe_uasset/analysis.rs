fn detected_references(package: &PackageProbe) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    for name in &package.names {
        if matches!(
            name.value.as_str(),
            "/Script/SB" | "DataTable" | "SBSkillTableProperty"
        ) || name.value.starts_with("ESBSkill")
        {
            refs.insert(name.value.clone());
        }
    }
    for import in &package.imports {
        refs.insert(format!(
            "import:{}:{}",
            import.class_name, import.object_name
        ));
    }
    for export in &package.exports {
        refs.insert(format!("export:{}", export.object_name));
    }
    refs
}

fn interesting_reference(value: &str) -> bool {
    matches!(
        value,
        "/Script/SB" | "DataTable" | "SBSkillTableProperty" | "SkillTable"
    ) || value.starts_with("ESBSkill")
        || ROW_TERMS.iter().any(|term| value.contains(term))
}

fn matched_terms(value: &str) -> Vec<&'static str> {
    ROW_TERMS
        .iter()
        .copied()
        .filter(|term| value.contains(term))
        .collect()
}

fn visible_strings(bytes: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    let mut start = None;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if matches!(byte, b' '..=b'~') {
            if start.is_none() {
                start = Some(index);
            }
        } else if let Some(start_index) = start.take() {
            push_visible(bytes, start_index, index, &mut strings);
        }
    }
    if let Some(start_index) = start {
        push_visible(bytes, start_index, bytes.len(), &mut strings);
    }
    strings
}

fn push_visible(bytes: &[u8], start: usize, end: usize, strings: &mut Vec<String>) {
    if end.saturating_sub(start) < 4 {
        return;
    }
    if let Ok(value) = std::str::from_utf8(&bytes[start..end]) {
        strings.push(value.to_owned());
    }
}
