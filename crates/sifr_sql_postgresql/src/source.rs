use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SUPPORTED_POSTGRESQL_MAJORS: [u16; 6] = [13, 14, 15, 16, 17, 18];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibpgQuerySource {
    pub server_major: u16,
    pub tag: String,
    pub commit: String,
    pub source_content_sha256: String,
    pub path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentSourceManifest {
    schema_version: u32,
    component_family: String,
    target: String,
    adapter: String,
    sources: Vec<LibpgQuerySource>,
}

pub fn embedded_sources() -> Result<Vec<LibpgQuerySource>, serde_json::Error> {
    let manifest: ComponentSourceManifest = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/component-sources.json"
    )))?;
    if manifest.schema_version != 1
        || manifest.component_family != "sifr.sql.postgresql.sql"
        || manifest.target != "wasm32-wasip2"
        || manifest.adapter != "provider-owned-json-ast-v1"
    {
        return Err(<serde_json::Error as serde::de::Error>::custom(
            "invalid PostgreSQL component source manifest authority",
        ));
    }
    let majors = manifest
        .sources
        .iter()
        .map(|source| source.server_major)
        .collect::<Vec<_>>();
    let unique = majors.iter().copied().collect::<BTreeSet<_>>();
    let valid_sources = majors == SUPPORTED_POSTGRESQL_MAJORS
        && unique.len() == SUPPORTED_POSTGRESQL_MAJORS.len()
        && manifest.sources.iter().all(|source| {
            source.commit.len() == 40
                && source.commit.bytes().all(|value| value.is_ascii_hexdigit())
                && source.source_content_sha256.len() == 64
                && source
                    .source_content_sha256
                    .bytes()
                    .all(|value| value.is_ascii_hexdigit())
                && source.tag.starts_with(&source.server_major.to_string())
        });
    if !valid_sources {
        return Err(<serde_json::Error as serde::de::Error>::custom(
            "invalid PostgreSQL parser source matrix",
        ));
    }
    Ok(manifest.sources)
}

pub fn embedded_source(server_major: u16) -> Result<Option<LibpgQuerySource>, serde_json::Error> {
    Ok(embedded_sources()?
        .into_iter()
        .find(|source| source.server_major == server_major))
}
