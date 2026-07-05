fn write_report(path: &Path, analysis: &ModExampleAnalysis) -> Result<(), ModAnalyzeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ModAnalyzeError::Io {
            path: parent.to_path_buf(),
            action: "create report parent directory",
            source,
        })?;
    }
    let mut writer = BufWriter::new(File::create(path).map_err(|source| ModAnalyzeError::Io {
        path: path.to_path_buf(),
        action: "create report",
        source,
    })?);

    writeln!(writer, "# Disabled Mod Example Analysis")?;
    writeln!(writer)?;
    writeln!(writer, "- Package: `{}`", analysis.mod_package)?;
    writeln!(
        writer,
        "- Visible string records: {}",
        analysis.mod_records.len()
    )?;
    writeln!(
        writer,
        "- Unique visible strings: {}",
        analysis.unique_mod_strings.len()
    )?;
    writeln!(
        writer,
        "- Unique base-game strings available for overlap check: {}",
        analysis.base_strings.len()
    )?;
    writeln!(
        writer,
        "- Base overlap strings: {}",
        analysis.base_overlaps.len()
    )?;
    writeln!(writer)?;

    writeln!(writer, "## Source Kinds")?;
    write_counts(&mut writer, &analysis.kind_counts)?;
    writeln!(writer)?;

    writeln!(writer, "## Interpretation")?;
    writeln!(
        writer,
        "- The disabled mod exposes `/Game/Local/Data/SkillTable`, `SkillTable.uasset`, `/Script/SB`, and `DataTable`, so it is best treated as a cooked DataTable override example."
    )?;
    writeln!(
        writer,
        "- The report is based on visible package metadata only. It does not decode row values or prove exact changed cells."
    )?;
    writeln!(
        writer,
        "- Strings that also appear in base packages are likely schema fields, enum values, asset names, or row aliases reused by the game."
    )?;
    writeln!(writer)?;

    for category in [
        "asset_paths",
        "asset_names",
        "script_paths",
        "table_schema",
        "combat_terms",
        "bool_like_fields",
        "game_symbols",
    ] {
        if let Some(values) = analysis.categories.get(category) {
            writeln!(
                writer,
                "## {}",
                category.replace('_', " ").to_ascii_titlecase()
            )?;
            write_string_list(&mut writer, values.iter(), 120)?;
            writeln!(writer)?;
        }
    }

    writeln!(writer, "## Base Overlap Sample")?;
    write_string_list(&mut writer, analysis.base_overlaps.iter(), 160)?;
    Ok(())
}

trait TitleCase {
    fn to_ascii_titlecase(&self) -> String;
}

impl TitleCase for str {
    fn to_ascii_titlecase(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn write_counts(
    writer: &mut BufWriter<File>,
    counts: &BTreeMap<String, usize>,
) -> Result<(), ModAnalyzeError> {
    let mut entries = counts.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
    for (name, count) in entries {
        writeln!(writer, "- `{name}`: {count}")?;
    }
    Ok(())
}

fn write_string_list<'a>(
    writer: &mut BufWriter<File>,
    values: impl IntoIterator<Item = &'a String>,
    limit: usize,
) -> Result<(), ModAnalyzeError> {
    for value in values.into_iter().take(limit) {
        writeln!(writer, "- `{value}`")?;
    }
    Ok(())
}
