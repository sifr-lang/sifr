# Review: M6 Typed IPC Design Gate — PASS

## Coverage verification

**Design artifact present and sufficient**
- `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` (232 lines) explicitly approves "the M6 typed IPC protocol shape before any serialization dependency is wired into generated projects or runtime code" (L5).

**Required topic coverage** — all present:
- Payload eligibility — `IpcSerializable` strictly above `Sendable`, explicit accepted list (bool/int/float/str/bytes/Option/Result/tuples/list/dict/generated records and enums) and explicit rejected list (file handles, Child/AsyncChild/ProcessHandle, task handles, locks, closures, raw pointers, **arbitrary pickle-like object graphs**) (L167–193).
- Serialization format — length-delimited `u32` LE + Postcard envelope, 16 MiB default cap, `serde_json`/`bincode` rejected for IPC frames (L33–42).
- Versioning/schema identity — canonical descriptor, stable 128-bit `schema_hash` v1, compatible-version negotiation, `UnsupportedSchema`/`UnsupportedVersion` (L44–65).
- Transport layering — M4 child stdin/stdout pipes; stderr stays diagnostic; `sifr.process` retains lifecycle/kill/terminate/timeout/supervision (L22–31).
- Child bootstrap — `Hello`/`Ready`/`Reject` family (L71–75).
- Result/error framing — `Run`/`Started`/`Completed`/`Failed` (L77–82).
- Cancellation/termination — `Cancel`/`Shutdown`/`Terminating` plus race semantics (L84–88, L140–151).
- Backpressure — bounded in-flight window (default 64), `try_run -> IpcSendError::Full(req)`, cancellation-safe `run` (L126–138).
- Close semantics — `shutdown` then `Terminating`; EOF-before-Terminating → `UnexpectedEof`; drop-without-close = compile diagnostic (L149–151).
- Panic-free malformed-frame behavior — typed `MalformedFrame{kind}` for truncated/oversize/decode/state/request_id; "Generated runtime code must not use data-dependent `unwrap`, `expect`, or `panic!` for malformed peer input" (L153–164).

**Layering / non-replacement**
- L9: "does not replace same-process channels, and does not replace raw process pipes for byte/text subprocess workflows."
- L31: foreign executables "remain raw process pipe users" unless they implement the negotiated protocol.

**Worker-pool deferral and CPython classification**
- L20, L124, L232: public process-worker pool deferred; not part of M6 closure.
- CPython-Shaped API Classification (L196–206): `ProcessPoolExecutor`=rejected, `multiprocessing.Process`=unsupported-with-diagnostic, `Queue`/`Pipe`=unsupported-with-diagnostic, `Pool`=rejected, `fork`/`forkserver`=rejected, `shared_memory`=rejected.

**No overclaiming**
- `verification/platform/supported_host_matrix.md` row still `blocked-on-concurrency-runtime-m6` for Linux/macOS/Windows; updated note references the design artifact and explicitly says "Runtime implementation and host evidence remain M6-owned follow-up work."
- Ledger still lists `M6: pending` (with `M6 typed IPC design gate: in progress.`).
- Diff touches only the design doc, host matrix, and ledger — no `Cargo.toml`, no `crates/**`, no Serde/Postcard wiring. Implementation Waves (L221–232) explicitly defers dependency wiring to the next PR after this gate.

**Ledger evidence**
- Added "M6 typed IPC design gate implementation:" section listing what was added and explicitly noting "This PR intentionally does not add Serde/Postcard dependency wiring or public process-worker APIs."
- Added "M6 typed IPC design gate targeted local validation:" with `git diff --check` PASS, file-size guardrail PASS (2246 files, 900-line limit), `wc -l … = 232`.

## Result

**PASS** — design artifact is sufficient, scope is correctly bounded to the M6 entry design gate, layering and deferral commitments hold, no implementation creep, and the ledger captures both the artifact and validation evidence.
