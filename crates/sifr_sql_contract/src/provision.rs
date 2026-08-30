use crate::{SchemaContractError, SchemaContractErrorKind};
use serde::{Deserialize, Serialize};

pub const TEST_CONNECTION_MANIFEST_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestConnectionManifest {
    pub schema_version: u32,
    pub provider: String,
    pub profile: String,
    pub schema_fingerprint: String,
    pub connection: ProvisionedConnection,
    pub cleanup: ProvisionedCleanup,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_unix_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "kebab-case")]
pub enum ProvisionedConnection {
    Tcp {
        host: String,
        port: u16,
        database: String,
        user: String,
        credential: ProvisionedCredential,
        tls: bool,
    },
    File {
        path: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "kebab-case")]
pub enum ProvisionedCredential {
    Environment { variable: String },
    Helper { command: String },
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProvisionedCleanup {
    pub tool_namespace: String,
    pub resource_id: String,
}

impl TestConnectionManifest {
    pub fn from_json(source: &str) -> Result<Self, SchemaContractError> {
        let manifest: Self = serde_json::from_str(source).map_err(|error| {
            SchemaContractError::new(
                SchemaContractErrorKind::Serialization,
                format!("invalid test connection manifest: {error}"),
            )
        })?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn to_canonical_json(&self) -> Result<String, SchemaContractError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|error| {
            SchemaContractError::new(
                SchemaContractErrorKind::Serialization,
                format!("cannot serialize test connection manifest: {error}"),
            )
        })
    }

    pub fn validate(&self) -> Result<(), SchemaContractError> {
        if self.schema_version != TEST_CONNECTION_MANIFEST_VERSION {
            return Err(invalid(format!(
                "test connection manifest version {} is unsupported",
                self.schema_version
            )));
        }
        require_text("provider", &self.provider)?;
        require_text("profile", &self.profile)?;
        require_text("schema fingerprint", &self.schema_fingerprint)?;
        require_text("cleanup tool namespace", &self.cleanup.tool_namespace)?;
        require_text("cleanup resource id", &self.cleanup.resource_id)?;
        match &self.connection {
            ProvisionedConnection::Tcp {
                host,
                port,
                database,
                user,
                credential,
                ..
            } => {
                require_text("connection host", host)?;
                if *port == 0 {
                    return Err(invalid("connection port must be non-zero"));
                }
                require_text("connection database", database)?;
                require_text("connection user", user)?;
                match credential {
                    ProvisionedCredential::Environment { variable } => {
                        if !valid_environment_name(variable) {
                            return Err(invalid(format!(
                                "credential environment variable '{variable}' is invalid"
                            )));
                        }
                    }
                    ProvisionedCredential::Helper { command } => {
                        require_text("credential helper command", command)?;
                    }
                    ProvisionedCredential::None => {}
                }
            }
            ProvisionedConnection::File { path } => require_text("connection file path", path)?,
        }
        Ok(())
    }
}

fn require_text(label: &str, value: &str) -> Result<(), SchemaContractError> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(invalid(format!("{label} must be non-empty printable text")));
    }
    Ok(())
}

fn valid_environment_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_uppercase() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn invalid(message: impl Into<String>) -> SchemaContractError {
    SchemaContractError::new(SchemaContractErrorKind::InvalidProfile, message)
}
