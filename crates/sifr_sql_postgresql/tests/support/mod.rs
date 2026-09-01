use semver::Version;
use sifr_sql_contract::{ProviderIdentity, normalize_schema};
use sifr_sql_postgresql::{
    LibpgQueryParser, PostgresCompilerComponent, PostgresComponentRequest,
    PostgresComponentResponse,
};
use std::collections::BTreeMap;

#[allow(dead_code)]
pub(crate) fn schema_for_writes(
    component: &PostgresCompilerComponent<LibpgQueryParser>,
    server_major: u16,
) -> sifr_sql_contract::SchemaIr {
    let response = component.execute(PostgresComponentRequest::NormalizeSchema {
        provider: provider(),
        server_major,
        documents: vec![(
            "db/write.sql".to_string(),
            "CREATE TABLE public.users (\
               id bigint PRIMARY KEY,\
               name text NOT NULL,\
               nickname text,\
               generated text GENERATED ALWAYS AS (name || id::text) STORED\
             );"
            .to_string(),
        )],
    });
    let PostgresComponentResponse::Schema(output) = response else {
        panic!("write schema must normalize: {response:?}");
    };
    normalize_schema(provider(), output.dialect, output.documents).unwrap()
}

pub(crate) fn schema_for_semantics(
    component: &PostgresCompilerComponent<LibpgQueryParser>,
    server_major: u16,
) -> sifr_sql_contract::SchemaIr {
    let response = component.execute(PostgresComponentRequest::NormalizeSchema {
        provider: provider(),
        server_major,
        documents: vec![(
            "db/semantic.sql".to_string(),
            "CREATE TABLE public.teams (id integer PRIMARY KEY, name text NOT NULL);\
             CREATE TABLE public.users (\
               id bigint PRIMARY KEY,\
               name text NOT NULL,\
               team_id integer NOT NULL REFERENCES public.teams(id),\
               nickname text\
             );"
            .to_string(),
        )],
    });
    let PostgresComponentResponse::Schema(output) = response else {
        panic!("semantic schema must normalize: {response:?}");
    };
    normalize_schema(provider(), output.dialect, output.documents).unwrap()
}

pub(crate) fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-postgresql@0.0.0#workspace".to_string(),
        package_version: Version::new(0, 0, 0),
        package_source: "workspace:crates/sifr_sql_postgresql".to_string(),
        package_graph_digest: "b".repeat(64),
        compiler_components: BTreeMap::from([(
            "sifr.sql.postgresql.sql".to_string(),
            "c".repeat(64),
        )]),
    }
}
