use bytes::Bytes;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
struct NestedPayload {
    label: String,
    tags: Vec<String>,
}

#[derive(Debug, Error)]
pub enum RoundtripError {
    #[error("invalid nested payload: {0}")]
    InvalidJson(#[from] serde_json::Error),
}

pub fn serde_json_roundtrip(input: &str) -> Result<String, RoundtripError> {
    let payload: NestedPayload = serde_json::from_str(input)?;
    Ok(serde_json::to_string(&payload)?)
}

pub fn bytes_roundtrip(input: &[u8]) -> Vec<u8> {
    Bytes::copy_from_slice(input).to_vec()
}

pub fn indexmap_roundtrip(
    input: &IndexMap<String, Vec<String>>,
) -> IndexMap<String, Vec<String>> {
    input.clone()
}

pub fn nested_indexmap_roundtrip(
    input: &IndexMap<String, IndexMap<String, String>>,
) -> IndexMap<String, IndexMap<String, String>> {
    input.clone()
}

pub fn indexmap_list_roundtrip(
    input: &[IndexMap<String, String>],
) -> Vec<IndexMap<String, String>> {
    input.to_vec()
}
