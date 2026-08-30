#![allow(clippy::expect_used)]

use sifr_sql_contract::{
    BindCompatibility, BindRejection, Cardinality, CodecContract, CodecIdentity, CodecRegistry,
    CommonSqlDiagnostic, DatabaseType, DecimalRepresentation, EffectContract, EncodeCheck,
    FetchMethod, InputType, IntegerSign, IntegerWidth, NullCodecBehavior, Nullability, ObjectId,
    PanicContainment, ParameterType, ProviderAnalysis, ProviderParameter, ProviderResultField,
    QueryEffect, SifrType, SqliteStorageClass, WireFormatIdentity, bind_compatibility,
    canonical_read_type, canonical_read_type_in,
};
use std::collections::BTreeSet;

const QUALIFICATION: &str =
    include_str!("../../../verification/areas/sql_platform/data/common_sql_qualification.json");

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

fn qualification_values(field: &str) -> BTreeSet<String> {
    let payload: serde_json::Value =
        serde_json::from_str(QUALIFICATION).expect("qualification JSON should parse");
    payload[field]
        .as_array()
        .expect("qualification field should be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("qualification value should be text")
                .to_string()
        })
        .collect()
}

fn database_family(database: &DatabaseType) -> &'static str {
    match database {
        DatabaseType::Boolean => "boolean",
        DatabaseType::Integer { .. } => "integer",
        DatabaseType::Decimal { .. } => "decimal",
        DatabaseType::Float32 => "float32",
        DatabaseType::Float64 => "float64",
        DatabaseType::Text { .. } => "text",
        DatabaseType::Binary { .. } => "binary",
        DatabaseType::Date => "date",
        DatabaseType::LocalTime { .. } => "local-time",
        DatabaseType::OffsetTime { .. } => "offset-time",
        DatabaseType::LocalDateTime { .. } => "local-date-time",
        DatabaseType::Instant { .. } => "timestamp-instant",
        DatabaseType::CalendarInterval => "calendar-interval",
        DatabaseType::Uuid => "uuid",
        DatabaseType::Json { .. } => "json",
        DatabaseType::Array { .. } => "array",
        DatabaseType::Enum { .. } => "enum",
        DatabaseType::Domain { .. } => "domain",
        DatabaseType::Composite { .. } => "composite",
        DatabaseType::Range { .. } => "range-and-multirange",
        DatabaseType::IpAddress => "ip-address",
        DatabaseType::IpNetwork => "ip-network",
        DatabaseType::MacAddress => "mac-address",
        DatabaseType::Custom { .. } => "custom",
        DatabaseType::SqliteDynamic { .. } => "sqlite-dynamic",
    }
}

fn bind_result_name(result: BindCompatibility) -> &'static str {
    match result {
        BindCompatibility::Exact => "exact",
        BindCompatibility::Fallible(EncodeCheck::ArrayShape) => "fallible-array-shape",
        BindCompatibility::Fallible(EncodeCheck::BinaryLength) => "fallible-binary-length",
        BindCompatibility::Fallible(EncodeCheck::DecimalPrecisionAndScale) => {
            "fallible-decimal-precision-and-scale"
        }
        BindCompatibility::Fallible(EncodeCheck::ExactIntegerRange) => {
            "fallible-exact-integer-range"
        }
        BindCompatibility::Fallible(EncodeCheck::Float32RangeAndPrecision) => {
            "fallible-float32-range-and-precision"
        }
        BindCompatibility::Fallible(EncodeCheck::TextLength) => "fallible-text-length",
        BindCompatibility::Rejected(BindRejection::ArrayElement) => "reject-array-element",
        BindCompatibility::Rejected(BindRejection::IntegerSign) => "reject-integer-sign",
        BindCompatibility::Rejected(BindRejection::IntegerWidth) => "reject-integer-width",
        BindCompatibility::Rejected(BindRejection::MissingCodec) => "reject-missing-codec",
        BindCompatibility::Rejected(BindRejection::NominalIdentity) => "reject-nominal-identity",
        BindCompatibility::Rejected(BindRejection::Nullability) => "reject-nullability",
        BindCompatibility::Rejected(BindRejection::UnsupportedPair) => "reject-unsupported-pair",
    }
}

#[test]
fn canonical_read_matrix_covers_every_locked_semantic_family() {
    let status = ObjectId::new("public.status");
    let custom_codec = codec_identity("app.money.v1");
    let custom_database = DatabaseType::Custom {
        identity: ObjectId::new("public.money"),
        codec: custom_codec.clone(),
    };
    let registry = CodecRegistry::for_profile(
        "postgresql-18",
        [CodecContract {
            identity: custom_codec.clone(),
            database_type: custom_database.clone(),
            sifr_type: SifrType::Custom {
                identity: "app.Money".to_string(),
            },
            server_profiles: BTreeSet::from(["postgresql-18".to_string()]),
            encode_error: "MoneyEncodeError".to_string(),
            decode_error: "MoneyDecodeError".to_string(),
            null_behavior: NullCodecBehavior::PassThrough,
            wire_format: wire_identity("postgresql.binary.money.v1"),
            panic_containment: PanicContainment::CatchAndRedact,
        }],
    )
    .expect("custom codec should register for its profile");
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
            DatabaseType::Array {
                element: Box::new(custom_database.clone()),
                dimensions: Some(1),
                element_nullability: Nullability::NonNull,
                preserves_lower_bounds: true,
            },
            SifrType::SqlArray {
                element: Box::new(SifrType::Custom {
                    identity: "app.Money".to_string(),
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
            custom_database.clone(),
            SifrType::Custom {
                identity: "app.Money".to_string(),
            },
        ),
        (
            DatabaseType::SqliteDynamic {
                storage_classes: BTreeSet::from([SqliteStorageClass::Integer]),
            },
            SifrType::Union {
                members: BTreeSet::from([SifrType::FixedInteger {
                    sign: IntegerSign::Signed,
                    width: IntegerWidth::Bits64,
                }]),
            },
        ),
    ];

    let observed_families = cases
        .iter()
        .map(|(database, _)| database_family(database).to_string())
        .collect::<BTreeSet<_>>();
    for (database, expected) in cases {
        assert_eq!(canonical_read_type_in(&database, &registry), Ok(expected));
    }
    assert_eq!(
        observed_families,
        qualification_values("database_type_families")
    );
    assert!(canonical_read_type(&custom_database).is_err());
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
    let custom_codec = codec_identity("app.money.v1");
    let custom_database = DatabaseType::Custom {
        identity: ObjectId::new("public.money"),
        codec: custom_codec.clone(),
    };
    let codecs = CodecRegistry::for_profile(
        "postgresql-18",
        [CodecContract {
            identity: custom_codec,
            database_type: custom_database.clone(),
            sifr_type: SifrType::Custom {
                identity: "app.Money".to_string(),
            },
            server_profiles: BTreeSet::from(["postgresql-18".to_string()]),
            encode_error: "MoneyEncodeError".to_string(),
            decode_error: "MoneyDecodeError".to_string(),
            null_behavior: NullCodecBehavior::PassThrough,
            wire_format: wire_identity("postgresql.binary.money.v1"),
            panic_containment: PanicContainment::CatchAndRedact,
        }],
    )
    .expect("custom codec should register");
    let array = |dimensions| DatabaseType::Array {
        element: Box::new(signed(IntegerWidth::Bits32)),
        dimensions,
        element_nullability: Nullability::NonNull,
        preserves_lower_bounds: true,
    };
    let cases = vec![
        (
            input(SifrType::FixedInteger {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits32,
            }),
            target(signed(IntegerWidth::Bits32)),
            BindCompatibility::Exact,
        ),
        (
            input(SifrType::Custom {
                identity: "app.Money".to_string(),
            }),
            target(custom_database),
            BindCompatibility::Exact,
        ),
        (
            input(SifrType::FixedInteger {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits64,
            }),
            target(DatabaseType::SqliteDynamic {
                storage_classes: BTreeSet::from([SqliteStorageClass::Integer]),
            }),
            BindCompatibility::Exact,
        ),
        (
            input(SifrType::ExactInteger),
            target(signed(IntegerWidth::Bits16)),
            BindCompatibility::Fallible(EncodeCheck::ExactIntegerRange),
        ),
        (
            input(SifrType::Float),
            target(DatabaseType::Float32),
            BindCompatibility::Fallible(EncodeCheck::Float32RangeAndPrecision),
        ),
        (
            input(SifrType::Decimal),
            target(DatabaseType::Decimal {
                precision: Some(5),
                scale: Some(2),
                representation: DecimalRepresentation::Decimal,
            }),
            BindCompatibility::Fallible(EncodeCheck::DecimalPrecisionAndScale),
        ),
        (
            input(SifrType::Numeric),
            target(DatabaseType::Decimal {
                precision: Some(5),
                scale: Some(2),
                representation: DecimalRepresentation::Decimal,
            }),
            BindCompatibility::Rejected(BindRejection::UnsupportedPair),
        ),
        (
            input(SifrType::Str),
            target(DatabaseType::Text {
                fixed: true,
                max_characters: Some(10),
            }),
            BindCompatibility::Fallible(EncodeCheck::TextLength),
        ),
        (
            input(SifrType::Bytes),
            target(DatabaseType::Binary { max_bytes: Some(8) }),
            BindCompatibility::Fallible(EncodeCheck::BinaryLength),
        ),
        (
            input(SifrType::SqlArray {
                element: Box::new(SifrType::FixedInteger {
                    sign: IntegerSign::Signed,
                    width: IntegerWidth::Bits32,
                }),
            }),
            target(array(Some(2))),
            BindCompatibility::Fallible(EncodeCheck::ArrayShape),
        ),
        (
            InputType {
                value: SifrType::Str,
                nullability: Nullability::Nullable,
            },
            target(DatabaseType::Text {
                fixed: false,
                max_characters: None,
            }),
            BindCompatibility::Rejected(BindRejection::Nullability),
        ),
        (
            InputType {
                value: SifrType::Union {
                    members: BTreeSet::from([SifrType::Str, SifrType::None]),
                },
                nullability: Nullability::NonNull,
            },
            ParameterType {
                database: DatabaseType::Text {
                    fixed: false,
                    max_characters: None,
                },
                nullability: Nullability::Nullable,
            },
            BindCompatibility::Exact,
        ),
        (
            InputType {
                value: SifrType::None,
                nullability: Nullability::NonNull,
            },
            ParameterType {
                database: DatabaseType::Text {
                    fixed: false,
                    max_characters: None,
                },
                nullability: Nullability::Nullable,
            },
            BindCompatibility::Exact,
        ),
        (
            InputType {
                value: SifrType::Union {
                    members: BTreeSet::from([SifrType::Str, SifrType::None]),
                },
                nullability: Nullability::NonNull,
            },
            target(DatabaseType::SqliteDynamic {
                storage_classes: BTreeSet::from([
                    SqliteStorageClass::Text,
                    SqliteStorageClass::Null,
                ]),
            }),
            BindCompatibility::Rejected(BindRejection::Nullability),
        ),
        (
            input(SifrType::FixedInteger {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits64,
            }),
            target(signed(IntegerWidth::Bits32)),
            BindCompatibility::Rejected(BindRejection::IntegerWidth),
        ),
        (
            input(SifrType::FixedInteger {
                sign: IntegerSign::Signed,
                width: IntegerWidth::Bits32,
            }),
            target(DatabaseType::Integer {
                sign: IntegerSign::Unsigned,
                width: IntegerWidth::Bits32,
            }),
            BindCompatibility::Rejected(BindRejection::IntegerSign),
        ),
        (
            input(SifrType::List {
                element: Box::new(SifrType::Str),
            }),
            target(array(Some(1))),
            BindCompatibility::Rejected(BindRejection::ArrayElement),
        ),
        (
            input(SifrType::Nominal {
                identity: ObjectId::new("public.left"),
            }),
            target(DatabaseType::Enum {
                identity: ObjectId::new("public.right"),
            }),
            BindCompatibility::Rejected(BindRejection::NominalIdentity),
        ),
        (
            input(SifrType::Custom {
                identity: "app.Missing".to_string(),
            }),
            target(DatabaseType::Custom {
                identity: ObjectId::new("public.missing"),
                codec: codec_identity("app.missing.v1"),
            }),
            BindCompatibility::Rejected(BindRejection::MissingCodec),
        ),
        (
            input(SifrType::Bool),
            target(DatabaseType::Text {
                fixed: false,
                max_characters: None,
            }),
            BindCompatibility::Rejected(BindRejection::UnsupportedPair),
        ),
    ];
    let mut observed = BTreeSet::new();
    for (input, target, expected) in cases {
        let actual = bind_compatibility(&input, &target, &codecs);
        assert_eq!(actual, expected);
        observed.insert(bind_result_name(actual).to_string());
    }
    assert_eq!(observed, qualification_values("bind_results"));
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
    let registry = CodecRegistry::for_profile("postgresql-18", [contract.clone()])
        .expect("one exact codec should register");
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

    let shared_database = signed(IntegerWidth::Bits32);
    let provider_contract = |identity: &str, profile: &str, wire: &str| CodecContract {
        identity: codec_identity(identity),
        database_type: shared_database.clone(),
        sifr_type: SifrType::FixedInteger {
            sign: IntegerSign::Signed,
            width: IntegerWidth::Bits32,
        },
        server_profiles: BTreeSet::from([profile.to_string()]),
        encode_error: "IntegerEncodeError".to_string(),
        decode_error: "IntegerDecodeError".to_string(),
        null_behavior: NullCodecBehavior::Reject,
        wire_format: wire_identity(wire),
        panic_containment: PanicContainment::CatchAndRedact,
    };
    let contracts = [
        provider_contract(
            "postgresql.int32.binary.v1",
            "postgresql-18",
            "postgresql.binary.int32.v1",
        ),
        provider_contract("mysql.int32.text.v1", "mysql-8.4", "mysql.text.int32.v1"),
    ];
    let postgresql = CodecRegistry::for_profile("postgresql-18", contracts.clone())
        .expect("PostgreSQL profile should select its wire codec");
    let mysql = CodecRegistry::for_profile("mysql-8.4", contracts)
        .expect("MySQL profile should select its wire codec");
    assert_eq!(postgresql.server_profile(), "postgresql-18");
    assert_eq!(
        postgresql
            .codec_for_database_type(&shared_database)
            .map(|codec| codec.identity.as_str()),
        Some("postgresql.int32.binary.v1"),
    );
    assert_eq!(
        mysql
            .codec_for_database_type(&shared_database)
            .map(|codec| codec.identity.as_str()),
        Some("mysql.int32.text.v1"),
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
    let registry = CodecRegistry::for_profile(
        "postgresql-18",
        [CodecContract {
            identity: codec_id.clone(),
            database_type: database.clone(),
            sifr_type: sifr_type.clone(),
            server_profiles: BTreeSet::from(["postgresql-18".to_string()]),
            encode_error: "IntegerEncodeError".to_string(),
            decode_error: "IntegerDecodeError".to_string(),
            null_behavior: NullCodecBehavior::Reject,
            wire_format: wire_identity("postgresql.binary.int32.v1"),
            panic_containment: PanicContainment::CatchAndRedact,
        }],
    )
    .expect("built-in codec should register");
    let analysis = ProviderAnalysis {
        server_profile: "postgresql-18".to_string(),
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

    let custom_codec = codec_identity("app.money.reject-null.v1");
    let custom_database = DatabaseType::Custom {
        identity: ObjectId::new("public.money"),
        codec: custom_codec.clone(),
    };
    let custom_registry = CodecRegistry::for_profile(
        "postgresql-18",
        [CodecContract {
            identity: custom_codec.clone(),
            database_type: custom_database.clone(),
            sifr_type: SifrType::Custom {
                identity: "app.Money".to_string(),
            },
            server_profiles: BTreeSet::from(["postgresql-18".to_string()]),
            encode_error: "MoneyEncodeError".to_string(),
            decode_error: "MoneyDecodeError".to_string(),
            null_behavior: NullCodecBehavior::Reject,
            wire_format: wire_identity("postgresql.binary.money.v1"),
            panic_containment: PanicContainment::CatchAndRedact,
        }],
    )
    .expect("custom codec should register");
    let nullable_custom = ProviderAnalysis {
        server_profile: "postgresql-18".to_string(),
        normalized_statement: "select money from ledger".to_string(),
        parameters: Vec::new(),
        result_fields: vec![ProviderResultField {
            name: "money".to_string(),
            sifr_type: SifrType::Union {
                members: BTreeSet::from([
                    SifrType::Custom {
                        identity: "app.Money".to_string(),
                    },
                    SifrType::None,
                ]),
            },
            database_type: custom_database,
            nullability: Nullability::Nullable,
            codec: custom_codec,
            source_object: None,
        }],
        cardinality: Cardinality::MANY,
        effects: EffectContract::new(QueryEffect::Read, BTreeSet::new(), BTreeSet::new())
            .expect("read effect should validate"),
        semantic_flags: BTreeSet::new(),
    };
    assert!(nullable_custom.validate(&custom_registry).is_err());
}
