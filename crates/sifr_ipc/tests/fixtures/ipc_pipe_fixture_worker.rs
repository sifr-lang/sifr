use sifr_ipc::{
    read_frame, write_frame, IpcConnectionConfig, IpcConnectionError, IpcConnectionState,
    IpcEnvelope, IpcHandshakeDecision, IpcMalformedKind, IpcRequestTrackerError, IpcShutdownMode,
    IpcTerminationReason, IpcWireSchema, IPC_DEFAULT_MAX_FRAME_BYTES,
};
use std::env;
use std::io::{stdin, stdout, StdinLock, StdoutLock};

const UNSUPPORTED_PREFIX: &[u8] = b"unsupported:";
const DEFAULT_SCHEMA_HASH: [u8; 16] = 0x4733_c89f_b23a_40ec_b5f3_bcda_99fb_34da_u128.to_be_bytes();

fn main() -> std::process::ExitCode {
    match run_worker() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(()) => std::process::ExitCode::FAILURE,
    }
}

fn run_worker() -> Result<(), ()> {
    let mut input = stdin().lock();
    let mut output = stdout().lock();
    let mut connection = IpcConnectionState::new(IpcConnectionConfig {
        max_in_flight: 1,
        ..IpcConnectionConfig::new(sample_schema())
    })
    .map_err(|_| ())?;

    let Some(hello) = read_or_report_malformed(&mut input, &mut output)? else {
        return Err(());
    };
    match connection.accept_parent_hello(&hello).map_err(|_| ())? {
        IpcHandshakeDecision::Ready(frame) | IpcHandshakeDecision::Reject(frame) => {
            write_frame(&mut output, &frame, IPC_DEFAULT_MAX_FRAME_BYTES).map_err(|_| ())?;
            if matches!(frame, IpcEnvelope::Reject { .. }) {
                return Ok(());
            }
        }
    }

    while let Some(frame) = read_or_report_malformed(&mut input, &mut output)? {
        match frame {
            IpcEnvelope::Run {
                request_id,
                ref payload,
            } if payload == b"hold" => {
                if let Err(error) = connection.apply_established_frame(&frame) {
                    return write_connection_error(&mut output, &mut connection, &error);
                }
                write_frame(
                    &mut output,
                    &IpcEnvelope::Started { request_id },
                    IPC_DEFAULT_MAX_FRAME_BYTES,
                )
                .map_err(|_| ())?;
            }
            IpcEnvelope::Run {
                request_id,
                payload,
            } => {
                if let Some(type_name) = unsupported_type_name(&payload) {
                    let unsupported = IpcEnvelope::UnsupportedPayload { type_name };
                    connection
                        .apply_established_frame(&unsupported)
                        .map_err(|_| ())?;
                    write_frame(&mut output, &unsupported, IPC_DEFAULT_MAX_FRAME_BYTES)
                        .map_err(|_| ())?;
                    return Ok(());
                }
                if let Err(error) = connection.apply_established_frame(&IpcEnvelope::Run {
                    request_id,
                    payload: payload.clone(),
                }) {
                    return write_connection_error(&mut output, &mut connection, &error);
                }
                write_frame(
                    &mut output,
                    &IpcEnvelope::Started { request_id },
                    IPC_DEFAULT_MAX_FRAME_BYTES,
                )
                .map_err(|_| ())?;
                let completed = IpcEnvelope::Completed {
                    request_id,
                    payload,
                };
                connection
                    .apply_established_frame(&completed)
                    .map_err(|_| ())?;
                write_frame(&mut output, &completed, IPC_DEFAULT_MAX_FRAME_BYTES)
                    .map_err(|_| ())?;
            }
            IpcEnvelope::Cancel { request_id } => {
                connection
                    .apply_established_frame(&IpcEnvelope::Cancel { request_id })
                    .map_err(|_| ())?;
                let failed = IpcEnvelope::Failed {
                    request_id,
                    error: b"cancelled".to_vec(),
                };
                connection
                    .apply_established_frame(&failed)
                    .map_err(|_| ())?;
                write_frame(&mut output, &failed, IPC_DEFAULT_MAX_FRAME_BYTES).map_err(|_| ())?;
            }
            IpcEnvelope::Shutdown { mode } => {
                connection
                    .apply_established_frame(&IpcEnvelope::Shutdown { mode })
                    .map_err(|_| ())?;
                let terminating = IpcEnvelope::Terminating {
                    reason: match mode {
                        IpcShutdownMode::Drain => IpcTerminationReason::Shutdown,
                        IpcShutdownMode::CancelInFlight => IpcTerminationReason::Cancelled,
                    },
                };
                connection
                    .apply_established_frame(&terminating)
                    .map_err(|_| ())?;
                write_frame(&mut output, &terminating, IPC_DEFAULT_MAX_FRAME_BYTES)
                    .map_err(|_| ())?;
                return Ok(());
            }
            other => {
                connection.apply_established_frame(&other).map_err(|_| ())?;
            }
        }
    }
    Ok(())
}

fn write_connection_error(
    output: &mut StdoutLock<'_>,
    connection: &mut IpcConnectionState,
    error: &IpcConnectionError,
) -> Result<(), ()> {
    let frame = match error {
        IpcConnectionError::Request(IpcRequestTrackerError::BackpressureFull { .. }) => {
            IpcConnectionState::protocol_error_frame(
                IpcMalformedKind::RequestId,
                "backpressure_full",
            )
        }
        IpcConnectionError::Request(IpcRequestTrackerError::DuplicateRequestId { .. }) => {
            IpcConnectionState::protocol_error_frame(
                IpcMalformedKind::DuplicateRequestId,
                "duplicate_request_id",
            )
        }
        IpcConnectionError::Request(IpcRequestTrackerError::UnknownRequestId { .. }) => {
            IpcConnectionState::protocol_error_frame(
                IpcMalformedKind::RequestId,
                "unknown_request_id",
            )
        }
        _ => IpcConnectionState::protocol_error_frame(IpcMalformedKind::State, "connection"),
    };
    connection.apply_established_frame(&frame).map_err(|_| ())?;
    write_frame(output, &frame, IPC_DEFAULT_MAX_FRAME_BYTES).map_err(|_| ())
}

fn read_or_report_malformed(
    input: &mut StdinLock<'_>,
    output: &mut StdoutLock<'_>,
) -> Result<Option<IpcEnvelope>, ()> {
    if let Ok(frame) = read_frame(input, IPC_DEFAULT_MAX_FRAME_BYTES) {
        return Ok(frame);
    }
    let malformed =
        IpcConnectionState::protocol_error_frame(IpcMalformedKind::Truncated, "truncated");
    write_frame(output, &malformed, IPC_DEFAULT_MAX_FRAME_BYTES).map_err(|_| ())?;
    Ok(None)
}

fn unsupported_type_name(payload: &[u8]) -> Option<String> {
    let suffix = payload.strip_prefix(UNSUPPORTED_PREFIX)?;
    std::str::from_utf8(suffix).ok().map(ToString::to_string)
}

fn sample_schema() -> IpcWireSchema {
    IpcWireSchema {
        name: env::var("SIFR_IPC_FIXTURE_SCHEMA_NAME")
            .unwrap_or_else(|_| "demo.worker.Echo".to_string()),
        version: 1,
        hash: env_schema_hash().unwrap_or(DEFAULT_SCHEMA_HASH),
        compatible_version_min: 1,
        compatible_version_max: 1,
    }
}

fn env_schema_hash() -> Option<[u8; 16]> {
    let encoded = env::var("SIFR_IPC_FIXTURE_SCHEMA_HASH").ok()?;
    if encoded.len() != 32 {
        return None;
    }
    let mut hash = [0_u8; 16];
    for (index, slot) in hash.iter_mut().enumerate() {
        let start = index * 2;
        let end = start + 2;
        let byte = u8::from_str_radix(encoded.get(start..end)?, 16).ok()?;
        *slot = byte;
    }
    Some(hash)
}
