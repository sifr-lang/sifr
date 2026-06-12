I've read the plan, execution doc, gap audit pass 1, and verified against the actual source. Now the brutal verdict.

# READY

All pass 1 blockers are now locked. Spot-checked each decision against the code that has to change:

| Pass 1 blocker | Pass 2 resolution | Verified against |
|---|---|---|
| §1 args plumbing impossible | "Diagnostic Transport" subsection: add `args` field to `HirDiagnostic`, `LowerCtx::error_with_code_args_at`, thread through `query_diagnostics.rs` | `sifr_ir/src/diagnostic_types.rs:6` confirms the field is missing today; `query_diagnostics.rs:117-148` already supports arg passthrough on top of `"{message}"`, so the template question is non-blocking |
| §2 layer ownership undecided | "Layer Ownership" subsection splits project/package `ImportFrom`→discovery, single-file `ImportFrom`→lowering, all-mode `Import`→lowering | `discovery.rs:589-628` already `return Err(...)` short-circuits before lowering, making the rule mechanically realizable |
| §3 `Stmt::Import` asymmetry | Explicit: lowering owns `Stmt::Import` in `mod_impl.rs`; discovery intentionally silent; frontend collectors carved out | `mod_impl.rs:611-620` is the named site; carveout text covers `query_diagnostics.rs` + `module_signatures.rs` |
| §4 duplicate prevention | "Duplicate prevention rule" paragraph + discovery fail-fast | Confirmed by `parse_import_closure_source_modules` early return |
| §5 probe-then-diagnose | "Bare module names ... are rejected ... only after normal top-level user/package resolution fails" + project mode probes workspace candidates first | Matches existing `Unresolved` path |
| §6 tail matching | Exact-tail-first then leading-root, with concrete `is_bare_stdlib_tail` helper and three worked examples | Implementable from `STDLIB_SOURCES` |
| §7 today-state framing | "Current bare stdlib import diagnostics are inconsistent" table in Context | Matches actual emission sites |
| §8 M1 test scope | M1 enumerates project/package/single-file × `Import`/`ImportFrom`, plus harness arg registration | Matches `diagnostic_contract_harness.rs:30-75` shape |
| §9 cross-layer sequencing | M1 explicitly owns discovery+package+lowering+frontend rendering | Single milestone, no orphan |
| §10 smaller items | `explicit_defaultdict_bindings` named, M4 grep extended to `sifr_type_system`, deque/Counter transition stated | All present |

## Non-blocking nits (not gating)

These are clarity improvements I'd accept post-implementation, not blockers:

- `crates/sifr_driver/src/project/compile_order.rs:22,58` walks `Stmt::ImportFrom` only and filters by `local_modules` — never emits diagnostics, never contributes bare-stdlib edges. The "no change needed" carveout only names the two frontend collectors; `compile_order.rs` rides the same reasoning but isn't explicitly listed. Trivial to derive.
- Single-file `Stmt::ImportFrom` patch site isn't pinpointed to `mod_impl.rs:460-468` / `import_diagnostics::unknown_import_target`, but "the import resolution path" is unambiguous given the only relevant site is right there.
- `imports::report_unknown_stdlib_module` for unknown `sifr.*` modules (e.g. `sifr.selectors`) is implicitly retained — Non-Goals and the SIFR-IMPORT-0008 scope (bare names only) make this clear, but the plan doesn't restate it.
- `imported_names` arg semantics for `import math` (empty) vs `from math import sqrt as s` (alias preservation) is left to the implementer.
- "Add an `args` field … or an equivalent typed diagnostic payload" leaves the exact shape to the implementer; acceptable design latitude.

None of these block M1. Proceed.
