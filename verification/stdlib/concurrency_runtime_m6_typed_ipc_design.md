# Concurrency Runtime M6 Typed IPC Design

Milestone: `milestone_concurrency_runtime_6`

Status: In progress. The design gate is approved, dependency metadata is wired, and `ipc_value_model_basic` validates the host-independent schema/frame/backpressure value model. Runtime frame encoding, process-pipe transport, malformed-frame behavior, cancellation, close, backpressure, and payload eligibility diagnostics remain M6 implementation work.

## Scope

M6 defines typed IPC as production substrate for future Sifr-native supervised process workers. It is not a public CPython multiprocessing adapter, does not replace same-process channels, and does not replace raw process pipes for byte/text subprocess workflows.

The accepted substrate is:

- generated typed IPC schemas,
- a versioned frame protocol over an accepted process transport,
- deterministic schema identity and compatibility negotiation,
- strict `IpcSerializable` payload eligibility,
- typed backpressure, close, cancellation, and malformed-frame evidence,
- stable diagnostics for unsupported CPython-shaped process-pool and multiprocessing APIs.

This design does not ship a public process-worker pool. A future worker API remains `deferred-to-phase-X` and must be built on `sifr.process` plus this `sifr.ipc` substrate.

## Current Evidence

| Surface | Evidence | Notes |
| --- | --- | --- |
| `sifr.ipc.SchemaId` / `schema_id(...)` | `ipc_value_model_basic` | Host-independent value model for generated schema identity records. This does not yet compute compiler-generated schema hashes. |
| `sifr.ipc.ProtocolVersion` / `protocol_version(...)` | `ipc_value_model_basic` | Host-independent value model for negotiated protocol version bounds. |
| `sifr.ipc.FrameKind` and frame-family constants | `ipc_value_model_basic` | Covers bootstrap, work, control, health, and protocol-error frame names as values. It does not encode or decode wire frames yet. |
| `sifr.ipc.BackpressurePolicy` / `default_backpressure()` | `ipc_value_model_basic` | Pins the default in-flight request window (`64`) and default max frame bytes (`16777216`) from the design. Runtime backpressure enforcement remains follow-up work. |
| Internal schema descriptor and hash v1 | `cargo test -p sifr_stdlib ipc_schema -- --nocapture` | `sifr_stdlib::ipc_schema` renders canonical schema descriptors and stable FNV-1a-128 schema hashes without adding a new hash dependency. Compiler integration and generated schema extraction remain follow-up work. |
| `ProcessPoolExecutor` and `Process` under `sifr.ipc` | `ipc_process_pool_executor_unsupported`; `ipc_multiprocessing_process_unsupported` | Missing-member diagnostics keep CPython-shaped process-pool and multiprocessing names out of the native IPC module. |

## Transport Boundary

The first accepted transport is M4-owned child process pipes. A Sifr-generated parent process owns the child stdin/stdout pipe pair for IPC frames:

- parent-to-child control and work frames are written to child stdin,
- child-to-parent bootstrap, status, result, and error frames are read from child stdout,
- stderr remains diagnostic output, not IPC payload transport,
- process lifecycle, kill, terminate, timeout, and supervision remain owned by `sifr.process`.

Typed IPC can later use another explicitly accepted transport, but it must reuse the same envelope, schema, cancellation, close, backpressure, and malformed-frame semantics. Arbitrary foreign executables are not typed IPC peers unless they implement the negotiated Sifr IPC protocol; otherwise they remain raw process pipe users.

## Wire Format

Each wire frame is length-delimited followed by a Postcard-encoded envelope:

1. `u32` little-endian payload byte length.
2. Postcard payload bytes for the generated `IpcEnvelope` variant.

The default maximum frame payload is 16 MiB. Generated peers may negotiate a lower limit during bootstrap. A payload length above the negotiated limit is `MalformedFrame(kind="oversize")`; the receiver reports the typed protocol error and closes the connection without panicking.

Postcard is used only for typed IPC payload frames after this design gate. It is not a general Sifr serialization baseline, and `serde_json` / `bincode` remain rejected for IPC payload frames in this phase.

## Schema Identity

Every generated IPC schema has a canonical descriptor:

- protocol schema version,
- fully qualified Sifr module path,
- public schema name,
- request, response, and error type names,
- field names, field order, optionality, and contained type descriptors,
- enum variant names and payload descriptors,
- declared compatible version range.

The compiler emits a stable 128-bit `schema_hash` from the canonical UTF-8 descriptor using Sifr schema-hash v1. The hash is compatibility evidence, not a cryptographic trust boundary. Compiler output must be deterministic across hosts.

Negotiation policy:

- exact hash match proceeds,
- otherwise, compatible schema versions proceed only when both peers declare an overlapping compatible version range,
- unknown schema identity returns `Reject(reason="unsupported_schema")` during bootstrap or `UnsupportedSchema` after a connection exists,
- incompatible schema versions return `Reject(reason="unsupported_version")` during bootstrap or `UnsupportedVersion` after a connection exists.

Until Sifr grows user-facing schema evolution annotations, generated schemas are exact-hash compatible only. The protocol still carries version ranges so the first implementation does not need a wire-format break later.

## Frame Families

Minimum frame families are fixed for M6.

Bootstrap:

- `Hello { protocol_min, protocol_max, schema_id, schema_hash, schema_version_min, schema_version_max, max_frame_bytes }`
- `Ready { protocol_version, schema_id, schema_hash, max_frame_bytes }`
- `Reject { reason, detail_code }`

Work:

- `Run { request_id, payload }`
- `Started { request_id }`
- `Completed { request_id, payload }`
- `Failed { request_id, error }`

Control:

- `Cancel { request_id }`
- `Shutdown { mode }`
- `Terminating { reason }`

Health:

- `Heartbeat { sequence }`
- `WorkerStatus { state, in_flight }`

Protocol errors:

- `MalformedFrame { kind, detail_code }`
- `UnsupportedVersion { protocol_min, protocol_max }`
- `UnsupportedSchema { schema_id }`
- `UnsupportedPayload { type_name }`

`request_id` values are connection-local unsigned integers allocated by the sender. Reuse before a terminal `Completed`, `Failed`, `Cancelled`, or connection close is a `MalformedFrame(kind="duplicate_request_id")`.

## Connection Semantics

The future protocol-level surface is `ipc.Connection[Req, Res, Err]`. It is a linear connection resource and may not be cloned. A connection owns:

- negotiated protocol version,
- schema identity,
- max frame bytes,
- bounded in-flight request window,
- transport handles,
- close state,
- typed outstanding request table.

The protocol-level operations are:

- `run(req: Req) -> Result[Res, IpcError[Err]]`
- `try_run(req: Req) -> Result[RequestHandle[Res, Err], IpcSendError[Req]]`
- `cancel(request_id) -> CancelOutcome`
- `shutdown(mode) -> Result[Terminating, IpcError[Err]]`
- `close() -> Result[Closed, IpcCloseError]`

The first implementation may expose only fixture-oriented internal helpers while the compiler/runtime prove schema generation, encoding, and diagnostics. Public worker-pool APIs remain deferred.

## Backpressure

Typed IPC has bounded in-flight requests. The default window is 64 requests unless a generated schema chooses a smaller value. There is no unbounded queue hidden behind `Connection`.

When the window is full:

- `try_run` returns `IpcSendError::Full(req)` and preserves ownership of the request,
- `run` awaits capacity but remains cancellation-safe,
- cancellation of a waiting `run` does not enqueue a request,
- transport write failure returns typed `Closed` or `Transport` evidence.

Payload encoding occurs after capacity is reserved. If encoding fails due to payload eligibility or schema mismatch, the request is not sent and ownership is not silently lost.

## Cancellation And Close

Cancellation is a typed protocol message, not a local drop shortcut.

- `Cancel { request_id }` is best-effort once `Run` has been sent.
- `Started` may race with `Cancel`.
- `Completed` or `Failed` may race with `Cancel`; terminal work evidence wins.
- If the protocol is still live, parent cancellation sends `Cancel` before process termination escalation.
- If the protocol is no longer live, process supervision returns typed process evidence from M4.

`Shutdown` stops new `Run` frames, allows in-flight requests to finish or be cancelled according to mode, then returns `Terminating`. EOF before `Terminating` is `IpcCloseError::UnexpectedEof`.

Dropping a live `Connection` without `close`, `shutdown`, or scope-owned cancellation is a compile-time diagnostic where the compiler can prove it. Runtime scope cleanup must still close transport handles without panicking.

## Malformed-Frame Handling

Malformed input is always typed evidence:

- invalid length prefix or truncated payload: `MalformedFrame(kind="truncated")`,
- length above limit: `MalformedFrame(kind="oversize")`,
- Postcard decode failure: `MalformedFrame(kind="decode")`,
- frame family not valid in the current state: `MalformedFrame(kind="state")`,
- duplicate or unknown request id: `MalformedFrame(kind="request_id")`,
- payload type not accepted by the schema: `UnsupportedPayload`.

After a malformed frame, the receiver sends a protocol error frame when possible and closes the connection. Generated runtime code must not use data-dependent `unwrap`, `expect`, or `panic!` for malformed peer input.

## Payload Eligibility

`IpcSerializable` is stricter than `Sendable`. A type is eligible only when the full type graph is owned, sendable, has a stable generated IPC schema, and can be encoded without borrowing process-local resources.

Initially accepted payload families:

- `bool`,
- Sifr integer and floating numeric values with the existing numeric boundary policy,
- `str`,
- `bytes`,
- `Option[T]` when `T: IpcSerializable`,
- `Result[T, E]` when both sides are `IpcSerializable`,
- fixed-shape tuples when every element is `IpcSerializable`,
- `list[T]` and `dict[str, T]` when contained values are `IpcSerializable`,
- generated records and enums with stable schemas.

Rejected payload families:

- file handles and path-dependent open resources,
- subprocess `Child`, `AsyncChild`, `ProcessHandle`, pipe readers, and pipe writers,
- task handles, join sets, cancellation scopes, and task groups,
- locks, semaphores, barriers, once cells, guards, and channel endpoints,
- closures, functions, bound methods, generators, iterators, and dynamic objects,
- borrowed references that could outlive their lexical scope,
- raw pointers, host handles, and foreign objects,
- arbitrary pickle-like object graphs.

Unsupported payloads are compile-time diagnostics where the compiler sees the concrete type. Runtime `UnsupportedPayload` is reserved for foreign peers, stale generated peers, or erased boundaries that cannot be proven statically.

## CPython-Shaped API Classification

M6 keeps CPython multiprocessing and process-pool families as evidence sources only:

- `concurrent.futures.ProcessPoolExecutor`: `rejected`; Sifr CPU parallelism is `sifr.parallel`, and future process workers must use typed IPC.
- `multiprocessing.Process`: `unsupported-with-diagnostic`; native process spawning is `sifr.process`.
- `multiprocessing.Queue` and `multiprocessing.Pipe`: `unsupported-with-diagnostic`; same-process communication is `sifr.sync`, process communication is typed IPC or raw pipes.
- `multiprocessing.Pool`: `rejected`; future worker pools require a Sifr-native API design over typed IPC.
- `fork` and `forkserver`: `rejected` unless a future host-limited ownership proof is recorded.
- `shared_memory`: `rejected` until explicit ownership, unlink, drop, aliasing, and host cleanup rules are proven.

The existing bare CPython import diagnostics and legacy `sifr.multiprocessing` diagnostic routing remain valid evidence. M6 may add focused fixtures for ProcessPoolExecutor and multiprocessing members that are not already covered.

## Observability And Redaction

IPC observability uses the shared platform fields and must redact payloads. Allowed event and metric labels include:

- `surface="ipc"`,
- frame family,
- protocol state,
- error kind,
- negotiated protocol version,
- schema id or schema hash only when generated and non-user-secret.

Payload bytes, decoded payload values, command lines, environment variables, and stderr contents are not metric labels or diagnostic messages unless an explicit redaction policy exists for that field.

## Implementation Waves After Approval

After this design is reviewed and recorded, M6 implementation should proceed in small PRs:

1. Add typed IPC feature metadata and locked Serde/Postcard dependency wiring only for generated code that uses `sifr.ipc`.
2. Add compiler-known `sifr.ipc` value model and diagnostics for unsupported CPython-shaped process-pool and multiprocessing APIs.
3. Add internal schema descriptor and schema-hash generation with unit tests.
4. Add frame encode/decode helpers with malformed-frame tests and no data-dependent panics.
5. Add process-pipe fixture coverage for bootstrap, request completion, cancellation, close, backpressure, malformed frame, and unsupported payload diagnostics.
6. Update CPython evidence matrices, host matrix, and execution ledger after each merged PR.

M6 closes only when the definition of done in the phase issue is met. A public process-worker pool is not part of M6 closure.
