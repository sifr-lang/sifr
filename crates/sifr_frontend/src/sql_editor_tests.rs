use super::*;
use crate::{FrontendDiagnosticStyle, FrontendSourceContext, compile_module_hir_with_source};
use semver::Version;
use sifr_lowering::ExternalDefs;
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, SCHEMA_IR_FORMAT_VERSION};

#[test]
fn documents_cover_sql_tokens_holes_semantics_and_fragment_scope() {
    let source = "def query(user_id: int) -> Template:\n    return t\"SELECT u.name FROM users AS u WHERE u.id = {user_id} LIMIT 1\"\n";
    let parsed = crate::parse_source_module(source, Some("editor.sifr")).expect("parse");
    let lowered = compile_module_hir_with_source(
        "editor",
        parsed.suite(),
        &ExternalDefs::default(),
        FrontendDiagnosticStyle::Bare,
        Some(FrontendSourceContext {
            display_path: "editor.sifr",
            source,
        }),
    )
    .expect("lower");
    let mut documents = sql_editor_documents(&lowered.module);
    assert_eq!(documents.len(), 1);
    let mut catalog = SqlEditorCatalog::default();
    catalog.symbols.insert(
        "users".to_string(),
        SqlEditorSymbol {
            name: "users".to_string(),
            kind: "relation".to_string(),
            database_type: None,
            sifr_type: None,
            nullable: None,
            definition_document: Some("schema.sql".to_string()),
            definition_range: Some(TextRange::new(TextSize::new(0), TextSize::new(5))),
        },
    );
    catalog.symbols.insert(
        "users.name".to_string(),
        SqlEditorSymbol {
            name: "users.name".to_string(),
            kind: "column".to_string(),
            database_type: Some("text".to_string()),
            sifr_type: Some("str".to_string()),
            nullable: Some(false),
            definition_document: Some("schema.sql".to_string()),
            definition_range: Some(TextRange::new(TextSize::new(6), TextSize::new(10))),
        },
    );
    catalog.symbols.insert(
        "orders.total".to_string(),
        SqlEditorSymbol {
            name: "orders.total".to_string(),
            kind: "column".to_string(),
            database_type: None,
            sifr_type: None,
            nullable: None,
            definition_document: None,
            definition_range: None,
        },
    );
    catalog.fragment_relations.insert(
        "users-only".to_string(),
        BTreeSet::from(["users".to_string()]),
    );
    documents[0] = documents[0].clone().with_semantics(
        catalog,
        Some("users-only".to_string()),
        vec!["int".to_string()],
        Vec::new(),
        "zero-or-one",
    );
    let document = &documents[0];
    assert_eq!(document.cardinality, "zero-or-one");
    assert!(document.tokens.iter().any(|token| token.text == "SELECT"));
    assert_eq!(document.parameter_source_ranges().len(), 1);
    assert!(
        document
            .completion_symbols(document.template.source_range.start())
            .iter()
            .any(|symbol| symbol.name == "users")
    );
    assert!(
        document
            .completion_symbols(document.template.source_range.start())
            .iter()
            .any(|symbol| symbol.name == "users.name")
    );
    assert!(
        !document
            .completion_symbols(document.template.source_range.start())
            .iter()
            .any(|symbol| symbol.name == "orders.total")
    );

    let name = document
        .tokens
        .iter()
        .find(|token| token.text == "name")
        .expect("name token");
    let name_source = document
        .template
        .source_range_for_virtual_range(name.virtual_range)
        .expect("name source");
    let fixes = document.fixes_for_diagnostic("SIFR-SQL-POSTGRESQL-0005", name_source);
    assert_eq!(fixes[0].kind, SqlEditorFixKind::Cast);
    assert_eq!(document.source_range_for_fix(&fixes[0]), Some(name_source));
}

#[test]
fn nominal_sql_holes_keep_canonical_identity_through_containers() {
    let uuid = Type::Class {
        identity: Some("sifr.uuid.UUID".to_string()),
        type_args: Vec::new(),
        name: "UUID".to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    };
    let array = Type::Class {
        identity: Some("sifr.sql.SqlArray".to_string()),
        type_args: vec![uuid],
        name: "SqlArray".to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    };
    assert_eq!(
        closed_type(&Type::Union(vec![array, Type::None])),
        Some(ClosedType::Optional {
            item: Box::new(ClosedType::Nominal {
                identity: "sifr.sql.SqlArray".to_string(),
                arguments: vec![ClosedType::Nominal {
                    identity: "sifr.uuid.UUID".to_string(),
                    arguments: Vec::new(),
                }],
            }),
        })
    );
}

#[test]
fn mysql_editor_links_follow_the_exact_profile_series() {
    let schema = SchemaIr {
        format_version: SCHEMA_IR_FORMAT_VERSION,
        provider: ProviderIdentity {
            package_id: "sifr-sql-mysql@0.0.0#editor".to_string(),
            package_version: Version::new(0, 0, 0),
            package_source: "workspace".to_string(),
            package_graph_digest: "a".repeat(64),
            compiler_components: BTreeMap::from([("mysql".to_string(), "b".repeat(64))]),
        },
        dialect: DialectIdentity {
            family: "mysql".to_string(),
            server_version: "8.4".to_string(),
            modes: BTreeSet::new(),
            features: BTreeSet::new(),
        },
        objects: BTreeMap::new(),
    };
    assert_eq!(
        provider_documentation_base(&schema).as_deref(),
        Some("https://dev.mysql.com/doc/refman/8.4/en/")
    );
}
