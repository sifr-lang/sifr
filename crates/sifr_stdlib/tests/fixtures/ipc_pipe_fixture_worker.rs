use sifr_stdlib::{
    read_frame, write_frame, IpcConnectionConfig, IpcConnectionState, IpcEnvelope,
    IpcHandshakeDecision, IpcMalformedKind, IpcShutdownMode, IpcTerminationReason, IpcWireSchema,
    IPC_DEFAULT_MAX_FRAME_BYTES,
};
use std::io::{stdin, stdout, StdinLock, StdoutLock};

fn main() -> std::process::ExitCode {
    match run_worker() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(()) => std::process::ExitCode::FAILURE,
    }
}

fn run_worker() -> Result<(), ()> {
    let mut input = stdin().lock();
    let mut output = stdout().lock();
    let mut connection =
        IpcConnectionState::new(IpcConnectionConfig::new(sample_schema())).map_err(|_| ())?;

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
                connection.apply_established_frame(&frame).map_err(|_| ())?;
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
                connection
                    .apply_established_frame(&IpcEnvelope::Run {
                        request_id,
                        payload: payload.clone(),
                    })
                    .map_err(|_| ())?;
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

fn read_or_report_malformed(
    input: &mut StdinLock<'_>,
    output: &mut StdoutLock<'_>,
) -> Result<Option<IpcEnvelope>, ()> {
    match read_frame(input, IPC_DEFAULT_MAX_FRAME_BYTES) {
        Ok(frame) => Ok(frame),
        Err(_) => {
            let malformed =
                IpcConnectionState::protocol_error_frame(IpcMalformedKind::Truncated, "truncated");
            write_frame(output, &malformed, IPC_DEFAULT_MAX_FRAME_BYTES).map_err(|_| ())?;
            Ok(None)
        }
    }
}

fn sample_schema() -> IpcWireSchema {
    IpcWireSchema {
        name: "demo.worker.Echo".to_string(),
        version: 1,
        hash: 0x4733_c89f_b23a_40ec_b5f3_bcda_99fb_34da_u128.to_be_bytes(),
        compatible_version_min: 1,
        compatible_version_max: 1,
    }
}
