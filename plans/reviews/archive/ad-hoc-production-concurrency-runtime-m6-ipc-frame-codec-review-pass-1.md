I reviewed the M6 typed IPC frame codec wave on branch `codex/concurrency-runtime-m6-ipc-frame-codec` against `origin/main`.

## Verdict: PASS

No blocking findings. The wave delivers the internal length-prefixed Postcard codec called out as M6 implementation step 4 in `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:240`, keeps scope discipline, and refrains from claiming transport / connection / cancellation / backpressure surfaces.

## Scope and dependency policy

- `Cargo.toml:87` adds `postcard = { version = "1.1.3", default-features = false, features = ["use-std"] }`, matching the Ring 4 M6 dependency decision and the prior dependency-metadata wave (no default features, `use-std` only).
- `crates/sifr_stdlib/Cargo.toml:10-11` pulls workspace `postcard` and `serde` only into `sifr_stdlib`. No other crate is widened.
- `crates/sifr_stdlib/src/lib.rs:15,36-40` declares the new `ipc_frame` module and re-exports the codec surface alongside the existing `ipc_schema` re-exports — consistent with the established internal-stdlib pattern; no compiler / codegen / lowering registrations were added, which is appropriate for an internal helper slice.
- No new public language surface, no `sifr.ipc` module wiring, no `ProcessPoolExecutor`/`Process` accept gates. Matches "this wave does not claim … connection-state handling, payload eligibility enforcement, cancellation, close, or runtime backpressure support."

## Correctness — encoder

`crates/sifr_stdlib/src/ipc_frame.rs:236-256`:

- Encodes the envelope first, then bounds-checks `payload.len()` against `u32::MAX` (via `u32::try_from`) and `max_frame_bytes` before assembling the frame. Order is correct — there is no way to silently truncate a length prefix.
- Uses `to_le_bytes()` for the prefix, matching the design's "`u32` little-endian payload byte length" (design doc line 49).
- No `unwrap`/`expect`/`panic!`; both fallible boundaries are typed (`IpcFrameError::Encode`, `IpcFrameError::LengthUnsupported`, `IpcFrameError::FrameTooLarge`).

## Correctness — decoder

`crates/sifr_stdlib/src/ipc_frame.rs:258-296`:

- Length prefix slice via `bytes.get(..IPC_LENGTH_PREFIX_BYTES)` so a short buffer yields `LengthPrefixTruncated`, never an index panic.
- Oversize check (`frame_len > max_frame_bytes`) precedes the payload-decode call, so a hostile peer cannot force a giant `Vec` allocation prior to bounds enforcement.
- `usize::try_from(frame_len)` and `payload_start.checked_add(payload_len)` cover the 16-bit / arithmetic-overflow corners with typed errors.
- Strict equality between `bytes.len()` and `payload_end` — both shorter (`PayloadTruncated`) and longer (`TrailingBytes`) inputs are typed errors. This is correct for a single-frame helper; a streaming reader will be a separate concern in the transport wave.
- Postcard decode failure is mapped to `IpcFrameError::Decode` via `map_err(|_| _)`, so postcard's own error rendering (which can quote bytes) never reaches `Display`.

## Malformed-input behavior — panic-free

I scanned `ipc_frame.rs` for `unwrap`, `expect`, panicking arithmetic, and unguarded indexing in runtime paths. None present outside the `#[cfg(test)]` block (test-only `panic!("sample hello frame should encode")` at `:327`/`:398`, which is fine). All fallible operations are propagated as typed `IpcFrameError`. This satisfies the design's "Generated runtime code must not use data-dependent `unwrap`, `expect`, or `panic!` for malformed peer input" (design doc line 176).

## Error redaction

`crates/sifr_stdlib/src/ipc_frame.rs:189-232` only formats numeric metadata — lengths and counts — never payload bytes or postcard's underlying error message. Specifically:

- `Decode` → fixed string `"failed to decode IPC frame"` (the postcard error is dropped at `:295`).
- `PayloadTruncated`/`TrailingBytes`/`FrameTooLarge`/`LengthUnsupported`/`LengthPrefixTruncated` → only `u32`/`usize` counts.

This satisfies the M6 observability rule that payload bytes are not metric labels or diagnostic messages (design doc line 231) and the ledger claim that "Errors must not render payload bytes."

## Test adequacy

`crates/sifr_stdlib/src/ipc_frame.rs:298-487` exercises 9 tests:

1. Hello round trip + `kind()` check.
2. All 15 frame-family variants round-trip (`Ready`, `Reject`, `Run`, `Started`, `Completed`, `Failed`, `Cancel`, `Shutdown`, `Terminating`, `Heartbeat`, `WorkerStatus`, `MalformedFrame`, `UnsupportedVersion`, `UnsupportedSchema`, `UnsupportedPayload`).
3. Encode rejects oversize against negotiated max.
4. Decode rejects truncated length prefix.
5. Decode rejects oversize before payload decode.
6. Decode rejects truncated payload.
7. Decode rejects invalid postcard payload.
8. Decode rejects trailing bytes.
9. `Display` for `Decode` renders no payload bytes.

The error-path coverage maps to every `IpcFrameError` variant except `Encode` and `LengthUnsupported`. `Encode` is hard to provoke (postcard would need a custom serializer failure), and `LengthUnsupported` is unreachable on 32/64-bit hosts — so this is appropriate, not a coverage gap.

## Documentation honesty

- `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:5` status line adds "internal `sifr_stdlib` helpers now encode/decode the length-prefixed Postcard envelope with malformed-frame tests" while keeping process-pipe transport, connection-state malformed-frame behavior, cancellation, close, backpressure, and payload eligibility diagnostics on the M6 follow-up list.
- `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:31` traceability row "Internal length-prefixed Postcard frame codec" explicitly notes "Process-pipe transport and connection-state handling remain follow-up work." No overclaim.
- `verification/platform/supported_host_matrix.md:40` adds a new "Typed IPC frame codec helpers" row marked `supported` on all three hosts; the row's notes restrict the claim to "host-independent length-prefixed Postcard envelope encoding/decoding, the default 16 MiB frame payload limit, and typed malformed-frame errors" and explicitly disclaim transport, connection, cancellation, close, eligibility, and backpressure. The pre-existing "Typed IPC frames over process pipes" row at line 41 remains `blocked-on-concurrency-runtime-m6`, correctly distinguishing the helper slice from end-to-end host support.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:1009-1028` adds the implementation entry, the targeted-validation entry, and a `Pending` review-loop placeholder, matching the prior schema-hash entry's structure.
- Ledger line-count claims: `wc -l` on the touched files reports `crates/sifr_stdlib/src/ipc_frame.rs` 487, `crates/sifr_stdlib/src/lib.rs` 436, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` 244 — exact match with the ledger numbers at `:1023`.

## Validation reproduction

The user-reported local results (`cargo fmt --check`, `cargo test -p sifr_stdlib ipc_frame -- --nocapture` × 9, `cargo clippy -p sifr_stdlib -- -D warnings`, `git diff --check`, `python3 scripts/check_file_size_guardrails.py` 2251 files / 900 cap) are consistent with the diff: no warnings would be expected from clippy on the code as written (no `unwrap`/`expect`, no unused, all public items have `#[must_use]` or return `Result`), and the new file is 487 lines — well under the 900-line guardrail.

## Non-blocking follow-ups (not gating this PR)

1. **`IpcFrameError::LengthUnsupported.frame_len` field is `usize` while every other length-bearing variant uses `u32`** (`ipc_frame.rs:180-187`). At `:272-274` the `usize::try_from(frame_len)` mapper also discards the original `u32` and substitutes `usize::MAX`, which is unreachable on 32/64-bit hosts but mildly misleading on theoretical 16-bit targets. Consider either widening the variant to `u32` and storing `frame_len` verbatim, or removing the dead 16-bit branch — but only if you touch this code again.
2. **`IpcWireSchema` bundles `schema_id` + `schema_hash` + compatible version range into one struct, while `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:85-86` still lists `schema_id`, `schema_hash`, `schema_version_min`, and `schema_version_max` as separate `Hello`/`Ready` fields.** Internal struct shape vs. design field-listing — both produce the same wire bytes through postcard, and the design is pseudo-code, so this is informational. When the public `sifr.ipc.SchemaId` lowering lands, the design's per-field framing may want a refresh to match the helper layout.
3. **`IpcMalformedKind::TrailingBytes` is not in the design's malformed-frame kinds list** (`verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:165-176`). Adding the kind is sensible — postcard accepts shorter-than-buffer inputs silently, so trailing-bytes detection is real protocol defense — but the design enumeration could be updated in a future doc pass for parity.
4. **Workspace `postcard` line ordering** (`Cargo.toml:87`). It is inserted between `once_cell` (line 86) and `num-bigint` (line 88); the section is already not strictly alphabetical (`num-*` already sits below `once_cell`), so this is a pre-existing inconsistency, not a regression. Worth a one-line sort the next time `Cargo.toml` is touched.

None of the above blocks this PR.
