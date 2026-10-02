use super::*;

#[test]
fn component_codec_payload_does_not_conflict_with_nullable_result_or_source_integer() {
    let database = signed(IntegerWidth::Bits64);
    let payload = canonical_read_type(&database).expect("integer payload");
    let codec = codec_identity("sqlite.int64.binary.v1");
    let mut analysis = ProviderAnalysis {
        server_profile: "sqlite-3.53.4".into(),
        normalized_statement: "SELECT ?1".into(),
        parameters: vec![ProviderParameter {
            slot: 0,
            database_type: database.clone(),
            codec: codec.clone(),
            nullability: Nullability::Nullable,
        }],
        result_fields: vec![ProviderResultField {
            name: "value".into(),
            database_type: database,
            codec,
            sifr_type: SifrType::Union {
                members: BTreeSet::from([payload.clone(), SifrType::None]),
            },
            nullability: Nullability::Nullable,
            source_object: None,
        }],
        cardinality: Cardinality::EXACTLY_ONE,
        effects: EffectContract::new(QueryEffect::Read, BTreeSet::new(), BTreeSet::new())
            .expect("read effects"),
        accessed_objects: BTreeSet::new(),
        semantic_flags: BTreeSet::new(),
        required_capabilities: BTreeSet::from(["sql.query.select".into()]),
    };
    for source in [
        payload.clone(),
        SifrType::ExactInteger,
        SifrType::Union {
            members: BTreeSet::from([payload, SifrType::None]),
        },
    ] {
        let registry = sifr_sql_contract::component_codec_registry(&analysis, &[source])
            .expect("canonical payload shared by bind/result");
        analysis
            .validate(&registry)
            .expect("nullable result applies nullability once");
    }
    // A codec cannot silently claim a different database representation.
    analysis.result_fields[0].database_type = DatabaseType::Text {
        fixed: false,
        max_characters: None,
    };
    assert!(sifr_sql_contract::component_codec_registry(&analysis, &[SifrType::Str]).is_err());
}
