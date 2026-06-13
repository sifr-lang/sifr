

Now I have the full picture. Let me compile the review.

---

## Review: Channel-Backed Async-For Iteration

### Q1: Is `ChannelReceiver[T]` as an infallible async iterator via `anext() -> Option[T]` acceptable for the current HIR/codegen model?

**Yes.** The model contract (async_concurrency_model.md, line 262) explicitly defines:

> `ChannelReceiver[T]` implements `AsyncIterator[T, Never]` by mapping a closed-and-drained `ClosedError` from `receive()` to `Ok(None)`.

The HIR `async_iterator_parts` (async_for.rs:55-71) extracts `anext()` from class methods, expecting `Result[Option[T], E]`. The runtime replacement (`lib.rs:1054-1059`) delivers this structurally:

```rust
async fn anext(&mut self) -> Option<T> {
    match self.receive().await {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}
```

The async fn body desugars to `Coroutine<Option<T>, Never>`, which is assignable to `Result[Option[T], Never]` (never fails). The error-type extraction yields `Type::Never`, satisfying the `AsyncIterator[T, Never]` contract. This is clean and consistent with the model.

### Q2: Does omitting `?` for Never iterators preserve the existing fallible async-for behavior while allowing `async def main() -> None` for channel-backed loops?

**Yes.** The codegen split in `stmt_support_emitter.rs:7662-7673`:

```rust
let infallible_iter = matches!(iter_error_ty.resolve_alias(), Type::Never);
let next_value = if infallible_iter {
    next_call                          // no Try wrapper — await only
} else {
    crate::RustExpr::Try(Box::new(next_call))  // wrap in ? for fallible
};
```

is exactly right:

- **Fallible iterators** (`anext() -> Result[Option[T], E]` where `E != Never`): `await anext().await?` propagates `Err(E)` through ordinary Sifr error handling. Enclosing functions must carry `E` or the type checker rejects at HIR level (`async_for.rs:131-138`).

- **Infallible iterators** (`anext() -> Option[T]`, error = `Never`): `await anext().await` with no `?`, yielding `Option<T>` directly. No error path exists, so `async def main() -> None` is valid.

The generated code confirms this:
```rust
// from async_for_channel.sifr — no Try, just await
let __sifr_async_next = receiver.anext().await;
```

The design is sound and the HIR/codegen boundary correctly gates both cases.

### Q3: Any risk from the `sync.sifr` method body being a minimal type-surface stub?

**Low risk, but worth noting.** The stub (`lib/sifr/sync.sifr:99-100`):

```sifr
async def anext(self) -> Option[T]:
    return None
```

exposes the correct type for HIR lowering and type checking. The real behavior comes from `replace_sync_channel_runtime_items` in codegen. The stub body is never executed — it is stripped and replaced wholesale.

The only latent risk is if the stub were ever used in isolation (e.g., as a library reference without the runtime replacement), it would silently return `None` regardless of actual channel state. However:

1. The runtime replacement is applied whenever `sync.sync` is imported and channel types are detected (`lib.rs:458-460`).
2. All positive validation uses the full generated output with runtime replacement.
3. The approach is consistent with how other sync channel types (`push`, `pop`, `receive`) already use runtime replacement for their real implementations.

This is a known and intentional pattern, not a footgun.

### Additional Observations

- **Validation completeness:** `async_for_channel.sifr` exercises the full path: send two values, close sender, `async for` drain. The assertions confirm values are received in order and `ClosedError` does not leak into the function signature.

- **Fallible companion fixture:** `async_for_stream_result.sifr` validates that fallible iterators (`anext() -> Result[Option[T], E]`) still require enclosing functions to carry the error type, keeping the dual-path behavior tested.

- **Manifest correctness:** `async_for_channel` is in the quick validation lane alongside its fallible sibling, providing continuous coverage.

### Verdict

All three design decisions are correct against the async concurrency model:

1. `ChannelReceiver[T].anext() -> Option[T]` is the canonical way to implement `AsyncIterator[T, Never]`, matching the model contract.
2. The `?`-omission for `Never` iterators preserves fallible behavior while enabling clean `async def main() -> None`.
3. The sync.sifr stub + runtime replacement pattern is intentional, low-risk, and consistent with the existing channel implementation approach.

REVIEW_STATUS: SATISFIED
