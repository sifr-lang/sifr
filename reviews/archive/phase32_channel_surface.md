# Review: milestone_async_5 channel endpoint surface slice

## Scope under review

- `lib/sifr/sync.sifr`: `Channel[T]`, `ChannelSender[T]`, `ChannelReceiver[T]`, `channel[T]()`, `bounded_channel[T](capacity)`
- E2E pass fixtures: `channel_basic.sifr`, `bounded_channel_basic.sifr` → `quick_e2e_manifest.json`
- Phase doc update: `internal_docs/phases/32_async_ecosystem.md`

Design sources: `internal_docs/async_concurrency_model.md` (canonical contract) + `internal_docs/phases/32_async_ecosystem.md` (implementation tracker).

---

## Findings by severity

### HIGH — Shared mutable buffer through `.clone()` breaks multi-producer intent

The `channel[T]()` factory clones the inner `Channel[T]` buffer for the sender side:

```sifr
def channel[T]() -> tuple[ChannelSender[T], ChannelReceiver[T]]:
    inner: Channel[T] = Channel([], -1)
    return (ChannelSender(inner.clone()), ChannelReceiver(inner))
```

`Channel.clone()` copies `_buffer: list[T]` by reference or shallow clone — both sender and receiver end up with *separate* buffer instances. Data sent on the sender never reaches the receiver. The fixtures mask this by directly constructing `ChannelReceiver(Channel([42], -1))` with an *identical* buffer, bypassing the factory entirely.

**Codegen evidence**: `ChannelSender::send` calls `self._channel.clone().push(value)` — clones the *entire* sender-side channel and pushes to the clone's buffer. The receiver's channel is untouched.

**Impact**: The factory produces two dead channels, not a working pair. This is not a deferred runtime behavior — it is a fundamental correctness failure of the surface itself.

**Fix required**: Sender and receiver must share *one* underlying buffer. The canonical implementation (Rust `mpsc::channel`) uses a shared queue behind `Arc`. Until proper shared-queue lowering lands, the minimum honest surface is that `channel[T]()` and `bounded_channel[T](capacity)` raise a compilation error or are absent, with an explicit "not yet implemented" comment. Shipping them as currently written is misleading.

### HIGH — Bounded capacity is ignored; backpressure cannot work

`bounded_channel[T](capacity)` stores `capacity` but `push()` never checks it:

```sifr
def push(self, own value: T) -> Result[None, ClosedError]:
    if self._closed:
        raise ClosedError()
    self._buffer.append(value)  # no capacity check
    return None
```

There is no `WouldBlockError` return for full channels, no async backpressure. This is explicitly called out in the deferral note, but the factory is still exported with its signature intact. Users calling `bounded_channel[T](5)` expect bounded behavior.

**Mitigating note**: The deferral note lists "backpressure" as deferred. However, exporting a named factory with a `capacity` parameter that is silently ignored sets a false expectation. The capacity field should be validated at minimum (reject capacity ≤ 0 for unbounded, and implement a real check or deferral stub for bounded).

### MEDIUM — Fixture constructs channels wrong, hiding the shared-bUFFER failure

Both fixtures do:

```sifr
receiver: ChannelReceiver[int] = ChannelReceiver(Channel([42], -1))
```

This creates a *new* channel with the pre-filled buffer, bypassing the broken factory entirely. The fixture validates the method signatures (`send`/`receive` are async, return types are `Result[...]`) but exercises zero real multi-producer or producer-consumer wiring.

This is acceptable *if* the fixture name and comments explicitly state "validates method surface only; no real queue wiring". The fixture should include a comment:

```sifr
# Validates method surface: async send/receive signatures, Result types, and
# basic construction. Real shared-queue wiring (sender/receiver sharing one
# buffer) is deferred to a later milestone_async_5 slice.
```

### MEDIUM — `ChannelSender.clone()` shares a separate buffer, not a shared queue

The milestone spec says "`ChannelSender[T]` is clonable; `ChannelReceiver[T]` is single-consumer in v1". The current `clone()` implementation returns `ChannelSender(self._channel)`, which shares the same `Channel[T]` reference as the original sender. Both clones and the receiver share one `Channel` instance — but the `Channel` itself holds a list, and the factory's `.clone()` already created separate buffers for sender vs receiver.

So the current `clone()` on a factory-created sender gives you two senders pointing to the same dead buffer. This doesn't match any real multi-producer semantics. Again, this should be deferred or explicitly stubbed with a note.

### MEDIUM — `bounded_channel[T]` capacity validation missing

`bounded_channel[T](capacity)` accepts any `int`, including negative values. `Channel([], -1)` is valid in the current code. For unbounded channels, negative capacity is a sentinel. For bounded channels, negative or zero capacity should be rejected (or documented as invalid input).

### LOW — `Channel.__str__` returns a bare string, not `"Channel[T]"`

```sifr
def __str__(self) -> str:
    return "Channel"
```

This is minor, but inconsistent with the pattern used by `ChannelSender` and `ChannelReceiver`. The generic type parameter is not reflected.

### LOW — No `ChannelReceiver` clone stub annotation

The spec says receiver is single-consumer in v1 but no diagnostic or stub prevents `ChannelReceiver.clone()`. Since `ChannelReceiver` has no `.clone()` method, this is implicitly enforced — but a future developer might add one. An explicit comment or stub returning a compile error would be clearer.

---

## Phase fit assessment

The slice is framed as a "surface/typing/basic execution slice only". The phase tracker defers runtime-backed shared queue state, sender clone sharing, close/drop semantics, backpressure, FIFO guarantees, async iteration, and cancellation exactness. These deferrals are accurate and correctly documented.

**However**, the shared-buffer failure (`HIGH`) is not a deferred behavior — it is a correctness defect in the surface that exists *right now*. The factory cannot produce a working channel pair. This goes beyond "deferred runtime semantics"; it makes the exported API fundamentally non-functional for its intended purpose.

---

## Regression risk

- **No regression in existing surfaces**: The additions are purely additive.
- **No test regression**: Quick lane passes.
- **Codegen stability**: Generated code is clean (no raw `.unwrap()` in user paths, no `panic!`).
- **Breaking change risk**: Low — the types and factories are new.

---

## Missing validation

1. A negative fixture validating that channel factories are not yet fully functional should exist with a comment noting the shared-buffer gap. Otherwise, future reviewers and developers may assume the current implementation is complete.
2. A negative fixture for `ChannelReceiver.clone()` (currently implicitly rejected by absence) should be formalized.
3. Capacity validation for `bounded_channel` with negative values should be either enforced or documented as deferred.

---

## Verdict

**NOT SATISFIED**

The slice correctly documents its deferrals and correctly adds method surfaces and signatures. However, it ships `channel[T]()` and `bounded_channel[T](capacity)` as working factory functions when they produce non-functional channel pairs: the sender and receiver each hold separate buffer instances and cannot communicate. This is a high-severity correctness defect, not a deferred runtime behavior.

The minimum acceptable fix before merging:
- **Remove or comment-out the broken factory functions** with an `sifr TODO` stub noting they require a shared underlying queue, **OR**
- **Wire a real shared buffer** through Sifr's ownership/Arc lowering before shipping the factory.

The fixtures may remain as method-surface validation but should include an explicit comment that they bypass the factory and do not validate multi-producer/receiver wiring.

The phase doc note is accurate and should be preserved.

---

*Reviewed: 2026-05-10*
