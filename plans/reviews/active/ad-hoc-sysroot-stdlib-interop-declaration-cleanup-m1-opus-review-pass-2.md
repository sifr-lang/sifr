## M1 pass 2 review — no unresolved actionable correctness issues

Both pass 1 findings are correctly addressed and I found no new correctness defects.

**Pass 1 finding 1 (async ellipsis stubs → ASYNC_NO_SUSPEND) is resolved.** In `annotations_and_function_lowering.rs:533-605` the `rust_interop` collection and `classify_rust_interop_stub_body` now run before any body-based async check. All four body-scanning async checks are gated on `!stub_body.skips_normal_body_lowering()`:
- ASYNC_NO_SUSPEND lookup (line 551-566)
- `first_yield_range_in_stmts` + mutable-borrow-across-yield (line 567-590)
- `first_await_range_in_stmts` inside the generator branch (line 583)
- `first_await_range_in_stmts` for mutable-borrow-across-await (line 591-604)

`is_async_generator` is also short-circuited to `false` for stubs (line 547-549), so nothing downstream re-enters body-based analysis. Regression coverage in `rust_interop_accepts_ellipsis_only_async_function_stub` locks this in.

**Pass 1 finding 2 (malformed @rust + ellipsis double-report) is resolved.** `has_rust_interop_decorator_syntax` (`rust_interop.rs:68-72`) walks decorator expressions by namespace name via `starts_with_rust_namespace`, so it returns true whenever the decorator sits in the `rust` namespace regardless of whether arg parsing succeeded. `classify_rust_interop_stub_body` now consumes that flag and suppresses the "non-interop ellipsis" secondary whenever a rust-namespace decorator is present — the primary malformed diagnostic from `collect_rust_interop_declarations` stands alone. Regression coverage in `rust_interop_does_not_report_non_interop_ellipsis_for_malformed_rust_decorator` locks it in.

**Cross-checked for regressions from the pass 2 fixes:**
- All four function-like paths (top-level, nested `Stmt::FunctionDef`, enum method, newtype method, regular class method) call `classify_rust_interop_stub_body` with the same `has_rust_interop_decorator_syntax` gate and skip body lowering via `RustInteropStubBody::skips_normal_body_lowering()`.
- All four paths preserve `return_type` from the annotation (top-level/nested via `ft.return_type.as_ref().clone()`, class/enum/newtype via `resolve_annotation_expr` on `func.returns`) before body lowering runs, so `Result[T, E]` metadata survives for downstream direct-interop mapping.
- Non-stub body-scanning code paths that still read `&func.body` (`collect_declared_nonlocals`, `function_body_contains_yield`, `hir_body_calls_function` against the empty HIR body) are all safe on a single ellipsis expression statement — they either return empty results or are downstream-gated by `stub_body`.
- `Expr::EllipsisLiteral` in `core_and_calls.rs:46-54` still catches ellipsis leaking into expression positions (assignment RHS, nested-if branches, etc.), covering the "ellipsis in a non-stub statement context" gap the classifier's top-level-only walk doesn't check.
- No duplicate diagnostics: malformed-decorator cases suppress the ellipsis secondary, and mixed-body cases emit only the `RUST_CONFIG_MALFORMED_DECORATOR` "exactly one ellipsis statement" diagnostic before skipping body lowering.

Nothing to flag; M1 is ready to close from a lowering-correctness standpoint.
