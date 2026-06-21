Reviewed the snippets. No blocking correctness/scope/test issues found. Two observations worth a glance but not blockers:

**Observations (non-blocking):**

1. `opaque_validation.rs:90-99` — `opaque_probe_obligations` reuses one tuple to mean different things by branch: opaque returns `(send, sync)`, non-opaque returns `(async_boundary, view)`. The call site at `rust_interop.rs:546-547` then binds both to `(requires_send, requires_sync)`. If this matches existing probe semantics (where async_boundary already implied Send and view already implied Sync) it's fine; if not, the non-opaque branch is silently mislabeling. Worth a one-line confirmation in the PR description, since the snippet doesn't show how those flags reach the final `RustBridgeProbe` struct.

2. `rust_interop_probe.rs:124-147` reads raw decorator arguments via `opaque_bool_argument` / `opaque_symbol_argument` instead of the already-parsed `OpaqueContract` (the validated source of truth from `opaque_contract.rs`). Today this is consistent because `parse_opaque_contract` does no normalization, but it's a divergence risk if defaults or aliases ever land. Minor; not a blocker for M39.6.

**Correctness spot-checks that pass:**
- `Handle::mark_closed` preserves `Poisoned` state (only transitions from `Open`); `mark_poisoned` unconditionally wins — matches the `poisoned_state_wins_over_closed_state` and `double_close_keeps_stable_closed_state` tests at `interop.rs:369-391`.
- `PoisonOnPanic::Drop` correctly guards on `disarmed || !panicking()` and uses `take()` to avoid double-marking.
- `close_method_kind_matches` (`opaque_validation.rs:76-85`) correctly splits sync `close` (Function kind, non-async-boundary) vs `aclose` (Async kind / async_boundary / Async effect).
- `parse_opaque_contract` collects all malformed-arg diagnostics, then short-circuits on missing `type=`, and returns Err on any accumulated diagnostics — no silent acceptance path.
- Diagnostic registry entry `SIFR-RUST-HANDLE-0001` args `[target, method]` align with `validate_opaque_close_contracts`' `push_diagnostic` call site.
- Fixture matrix rows (`opaque_handle_tokenizer`, `close_after_use`) correctly mark resource-shaped wrappers as `planned` under `opaque_resource_matrix`, matching the scope note.

Ready for `scripts/run_all_tests.sh --profile create-pr` validation.
