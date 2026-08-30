use sifr_sql_contract::QuerySignatureArtifact;
use std::fs;
use std::path::{Path, PathBuf};

pub const QUERY_SIGNATURE_ARTIFACT_NAME: &str = "sifr-query-signatures.json";

pub fn emit_query_signature_artifact(
    output_dir: &Path,
    artifact: &QuerySignatureArtifact,
) -> Result<PathBuf, String> {
    fs::create_dir_all(output_dir).map_err(|error| {
        format!(
            "cannot create query-signature output directory '{}': {error}",
            output_dir.display()
        )
    })?;
    let destination = output_dir.join(QUERY_SIGNATURE_ARTIFACT_NAME);
    let temporary = output_dir.join(format!(".{QUERY_SIGNATURE_ARTIFACT_NAME}.tmp"));
    let payload = artifact.canonical_json().map_err(|error| error.message)?;
    fs::write(&temporary, payload).map_err(|error| {
        format!(
            "cannot write query-signature artifact '{}': {error}",
            temporary.display()
        )
    })?;
    fs::rename(&temporary, &destination).map_err(|error| {
        format!(
            "cannot publish query-signature artifact '{}': {error}",
            destination.display()
        )
    })?;
    Ok(destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_sql_contract::QuerySignatureArtifact;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn emission_atomically_replaces_a_canonical_artifact() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let output = std::env::temp_dir().join(format!(
            "sifr-query-signature-test-{}-{nonce}",
            std::process::id()
        ));
        let first = QuerySignatureArtifact::build("app@1", []).unwrap();
        let second = QuerySignatureArtifact::build("app@2", []).unwrap();

        let destination = emit_query_signature_artifact(&output, &first).unwrap();
        emit_query_signature_artifact(&output, &second).unwrap();
        let persisted: QuerySignatureArtifact =
            serde_json::from_slice(&fs::read(&destination).unwrap()).unwrap();

        assert_eq!(persisted, second);
        assert!(
            !output
                .join(format!(".{QUERY_SIGNATURE_ARTIFACT_NAME}.tmp"))
                .exists()
        );
        fs::remove_dir_all(output).unwrap();
    }
}
