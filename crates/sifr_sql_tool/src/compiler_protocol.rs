//! Bounded SQL compiler messages. The launcher owns executable selection, input
//! admission and deadlines; matching wire identities does not authenticate a peer.
use serde::{Deserialize, Serialize};
use sifr_ipc::{
    IPC_DEFAULT_MAX_FRAME_BYTES, IpcEnvelope, IpcShutdownMode, IpcTerminationReason, IpcWireSchema,
    read_frame, write_frame,
};
use sifr_sql_contract::{MigrationSourceDeclaration, ProfileAuthority, build_profile_authority};
use std::io::{Read, Write};

mod declarations;
#[cfg(test)]
mod tests;

pub const MAX_FRAME_BYTES: u32 = IPC_DEFAULT_MAX_FRAME_BYTES;
const VERSION: u16 = 1;

#[derive(Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Profile { name: String },
    Migration { source: String },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Response {
    Profile {
        authority: ProfileAuthority,
    },
    Migration {
        declarations: Vec<declarations::Declaration>,
    },
}

/// Implemented only by the canonical worker after binding the selected project
/// and admitted inputs. No request can select another root or compiler.
pub trait CompilerOperations {
    fn profile(&mut self, name: &str) -> Result<ProfileAuthority, String>;
    fn migration(&mut self, source: &str) -> Result<Vec<MigrationSourceDeclaration>, String>;
}

fn schema(source_identity: &str) -> Result<IpcWireSchema, String> {
    if source_identity.len() != 64
        || !source_identity
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid SQL compiler source identity".into());
    }
    Ok(IpcWireSchema {
        name: format!("sifr.sql.compiler.{source_identity}"),
        version: 1,
        hash: sifr_ipc::fnv1a_128(b"sifr.sql.compiler.v1:profile,migration;strict-json")
            .to_be_bytes(),
        compatible_version_min: 1,
        compatible_version_max: 1,
    })
}

fn read(stream: &mut impl Read) -> Result<IpcEnvelope, String> {
    read_frame(stream, MAX_FRAME_BYTES)
        .map_err(|error| format!("SQL compiler frame read failed: {error:?}"))?
        .ok_or_else(|| "SQL compiler channel closed".into())
}

fn write(stream: &mut impl Write, message: &IpcEnvelope) -> Result<(), String> {
    write_frame(stream, message, MAX_FRAME_BYTES)
        .map_err(|error| format!("SQL compiler frame write failed: {error:?}"))
}

fn decode<T: serde::de::DeserializeOwned>(payload: &[u8]) -> Result<T, String> {
    serde_json::from_slice(payload)
        .map_err(|error| format!("invalid SQL compiler payload: {error}"))
}

fn encode(value: &impl Serialize) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value)
        .map_err(|error| format!("cannot encode SQL compiler payload: {error}"))
}

/// One synchronous request at a time. Any error poisons the session; the launcher
/// must close the channel and reap the owned worker. The stream must enforce the
/// launcher's operation deadline and cancellation policy.
pub struct CompilerClient<S> {
    stream: S,
    next_request: u64,
    failed: bool,
}

impl<S: Read + Write> CompilerClient<S> {
    pub(crate) fn transport_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    pub fn connect(mut stream: S, source_identity: &str) -> Result<Self, String> {
        let expected = schema(source_identity)?;
        write(
            &mut stream,
            &IpcEnvelope::Hello {
                protocol_min: VERSION,
                protocol_max: VERSION,
                schema: expected.clone(),
                max_frame_bytes: MAX_FRAME_BYTES,
            },
        )?;
        match read(&mut stream)? {
            IpcEnvelope::Ready {
                protocol_version: VERSION,
                schema,
                max_frame_bytes: MAX_FRAME_BYTES,
            } if schema == expected => Ok(Self {
                stream,
                next_request: 1,
                failed: false,
            }),
            _ => Err("SQL compiler protocol or source identity mismatch".into()),
        }
    }

    pub fn profile(&mut self, name: &str) -> Result<ProfileAuthority, String> {
        let result = self
            .request(Request::Profile { name: name.into() })
            .and_then(|response| {
                let Response::Profile { authority } = response else {
                    return Err("SQL compiler returned the wrong operation".into());
                };
                let rebuilt = build_profile_authority(authority.profile.clone())
                    .map_err(|error| error.to_string())?;
                if rebuilt != authority || authority.profile.name != name {
                    return Err("SQL compiler profile identity is inconsistent".into());
                }
                Ok(authority)
            });
        self.failed |= result.is_err();
        result
    }

    pub fn migration(&mut self, source: &str) -> Result<Vec<MigrationSourceDeclaration>, String> {
        let result = self
            .request(Request::Migration {
                source: source.into(),
            })
            .and_then(|response| {
                let Response::Migration { declarations } = response else {
                    return Err("SQL compiler returned the wrong operation".into());
                };
                declarations.into_iter().map(TryInto::try_into).collect()
            });
        self.failed |= result.is_err();
        result
    }

    fn request(&mut self, request: Request) -> Result<Response, String> {
        if self.failed {
            return Err("SQL compiler session already failed".into());
        }
        self.failed = true;
        let request_id = self.next_request;
        self.next_request = request_id
            .checked_add(1)
            .ok_or("SQL compiler request ID exhausted")?;
        write(
            &mut self.stream,
            &IpcEnvelope::Run {
                request_id,
                payload: encode(&request)?,
            },
        )?;
        match read(&mut self.stream)? {
            IpcEnvelope::Completed {
                request_id: actual,
                payload,
            } if actual == request_id => {
                let response = decode(&payload)?;
                self.failed = false;
                Ok(response)
            }
            IpcEnvelope::Failed {
                request_id: actual,
                error,
            } if actual == request_id => String::from_utf8(error)
                .map_or_else(|_| Err("SQL compiler diagnostic is not UTF-8".into()), Err),
            _ => Err("SQL compiler response has an invalid state or request ID".into()),
        }
    }

    /// Acknowledgement is required; dropping the client alone is not success.
    pub fn finish(mut self) -> Result<(), String> {
        if self.failed {
            return Err("SQL compiler session already failed".into());
        }
        write(
            &mut self.stream,
            &IpcEnvelope::Shutdown {
                mode: IpcShutdownMode::Drain,
            },
        )?;
        if read(&mut self.stream)?
            != (IpcEnvelope::Terminating {
                reason: IpcTerminationReason::Shutdown,
            })
        {
            return Err("SQL compiler shutdown was not acknowledged".into());
        }
        Ok(())
    }
}

/// Runs the wire state machine, not the compiler or admission policy. The process
/// owner must bound execution and kill descendants on EOF, failure or cancellation.
pub fn serve(
    stream: &mut (impl Read + Write),
    source_identity: &str,
    operations: &mut impl CompilerOperations,
) -> Result<(), String> {
    let expected = schema(source_identity)?;
    match read(stream)? {
        IpcEnvelope::Hello {
            protocol_min: VERSION,
            protocol_max: VERSION,
            schema,
            max_frame_bytes: MAX_FRAME_BYTES,
        } if schema == expected => {}
        _ => return Err("SQL compiler protocol or source identity mismatch".into()),
    }
    write(
        stream,
        &IpcEnvelope::Ready {
            protocol_version: VERSION,
            schema: expected,
            max_frame_bytes: MAX_FRAME_BYTES,
        },
    )?;
    let mut next_request = 1u64;
    loop {
        match read(stream)? {
            IpcEnvelope::Shutdown {
                mode: IpcShutdownMode::Drain,
            } => {
                return write(
                    stream,
                    &IpcEnvelope::Terminating {
                        reason: IpcTerminationReason::Shutdown,
                    },
                );
            }
            IpcEnvelope::Run {
                request_id,
                payload,
            } if request_id == next_request => {
                next_request = next_request
                    .checked_add(1)
                    .ok_or("SQL compiler request ID exhausted")?;
                let request: Request = decode(&payload)?;
                let response = match request {
                    Request::Profile { name } => operations
                        .profile(&name)
                        .map(|authority| Response::Profile { authority }),
                    Request::Migration { source } => {
                        operations
                            .migration(&source)
                            .map(|values| Response::Migration {
                                declarations: values.iter().map(Into::into).collect(),
                            })
                    }
                };
                match response {
                    Ok(response) => write(
                        stream,
                        &IpcEnvelope::Completed {
                            request_id,
                            payload: encode(&response)?,
                        },
                    )?,
                    Err(error) => {
                        write(
                            stream,
                            &IpcEnvelope::Failed {
                                request_id,
                                error: error.into_bytes(),
                            },
                        )?;
                        return Err("SQL compiler operation failed".into());
                    }
                }
            }
            _ => return Err("SQL compiler request has an invalid state or request ID".into()),
        }
    }
}
