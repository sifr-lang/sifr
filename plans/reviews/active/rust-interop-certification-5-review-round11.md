## Round-11 review: certification_5 remediation of round-10 findings B1, B2, C, E

**Method.** Read-only code read across the listed scope, plus two read-only probes: (1) an out-of-tree crate (`/tmp/c5probe`, path-deps on `sifr_lowering`/`sifr_codegen`) that lowers a `.sifr` file and prints the generated Rust; (2) `rustc --emit=metadata` over that generated Rust with the target crate stubbed, linked against `target/release/libsifr_runtime.rlib`. No repo files modified, no test suites run. (Build artifacts for the probe crate landed under the gitignored `target/`.)

---

### B1 — sync `Self.*` + recoverable panic wrapper emits uncompilable Rust: **CLOSED**

`self_method_call` now returns a `(RustStmt::Let, RustExpr)` pair (`crates/sifr_codegen/src/rust_interop_direct.rs:122-149`) with the `RustExpr::Try` on the `let` value (`:202`), and `rust_interop_method_body` prepends that binding before `return_stmt_for_type` (`:85-118`), so `catch_panic_expr` (`rust_interop_panic.rs:197-210`) only ever wraps `__sifr_opaque_inner.<method>(…)`.

Verified on the real emitter, not just the shape. For `opaque_resource_core/positive/stdlib_handle_close_poison_lifecycle.sifr`:

```rust
fn read_text(&self) -> Result<String, __SifrUnion_…> {
    let __sifr_opaque_inner = self.inner_ref().map_err(|__sifr_handle_error| match … })?;
    match ::sifr_runtime::interop::catch_rust_panic(|| __sifr_opaque_inner.read_text()) { … }
}
```

Both flagship fixtures' generated modules **type-check under rustc** (only an `unused_variable` warning for `close_result`): `stdlib_handle_close_poison_lifecycle` and `declared_send_sync_copy_handle`. The `?` residual is now `From<Union> for Union` (reflexive), the closure has no `UnwindSafe` bound to satisfy (`interop.rs:64-70` uses `AssertUnwindSafe`), `RustPanicErrorBridge::message()` is public (`interop.rs:279`), and the pre-bound `&T` is captured by ref with autoderef.

No typing/lifetime regression from pre-binding: the consuming variant binds `self.into_inner()` → owned `T` and the closure captures by move (`fn close(self)` on `Handle<T>`, probe `c5`); the async variant emits `let … = self.inner_ref()…?;` then `__sifr_opaque_inner.read().await` with no wrapper (probe `c6`).

### B2 — representability gate vs. what codegen emits: **CLOSED** for the reported `PythonError` shape

`opaque_self_state_error_is_representable` (`crates/sifr_lowering/src/lower/classes/rust_opaque_validation.rs:137-181`) excludes `is_python_error_contract()` and `RustPanicError`-named ordinary members, requires a `message: str` field with all-`str` fields, and for unions requires exactly one ordinary member plus ≤1 `RustPanicError`. Probe results: `Result[str, PythonError]` and `Result[str, PythonError | RustPanicError]` are both rejected with `SIFR-RUST-CONFIG-0001`; so are two-ordinary-member unions, bare `RustPanicError`, and non-message shapes. The `python_error_expr` field-access arm (`rust_interop_error_mapping.rs:56-58`) is therefore unreachable from `Self.*`, and the E0609/private-field emission from round-10 is gone. One residual hole in the same gate is reported below (finding 2).

### C — `Self.*` on static/class methods reaching the `class_method_emitter.rs:726` panic: **CLOSED**

`rust_opaque_validation.rs:32-48` rejects any `Self`-rooted target on a `method_kind != MethodKind::Regular` member before codegen. Probes confirm the exact round-10 repro now diagnoses instead of panicking: `@staticmethod @rust(Self.describe) def describe() -> Result[str, TokenError]`, the same with `-> str`, and `@classmethod` all produce `SIFR-RUST-CONFIG-0001` "require regular instance methods with a handle receiver". Non-opaque owners are rejected at `:21-31`, and module-level `Self.*` targets are rejected earlier at `lower/rust_interop.rs:535`. `MethodKind` has only `Regular | ClassMethod | StaticMethod` (`sifr_ir/src/hir_nodes.rs:120-124`), so there is no third receiverless kind left. A `def encode(text: str)` member is not a hole: the first parameter is the receiver, and the emitter produces `fn encode(&self)` — no `E0424` shape.

### E — consuming-close receiver rule vs. published passing fixtures: **NOT CLOSED**

The named fixture is repaired: `opaque_resource_core/positive/stdlib_handle_close_poison_lifecycle.sifr:17` now takes `own handle`, and the guard test's hardcoded list grew to four paths including it (`crates/sifr_lowering/src/lower/rust_interop_tests.rs:636-670`).

But the escape mechanism round-10 identified — a hardcoded fixture list — still omits `callback_subscription_core/positive/signal_subscription_cancel_shutdown.sifr`, which has the *same* rejected shape (`await subscription.aclose()` on the borrowed parameter `subscription`, fixture line 27-29). It is published `evidence-status: passing` / `expected-result: pass` (`fixture.json` positive: `"expected_result": "pass"`, `"status": "passing"`), and its validating driver test uses a local `CALLBACK_SOURCE` constant rather than the fixture file (`crates/sifr_driver/src/build/rust_interop_callback_contract_tests.rs:27-33`), so nothing compiles the source. Details in finding 1.

---

### Findings

**1. `callback_subscription_core` positive fixture is rejected by the consuming-cleanup rule (E residual, confirmed).** Probe on the fixture file: `SIFR-OWN-0003 cannot consume borrowed parameter 'subscription' through Rust opaque cleanup; accept it with 'own'`, from `methods_lambdas_and_comprehensions.rs:226-238`. Minimal repro without the `ThreadsafeCallback` prelude artifact reproduces it, so this is not a harness artifact. Fix is the same one-line shape as the repaired fixture (`own subscription`), plus adding this path (and, for symmetry, `opaque_resource_matrix/positive`, which does satisfy the rule) to the guard list at `rust_interop_tests.rs:637-670`. `callback_subscription_ecosystem/positive/subscription_cancel_shutdown.sifr` has the same shape but is `evidence-status: planned` / `expected-result: future-owned`, so it is not a contract violation.

**2. The representability gate still admits reserved message-alias error names with extra fields (B2 residual, confirmed).** `opaque_self_state_error_is_representable` accepts any all-`str` class with a `message` field, but `bridge_error_expr_with_contract` reaches the `is_message_error_alias` arm first (`rust_interop_error_mapping.rs:45-55`, name list at `:119-130`) and initializes **only** `message`. Probe: `class HttpError(Error): message: str; detail: str` on `@rust(Self.fetch) def fetch(self) -> Result[str, HttpError]` passes lowering and emits three uncompilable initializers; rustc gives `error[E0063]: missing field 'detail' in initializer of 'HttpError'` at the Closed arm, the Poisoned arm, and the bridge `map_err`. Same for `NetError`, `TlsError`, `HeaderError`, `HttpError`, `ProcessError`, `SignalError`, `DiagnosticError`. Note this arm is not `Self.*`-specific — a plain `@rust(bridge.net.fetch) def fetch(…) -> Result[str, HttpError]` emits the identical missing-field initializer, so the underlying codegen defect predates this diff; what is in scope is that the new gate claims representability for it. Narrowest fix inside this scope: reject an accepted error class whose name hits `is_message_error_alias` and whose field set is larger than `{message}`.

Not reported as findings: `"Rust handle is closed".to_string().to_string()` (valid, cosmetic); the fixture-matrix entries that validate `Self.*` fixtures via `sifr_runtime`/driver unit tests rather than compiling the source (an evidence-coverage gap round-10 already recorded, unchanged here). Docs (`docs/rust-interop.mdx:210-220`) and `internal_docs/rust_interop_architecture.md` state the new `Self.*` restrictions and the owned-cleanup rule, so finding 1 is a fixture-vs-contract inconsistency, not a doc drift.

VERDICT: NOT SATISFIED
