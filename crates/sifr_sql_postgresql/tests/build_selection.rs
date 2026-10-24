#[path = "../src/build_selection.rs"]
mod build_selection;

use sifr_sql_postgresql::{LibpgQueryParser, PostgresParser};

#[test]
fn no_feature_preserves_the_default_and_each_major_is_selectable() -> std::io::Result<()> {
    assert_eq!(build_selection::select_major(&[], false)?, 18);
    for major in 13..=18 {
        assert_eq!(build_selection::select_major(&[major], false)?, major);
    }
    Ok(())
}

#[test]
fn ambiguous_and_unsupported_selections_fail() {
    for first in 13..=18 {
        for second in 13..=18 {
            assert!(build_selection::select_major(&[first, second], false).is_err());
        }
    }
    for major in [0, 12, 19, u16::MAX] {
        assert!(build_selection::select_major(&[major], false).is_err());
    }
}

#[test]
fn legacy_environment_selection_is_rejected() {
    assert!(build_selection::select_major(&[], true).is_err());
    for major in 13..=18 {
        assert!(build_selection::select_major(&[major], true).is_err());
    }
}

#[test]
fn native_parser_matches_selected_and_requested_major() -> Result<(), Box<dyn std::error::Error>> {
    let features = [
        (13, cfg!(feature = "postgresql-13")),
        (14, cfg!(feature = "postgresql-14")),
        (15, cfg!(feature = "postgresql-15")),
        (16, cfg!(feature = "postgresql-16")),
        (17, cfg!(feature = "postgresql-17")),
        (18, cfg!(feature = "postgresql-18")),
    ]
    .into_iter()
    .filter_map(|(major, enabled)| enabled.then_some(major))
    .collect::<Vec<_>>();
    let selected = build_selection::select_major(&features, false)?;
    assert_eq!(LibpgQueryParser.server_major(), selected);
    // Parsing also checks that the native C parse-tree version matches Rust.
    assert_eq!(LibpgQueryParser.parse("SELECT 1")?.len(), 1);
    if let Some(requested) = std::env::var_os("SIFR_POSTGRESQL_TEST_MAJOR") {
        let requested = requested
            .to_str()
            .ok_or("invalid requested PostgreSQL major")?;
        assert_eq!(LibpgQueryParser.server_major(), requested.parse::<u16>()?);
    }
    Ok(())
}
