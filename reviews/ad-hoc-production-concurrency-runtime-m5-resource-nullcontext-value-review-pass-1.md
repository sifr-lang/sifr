## Verdict: **PASS**

Verified the implementation against the diff and confirmed it matches the claimed behavior.

### What I verified

**Source surface (`lib/sifr/resource.sifr`):** Generic `NullContext[T]` with required `value: T` field; `__enter__` returns `self.value`; `nullcontext[T](own value: T = None)` forwards the value with no side effects. Pure Sifr — no CPython adapter.

**Generated Rust (re-emitted):**
- `struct NullContext<T: Clone>` with minimal `Clone` bound
- `fn nullcontext<T: Clone + 'static>` (the `'static` is needed because the type may participate in a guard's `impl Drop` and serves no surprising role)
- With-guards correctly preserve full generic instantiation: `NullContext<()>`, `NullContext<i64>`, `NullContext<String>` — the previously-missing generic args bug is fixed
- Cleanup runs via `impl Drop` on `__WithGuard*`, deterministic ordering, scoped strictly inside the `with` block
- No `unwrap()` / `expect()` / `panic!` / `?` in any user-path code. `__enter__` is `value.clone()`, `__exit__` is empty
- No Tokio handle, no async runtime — synchronous `with` only

**Targeted runtime check:** `cargo run … resource_nullcontext_basic.sifr` re-ran cleanly with `cache_hit=true` (artifact reused, test still gated by passing run).

**Scope discipline of the codegen changes:**
- `is_nullcontext_value_forwarder` is tightly name-pinned (`"nullcontext"` + `NullContext` return + single-Return-of-NullContext-ctor body + args that are all forwarded params). Consistent with the existing per-class special-case for `deque` in `generic_bounds_helpers.rs`.
- The `NoneLiteral` → `Unit` lowering is gated by `Type::None | Type::TypeVar(_)` and runs *before* `is_option_type` (which does not match those), so `Option<U>` parameters keep their `Option::None` lowering. Narrow enough to avoid collateral damage.
- The `with`-guard fix swaps `name.clone()` for `rust_type_with_generics(value.ty())` only inside the synchronous `try_lower_with_stmt_for_ir` path — async `with` is untouched.

**Docs honesty:** Status lines, supported-host matrix, traceability table, and substrate inventory all explicitly cap the claim at "no-value and value-carrying generic `nullcontext(...)` via the synchronous `with` protocol" and continue to list ExitStack/AsyncExitStack, `closing`/`aclosing`, cancellation cleanup ordering, and task-context propagation as M5 follow-up. No overclaim against the CPython evidence (which only asserts that `with nullcontext(c) as x` yields the same `c`).

### Non-blocking notes
- The `nullcontext` bound special-case is a stdlib-coupled string match. Acceptable given the existing `deque` precedent, but a structural marker (e.g. an attribute on the HIR function for "stdlib value-forwarder") would be more durable if more such helpers land.
- `'static` on `nullcontext<T>` is conservative; it is harmless here but worth noting in case future helpers want to carry borrowed values.
