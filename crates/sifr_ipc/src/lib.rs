//! Shared Sifr IPC protocol, schema, transport, and request tracking.
//!
//! This crate owns host-independent IPC wire types and helpers used by compiler
//! lowering and runtime-facing verification fixtures. It does not own public
//! stdlib behavior.

mod ipc_connection;
mod ipc_frame;
mod ipc_payload;
mod ipc_request_tracker;
mod ipc_schema;
mod ipc_transport;

pub use ipc_connection::{
    IpcConnectionConfig, IpcConnectionError, IpcConnectionPhase, IpcConnectionState,
    IpcHandshakeDecision, negotiate_protocol_version, schema_ranges_overlap, schemas_match_exact,
};
pub use ipc_frame::{
    IPC_DEFAULT_MAX_FRAME_BYTES, IPC_LENGTH_PREFIX_BYTES, IpcEnvelope, IpcFrameError,
    IpcMalformedKind, IpcRejectReason, IpcShutdownMode, IpcTerminationReason, IpcWireFrameKind,
    IpcWireSchema, IpcWorkerState, decode_frame, encode_frame,
};
pub use ipc_payload::{IpcPayloadEligibilityError, validate_ipc_payload_type};
pub use ipc_request_tracker::{IpcRequestTracker, IpcRequestTrackerError, IpcRequestTrackerState};
pub use ipc_schema::{
    IpcSchemaDescriptor, IpcSchemaField, IpcSchemaType, IpcSchemaVariant,
    canonical_schema_descriptor, fnv1a_128, schema_hash_hex_v1, schema_hash_v1,
};
pub use ipc_transport::{IpcTransportError, read_frame, write_frame};
