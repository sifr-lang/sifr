Reviewed the diff and cross-checked against the retained manifest, the `sifr_retained_intrinsics` fallback source, the intrinsic registry, and the M12/M13 plan sections.

## Verdict: READY

## Findings (ordered by severity)

None blocking. No correctness or brittleness issues that would justify holding the M12a PR.

## Notes on the review focal points

1. **Required-state guard scope** - Correctly narrow. `REQUIRED_SURFACE_STATES` pins exactly the three surface IDs M12a is hardening (`_sifr.runtime::observability_glue`, `_sifr.task::language_runtime_glue`, `generated-test-glue`) - all already `retained-by-design` in `internal_docs/stdlib_retained_compiler_intrinsics.toml:63,201,227`. Not too brittle for M13: the plan (`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:1092`) explicitly calls for "Convert temporary migration guards into permanent no-regression guards" at final closure, and widening this dict is a trivial follow-up. The guard composes cleanly with the existing `_validate_transitions` "new manifest rows must be retained-by-design" invariant - no overlap, no ordering hazard.

2. **Comment updates accuracy** - Accurate. The fallback signatures for `runtime_emit_diagnostic` and `task_current_context` live in `crates/sifr_retained_intrinsics/src/runtime.rs:6` and `crates/sifr_retained_intrinsics/src/task.rs:6`, not in the `.sifr` private-module files. The prior "Concrete Rust interop declarations migrate here in later milestones" comment was stale - nothing is scheduled to migrate into these two `.sifr` files, matching M12 acceptance criterion "Retained-by-design entries are precise and stable."

3. **Task codegen test coverage** - Focused, not overfit. Asserts (a) `task_current_context` with no args lowers, (b) `required_feature = Tokio` (matches `registry.rs:97`), (c) rendered ident is `__sifr_task_current_context()` (matches `registry/task.rs:7`). No implementation details beyond the public dispatch contract are locked in.

## Non-blocking suggestions

- The runtime intrinsic has a companion arity-rejection test (`runtime_diagnostic_intrinsic_rejects_wrong_arity`, `registry_core_tests.rs:139`); an equivalent `task_current_context` negative test would close the symmetry (`task::lower_task_current_context` returns `None` on non-empty args). Cheap to add, but not required for M12a.
- The failure message `"{surface_id}: required retained manifest surface is missing"` doesn't include the expected state. Minor - the message phrasing implies retained-by-design, and the dict is one screen up in the file.
- `_validate_required_surface_states` is invoked in `main()` but not from `_self_test`'s existing manifest-level fixtures; it is only exercised through the three dedicated fixtures. Fine; the intent (unit-scope the new function) is clear.
