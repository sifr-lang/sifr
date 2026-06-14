# Wave 2.final Review — Sifr Verification Standard Closure

## Blocking findings

### B1. `cargo_metadata_classification.json` still labels `sifr_codegen` as `merge-red-blocker`

`verification/areas/coverage_matrix/data/cargo_metadata_classification.json:25` still has:

```json
{"name": "sifr_codegen", "kind": "lib", "classification": "first_party_compiler", "profile_assignment": "merge-red-blocker"}
```

This is the *only* surviving red-blocker label for the codegen surface and it directly contradicts the documented profile policy. `verification/policy/profile_policy.md:47-48` is explicit:

> Targets and features use `merge-red-blocker` **only for a suite that is planned for merge membership but intentionally not executed until its red-blocker closes**.

After this wave the suite is `status: blocking`, `executed_in_merge: true` in all four profiles, so the documented precondition for `merge-red-blocker` no longer holds. This is exactly the red-blocker metadata the phase commits to retiring at gate closure (question 1). The reason the matrix checker doesn't flag it is incidental: `validate_targets` (`verification/areas/coverage_matrix/checks/coverage_matrix.py:454-456`) only verifies the value is in `VALID_PROFILE_ASSIGNMENTS` and never cross-checks against the merge membership row, and `validate_cargo_metadata_classification` (lines 380-397) is satisfied as long as *either* `executed_in_merge=true` *or* a `red-blocker` membership exists. Neither catches the stale label.

Fix: change `profile_assignment` to `"merge"` in this row. The phase's "no remaining codegen red-blocker in profile/matrix semantics" promise isn't honored until this is updated. (Optional follow-up: add a `validate_targets` cross-check so `merge-red-blocker` is only accepted when the matching merge membership is actually `red-blocker`/`executed_in_merge=false`.)

## Non-blocking findings

### N1. Selftest only guards the `merge` profile

`verification/runner/sifr_verify/selftest.py:86,104-108` loads only `load_all_profiles()["merge"]` and checks `sifr_codegen` status/execution there. Demoting `sifr_codegen` back to `red-blocker` in `create-pr`, `nightly`, or `release` would pass selftest, schema, `validate_crate_test_membership`, and the matrix checker, because the matrix only inspects merge memberships (`coverage_matrix.py:361-397`). Given the four profiles are intentionally kept in lockstep on this row, a tiny loop iterating all of `("create-pr","merge","nightly","release")` would close this gap. Not blocking — the merge profile is the gate the user explicitly asked about — but the protection is narrower than it looks.

### N2. `option_binding_requires_mut_for_ir` is structural, not use-driven

`condition_lowering.rs:152-173` emits `Some(mut option_var)` whenever the local's type is `Option<Class>` and the class is anywhere in `recursive_fields`. This catches the e2e shape correctly, but it also binds `mut` for any narrowed recursive option even when the narrowed body only reads fields — i.e. it can emit `unused_mut` shapes. This is consistent with the existing local-class binding behavior asserted by `test_local_recursive_node_binding_is_mutable_for_child_moves`, so it isn't regressing precedent, and the test suite proves no current fixture trips a warning. Worth filing a follow-up to drive `mut` off actual `.take()` use rather than a class-recursion heuristic; not blocking for this wave.

### N3. Regression test asserts on rendered strings only

`recursive_node_codegen_tests.rs:139-146` asserts that the generated Rust contains `let Some(mut term) = term else` and `term.expr.take().map(...)`. That catches the binding-shape regression the e2e exposed and matches the rest of the file's test style, but it does not compile-check the produced Rust. If a future change keeps the `mut` prefix but breaks the surrounding let-else/closure scope, this test would still pass while the e2e regresses. The e2e `recursive_mutual_classes_runtime.sifr` run is the real safety net here — make sure that fixture stays referenced in the codegen merge membership, since the codegen unit test alone isn't a complete proof of the fix.

### N4. IPC fixture lock poisons cascade on test panic

`crates/sifr_stdlib/tests/ipc_process_pipe_fixture.rs:14,35-37,69-75,104`. Scope is correct — the lock serializes only the cargo-run/bootstrap handoff (which shares `target/ipc_process_pipe_fixture_worker` and the cargo build lock) and is dropped right after the worker emits its bootstrap frame, so it does not mask runtime-level product bugs in `IpcConnectionState`. But because the guard is held across `spawn` + `read_frame`, a panic in either path (e.g. cargo build failure, worker exiting early) poisons the static `Mutex`, and the subsequent `.lock().expect("IPC fixture startup lock is not poisoned")` turns the next test's failure into a misleading expect-message instead of the real spawn error. Consider `.lock().unwrap_or_else(|e| e.into_inner())` so the first failing test surfaces the actual cause rather than being shadowed by poison cascades.

### N5. Stale `surface_id: codegen_red_blocker`

`compiler_surface_matrix.json:140` keeps the row id `codegen_red_blocker` even though the row is now plain `blocking` with no red-blocker fields. The name is misleading; rename to e.g. `codegen_merge_blocking` is a low-risk cleanup. Cross-refs are limited to the matrix file itself.

### N6. Validation evidence is missing two AGENTS.md required commands

The evidence list covers `cargo test -p sifr_codegen`, the targeted e2e, `--profile merge`/`--profile create-pr`, `cargo fmt --check`, `check_file_size_guardrails.py`, and `git diff --check`. It does **not** include the two other commands AGENTS.md lists as guardrails:

- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`

`run_all_tests.sh` likely invokes both internally — but the explicit local-validation manifest in the phase doc and the user's evidence summary should call them out, otherwise reviewers can't tell whether they ran. Add them to the Wave 2.final implementation-notes "Validation" line for parity with the other waves' bookkeeping.

### N7. Inventory `captured_on` vs. count

`codegen_red_blocker_inventory.json` keeps `captured_on: 2026-06-14` but the `test_result` block now reports `709/0/709` — a Wave 2.final number. Since the same date covered every wave in this run, it's not factually wrong, but a reader sees a date that matches the *original* capture and counts that don't. A short note like `"updated_on": "2026-06-14 (post-Wave-2.final)"` or simply moving the count into a `closure_snapshot` block would make the inventory unambiguous.

## Answers to the specific questions

1. **Profile promotion honesty:** Profiles + selftest + matrix row + governance text are correct. The hold-out is **B1** — the cargo classification still carries the red-blocker label and contradicts the profile policy text. Until that is updated, the promotion is not yet honest end-to-end.
2. **Selftest demotion protection:** Correct for `merge` (the asked-about gate). See **N1** for the gap across the other three profiles.
3. **Condition-lowering fix correctness/scope:** Correct and minimally scoped. It honors borrowed-param paths (no spurious mut on `&Option<T>`), falls back to the existing `mutated_vars` short-circuit, and only widens to recursive class locals. See **N2** for the "structural vs. use-driven" caveat.
4. **Regression meaningful enough:** Yes for the shape that broke (binding-mutability + child-move). It is not a full compile-check though; the e2e `recursive_mutual_classes_runtime.sifr` is what proves the end-to-end shape — keep both. See **N3**.
5. **IPC fixture lock:** Correctly scoped to the cargo-run/bootstrap handoff and released before runtime semantics begin, so it does not mask product issues in `IpcConnectionState`. Resilience nit only — see **N4**.
6. **Doc/inventory honesty:** The 708 → 709 transition is described correctly in both the triage doc and the phase doc; the merge timing (780.24s post-fix vs. 848.99s catch-the-bug vs. Wave 1 986.72s baseline) and the remaining advisories (warm-cache rebuild on create-pr, group-skew on merge) are stated. Caveats: `inventory.json` `captured_on` reads as a Wave-2.0 date (**N7**), the matrix `surface_id` still says `codegen_red_blocker` (**N5**), and the cargo classification still says `merge-red-blocker` (**B1**).
7. **Missing validation:** Add `cargo clippy --workspace -- -D warnings` and `python3 scripts/check_hir_maintainability_guardrails.py` to the validation list (**N6**). Also worth one more `uv run … profiles check` after fixing **B1** to confirm the matrix-strict path still passes once the cargo classification flips to `merge`.

## Recommendation

Hold for **B1**: flip `cargo_metadata_classification.json`'s `sifr_codegen` row to `profile_assignment: "merge"`, rerun `uv run --project verification --locked python -m sifr_verify profiles check` and `scripts/run_all_tests.sh --profile merge --emit-plan`. The remaining items are non-blocking and can ride in follow-ups.
