## Wave 2.0 Codegen Triage — Review Pass 1 Findings

Reviewed PR #2561 / branch `codex/wave-2-codegen-triage`. The diff is artifact-only (`plans/issues/active/codegen-test-triage.md`, `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md`, and `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json`). No compiler code is touched, so the Wave 2.0 "inventory only" constraint is respected.

### Verdict

**Not yet mergeable.** Three blocking issues and several non-blocking follow-ups below. The inventory is the right shape and the test-by-test classifications are mostly defensible, but the taxonomy contradicts the phase's own definitions in one place, the per-row vocabulary deviates from the matrix vocabulary the phase mandates, and one source-location entry leaks a maintainer-local absolute path into a checked-in artifact.

### Confirmation against the review checklist

- **Failure coverage 1:1.** The JSON inventory has exactly 52 rows (`codegen-red-0001`…`0052`), all in lexicographic order, and each matches a row in `plans/issues/active/codegen-test-triage.md` numbered 1–52 in the same order. The four classification counts add up (36 stale-expectation + 6 obsolete-test + 7 compiler-bug + 3 production-bug = 52) and match the totals stated in both artifacts.
- **Per-row required fields present.** Every row carries the five fields the phase requires for this inventory: `current_output`, `expected_output_or_snapshot`, `affected_compiler_contract`, `owner`, and `closes_in_wave`. Optional fields (`source_location`, `panic`, `reproduction_command`, `status`, `proposed_pr_slice`) are present uniformly.
- **Ledger does not overstate.** Wave 2.0 Implementation Notes accurately describe the status as "inventory drafted locally; PR/review/merge pending" and only claim local validation, not merge readiness. Wave 2.final remains explicitly future-dated. No claim of compiler fix or merge promotion has been smuggled in.

### Blockers (must fix before Wave 2.0 merges)

**B1. `production-bug` classification contradicts the phase decision text.** The phase document decides:
- *real compiler bug → fix root cause + regression in repair PR;*
- *unresolved production bug → not fixed in this phase, escalate to issue-linked sentinel.*

The inventory assigns rows 12 (`test_try_finally_runs_cleanup_before_timeout_propagates`), 29 (`test_generate_rust_generator_conditional_yield_preserves_else_branch`), and 37 (`test_self_field_clone_suppression_is_scoped_and_non_sticky`) to `production-bug`, but `proposed_pr_slice: "2.5"` and the row's replacement target ("Fix generator lowering…", "Fix scoped clone suppression…", "Add regression proving finally cleanup runs…") describe a *fix*, not a sentinel. By the phase doc these are `compiler-bug` (real, user-visible, to be fixed). Either reclassify all three as `compiler-bug` or move them to `verification/areas/regression/crashes` (or equivalent) as sentinels with issue links. Leaving the taxonomy at odds with the repair plan will confuse every reviewer of Wave 2.5.

**B2. `closes_in_wave` uses compound `"2.1"`/`"2.2"`/…/`"2.5"` values, contrary to the matrix vocabulary the phase mandates.** The phase decision text reads: *"Each row carries a `closes_in_wave` field naming exactly one wave in 1-9. Subwaves are expressed via `closes_in_subwave` … the matrix check rejects unknown wave or subwave names."* The per-failure rows should set `closes_in_wave: 2` and `closes_in_subwave: "1"` (or `"2"`, … `"5"`). The top-level `red_blocker.must_be_executed_by: "Wave 2.final"` has the same problem; the prior convention used by the same phase elsewhere is `closes_in_wave: 2` + `closes_in_subwave: "final"`. Either bring the inventory into compliance now or add an explicit exemption clause to the phase decisions doc — silent divergence will break the Wave 10 negative self-tests that the closeout list already promises.

**B3. `codegen-red-0052.source_location` leaks an absolute, maintainer-local path into a checked-in artifact.** Current value: `"/Users/yaseralnajjar/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/insta-1.47.2/src/runtime.rs:719:13"`. That is the insta runtime panic site on the maintainer's machine. The actual failing test is at `crates/sifr_codegen/src/render/render_helpers.rs:302` (`fn renders_function_type_param_bounds`). Replace with a repository-relative test location. Beyond the privacy aspect, this entry cannot be reproduced or `cargo test`'d from another developer's checkout or from CI as written.

### Non-blocking follow-ups

- **F1. Four `source_location` entries point at a shared parse-helper panic site rather than the failing test.** Rows 23, 25, 26 (and earlier row 4) all read `crates/sifr_codegen/src/lib_codegen_tests/async_control_codegen_tests.rs:8:39`; row 35 reads `…:9:49`. The actual tests live in `classes_and_basics_codegen_tests.rs` and `iterators_and_generators_codegen_tests.rs`. Example confirmed: `test_fieldless_class_gets_default_constructor` is at `classes_and_basics_codegen_tests.rs:69`, not `async_control_codegen_tests.rs:8`. This makes the inventory less useful for the 2.x repair PRs without changing the classification.

- **F2. Wave 2.0 ledger parity claim is slightly stronger than the log can prove.** The log (`target/wave2/sifr_codegen_nocapture.log`) has 51 `... FAILED` lines and 654 `... ok` lines, because thread-interleaved panic output ate the suffix on `test_production_lowering_contract_uses_result_helpers_only`'s status line. The test result summary says `655 passed; 52 failed`, so the 52-row inventory is correct; but a strict line-by-line parity tool would miss one. Worth either documenting the interleaving-tolerant parity rule, or rerunning with `--test-threads=1` so the captured log matches counts on its face.

- **F3. PR slice 2.5 mixes `compiler-bug` (row 45) with `production-bug` (rows 12, 29, 37).** The triage doc's slice 2.5 description ("user-visible generated-Rust semantic defects") fits both, so the slice is fine, but the per-row classifications still need to be internally consistent — resolving B1 will tighten this automatically.

- **F4. `affected_compiler_contract` reuses short labels** ("source fixture validity", "production-source architecture guard", "async fixture policy"). Acceptable for grouping, but Wave 2.5 will want a stable per-row contract id that can be promoted to a regression assertion. Consider adding `contract_id` per row as a follow-up so 2.x PRs can cite it in commit messages.

- **F5. Top-level `red_blocker` block does not yet carry the matrix-row required fields** (`triage_file`, `issue`, `expiry`). The phase decisions doc requires them for `red-blocker` rows. If those live elsewhere (e.g. `compiler_surface_matrix.json`), a one-line `triage_file: "plans/issues/active/codegen-test-triage.md"` cross-link inside this artifact would still help review.

### What is already good

- The 1:1 mapping between the triage markdown and the JSON, both ordered identically, makes diff review tractable.
- Including `panic` strings verbatim per row is a small but real win for the next reviewer who wants to confirm a classification without rerunning cargo.
- The proposed PR slices (2.1 literal/render refresh, 2.2 fixture validity, 2.3 obsolete architecture guards, 2.4 internal lowering, 2.5 user-visible) are coherent and small enough to land as separate reviewable PRs. Replacement/regression targets read as instructions a Wave 2.x author can act on without re-deriving the bug.
- The diff respects the Wave 2.0 "no compiler code changes" constraint exactly.

### Recommendation

Address B1 + B2 + B3 in the same revision, then this PR is ready for a pass 2. F1 and F2 can be folded into the same revision opportunistically since they are one-line-each fixes; F3–F5 are fine to defer.
