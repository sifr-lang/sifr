Both clean. All verification is complete — here is the consolidated final report.

---

# Whole-Phase Closure Review — Declaration-First Python Interop (PR #3000), Pass 6

**Fresh read-only review of committed HEAD `4bceedf18`** (branch `codex/m17-phase-closure`, base `origin/main`, 127 files, +2027/−499), covering the post-pass-5 IR-hygiene remediation, all pass-5 closure conclusions, and the complete M0–M17 diff. Method: full line-by-line review of the two commits added since the satisfied pass-5 tree (`4bceedf18` code remediation, `cefacb5e6` review-artifact/plan bookkeeping); exhaustive static enumeration of `RustExpr::Ident` construction sites in `sifr_codegen`; live compile/run probes with a debug compiler verified current at this tree; focused and corpus-wide test suites re-run at HEAD; spot re-verification of every load-bearing pass-5 claim. No repository files were modified; all probes ran in `/tmp`. No subagents were used.

## 1. Structural completeness of the remediation (required check 1)

The merge-gate failure mode was: strict assembled-IR validation (`ir_validate.rs:257-269`, a hard `assert!` at `lib_modules_and_codegen.rs:710-719` and `entrypoints.rs:160-169`) rejects any `RustExpr::Ident` that is not a plain identifier, while several lowering routes still built namespaced call targets (`Point::origin`) as raw `Ident` strings. Commit `4bceedf18` adds `plain_call_target_for_ir` (`expr_call_metadata.rs:14-19`: `::` → `Path(split)`, else `Ident`) with a focused unit test, and routes five sites through it.

I enumerated **all 1269 `RustExpr::Ident` constructions** in non-test `sifr_codegen` source and classified every one that can receive a dynamic string. Every reachable plain/namespaced call-target construction now handles `::` structurally:

- **Registry signature calls** — `plain_call_args.rs:290` → helper (fixed).
- **Registry recursive calls** — `recursive_exprs.rs:68` → helper (fixed); the iterator-call target at `:50` takes only fixed names from `registry_iterator_op_func_name` (`iter`/`next`/`map`/…).
- **Simple awaited calls** — `leaves_and_plain_calls.rs:445` → helper (fixed).
- **Timeout-aware await adapter** — `await_and_async_comprehension.rs:41` → helper (fixed).
- **Structured TaskScope/TaskGroup spawn call arguments** — `iterators_and_callables.rs:397` → helper (fixed).
- **Main statement-level plain-call macro** (`expr_call_and_literal_helpers.rs:733-740`) and **print-call route** (`print_calls.rs:82-88`) — already `Path`-split on `::`; unchanged and correct.
- **`try_lower_simple_call_expr`** (`leaves_and_plain_calls.rs:742`) — explicitly returns `None` for `::` targets at line 722, deferring to the structured emitter; the eight other `Ident(func)` sites in that function fire only on fixed `__sifr_*` intrinsic names.
- **Constructor targets** — `Path` everywhere (`recursive_exprs.rs:301,318`; both `ctor_func` error-coercion sites in `plain_call_args.rs:183-190` and `stmt_expr_method_and_question_mark.rs:401-408` are `::`-aware `Path` builders); `parse_identifier_path_expr` validates each segment as a plain ident before choosing Ident/Path.
- **All remaining dynamic-`Ident` sources** are HIR variable/parameter names or compiler-synthesized `__sifr_*`/`__e`-style temporaries. Namespaced names are synthesized in exactly one lowering site (`method_argument_ownership.rs:166`, `format!("{class_name}::{method_name}")`) and only into `HirExpr::Call.func` — `class_name` is a Python identifier (`name.id`), so no generic/call/field syntax can enter. `HirExpr::Name` cannot carry `::`: an uncalled method reference is rejected at type-check (live-verified: `f = Point.origin` → `SIFR-TYPE-0012`).
- **Renderer parity**: the old Ident-with-`::` route and the new `Path` route render identically (`render_identifier` is a no-op for non-plain strings; both funnel into `render_compiler_path_string`), so no output change beyond validation acceptance. Keyword-escaped locals are unaffected — escaping happens at render time, never in Ident payloads (live-verified: locals named `loop`/`struct` compile and run).
- **`Verbatim` nodes** are syn-parse-validated; every dynamic `Verbatim` in the tree is a debug-formatted literal.

The remediation is structurally complete. No remaining reachable `RustExpr::Ident` can receive raw `::`, generic, call, or field syntax.

## 2. Merge-gate failures and analogues fixed, no regressions (required check 2)

- `classmethod_basic.sifr`, `cls_constructor.sifr`, `staticmethod_basic.sifr` all **compile and run cleanly** at HEAD with exit 0; emitted Rust shows structured `Point::origin()`.
- A fresh async analogue probe (staticmethod/classmethod `async` declarations invoked as a plain await, inside `async with task.timeout(...)`, and as a `scope.spawn(...)` argument) **compiles through strict validation and runs to completion** — covering the awaited, timeout-aware, and structured-task-argument routes the commit touched.
- `test_emit_pass_fixtures_do_not_include_unwrap_or_expect` **passes at HEAD** (56.6s), pushing all 674 pass fixtures through codegen with the strict assembled-IR assert — corpus-wide confirmation.
- Full `sifr_codegen` suite: **890/890 pass**, including the new helper test. `cargo fmt --check` and `cargo clippy -D warnings` are clean on the touched crate; the maintainability guardrail passes; no touched file exceeds 900 lines.
- The `recursive_exprs.rs` change is semantics-preserving (it centralizes the identical Path/Ident split that branch previously inlined).

## 3. Pass-5 conclusions recheck and independent diff sweep (required check 3)

The delta since the pass-5 tree is exactly the two commits above; both are fully reviewed. I additionally re-verified pass 5's load-bearing conclusions directly at HEAD:

- **NB-1** nested declarations: hard `SIFR-PYCALL-0001` at `statement_dispatch.rs:684-692`; live-reproduced.
- **NF-1** wrapped decorators: `@python(math.sqrt)()` and the nested wrapped form both fail `SIFR-PYCALL-0001` live; the plain ellipsis declaration control (`Result[float, PythonError]` with the `sifr.python` import) passes `check` with no errors — no over-rejection.
- **RuntimeFault shadow**: `cause_variant` (`outcome.rs:3-19`) and the sync classifier match only canonical `sifr.*` identities.
- **NM-2 canonical worker identities**: all four synthesis sites stamp `Some("sifr.parallel.{name}")` (`parallel_calls.rs:278`, `task_calls.rs:216`, `task_scope_offload_calls.rs:167,291`, `task_join_set_calls.rs:471`).
- **AM-9 IR hygiene**: `ir_validate.rs` diff reviewed in full — strict Ident validation, syn-validated Verbatim statement/expression nodes, negative unit tests present.
- **AM-6 DLPack**: `argument.rs` disarm-before-attach/re-arm-on-loop-thread reviewed in full; matches the double-free-closure description.
- The pass-5 review artifact itself is now committed (`cefacb5e6`), and the plan's new status paragraph (lines 2676-2684) accurately describes the merge-gate exposure, the helper, the routed sites, and the test evidence — all of which I independently confirmed.

The remaining 120-odd diff files are unchanged since pass 5's satisfied full sweep; my sampling of the non-codegen areas (runtime callbacks, binding authoring, diagnostics catalog, docs) found nothing contradicting it. **No new blocker, major, or actionable minor.**

## 4. Pending procedural closure-unit steps (not defects — required check 4)

The plan's at-merge instruction (lines 2686-2690) is internally consistent; these remain explicitly planned, not findings:

1. Persist this satisfied pass-6 report (the untracked `…pass-6.md` is an empty placeholder — this review's own artifact slot) and record it plus exact merge-gate evidence in the plan.
2. Run the authoritative **merge-profile gate on this exact tree** (the last full merge gate predates `4bceedf18`; my corpus-wide emit-validation, 890-test, fmt/clippy/guardrail runs at HEAD are corroborating but not the authoritative gate).
3. Check M17 Wave 4, flip status to `completed`, update PY-2 roadmap / phase index / architecture summary naming PR #3000, archive the plan file, merge.
4. Working-tree hygiene: `third_party/ruff` still carries the same uncommitted whitespace-only one-hunk reformat pass 5 flagged (committed submodule pointer unchanged) — revert or fold before the exact-tree gate.

## 5. Optional / out-of-scope observations (non-blocking — required check 5)

- **Pre-existing core-language keyword-method gap, worth filing separately** (not introduced or touched by this phase): a user method named after a Rust keyword (e.g. `def match`) emits an escaped definition (`fn r#match`) but unescaped call sites (`m.match(...)`, `Matcher::match(...)`) — `MethodCall` renders `{method}` raw and `Path` segments are unescaped — producing a rustc failure surfaced as `SIFR-BUILD-0005`. Verified identical for instance methods (a route this diff never touches); the render lines are unchanged across the whole phase diff. Analogous in status to the core-language try/finally divergence pass 2 recorded.
- `ir_validate` checks `Ident` expressions but not `MethodCall.method` strings, which legitimately carry turbofish (e.g. `collect::<HashSet<_>>`) — pre-existing design, syn-safe at render; hygiene-hardening candidate only.
- Pass-5's carried observations remain accurate and non-blocking: user-defined `WorkerRuntimeError` evidence-label precision, cosmetic preamble lines in the pass-1/2 artifacts, the M17 checkbox/Wave-4 disambiguation resolved by the at-merge instruction, and the NF-1 subscript-case unit-coverage polish.

## Conclusion

The IR-hygiene remediation is structurally complete: every reachable plain/namespaced call-target construction either uses the shared `plain_call_target_for_ir` helper, was already `Path`-structured, or is provably fed fixed plain identifiers; no `HirExpr` channel can smuggle raw syntax into an `Ident` payload. The three merge-gate fixtures and their async/structured analogues compile through strict validation and run correctly with no behavior regression (renderer parity confirmed, 890/890 codegen tests, corpus-wide emit validation over all 674 pass fixtures, fmt/clippy/guardrails clean). Every pass-5 closure conclusion re-verified at HEAD; the independent sweep of the complete M0–M17 diff found no blocker, major, or actionable minor. What remains is exclusively the explicitly planned closure procedure: exact-tree merge-profile gate (after folding the submodule whitespace hunk), bookkeeping flip, archival, and merge.

VERDICT: SATISFIED
