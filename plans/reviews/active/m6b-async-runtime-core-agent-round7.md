I've verified all the key logic. Let me confirm my read of the three flags:

**Flag 3 (gate allowlist tightening) — holds.** `_is_supported_stdlib_core` (gate:87-94) requires `row_id ∈ {async_runtime_core, opaque_resource_core}` **and** `category==supported` **and** both evidences `passing` **and** no `future_owner`. Anything else that is `supported` but not `FUTURE_OWNED` fails at line 66-70. So marking `async_runtime_reqwest` supported would still be rejected — the change tightened, not loosened. Self-test still exercises the rejection path via `opaque_resource_matrix` (a non-allowlisted supported row) and the failing-evidence path on an allowlisted row.

**Flag 4 (id-based self-test selection) — holds.** `_self_test_row` raises `AssertionError` if the id is absent (line 273), so a typo'd id fails loudly rather than vacuously passing.

**Flag 5 (sync-close-only rejection) — genuinely covered.** Traced through `validate_opaque_close_contracts` + `close_method_kind_matches`: `close=async_close` → required method `aclose`. The new `rejects_sync_close_only` test supplies only a method named `close`, so `name == method_name` fails → `SIFR-RUST-HANDLE-0001` fires. The `requires_async_aclose` test supplies `aclose` with `Function` kind → `close_method_kind_matches("aclose", Function)` is false → rejected. `accepts_async_aclose` supplies `aclose`/`Async` → matches. Class names (`Tokenizer`) align between opaque class and method owner, so the matcher's `owner_class == class_name` guard is actually exercised. All three are meaningful.

**Flag 2 (`_sifr.time` repoint) — sound.** Time leaves (sleep/monotonic) route through tokio time, which `async_runtime_core` covers (current-thread affinity, async lifecycle); none need reqwest/network semantics. Repoint from `async_runtime_reqwest` → `async_runtime_core` is correct.

One **non-blocking observation** (residual flag 1):

The `async_runtime_core` compatibility-matrix `notes` claim "**task cancellation/drop semantics**" as part of the supported capability, but neither cited evidence fixture exercises cancellation — the positive fixture only covers async-close lifecycle + current-thread affinity + observe, and the negative covers hidden-blocking rejection. The cancel/join determinism guarantee rests solely on the `sifr_runtime` unit test `async_handle_close_and_cancel_join_are_deterministic`, which is not referenced as matrix evidence. This isn't a correctness defect (the behavior *is* tested via cargo), but the certification-evidence coupling is looser than the notes imply. If you want the matrix to be strictly self-documenting, either trim "task cancellation/drop semantics" from the notes or add a fixture/evidence pointer for it. Not blocking.

Everything else — evidence-id consistency between the two matrices, the `check_fixture_matrix` required-fixtures addition, fixture files present, and the reqwest-stays-future-owned handoff — checks out. The `_is_supported_stdlib_core` guard and the future-owned backstop remain intact.

READY
