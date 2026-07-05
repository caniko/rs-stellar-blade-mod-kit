//! Combat reconnaissance filters.

use bouldy_runtime::{
    DiscoveryCandidate, DiscoveryContext, DiscoveryQuery, DISCOVERY_KIND_CLASS,
    DISCOVERY_KIND_FUNCTION, DISCOVERY_KIND_OBJECT, DISCOVERY_KIND_PROPERTY,
};

/// Search terms used to find candidate Stellar Blade combat symbols.
pub const COMBAT_SEARCH_TERMS: &[&str] = &[
    "player",
    "eve",
    "combat",
    "enemy",
    "combatant",
    "parry",
    "perfectparry",
    "guard",
    "block",
    "dodge",
    "evade",
    "avoid",
    "damage",
    "hit",
    "reaction",
    "stagger",
    "balance",
    "groggy",
    "lockon",
    "target",
    "input",
    "window",
    "attack",
    "intent",
    "punish",
];

const EXPORT_CHANNEL: &str = "stellar_blade.combat_recon";

/// Scan and export combat-related discovery candidates.
pub fn scan_combat_candidates(discovery: DiscoveryContext) -> usize {
    let query = combat_query();
    discovery.scan_and_export_json_records(&query, EXPORT_CHANNEL, is_combat_candidate)
}

/// Build the default combat candidate query.
pub fn combat_query() -> DiscoveryQuery {
    DiscoveryQuery::new(COMBAT_SEARCH_TERMS.iter().copied())
        .with_kind_mask(
            DISCOVERY_KIND_OBJECT
                | DISCOVERY_KIND_CLASS
                | DISCOVERY_KIND_FUNCTION
                | DISCOVERY_KIND_PROPERTY,
        )
        .with_max_results(512)
}

/// Return whether a candidate looks relevant to combat-loop discovery.
pub fn is_combat_candidate(candidate: &DiscoveryCandidate) -> bool {
    candidate.matches_any_term(COMBAT_SEARCH_TERMS.iter().copied())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bouldy_runtime::{DISCOVERY_KIND_FUNCTION, DISCOVERY_KIND_PROPERTY};

    #[test]
    fn query_uses_combat_terms_and_expected_kinds() {
        let query = combat_query();
        assert!(query.terms().contains(&"parry".to_owned()));
        assert!(query.terms().contains(&"dodge".to_owned()));
        assert_eq!(query.max_results(), 512);
        assert_ne!(query.kind_mask() & DISCOVERY_KIND_FUNCTION, 0);
        assert_ne!(query.kind_mask() & DISCOVERY_KIND_PROPERTY, 0);
    }

    #[test]
    fn candidate_filter_matches_combat_terms_across_fields() {
        let candidate = DiscoveryCandidate {
            kind: DISCOVERY_KIND_FUNCTION,
            name: Some("CanPerfectParry".to_owned()),
            path: Some("/Script/SB.PlayerCombatComponent".to_owned()),
            owner: Some("Eve".to_owned()),
            flags: 0,
            schema_version: 2,
            chunk_index: Some(1),
            object_index: Some(24),
        };
        assert!(is_combat_candidate(&candidate));
    }

    #[test]
    fn candidate_filter_rejects_unrelated_symbols() {
        let candidate = DiscoveryCandidate {
            kind: DISCOVERY_KIND_PROPERTY,
            name: Some("WeatherIntensity".to_owned()),
            path: Some("/Script/SB.WorldLighting".to_owned()),
            owner: Some("Environment".to_owned()),
            flags: 0,
            schema_version: 2,
            chunk_index: Some(2),
            object_index: Some(48),
        };
        assert!(!is_combat_candidate(&candidate));
    }

    #[test]
    fn candidate_record_format_is_stable_and_escaped() {
        let candidate = DiscoveryCandidate {
            kind: DISCOVERY_KIND_FUNCTION,
            name: Some("Parry\"Window".to_owned()),
            path: Some("/Script/SB\\Combat".to_owned()),
            owner: None,
            flags: 7,
            schema_version: 2,
            chunk_index: Some(3),
            object_index: None,
        };
        assert_eq!(
            candidate.record_json(),
            "{\"schema_version\":2,\"kind\":4,\"name\":\"Parry\\\"Window\",\"path\":\"/Script/SB\\\\Combat\",\"owner\":null,\"flags\":7,\"chunk_index\":3,\"object_index\":null}"
        );
    }
}
