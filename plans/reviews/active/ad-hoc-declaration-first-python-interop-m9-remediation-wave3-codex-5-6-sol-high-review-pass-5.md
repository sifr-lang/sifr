# M9 remediation Wave 3 — Codex 5.6 Sol high review pass 5

Reviewer configuration: `gpt-5.6-sol`, high reasoning, fast service tier, read-only full-wave review.

## Verdict: blocked

1. **High — consuming async owner close still permits cancellation before retained callback drain.** The cancellation mask in `crates/sifr_codegen/src/python_interop_async/callback_frame.rs` activates only when the declaration has callback parameters. A consuming `aclose` instead passes retained owner state through `close_callbacks` from `crates/sifr_codegen/src/python_interop_async/conversions.rs`, while its callback list is empty. It therefore uses the parent carrier directly. Cancellation inside `crates/sifr_runtime/src/python/async_declaration.rs` can abort the wrapper before the awaited owner close, allowing cancellation to complete before drain and capture release.

   Make `close_callbacks.is_some()` activate the same child-carrier finalization scope, and add codegen plus runtime-API cancellation coverage for retained owner `aclose`. This can be tested safely at the runtime layer without spawning a non-`Send` Sifr future.

