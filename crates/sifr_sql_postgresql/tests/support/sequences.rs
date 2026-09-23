use super::support::provider;
use sifr_sql_contract::{
    DdlReflection, MigrationDialect, ObjectId, SchemaIr, SemanticValue, normalize_schema,
};
use sifr_sql_postgresql::{
    LibpgQueryParser, PostgresCompilerComponent, PostgresComponentRequest,
    PostgresComponentResponse, PostgresMigrationDialect, PostgresParser, postgresql_capabilities,
};

const CREATE: &str = "CREATE TABLE owners (first bigint, second bigint); CREATE SEQUENCE seq AS integer OWNED BY owners.first;";

fn normalize(documents: &[(&str, &str)]) -> SchemaIr {
    let response = PostgresCompilerComponent::new(LibpgQueryParser).execute(
        PostgresComponentRequest::NormalizeSchema {
            provider: provider(),
            server_major: LibpgQueryParser.server_major(),
            documents: documents
                .iter()
                .map(|(name, sql)| (name.to_string(), sql.to_string()))
                .collect(),
        },
    );
    let PostgresComponentResponse::Schema(output) = response else {
        panic!("schema normalization failed: {response:?}");
    };
    normalize_schema(provider(), output.dialect, output.documents).expect("schema")
}

fn assert_owner(schema: &SchemaIr, owner: Option<&str>) {
    let sequence = &schema.objects[&ObjectId::new("public.seq")];
    assert_eq!(
        sequence.semantic.get("owned-by"),
        owner
            .map(|owner| SemanticValue::Text(owner.to_string()))
            .as_ref()
    );
    let expected = std::iter::once(ObjectId::new("public"))
        .chain(owner.map(ObjectId::new))
        .collect();
    assert_eq!(sequence.dependencies, expected);
    assert_eq!(
        sequence.semantic.get("data-type"),
        Some(&SemanticValue::Text("integer".to_string()))
    );
}

#[test]
fn sequence_ownership_updates_replace_edges_across_documents() {
    let schema = normalize(&[
        ("01.sql", CREATE),
        ("02.sql", "ALTER SEQUENCE seq OWNED BY owners.second;"),
    ]);
    assert_owner(&schema, Some("public.owners.second"));
    assert_eq!(
        schema.objects[&ObjectId::new("public.seq")]
            .source
            .as_ref()
            .unwrap()
            .document,
        "02.sql"
    );
    let schema = normalize(&[
        ("01.sql", CREATE),
        ("02.sql", "ALTER SEQUENCE seq OWNED BY owners.second;"),
        ("03.sql", "ALTER SEQUENCE seq OWNED BY NONE;"),
    ]);
    assert_owner(&schema, None);
    let schema = normalize(&[(
        "one.sql",
        &format!(
            "{CREATE} ALTER SEQUENCE seq OWNED BY owners.second; ALTER SEQUENCE seq OWNED BY NONE;"
        ),
    )]);
    assert_owner(&schema, None);
}

#[test]
fn sequence_migration_reflection_preserves_existing_object_changes() {
    let input = normalize(&[("01.sql", CREATE)]);
    let dialect = PostgresMigrationDialect::new(
        LibpgQueryParser,
        LibpgQueryParser.server_major().to_string(),
        postgresql_capabilities(),
    );
    let DdlReflection::Reflected { schema, .. } = dialect
        .reflect_ddl(&input, "ALTER SEQUENCE seq OWNED BY owners.second;")
        .unwrap()
    else {
        panic!("ownership reassignment must reflect");
    };
    assert_owner(&schema, Some("public.owners.second"));
    let DdlReflection::Reflected { schema, .. } = dialect
        .reflect_ddl(&schema, "ALTER SEQUENCE seq OWNED BY NONE;")
        .unwrap()
    else {
        panic!("ownership removal must reflect");
    };
    assert_owner(&schema, None);
    for sql in [
        "ALTER SEQUENCE seq OWNED BY owners.second INCREMENT BY 5;",
        "ALTER SEQUENCE seq CACHE 4 OWNED BY NONE;",
        "ALTER SEQUENCE seq OWNED BY owners.first OWNED BY owners.second;",
        "ALTER SEQUENCE seq RESTART;",
    ] {
        assert!(
            LibpgQueryParser.parse(sql).is_err(),
            "must reject the complete ALTER: {sql}"
        );
        assert!(
            matches!(
                dialect.reflect_ddl(&input, sql).unwrap(),
                DdlReflection::Opaque
            ),
            "must not falsely reflect: {sql}"
        );
    }
    for sql in [
        "ALTER SEQUENCE seq OWNED BY owners.missing;",
        "ALTER SEQUENCE owners OWNED BY NONE;",
        "ALTER SEQUENCE seq OWNED BY absent.first;",
    ] {
        assert!(
            dialect.reflect_ddl(&input, sql).is_err(),
            "invalid ownership must not reflect: {sql}"
        );
    }
}

#[test]
fn sequence_creation_requires_explicit_no_op_and_never_replaces_existing_objects() {
    let original = normalize(&[(
        "01.sql",
        "CREATE SEQUENCE seq AS integer INCREMENT 5 CACHE 3;",
    )]);
    let no_op = normalize(&[
        (
            "01.sql",
            "CREATE SEQUENCE seq AS integer INCREMENT 5 CACHE 3;",
        ),
        (
            "02.sql",
            "CREATE SEQUENCE IF NOT EXISTS seq AS bigint INCREMENT 9 CACHE 7;",
        ),
    ]);
    assert_eq!(original.objects, no_op.objects);

    for documents in [
        vec![(
            "one.sql".to_string(),
            "CREATE SEQUENCE seq; CREATE SEQUENCE seq;".to_string(),
        )],
        vec![
            ("01.sql".to_string(), "CREATE SEQUENCE seq;".to_string()),
            ("02.sql".to_string(), "CREATE SEQUENCE seq;".to_string()),
        ],
        vec![(
            "one.sql".to_string(),
            "CREATE TABLE seq (id integer); CREATE SEQUENCE seq;".to_string(),
        )],
    ] {
        let response = PostgresCompilerComponent::new(LibpgQueryParser).execute(
            PostgresComponentRequest::NormalizeSchema {
                provider: provider(),
                server_major: LibpgQueryParser.server_major(),
                documents,
            },
        );
        let PostgresComponentResponse::Diagnostic(diagnostic) = response else {
            panic!("duplicate CREATE SEQUENCE must fail: {response:?}");
        };
        assert!(
            diagnostic.message.contains("existing object"),
            "{diagnostic:?}"
        );
    }

    let no_missing_alter = normalize(&[
        ("01.sql", "CREATE SEQUENCE seq;"),
        ("02.sql", "ALTER SEQUENCE IF EXISTS absent OWNED BY NONE;"),
    ]);
    assert!(
        no_missing_alter
            .objects
            .contains_key(&ObjectId::new("public.seq"))
    );
}

#[test]
fn nextval_defaults_keep_sequence_identity_and_serial_has_an_explicit_boundary() {
    let schema = normalize(&[(
        "schema.sql",
        "CREATE SEQUENCE ids; CREATE TABLE entries (id bigint DEFAULT nextval('ids'::regclass));",
    )]);
    let column = &schema.objects[&ObjectId::new("public.entries.id")];
    assert_eq!(
        column.semantic.get("default-sequence"),
        Some(&SemanticValue::Text("public.ids".to_string()))
    );
    assert!(column.dependencies.contains(&ObjectId::new("public.ids")));

    for sql in [
        "CREATE TABLE entries (id serial);",
        "CREATE TABLE entries (id smallserial);",
        "CREATE TABLE entries (id bigserial);",
        "CREATE TABLE entries (id bigint DEFAULT nextval('missing'::regclass));",
    ] {
        let response = PostgresCompilerComponent::new(LibpgQueryParser).execute(
            PostgresComponentRequest::NormalizeSchema {
                provider: provider(),
                server_major: LibpgQueryParser.server_major(),
                documents: vec![("schema.sql".to_string(), sql.to_string())],
            },
        );
        assert!(
            matches!(response, PostgresComponentResponse::Diagnostic(_)),
            "{sql}: {response:?}"
        );
    }
}
