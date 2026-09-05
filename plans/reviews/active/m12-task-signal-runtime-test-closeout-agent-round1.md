# M12 Task, Signal, Runtime Observability, and Test Helpers Closeout Review

## Verdict: READY

M12 can be marked merged after PR #2914 (sha `4fe73924a`). All M12 acceptance criteria are met, with a single non-blocking stale-comment carryover.

## Findings

### Non-blocking

**F1 - `stdlib/_sifr/test.sifr:2` still carries stale "migrate later" comment.** M12a corrected the same stale wording in `stdlib/_sifr/runtime.sifr` and `stdlib/_sifr/task.sifr` (per the M12a agent round 1 review notes at `plans/reviews/active/m12a-retained-design-classification-agent-round1.md:13`) but did not touch `_sifr/test.sifr`, which still reads "Concrete Rust interop declarations migrate here in later milestones." The `assert_*` intrinsics are retained-by-design under `generated-test-glue` (`internal_docs/stdlib_retained_compiler_intrinsics.toml:63-79`) and are not scheduled to migrate into `_sifr/test.sifr` - nothing actually imports the module (`grep 'from _sifr.test'` in `stdlib/` returns nothing). Cosmetic drift only; no guard breakage. Recommended to sweep into an M12 closeout commit for consistency but not required for closure.

**F2 - Missing arity negative test for `task_current_context`.** The M12a review at line 19 flagged that a companion `task_current_context_rejects_wrong_arity` (mirroring `runtime_diagnostic_intrinsic_rejects_wrong_arity` at `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:139`) would close symmetry with `crates/sifr_codegen/src/intrinsics/registry/task.rs:4-6` (which returns `None` on non-empty args). Nothing merged this test; cheap follow-up.

## Answers to focal questions

1. **M12-owned surfaces precisely retained-by-design.** Yes. `_sifr.runtime::observability_glue`, `_sifr.task::language_runtime_glue`, and `generated-test-glue` are all `retained-by-design` (`stdlib_retained_compiler_intrinsics.toml:201-243`, `63-79`) and now pinned by `REQUIRED_SURFACE_STATES` in `scripts/check_stdlib_manifest_schema.py:51-55`, invoked from `main()` at line 62 and covered by three self-test fixtures at lines 625-683. Their compiler surfaces are minimal: one `task_current_context` lowerer (`crates/sifr_codegen/src/intrinsics/registry/task.rs`), one `runtime_emit_diagnostic` lowerer (`registry/runtime.rs`), seven `assert_*` lowerers (`registry/test.rs`), and their exact-arity fallback signatures in `crates/sifr_retained_intrinsics/src/{task.rs,runtime.rs,math_test.rs}`.

2. **Signal classification closed/recorded.** Yes. `_sifr.signal` sits at `closing` (`toml:189-198`) with `certification_rows = ["callback_subscription_core"]` recording M10 evidence. `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:284-291` regressions prove `signal_ctrl_c`/`signal_terminate`/`signal_shutdown` no longer lower through compiler dispatch; `sifr.signal` (`stdlib/sifr/signal.sifr:2`) imports from `_sifr.signal` whose declarations point at `sifr_stdlib.signals.*`.

3. **Public stdlib wrapper behavior still in compiler intrinsic dispatch?** No. The M12-owned wrappers that live in `stdlib/sifr/*.sifr` (`test.sifr`, `runtime.sifr`, `task.sifr`, `signal.sifr`) all reside in Sifr source. The three intrinsics still dispatched by the compiler (`task_current_context`, `runtime_emit_diagnostic`, seven `assert_*`) are the retained-by-design language/harness/observability glue this milestone explicitly preserves.

4. **M12-owned preambles mixed with migrated stdlib behavior?** No. The six preambles under `_sifr.task::language_runtime_glue` (`task_context_runtime.rs`, `task_runtime.rs`, `join_set_runtime.rs`, `parallel_runtime.rs`, `task_scope_offload_runtime.rs`, `cpu_offload_runtime.rs`) contain only task/scope/join-set/offload runtime machinery - no signal, no logging, no random, no fs behavior. Note that `mixed-io-logging-random-preamble` (`io_file_handles.rs`) and the `_sifr.fs`/`_sifr.sys` retained rows remain `retained` in the manifest, but the review context explicitly scopes those to M13 final closure (`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:1076-1105`).

5. **Should M12 be marked merged now?** Yes. Update the M12 evidence cell (currently `planned` at `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:104`) to `merged` with `PR #2914 - sha=4fe73924`. No additional M12 implementation slice is required. Recommend folding F1 (and optionally F2) into the closeout commit that updates the evidence table; both are trivial.
