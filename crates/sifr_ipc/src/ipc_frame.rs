use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const IPC_LENGTH_PREFIX_BYTES: usize = 4;
pub const IPC_DEFAULT_MAX_FRAME_BYTES: u32 = 16 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IpcWireSchema {
    pub name: String,
    pub version: u32,
    pub hash: [u8; 16],
    pub compatible_version_min: u32,
    pub compatible_version_max: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IpcWireFrameKind {
    Hello,
    Ready,
    Reject,
    Run,
    Started,
    Completed,
    Failed,
    Cancel,
    Shutdown,
    Terminating,
    Heartbeat,
    WorkerStatus,
    MalformedFrame,
    UnsupportedVersion,
    UnsupportedSchema,
    UnsupportedPayload,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IpcEnvelope {
    Hello {
        protocol_min: u16,
        protocol_max: u16,
        schema: IpcWireSchema,
        max_frame_bytes: u32,
    },
    Ready {
        protocol_version: u16,
        schema: IpcWireSchema,
        max_frame_bytes: u32,
    },
    Reject {
        reason: IpcRejectReason,
        detail_code: String,
    },
    Run {
        request_id: u64,
        payload: Vec<u8>,
    },
    Started {
        request_id: u64,
    },
    Completed {
        request_id: u64,
        payload: Vec<u8>,
    },
    Failed {
        request_id: u64,
        error: Vec<u8>,
    },
    Cancel {
        request_id: u64,
    },
    Shutdown {
        mode: IpcShutdownMode,
    },
    Terminating {
        reason: IpcTerminationReason,
    },
    Heartbeat {
        sequence: u64,
    },
    WorkerStatus {
        state: IpcWorkerState,
        in_flight: u32,
    },
    MalformedFrame {
        kind: IpcMalformedKind,
        detail_code: String,
    },
    UnsupportedVersion {
        protocol_min: u16,
        protocol_max: u16,
    },
    UnsupportedSchema {
        schema_id: IpcWireSchema,
    },
    UnsupportedPayload {
        type_name: String,
    },
}

impl IpcEnvelope {
    #[must_use]
    pub const fn kind(&self) -> IpcWireFrameKind {
        match self {
            Self::Hello { .. } => IpcWireFrameKind::Hello,
            Self::Ready { .. } => IpcWireFrameKind::Ready,
            Self::Reject { .. } => IpcWireFrameKind::Reject,
            Self::Run { .. } => IpcWireFrameKind::Run,
            Self::Started { .. } => IpcWireFrameKind::Started,
            Self::Completed { .. } => IpcWireFrameKind::Completed,
            Self::Failed { .. } => IpcWireFrameKind::Failed,
            Self::Cancel { .. } => IpcWireFrameKind::Cancel,
            Self::Shutdown { .. } => IpcWireFrameKind::Shutdown,
            Self::Terminating { .. } => IpcWireFrameKind::Terminating,
            Self::Heartbeat { .. } => IpcWireFrameKind::Heartbeat,
            Self::WorkerStatus { .. } => IpcWireFrameKind::WorkerStatus,
            Self::MalformedFrame { .. } => IpcWireFrameKind::MalformedFrame,
            Self::UnsupportedVersion { .. } => IpcWireFrameKind::UnsupportedVersion,
            Self::UnsupportedSchema { .. } => IpcWireFrameKind::UnsupportedSchema,
            Self::UnsupportedPayload { .. } => IpcWireFrameKind::UnsupportedPayload,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IpcRejectReason {
    UnsupportedVersion,
    UnsupportedSchema,
    MalformedFrame,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IpcShutdownMode {
    Drain,
    CancelInFlight,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IpcTerminationReason {
    Shutdown,
    Cancelled,
    ProtocolError,
    TransportClosed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IpcWorkerState {
    Starting,
    Ready,
    Busy,
    Draining,
    Terminating,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IpcMalformedKind {
    Truncated,
    Oversize,
    Decode,
    State,
    RequestId,
    DuplicateRequestId,
    TrailingBytes,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpcFrameError {
    Encode,
    Decode,
    LengthPrefixTruncated {
        received: usize,
    },
    PayloadTruncated {
        expected: u32,
        received: usize,
    },
    FrameTooLarge {
        frame_len: u32,
        max_frame_bytes: u32,
    },
    LengthUnsupported {
        frame_len: usize,
    },
    TrailingBytes {
        frame_len: u32,
        trailing: usize,
    },
}

impl Display for IpcFrameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Encode => formatter.write_str("failed to encode IPC frame"),
            Self::Decode => formatter.write_str("failed to decode IPC frame"),
            Self::LengthPrefixTruncated { received } => {
                write!(
                    formatter,
                    "truncated IPC frame length prefix: received {received} bytes"
                )
            }
            Self::PayloadTruncated { expected, received } => {
                write!(
                    formatter,
                    "truncated IPC frame payload: expected {expected} bytes, received {received}"
                )
            }
            Self::FrameTooLarge {
                frame_len,
                max_frame_bytes,
            } => {
                write!(
                    formatter,
                    "IPC frame payload length {frame_len} exceeds maximum {max_frame_bytes}"
                )
            }
            Self::LengthUnsupported { frame_len } => {
                write!(
                    formatter,
                    "IPC frame payload length {frame_len} is unsupported"
                )
            }
            Self::TrailingBytes {
                frame_len,
                trailing,
            } => {
                write!(
                    formatter,
                    "IPC frame payload length {frame_len} left {trailing} trailing bytes"
                )
            }
        }
    }
}

impl std::error::Error for IpcFrameError {}

pub fn encode_frame(
    envelope: &IpcEnvelope,
    max_frame_bytes: u32,
) -> Result<Vec<u8>, IpcFrameError> {
    let payload = postcard::to_stdvec(envelope).map_err(|_| IpcFrameError::Encode)?;
    let payload_len =
        u32::try_from(payload.len()).map_err(|_| IpcFrameError::LengthUnsupported {
            frame_len: payload.len(),
        })?;
    if payload_len > max_frame_bytes {
        return Err(IpcFrameError::FrameTooLarge {
            frame_len: payload_len,
            max_frame_bytes,
        });
    }

    let mut encoded = Vec::with_capacity(IPC_LENGTH_PREFIX_BYTES + payload.len());
    encoded.extend_from_slice(&payload_len.to_le_bytes());
    encoded.extend_from_slice(&payload);
    Ok(encoded)
}

pub fn decode_frame(bytes: &[u8], max_frame_bytes: u32) -> Result<IpcEnvelope, IpcFrameError> {
    let Some(prefix) = bytes.get(..IPC_LENGTH_PREFIX_BYTES) else {
        return Err(IpcFrameError::LengthPrefixTruncated {
            received: bytes.len(),
        });
    };
    let frame_len = u32::from_le_bytes([prefix[0], prefix[1], prefix[2], prefix[3]]);
    if frame_len > max_frame_bytes {
        return Err(IpcFrameError::FrameTooLarge {
            frame_len,
            max_frame_bytes,
        });
    }

    let payload_len = usize::try_from(frame_len).map_err(|_| IpcFrameError::LengthUnsupported {
        frame_len: usize::MAX,
    })?;
    let payload_start = IPC_LENGTH_PREFIX_BYTES;
    let payload_end =
        payload_start
            .checked_add(payload_len)
            .ok_or(IpcFrameError::LengthUnsupported {
                frame_len: payload_len,
            })?;
    if bytes.len() < payload_end {
        return Err(IpcFrameError::PayloadTruncated {
            expected: frame_len,
            received: bytes.len().saturating_sub(payload_start),
        });
    }
    if bytes.len() > payload_end {
        return Err(IpcFrameError::TrailingBytes {
            frame_len,
            trailing: bytes.len() - payload_end,
        });
    }

    postcard::from_bytes(&bytes[payload_start..payload_end]).map_err(|_| IpcFrameError::Decode)
}

#[cfg(test)]
mod tests {
    use super::{
        decode_frame, encode_frame, IpcEnvelope, IpcFrameError, IpcMalformedKind, IpcRejectReason,
        IpcShutdownMode, IpcTerminationReason, IpcWireFrameKind, IpcWireSchema, IpcWorkerState,
        IPC_DEFAULT_MAX_FRAME_BYTES,
    };

    fn sample_schema() -> IpcWireSchema {
        IpcWireSchema {
            name: "demo.worker.Echo".to_string(),
            version: 1,
            hash: 0x4733_c89f_b23a_40ec_b5f3_bcda_99fb_34da_u128.to_be_bytes(),
            compatible_version_min: 1,
            compatible_version_max: 1,
        }
    }

    fn sample_hello() -> IpcEnvelope {
        IpcEnvelope::Hello {
            protocol_min: 1,
            protocol_max: 1,
            schema: sample_schema(),
            max_frame_bytes: IPC_DEFAULT_MAX_FRAME_BYTES,
        }
    }

    fn encode_sample_hello() -> Vec<u8> {
        let Ok(encoded) = encode_frame(&sample_hello(), IPC_DEFAULT_MAX_FRAME_BYTES) else {
            panic!("sample hello frame should encode");
        };
        encoded
    }

    #[test]
    fn frame_round_trip_preserves_envelope_and_kind() {
        let expected = sample_hello();
        let encoded = encode_sample_hello();
        let decoded = decode_frame(&encoded, IPC_DEFAULT_MAX_FRAME_BYTES);

        assert_eq!(decoded, Ok(expected));
        assert_eq!(sample_hello().kind(), IpcWireFrameKind::Hello);
    }

    #[test]
    fn frame_families_round_trip_through_postcard_payloads() {
        let frames = vec![
            IpcEnvelope::Ready {
                protocol_version: 1,
                schema: sample_schema(),
                max_frame_bytes: 1024,
            },
            IpcEnvelope::Reject {
                reason: IpcRejectReason::UnsupportedSchema,
                detail_code: "schema_mismatch".to_string(),
            },
            IpcEnvelope::Run {
                request_id: 7,
                payload: vec![1, 2, 3],
            },
            IpcEnvelope::Started { request_id: 7 },
            IpcEnvelope::Completed {
                request_id: 7,
                payload: vec![4, 5],
            },
            IpcEnvelope::Failed {
                request_id: 7,
                error: vec![9],
            },
            IpcEnvelope::Cancel { request_id: 7 },
            IpcEnvelope::Shutdown {
                mode: IpcShutdownMode::CancelInFlight,
            },
            IpcEnvelope::Terminating {
                reason: IpcTerminationReason::Shutdown,
            },
            IpcEnvelope::Heartbeat { sequence: 11 },
            IpcEnvelope::WorkerStatus {
                state: IpcWorkerState::Busy,
                in_flight: 2,
            },
            IpcEnvelope::MalformedFrame {
                kind: IpcMalformedKind::Decode,
                detail_code: "decode".to_string(),
            },
            IpcEnvelope::UnsupportedVersion {
                protocol_min: 1,
                protocol_max: 2,
            },
            IpcEnvelope::UnsupportedSchema {
                schema_id: sample_schema(),
            },
            IpcEnvelope::UnsupportedPayload {
                type_name: "sifr.process.Child".to_string(),
            },
        ];

        for frame in frames {
            let encoded = encode_frame(&frame, IPC_DEFAULT_MAX_FRAME_BYTES);
            let Ok(encoded) = encoded else {
                panic!("test frame should encode");
            };
            assert_eq!(
                decode_frame(&encoded, IPC_DEFAULT_MAX_FRAME_BYTES),
                Ok(frame)
            );
        }
    }

    #[test]
    fn encode_rejects_payloads_above_negotiated_max() {
        let result = encode_frame(&sample_hello(), 1);

        assert!(matches!(
            result,
            Err(IpcFrameError::FrameTooLarge {
                frame_len: _,
                max_frame_bytes: 1
            })
        ));
    }

    #[test]
    fn decode_rejects_truncated_length_prefix() {
        assert_eq!(
            decode_frame(&[0, 1], IPC_DEFAULT_MAX_FRAME_BYTES),
            Err(IpcFrameError::LengthPrefixTruncated { received: 2 })
        );
    }

    #[test]
    fn decode_rejects_oversize_before_payload_decode() {
        let mut bytes = (IPC_DEFAULT_MAX_FRAME_BYTES + 1).to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0, 1, 2, 3]);

        assert_eq!(
            decode_frame(&bytes, IPC_DEFAULT_MAX_FRAME_BYTES),
            Err(IpcFrameError::FrameTooLarge {
                frame_len: IPC_DEFAULT_MAX_FRAME_BYTES + 1,
                max_frame_bytes: IPC_DEFAULT_MAX_FRAME_BYTES
            })
        );
    }

    #[test]
    fn decode_rejects_truncated_payload() {
        let mut bytes = 5_u32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[1, 2, 3]);

        assert_eq!(
            decode_frame(&bytes, IPC_DEFAULT_MAX_FRAME_BYTES),
            Err(IpcFrameError::PayloadTruncated {
                expected: 5,
                received: 3
            })
        );
    }

    #[test]
    fn decode_rejects_invalid_postcard_payload() {
        let mut bytes = 2_u32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[255, 255]);

        assert_eq!(
            decode_frame(&bytes, IPC_DEFAULT_MAX_FRAME_BYTES),
            Err(IpcFrameError::Decode)
        );
    }

    #[test]
    fn decode_rejects_trailing_bytes_for_single_frame_helper() {
        let mut bytes = encode_sample_hello();
        bytes.push(0);

        assert!(matches!(
            decode_frame(&bytes, IPC_DEFAULT_MAX_FRAME_BYTES),
            Err(IpcFrameError::TrailingBytes {
                frame_len: _,
                trailing: 1
            })
        ));
    }

    #[test]
    fn frame_errors_do_not_render_payload_bytes() {
        let err = IpcFrameError::Decode;

        assert_eq!(err.to_string(), "failed to decode IPC frame");
    }
}
