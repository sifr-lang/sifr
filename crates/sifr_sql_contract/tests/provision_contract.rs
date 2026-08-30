use sifr_sql_contract::{
    ProvisionedCleanup, ProvisionedConnection, ProvisionedCredential,
    TEST_CONNECTION_MANIFEST_VERSION, TestConnectionManifest,
};

#[test]
fn connection_manifest_is_structured_and_never_contains_inline_credentials() {
    let manifest = TestConnectionManifest {
        schema_version: TEST_CONNECTION_MANIFEST_VERSION,
        provider: "postgresql".to_string(),
        profile: "app".to_string(),
        schema_fingerprint: "sha256:test".to_string(),
        connection: ProvisionedConnection::Tcp {
            host: "127.0.0.1".to_string(),
            port: 5432,
            database: "test".to_string(),
            user: "tester".to_string(),
            credential: ProvisionedCredential::Environment {
                variable: "SIFR_TEST_PASSWORD".to_string(),
            },
            tls: false,
        },
        cleanup: ProvisionedCleanup {
            tool_namespace: "sql".to_string(),
            resource_id: "fixture-1".to_string(),
        },
        expires_unix_seconds: None,
    };
    let encoded = manifest.to_canonical_json().expect("encode manifest");
    assert!(!encoded.contains("password-value"));
    assert_eq!(
        TestConnectionManifest::from_json(&encoded).expect("decode manifest"),
        manifest
    );

    let mut invalid = manifest.clone();
    invalid.connection = tcp_credential(ProvisionedCredential::Environment {
        variable: "not-safe".to_string(),
    });
    assert!(invalid.validate().is_err());

    let mut helper = manifest;
    helper.connection = tcp_credential(ProvisionedCredential::Helper {
        executable: "/usr/bin/security".to_string(),
        args: vec!["find-generic-password".to_string(), "-w".to_string()],
    });
    assert!(helper.validate().is_ok());
    if let ProvisionedConnection::Tcp { credential, .. } = &mut helper.connection
        && let ProvisionedCredential::Helper { executable, .. } = credential
    {
        *executable = "sh -c secret".to_string();
    }
    assert!(helper.validate().is_err());
}

fn tcp_credential(credential: ProvisionedCredential) -> ProvisionedConnection {
    ProvisionedConnection::Tcp {
        host: "127.0.0.1".to_string(),
        port: 5432,
        database: "test".to_string(),
        user: "tester".to_string(),
        credential,
        tls: false,
    }
}
