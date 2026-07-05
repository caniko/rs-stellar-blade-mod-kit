fn write_summary(path: &Path, package: &PackageProbe) -> Result<(), UassetError> {
    let mut writer = writer(path)?;
    writeln!(writer, "# SkillTable UAsset Probe")?;
    writeln!(writer)?;
    writeln!(writer, "- UAsset: `{}`", package.uasset_path.display())?;
    writeln!(writer, "- UExp: `{}`", package.uexp_path.display())?;
    writeln!(writer, "- UAsset size: {}", package.uasset_size)?;
    writeln!(writer, "- UExp size: {}", package.uexp_size)?;
    writeln!(writer, "- Package tag: `0x{:08x}`", package.summary.tag)?;
    writeln!(
        writer,
        "- Legacy file version: {}",
        package.summary.legacy_file_version
    )?;
    writeln!(
        writer,
        "- Total header size: {}",
        package.summary.total_header_size
    )?;
    writeln!(writer, "- Package name: `{}`", package.summary.package_name)?;
    writeln!(
        writer,
        "- Package flags: `0x{:08x}`",
        package.summary.package_flags
    )?;
    writeln!(
        writer,
        "- Name map: count={} offset={}",
        package.summary.name_count, package.summary.name_offset
    )?;
    writeln!(
        writer,
        "- Imports: count={} offset={}",
        package.summary.import_count, package.summary.import_offset
    )?;
    writeln!(
        writer,
        "- Exports: count={} offset={}",
        package.summary.export_count, package.summary.export_offset
    )?;
    writeln!(
        writer,
        "- Depends offset: {}",
        package.summary.depends_offset
    )?;
    writeln!(writer)?;
    writeln!(writer, "## Detected Classes And Symbols")?;
    for value in detected_references(package) {
        writeln!(writer, "- `{value}`")?;
    }
    Ok(())
}

fn write_names(path: &Path, package: &PackageProbe) -> Result<(), UassetError> {
    let mut writer = writer(path)?;
    for name in &package.names {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"index\":{},\"value\":{},\"flags\":{}}}",
            name.index,
            json_string(&name.value),
            name.flags
        )?;
    }
    Ok(())
}

fn write_imports(path: &Path, package: &PackageProbe) -> Result<(), UassetError> {
    let mut writer = writer(path)?;
    for import in &package.imports {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"index\":{},\"class_package\":{},\"class_name\":{},\"outer_index\":{},\"object_name\":{}}}",
            import.index,
            json_string(&import.class_package),
            json_string(&import.class_name),
            import.outer_index,
            json_string(&import.object_name)
        )?;
    }
    Ok(())
}

fn write_exports(path: &Path, package: &PackageProbe) -> Result<(), UassetError> {
    let mut writer = writer(path)?;
    for export in &package.exports {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"index\":{},\"class_index\":{},\"super_index\":{},\"outer_index\":{},\"object_name\":{},\"object_flags\":{},\"serial_size\":{},\"serial_offset\":{}}}",
            export.index,
            export.class_index,
            export.super_index,
            export.outer_index,
            json_string(&export.object_name),
            export.object_flags,
            export.serial_size,
            export.serial_offset
        )?;
    }
    Ok(())
}

fn write_row_candidates(path: &Path, package: &PackageProbe) -> Result<(), UassetError> {
    let mut writer = writer(path)?;
    let mut candidates = BTreeSet::new();
    for name in &package.names {
        if interesting_reference(&name.value) {
            candidates.insert(name.value.clone());
        }
    }
    for value in &package.external_payload_strings {
        candidates.insert(value.clone());
    }
    for candidate in candidates {
        writeln!(
            writer,
            "{{\"schema_version\":1,\"value\":{},\"matched_terms\":{}}}",
            json_string(&candidate),
            json_array_strings(matched_terms(&candidate))
        )?;
    }
    Ok(())
}
