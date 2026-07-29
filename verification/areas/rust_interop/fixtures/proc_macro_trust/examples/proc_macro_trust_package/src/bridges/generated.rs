#[derive(serde_derive::SifrGenerated)]
struct GeneratedSchema;

pub fn decode(input: &[u8]) -> String {
    format!(
        "id={}|payload={}|{}|{}",
        prost_build::schema_version(),
        String::from_utf8_lossy(input),
        GeneratedSchema::sifr_proc_macro_marker(),
        prost_build::generated_artifact(),
    )
}
