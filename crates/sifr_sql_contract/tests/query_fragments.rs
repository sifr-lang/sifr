#![allow(clippy::expect_used, clippy::unwrap_used)]

use semver::Version;
use sifr_sql_contract::{
    Cardinality, CodecContract, CodecIdentity, CodecRegistry, DialectIdentity, EffectContract,
    EffectTransformation, FragmentCategory, FragmentDraft, NullCodecBehavior, Nullability,
    ObjectId, PackageCapabilityResolver, PanicContainment, PoolingMode, PredicateContext,
    ProfileModuleRegistry, ProviderAnalysis, ProviderIdentity, ProviderParameter,
    ProviderResultField, QueryContractErrorKind, QueryDefinitionScope, QueryEffect, QueryOrigin,
    QuerySignatureRegistry, QuerySymbol, QuerySymbolKind, QueryTemplateContract,
    QueryTemplateDraft, ResultTransformation, SchemaDocument, SchemaDocumentKind, SchemaEvidence,
    SchemaProfile, SchemaStrictness, SessionContract, SifrType, SqlFragment, SqlPrecedence,
    StaticFragmentOrigin, UnsafeSyntaxGrant, UnsafeSyntaxLint, WireFormatIdentity, all_predicates,
    any_predicates, build_profile_authority, decode_generated_identifier, decode_generated_path,
    encode_generated_identifier, encode_generated_path, generate_profile_module, normalize_schema,
    not_predicate,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn generated_identifiers_are_injective_reversible_and_readable_when_safe() {
    let values = [
        "users",
        "if",
        "if_",
        "a__b",
        "_sifr_sql_6966",
        "München",
        "_private",
    ];
    let encoded = values
        .iter()
        .map(|value| encode_generated_identifier(value).expect("valid identifier"))
        .collect::<BTreeSet<_>>();
    assert_eq!(encoded.len(), values.len());
    assert_eq!(encode_generated_identifier("users").unwrap(), "users");
    for value in values {
        let encoded = encode_generated_identifier(value).unwrap();
        assert_eq!(decode_generated_identifier(&encoded).unwrap(), value);
    }

    let paths = [
        vec!["a".to_string(), "b".to_string()],
        vec!["a__b".to_string()],
        vec!["if".to_string(), "value".to_string()],
        vec!["x_".to_string(), "y".to_string()],
        vec!["x".to_string(), "_y".to_string()],
    ];
    let encoded_paths = paths
        .iter()
        .map(|path| encode_generated_path(path).unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(encoded_paths.len(), paths.len());
    for path in paths {
        let encoded = encode_generated_path(&path).unwrap();
        assert_eq!(decode_generated_path(&encoded).unwrap(), path);
    }
}

#[test]
fn profile_registry_is_canonical_and_queryable_by_every_identity() {
    let profile_authority = authority("app");
    let module = generate_profile_module(&profile_authority).expect("generated module");
    let module_path = module.module_path.clone();
    let nominal = profile_authority.nominal_identity.clone();
    let mut registry = ProfileModuleRegistry::default();
    registry
        .register(profile_authority, module)
        .expect("registry entry should match authority");

    assert_eq!(registry.len(), 1);
    assert_eq!(
        registry.profile("app").unwrap().module().module_path,
        module_path
    );
    assert_eq!(
        registry
            .module_path(&module_path)
            .unwrap()
            .authority()
            .nominal_identity,
        nominal
    );
    assert_eq!(
        registry
            .nominal_identity(&nominal)
            .unwrap()
            .authority()
            .profile
            .name,
        "app"
    );
    assert!(registry.cache_fragment().contains("sifr.sql.schemas.app"));

    let duplicate = authority("app");
    let module = generate_profile_module(&duplicate).unwrap();
    assert!(registry.register(duplicate, module).is_err());
}

#[test]
fn reusable_query_and_row_of_keep_one_static_identity_and_structural_row() {
    let (registry, codecs) = registry_and_codecs();
    let query = compile_query(&registry, &codecs, Cardinality::AT_MOST_ONE);
    let same = compile_query(&registry, &codecs, Cardinality::AT_MOST_ONE);
    assert_eq!(query.identity, same.identity);
    assert!(query.returns_rows());

    let mut signatures = QuerySignatureRegistry::default();
    signatures
        .register(QuerySymbol {
            module: "queries.users".to_string(),
            name: "find_user".to_string(),
            kind: QuerySymbolKind::TopLevelReusable,
            exported: true,
            template: query.clone(),
        })
        .expect("top-level query should register");
    let row = signatures
        .row_of("queries.users", "find_user", true)
        .expect("exported RowOf should resolve");
    assert_eq!(row.template_identity, query.identity);
    assert_eq!(row.fields, vec![("active".to_string(), SifrType::Bool)]);

    let local_error = signatures
        .register(QuerySymbol {
            module: "queries.users".to_string(),
            name: "local".to_string(),
            kind: QuerySymbolKind::LocalFunction,
            exported: false,
            template: query,
        })
        .expect_err("local query must not become a RowOf symbol");
    assert_eq!(local_error.kind, QueryContractErrorKind::InvalidRowOf);
}

#[test]
fn cardinality_adapters_are_explicit_and_first_reports_unordered_queries() {
    let (registry, codecs) = registry_and_codecs();
    let many = compile_query(&registry, &codecs, Cardinality::MANY);
    let narrowed = many.clone().expect_at_most_one().expect("valid adapter");
    assert_eq!(narrowed.cardinality, Cardinality::AT_MOST_ONE);
    let (first, warning) = many
        .first("SELECT active FROM users LIMIT 1")
        .expect("provider supplied limited plan");
    assert_eq!(first.cardinality, Cardinality::AT_MOST_ONE);
    assert!(warning.is_some());
}

#[test]
fn fragments_enforce_category_profile_scope_alias_hygiene_and_precedence() {
    let (registry, codecs) = registry_and_codecs();
    let query = compile_query(&registry, &codecs, Cardinality::MANY);
    let mut scope = QueryDefinitionScope::new(query.identity.as_str()).unwrap();
    let alias = scope
        .relation_alias(
            ObjectId::new("public.users"),
            "u",
            StaticFragmentOrigin::QueryDefinition,
        )
        .unwrap();
    assert!(
        scope
            .relation_alias(
                ObjectId::new("public.users"),
                "dynamic",
                StaticFragmentOrigin::RuntimeBranch,
            )
            .is_err()
    );

    let context = PredicateContext {
        query_identity: query.identity.clone(),
        profile_identity: query.profile_identity.clone(),
        dialect: query.dialect.clone(),
        scope: BTreeSet::from([alias.identity.clone()]),
    };
    let first = predicate(&context, "u.active = TRUE", SqlPrecedence::COMPARISON);
    let second = predicate(&context, "u.deleted = FALSE", SqlPrecedence::COMPARISON);
    let both = all_predicates(&context, vec![first.clone(), second]).unwrap();
    assert_eq!(
        both.canonical_syntax,
        "u.active = TRUE AND u.deleted = FALSE"
    );
    assert_eq!(
        all_predicates(&context, Vec::new())
            .unwrap()
            .canonical_syntax,
        "TRUE"
    );
    assert_eq!(
        any_predicates(&context, Vec::new())
            .unwrap()
            .canonical_syntax,
        "FALSE"
    );
    assert_eq!(
        not_predicate(&context, first).unwrap().canonical_syntax,
        "NOT u.active = TRUE"
    );
    assert!(
        both.validate_insertion(
            FragmentCategory::OrderBy,
            &context.query_identity,
            &context.profile_identity,
            &context.dialect,
            &context.scope,
        )
        .is_err()
    );
}

#[test]
fn ordinary_runtime_text_has_no_fragment_path_and_unsafe_syntax_is_audited() {
    let (registry, codecs) = registry_and_codecs();
    let query = compile_query(&registry, &codecs, Cardinality::MANY);
    let context = PredicateContext {
        query_identity: query.identity,
        profile_identity: query.profile_identity,
        dialect: query.dialect,
        scope: BTreeSet::new(),
    };
    let denied = TestCapabilities::default();
    assert!(
        UnsafeSyntaxGrant::from_package_resolver(
            &denied,
            "app@1.0.0",
            UnsafeSyntaxLint::Warn,
            "operator-selected maintenance statement",
        )
        .is_err()
    );
    let allowed = TestCapabilities {
        unsafe_syntax: true,
    };
    assert!(
        UnsafeSyntaxGrant::from_package_resolver(
            &allowed,
            "app@1.0.0",
            UnsafeSyntaxLint::Deny,
            "operator-selected maintenance statement",
        )
        .is_err()
    );
    let grant = UnsafeSyntaxGrant::from_package_resolver(
        &allowed,
        "app@1.0.0",
        UnsafeSyntaxLint::Warn,
        "operator-selected maintenance statement",
    )
    .unwrap();
    let fragment = SqlFragment::unsafe_checked(
        fragment_draft(&context, "VACUUM", FragmentCategory::Command),
        &grant,
    )
    .unwrap();
    assert_eq!(
        fragment.unsafe_audit.unwrap().capability,
        "sql.unsafe-syntax"
    );
}

#[derive(Default)]
struct TestCapabilities {
    unsafe_syntax: bool,
}

impl PackageCapabilityResolver for TestCapabilities {
    fn allows(&self, package_identity: &str, capability: &str) -> bool {
        self.unsafe_syntax && package_identity == "app@1.0.0" && capability == "sql.unsafe-syntax"
    }
}

fn registry_and_codecs() -> (ProfileModuleRegistry, CodecRegistry) {
    let authority = authority("app");
    let module = generate_profile_module(&authority).unwrap();
    let mut registry = ProfileModuleRegistry::default();
    registry.register(authority, module).unwrap();
    let identity = CodecIdentity::new("postgresql.bool.v1").unwrap();
    let codecs = CodecRegistry::for_profile(
        "postgresql-18",
        [CodecContract {
            identity,
            database_type: sifr_sql_contract::DatabaseType::Boolean,
            sifr_type: SifrType::Bool,
            server_profiles: BTreeSet::from(["postgresql-18".to_string()]),
            encode_error: "sifr.sql.EncodeError".to_string(),
            decode_error: "sifr.sql.DecodeError".to_string(),
            null_behavior: NullCodecBehavior::PassThrough,
            wire_format: WireFormatIdentity::new("postgresql.binary.bool.v1").unwrap(),
            panic_containment: PanicContainment::CatchAndRedact,
        }],
    )
    .unwrap();
    (registry, codecs)
}

fn compile_query(
    registry: &ProfileModuleRegistry,
    codecs: &CodecRegistry,
    cardinality: Cardinality,
) -> QueryTemplateContract {
    let codec = CodecIdentity::new("postgresql.bool.v1").unwrap();
    QueryTemplateContract::compile(
        registry,
        "app",
        QueryTemplateDraft {
            origin: QueryOrigin::new("queries.users", "find_user", 10, 60).unwrap(),
            analysis: ProviderAnalysis {
                server_profile: "postgresql-18".to_string(),
                normalized_statement: "SELECT active FROM users WHERE active = $1".to_string(),
                parameters: vec![ProviderParameter {
                    slot: 0,
                    database_type: sifr_sql_contract::DatabaseType::Boolean,
                    nullability: Nullability::NonNull,
                    codec: codec.clone(),
                }],
                result_fields: vec![ProviderResultField {
                    name: "active".to_string(),
                    sifr_type: SifrType::Bool,
                    database_type: sifr_sql_contract::DatabaseType::Boolean,
                    nullability: Nullability::NonNull,
                    codec,
                    source_object: Some(ObjectId::new("public.users.active")),
                }],
                cardinality,
                effects: EffectContract::new(
                    QueryEffect::Read,
                    BTreeSet::from([ObjectId::new("public.users")]),
                    BTreeSet::new(),
                )
                .unwrap(),
                accessed_objects: BTreeSet::from([
                    ObjectId::new("public.users"),
                    ObjectId::new("public.users.active"),
                ]),
                semantic_flags: BTreeSet::new(),
                required_capabilities: BTreeSet::from([
                    "sql.bind.parameters".to_string(),
                    "sql.expression.equality".to_string(),
                    "sql.query.select".to_string(),
                ]),
            },
            parameter_types: vec![SifrType::Bool],
            deterministic_order: false,
            fragment_identities: Vec::new(),
        },
        codecs,
    )
    .unwrap()
}

fn predicate(context: &PredicateContext, syntax: &str, precedence: SqlPrecedence) -> SqlFragment {
    SqlFragment::checked(FragmentDraft {
        precedence,
        ..fragment_draft(context, syntax, FragmentCategory::Predicate)
    })
    .unwrap()
}

fn fragment_draft(
    context: &PredicateContext,
    syntax: &str,
    category: FragmentCategory,
) -> FragmentDraft {
    FragmentDraft {
        query_identity: context.query_identity.clone(),
        profile_identity: context.profile_identity.clone(),
        dialect: context.dialect.clone(),
        category,
        input_scope: context.scope.clone(),
        output_scope: context.scope.clone(),
        required_aliases: context.scope.clone(),
        introduced_aliases: BTreeSet::new(),
        free_identifiers: BTreeSet::new(),
        parameters: Vec::new(),
        result: ResultTransformation::Preserve,
        effect: EffectTransformation::Preserve,
        precedence: SqlPrecedence::ATOM,
        canonical_syntax: syntax.to_string(),
        origin: StaticFragmentOrigin::QueryDefinition,
    }
}

fn authority(name: &str) -> sifr_sql_contract::ProfileAuthority {
    let schema = normalize_schema(
        ProviderIdentity {
            package_id: "sifr-sql-postgresql@1.0.0#registry".to_string(),
            package_version: Version::new(1, 0, 0),
            package_source: "registry+https://github.com/rust-lang/crates.io-index".to_string(),
            package_graph_digest: "sha256:locked-graph".to_string(),
            compiler_components: BTreeMap::from([("postgresql@1.0.0".to_string(), "a".repeat(64))]),
        },
        DialectIdentity {
            family: "postgresql".to_string(),
            server_version: "18".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::new(),
        },
        [SchemaDocument {
            kind: SchemaDocumentKind::SqlDdl,
            document: "db/schema.sql".to_string(),
            objects: Vec::new(),
        }],
    )
    .unwrap();
    build_profile_authority(SchemaProfile {
        package_id: "app@1.0.0#registry".to_string(),
        name: name.to_string(),
        source_files: BTreeSet::from(["db/schema.sql".to_string()]),
        source_fingerprints: BTreeMap::from([("db/schema.sql".to_string(), "b".repeat(64))]),
        evidence: SchemaEvidence::MigrationHead,
        strictness: SchemaStrictness::Compatible,
        pooling: PoolingMode::Session,
        session: SessionContract::default(),
        accepted_signers: BTreeSet::new(),
        capabilities: BTreeSet::from(["sql.query.select".to_string()]),
        schema,
    })
    .unwrap()
}
