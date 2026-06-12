Verdict: READY

The phase is implementation-ready and no hidden compatibility-call shim decisions remain.

Blocking gaps: none. The plan resolves every code path surfaced in the local scan:

- `resolve_python_compat_call_alias` (math/heapq/collections non-defaultdict) → removed in M2; `collections.defaultdict` and bare `defaultdict` short-circuits → removed in M3; transitional helper named.
- `resolve_bare_python_compat_call_alias` (deque/Counter) → removed in M2; bare `defaultdict` → M3 via `explicit_defaultdict_bindings`.
- `synthetic_imports` / `synthetic_import_aliases` / `ensure_synthetic_stdlib_import` and the `mod_impl.rs` consumer → producer and consumer removal both pinned in M2, with M4 grep guardrail.
- `mod_impl.rs` generic `Stmt::Import` unsupported diagnostic → replaced by lowering-owned `SIFR-IMPORT-0008` only when the module matches a stdlib tail.
- `call_builtins.rs` unconditional bare `defaultdict` → removed in M3, gated on explicit binding state.
- `class_field_inference.rs` bare deque/Counter and bare defaultdict inference → deque/Counter removed in M2, defaultdict gated on the explicit binding state in M3.
- `HirDiagnostic` lacking args → M1 adds `args` plus `LowerCtx::error_with_code_args_at` and frontend rendering thread-through; scalar-only `DiagnosticArg` handled by the comma-separated `imported_names` decision (with explicit empty-string rule for `Stmt::Import`).
- Driver `discovery` / `package_discovery` `ImportFrom` ownership and `Stmt::Import` exclusion → explicitly assigned, with duplicate-prevention rule.
- `compile_order.rs`, `query_diagnostics.rs`, `module_signatures.rs` → explicitly carved out as non-emitters.
- `STDLIB_SOURCES`-derived tail set with exact-then-root matching and `_sifr.*` exclusion → fully specified via shared `sifr_stdlib` helpers.
- `__compat_sifr_math/heapq/collections` codegen canonicalization and tests → M2 removal; retained async/task aliases carved out in Decision 8 with the generic `is_compat_stdlib_alias` guard intentionally kept.
- `__compat_defaultdict_*` → renamed to `__sifr_defaultdict_*` in M3, with M4 guardrail.
- Demos/e2e bare usage → M2 grep-driven classification across the three named directories.

Non-blocking observations:

- M4's first guardrail grep covers `math|heapq|collections` synthetic aliases but not bare `deque`/`Counter` synthetic generation. The producers (`resolve_bare_python_compat_call_alias`, `synthetic_imports`, `synthetic_import_aliases`) are already covered by the second M4 grep, so this is not a gap, just worth noting that bare-class compat regressions are caught via the producer grep rather than an alias-name grep.
- Decision 8 retains `__compat_sifr_sync_*` and `__compat_sifr_concurrent_*`; the renaming sweep that aligns `__compat_*` semantics with "compatibility paths only" is intentionally deferred. The plan states this explicitly, so no edit needed.
