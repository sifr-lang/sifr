## Review: M6 Typed IPC Stream Read/Write Helpers — **PASS**

No blocking findings. The wave is correct, panic-free, scope-disciplined, and honestly documented.

### Correctness sweep

**`read_frame` (`crates/sifr_stdlib/src/ipc_transport.rs:17–53`)**
- Length prefix is read via the `read_prefix` helper which cleanly distinguishes three states: no bytes at all → `Ok(None)` (line 74), partial prefix then EOF → `Some(received)` with `received < 4` mapped to `IpcFrameError::LengthPrefixTruncated` (lines 25–30, 75), and full prefix → `Some(4)`. The mapping is unambiguous and matches the design doc's "clean EOF before any length prefix" contract.
- `frame_len > max_frame_bytes` is checked **before** allocating the payload buffer (lines 33–39). This is the critical anti-amplification check — a hostile peer claiming a 4 GiB frame cannot force a multi-GiB `vec![0; n]`. ✓
- `usize::try_from(frame_len)` (lines 40–44) is dead code on 32/64-bit hosts; the sentinel `frame_len: usize::MAX` mirrors the pre-existing pattern in `ipc_frame.rs:272–274`, so consistency is preserved.
- `read_payload` (lines 84–105) handles `Ok(0)` mid-payload as `PayloadTruncated` with accurate `received` count, retries `Interrupted`, and maps all other I/O errors to opaque `IpcTransportError::Read` — no host path, no payload bytes, no `io::Error` kind leaks downstream.
- The frame is reconstructed and routed through `decode_frame` for the final Postcard decode. Slightly redundant (re-validates bounds we just enforced) but functionally correct: by construction the reconstructed buffer is exactly `prefix_len + payload_len`, so `TrailingBytes` cannot fire here.

**`write_frame` (lines 55–65)**
- Delegates to `encode_frame` which enforces `max_frame_bytes` (so the `write_frame_reports_encode_limit_errors` test exercises a real `FrameTooLarge`).
- `write_all` + explicit `flush`; both map every `io::Error` to opaque `IpcTransportError::Write` (lines 63, 64). ✓

**Hostile-input sweep**
- Empty stream → `Ok(None)`. ✓ (test line 150)
- 1–3 byte stream → `LengthPrefixTruncated`. ✓ (test line 160)
- Oversize prefix → `FrameTooLarge` before any payload allocation. ✓ (test line 172)
- Zero-length frame `[0,0,0,0]` → `read_payload` returns an empty `Vec` immediately (loop guard); `decode_frame` then routes `postcard::from_bytes(&[])` to `IpcFrameError::Decode`. No panic. (Not directly tested — see follow-up 4.)
- Truncated payload → `PayloadTruncated` with correct counts. ✓ (test line 187)

**Panic-free guarantee** — grep confirms no `unwrap()`, `expect`, `panic!`, `unimplemented!`, `todo!`, or `unreachable!` anywhere in the new file. All length conversions are `try_from`. ✓

### Scope discipline

The diff adds exactly `read_frame`, `write_frame`, `IpcTransportError`, and the `From<IpcFrameError>` conversion plus tests. No connection state machine, no payload eligibility enforcement, no cancellation/close protocol, no backpressure runtime, no child-process fixture, no `sifr.ipc` surface lowering touched. `lib.rs` change is a single `mod` + single `pub use`. ✓

### Documentation honesty

- `verification/platform/supported_host_matrix.md:41` — claims only host-independent stream helpers and explicitly enumerates what is **not** claimed (child-process fixture transport, connection-state handling, payload eligibility, cancellation, close, backpressure).
- `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:5,32` — status update and evidence row use the phrasing "drops raw I/O error details so payload bytes and host paths are not rendered" which matches the implementation.
- `issues/...execution.md:1039–1054` — implementation/validation/follow-up sections distinguish what shipped from what remains; the `Typed IPC frames over process pipes` row stays `blocked-on-concurrency-runtime-m6`. ✓

### Non-blocking follow-ups

1. **Read-side I/O error mapping is uncovered.** `FailingWriter` proves the write branch, but the `Err(_) => Err(IpcTransportError::Read)` arms at `ipc_transport.rs:78` and `:101` are not exercised. A custom `Read` impl returning a non-`Interrupted` error mid-prefix and mid-payload would close the gap.
2. **Flush-failure branch is uncovered.** `FailingWriter::flush` returns `Ok(())` (line 257), so the second `map_err` in `write_frame` (line 64) is not tested.
3. **`Interrupted` retry behavior is not tested.** Worth a single `Read` impl that returns `Interrupted` once, then valid data — the protocol invariant relies on the retry.
4. **Zero-length frame (`[0,0,0,0]`) decode-failure is not directly tested.** Currently handled correctly (postcard rejects empty input via `IpcFrameError::Decode`), but locking it down would harden the malformed-frame surface.
5. **Multi-frame stream not tested.** Back-to-back `write_frame; write_frame; read_frame; read_frame` would confirm `read_frame` consumes exactly one frame and leaves stream offset correct for the next call — important for the upcoming child-process wave.
6. **Redundant frame re-assembly.** `read_frame` rebuilds `prefix ++ payload` into a fresh `Vec` (lines 47–49) just to call `decode_frame`, which re-validates already-known bounds. At the 16 MiB ceiling that's a 16 MiB peak copy. Calling `postcard::from_bytes(&payload)` directly (with an explicit `IpcFrameError::Decode` map) would avoid the copy. Defensible as defense-in-depth, but worth a follow-up note.
7. **`IpcTransportError` does not impl `Display` or `std::error::Error`** (its inner `IpcFrameError` does). Fine while the surface stays internal, but the moment a caller wants `?`-propagation into anything boxed it will need an impl or a wrapper.
8. **`LengthUnsupported { frame_len: usize::MAX }` sentinel** (lines 41–44) reports `usize::MAX` rather than the real `frame_len`. Unreachable on supported 32/64-bit hosts and consistent with the existing `decode_frame` pattern, so cosmetic.

**Verdict: PASS.** Ready to merge; the follow-ups above are nice-to-haves for the next M6 wave (child-process fixture transport), not gates on this one.
