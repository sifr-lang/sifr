I have enough context to render the review. Generated Rust matches what the refreshed assertions claim, inventory is internally consistent, and tracking docs align with the work.

# Wave 2.5 Codegen Red-Blocker Review

**Verdict:** approved with nits

## Blocking findings
None.

## Non-blocking nits

1. **Weak `__indexed_char` substring assertion** — `crates/sifr_codegen/src/lib_codegen_tests/structured_lowering_codegen_tests.rs:523`. `assert!(generated.rust_source.contains("__indexed_char"))` is redundant: the binding name already appears in the let-else pattern asserted at line 522, so this line cannot fail independently. The pre-Wave 2.5 form (`__indexed_char.to_string()`) carried information — the new form does not. Either drop it or replace with an assertion that the name is referenced in the post-guard tail expression (e.g. `__sifr_chars_text.get(... .map(|c| c.to_string()) else { ... }; __indexed_char`).

2. **Behavior delta worth a one-line note in the tracking doc** — `test_structured_stmt_path_handles_non_optional_string_index_return_expr`. The pre-Wave 2.5 codegen was `text.chars().nth(j as usize)`; the current codegen materializes a `Vec<char>` cache (`let mut __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();`) and then `.get(...)`. That is a real codegen-path change (not just literal spelling), and the test now silently validates the cache shape rather than the prior direct-iter shape. The triage doc summary ("only the assertions were stale") elides this. The safety contract (let-else, no unwrap) is genuinely preserved, so the stale-expectation classification is defensible — but the doc could honestly say "implementation moved to a Vec<char> cache while preserving the let-else safety guard."

3. **Unused `let mut`**, ditto. The materialized `__sifr_chars_text` is only read via `.get(...)`. Asserting `let mut` in the test bakes in a clippy/lint smell from the codegen. Non-blocking — the test must match what codegen emits — but worth filing an `unused_mut` cleanup against the structured-return char-cache emitter.

## Verification notes

Confirmed by direct inspection (no edits made):

- **Q1 — legitimacy of stale-expectation reclassification.** All four refreshed tests still assert their core contracts:
  - `test_try_finally_runs_cleanup_before_timeout_propagates` (`async_runtime_codegen_tests.rs:60-68`): keeps the `cleanup_pos < rethrow_pos` ordering check and `__sifr_try_finally_res` rethrow guard. The only diff is `1 as i64` → `1_i64` (a literal-spelling normalization tracked across Wave 2.1).
  - `test_generate_rust_generator_conditional_yield_preserves_else_branch` (`iterators_and_generators_codegen_tests.rs:147-160`): still asserts `} else {` and `_yields.push(i);` inside the conditional region. New `i += 1_i64;` reflects an augassign normalization in codegen, not a behavior loss.
  - `test_self_field_clone_suppression_is_scoped_and_non_sticky` (`iterators_and_generators_codegen_tests.rs:618-629`): both **negative** assertions (`!self.items.clone().push(x)`, `!self.table.clone().get("k")`) are preserved — these are the load-bearing checks for clone suppression. Positive assertions relaxed from `return X;` to `X` substrings, consistent with the tail-expression contract.
  - `test_structured_stmt_path_handles_non_optional_string_index_return_expr` (`structured_lowering_codegen_tests.rs:516-527`): still proves a let-else (no `.unwrap()`, no panic), still asserts `lowering_stats.stmt_structured >= 1`. See nit #2 above.

- **Q2 — contracts preserved.** Cleanup-before-rethrow ✓, generator else preservation ✓, no `self.table.clone()` with `.copied()` lookup ✓, label clone ✓, compiler-verified string-index guard ✓.

- **Q3 — inventory consistency.** `jq` check confirms: 52 total rows, all `status: closed`; `red_blocker.status: closed`, `failure_count: 0`, `test_result: 708/0/708`; slice histogram `2.1:20, 2.2:16, 2.3:6, 2.4:6, 2.5:4 = 52`; classification `stale-expectation:40, obsolete-test:6, compiler-bug:6 = 52`. Wave 2.5 rows 0012/0029/0037/0045 all closed and reclassified to `stale-expectation`. Triage doc counts match (`stale-expectation: 40`, `compiler-bug: 6`, `obsolete-test: 6`).

- **Q4 — tracking docs.** Wave 2.4 PR link verified against `git log` (`94a238b21 Merge pull request #2565`). Result progression in `codegen-test-triage.md` (52→32→16→10→4→0 failed) is self-consistent. The phase-plan Wave 2.5 Implementation Notes correctly mirror the inventory state. Minor honesty gap noted in nit #2.

- **Q5 — readiness.** Code is mergeable. The four reclassifications are honest (test changes are pure assertion refreshes; no production code was touched to make tests pass), validation evidence is complete, and the red-blocker gate is consistently closed across all three artifacts.
