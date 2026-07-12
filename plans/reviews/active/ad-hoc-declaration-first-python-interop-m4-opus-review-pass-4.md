I've reviewed the narrow M4 correction in `crates/sifr_lowering/src/lower/classes/class_body_lowering.rs` and traced its interactions with the ownership/convention/must-use paths.

**What the correction actually does**

```rust
let convention = if ctx.must_use_obligation_for_type(&param_ty).is_some() {
    ast_convention_to_param(param.parameter.convention, &param_ty)
} else {
    ParamConvention::default()
};
```

- `ParamConvention::default()` = `{ Borrow, Immutable }` (types/definitions.rs:311–315). This is exactly the value used in the pre‑M4 line `convention: ParamConvention::default()`, so ordinary class method params round‑trip to the same ABI they had before M4.
- `must_use_obligation_for_type` (mod_context.rs:216–234) only returns `Some` for opaque Python classes with `cleanup ∈ {close, async_close, context, async_context}` and their container wrappers. Every non‑opaque Move/Copy type falls to the `default()` branch.

**M4 opaque cleanup=close guarantees still hold**

- Opaque `own`‑annotated params: the must‑use branch runs `ast_convention_to_param`, producing `Own`. The subsequent block at 496–501 then calls `record_must_use_binding`, so the exit check at 507–524 will flag any owned opaque param that isn't moved/closed. Unchanged behavior.
- Opaque `borrow` (implicit) params: `ast_convention_to_param(borrow, Move)` = `Borrow`; `is_owned() == false`; not tracked. Correct — a borrowed opaque doesn't take on the must‑use obligation.
- Consuming `Self.close` receiver: `receiver_is_owned` (python_interop.rs:604–609) is derived directly from AST `own self` before `params` is populated, so `HirParam.convention` for the receiver was never involved. The `semantic_close_methods` counter (583–599) and the "consuming method is reserved for cleanup=close" guard (601–614) both key off `python_interop.consumes_receiver`, which is untouched by this correction.
- Container types (`Result[None, PythonError]`, `list[Opaque]`, `dict[str, Opaque]`, unions/tuples containing opaque) still flow through `must_use_obligation_for_type` recursively (mod_context.rs:224–231), so nested obligations propagate.

**Ordinary class method ABI is restored**

- `class_method_emitter::lower_class_method_param_type` (crates/sifr_codegen/src/class_method_emitter.rs:311–320) emits `&T` only when `param_ty.ownership() != Copy && convention.is_borrowed()`. Restoring `ParamConvention::default() = Borrow` for non‑opaque types keeps Copy types (like `i64`) as owned in the emitted Rust signature — the send‑call regression's precondition is removed at the source.
- The FunctionType stored by `class_type_collection.rs` still routes params through `FunctionType::new` (Own for Copy/TypeVar, Borrow for Move), which is what channel/callable type inference reads. Both sides now agree again.
- Free functions still apply `ast_convention_to_param` unconditionally (annotations_and_function_lowering.rs:465, 493, 514, 541). The asymmetry between class methods (narrow, opaque‑only) and free functions (full AST) is intentional pre‑M4 compatibility, not a regression.

**Validation cross‑checked**

- The correction's `if is_owned() { record_must_use_binding }` block (497–501) is the sole gate for HirParam‑derived must‑use bookkeeping, and it's driven by the corrected convention — so narrowing convention to must‑use types is exactly what the tracker consumes.
- The runtime reports `cargo test -p sifr_lowering python_interop --lib` = 22 passed and `run_e2e_pass.sh --profile create-pr` = 130 passed / 0 failed, which is the merge‑gate profile.

No actionable blocker found. The correction is a minimal restoration of pre‑M4 convention for non‑opaque types that leaves every must‑use/opaque cleanup enforcement path — recording, exit rejection, semantic‑close counting, consuming‑receiver guard — intact.

SATISFIED
