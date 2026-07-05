fn parse_package(uasset_path: &Path, uexp_path: &Path) -> Result<PackageProbe, UassetError> {
    let bytes = fs::read(uasset_path).map_err(|source| UassetError::Io {
        path: uasset_path.to_path_buf(),
        action: "read uasset",
        source,
    })?;
    let uasset_size = bytes.len() as u64;
    let uexp_bytes = fs::read(uexp_path).map_err(|source| UassetError::Io {
        path: uexp_path.to_path_buf(),
        action: "read uexp",
        source,
    })?;
    let uexp_size = uexp_bytes.len() as u64;

    let mut cursor = ByteCursor::new(&bytes);
    let tag = cursor.read_u32()?;
    if tag != PACKAGE_TAG {
        return Err(UassetError::Unsupported(format!(
            "unexpected package tag 0x{tag:08x}; expected 0x{PACKAGE_TAG:08x}"
        )));
    }
    let legacy_file_version = cursor.read_i32()?;
    let _legacy_ue3_version = cursor.read_i32()?;
    let _file_version_ue4 = cursor.read_i32()?;
    let _file_version_licensee_ue4 = cursor.read_i32()?;
    let custom_version_count = cursor.read_i32()?;
    if custom_version_count != 0 {
        return Err(UassetError::Unsupported(format!(
            "custom version container count {custom_version_count} is not supported yet"
        )));
    }
    let total_header_size = cursor.read_i32()?;
    let package_name = cursor.read_fstring()?;
    let package_flags = cursor.read_u32()?;
    let name_count = cursor.read_i32()?;
    let name_offset = cursor.read_i32()?;
    let _gatherable_text_data_count = cursor.read_i32()?;
    let _gatherable_text_data_offset = cursor.read_i32()?;
    let export_count = cursor.read_i32()?;
    let export_offset = cursor.read_i32()?;
    let import_count = cursor.read_i32()?;
    let import_offset = cursor.read_i32()?;
    let depends_offset = cursor.read_i32()?;

    validate_range("name map", name_offset, name_count, &bytes)?;
    validate_range("import map", import_offset, import_count, &bytes)?;
    validate_range("export map", export_offset, export_count, &bytes)?;

    let summary = PackageSummary {
        tag,
        legacy_file_version,
        total_header_size,
        package_name,
        package_flags,
        name_count,
        name_offset,
        export_count,
        export_offset,
        import_count,
        import_offset,
        depends_offset,
    };
    let names = parse_names(&bytes, name_offset as usize, name_count as usize)?;
    let imports = parse_imports(
        &bytes,
        import_offset as usize,
        import_count as usize,
        &names,
    )?;
    let exports = parse_exports(
        &bytes,
        export_offset as usize,
        export_count as usize,
        &names,
    )?;
    let external_payload_strings = visible_strings(&uexp_bytes)
        .into_iter()
        .filter(|value| interesting_reference(value))
        .collect();

    Ok(PackageProbe {
        uasset_path: uasset_path.to_path_buf(),
        uexp_path: uexp_path.to_path_buf(),
        uasset_size,
        uexp_size,
        summary,
        names,
        imports,
        exports,
        external_payload_strings,
    })
}

fn validate_range(label: &str, offset: i32, count: i32, bytes: &[u8]) -> Result<(), UassetError> {
    if offset < 0 || count < 0 || offset as usize > bytes.len() {
        return Err(UassetError::Unsupported(format!(
            "invalid {label} range: offset={offset} count={count} file_size={}",
            bytes.len()
        )));
    }
    Ok(())
}

fn parse_names(bytes: &[u8], offset: usize, count: usize) -> Result<Vec<NameEntry>, UassetError> {
    let mut cursor = ByteCursor::new_at(bytes, offset)?;
    let mut names = Vec::with_capacity(count);
    for index in 0..count {
        let value = cursor.read_fstring()?;
        let flags = cursor.read_u32()?;
        names.push(NameEntry {
            index,
            value,
            flags,
        });
    }
    Ok(names)
}

fn parse_imports(
    bytes: &[u8],
    offset: usize,
    count: usize,
    names: &[NameEntry],
) -> Result<Vec<ImportEntry>, UassetError> {
    let mut cursor = ByteCursor::new_at(bytes, offset)?;
    let mut imports = Vec::with_capacity(count);
    for index in 0..count {
        let class_package = cursor.read_fname(names)?;
        let class_name = cursor.read_fname(names)?;
        let outer_index = cursor.read_i32()?;
        let object_name = cursor.read_fname(names)?;
        imports.push(ImportEntry {
            index,
            class_package,
            class_name,
            outer_index,
            object_name,
        });
    }
    Ok(imports)
}

fn parse_exports(
    bytes: &[u8],
    offset: usize,
    count: usize,
    names: &[NameEntry],
) -> Result<Vec<ExportEntry>, UassetError> {
    let mut cursor = ByteCursor::new_at(bytes, offset)?;
    let mut exports = Vec::with_capacity(count);
    for index in 0..count {
        let class_index = cursor.read_i32()?;
        let super_index = cursor.read_i32()?;
        let _template_index = cursor.read_i32()?;
        let outer_index = cursor.read_i32()?;
        let object_name = cursor.read_fname(names)?;
        let object_flags = cursor.read_u32()?;
        let serial_size = cursor.read_i64()?;
        let serial_offset = cursor.read_i64()?;
        exports.push(ExportEntry {
            index,
            class_index,
            super_index,
            outer_index,
            object_name,
            object_flags,
            serial_size,
            serial_offset,
        });
        cursor.skip(104usize.saturating_sub(40))?;
    }
    Ok(exports)
}
