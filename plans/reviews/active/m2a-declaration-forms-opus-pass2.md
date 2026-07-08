I've reviewed the diff thoroughly, including the new `private_stdlib_imports.rs` helper module, the expanded `is_supported_annotated_module_constant_expr` predicate, and the metadata registration compared to the public `sifr.*` import path.

## Assessment

**Structure & correctness:**

- `resolve_compiled_private_imports` (`crates/sifr_lowering/src/lower/private_stdlib_imports.rs:6-29`) correctly walks function → class → constant → retained-fallback in the same order as the public `sifr.*` handler (`mod_impl.rs:353-528`). Return value semantics are right: `false` only when the module is absent from externals (so the caller falls through to retained-only), `true` even when individual names fail (with per-name `missing_member` diagnostics).
- Function metadata parity: `import_function_metadata` covers `functions`, `function_defaults`, `function_varargs`, `function_workloads`, `generic_functions`, and `type_param_bounds` — equivalent to `mod_impl.rs:353-405`.
- Class metadata parity: `resolve_class` + `import_class_type_params` + `register_constructor` + `import_class_bounds` cover `class_types`, `class_declared_type_params`, `generic_functions` (when type params non-empty), `error_types`, constructor into `ctx.functions` (preferring `new` method params, otherwise fields), and method defaults/varargs/workloads — equivalent to `mod_impl.rs:410-495`.
- Constant metadata parity: `resolve_constant` covers `scope.define` and `const_integer_values` — equivalent to `mod_impl.rs:498-512`.
- Retained fallback: only registers into `ctx.functions` or `ctx.scope.define`, which matches the retained fallback path used elsewhere; retained intrinsics have no defaults/varargs/etc., so no coverage is being dropped.
- `local_name_for` for aliases matches the caller's `local_name_for` closure exactly.

**Expression predicate:**

- `is_supported_annotated_module_constant_expr` correctly rejects `HirExpr::Name` (default catch-all `_ => false`), preventing lowercase Rust identifier leakage.
- Recursive operators (UnaryOp/BinOp/Compare/BoolOp) validate operands, so `-pi` or `2 * pi` where `pi` is a name would also be rejected. `1.0 / 0.0` (only float literals) is accepted, which is what `_sifr.math`'s `inf`/`nan` need.
- Note: this is a strict expansion — previously only `ConstructorCall` was accepted for the non-integer branch, so user-code `x: float = 3.14` at module scope silently failed downstream ("undefined variable") and now succeeds. Not a regression, but a semver-visible expansion.

**Tests added:**

- `public_sysroot_stdlib_source_resolves_compiled_private_classes` — exercises constructor registration via the `PrivateThing(1)` call site (would fail without `register_constructor`).
- `user_source_cannot_import_compiled_private_constant` — confirms user code still hits `forbidden_intrinsic` even when the private module has compiled exports.
- `annotated_scalar_module_constant_name_alias_is_not_collected` — locks in the Name-rejection.
- `math_private_declarations_codegen_through_sifr_stdlib` (moved to its own file) — the golden e2e that ties `_sifr.math` compilation, `sifr.math` re-export, and codegen constant mappings together.

## Minor gaps (not blockers)

1. No test verifies that compiled private function *metadata* (defaults, varargs, workloads, generics, type-param bounds) is imported through the private-import path. The class metadata test exercises constructor registration, but the analogous function metadata paths in `import_function_metadata` are exercised only implicitly via the stdlib bootstrap.
2. No test covers alias handling in the compiled private path: `from _sifr.foo import bar as baz` — the `local_name_for` helper is correct, but a regression here wouldn't be caught.
3. Silently dropping non-integer Name aliases (`pi_alias: float = pi`) means a stdlib author who writes one gets a downstream "undefined variable" rather than a targeted diagnostic. Not incorrect, but a nice-to-have.

None of these rise to correctness bugs for M2a scope.

READY
