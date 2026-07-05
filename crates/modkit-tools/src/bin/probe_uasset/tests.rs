#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_strings_extracts_ascii_runs() {
        assert_eq!(
            visible_strings(b"\0JustParry1\0xx\0ComboParry\0"),
            vec!["JustParry1".to_owned(), "ComboParry".to_owned()]
        );
    }

    #[test]
    fn interesting_reference_matches_skill_symbols() {
        assert!(interesting_reference("SBSkillTableProperty"));
        assert!(interesting_reference("ESBSkillType"));
        assert!(interesting_reference("JustParry1"));
        assert!(!interesting_reference("Weather"));
    }

    #[test]
    fn reads_fstring_and_fname() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&6i32.to_le_bytes());
        bytes.extend_from_slice(b"Name\0\0");
        let mut cursor = ByteCursor::new(&bytes);
        assert_eq!(cursor.read_fstring().unwrap(), "Name\0");

        let names = vec![NameEntry {
            index: 0,
            value: "SkillTable".to_owned(),
            flags: 0,
        }];
        let mut fname_bytes = Vec::new();
        fname_bytes.extend_from_slice(&0i32.to_le_bytes());
        fname_bytes.extend_from_slice(&0i32.to_le_bytes());
        let mut cursor = ByteCursor::new(&fname_bytes);
        assert_eq!(cursor.read_fname(&names).unwrap(), "SkillTable");
    }
}
