#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_fields() {
        let line = "{\"path\":\"SkillTable.uasset\",\"size\":156665,\"encrypted\":false}";
        assert_eq!(
            required_string_field(line, "path").unwrap(),
            "SkillTable.uasset"
        );
        assert_eq!(required_u64_field(line, "size").unwrap(), 156665);
        assert!(!required_bool_field(line, "encrypted").unwrap());
    }

    #[test]
    fn report_detects_base_overlap() {
        let report = CompareReport::build(
            vec![],
            BTreeSet::from(["JustParry1".to_owned()]),
            vec![VisibleStringRecord {
                source_package: "pakchunk0-WindowsNoEditor".to_owned(),
                source_kind: "utoc".to_owned(),
                raw_string: "JustParry1".to_owned(),
            }],
            vec![],
        );
        assert!(report.base_overlap.contains("JustParry1"));
        assert!(report.focus_hits.contains_key("JustParry1"));
    }

    #[test]
    fn report_includes_iostore_focus_hits() {
        let report = CompareReport::build(
            vec![],
            BTreeSet::new(),
            vec![],
            vec![IoStoreEntry {
                container: "pakchunk0-WindowsNoEditor".to_owned(),
                chunk_type: "ExportBundleData".to_owned(),
                size: 100,
                path: Some("../../../SB/Content/Local/Data/SkillTable.uasset".to_owned()),
            }],
        );

        assert!(report.iostore_focus_hits.contains_key("SkillTable"));
    }
}
