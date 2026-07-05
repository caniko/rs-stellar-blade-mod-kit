#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_terms_trims_deduplicates_and_lowercases() {
        assert_eq!(
            parse_terms(" Parry,skill,parry, Guard ").unwrap(),
            vec!["parry", "skill", "guard"]
        );
    }

    #[test]
    fn extract_visible_strings_reports_offsets() {
        let bytes = b"\0abc\0Result_Hit_JustParry.uasset\0xy";
        let mut records = Vec::new();
        extract_visible_strings(bytes, |offset, value| {
            records.push((offset, value.to_owned()));
        });

        assert_eq!(records, vec![(5, "Result_Hit_JustParry.uasset".to_owned())]);
    }

    #[test]
    fn matched_terms_uses_case_insensitive_substrings() {
        let terms = parse_terms("parry,range,guard").unwrap();
        assert_eq!(
            matched_terms("EVE_M_BotRange_Result_Hit_JustParry.uasset", &terms),
            vec!["parry", "range"]
        );
    }

    #[test]
    fn json_escape_handles_control_characters() {
        assert_eq!(
            json_string("Parry\"Window\\A\nB"),
            "\"Parry\\\"Window\\\\A\\nB\""
        );
    }
}
