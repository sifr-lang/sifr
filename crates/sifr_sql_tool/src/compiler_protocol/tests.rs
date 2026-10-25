use super::*;
use serde_json::json;
use sifr_sql_contract::{
    DialectIdentity, MigrationNodeId, MigrationSourceStep, MigrationSourceStepKind, PoolingMode,
    ProviderIdentity, SchemaEvidence, SchemaIr, SchemaProfile, SchemaStrictness, SessionContract,
};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Cursor};

const SOURCE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

struct Script {
    input: Cursor<Vec<u8>>,
    output: Vec<u8>,
}
impl Script {
    fn new(frames: Vec<IpcEnvelope>) -> Self {
        let mut bytes = Vec::new();
        for frame in frames {
            write(&mut bytes, &frame).unwrap();
        }
        Self {
            input: Cursor::new(bytes),
            output: Vec::new(),
        }
    }
}
impl Read for Script {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.input.read(bytes)
    }
}
impl Write for Script {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.output.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn hello() -> IpcEnvelope {
    IpcEnvelope::Hello {
        protocol_min: VERSION,
        protocol_max: VERSION,
        schema: schema(SOURCE).unwrap(),
        max_frame_bytes: MAX_FRAME_BYTES,
    }
}
fn ready() -> IpcEnvelope {
    IpcEnvelope::Ready {
        protocol_version: VERSION,
        schema: schema(SOURCE).unwrap(),
        max_frame_bytes: MAX_FRAME_BYTES,
    }
}
fn completed(response: &Response) -> IpcEnvelope {
    IpcEnvelope::Completed {
        request_id: 1,
        payload: encode(response).unwrap(),
    }
}
fn declaration() -> MigrationSourceDeclaration {
    let kinds = vec![
        MigrationSourceStepKind::Begin,
        MigrationSourceStepKind::RecoveryPoint {
            name: "before-schema".into(),
        },
        MigrationSourceStepKind::Ddl {
            statement: "CREATE TABLE t(id INTEGER)".into(),
        },
        MigrationSourceStepKind::SqlData {
            statement: "INSERT INTO t VALUES (1)".into(),
        },
        MigrationSourceStepKind::Assertion {
            statement: "SELECT COUNT(*) FROM t".into(),
        },
        MigrationSourceStepKind::Commit,
    ];
    let steps: Vec<_> = kinds
        .into_iter()
        .enumerate()
        .map(|(i, kind)| MigrationSourceStep {
            id: MigrationNodeId::new(format!("step-{i}")).unwrap(),
            kind,
        })
        .collect();
    MigrationSourceDeclaration {
        id: MigrationNodeId::new("new-table").unwrap(),
        parents: vec![MigrationNodeId::new("base").unwrap()],
        function: "apply_table".into(),
        author: "author".into(),
        created_at: "2026-10-25".into(),
        rollback: Some(steps.clone()),
        steps,
    }
}
fn authority() -> ProfileAuthority {
    build_profile_authority(SchemaProfile {
        package_id: "app@1.0.0#path".into(),
        name: "app".into(),
        source_files: BTreeSet::from(["db/schema.sql".into()]),
        source_fingerprints: BTreeMap::from([("db/schema.sql".into(), "b".repeat(64))]),
        evidence: SchemaEvidence::Introspection,
        strictness: SchemaStrictness::Exact,
        pooling: PoolingMode::Session,
        session: SessionContract::default(),
        accepted_signers: BTreeSet::new(),
        capabilities: BTreeSet::from(["sql.query.select".into()]),
        schema: SchemaIr {
            format_version: 1,
            provider: ProviderIdentity {
                package_id: "sifr-sql-sqlite".into(),
                package_version: semver::Version::new(0, 0, 0),
                package_source: "path+sqlite".into(),
                package_graph_digest: "c".repeat(64),
                compiler_components: BTreeMap::new(),
            },
            dialect: DialectIdentity {
                family: "sqlite".into(),
                server_version: "3.53.4".into(),
                modes: BTreeSet::new(),
                features: BTreeSet::new(),
            },
            objects: BTreeMap::new(),
        },
    })
    .unwrap()
}
#[derive(Default)]
struct Operations {
    calls: usize,
}
impl CompilerOperations for Operations {
    fn profile(&mut self, name: &str) -> Result<ProfileAuthority, String> {
        self.calls += 1;
        assert_eq!(name, "app");
        Ok(authority())
    }
    fn migration(&mut self, source: &str) -> Result<Vec<MigrationSourceDeclaration>, String> {
        self.calls += 1;
        assert_eq!(source, "admitted source");
        Ok(vec![declaration()])
    }
}

#[test]
#[cfg(unix)]
fn duplex_session_preserves_profiles_all_steps_and_shutdown() {
    use std::os::unix::net::UnixStream;
    let (left, mut right) = UnixStream::pair().unwrap();
    for stream in [&left, &right] {
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
    }
    let worker = std::thread::spawn(move || {
        let mut operations = Operations::default();
        serve(&mut right, SOURCE, &mut operations).unwrap();
        assert_eq!(operations.calls, 2);
    });
    let mut client = CompilerClient::connect(left, SOURCE).unwrap();
    assert_eq!(client.profile("app").unwrap(), authority());
    assert_eq!(
        client.migration("admitted source").unwrap(),
        vec![declaration()]
    );
    client.finish().unwrap();
    worker.join().unwrap();
}

#[test]
fn declarations_reject_invalid_ids_in_every_position_and_unknown_fields() {
    let original = serde_json::to_value(declarations::Declaration::from(&declaration())).unwrap();
    for pointer in ["/id", "/parents/0", "/steps/0/id", "/rollback/0/id"] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = json!("../outside");
        let wire: declarations::Declaration = serde_json::from_value(changed).unwrap();
        assert!(
            MigrationSourceDeclaration::try_from(wire).is_err(),
            "{pointer}"
        );
    }
    for pointer in ["", "/steps/0", "/steps/0/kind", "/rollback/0"] {
        let mut changed = original.clone();
        changed
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), json!(true));
        assert!(
            serde_json::from_value::<declarations::Declaration>(changed).is_err(),
            "{pointer}"
        );
    }
    for index in 0..6 {
        let mut changed = original.clone();
        changed["steps"][index]["kind"]
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), json!(true));
        assert!(
            serde_json::from_value::<declarations::Declaration>(changed).is_err(),
            "{index}"
        );
    }
    let mut changed = original;
    changed["steps"][0]["kind"]["kind"] = json!("execute_arbitrary");
    assert!(serde_json::from_value::<declarations::Declaration>(changed).is_err());
}

#[test]
fn handshake_rejects_source_version_schema_and_frame_limit_mismatches() {
    let mut variants = Vec::new();
    for field in ["source", "version", "schema", "limit"] {
        let mut frame = ready();
        if let IpcEnvelope::Ready {
            protocol_version,
            schema,
            max_frame_bytes,
        } = &mut frame
        {
            match field {
                "source" => schema.name.push('b'),
                "version" => *protocol_version += 1,
                "schema" => schema.hash[0] ^= 1,
                "limit" => *max_frame_bytes -= 1,
                _ => unreachable!(),
            }
        }
        variants.push(frame);
    }
    variants.push(IpcEnvelope::Started { request_id: 1 });
    for frame in variants {
        assert!(CompilerClient::connect(Script::new(vec![frame]), SOURCE).is_err());
    }
    for source in ["", "a", &"g".repeat(64)] {
        assert!(CompilerClient::connect(Script::new(vec![ready()]), source).is_err());
    }
}

#[test]
fn response_checks_reject_wrong_ids_operations_invalid_profiles_and_replay() {
    let mut wrong_id = completed(&Response::Profile {
        authority: authority(),
    });
    if let IpcEnvelope::Completed { request_id, .. } = &mut wrong_id {
        *request_id = 2;
    }
    let mut inconsistent = authority();
    inconsistent.nominal_identity.push_str("-wrong");
    let frames = vec![
        wrong_id,
        completed(&Response::Migration {
            declarations: vec![],
        }),
        completed(&Response::Profile {
            authority: inconsistent,
        }),
        IpcEnvelope::Started { request_id: 1 },
        IpcEnvelope::Failed {
            request_id: 1,
            error: vec![255],
        },
        IpcEnvelope::Completed {
            request_id: 1,
            payload: b"{} trailing".to_vec(),
        },
    ];
    for response in frames {
        let mut client =
            CompilerClient::connect(Script::new(vec![ready(), response]), SOURCE).unwrap();
        assert!(client.profile("app").is_err());
        assert!(
            client
                .profile("app")
                .unwrap_err()
                .contains("already failed")
        );
        assert!(client.finish().is_err());
    }
    let response = completed(&Response::Profile {
        authority: authority(),
    });
    let mut client = CompilerClient::connect(
        Script::new(vec![ready(), response.clone(), response]),
        SOURCE,
    )
    .unwrap();
    assert!(client.profile("app").is_ok());
    assert!(client.profile("app").is_err());
    let mut client = CompilerClient::connect(
        Script::new(vec![
            ready(),
            completed(&Response::Profile {
                authority: authority(),
            }),
        ]),
        SOURCE,
    )
    .unwrap();
    assert!(client.profile("other-profile").is_err());
}

#[test]
fn semantic_diagnostics_are_preserved_exactly_and_eof_is_failure() {
    let diagnostic = "line 3: invalid migration\nline 4: unknown name";
    let mut client = CompilerClient::connect(
        Script::new(vec![
            ready(),
            IpcEnvelope::Failed {
                request_id: 1,
                error: diagnostic.as_bytes().to_vec(),
            },
        ]),
        SOURCE,
    )
    .unwrap();
    assert_eq!(client.migration("admitted source").unwrap_err(), diagnostic);
    let mut client = CompilerClient::connect(Script::new(vec![ready()]), SOURCE).unwrap();
    assert!(
        client
            .migration("admitted source")
            .unwrap_err()
            .contains("closed")
    );
}

#[test]
fn server_rejects_unknown_operation_fields_versions_and_invalid_sequence_before_execution() {
    let variants = vec![
        IpcEnvelope::Run {
            request_id: 0,
            payload: encode(&Request::Profile { name: "app".into() }).unwrap(),
        },
        IpcEnvelope::Run {
            request_id: 1,
            payload: br#"{"operation":"exec","program":"sh"}"#.to_vec(),
        },
        IpcEnvelope::Run {
            request_id: 1,
            payload: br#"{"operation":"profile","name":"app","root":"/outside"}"#.to_vec(),
        },
        IpcEnvelope::Run {
            request_id: 1,
            payload: br#"{"operation":"profile","name":"app","name":"duplicate"}"#.to_vec(),
        },
        IpcEnvelope::Completed {
            request_id: 1,
            payload: Vec::new(),
        },
    ];
    for variant in variants {
        let mut script = Script::new(vec![hello(), variant]);
        let mut operations = Operations::default();
        assert!(serve(&mut script, SOURCE, &mut operations).is_err());
        assert_eq!(operations.calls, 0);
    }
    let mut bad_hello = hello();
    if let IpcEnvelope::Hello { protocol_min, .. } = &mut bad_hello {
        *protocol_min = 0;
    }
    let mut script = Script::new(vec![bad_hello]);
    let mut operations = Operations::default();
    assert!(serve(&mut script, SOURCE, &mut operations).is_err());
    assert!(script.output.is_empty());
    let request = IpcEnvelope::Run {
        request_id: 1,
        payload: encode(&Request::Profile { name: "app".into() }).unwrap(),
    };
    let mut script = Script::new(vec![hello(), request.clone(), request]);
    assert!(serve(&mut script, SOURCE, &mut operations).is_err());
    assert_eq!(operations.calls, 1);
}

#[test]
fn framing_rejects_oversized_truncated_and_trailing_payloads() {
    let valid = sifr_ipc::encode_frame(&ready(), MAX_FRAME_BYTES).unwrap();
    for bytes in [
        (MAX_FRAME_BYTES + 1).to_le_bytes().to_vec(),
        vec![1, 2],
        valid[..valid.len() - 1].to_vec(),
        {
            let mut bytes = valid.clone();
            let length = u32::from_le_bytes(bytes[..4].try_into().unwrap()) + 1;
            bytes[..4].copy_from_slice(&length.to_le_bytes());
            bytes.push(0);
            bytes
        },
    ] {
        let stream = Script {
            input: Cursor::new(bytes),
            output: Vec::new(),
        };
        assert!(CompilerClient::connect(stream, SOURCE).is_err());
    }
    let mut client = CompilerClient::connect(Script::new(vec![ready()]), SOURCE).unwrap();
    assert!(
        client
            .migration(&"s".repeat(MAX_FRAME_BYTES as usize))
            .is_err()
    );
}
