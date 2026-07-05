fn write_summary(path: &Path, analysis: &Analysis) -> Result<(), AnalyzeError> {
    let mut writer = writer(path)?;
    writeln!(writer, "# Stellar Blade Candidate Analysis")?;
    writeln!(writer)?;
    writeln!(
        writer,
        "- Total candidate records: {}",
        analysis.total_records
    )?;
    writeln!(
        writer,
        "- Unique candidate keys: {}",
        analysis.unique_records.len()
    )?;
    writeln!(writer)?;
    writeln!(writer, "## Evidence Kinds")?;
    write_counts(&mut writer, &analysis.evidence_kind_counts, 20)?;
    writeln!(writer)?;
    writeln!(writer, "## Terms")?;
    write_counts(&mut writer, &analysis.term_counts, 20)?;
    writeln!(writer)?;
    writeln!(writer, "## Top Packages")?;
    write_counts(&mut writer, &analysis.package_counts, 20)?;
    writeln!(writer)?;
    writeln!(writer, "## Top Prefixes")?;
    write_counts(&mut writer, &analysis.prefix_counts, 30)?;
    writeln!(writer)?;
    writeln!(writer, "## Top Deduplicated Candidates")?;
    write_candidate_list(&mut writer, analysis.unique_records.iter().take(50))?;
    Ok(())
}

fn write_package_report(path: &Path, analysis: &Analysis) -> Result<(), AnalyzeError> {
    let mut writer = writer(path)?;
    writeln!(writer, "# Candidate Packages")?;
    writeln!(writer)?;
    write_counts(&mut writer, &analysis.package_counts, usize::MAX)?;
    Ok(())
}

fn write_prefix_report(path: &Path, analysis: &Analysis) -> Result<(), AnalyzeError> {
    let mut writer = writer(path)?;
    writeln!(writer, "# Candidate Prefixes")?;
    writeln!(writer)?;
    write_counts(&mut writer, &analysis.prefix_counts, usize::MAX)?;
    Ok(())
}

fn write_term_report(path: &Path, term: &str, analysis: &Analysis) -> Result<(), AnalyzeError> {
    let mut writer = writer(path)?;
    writeln!(writer, "# `{term}` Candidates")?;
    writeln!(writer)?;

    let mut package_counts = BTreeMap::new();
    let mut prefix_counts = BTreeMap::new();
    let filtered = analysis
        .unique_records
        .iter()
        .filter(|record| {
            record
                .candidate
                .matched_terms
                .iter()
                .any(|matched| matched == term)
        })
        .collect::<Vec<_>>();
    for record in &filtered {
        let package = record
            .candidate
            .source_package
            .as_deref()
            .unwrap_or("<loose>")
            .to_owned();
        *package_counts.entry(package).or_insert(0) += record.occurrences;
        *prefix_counts
            .entry(candidate_prefix(&record.candidate.raw_value))
            .or_insert(0) += record.occurrences;
    }

    writeln!(writer, "- Unique candidates: {}", filtered.len())?;
    writeln!(writer)?;
    writeln!(writer, "## Packages")?;
    write_counts(&mut writer, &package_counts, 20)?;
    writeln!(writer)?;
    writeln!(writer, "## Prefixes")?;
    write_counts(&mut writer, &prefix_counts, 30)?;
    writeln!(writer)?;
    writeln!(writer, "## Candidates")?;
    write_candidate_list(&mut writer, filtered.into_iter().take(200))?;
    Ok(())
}

fn write_counts(
    writer: &mut BufWriter<File>,
    counts: &BTreeMap<String, usize>,
    limit: usize,
) -> Result<(), AnalyzeError> {
    let mut entries = counts.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
    for (name, count) in entries.into_iter().take(limit) {
        writeln!(writer, "- `{name}`: {count}")?;
    }
    Ok(())
}

fn write_candidate_list<'a>(
    writer: &mut BufWriter<File>,
    records: impl IntoIterator<Item = &'a DedupedCandidate>,
) -> Result<(), AnalyzeError> {
    for record in records {
        let candidate = &record.candidate;
        writeln!(
            writer,
            "- `{}` ({}) terms=`{}` package=`{}` kind=`{}` sources={}",
            candidate.raw_value,
            record.occurrences,
            candidate.matched_terms.join(","),
            candidate.source_package.as_deref().unwrap_or("<loose>"),
            candidate.source_kind.as_deref().unwrap_or("<unknown>"),
            record.all_sources.len()
        )?;
    }
    Ok(())
}
