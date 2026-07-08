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
    negotiate_protocol_version, schema_ranges_overlap, schemas_match_exact, IpcConnectionConfig,
    IpcConnectionError, IpcConnectionPhase, IpcConnectionState, IpcHandshakeDecision,
};
pub use ipc_frame::{
    decode_frame, encode_frame, IpcEnvelope, IpcFrameError, IpcMalformedKind, IpcRejectReason,
    IpcShutdownMode, IpcTerminationReason, IpcWireFrameKind, IpcWireSchema, IpcWorkerState,
    IPC_DEFAULT_MAX_FRAME_BYTES, IPC_LENGTH_PREFIX_BYTES,
};
pub use ipc_payload::{validate_ipc_payload_type, IpcPayloadEligibilityError};
pub use ipc_request_tracker::{IpcRequestTracker, IpcRequestTrackerError, IpcRequestTrackerState};
pub use ipc_schema::{
    canonical_schema_descriptor, fnv1a_128, schema_hash_hex_v1, schema_hash_v1,
    IpcSchemaDescriptor, IpcSchemaField, IpcSchemaType, IpcSchemaVariant,
};
pub use ipc_transport::{read_frame, write_frame, IpcTransportError};
