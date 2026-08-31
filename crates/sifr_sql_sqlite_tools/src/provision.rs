use crate::command::{CommandError, command_error, load_authority};
use crate::lower_hex;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use sifr_sql_contract::{
    ProfileAuthority, ProvisionedCleanup, ProvisionedConnection, TEST_CONNECTION_MANIFEST_VERSION,
    TestConnectionManifest, schema_fingerprint,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const TEST_DIRECTORY: &str = ".sifr/sql-test/sqlite";

pub async fn provision_test_database(
    workspace_root: &Path,
    profile_name: &str,
) -> Result<TestConnectionManifest, CommandError> {
    let authority = load_authority(workspace_root, profile_name)?;
    provision_with_authority(workspace_root, profile_name, &authority)
}

fn provision_with_authority(
    workspace_root: &Path,
    profile_name: &str,
    authority: &ProfileAuthority,
) -> Result<TestConnectionManifest, CommandError> {
    if authority.profile.schema.dialect.family != "sqlite" {
        return Err(command_error("test provision requires a SQLite profile"));
    }
    let directory = workspace_root.join(TEST_DIRECTORY);
    fs::create_dir_all(&directory)
        .map_err(|_| command_error("cannot create the SQLite test directory"))?;
    let resource_id = unique_resource_id(profile_name)?;
    let path = directory.join(format!("{resource_id}.sqlite3"));
    let connection = Connection::open(&path)
        .map_err(|_| command_error("cannot create the SQLite test database"))?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|_| command_error("cannot enable SQLite test foreign keys"))?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(|_| command_error("cannot enable SQLite test WAL mode"))?;
    connection
        .pragma_update(None, "trusted_schema", false)
        .map_err(|_| command_error("cannot secure the SQLite test database"))?;
    drop(connection);
    let schema_fingerprint = schema_fingerprint(&authority.profile.schema)
        .map_err(|failure| command_error(failure.to_string()))?
        .as_str()
        .to_string();
    let manifest = TestConnectionManifest {
        schema_version: TEST_CONNECTION_MANIFEST_VERSION,
        provider: authority.profile.schema.provider.package_id.clone(),
        profile: profile_name.to_string(),
        schema_fingerprint,
        connection: ProvisionedConnection::File {
            path: path.to_string_lossy().into_owned(),
        },
        cleanup: ProvisionedCleanup {
            tool_namespace: "sql".to_string(),
            resource_id,
        },
        expires_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| duration.as_secs().checked_add(3_600)),
    };
    manifest
        .validate()
        .map_err(|failure| command_error(failure.to_string()))?;
    Ok(manifest)
}

pub async fn cleanup_test_database(
    workspace_root: &Path,
    resource_id: &str,
) -> Result<(), CommandError> {
    if !valid_resource_id(resource_id) {
        return Err(command_error("SQLite cleanup resource identity is invalid"));
    }
    let directory = canonical_existing_or_parent(&workspace_root.join(TEST_DIRECTORY))?;
    let path = directory.join(format!("{resource_id}.sqlite3"));
    let parent = path
        .parent()
        .ok_or_else(|| command_error("SQLite cleanup path has no parent"))?;
    if parent != directory || !path.is_file() {
        return Err(command_error(
            "SQLite cleanup resource does not exist in the owned test directory",
        ));
    }
    fs::remove_file(&path)
        .map_err(|_| command_error("cannot remove the provisioned SQLite database"))?;
    for suffix in ["-wal", "-shm", "-journal"] {
        let sidecar = PathBuf::from(format!("{}{suffix}", path.display()));
        if sidecar.is_file() {
            fs::remove_file(sidecar)
                .map_err(|_| command_error("cannot remove a SQLite test sidecar"))?;
        }
    }
    Ok(())
}

fn canonical_existing_or_parent(path: &Path) -> Result<PathBuf, CommandError> {
    if path.is_dir() {
        path.canonicalize()
            .map_err(|_| command_error("cannot resolve the SQLite test directory"))
    } else {
        fs::create_dir_all(path)
            .map_err(|_| command_error("cannot create the SQLite test directory"))?;
        path.canonicalize()
            .map_err(|_| command_error("cannot resolve the SQLite test directory"))
    }
}

fn unique_resource_id(profile: &str) -> Result<String, CommandError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| command_error("system clock cannot create a SQLite test identity"))?
        .as_nanos();
    let mut digest = Sha256::new();
    digest.update(profile.as_bytes());
    digest.update(timestamp.to_le_bytes());
    digest.update(std::process::id().to_le_bytes());
    Ok(format!(
        "sifr_sqlite_{}",
        &lower_hex(&digest.finalize())[..32]
    ))
}

fn valid_resource_id(value: &str) -> bool {
    value.starts_with("sifr_sqlite_")
        && value.len() == 44
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{cleanup_test_database, provision_with_authority};
    use semver::Version;
    use sifr_sql_contract::{
        DialectIdentity, PoolingMode, ProfileAuthority, ProviderIdentity, ProvisionedConnection,
        SCHEMA_IR_FORMAT_VERSION, SchemaEvidence, SchemaIr, SchemaProfile, SchemaStrictness,
        SessionContract, build_profile_authority,
    };
    use std::collections::{BTreeMap, BTreeSet};

    #[tokio::test(flavor = "current_thread")]
    async fn provision_manifest_and_cleanup_are_confined_to_the_owned_directory() {
        let workspace = tempfile::tempdir().expect("workspace");
        let manifest =
            provision_with_authority(workspace.path(), "app", &authority()).expect("provision");
        manifest.validate().expect("common manifest");
        let path = match &manifest.connection {
            ProvisionedConnection::File { path } => std::path::PathBuf::from(path),
            ProvisionedConnection::Tcp { .. } => panic!("SQLite provision returned TCP"),
        };
        assert!(path.is_file());
        assert!(path.starts_with(workspace.path().join(".sifr/sql-test/sqlite")));
        assert!(
            cleanup_test_database(workspace.path(), "production")
                .await
                .is_err()
        );
        cleanup_test_database(workspace.path(), &manifest.cleanup.resource_id)
            .await
            .expect("cleanup");
        assert!(!path.exists());
    }

    fn authority() -> ProfileAuthority {
        let provider = ProviderIdentity {
            package_id: "sifr-sql-sqlite@0.0.0#qualification".to_string(),
            package_version: Version::new(0, 0, 0),
            package_source: "workspace".to_string(),
            package_graph_digest: "a".repeat(64),
            compiler_components: BTreeMap::from([("sqlite".to_string(), "b".repeat(64))]),
        };
        let schema = SchemaIr {
            format_version: SCHEMA_IR_FORMAT_VERSION,
            provider,
            dialect: DialectIdentity {
                family: "sqlite".to_string(),
                server_version: "3.53.2".to_string(),
                modes: BTreeSet::new(),
                features: BTreeSet::from(["json".to_string()]),
            },
            objects: BTreeMap::new(),
        };
        build_profile_authority(SchemaProfile {
            package_id: "app@0.0.0#qualification".to_string(),
            name: "app".to_string(),
            source_files: BTreeSet::from(["db/schema.sql".to_string()]),
            source_fingerprints: BTreeMap::from([("db/schema.sql".to_string(), "c".repeat(64))]),
            evidence: SchemaEvidence::MigrationHead,
            strictness: SchemaStrictness::Exact,
            pooling: PoolingMode::Session,
            session: SessionContract {
                search_path: vec!["main".to_string()],
                ..SessionContract::default()
            },
            accepted_signers: BTreeSet::new(),
            capabilities: BTreeSet::from(["sql.query.select".to_string()]),
            schema,
        })
        .expect("authority")
    }
}
