#![allow(clippy::expect_used, clippy::unwrap_used)]

use sifr_sql_contract::{
    Cardinality, CheckedCodecBinding, CodecContract, CodecDecoderSignature, CodecEncoderSignature,
    CodecFunctionIdentity, CodecIdentity, ConflictBatchBehavior, DatabaseType, EffectContract,
    IntegerSign, IntegerWidth, NullCodecBehavior, ObjectId, PanicContainment, ProjectionPolicy,
    ProjectionStability, ProviderParameterLimit, PublicQueryChangeKind, QueryEffect,
    QueryParameterSlot, QuerySignatureArtifact, QuerySignatureEntry, SifrType,
    ValuesFragmentContract, WireFormatIdentity, compare_query_signatures,
};
use std::collections::BTreeSet;

#[test]
fn values_batches_require_explicit_provider_safe_chunking() {
    let limit = ProviderParameterLimit::new(6).unwrap();
    assert!(
        ValuesFragmentContract::checked(
            columns(),
            4,
            2,
            ConflictBatchBehavior::AbortBatch,
            limit,
            None,
        )
        .is_err()
    );
    let checked = ValuesFragmentContract::checked(
        columns(),
        4,
        2,
        ConflictBatchBehavior::UpdateConflicts,
        limit,
        Some(3),
    )
    .unwrap();
    assert_eq!(checked.chunks.len(), 2);
    assert_eq!(checked.chunks[0].parameter_count, 6);
    assert_eq!(checked.chunks[1].parameter_count, 2);
}

#[test]
fn projection_policy_expands_private_star_and_rejects_exported_star() {
    let projection = ProjectionStability {
        emitted_columns: vec!["id".to_string(), "name".to_string()],
        used_star: true,
        unstable_expressions: Vec::new(),
        duplicate_names: BTreeSet::new(),
        schema_sensitive_types: BTreeSet::new(),
    };
    assert_eq!(
        projection
            .clone()
            .validate(ProjectionPolicy::Private)
            .unwrap(),
        ["id", "name"]
    );
    let error = projection.validate(ProjectionPolicy::Exported).unwrap_err();
    assert!(error.machine_fix.unwrap().contains("explicit"));
}

#[test]
fn checked_custom_codec_has_one_exact_owned_fallible_inverse() {
    let codec = CodecIdentity::new("app.codec.token.v1").unwrap();
    let database_type = DatabaseType::Custom {
        identity: ObjectId::new("app.token"),
        codec: codec.clone(),
    };
    let contract = CodecContract {
        identity: codec,
        database_type: database_type.clone(),
        sifr_type: SifrType::Custom {
            identity: "app.Token".to_string(),
        },
        server_profiles: BTreeSet::from(["postgresql-18".to_string()]),
        encode_error: "app.TokenEncodeError".to_string(),
        decode_error: "app.TokenDecodeError".to_string(),
        null_behavior: NullCodecBehavior::PassThrough,
        wire_format: WireFormatIdentity::new("app.token.binary.v1").unwrap(),
        panic_containment: PanicContainment::CatchAndRedact,
    };
    let binding = CheckedCodecBinding::checked(
        &contract,
        CodecFunctionIdentity::new("app.token.encode").unwrap(),
        CodecFunctionIdentity::new("app.token.decode").unwrap(),
        &CodecEncoderSignature {
            input: contract.sifr_type.clone(),
            output: database_type.clone(),
            owned: true,
            fallible: true,
        },
        &CodecDecoderSignature {
            input: database_type.clone(),
            output: contract.sifr_type.clone(),
            owned: true,
            fallible: true,
        },
    )
    .unwrap();
    assert_eq!(binding.database_type, database_type);
}

#[test]
fn signature_artifacts_are_canonical_and_report_breaking_changes() {
    let baseline = QuerySignatureArtifact::build("app@1", [signature(Cardinality::MANY)]).unwrap();
    let candidate =
        QuerySignatureArtifact::build("app@1", [signature(Cardinality::AT_MOST_ONE)]).unwrap();
    assert_eq!(
        compare_query_signatures(&baseline, &candidate)[0].kind,
        PublicQueryChangeKind::Cardinality
    );
    assert_eq!(
        serde_json::from_slice::<QuerySignatureArtifact>(&baseline.canonical_json().unwrap())
            .unwrap(),
        baseline
    );
    assert!(
        baseline.entries["app.queries::find_users"]
            .schema_dependencies
            .contains(&ObjectId::new("public.audit"))
    );
}

fn columns() -> Vec<(ObjectId, SifrType)> {
    vec![
        (
            ObjectId::new("public.users.id"),
            SifrType::FixedInteger {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits64,
            },
        ),
        (ObjectId::new("public.users.name"), SifrType::Str),
    ]
}

fn signature(cardinality: Cardinality) -> QuerySignatureEntry {
    QuerySignatureEntry {
        module: "app.queries".to_string(),
        symbol: "find_users".to_string(),
        template_identity: "a".repeat(64),
        profile_identity: "app.Schema".to_string(),
        schema_fingerprint: "b".repeat(64),
        parameters: vec![QueryParameterSlot {
            slot: 0,
            sifr_type: SifrType::FixedInteger {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits64,
            },
        }],
        result: vec![("id".to_string(), columns()[0].1.clone())],
        cardinality,
        effects: EffectContract::new(
            QueryEffect::ReadWrite,
            BTreeSet::from([ObjectId::new("public.users")]),
            BTreeSet::from([ObjectId::new("public.audit")]),
        )
        .unwrap(),
        schema_dependencies: BTreeSet::from([
            ObjectId::new("public.audit"),
            ObjectId::new("public.users"),
        ]),
    }
}
