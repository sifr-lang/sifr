use super::*;

#[test]
fn attached_api_set_membership_checks_the_stored_canonical_identity() {
    let mut defs = ExternalDefs::default();
    defs.attached_api_sets
        .entry("declared".to_string())
        .or_default()
        .insert(
            "Api".to_string(),
            sifr_ir::AttachedApiSetDeclaration {
                identity: sifr_ir::AttachedApiSetIdentity {
                    module: "actual".to_string(),
                    symbol: "Api".to_string(),
                },
                range: ruff_text_size::TextRange::default(),
            },
        );

    assert!(
        !defs.contains_attached_api_set(&sifr_ir::AttachedApiSetIdentity {
            module: "declared".to_string(),
            symbol: "Api".to_string(),
        })
    );
}
