use crate::{
    IpcEnvelope, IpcMalformedKind, IpcRejectReason, IpcRequestTracker, IpcRequestTrackerError,
    IpcWireFrameKind, IpcWireSchema, IPC_DEFAULT_MAX_FRAME_BYTES,
};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpcConnectionPhase {
    Initialized,
    HelloSent,
    Ready,
    Draining,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IpcConnectionConfig {
    pub protocol_min: u16,
    pub protocol_max: u16,
    pub schema: IpcWireSchema,
    pub max_frame_bytes: u32,
    pub max_in_flight: u32,
}

impl IpcConnectionConfig {
    #[must_use]
    pub fn new(schema: IpcWireSchema) -> Self {
        Self {
            protocol_min: 1,
            protocol_max: 1,
            schema,
            max_frame_bytes: IPC_DEFAULT_MAX_FRAME_BYTES,
            max_in_flight: 64,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpcHandshakeDecision {
    Ready(IpcEnvelope),
    Reject(IpcEnvelope),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpcConnectionError {
    InvalidProtocolRange {
        protocol_min: u16,
        protocol_max: u16,
    },
    InvalidSchemaRange {
        schema_version_min: u32,
        schema_version_max: u32,
    },
    InvalidMaxFrameBytes,
    InvalidFrameForPhase {
        phase: IpcConnectionPhase,
        frame_kind: IpcWireFrameKind,
    },
    UnsupportedVersion {
        local_min: u16,
        local_max: u16,
        remote_min: u16,
        remote_max: u16,
    },
    UnsupportedSchema,
    RemoteRejected {
        reason: IpcRejectReason,
    },
    Request(IpcRequestTrackerError),
}

impl Display for IpcConnectionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProtocolRange {
                protocol_min,
                protocol_max,
            } => write!(
                formatter,
                "invalid IPC protocol range {protocol_min}..{protocol_max}"
            ),
            Self::InvalidSchemaRange {
                schema_version_min,
                schema_version_max,
            } => write!(
                formatter,
                "invalid IPC schema version range {schema_version_min}..{schema_version_max}"
            ),
            Self::InvalidMaxFrameBytes => {
                formatter.write_str("invalid IPC maximum frame byte limit")
            }
            Self::InvalidFrameForPhase { phase, frame_kind } => write!(
                formatter,
                "IPC frame {frame_kind:?} is not valid while connection is {phase:?}"
            ),
            Self::UnsupportedVersion {
                local_min,
                local_max,
                remote_min,
                remote_max,
            } => write!(
                formatter,
                "unsupported IPC protocol range: local {local_min}..{local_max}, remote {remote_min}..{remote_max}"
            ),
            Self::UnsupportedSchema => formatter.write_str("unsupported IPC schema identity"),
            Self::RemoteRejected { reason } => {
                write!(formatter, "IPC peer rejected bootstrap: {reason:?}")
            }
            Self::Request(error) => Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for IpcConnectionError {}

impl From<IpcRequestTrackerError> for IpcConnectionError {
    fn from(error: IpcRequestTrackerError) -> Self {
        Self::Request(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IpcConnectionState {
    config: IpcConnectionConfig,
    phase: IpcConnectionPhase,
    negotiated_protocol_version: Option<u16>,
    negotiated_max_frame_bytes: u32,
    requests: IpcRequestTracker,
}

impl IpcConnectionState {
    pub fn new(config: IpcConnectionConfig) -> Result<Self, IpcConnectionError> {
        validate_config(&config)?;
        let max_frame_bytes = config.max_frame_bytes;
        let max_in_flight = config.max_in_flight;
        Ok(Self {
            config,
            phase: IpcConnectionPhase::Initialized,
            negotiated_protocol_version: None,
            negotiated_max_frame_bytes: max_frame_bytes,
            requests: IpcRequestTracker::new(max_in_flight),
        })
    }

    #[must_use]
    pub const fn phase(&self) -> IpcConnectionPhase {
        self.phase
    }

    #[must_use]
    pub const fn negotiated_protocol_version(&self) -> Option<u16> {
        self.negotiated_protocol_version
    }

    #[must_use]
    pub const fn negotiated_max_frame_bytes(&self) -> u32 {
        self.negotiated_max_frame_bytes
    }

    #[must_use]
    pub fn in_flight_len(&self) -> usize {
        self.requests.in_flight_len()
    }

    #[must_use]
    pub fn is_in_flight(&self, request_id: u64) -> bool {
        self.requests.is_in_flight(request_id)
    }

    pub fn begin_parent_handshake(&mut self) -> Result<IpcEnvelope, IpcConnectionError> {
        self.ensure_phase(IpcConnectionPhase::Initialized, IpcWireFrameKind::Hello)?;
        self.phase = IpcConnectionPhase::HelloSent;
        Ok(IpcEnvelope::Hello {
            protocol_min: self.config.protocol_min,
            protocol_max: self.config.protocol_max,
            schema: self.config.schema.clone(),
            max_frame_bytes: self.config.max_frame_bytes,
        })
    }

    pub fn accept_worker_bootstrap(
        &mut self,
        frame: &IpcEnvelope,
    ) -> Result<(), IpcConnectionError> {
        if self.phase != IpcConnectionPhase::HelloSent {
            return Err(IpcConnectionError::InvalidFrameForPhase {
                phase: self.phase,
                frame_kind: frame.kind(),
            });
        }
        match frame {
            IpcEnvelope::Ready {
                protocol_version,
                schema,
                max_frame_bytes,
            } => self.accept_ready(*protocol_version, schema, *max_frame_bytes),
            IpcEnvelope::Reject { reason, .. } => {
                self.close();
                Err(IpcConnectionError::RemoteRejected { reason: *reason })
            }
            _ => Err(IpcConnectionError::InvalidFrameForPhase {
                phase: self.phase,
                frame_kind: frame.kind(),
            }),
        }
    }

    pub fn accept_parent_hello(
        &mut self,
        frame: &IpcEnvelope,
    ) -> Result<IpcHandshakeDecision, IpcConnectionError> {
        if self.phase != IpcConnectionPhase::Initialized {
            return Err(IpcConnectionError::InvalidFrameForPhase {
                phase: self.phase,
                frame_kind: frame.kind(),
            });
        }
        let IpcEnvelope::Hello {
            protocol_min,
            protocol_max,
            schema,
            max_frame_bytes,
        } = frame
        else {
            return Err(IpcConnectionError::InvalidFrameForPhase {
                phase: self.phase,
                frame_kind: frame.kind(),
            });
        };

        let Some(protocol_version) = negotiate_protocol_version(
            self.config.protocol_min,
            self.config.protocol_max,
            *protocol_min,
            *protocol_max,
        ) else {
            self.close();
            return Ok(IpcHandshakeDecision::Reject(IpcEnvelope::Reject {
                reason: IpcRejectReason::UnsupportedVersion,
                detail_code: "unsupported_version".to_string(),
            }));
        };

        if !schemas_match_exact(&self.config.schema, schema) {
            self.close();
            return Ok(IpcHandshakeDecision::Reject(IpcEnvelope::Reject {
                reason: IpcRejectReason::UnsupportedSchema,
                detail_code: "unsupported_schema".to_string(),
            }));
        }

        let negotiated_max_frame_bytes =
            negotiate_max_frame_bytes(self.config.max_frame_bytes, *max_frame_bytes)?;
        self.mark_ready(protocol_version, negotiated_max_frame_bytes);
        Ok(IpcHandshakeDecision::Ready(IpcEnvelope::Ready {
            protocol_version,
            schema: self.config.schema.clone(),
            max_frame_bytes: negotiated_max_frame_bytes,
        }))
    }

    pub fn apply_established_frame(
        &mut self,
        frame: &IpcEnvelope,
    ) -> Result<(), IpcConnectionError> {
        if !matches!(
            self.phase,
            IpcConnectionPhase::Ready | IpcConnectionPhase::Draining
        ) {
            return Err(IpcConnectionError::InvalidFrameForPhase {
                phase: self.phase,
                frame_kind: frame.kind(),
            });
        }
        match frame {
            IpcEnvelope::Hello { .. } | IpcEnvelope::Ready { .. } | IpcEnvelope::Reject { .. } => {
                Err(IpcConnectionError::InvalidFrameForPhase {
                    phase: self.phase,
                    frame_kind: frame.kind(),
                })
            }
            IpcEnvelope::Shutdown { mode } => {
                self.requests.begin_shutdown(*mode);
                self.phase = IpcConnectionPhase::Draining;
                Ok(())
            }
            IpcEnvelope::Terminating { .. } => {
                self.close();
                Ok(())
            }
            IpcEnvelope::MalformedFrame { .. }
            | IpcEnvelope::UnsupportedVersion { .. }
            | IpcEnvelope::UnsupportedSchema { .. }
            | IpcEnvelope::UnsupportedPayload { .. } => {
                self.close();
                Ok(())
            }
            _ => self.requests.apply_frame(frame).map_err(Into::into),
        }
    }

    #[must_use]
    pub fn protocol_error_frame(kind: IpcMalformedKind, detail_code: &str) -> IpcEnvelope {
        IpcEnvelope::MalformedFrame {
            kind,
            detail_code: detail_code.to_string(),
        }
    }

    fn accept_ready(
        &mut self,
        protocol_version: u16,
        schema: &IpcWireSchema,
        max_frame_bytes: u32,
    ) -> Result<(), IpcConnectionError> {
        if !version_in_range(
            protocol_version,
            self.config.protocol_min,
            self.config.protocol_max,
        ) {
            self.close();
            return Err(IpcConnectionError::UnsupportedVersion {
                local_min: self.config.protocol_min,
                local_max: self.config.protocol_max,
                remote_min: protocol_version,
                remote_max: protocol_version,
            });
        }
        if !schemas_match_exact(&self.config.schema, schema) {
            self.close();
            return Err(IpcConnectionError::UnsupportedSchema);
        }
        let negotiated_max_frame_bytes =
            negotiate_max_frame_bytes(self.config.max_frame_bytes, max_frame_bytes)?;
        self.mark_ready(protocol_version, negotiated_max_frame_bytes);
        Ok(())
    }

    fn mark_ready(&mut self, protocol_version: u16, max_frame_bytes: u32) {
        self.phase = IpcConnectionPhase::Ready;
        self.negotiated_protocol_version = Some(protocol_version);
        self.negotiated_max_frame_bytes = max_frame_bytes;
    }

    fn close(&mut self) {
        self.phase = IpcConnectionPhase::Closed;
        self.requests.close();
    }

    fn ensure_phase(
        &self,
        expected: IpcConnectionPhase,
        frame_kind: IpcWireFrameKind,
    ) -> Result<(), IpcConnectionError> {
        if self.phase == expected {
            return Ok(());
        }
        Err(IpcConnectionError::InvalidFrameForPhase {
            phase: self.phase,
            frame_kind,
        })
    }
}

#[must_use]
pub fn negotiate_protocol_version(
    local_min: u16,
    local_max: u16,
    remote_min: u16,
    remote_max: u16,
) -> Option<u16> {
    let min_supported = local_min.max(remote_min);
    let max_supported = local_max.min(remote_max);
    if min_supported <= max_supported {
        Some(max_supported)
    } else {
        None
    }
}

#[must_use]
pub fn schemas_match_exact(local: &IpcWireSchema, remote: &IpcWireSchema) -> bool {
    local.name == remote.name
        && local.hash == remote.hash
        && local.version == remote.version
        && schema_ranges_overlap(local, remote)
}

#[must_use]
pub fn schema_ranges_overlap(local: &IpcWireSchema, remote: &IpcWireSchema) -> bool {
    let min_supported = local
        .compatible_version_min
        .max(remote.compatible_version_min);
    let max_supported = local
        .compatible_version_max
        .min(remote.compatible_version_max);
    min_supported <= max_supported
}

fn validate_config(config: &IpcConnectionConfig) -> Result<(), IpcConnectionError> {
    if config.protocol_min > config.protocol_max {
        return Err(IpcConnectionError::InvalidProtocolRange {
            protocol_min: config.protocol_min,
            protocol_max: config.protocol_max,
        });
    }
    if config.schema.compatible_version_min > config.schema.compatible_version_max {
        return Err(IpcConnectionError::InvalidSchemaRange {
            schema_version_min: config.schema.compatible_version_min,
            schema_version_max: config.schema.compatible_version_max,
        });
    }
    if config.max_frame_bytes == 0 {
        return Err(IpcConnectionError::InvalidMaxFrameBytes);
    }
    Ok(())
}

fn negotiate_max_frame_bytes(
    local_max_frame_bytes: u32,
    remote_max_frame_bytes: u32,
) -> Result<u32, IpcConnectionError> {
    if remote_max_frame_bytes == 0 {
        return Err(IpcConnectionError::InvalidMaxFrameBytes);
    }
    Ok(local_max_frame_bytes.min(remote_max_frame_bytes))
}

fn version_in_range(version: u16, min: u16, max: u16) -> bool {
    min <= version && version <= max
}

#[cfg(test)]
mod tests {
    use super::{
        negotiate_protocol_version, schema_ranges_overlap, schemas_match_exact,
        IpcConnectionConfig, IpcConnectionError, IpcConnectionPhase, IpcConnectionState,
        IpcHandshakeDecision,
    };
    use crate::{
        IpcEnvelope, IpcMalformedKind, IpcRejectReason, IpcShutdownMode, IpcTerminationReason,
        IpcWireSchema,
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

    fn configured_state() -> IpcConnectionState {
        IpcConnectionState::new(IpcConnectionConfig::new(sample_schema()))
            .expect("sample config is valid")
    }

    fn ready_state() -> IpcConnectionState {
        let mut state = configured_state();
        state
            .begin_parent_handshake()
            .expect("initialized state can emit hello");
        state
            .accept_worker_bootstrap(&IpcEnvelope::Ready {
                protocol_version: 1,
                schema: sample_schema(),
                max_frame_bytes: 2048,
            })
            .expect("sample ready frame is compatible");
        state
    }

    #[test]
    fn protocol_negotiation_selects_highest_overlap() {
        assert_eq!(negotiate_protocol_version(1, 4, 2, 3), Some(3));
        assert_eq!(negotiate_protocol_version(3, 4, 1, 2), None);
    }

    #[test]
    fn schema_matching_requires_identity_and_overlapping_range() {
        let schema = sample_schema();
        let mut incompatible_range = schema.clone();
        incompatible_range.compatible_version_min = 2;
        incompatible_range.compatible_version_max = 3;
        let mut changed_hash = schema.clone();
        changed_hash.hash = [7; 16];

        assert!(schemas_match_exact(&schema, &schema));
        assert!(!schemas_match_exact(&schema, &incompatible_range));
        assert!(!schemas_match_exact(&schema, &changed_hash));
        assert!(!schema_ranges_overlap(&schema, &incompatible_range));
    }

    #[test]
    fn parent_handshake_emits_hello_and_waits_for_bootstrap() {
        let mut state = configured_state();

        assert_eq!(
            state.begin_parent_handshake(),
            Ok(IpcEnvelope::Hello {
                protocol_min: 1,
                protocol_max: 1,
                schema: sample_schema(),
                max_frame_bytes: 16 * 1024 * 1024,
            })
        );
        assert_eq!(state.phase(), IpcConnectionPhase::HelloSent);
    }

    #[test]
    fn worker_accepts_compatible_hello_and_returns_ready() {
        let mut worker = configured_state();
        let hello = IpcEnvelope::Hello {
            protocol_min: 1,
            protocol_max: 2,
            schema: sample_schema(),
            max_frame_bytes: 4096,
        };

        assert_eq!(
            worker.accept_parent_hello(&hello),
            Ok(IpcHandshakeDecision::Ready(IpcEnvelope::Ready {
                protocol_version: 1,
                schema: sample_schema(),
                max_frame_bytes: 4096,
            }))
        );
        assert_eq!(worker.phase(), IpcConnectionPhase::Ready);
        assert_eq!(worker.negotiated_protocol_version(), Some(1));
        assert_eq!(worker.negotiated_max_frame_bytes(), 4096);
    }

    #[test]
    fn worker_rejects_unsupported_protocol_without_panicking() {
        let mut worker = configured_state();
        let hello = IpcEnvelope::Hello {
            protocol_min: 2,
            protocol_max: 3,
            schema: sample_schema(),
            max_frame_bytes: 4096,
        };

        assert_eq!(
            worker.accept_parent_hello(&hello),
            Ok(IpcHandshakeDecision::Reject(IpcEnvelope::Reject {
                reason: IpcRejectReason::UnsupportedVersion,
                detail_code: "unsupported_version".to_string(),
            }))
        );
        assert_eq!(worker.phase(), IpcConnectionPhase::Closed);
    }

    #[test]
    fn worker_rejects_unknown_schema_without_payload_details() {
        let mut worker = configured_state();
        let mut schema = sample_schema();
        schema.name = "demo.worker.Other".to_string();
        let hello = IpcEnvelope::Hello {
            protocol_min: 1,
            protocol_max: 1,
            schema,
            max_frame_bytes: 4096,
        };

        assert_eq!(
            worker.accept_parent_hello(&hello),
            Ok(IpcHandshakeDecision::Reject(IpcEnvelope::Reject {
                reason: IpcRejectReason::UnsupportedSchema,
                detail_code: "unsupported_schema".to_string(),
            }))
        );
        assert_eq!(worker.phase(), IpcConnectionPhase::Closed);
    }

    #[test]
    fn parent_accepts_worker_ready_after_hello() {
        let mut parent = configured_state();
        assert!(parent.begin_parent_handshake().is_ok());

        assert_eq!(
            parent.accept_worker_bootstrap(&IpcEnvelope::Ready {
                protocol_version: 1,
                schema: sample_schema(),
                max_frame_bytes: 2048,
            }),
            Ok(())
        );
        assert_eq!(parent.phase(), IpcConnectionPhase::Ready);
        assert_eq!(parent.negotiated_max_frame_bytes(), 2048);
    }

    #[test]
    fn parent_rejects_forged_ready_schema() {
        let mut parent = configured_state();
        let mut schema = sample_schema();
        schema.hash = [9; 16];
        assert!(parent.begin_parent_handshake().is_ok());

        assert_eq!(
            parent.accept_worker_bootstrap(&IpcEnvelope::Ready {
                protocol_version: 1,
                schema,
                max_frame_bytes: 2048,
            }),
            Err(IpcConnectionError::UnsupportedSchema)
        );
        assert_eq!(parent.phase(), IpcConnectionPhase::Closed);
    }

    #[test]
    fn frames_before_ready_are_state_errors() {
        let mut state = configured_state();

        assert!(matches!(
            state.apply_established_frame(&IpcEnvelope::Run {
                request_id: 1,
                payload: vec![1],
            }),
            Err(IpcConnectionError::InvalidFrameForPhase { .. })
        ));
    }

    #[test]
    fn established_run_and_completion_update_request_tracking() {
        let mut state = ready_state();

        assert_eq!(
            state.apply_established_frame(&IpcEnvelope::Run {
                request_id: 7,
                payload: vec![1, 2, 3],
            }),
            Ok(())
        );
        assert!(state.is_in_flight(7));
        assert_eq!(
            state.apply_established_frame(&IpcEnvelope::Completed {
                request_id: 7,
                payload: vec![4],
            }),
            Ok(())
        );
        assert_eq!(state.in_flight_len(), 0);
    }

    #[test]
    fn duplicate_request_id_is_reported_through_connection_state() {
        let mut state = ready_state();
        let run = IpcEnvelope::Run {
            request_id: 7,
            payload: vec![1],
        };

        assert_eq!(state.apply_established_frame(&run), Ok(()));
        assert!(matches!(
            state.apply_established_frame(&run),
            Err(IpcConnectionError::Request(_))
        ));
    }

    #[test]
    fn shutdown_drains_and_rejects_new_runs() {
        let mut state = ready_state();
        assert_eq!(
            state.apply_established_frame(&IpcEnvelope::Shutdown {
                mode: IpcShutdownMode::Drain,
            }),
            Ok(())
        );

        assert_eq!(state.phase(), IpcConnectionPhase::Draining);
        assert!(matches!(
            state.apply_established_frame(&IpcEnvelope::Run {
                request_id: 9,
                payload: vec![1],
            }),
            Err(IpcConnectionError::Request(_))
        ));
    }

    #[test]
    fn terminating_frame_closes_connection() {
        let mut state = ready_state();

        assert_eq!(
            state.apply_established_frame(&IpcEnvelope::Terminating {
                reason: IpcTerminationReason::Shutdown,
            }),
            Ok(())
        );
        assert_eq!(state.phase(), IpcConnectionPhase::Closed);
    }

    #[test]
    fn protocol_error_frame_helper_redacts_details_to_codes() {
        assert_eq!(
            IpcConnectionState::protocol_error_frame(IpcMalformedKind::State, "state"),
            IpcEnvelope::MalformedFrame {
                kind: IpcMalformedKind::State,
                detail_code: "state".to_string(),
            }
        );
    }
}
