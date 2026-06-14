## Wave 2.0 Codegen Triage — Review Pass 2

Reviewed PR #2561 / branch `codex/wave-2-codegen-triage` and the pass-1 follow-up commit `60086d42c`. Diff against `main` is still artifact-only:

- `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md`
- `plans/issues/active/codegen-test-triage.md`
- `plans/reviews/active/ad-hoc-world-class-verification-wave-2-0-review-pass-1.md`
- `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`

No file under `crates/`, `scripts/`, `verification/runner/`, `Cargo.toml`, or `Cargo.lock` is touched. The "no compiler code changes" constraint is respected.

### Verdict

**Satisfied for Wave 2.0 merge.** All three pass-1 blockers are fully addressed and the inventory remains a 1:1 fit for the 52 failing `sifr_codegen` tests. Only optional polish items remain, listed at the bottom.

### Pass-1 blocker confirmations

- **B1 — `production-bug` taxonomy contradiction** → addressed. Rows 12 (`test_try_finally_runs_cleanup_before_timeout_propagates`), 29 (`test_generate_rust_generator_conditional_yield_preserves_else_branch`), and 37 (`test_self_field_clone_suppression_is_scoped_and_non_sticky`) are now `classification: "compiler-bug"`. `jq` confirms `production_bug_rows == 0` and the new total `compiler-bug` count is 10 (rows 12, 29, 37, 45, 46, 47, 48, 49, 50, 51). The triage markdown counts (`compiler-bug: 10`, `production-bug: 0`) and the phase-ledger Wave 2.0 Implementation Notes both match. The `classification_values` array still lists `production-bug` as a *legal* value — that is consistent with the phase decision text (the vocabulary, not the active assignment).
- **B2 — `closes_in_wave` vocabulary** → addressed. Every one of the 52 rows now has `closes_in_wave: 2` (integer) plus `closes_in_subwave` in `{"1","2","3","4","5"}`. The top-level `red_blocker` block carries `closes_in_wave: 2`, `closes_in_subwave: "final"`, and `expiry: "Wave 2.final"`. This matches the phase's matrix vocabulary and won't trip the Wave 10 negative self-tests. The triage markdown's `proposed_pr_slice` column is still rendered as compound `"2.1"`…`"2.5"` labels, but that field is now explicitly called out as a "human-readable repair slice label," not a matrix value — acceptable.
- **B3 — maintainer-local absolute path on row 52** → addressed. Row 52's `source_location` is now `crates/sifr_codegen/src/render/render_helpers.rs:302:5`, and I confirmed `fn renders_function_type_param_bounds` lives at that line. `jq` over the inventory confirms `abs_path_rows == 0` — no remaining `/Users/` or other absolute paths.

### Pass-1 follow-up status

- **F1 — shared parse-helper `source_location`s** → opportunistically fixed. Verified by grep:
  - Row 23 → `classes_and_basics_codegen_tests.rs:69` matches `fn test_fieldless_class_gets_default_constructor`.
  - Row 25 → `…:183` matches `fn test_guarded_non_option_compare_does_not_emit_some_wrapping`.
  - Row 26 → `…:52` matches `fn test_mut_on_local_nested_function_mutborrow_call_argument`.
  - Row 35 → `iterators_and_generators_codegen_tests.rs:224` matches `fn test_generate_rust_open_uses_canonical_filehandle_constructor`.
  - Row 4 → `async_control_codegen_tests.rs:96` matches `fn test_generate_rust_elides_unreachable_returns_after_always_exit_paths`.
- **F2 — 51-vs-52 FAILED status lines in the captured log** → unchanged but reconcilable. The local log still has 51 `… FAILED` lines (thread interleave on `test_production_lowering_contract_uses_result_helpers_only`), but its bottom `failures:` block has 52 entries and the result line is unambiguous: `test result: FAILED. 655 passed; 52 failed; 0 ignored`. The inventory is correct; only the line-by-line parity is fragile. Non-blocking.
- **F3 — slice 2.5 mixed classifications** → auto-resolved by B1.
- **F4 — stable `contract_id` per row** → not added. Pass 1 marked deferrable.
- **F5 — top-level matrix-row fields** → addressed. The `red_blocker` block now carries `triage_file`, `issue`, and `expiry`.

### Inventory 1:1 parity check

- `jq '.failures | length' == 52`.
- All 52 `id`s unique (`sort -u | wc -l == 52`).
- All 52 ids lexicographic and gap-free `codegen-red-0001`…`0052`.
- The captured `target/wave2/sifr_codegen_nocapture.log` failures: block has 52 entries; the inventory row order mirrors it.
- `cargo test -p sifr_codegen -- --nocapture` produced `655 passed; 52 failed; 707 total`, matching `test_result` in the JSON.
- Each row carries the five phase-required fields (`current_output`, `expected_output_or_snapshot`, `affected_compiler_contract`, `owner`, `closes_in_wave`) plus the optional `source_location`, `panic`, `reproduction_command`, `status`, `proposed_pr_slice`, `closes_in_subwave`. The inventory remains useful for downstream 2.1..2.5 repair PRs — the replacement/regression targets continue to read as actionable instructions.

### Non-blocking follow-ups (defer to Wave 2.x repair PRs)

- **N1 (carryover from F4).** Consider adding a stable per-row `contract_id` in the first 2.x PR that re-touches the inventory, so commit messages can cite it (e.g. `contract_id: integer-literal-rendering-normalized`).
- **N2 (carryover from F2).** Optionally re-run nocapture with `--test-threads=1` next time to make the log self-parable, or document the "trust the `failures:` block, not the per-test status lines" rule alongside the inventory.
- **N3 (new).** The post-review ledger note records that the create-pr validation lane finished with a passing report but the e2e process kept pipes open and required a manual terminate. This is currently treated as an artifact-only concern, but it's a latent CI hazard worth a separate tracking ticket — cold-cache CI runs may stall on the same handle leak. Not in scope for Wave 2.0.
- **N4 (new).** `classification_values` still lists `production-bug` as a legal value. With 0 rows using it across Wave 2, consider one of: (a) drop it from the schema with a short comment, or (b) keep it but add a routing note ("escalate to crashes sentinel area, not this inventory") to deter future authors from reaching for it. Schema choice, not a correctness issue.

### Recommendation

Approve and merge Wave 2.0 as the failure inventory it claims to be. Wave 2.1 (`2.1`: literal/render refresh, 36 rows mostly) can start on top of this PR immediately — it is the smallest and least risky slice and won't be blocked by N1–N4.
