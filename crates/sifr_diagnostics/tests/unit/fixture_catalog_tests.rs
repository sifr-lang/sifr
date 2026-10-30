use std::path::Path;

use serde_json::{Value, json};

use super::fixture_catalog::FixtureCatalog;

const CODE: &str = "SIFR-TYPE-0002";

fn catalog() -> Value {
    json!({"schema_version": 1, "codes": [
        {"code": CODE, "representative_fixture": "Cargo.toml"}
    ]})
}

#[test]
fn verification_catalog_supplies_fixtures_without_production_metadata() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures =
        FixtureCatalog::parse(root, [CODE].into_iter(), &catalog()).expect("valid catalog");
    assert_eq!(fixtures.get(CODE), Some("Cargo.toml"));
    assert_eq!(fixtures.get("SIFR-TYPE-0000"), None);
}

#[test]
fn malformed_and_incomplete_catalogs_are_rejected() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut invalid = vec![
        Value::Null,
        json!({}),
        json!({"schema_version": true, "codes": []}),
        json!({"schema_version": 1, "codes": []}),
        json!({"schema_version": 1, "codes": [null]}),
    ];
    let mut duplicate = catalog();
    let row = duplicate["codes"][0].clone();
    duplicate["codes"].as_array_mut().expect("array").push(row);
    invalid.push(duplicate);
    for (key, values) in [
        ("code", vec![Value::Null, json!("SIFR-TYPE-0000")]),
        (
            "representative_fixture",
            vec![
                Value::Null,
                json!(""),
                json!("../Cargo.toml"),
                json!("/Cargo.toml"),
                json!("missing.rs"),
                json!("a`b"),
                json!("a\nb"),
                json!("Cargo.toml::symbol"),
                json!("src/lib.rs::"),
                json!("src/lib.rs::a::b"),
            ],
        ),
    ] {
        for value in values {
            let mut bad = catalog();
            bad["codes"][0][key] = value;
            invalid.push(bad);
        }
    }
    for value in invalid {
        assert!(
            FixtureCatalog::parse(root, [CODE].into_iter(), &value).is_err(),
            "{value}"
        );
    }
}
