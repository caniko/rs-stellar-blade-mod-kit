#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_generated_candidate_line() {
        let line = "{\"schema_version\":1,\"evidence_kind\":\"visible_string_candidate\",\"source_path\":\"/tmp/pakchunk0.utoc\",\"source_package\":\"pakchunk0\",\"source_kind\":\"utoc\",\"byte_offset\":123,\"matched_terms\":[\"parry\",\"range\"],\"raw_value\":\"Result_Hit_JustParry.uasset\"}";
        let parsed = parse_candidate_line(line).unwrap();

        assert_eq!(parsed.evidence_kind, "visible_string_candidate");
        assert_eq!(parsed.source_package.as_deref(), Some("pakchunk0"));
        assert_eq!(parsed.byte_offset, Some(123));
        assert_eq!(parsed.matched_terms, vec!["parry", "range"]);
        assert_eq!(parsed.raw_value, "Result_Hit_JustParry.uasset");
    }

    #[test]
    fn parses_null_fields_from_loose_candidate() {
        let line = "{\"schema_version\":1,\"evidence_kind\":\"loose_file_candidate\",\"source_path\":\"/tmp/SkillPreview_JustParry1.bk2\",\"source_package\":null,\"source_kind\":\"Movies\",\"byte_offset\":null,\"matched_terms\":[\"skill\",\"parry\"],\"raw_value\":\"SkillPreview_JustParry1.bk2\"}";
        let parsed = parse_candidate_line(line).unwrap();

        assert_eq!(parsed.source_package, None);
        assert_eq!(parsed.byte_offset, None);
        assert_eq!(parsed.source_kind.as_deref(), Some("Movies"));
    }

    #[test]
    fn candidate_prefix_uses_last_path_component_and_first_token() {
        assert_eq!(
            candidate_prefix("/Game/Local/Data/SkillTable"),
            "SkillTable"
        );
        assert_eq!(candidate_prefix("EVE_M_BotRange_Result.uasset"), "EVE");
    }

    #[test]
    fn analysis_deduplicates_by_evidence_package_kind_and_value() {
        let candidate = Candidate {
            evidence_kind: "visible_string_candidate".to_owned(),
            source_path: "/tmp/a.utoc".to_owned(),
            source_package: Some("pakchunk0".to_owned()),
            source_kind: Some("utoc".to_owned()),
            byte_offset: Some(1),
            matched_terms: vec!["parry".to_owned()],
            raw_value: "Result_Hit_JustParry.uasset".to_owned(),
        };
        let mut duplicate = candidate.clone();
        duplicate.byte_offset = Some(9);
        let analysis = Analysis::build(vec![candidate, duplicate]);

        assert_eq!(analysis.total_records, 2);
        assert_eq!(analysis.unique_records.len(), 1);
        assert_eq!(analysis.unique_records[0].occurrences, 2);
    }
}
