# Ad Hoc Phase: Demo Closure and Compiler Correctness

Status: open (documented 2026-03-28)
Context: corrective follow-up phase inserted after the latest `demos/` reliability sweep and after the Phase 31 algorithmic compatibility carry-forward plan was reviewed for language-rot risk
Execution readiness: implementation-ready in dependency order; compiler root causes must be fixed before any broad demo sweep is treated as closed
Execution ledger: `issues/ad-hoc-demo-closure-and-compiler-correctness-execution.md`

## Objective

Close the remaining renamed `demos/` failures by fixing the real owning layer for each break:

- compiler/codegen/frontend bugs when the demo is exercising a valid Sifr feature,
- demo-source adaptation only when newer Sifr semantics intentionally changed and the demo is stale,
- and explicit curation decisions when a demo is no longer a `run`-positive artifact.

This phase is not a request to make every current demo pass by any means necessary. The target is one coherent, production-grade closure pass that preserves Sifr's existing guarantees:

- static type safety,
- ownership and borrow discipline,
- no user-triggerable panic paths,
- deterministic lowering,
- and Rust-consistent semantics rather than compatibility-shaped hacks.

## Source of Truth

- latest demo failure inventory from the 2026-03-28 rerun in the current workspace
- Phase 31 policy and guardrails:
  - `internal_docs/phases/31_algorithmic_compatibility_and_leetcode_coverage.md`
  - `internal_docs/verification/phase31_leetcode_corpus_policy.md`
  - `reviews/archive/phase31-rot-risk-pass-1.md`
  - `issues/archive/phase31-ad-hoc-followup-milestones.md`
- related ad hoc phases:
  - `issues/archive/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`
  - `issues/archive/ad-hoc-runtime-and-file-object-parity-expansion.md`
  - `issues/archive/ad-hoc-full-recursive-type-feature.md`
  - `issues/archive/ad-hoc-own-mut-parameter-convention.md`
- implementation hotspots:
  - `crates/sifr_hir/`
  - `crates/sifr_codegen/`
  - `crates/sifr_driver/`
  - `lib/sifr/`
  - `demos/`

## Why This Needs Its Own Phase

The current failures are not one-off demo defects. They cluster into a small number of shared root causes:

- iterator-returning lowering still emits Rust with invalid lifetimes,
- runtime/file-object parity work left a `FileHandle` layout mismatch across generated initializers,
- some generated Rust still loses borrowing intent at call and map-access sites,
- recursive demo surfaces still expose residual frontend/lowering gaps,
- and one remaining demo still needs canonical source adaptation to current ownership semantics.

That is phase-worthy because:

- the affected demos span multiple milestones and feature areas,
- the compiler buckets are broad enough to mask each other in reruns,
- and the Phase 31 compatibility work makes it important to close these as general language rules rather than demo-specific or LeetCode-shaped patches.

## Depends on

- `issues/archive/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`
  - wave 1 is corrective closure on top of the landed canonical iteration architecture rather than a new iterator-model redesign
- `issues/archive/ad-hoc-runtime-and-file-object-parity-expansion.md`
  - wave 2 is corrective closure on top of landed runtime/file-object work and must restore one canonical `FileHandle` initializer contract
- `issues/archive/ad-hoc-full-recursive-type-feature.md`
  - wave 5 is limited to residual closure work inside the landed recursive-type contract
  - if a failing recursive shape requires net-new recursive feature expansion, it must be routed back to that phase rather than patched locally here
- `issues/archive/ad-hoc-own-mut-parameter-convention.md`
  - wave 3 may use already-landed `own mut` semantics, but it must not reopen parameter-convention design
- Phase 27 runtime-safety invariants remain mandatory
- Phase 29 local-first validation and full-suite closure remain mandatory
- Phase 31 carry-forward planning guardrails remain mandatory:
  - `reviews/archive/phase31-rot-risk-pass-1.md`
  - `internal_docs/phases/31_algorithmic_compatibility_and_leetcode_coverage.md`
  - `internal_docs/verification/phase31_leetcode_corpus_policy.md`
  - this phase inherits the same non-negotiable rule:
    compiler changes must be general language/compiler rules with regression coverage beyond the compatibility corpus, not corpus-shaped recognizers or compatibility hacks

## Entry Baseline

Latest known renamed-demo failures after `local_shadowing` was fixed on 2026-03-28:

- latest full sweep contract in this workspace:
  - `266` `sifr run` demo entrypoints
  - `9` demo-local `sifr test` directories
  - `275` total checks
  - `22` failing `run` demos in that full sweep
  - `9/9` demo-local `test` directories passing
- reproducible discovery commands for the baseline sweep:
  - run entrypoints:
    ```bash
    {
      find demos -maxdepth 1 -type f -name '*.sifr'
      find demos -type f -name 'main.sifr'
      find demos -type f -name '*_demo.sifr'
    } | sort -u \
      | grep -v '/negative_cases/' \
      | grep -vE '/(helper|shared|provider|consumer|worker|formatter|models|utils|scratch|unrelated_not_in_graph|a_provider|a_consumer|z_provider|test_matrix)\.sifr$' \
      | grep -vE '/test_[^/]+\.sifr$' \
      | grep -v '/milestone_borrow_hardening_demo/exclusivity_error_demo.sifr$'
    ```
  - demo-local test directories:
    ```bash
    find demos -type f -name 'test_*.sifr' \
      | grep -v '/negative_cases/' \
      | xargs -n1 dirname \
      | sort -u
    ```
- execution rule for that sweep:
  - run `target/debug/sifr run <path>` for each discovered run entrypoint
  - run `target/debug/sifr test <dir>` for each discovered demo-local test directory
- post-sweep delta:
  - `demos/local_shadowing/main.sifr` was rerun directly on 2026-03-28 and confirmed passing
  - active unresolved set for this phase is therefore reduced from `22` to `21` without changing the sweep definition
- sweep-definition note:
  - implementation and validation for this phase must continue using the same demo sweep contract unless an explicit follow-up change is documented in the execution ledger

- `demos/advanced_class_libraries/main.sifr`
- `demos/class_libraries/main.sifr`
- `demos/custom_iterables/main.sifr`
- `demos/defaultdict/main.sifr`
- `demos/extended_itertools/main.sifr`
- `demos/generic_stdlib/main.sifr`
- `demos/iterator_basics/main.sifr`
- `demos/iterator_integration/main.sifr`
- `demos/iterators_and_randomness/main.sifr`
- `demos/itertools/main.sifr`
- `demos/itertools_iterables/main.sifr`
- `demos/itertools_iterators/main.sifr`
- `demos/mut_sort/main.sifr`
- `demos/nested_recursive_helpers/main.sifr`
- `demos/ordering_rules/main.sifr`
- `demos/pure_stdlib/main.sifr`
- `demos/python_regressions/main.sifr`
- `demos/recursive_records/main.sifr`
- `demos/regex_and_filesystem/main.sifr`
- `demos/system_tools/main.sifr`
- `demos/tuple_assignment/main.sifr`

Current root-cause classification:

- compiler/codegen/high-confidence:
  - iterator lifetime lowering bug: `12` demos
  - `FileHandle` layout drift: `3` demos
  - borrow/type mismatch in generated Rust: `2` demos
  - recursive lowering/access gap: `2` demos
- demo-source adaptation:
  - `mut_sort`: `1` demo
- ambiguous ownership/mutability case:
  - `tuple_assignment`: `1` demo

Out-of-scope from this phase baseline:

- `demos/local_shadowing/main.sifr` is no longer failing
- all demo-local `test` directories were already green in the latest sweep

## Language Contract and Guardrails

### Core contract

This phase must preserve and strengthen:

- no generated user-path `panic!`, `.unwrap()`, or `.expect()`
- explicit ownership rather than hidden cloning or aliasing
- explicit optional/union behavior rather than pretend-total access
- deterministic lowering and diagnostics
- one general compiler rule per fix, not per-demo recognizers

### Demo adaptation policy

A demo may be edited in this phase only if all of the following are true:

- the current compiler behavior is consistent with intentional Sifr semantics,
- the demo is stale against those semantics,
- and the adaptation uses already-landed language features rather than introducing a workaround shape.

This phase must not rewrite demos to paper over compiler bugs.

### Compatibility policy

The Phase 31 LeetCode corpus remains a verification input, not a language spec.

Therefore this phase must not:

- add LeetCode-specific recognizers,
- add demo-name-specific lowering branches,
- weaken ownership to accept Python-shaped aliasing,
- weaken parse safety or optional safety,
- or add eager fallback behavior to hide iterator/backend defects.

If a fix cannot be expressed as a general compiler or language rule, it does not belong in the compiler portion of this phase.

## Scope

This phase owns:

- iterator lifetime-safe lowering for canonical iterator pipelines
- runtime/file-object initializer consistency for generated Rust
- generated Rust borrow correctness at call and mutable-map access sites
- recursive demo closure when blocked by residual lowering/frontend gaps
- one canonical demo adaptation for `mut_sort`
- final classification and closure of `tuple_assignment`
- full rerun evidence for `demos/` after each bucket closure

This phase does not own:

- broad new language expansion unrelated to the failing demos
- weakening Sifr safety principles for compatibility
- demo churn to route around compiler defects
- broad LeetCode corpus work beyond the guardrails already documented in Phase 31

## Execution Order

### wave_1_iterator_lifetime_codegen

status: pending

Root cause:

- generated Rust returns boxed or adapter-backed iterators that borrow local temporaries, producing `E0515`

Affected demos:

- `demos/custom_iterables/main.sifr`
- `demos/extended_itertools/main.sifr`
- `demos/generic_stdlib/main.sifr`
- `demos/iterator_basics/main.sifr`
- `demos/iterator_integration/main.sifr`
- `demos/iterators_and_randomness/main.sifr`
- `demos/itertools/main.sifr`
- `demos/itertools_iterables/main.sifr`
- `demos/itertools_iterators/main.sifr`
- `demos/ordering_rules/main.sifr`
- `demos/pure_stdlib/main.sifr`
- `demos/python_regressions/main.sifr`

Required closure:

- iterator lowering must preserve ownership/lifetime validity without hidden eager materialization
- emitted Rust must not return iterators backed by locals that go out of scope
- the implementation must follow the canonical iteration architecture rather than introducing special-case lowering for the affected demos

Definition of done:

- all demos in this bucket run
- new regression coverage exists outside the current demo set
- no lazy API silently becomes eager as the fix mechanism

### wave_2_filehandle_layout_closure

status: pending

Root cause:

- generated/manual Rust initialization paths drifted from the current `FileHandle` layout and now miss `_closed`, producing `E0063`

Affected demos:

- `demos/advanced_class_libraries/main.sifr`
- `demos/class_libraries/main.sifr`
- `demos/system_tools/main.sifr`

Required closure:

- one canonical initializer contract for file-handle values across runtime and codegen
- no field-by-field drift between emitted code and runtime types

Definition of done:

- all demos in this bucket run
- runtime/file-object regression coverage locks the new contract

### wave_3_demo_adaptation_mut_sort

status: pending

Root cause:

- the demo uses an ownership/mutability shape that is stale against current Sifr semantics

Affected demo:

- `demos/mut_sort/main.sifr`

Required closure:

- adapt the demo to canonical current semantics only if the compiler behavior is confirmed intentional
- if investigation shows the failure is actually compiler-side, reclassify this item into the owning compiler bucket instead of forcing a demo rewrite

Definition of done:

- the demo runs in canonical Sifr form
- the final source does not rely on legacy semantics or workaround-y ownership patterns

### wave_4_generated_borrow_mismatch

status: pending

Root cause:

- generated Rust loses borrowing intent, producing `E0308`

Affected demos:

- `demos/defaultdict/main.sifr`
- `demos/regex_and_filesystem/main.sifr`

Required closure:

- map/dict mutable access must borrow keys correctly
- wrapper and helper call emission must preserve borrowed argument shapes where the Rust surface requires references
- the fix must be generalized by emitter rule, not keyed to these call sites

Definition of done:

- all demos in this bucket run
- at least one non-demo regression test covers each generalized emitter rule

### wave_5_recursive_surface_closure

status: pending

Root cause:

- residual recursive lowering/access gaps remain even after the broader recursive-type work

Affected demos:

- `demos/nested_recursive_helpers/main.sifr`
- `demos/recursive_records/main.sifr`

Required closure:

- fix only the residual gaps that belong to already-supported recursive semantics
- if a failing shape requires feature expansion beyond the landed recursive-type contract, route it back to the recursive-type phase as an explicit gap rather than patching it locally

Definition of done:

- the demos run under one coherent recursive lowering contract
- no `TreeNode`/`ListNode`-style special casing is introduced

### wave_6_tuple_assignment_final_classification

status: pending

Open question:

- `demos/tuple_assignment/main.sifr` still appears more likely to be a mutability-propagation/compiler issue than a stale demo, but that judgment should be made only after the earlier buckets land

Required closure:

- re-run after waves 1 through 5
- classify it as either:
  - closed incidentally,
  - compiler-owned,
  - or true demo adaptation

Definition of done:

- the ownership of this remaining failure is explicit and justified
- the final fix follows the same guardrails as the rest of the phase

## Validation Contract

Before closing any wave:

- run the directly affected demos
- run targeted compiler/e2e coverage for the owning root cause
- run `cargo test -p sifr -- --skip test_e2e_pass`
- run `scripts/run_all_tests.sh --profile quick`

Before closing the full phase:

- rerun the full `demos/` sweep using the baseline discovery commands from `Entry Baseline`
- run `scripts/run_all_tests.sh`
- record the before/after failing-demo counts and residual explanations in the execution ledger

## Exit Gate

This phase is closed only when all of the following are true:

- all `21` currently in-scope failing renamed demos run successfully, or any remaining non-pass case has been explicitly reclassified with owning-phase documentation and removed from the active failing set by policy rather than omission
- the full `demos/` sweep still uses the documented baseline sweep contract unless a deliberate contract update is recorded in the execution ledger
- each closed wave has recorded evidence in `issues/ad-hoc-demo-closure-and-compiler-correctness-execution.md`
- no regression is introduced against the inherited Phase 27 runtime-safety invariants, Phase 29 local-first validation contract, or Phase 31 carry-forward anti-rot policy
- `cargo test -p sifr -- --skip test_e2e_pass`, `scripts/run_all_tests.sh --profile quick`, and `scripts/run_all_tests.sh` all pass at final phase closure
- the execution ledger records the before/after failing-demo counts, residual classifications, validation commands, and the merged PR links for each closed wave

## Non-goals

- turning intentionally strict Sifr behavior back into Python-like permissiveness
- silently cloning iterators or collections to hide ownership/lifetime problems
- demo-only shims for compiler defects
- LeetCode-specific or demo-name-specific compiler logic
- broad roadmap expansion beyond the listed buckets

## Ready-for-Implementation Judgment

This phase is ready to implement now because:

- the failing surface is already reduced to a small number of root-cause buckets,
- the bucket ordering is dependency-aware,
- one known demo-side fix has already been removed from scope (`local_shadowing`),
- and the policy boundary between canonical demo adaptation and compiler work is explicit.

The only intentionally deferred classification is `tuple_assignment`, and that deferral is deliberate: it should be judged only after the higher-leverage compiler buckets are closed.
