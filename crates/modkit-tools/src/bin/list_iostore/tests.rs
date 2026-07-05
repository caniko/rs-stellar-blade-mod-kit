#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_retoc_list_line_with_path() {
        let line = "pakchunk0 17b216c2388eefa600000002 8cc33152c8258b17095a4f329d936a14da5561be000000000000000000000000 12028989504155464215 ExportBundleData 301498 ../../../SB/Content/Local/Data/SkillTable.uasset";

        let entry = parse_list_line(line).unwrap();

        assert_eq!(entry.container, "pakchunk0");
        assert_eq!(entry.chunk_type, "ExportBundleData");
        assert_eq!(entry.size, 301498);
        assert_eq!(
            entry.path.as_deref(),
            Some("../../../SB/Content/Local/Data/SkillTable.uasset")
        );
    }

    #[test]
    fn parses_retoc_list_line_without_path_or_package() {
        let line = "pakchunk0 9ec3817eb267fa8f0000000a 6e9c360b5dc033f4332023ddb3d99e27dd4934c5000000000000000000000000 - ContainerHeader 96 -";

        let entry = parse_list_line(line).unwrap();

        assert_eq!(entry.package_id, None);
        assert_eq!(entry.path, None);
    }

    #[test]
    fn focus_hits_match_paths_case_insensitively() {
        let entries = vec![RetocEntry {
            container: "pakchunk0".to_owned(),
            chunk_id: "abc".to_owned(),
            hash: "def".to_owned(),
            package_id: None,
            chunk_type: "ExportBundleData".to_owned(),
            size: 1,
            path: Some("../../../SB/Content/Local/Data/SkillTable.uasset".to_owned()),
        }];

        let hits = focus_hits(&entries, &["skilltable".to_owned()]);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].1, vec!["skilltable"]);
    }

    #[test]
    fn parse_terms_trims_deduplicates_and_sorts() {
        assert_eq!(
            parse_terms(" GuardSkill,SkillTable,GuardSkill ,,"),
            vec!["GuardSkill", "SkillTable"]
        );
    }
}
