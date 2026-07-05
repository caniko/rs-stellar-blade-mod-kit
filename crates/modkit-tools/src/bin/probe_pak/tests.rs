#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_ascii_fstring() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&6i32.to_le_bytes());
        bytes.extend_from_slice(b"Test\0\0");
        let mut cursor = ByteCursor::new(&bytes);

        assert_eq!(cursor.read_fstring().unwrap(), "Test\0");
    }

    #[test]
    fn parses_index_with_prefix_seed() {
        let mut index = Vec::new();
        index.extend_from_slice(&0x9e2a83c1u32.to_le_bytes());
        push_fstring(&mut index, "../../../");
        index.extend_from_slice(&1i32.to_le_bytes());
        push_fstring(&mut index, "SB/Content/Local/Data/SkillTable.uasset");
        index.extend_from_slice(&0i64.to_le_bytes());
        index.extend_from_slice(&123i64.to_le_bytes());
        index.extend_from_slice(&123i64.to_le_bytes());
        index.extend_from_slice(&0u32.to_le_bytes());
        index.extend_from_slice(&[7; 20]);
        index.push(0);
        index.extend_from_slice(&0u32.to_le_bytes());

        let (seed, mount_point, entries) = parse_index(&index).unwrap();

        assert_eq!(seed, Some(0x9e2a83c1));
        assert_eq!(mount_point, "../../../");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "SB/Content/Local/Data/SkillTable.uasset");
        assert_eq!(entries[0].size, 123);
    }

    #[test]
    fn safe_output_path_rejects_parent_components() {
        assert!(safe_output_path(Path::new("out"), "../bad").is_err());
        assert_eq!(
            safe_output_path(Path::new("out"), "SB/Content/A.uasset").unwrap(),
            PathBuf::from("out/SB/Content/A.uasset")
        );
    }

    #[test]
    fn hex_formats_bytes() {
        assert_eq!(hex(&[0, 1, 0xab, 0xff]), "0001abff");
    }

    #[test]
    fn package_format_identifies_supported_and_gated_backends() {
        assert_eq!(package_format(Path::new("a.pak")), PackageFormat::Pak);
        assert_eq!(
            package_format(Path::new("a.utoc")),
            PackageFormat::IoStoreToc
        );
        assert_eq!(
            package_format(Path::new("a.ucas")),
            PackageFormat::IoStoreContainer
        );
        assert_eq!(package_format(Path::new("a.bin")), PackageFormat::Unknown);
    }

    fn push_fstring(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&((value.len() + 1) as i32).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(0);
    }
}
