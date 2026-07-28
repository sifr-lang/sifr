## Round-10 review: certification_5 remediation vs. round-9 findings A–E

Method: code read across the listed scope, plus a read-only codegen probe (out-of-tree binary calling `sifr_lowering::lower_module` + `sifr_codegen::generate_rust_with_metadata`) and `./target/debug/sifr check` on the rust_interop fixture corpus. No repo files modified; no test suites run.

---

### A — cross-module / re-exported consuming-close ownership metadata: **CLOSED**

`ExternalDefs.rust_consuming_methods` (`crates/sifr_lowering/src/lower/external_defs.rs:22`) now carries `module → class → consuming method names`. It is populated locally in `ClassMethodExports::record_local` (`crates/sifr_frontend/src/class_method_exports.rs:50-67`), propagated across re-export chains by `record_imported` keyed on the **local** name (`:70-87`), and re-registered into `ctx.rust_consuming_methods` at import time by `register_imported_rust_consuming_methods` (`crates/sifr_lowering/src/lower/imports.rs:31-49`, called at `:212`), using the same alias basis (`local_name_for`) that `type_for_import` uses to rename the class identity — so `import X as Y` keys match the `Type::Class` name at the call site. Registration sits in `resolve_imports_early`, which runs at `mod_impl.rs:72` before the later import pass, so the other `register_imported_class_instance_methods` call sites (`mod_impl.rs:473`, `:680`, stdlib path) are not a hole; `level > 1` relative imports are rejected outright (`mod_impl.rs:257-268`). Covered by `test_project_lowering_propagates_imported_rust_opaque_close_ownership` and `..._reexported_...` (`crates/sifr_driver/src/tests/project_graph.rs:77-163`), the latter through an aliased facade. No gap found.

### B — typed `Self.*` Closed/Poisoned mapping: **NOT CLOSED** (two confirmed accepted-source → rustc failures)

**B1 (severe): every sync `Self.*` method whose declared error includes `RustPanicError` emits uncompilable Rust.** `self_method_call` builds `RustExpr::Try(self.inner_ref().map_err(…))` as the receiver (`crates/sifr_codegen/src/rust_interop_direct.rs:116-189`); `return_stmt_for_type` then hands the whole expression to `recoverable_sync_panic_result_expr` (`:202-211`), which wraps it in `catch_panic_expr` — a closure (`crates/sifr_codegen/src/rust_interop_panic.rs:197-210`). The `?` therefore lands *inside* the `catch_rust_panic` closure. Probe output for the `opaque_handle_tokenizer` fixture shape:

```rust
match ::sifr_runtime::interop::catch_rust_panic(|| self.inner_ref().map_err(|__sifr_handle_error| match … })?.encode(text)) {
```

Minimal rustc repro of that shape gives `error[E0277]: '?' couldn't convert the error to 'String' … the trait 'From<Declared>' is not implemented`. The closure's return type is whatever the user's `T::encode` returns, so the residual conversion never exists. This is the exact shape of both flagship `Self.*` fixtures — `opaque_handle_tokenizer/positive/declared_send_sync_copy_handle.sifr:11-12` and `opaque_resource_core/positive/stdlib_handle_close_poison_lifecycle.sifr:13-14` — and it is the shape the panic-boundary rules mandate whenever the target is not `trusted_no_panic`. Async `Self.*` and `trusted_no_panic` + plain-error variants are fine (no wrapper), which is why the two runtime fixtures pass: nothing in the corpus ever runs rustc over a sync `Self.*` body.

**B2: the new representability gate does not match what codegen actually emits for the `PythonError` contract shape.** `opaque_self_state_error_is_representable` (`crates/sifr_lowering/src/lower/classes/rust_opaque_validation.rs:120-163`) accepts any all-`str` class with a `message` field; `bridge_error_contract_expr` reaches its `is_python_error_contract()` arm first (`crates/sifr_codegen/src/rust_interop_error_mapping.rs:56-58`) and emits *field accesses* on the value. Probe output for `Result[str, PythonError]` on `@rust(Self.describe)`:

```rust
PythonError { message: "Rust handle is closed".to_string().message.to_string(), … }
…
PythonError { message: __sifr_stored_panic.message.to_string(), … }
```

→ E0609 on `String`, and `RustPanicErrorBridge::message` is a private field/method, after a clean `sifr check`. (The `IOError` special case at `:60-70` happens to be safe because `__io_err` is generic over `Display`.)

### C — non-opaque `Self.*` diagnostics before codegen: **PARTIALLY CLOSED**

The exact round-9 repro is closed: `rust_opaque_validation.rs:12-31` now reports `RUST_CONFIG_MALFORMED_DECORATOR` for a `Self.*` target on a non-`@rust.opaque` class (verified). But the Result-return gate at `:32-43` filters on `method_kind == MethodKind::Regular`, and nothing rejects a `Self.*` target on a static/class method. Confirmed by probe:

- `@staticmethod @rust(Self.describe) def describe() -> str: ...` on an opaque class → `thread 'main' panicked at crates/sifr_codegen/src/class_method_emitter.rs:726: class method IR lowering produced empty body for non-unit return: Tokenizer::describe` — the same compiler panic C described, from accepted source.
- Same with `-> Result[str, TokenError]` → emits `fn describe() -> Result<String, TokenError> { self.inner_ref()… }` in a receiverless trait method → E0424.

### D — `Self.*` bridge receiver contract: **CLOSED**

`signature_contract` now suppresses the synthesized `self` param when the target root is `Self`, and only synthesizes it for `MethodKind::Regular`, with `Own`/`Borrow` picked from `consumes_receiver` (`crates/sifr_codegen/src/rust_interop_bridge_contract.rs:168-206`, backed by the new `ModuleFunction::{method_kind, consumes_receiver}` at `:249-315`). That matches the codegen `SELF_ROOT` branch, which passes no receiver (`rust_interop_direct.rs:84-101`). `stdlib_async_resource_lifecycle.sifr:11-12` no longer publishes a phantom `Handle<…>` bridge parameter.

### E — consuming-close receiver shape: **rejection CLOSED, but it regresses a declared-passing fixture**

`methods_lambdas_and_comprehensions.rs:226-247` now rejects both borrowed-parameter `Name` receivers and — new — every non-`Name` receiver ("must consume an owned local binding; field and temporary receivers cannot prove exclusive ownership"), covered by `rust_opaque_close_rejects_field_receiver_without_owned_local`. The E0507 hole is gone.

The regression: `verification/areas/rust_interop/fixtures/opaque_resource_core/positive/stdlib_handle_close_poison_lifecycle.sifr:19` still calls `handle.close()` on a borrowed `handle`, so this `expected-result: pass` / `evidence-status: passing` fixture is now rejected:

```
error[SIFR-OWN-0003]: cannot consume borrowed parameter 'handle' through Rust opaque cleanup; accept it with `own`
```

It escaped notice because the new guard test hardcodes only three fixture paths (`crates/sifr_lowering/src/lower/rust_interop_tests.rs:583-607`) and this fixture's matrix entry validates against a `sifr_runtime` unit test rather than compiling the source (`.../opaque_resource_core/fixture.json:19-30`). It was the only positive fixture in the area failing for a reason attributable to this change (the other `sifr check` failures — `ThreadsafeCallback`, `object`, `SIFR-ASYNC-0001` — are prelude/harness artifacts unrelated to this diff).

---

### Bottom line

A and D are fully closed; E's enforcement is closed but left one published passing fixture broken; C closed only the `Regular`-method case and the `panic!` at `class_method_emitter.rs:726` is still reachable from accepted source; B's validation gate does not cover the two shapes codegen actually mishandles, and the primary sync `Self.*` shape used by the certification fixtures cannot compile at all.

VERDICT: NOT SATISFIED
