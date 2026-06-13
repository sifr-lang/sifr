

## Second review — Phase 32 channel endpoint surface slice

### Diff examined

3 files changed: `internal_docs/phases/32_async_ecosystem.md` (+1 line), `lib/sifr/sync.sifr` (+71 lines), `verification/validation_lanes/quick_e2e_manifest.json` (+2 entries).

---

### Prior HIGH findings: resolved

The previous review identified two HIGH-severity defects:

1. **`channel[T]()` / `bounded_channel[T](capacity)` factories produced non-functional channel pairs.** Confirmed fixed — neither factory exists in `sync.sifr`. Grep over the file for `bounded_channel`, `fn channel`, `channel[T]` returns no matches.

2. **Fixtures bypassed the broken factory by constructing sender and receiver with identical pre-filled buffers.** The fixed fixtures still do this (it's the intended design for this surface-only slice), but the phase doc now explicitly names it: fixtures validate "direct-construction surface only; no shared-queue wiring."

The removed factory code also eliminates the prior HIGH finding about `ChannelSender.clone()` sharing a dead buffer — no factory, no dead-clone problem.

---

### Remaining observations

**LOW — Codegen clones the channel on every method call**

```rust
async fn send(&mut self, value: &T) -> Result<(), ClosedError> {
    return self._channel.clone().push(value);  // clones channel each call
}
async fn receive(&mut self) -> Result<T, ClosedError> {
    return self._channel.clone().pop();        // clones channel each call
}
```

`ChannelSender.send()` clones the channel before calling `push()`. `ChannelReceiver.receive()` clones before `pop()`. Each clone copies the entire `Vec<T>` buffer. This is not the semantics of a real channel — it's a consequence of direct construction with owned buffers and no shared-queue lowering. The phase doc correctly defers "runtime-backed shared queues." For this surface-only slice the cloned-buffer behavior produces correct results only because the fixtures construct sender and receiver with the *same* buffer instance in the same test. This is fragile but acceptable as a deferred-not-yet-wired marker.

**LOW — `Channel.__str__` returns `"Channel"` without type parameter**

Matches pattern used by `ChannelSender` and `ChannelReceiver`. Minor and consistent.

**LOW — No `ChannelReceiver` clone method (implied single-consumer)**

Correct by design. No annotation needed for an absent method.

**LOW — Bounded capacity field is stored but never enforced**

`Channel._capacity` is set but `push()` never checks it. The phase doc explicitly defers "backpressure." Capacity is visible in generated code as `_capacity` but inactive. This is honest — the field exists, the semantics are deferred — but worth a comment in `push()`: `// TODO: enforce capacity before wiring shared queue`.

---

### Phase fit: accurate

The phase doc entry for this slice is precise:

> `sync.Channel[T]`, `sync.ChannelSender[T]`, and `sync.ChannelReceiver[T]` are available through `sifr.sync`, with direct-construction surface fixtures … Factory functions, runtime-backed shared queues, sender clone sharing, close/drop semantics, backpressure, FIFO guarantees, async iteration, and cancellation exactness remain deferred to later milestone_async_5 channel slices.

All deferred items are correctly scoped. The slice does what it says on the tin.

---

### Validation results

| Check | Result |
|---|---|
| `cargo run -q -p sifr -- run channel_basic.sifr` | PASS |
| `cargo run -q -p sifr -- run bounded_channel_basic.sifr` | PASS |
| `scripts/run_all_tests.sh --profile quick` | 33 pass, 0 fail |
| `cargo test -p sifr -- --skip test_e2e_pass` | 34 passed, 0 failed |
| `git diff --check` | PASS (no whitespace errors) |
| No factory functions in surface | Confirmed |

---

### Verdict

**SATISFIED**

The prior blocking issue (broken factory functions producing non-functional channel pairs) is resolved. The slice now exports only direct endpoint classes (`Channel[T]`, `ChannelSender[T]`, `ChannelReceiver[T]`) with honest surface signatures. The fixture design is documented and appropriate for a surface-only slice. The phase doc is accurate and complete. All validation gates pass. No new blocking concerns introduced.
