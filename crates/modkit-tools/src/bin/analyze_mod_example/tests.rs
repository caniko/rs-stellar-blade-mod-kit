#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_visible_string_line() {
        let line = "{\"schema_version\":1,\"evidence_kind\":\"visible_string_candidate\",\"source_package\":\"SB_ImprovedPerfectDefense_Extended_P\",\"source_kind\":\"small_pak\",\"source_path\":\"/tmp/example.pak\",\"byte_offset\":85,\"matched_terms\":[\"skill\"],\"raw_string\":\"/Game/Local/Data/SkillTable\"}";
        let parsed = parse_visible_string_line(line).unwrap();

        assert_eq!(parsed.source_package, DEFAULT_MOD_PACKAGE);
        assert_eq!(parsed.byte_offset, 85);
        assert_eq!(parsed.raw_string, "/Game/Local/Data/SkillTable");
    }

    #[test]
    fn classifies_mod_strings() {
        assert_eq!(
            classify_string("/Game/Local/Data/SkillTable"),
            vec!["asset_paths", "table_schema", "combat_terms"]
        );
        assert_eq!(
            classify_string("bIgnoreBlockSkill"),
            vec!["combat_terms", "bool_like_fields"]
        );
    }

    #[test]
    fn detects_base_overlaps() {
        let records = vec![
            VisibleStringRecord {
                source_package: DEFAULT_MOD_PACKAGE.to_owned(),
                source_kind: "small_pak".to_owned(),
                source_path: "mod.pak".to_owned(),
                byte_offset: 1,
                raw_string: "JustParry1".to_owned(),
            },
            VisibleStringRecord {
                source_package: "pakchunk0-WindowsNoEditor".to_owned(),
                source_kind: "utoc".to_owned(),
                source_path: "base.utoc".to_owned(),
                byte_offset: 2,
                raw_string: "JustParry1".to_owned(),
            },
        ];
        let analysis = ModExampleAnalysis::build(records, DEFAULT_MOD_PACKAGE);

        assert!(analysis.base_overlaps.contains("JustParry1"));
    }
}
