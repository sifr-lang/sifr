#![allow(clippy::expect_used)]

use sifr_sql_contract::{
    BindCompatibility, BindRejection, Cardinality, CodecContract, CodecIdentity, CodecRegistry,
    CommonSqlDiagnostic, DatabaseType, DecimalRepresentation, EffectContract, EncodeCheck,
    FetchMethod, InputType, IntegerSign, IntegerWidth, NullCodecBehavior, Nullability, ObjectId,
    PanicContainment, ParameterType, ProviderAnalysis, ProviderParameter, ProviderResultField,
    QueryEffect, SifrType, SqliteStorageClass, WireFormatIdentity, bind_compatibility,
    canonical_read_type,
};
use std::collections::BTreeSet;

fn codec_identity(value: &str) -> CodecIdentity {
    CodecIdentity::new(value).expect("test codec identity should be valid")
}

fn wire_identity(value: &str) -> WireFormatIdentity {
    WireFormatIdentity::new(value).expect("test wire identity should be valid")
}

fn signed(width: IntegerWidth) -> DatabaseType {
    DatabaseType::Integer {
        sign: IntegerSign::Signed,
        width,
    }
}

fn input(value: SifrType) -> InputType {
    InputType {
        value,
        nullability: Nullability::NonNull,
    }
}

fn target(database: DatabaseType) -> ParameterType {
    ParameterType {
        database,
        nullability: Nullability::NonNull,
    }
}

#[test]
fn canonical_read_matrix_covers_every_locked_semantic_family() {
    let status = ObjectId::new("public.status");
    let custom_codec = codec_identity("app.money.v1");
    let cases = vec![
        (DatabaseType::Boolean, SifrType::Bool),
        (
            signed(IntegerWidth::Bits16),
            SifrType::FixedInteger {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits16,
            },
        ),
        (
            DatabaseType::Integer {
                sign: IntegerSign::Unsigned,
                width: IntegerWidth::Bits64,
            },
            SifrType::FixedInteger {
                sign: IntegerSign::Unsigned,
                width: IntegerWidth::Bits64,
            },
        ),
        (
            DatabaseType::Decimal {
                precision: Some(18),
                scale: Some(2),
                representation: DecimalRepresentation::Decimal,
            },
            SifrType::Decimal,
        ),
        (
            DatabaseType::Decimal {
                precision: Some(100),
                scale: Some(20),
                representation: DecimalRepresentation::BigDecimal,
            },
            SifrType::BigDecimal,
        ),
        (
            DatabaseType::Decimal {
                precision: None,
                scale: None,
                representation: DecimalRepresentation::Numeric,
            },
            SifrType::Numeric,
        ),
        (DatabaseType::Float32, SifrType::Float),
        (DatabaseType::Float64, SifrType::Float),
        (
            DatabaseType::Text {
                fixed: false,
                max_characters: None,
            },
            SifrType::Str,
        ),
        (DatabaseType::Binary { max_bytes: None }, SifrType::Bytes),
        (DatabaseType::Date, SifrType::Date),
        (
            DatabaseType::LocalTime { precision: 9 },
            SifrType::LocalTime,
        ),
        (
            DatabaseType::OffsetTime { precision: 9 },
            SifrType::OffsetTime,
        ),
        (
            DatabaseType::LocalDateTime { precision: 9 },
            SifrType::LocalDateTime,
        ),
        (DatabaseType::Instant { precision: 9 }, SifrType::Instant),
        (DatabaseType::CalendarInterval, SifrType::CalendarInterval),
        (DatabaseType::Uuid, SifrType::Uuid),
        (DatabaseType::Json { binary: true }, SifrType::JsonValue),
        (
            DatabaseType::Array {
                element: Box::new(signed(IntegerWidth::Bits32)),
                dimensions: Some(2),
                element_nullability: Nullability::Nullable,
                preserves_lower_bounds: true,
            },
            SifrType::SqlArray {
                element: Box::new(SifrType::Union {
                    members: BTreeSet::from([
                        SifrType::FixedInteger {
                            sign: IntegerSign::Signed,
                            width: IntegerWidth::Bits32,
                        },
                        SifrType::None,
                    ]),
                }),
            },
        ),
        (
            DatabaseType::Enum {
                identity: status.clone(),
            },
            SifrType::Nominal {
                identity: status.clone(),
            },
        ),
        (
            DatabaseType::Domain {
                identity: ObjectId::new("public.email"),
                base: Box::new(DatabaseType::Text {
                    fixed: false,
                    max_characters: None,
                }),
            },
            SifrType::Nominal {
                identity: ObjectId::new("public.email"),
            },
        ),
        (
            DatabaseType::Composite {
                identity: ObjectId::new("public.address"),
            },
            SifrType::Nominal {
                identity: ObjectId::new("public.address"),
            },
        ),
        (
            DatabaseType::Range {
                element: Box::new(DatabaseType::Date),
                multirange: false,
            },
            SifrType::Range {
                element: Box::new(SifrType::Date),
                multirange: false,
            },
        ),
        (
            DatabaseType::Range {
                element: Box::new(DatabaseType::Date),
                multirange: true,
            },
            SifrType::Range {
                element: Box::new(SifrType::Date),
                multirange: true,
            },
        ),
        (DatabaseType::IpAddress, SifrType::IpAddress),
        (DatabaseType::IpNetwork, SifrType::IpNetwork),
        (DatabaseType::MacAddress, SifrType::MacAddress),
        (
            DatabaseType::Custom {
                identity: ObjectId::new("public.money"),
                codec: custom_codec.clone(),
            },
            SifrType::Custom {
                identity: custom_codec.as_str().to_string(),
            },
        ),
    ];

    for (database, expected) in cases {
        assert_eq!(canonical_read_type(&database), Ok(expected));
    }
}

#[test]
fn sqlite_dynamic_storage_preserves_the_complete_union() {
    let database = DatabaseType::SqliteDynamic {
        storage_classes: BTreeSet::from([
            SqliteStorageClass::Integer,
            SqliteStorageClass::Real,
            SqliteStorageClass::Text,
            SqliteStorageClass::Blob,
            SqliteStorageClass::Null,
        ]),
    };
    let SifrType::Union { members } =
        canonical_read_type(&database).expect("complete SQLite storage set should map")
    else {
        panic!("SQLite dynamic mapping must remain a union");
    };
    assert_eq!(members.len(), 5);
    assert!(members.contains(&SifrType::None));
}

#[test]
fn bind_matrix_is_closed_and_preserves_width_nullability_and_shape() {
    let codecs = CodecRegistry::default();
    assert_eq!(
        bind_compatibility(
            &input(SifrType::FixedInteger {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits32,
            }),
            &target(signed(IntegerWidth::Bits32)),
            &codecs,
        ),
        BindCompatibility::Exact,
    );
    assert_eq!(
        bind_compatibility(
            &input(SifrType::ExactInteger),
            &target(signed(IntegerWidth::Bits16)),
            &codecs,
        ),
        BindCompatibility::Fallible(EncodeCheck::ExactIntegerRange),
    );
    assert_eq!(
        bind_compatibility(
            &input(SifrType::FixedInteger {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits64,
            }),
            &target(signed(IntegerWidth::Bits32)),
            &codecs,
        ),
        BindCompatibility::Rejected(BindRejection::IntegerWidth),
    );
    assert_eq!(
        bind_compatibility(
            &InputType {
                value: SifrType::Str,
                nullability: Nullability::Nullable,
            },
            &target(DatabaseType::Text {
                fixed: false,
                max_characters: None,
            }),
            &codecs,
        ),
        BindCompatibility::Rejected(BindRejection::Nullability),
    );
    assert_eq!(
        bind_compatibility(
            &input(SifrType::Float),
            &target(DatabaseType::Float32),
            &codecs,
        ),
        BindCompatibility::Fallible(EncodeCheck::Float32RangeAndPrecision),
    );
    assert_eq!(
        bind_compatibility(
            &input(SifrType::Str),
            &target(DatabaseType::Text {
                fixed: true,
                max_characters: Some(10),
            }),
            &codecs,
        ),
        BindCompatibility::Fallible(EncodeCheck::TextLength),
    );
    assert_eq!(
        bind_compatibility(
            &input(SifrType::List {
                element: Box::new(SifrType::FixedInteger {
                    sign: IntegerSign::Signed,
                    width: IntegerWidth::Bits32,
                }),
            }),
            &target(DatabaseType::Array {
                element: Box::new(signed(IntegerWidth::Bits32)),
                dimensions: Some(1),
                element_nullability: Nullability::NonNull,
                preserves_lower_bounds: true,
            }),
            &codecs,
        ),
        BindCompatibility::Exact,
    );
}

#[test]
fn common_diagnostic_mapping_is_closed_and_stable() {
    assert_eq!(CommonSqlDiagnostic::DatabaseType.code(), "SIFR-SQL-0001");
    assert_eq!(
        CommonSqlDiagnostic::for_bind_rejection(BindRejection::Nullability).code(),
        "SIFR-SQL-0003",
    );
    assert_eq!(
        CommonSqlDiagnostic::for_bind_rejection(BindRejection::MissingCodec).code(),
        "SIFR-SQL-0004",
    );
    assert_eq!(CommonSqlDiagnostic::Ownership.code(), "SIFR-SQL-0008");
}

#[test]
fn codec_registry_requires_exact_database_and_sifr_identities() {
    let codec_id = codec_identity("app.money.v1");
    let database = DatabaseType::Custom {
        identity: ObjectId::new("public.money"),
        codec: codec_id.clone(),
    };
    let contract = CodecContract {
        identity: codec_id.clone(),
        database_type: database.clone(),
        sifr_type: SifrType::Custom {
            identity: "app.Money".to_string(),
        },
        server_profiles: BTreeSet::from(["postgresql-18".to_string()]),
        encode_error: "MoneyEncodeError".to_string(),
        decode_error: "MoneyDecodeError".to_string(),
        null_behavior: NullCodecBehavior::PassThrough,
        wire_format: wire_identity("postgresql.binary.money.v1"),
        panic_containment: PanicContainment::CatchAndRedact,
    };
    let registry =
        CodecRegistry::from_contracts([contract.clone()]).expect("one exact codec should register");
    assert_eq!(registry.codec(&codec_id), Some(&contract));
    assert_eq!(
        bind_compatibility(
            &input(SifrType::Custom {
                identity: "app.Money".to_string(),
            }),
            &target(database),
            &registry,
        ),
        BindCompatibility::Exact,
    );
}

#[test]
fn cardinality_is_a_complete_interval_lattice_and_does_not_select_containers() {
    let values = [
        Cardinality::BOTTOM,
        Cardinality::ZERO,
        Cardinality::AT_MOST_ONE,
        Cardinality::EXACTLY_ONE,
        Cardinality::ONE_OR_MORE,
        Cardinality::MANY,
        Cardinality::new(2, Some(8)).expect("valid interval"),
    ];
    for left in values {
        assert_eq!(left.join(Cardinality::BOTTOM), left);
        assert_eq!(left.meet(Cardinality::MANY), left);
        for right in values {
            assert_eq!(left.join(right), right.join(left));
            assert_eq!(left.meet(right), right.meet(left));
        }
    }
    assert!(Cardinality::AT_MOST_ONE.supports(FetchMethod::FetchOne, true));
    assert!(!Cardinality::MANY.supports(FetchMethod::FetchOne, true));
    assert!(Cardinality::MANY.supports(FetchMethod::FetchAll, true));
    assert!(Cardinality::MANY.supports(FetchMethod::Stream, true));
    assert!(Cardinality::ZERO.supports(FetchMethod::Execute, false));
}

#[test]
fn effects_keep_referenced_and_affected_objects_distinct() {
    let users = ObjectId::new("public.users");
    let audit = ObjectId::new("public.audit");
    let read = EffectContract::new(
        QueryEffect::Read,
        BTreeSet::from([users.clone()]),
        BTreeSet::new(),
    )
    .expect("read effect should validate");
    assert!(read.application_safe());
    let write = EffectContract::new(
        QueryEffect::ReadWrite,
        BTreeSet::from([users]),
        BTreeSet::from([audit.clone()]),
    )
    .expect("read-write effect should validate");
    assert_eq!(write.affected_objects, BTreeSet::from([audit]));
    assert!(
        EffectContract::new(
            QueryEffect::Read,
            BTreeSet::new(),
            BTreeSet::from([ObjectId::new("public.users")]),
        )
        .is_err()
    );
}

#[test]
fn provider_analysis_exposes_only_validated_common_semantics() {
    let codec_id = codec_identity("postgresql.int32.binary.v1");
    let database = signed(IntegerWidth::Bits32);
    let sifr_type = SifrType::FixedInteger {
        sign: IntegerSign::Signed,
        width: IntegerWidth::Bits32,
    };
    let registry = CodecRegistry::from_contracts([CodecContract {
        identity: codec_id.clone(),
        database_type: database.clone(),
        sifr_type: sifr_type.clone(),
        server_profiles: BTreeSet::from(["postgresql-18".to_string()]),
        encode_error: "IntegerEncodeError".to_string(),
        decode_error: "IntegerDecodeError".to_string(),
        null_behavior: NullCodecBehavior::Reject,
        wire_format: wire_identity("postgresql.binary.int32.v1"),
        panic_containment: PanicContainment::CatchAndRedact,
    }])
    .expect("built-in codec should register");
    let analysis = ProviderAnalysis {
        normalized_statement: "select $1::integer as value".to_string(),
        parameters: vec![ProviderParameter {
            slot: 0,
            database_type: database.clone(),
            nullability: Nullability::NonNull,
            codec: codec_id.clone(),
        }],
        result_fields: vec![ProviderResultField {
            name: "value".to_string(),
            sifr_type,
            database_type: database,
            nullability: Nullability::NonNull,
            codec: codec_id,
            source_object: None,
        }],
        cardinality: Cardinality::EXACTLY_ONE,
        effects: EffectContract::new(QueryEffect::Read, BTreeSet::new(), BTreeSet::new())
            .expect("read effect should validate"),
        semantic_flags: BTreeSet::from(["stable-result-name".to_string()]),
    };
    assert!(analysis.validate(&registry).is_ok());

    let mut invalid = analysis;
    invalid.parameters[0].slot = 7;
    assert!(invalid.validate(&registry).is_err());
}
