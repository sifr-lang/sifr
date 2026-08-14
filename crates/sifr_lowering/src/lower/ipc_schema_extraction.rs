use super::ipc_payload_calls;
use sifr_ipc::{IpcSchemaField, IpcSchemaType, IpcSchemaVariant};
use sifr_type_system::Type;

pub(in crate::lower) fn extract_ipc_schema_type(ty: &Type) -> IpcSchemaType {
    extract_ipc_schema_type_inner(ty.resolve_alias())
}

fn extract_ipc_schema_type_inner(ty: &Type) -> IpcSchemaType {
    if ipc_payload_calls::non_ipc_serializable_reason(ty).is_some() {
        return unsupported_schema_type(ty);
    }
    match ty {
        Type::Bool | Type::LiteralBool(_) => IpcSchemaType::Bool,
        Type::Int | Type::FixedInt(_) | Type::LiteralInt(_) => IpcSchemaType::Int,
        Type::Float => IpcSchemaType::Float,
        Type::Str | Type::LiteralStr(_) => IpcSchemaType::Str,
        Type::Bytes => IpcSchemaType::Bytes,
        Type::None | Type::Never => IpcSchemaType::None,
        Type::List(inner) => IpcSchemaType::List(Box::new(extract_ipc_schema_type(inner))),
        Type::Dict(_, value) => IpcSchemaType::DictStr(Box::new(extract_ipc_schema_type(value))),
        Type::Result(ok, err) => IpcSchemaType::Result(
            Box::new(extract_ipc_schema_type(ok)),
            Box::new(extract_ipc_schema_type(err)),
        ),
        Type::Tuple(items) => IpcSchemaType::Tuple(
            items
                .iter()
                .map(extract_ipc_schema_type)
                .collect::<Vec<_>>(),
        ),
        Type::Union(_) => {
            extract_option_schema_type(ty).unwrap_or_else(|| IpcSchemaType::Unsupported {
                type_name: ty.display_name(),
            })
        }
        Type::Alias { body, .. } | Type::Newtype { inner: body, .. } => {
            extract_ipc_schema_type(body)
        }
        Type::Class { name, fields, .. } => IpcSchemaType::Record {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(field_name, field_ty)| IpcSchemaField {
                    name: field_name.clone(),
                    ty: extract_ipc_schema_type(field_ty),
                })
                .collect::<Vec<_>>(),
        },
        Type::Enum { name, variants, .. } => IpcSchemaType::Enum {
            name: name.clone(),
            variants: variants
                .iter()
                .map(|(variant_name, _)| IpcSchemaVariant {
                    name: variant_name.clone(),
                    payload: None,
                })
                .collect::<Vec<_>>(),
        },
        Type::Set(_)
        | Type::PythonBuffer(_)
        | Type::PythonArrow(_)
        | Type::PythonDlpackTensor(_)
        | Type::PythonDlpackStream
        | Type::BigInt
        | Type::Decimal
        | Type::BigDecimal
        | Type::Task(_, _)
        | Type::TaskResult(_, _)
        | Type::BlockingTask(_, _)
        | Type::JoinSet(_, _)
        | Type::Coroutine(_, _)
        | Type::Awaitable(_)
        | Type::AsyncIterator(_, _)
        | Type::AsyncGenerator(_, _)
        | Type::Failure(_)
        | Type::TimeoutResult(_)
        | Type::Select2(_, _)
        | Type::Function(_)
        | Type::AsyncFunction(_)
        | Type::Callable(..)
        | Type::AsyncCallable(..)
        | Type::Iterator(_)
        | Type::Iterable(_)
        | Type::Range
        | Type::Any
        | Type::Unknown
        | Type::TypeVar(_)
        | Type::Protocol { .. }
        | Type::Intersection(_) => unsupported_schema_type(ty),
    }
}

fn extract_option_schema_type(ty: &Type) -> Option<IpcSchemaType> {
    let payload = ty.optional_member_type()?;
    Some(IpcSchemaType::Option(Box::new(extract_ipc_schema_type(
        &payload,
    ))))
}

fn unsupported_schema_type(ty: &Type) -> IpcSchemaType {
    IpcSchemaType::Unsupported {
        type_name: ty.display_name(),
    }
}

#[cfg(test)]
mod tests {
    use super::extract_ipc_schema_type;
    use sifr_ipc::{
        canonical_schema_descriptor, IpcSchemaDescriptor, IpcSchemaType, IpcWireSchema,
    };
    use sifr_type_system::{FunctionType, Type};

    fn generated_echo_descriptor() -> IpcSchemaDescriptor {
        let request = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "EchoRequest".to_string(),
            fields: vec![
                ("message".to_string(), Type::Str),
                (
                    "tags".to_string(),
                    Type::List(Box::new(Type::Union(vec![Type::None, Type::Str]))),
                ),
                (
                    "metadata".to_string(),
                    Type::Dict(Box::new(Type::Str), Box::new(Type::Bytes)),
                ),
                (
                    "outcome".to_string(),
                    Type::Result(Box::new(Type::Bool), Box::new(Type::Str)),
                ),
                (
                    "coords".to_string(),
                    Type::Tuple(vec![Type::Int, Type::Float]),
                ),
            ],
            methods: vec![],
            parent_class: None,
        };

        IpcSchemaDescriptor {
            protocol_schema_version: 1,
            module_path: "demo.ipc".to_string(),
            schema_name: "Echo".to_string(),
            compatible_version_min: 1,
            compatible_version_max: 1,
            request: extract_ipc_schema_type(&request),
            response: extract_ipc_schema_type(&Type::Enum {
                identity: None,
                name: "EchoStatus".to_string(),
                variants: vec![
                    ("Accepted".to_string(), Some(1)),
                    ("Rejected".to_string(), Some(2)),
                ],
            }),
            error: extract_ipc_schema_type(&Type::None),
        }
    }

    #[test]
    fn extracts_initial_payload_schema_families() {
        let descriptor = generated_echo_descriptor();

        assert_eq!(
            canonical_schema_descriptor(&descriptor),
            "sifr-ipc-schema-v1\nprotocol_schema_version=1\nmodule=demo.ipc\nschema=Echo\ncompatible=1..1\nrequest=record(EchoRequest{message:str,tags:list(option(str)),metadata:dict(str,bytes),outcome:result(bool,str),coords:tuple(int,float)})\nresponse=enum(EchoStatus{Accepted,Rejected})\nerror=none"
        );
    }

    #[cfg(unix)]
    #[test]
    fn generated_schema_drives_unix_fixture_worker_bootstrap_and_round_trip() {
        use sifr_ipc::{
            read_frame, schema_hash_hex_v1, schema_hash_v1, validate_ipc_payload_type, write_frame,
            IpcConnectionConfig, IpcConnectionState, IpcEnvelope, IpcShutdownMode,
            IpcTerminationReason, IPC_DEFAULT_MAX_FRAME_BYTES,
        };
        use std::path::{Path, PathBuf};
        use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

        struct WorkerProcess {
            child: Child,
            stdin: ChildStdin,
            stdout: ChildStdout,
        }

        impl WorkerProcess {
            fn spawn(schema_name: &str, schema_hash: &str) -> Self {
                let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                let repo_root = manifest_dir
                    .parent()
                    .and_then(Path::parent)
                    .expect("lowering crate lives under crates/sifr_lowering");
                let ipc_manifest = repo_root.join("crates").join("sifr_ipc").join("Cargo.toml");
                let target_dir = repo_root
                    .join("target")
                    .join("ipc_generated_worker_boundary_fixture");
                let mut child =
                    Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
                        .arg("run")
                        .arg("--quiet")
                        .arg("--manifest-path")
                        .arg(ipc_manifest)
                        .arg("--target-dir")
                        .arg(target_dir)
                        .arg("--features")
                        .arg("__test_fixture")
                        .arg("--bin")
                        .arg("sifr-ipc-pipe-fixture-worker")
                        .env("SIFR_IPC_FIXTURE_SCHEMA_NAME", schema_name)
                        .env("SIFR_IPC_FIXTURE_SCHEMA_HASH", schema_hash)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("spawn IPC fixture worker");
                let stdin = child.stdin.take().expect("worker stdin is piped");
                let stdout = child.stdout.take().expect("worker stdout is piped");
                Self {
                    child,
                    stdin,
                    stdout,
                }
            }

            fn finish(self) {
                drop(self.stdin);
                drop(self.stdout);
                let output = self
                    .child
                    .wait_with_output()
                    .expect("wait for IPC fixture worker");
                assert!(
                    output.status.success(),
                    "worker failed: status={:?}, stderr={}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        let descriptor = generated_echo_descriptor();
        validate_ipc_payload_type(&descriptor.request).expect("request schema is eligible");
        validate_ipc_payload_type(&descriptor.response).expect("response schema is eligible");
        validate_ipc_payload_type(&descriptor.error).expect("error schema is eligible");
        let schema_name = format!("{}.{}", descriptor.module_path, descriptor.schema_name);
        let wire_schema = IpcWireSchema {
            name: schema_name.clone(),
            version: 1,
            hash: schema_hash_v1(&descriptor).to_be_bytes(),
            compatible_version_min: 1,
            compatible_version_max: 1,
        };
        let mut worker = WorkerProcess::spawn(&schema_name, &schema_hash_hex_v1(&descriptor));
        let mut connection = IpcConnectionState::new(IpcConnectionConfig::new(wire_schema.clone()))
            .expect("generated schema config is valid");

        let hello = connection
            .begin_parent_handshake()
            .expect("parent begins generated-schema handshake");
        write_frame(&mut worker.stdin, &hello, IPC_DEFAULT_MAX_FRAME_BYTES)
            .expect("write generated-schema hello");
        let ready = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
            .expect("read generated-schema ready")
            .expect("worker emits ready");
        assert_eq!(
            ready,
            IpcEnvelope::Ready {
                protocol_version: 1,
                schema: wire_schema,
                max_frame_bytes: IPC_DEFAULT_MAX_FRAME_BYTES,
            }
        );
        connection
            .accept_worker_bootstrap(&ready)
            .expect("parent accepts generated-schema ready");

        let run = IpcEnvelope::Run {
            request_id: 101,
            payload: b"generated-schema".to_vec(),
        };
        connection
            .apply_established_frame(&run)
            .expect("parent reserves generated-schema request");
        write_frame(&mut worker.stdin, &run, IPC_DEFAULT_MAX_FRAME_BYTES)
            .expect("write generated-schema request");
        let started = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
            .expect("read started")
            .expect("worker emits started");
        assert_eq!(started, IpcEnvelope::Started { request_id: 101 });
        connection
            .apply_established_frame(&started)
            .expect("parent accepts generated-schema started");
        let completed = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
            .expect("read completed")
            .expect("worker emits completed");
        assert_eq!(
            completed,
            IpcEnvelope::Completed {
                request_id: 101,
                payload: b"generated-schema".to_vec(),
            }
        );
        connection
            .apply_established_frame(&completed)
            .expect("parent accepts generated-schema completion");

        let shutdown = IpcEnvelope::Shutdown {
            mode: IpcShutdownMode::Drain,
        };
        connection
            .apply_established_frame(&shutdown)
            .expect("parent enters draining state");
        write_frame(&mut worker.stdin, &shutdown, IPC_DEFAULT_MAX_FRAME_BYTES)
            .expect("write generated-schema shutdown");
        let terminating = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
            .expect("read terminating")
            .expect("worker emits terminating");
        assert_eq!(
            terminating,
            IpcEnvelope::Terminating {
                reason: IpcTerminationReason::Shutdown,
            }
        );
        connection
            .apply_established_frame(&terminating)
            .expect("parent accepts generated-schema terminating");
        worker.finish();
    }

    #[test]
    fn extracts_unsupported_payload_evidence() {
        let process_reader = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "PipeReader".to_string(),
            fields: vec![("_handle".to_string(), Type::Int)],
            methods: vec![],
            parent_class: None,
        };
        let callable = Type::Function(FunctionType::new(
            vec![("value".to_string(), Type::Int)],
            Type::Int,
        ));

        assert_eq!(
            extract_ipc_schema_type(&process_reader),
            IpcSchemaType::Unsupported {
                type_name: "PipeReader".to_string(),
            }
        );
        assert_eq!(
            extract_ipc_schema_type(&callable),
            IpcSchemaType::Unsupported {
                type_name: "function".to_string(),
            }
        );
    }
}
