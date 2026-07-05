fn write_report(path: &Path, report: &CompareReport) -> Result<(), CompareError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CompareError::Io {
            path: parent.to_path_buf(),
            action: "create report parent directory",
            source,
        })?;
    }
    let mut writer = BufWriter::new(File::create(path).map_err(|source| CompareError::Io {
        path: path.to_path_buf(),
        action: "create report",
        source,
    })?);
    writeln!(writer, "# SkillTable Evidence Comparison")?;
    writeln!(writer)?;
    writeln!(writer, "## Confirmed Mod Package Contents")?;
    for entry in &report.pak_entries {
        writeln!(
            writer,
            "- `{}` size={} encrypted={}",
            entry.path, entry.size, entry.encrypted
        )?;
    }
    writeln!(writer)?;
    writeln!(writer, "## Inferred SkillTable Symbols")?;
    for value in report.row_candidates.iter().take(240) {
        writeln!(writer, "- `{value}`")?;
    }
    writeln!(writer)?;
    writeln!(writer, "## Base-Game Focus Hits")?;
    for (term, hits) in &report.focus_hits {
        writeln!(writer, "### `{term}`")?;
        for hit in hits.iter().take(40) {
            writeln!(
                writer,
                "- `{}` from `{}` ({})",
                hit.raw_string, hit.source_package, hit.source_kind
            )?;
        }
    }
    writeln!(writer)?;
    writeln!(writer, "## Io Store List Hits")?;
    if report.iostore_focus_hits.is_empty() {
        writeln!(writer, "- No Io Store list input or no focus hits.")?;
    } else {
        for (term, hits) in &report.iostore_focus_hits {
            writeln!(writer, "### `{term}`")?;
            for hit in hits.iter().take(80) {
                writeln!(
                    writer,
                    "- `{}` from `{}` ({}) size={}",
                    hit.path.as_deref().unwrap_or("<no path>"),
                    hit.container,
                    hit.chunk_type,
                    hit.size
                )?;
            }
        }
    }
    writeln!(writer)?;
    writeln!(writer, "## Base Overlap Strings")?;
    for value in &report.base_overlap {
        writeln!(writer, "- `{value}`")?;
    }
    writeln!(writer)?;
    writeln!(writer, "## Unresolved")?;
    for item in &report.unresolved {
        writeln!(writer, "- {item}")?;
    }
    Ok(())
}
