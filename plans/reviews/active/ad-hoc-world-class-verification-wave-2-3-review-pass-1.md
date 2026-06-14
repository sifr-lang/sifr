## Wave 2.3 External Review

**Verdict: No blockers. Wave 2.3 is approved to open the PR.**

### Findings

All changes are architecture-guard text retargets, not compiler-behavior changes. Each new guard target was verified against live source.

| # | Severity | Location | Note |
|---|---|---|---|
| 1 | nit | `crates/sifr_codegen/src/intrinsic_method_emitters/narrowing_helpers.rs:41-46` | `prod_src` joins only the 3 named files. The negative-string assertions (`lower_registry_expr_with_string_path`, `render_expr_via_string_only`) only catch reintroductions inside those 3 files; the original guard scanned the whole parent file. Scope is narrower but matches the new helper ownership boundary — acceptable for an obsolete-test retarget. |
| 2 | nit | `crates/sifr_codegen/src/lib_codegen_tests/structured_lowering_codegen_tests.rs:585-588` | `emit_stmt` wrapper end is located via `find("\n    }\n}")`, i.e. assumes `emit_stmt` is the last method in its impl block (currently true at `lib_emitter_state.rs:810-836`, impl closes at line 837). If a method is appended after `emit_stmt`, this guard could pick a too-large slice. Not a blocker; consider replacing with a `\n    pub(crate) fn ` next-fn search later. |

No correctness, security, or behavior concerns found.

### Review-question answers

1. **Closes 6 rows without hiding compiler bugs?** Yes. All 6 closed rows are `classification: obsolete-test`. The diff only updates `include_str!` targets and asserted-symbol names to follow the post-decomposition module layout (`collection_methods.rs`, `recursive_exprs.rs`, `expr_render_helpers/field_and_stdlib_rewrites.rs`, `lower_expr/leaves_and_plain_calls.rs`, `lib_emitter_state.rs`, `stmt_support_emitter/statement_output.rs`, `entrypoints.rs`). No compiler source touched; no failing test silenced beyond its retargeted scope. The 10 remaining open rows are all `classification: compiler-bug`, so the Wave 2.4/2.5 compiler-bug surface is preserved.

2. **Are new guards meaningful?** Yes. Verified each guard's referenced text actually exists in the named owner module (`pub(crate) fn try_lower_registry_expr_strict/_exprs_strict` in `collection_methods.rs:641,652`; `_recursive` in `recursive_exprs.rs:7`; `_result` in `field_and_stdlib_rewrites.rs:265`; `fn emit_stmt` wrapper in `lib_emitter_state.rs:810`; `try_lower_simple_stmt_with_scope_result_and_bindings` in `lib_emitter_state.rs:383`; generator-init missing strings in `statement_output.rs:11,25`; `try_lower_leaf_expr_result` in `leaves_and_plain_calls.rs:80`; `if !emitter.body_items.is_empty()` in `entrypoints.rs:96`). The negative assertions on `lib.rs` (now 95 lines) all hold — no banned tokens present.

3. **Inventory internally consistent?** Yes. `red_blocker.failure_count = 10`, `test_result = 697/10/707`, exactly 6 rows with `closes_in_subwave: "3"` flipped to `status: closed`, and the 10 open rows split 6× Wave 2.4 + 4× Wave 2.5, all `classification: compiler-bug`. Matches the user-reported `jq` snapshot exactly.

4. **Docs accurate for closure evidence?** Yes. `plans/issues/active/codegen-test-triage.md` records the 691→697 / 16→10 transition and the Wave 2.2 PR (#2563). `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md` Wave 2.3 Implementation Notes capture scope (6 rows), the retargeted modules, the validation command set, the two transient core-language audit timeouts, and the `create-pr` 197.18 s wall time. The phase-status header is consistent with Wave 2.2 merged + 2.3 in progress.

5. **Blockers before PR?** None.
