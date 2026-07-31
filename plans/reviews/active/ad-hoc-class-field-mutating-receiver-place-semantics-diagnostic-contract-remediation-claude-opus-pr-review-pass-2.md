Read-only review pass 2. No files modified, no builds/tests/benchmarks run.

---

# Focused Diagnostic-Contract Remediation Review — Pass 2

**Head inspected:** `12e6ebbb23e0a967dc8544b6c6d9afb116e56b89`
**Diff scope:** `origin/main..HEAD` = 2 commits (`87e35414b`, `12e6ebbb2`), 13 files, +711/−101.

## Pass-1 findings verified closed

- **BLOCKING 1 (PROTO-0006 identity + randomized order) — closed in code.** `class_name` is now in the registry entry (`crates/sifr_diagnostics/src/codes/registry/registry_entries/calls_flow_and_protocols.rs:429-432`: template, `declared_args`, `dedupe_args`), populated at the single emitter (`crates/sifr_lowering/src/lower/method_receiver_diagnostics.rs:12-36`), and reflected in `internal_docs/diagnostic_codes.md:222` and the phase plan (`plans/issues/active/…:669-671`). `docs/errors/SIFR-PROTO-0006.mdx` does not embed templates or arg lists, so no regeneration is owed. The `HashSet` iteration is gone: `validate_fixed_receiver_mutations` collects `FixedReceiverMutation` records and sorts by `(range.start(), class_name, method)` before emitting (`method_receiver_analysis.rs:227-256`), which is total and deterministic even when `method_source_ranges` misses an entry (`unwrap_or_default()` ties break on class/method).
- **Six-violation recovery survival — closed.** With `class_name` in `dedupe_args`, `recovery_dedupe_args` (`crates/sifr_driver/src/diagnostics.rs:268-283`) yields six distinct `SimilarDiagnosticGroupKey`s, so each group holds one diagnostic and `MAX_SIMILAR_DIAGNOSTICS_PER_GROUP` never trips (`:173-186`). `test_fixed_receiver_diagnostics_survive_similar_recovery_cap` (`crates/sifr/tests/e2e_support/e2e_entrypoints.rs:404-467`) asserts exactly six `class_name` values through the real `apply_diagnostic_recovery_limits`; an omission summary would shrink the vector and fail the assertion, so absence of the summary is genuinely pinned.
- **OWN-0005 phase-owned path — closed.** `immutable_parameter_field_receivers_report_root_binding_argument` (`method_receiver_diagnostics_tests.rs:191-228`) covers both the borrowed and `own` parameter field-receiver shapes reaching `report_immutable_root` and asserts `binding == "owner"`. The type/place leak is gone: `report_immutable_root` (`method_receiver_places.rs:587-600`) no longer feeds `place_display` into `binding`; the fallback routes to `SIFR-OWN-0014` with `place`. I confirmed the fallback is in fact unreachable on this path — `InvalidPlace::ImmutableParameter` is only returned after `extract_place` succeeded and `retained_binding(place.root)` was `Some` (`:359-385`), and `extract_place` only accepts `Name{binding_id: Some}` / `FieldAccess` (`:398-427`), a strict subset of `root_binding_id` (`:633-641`). The defensive branch is therefore dead but contract-safe rather than wrong.
- **Centralization — closed.** Both PROTO emitters live in `method_receiver_diagnostics.rs` (66 lines, single responsibility), and `method_receiver_analysis.rs` now threads named `FixedReceiverMutation` / `ProtocolReceiverMismatch` records (`:18-30`) instead of the five-element tuple and pre-formatted message string. Helper messages match the registry templates byte-for-byte.
- **Guardrail hardening — closed.** `e2e_entrypoints.rs:341-401` now asserts `DiagnosticState::Active` (`:363-368`), iterates *every* diagnostic of each code rather than `.find` (`:383`), asserts key presence, and rejects empty/whitespace `DiagnosticArg::String` values (`:392-398`). All API used is public (`registry.rs:309,371,373,376,857`). The scope decision is recorded in the test (`:346-348`) and in the plan with named ownership (`plans/issues/active/…:691-697`: 120 gaps, 11 families, `diagnostics` owner).
- **Plan record — closed.** Pass-1 findings and responses are in the ledger (`plans/issues/active/…:1161-1176`) and both new `crates/sifr` tests are named explicitly in the validation block (`:1180-1191`).

No new correctness, dedupe, or maintainability defect in the Rust changes; all touched files are under the cap (largest: `e2e_entrypoints.rs` 829, `method_receiver_places.rs` 703, `method_receiver_analysis_tests.rs` 796).

---

## BLOCKING 1 — The PROTO-0006 message change leaves a stale checked-in diagnostics baseline, so the authoritative merge gate fails

Commit `12e6ebbb2` changed the rendered message to prepend `class '{class_name}' `, but the diagnostics verification area still holds the old text:

- `verification/areas/diagnostics/fixtures/diagnostics/e2e_operator_receiver_mutation_rejected/baselines/check-compact.stderr.txt:2`
  `E SIFR-PROTO-0006 …/main.sifr:6:9 method '__eq__' cannot mutate its receiver because Rust trait 'PartialEq' fixes the receiver convention`
- Fixture (`…/e2e_operator_receiver_mutation_rejected/main.sifr:3-8`) is `class Counter` with a mutating `__eq__`, so the emitter now produces `class 'Counter' method '__eq__' cannot mutate…`.
- The comparison is exact string equality on normalized stderr: `verification/runner/sifr_verify/area_adapter.py:600-606` (`if stderr_norm != stderr_file.read_text(...): mismatches.append("stderr")`). No fuzzy or code-only matching.
- The case is registered in the `baselines` suite: `verification/areas/diagnostics/manifest.json:1283-1290` (`command: "check"`, `diagnostic_formats: ["compact"]`).

**Failure scenario (deterministic, not probabilistic).** `scripts/run_all_tests.sh` with the default merge profile runs the diagnostics area with suites `["rules", "baselines"]` (`verification/profiles/merge.json:157-161`; also nightly/release). The `baselines` case for `e2e_operator_receiver_mutation_rejected` compiles the fixture, gets the new message, compares against the stale baseline, and reports a `stderr` mismatch — the merge gate that `AGENTS.md` designates authoritative exits non-zero on this head. Reproducing it needs no host contention or flake.

This is exactly the gap the supplied evidence cannot see: the `create-pr` profile runs only the diagnostics `rules` suite (`verification/profiles/create-pr.json:118-124`), and none of the listed commands (`-p sifr_lowering`, annotated e2e fail suite, the two `crates/sifr` tests, registry skeleton, `gen-error-docs --check`, docs links, HIR/file-size guardrails, `fmt`, `clippy`) touch `verification/areas/diagnostics` baselines. I checked the blast radius: this is the only stale artifact — a whole-repo grep for `cannot mutate its receiver` hits only the registry, the new helper, the plan, `internal_docs/diagnostic_codes.md`, and this baseline; the OWN-0005/OWN-0014/PROTO-0005 baselines (`e2e_own_parameter_method_mutation_requires_mut`, `e2e_unsupported_narrowed_optional_mutating_receiver`, `e2e_protocol_receiver_mutability_mismatch`) carry unchanged messages, `hir_mixed_semantic_recovery/baselines/check-json.stderr.txt` only carries `SIFR-OWN-0002` (untouched by this branch), and `verification/areas/diagnostics/data/*.json` record no templates or arg lists for PROTO-0006.

Closure: re-bless just that fixture (`verification/areas/diagnostics/runner.py --suite baselines --bless`, then keep only the intended one-line diff) and add the diagnostics `baselines` suite to the recorded evidence, since a phase whose whole subject is rendered-message/argument contracts should not be validated on a profile that excludes rendered-message baselines.

---

## NON-BLOCKING 2 — The "declaration ordered" assertions cannot distinguish source order from alphabetical order, and the user-visible order after recovery is now alphabetical

The ordering fix sorts on `range.start()` first (`method_receiver_analysis.rs:245-252`), which is correct. But neither test can observe that:

- `fixed_receiver_diagnostics_are_declaration_ordered_and_class_distinct` (`method_receiver_diagnostics_tests.rs:129-188`) declares `Alpha, Bravo, Charlie, Delta, Echo, Foxtrot` — declaration order and lexicographic order coincide, so an implementation that sorted only by `class_name` (dropping the range key) passes unchanged.
- `test_fixed_receiver_diagnostics_survive_similar_recovery_cap` (`e2e_entrypoints.rs:404-467`) uses the same alphabetical names, and it observes group order rather than emission order anyway: `apply_diagnostic_recovery_limits` rebuilds output by iterating a `BTreeMap<SimilarDiagnosticGroupKey, …>` (`crates/sifr_driver/src/diagnostics.rs:167-179`), and `class_name` is part of `dedupe_args`, so the retained order is lexicographic by class name.

**Failure scenario.** A future fail fixture declaring `class Zulu` (line 3) then `class Alpha` (line 20), both with mutating `__eq__`: HIR emits `Zulu, Alpha` in source order, but the CLI path (`crates/sifr/src/diagnostic_rendering_and_run.rs:24-27` → `apply_diagnostic_recovery_limits`, no subsequent span sort anywhere in `sifr_driver`) prints `Alpha` first. The annotated fail suite documents that expectation annotations preserve declaration order, so the fixture's annotations must be written in an order that contradicts the source — and the plan's claim that "fixed-receiver violations are emitted in source order" (`plans/issues/active/…:672-676`) is true only of HIR emission, not of the rendered stream this phase now splits into per-class groups. Group-order re-sequencing is pre-existing behavior of the recovery limiter for any code with discriminating `dedupe_args`, but this branch is what puts PROTO-0005/0006 into it.

Closure: use one non-alphabetical class name in at least one ordering test (e.g. `Zulu` declared first) so the range key is actually pinned, and narrow the plan sentence to say HIR emission order is source-ordered while the rendered stream is grouped by recovery identity.

---

## Summary

Every pass-1 finding is closed in the Rust surface, and the pass-5 structured-argument findings remain closed: all five phase-owned codes populate their registry-declared arguments on the rendered-envelope path, PROTO-0006 carries `class_name` end-to-end (registry/template/args/dedupe/internal docs/plan), emission is deterministic through a responsibility-scoped helper module with named records, six same-dunder violations provably survive the similar-group cap with no omission summary, the phase-owned OWN-0005 field-receiver path is exact-asserted at `binding == "owner"` with the type/place leak removed, and the guardrail now checks Active state, all occurrences, key presence, and non-empty values with its scope and 120-gap deferral recorded in both code and plan with named ownership.

One blocking defect remains: the PROTO-0006 message widening did not update `verification/areas/diagnostics/.../e2e_operator_receiver_mutation_rejected/baselines/check-compact.stderr.txt:2`, which the merge profile compares byte-for-byte — the authoritative gate fails deterministically on this head, and the reported create-pr-scoped evidence structurally cannot detect it. One non-blocking finding covers ordering assertions that cannot distinguish source order from alphabetical order, plus the plan's over-broad source-order claim for the rendered stream.

NOT SATISFIED
