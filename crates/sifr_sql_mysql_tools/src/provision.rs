use crate::command::{CommandError, command_error, load_authority};
use crate::lower_hex;
use mysql_async::{Conn, Opts, prelude::Queryable};
use sha2::{Digest, Sha256};
use sifr_sql_contract::{
    ProfileAuthority, ProvisionedCleanup, ProvisionedConnection, ProvisionedCredential,
    TEST_CONNECTION_MANIFEST_VERSION, TestConnectionManifest, schema_fingerprint,
};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

const PASSWORD_ENVIRONMENT: &str = "SIFR_SQL_TEST_PASSWORD";

pub async fn provision_test_database(
    workspace_root: &Path,
    profile_name: &str,
    admin_url: &str,
    password: &str,
) -> Result<TestConnectionManifest, CommandError> {
    let authority = load_authority(workspace_root, profile_name)?;
    provision_with_authority(&authority, profile_name, admin_url, password).await
}

async fn provision_with_authority(
    authority: &ProfileAuthority,
    profile_name: &str,
    admin_url: &str,
    password: &str,
) -> Result<TestConnectionManifest, CommandError> {
    ensure_mysql(authority)?;
    if password.is_empty() || password.chars().any(char::is_control) {
        return Err(command_error(format!(
            "{PASSWORD_ENVIRONMENT} must contain a non-empty printable password"
        )));
    }
    let target =
        Url::parse(admin_url).map_err(|_| command_error("MySQL provisioning URL is invalid"))?;
    if target.scheme() != "mysql" {
        return Err(command_error("MySQL provisioning URL must use mysql://"));
    }
    let host = target
        .host_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| command_error("MySQL provisioning URL has no host"))?
        .to_string();
    let port = target.port().unwrap_or(3306);
    let suffix = unique_suffix(profile_name)?;
    let database = format!("sifr_test_{}", &suffix[..20]);
    let user = format!("sifr_{}", &suffix[..20]);
    let resource_id = format!("{database}.{user}");
    let opts = Opts::from_url(admin_url)
        .map_err(|_| command_error("MySQL provisioning connection is invalid"))?;
    let mut connection = Conn::new(opts)
        .await
        .map_err(|_| command_error("cannot connect to MySQL for test provisioning"))?;
    let result = create_test_principal(&mut connection, &database, &user, password).await;
    let _disconnect = connection.disconnect().await;
    result?;
    let schema_fingerprint = schema_fingerprint(&authority.profile.schema)
        .map_err(|error| command_error(error.to_string()))?
        .as_str()
        .to_string();
    let manifest = TestConnectionManifest {
        schema_version: TEST_CONNECTION_MANIFEST_VERSION,
        provider: authority.profile.schema.provider.package_id.clone(),
        profile: profile_name.to_string(),
        schema_fingerprint,
        connection: ProvisionedConnection::Tcp {
            host,
            port,
            database,
            user,
            credential: ProvisionedCredential::Environment {
                variable: PASSWORD_ENVIRONMENT.to_string(),
            },
            tls: target.query_pairs().any(|(key, value)| {
                key == "ssl-mode" && !matches!(value.as_ref(), "DISABLED" | "disabled")
            }),
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
        .map_err(|error| command_error(error.to_string()))?;
    Ok(manifest)
}

pub async fn cleanup_test_database(admin_url: &str, resource_id: &str) -> Result<(), CommandError> {
    let (database, user) = parse_resource_id(resource_id)?;
    let opts = Opts::from_url(admin_url)
        .map_err(|_| command_error("MySQL cleanup connection is invalid"))?;
    let mut connection = Conn::new(opts)
        .await
        .map_err(|_| command_error("cannot connect to MySQL for test cleanup"))?;
    let database_result = connection
        .query_drop(format!("DROP DATABASE IF EXISTS `{database}`"))
        .await
        .map_err(|_| command_error("cannot remove the provisioned MySQL database"));
    let user_result = connection
        .query_drop(format!("DROP USER IF EXISTS '{user}'@'%'"))
        .await
        .map_err(|_| command_error("cannot remove the provisioned MySQL user"));
    let _disconnect = connection.disconnect().await;
    database_result.and(user_result)
}

async fn create_test_principal(
    connection: &mut Conn,
    database: &str,
    user: &str,
    password: &str,
) -> Result<(), CommandError> {
    connection
        .query_drop(format!(
            "CREATE DATABASE `{database}` CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci"
        ))
        .await
        .map_err(|_| command_error("cannot create the provisioned MySQL database"))?;
    let sql_modes: String = connection
        .query_first("SELECT @@SESSION.sql_mode")
        .await
        .map_err(|_| command_error("cannot read MySQL SQL mode for test provisioning"))?
        .ok_or_else(|| command_error("MySQL returned no SQL mode for test provisioning"))?;
    let password = password_literal(password, &sql_modes);
    let create_user = connection
        .query_drop(format!("CREATE USER '{user}'@'%' IDENTIFIED BY {password}"))
        .await;
    if create_user.is_err() {
        let _cleanup = connection
            .query_drop(format!("DROP DATABASE IF EXISTS `{database}`"))
            .await;
        return Err(command_error("cannot create the provisioned MySQL user"));
    }
    let grant = connection
        .query_drop(format!("GRANT ALL ON `{database}`.* TO '{user}'@'%'"))
        .await;
    if grant.is_err() {
        let _cleanup_database = connection
            .query_drop(format!("DROP DATABASE IF EXISTS `{database}`"))
            .await;
        let _cleanup_user = connection
            .query_drop(format!("DROP USER IF EXISTS '{user}'@'%'"))
            .await;
        return Err(command_error("cannot grant the provisioned MySQL user"));
    }
    Ok(())
}

fn ensure_mysql(authority: &ProfileAuthority) -> Result<(), CommandError> {
    if authority.profile.schema.dialect.family == "mysql" {
        Ok(())
    } else {
        Err(command_error("test provision requires a MySQL profile"))
    }
}

fn unique_suffix(profile: &str) -> Result<String, CommandError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| command_error("system clock cannot create a test identity"))?
        .as_nanos();
    let mut digest = Sha256::new();
    digest.update(profile.as_bytes());
    digest.update(timestamp.to_le_bytes());
    digest.update(std::process::id().to_le_bytes());
    Ok(lower_hex(&digest.finalize()))
}

fn parse_resource_id(value: &str) -> Result<(&str, &str), CommandError> {
    let (database, user) = value
        .split_once('.')
        .ok_or_else(|| command_error("MySQL cleanup resource identity is invalid"))?;
    if valid_generated(database, "sifr_test_") && valid_generated(user, "sifr_") {
        Ok((database, user))
    } else {
        Err(command_error("MySQL cleanup resource identity is invalid"))
    }
}

fn valid_generated(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn password_literal(password: &str, sql_modes: &str) -> String {
    let no_backslash_escapes = sql_modes
        .split(',')
        .any(|mode| mode.eq_ignore_ascii_case("NO_BACKSLASH_ESCAPES"));
    let mut literal = String::with_capacity(password.len().saturating_add(2));
    literal.push('\'');
    for character in password.chars() {
        match character {
            '\'' => literal.push_str("''"),
            '\\' if !no_backslash_escapes => literal.push_str("\\\\"),
            _ => literal.push(character),
        }
    }
    literal.push('\'');
    literal
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{
        cleanup_test_database, parse_resource_id, password_literal, provision_with_authority,
    };
    use mysql_async::{Conn, Opts, prelude::Queryable};
    use semver::Version;
    use sifr_sql_contract::{
        DialectIdentity, PoolingMode, ProfileAuthority, ProviderIdentity, ProvisionedConnection,
        ProvisionedCredential, SCHEMA_IR_FORMAT_VERSION, SchemaEvidence, SchemaIr, SchemaProfile,
        SchemaStrictness, SessionContract, build_profile_authority,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use url::Url;

    #[test]
    fn cleanup_accepts_only_generated_resource_identities() {
        assert!(parse_resource_id("sifr_test_abc.sifr_abc").is_ok());
        assert!(parse_resource_id("production.root").is_err());
        assert!(parse_resource_id("sifr_test_a.sifr_a;DROP").is_err());
    }

    #[test]
    fn password_literal_obeys_the_live_backslash_mode() {
        assert_eq!(
            password_literal("a'b\\c", "STRICT_TRANS_TABLES"),
            "'a''b\\\\c'"
        );
        assert_eq!(
            password_literal("a'b\\c", "NO_BACKSLASH_ESCAPES"),
            "'a''b\\c'"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore = "requires SIFR_MYSQL_TEST_URL"]
    async fn live_provision_manifest_credentials_and_cleanup_are_safe() {
        const PASSWORD: &str = "sifr'\\provisioned-qualification";
        let admin_url = std::env::var("SIFR_MYSQL_TEST_URL").expect("admin URL");
        let series = std::env::var("SIFR_MYSQL_TEST_SERIES").expect("server series");
        let authority = authority(&series);
        let manifest = provision_with_authority(&authority, "app", &admin_url, PASSWORD)
            .await
            .expect("provision test database");
        manifest.validate().expect("common connection manifest");
        let canonical = manifest.to_canonical_json().expect("canonical manifest");
        assert!(!canonical.contains(PASSWORD));
        assert!(!canonical.contains(&admin_url));

        let (host, port, database, user, credential) = match &manifest.connection {
            ProvisionedConnection::Tcp {
                host,
                port,
                database,
                user,
                credential,
                ..
            } => (host, port, database, user, credential),
            ProvisionedConnection::File { .. } => panic!("MySQL provision returned a file"),
        };
        assert_eq!(
            credential,
            &ProvisionedCredential::Environment {
                variable: "SIFR_SQL_TEST_PASSWORD".to_string(),
            }
        );
        let mut provisioned_url = Url::parse(&admin_url).expect("parse admin URL");
        provisioned_url.set_host(Some(host)).expect("set host");
        provisioned_url.set_port(Some(*port)).expect("set port");
        provisioned_url.set_username(user).expect("set user");
        provisioned_url
            .set_password(Some(PASSWORD))
            .expect("set password");
        provisioned_url.set_path(&format!("/{database}"));
        let probe = async {
            let mut connection =
                Conn::new(Opts::from_url(provisioned_url.as_str()).expect("provisioned options"))
                    .await?;
            let value: Option<u8> = connection.query_first("SELECT 1").await?;
            connection.disconnect().await?;
            Ok::<_, mysql_async::Error>(value)
        }
        .await;
        let cleanup = cleanup_test_database(&admin_url, &manifest.cleanup.resource_id).await;
        cleanup.expect("cleanup provisioned database and user");
        assert_eq!(probe.expect("connect with provisioned credential"), Some(1));
    }

    fn authority(series: &str) -> ProfileAuthority {
        let modes = BTreeSet::from([
            "character-set:utf8mb4".to_string(),
            "collation:utf8mb4_0900_ai_ci".to_string(),
        ]);
        let provider = ProviderIdentity {
            package_id: "sifr-sql-mysql@0.0.0#qualification".to_string(),
            package_version: Version::new(0, 0, 0),
            package_source: "workspace".to_string(),
            package_graph_digest: "a".repeat(64),
            compiler_components: BTreeMap::from([("mysql".to_string(), "b".repeat(64))]),
        };
        let schema = SchemaIr {
            format_version: SCHEMA_IR_FORMAT_VERSION,
            provider,
            dialect: DialectIdentity {
                family: "mysql".to_string(),
                server_version: series.to_string(),
                modes,
                features: BTreeSet::new(),
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
                search_path: vec!["app".to_string()],
                character_set: Some("utf8mb4".to_string()),
                collation: Some("utf8mb4_0900_ai_ci".to_string()),
                ..SessionContract::default()
            },
            accepted_signers: BTreeSet::new(),
            capabilities: BTreeSet::from(["sql.query.select".to_string()]),
            schema,
        })
        .expect("test profile authority")
    }
}
