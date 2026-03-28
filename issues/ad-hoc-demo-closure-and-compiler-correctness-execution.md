# Ad Hoc Phase: Demo Closure and Compiler Correctness — Execution Ledger

Status: open (created 2026-03-28)
Owning phase: `issues/ad-hoc-demo-closure-and-compiler-correctness.md`

## Entry Baseline

- Baseline date: `2026-03-28`
- Full sweep contract:
  - `266` `sifr run` demo entrypoints
  - `9` demo-local `sifr test` directories
  - `275` total checks
  - `22` failing `run` demos in the full sweep
  - `9/9` demo-local `test` directories passing
- Baseline discovery commands:
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
  - sweep execution rule:
    - `target/debug/sifr run <path>` for each discovered run entrypoint
    - `target/debug/sifr test <dir>` for each discovered demo-local test directory
- Post-sweep direct rerun delta:
  - `demos/local_shadowing/main.sifr` confirmed passing on `2026-03-28`
  - active unresolved renamed-demo set for this phase: `21`
- Supporting emit-audit note:
  - full-demo emit sweep on `2026-03-28`: `266` run-entrypoint discoveries evaluated through `emit`
  - `24` emit-time failures in the broader demo tree
  - `22` of those are current `emit`-mode project/module-resolution failures for multi-file demos
  - `2` are real pre-codegen failures: `mut_sort`, `recursive_records`
  - emitted Rust also confirmed live compiler/codegen defects for iterator lifetimes, `FileHandle` initializers, `defaultdict`, and `tuple_assignment`
- Baseline evidence anchors:
  - iterator lifetime shape: `demos/custom_iterables/emitted.rs`
  - `FileHandle` constructor bypass: `demos/system_tools/emitted.rs`
  - borrowed-key mismatch: `demos/defaultdict/emitted.rs`
  - receiver-mutability mismatch: `demos/tuple_assignment/emitted.rs`
  - recursive frontend diagnostic: `demos/recursive_records/emit.stderr`
  - tentative demo-adaptation diagnostic: `demos/mut_sort/emit.stderr`

## Wave Status

### wave_1_iterator_lifetime_codegen

status: pending

- PR:
- Validation:
- Notes:
  - record whether closure evidence came from live `emit`, persisted `idiomatic.rs`, or both

### wave_2_filehandle_layout_closure

status: pending

- PR:
- Validation:
- Notes:
  - record which constructor-bypass sites were eliminated or regenerated under the canonical `FileHandle` construction rule

### wave_3_demo_adaptation_mut_sort

status: pending

- PR:
- Validation:
- Notes:
  - record the ownership decision first: intentional semantics vs compiler reclassification

### wave_4_generated_borrow_mismatch

status: pending

- PR:
- Validation:
- Notes:
  - record whether each closure used borrowed-key lookup, `entry(...).or_insert(...)`, borrowed wrapper-call emission, or another generalized rule

### wave_5_recursive_surface_closure

status: pending

- PR:
- Validation:
- Notes:
  - record whether each fix landed in frontend expression support, later lowering, or both

### wave_6_tuple_assignment_codegen_closure

status: pending

- PR:
- Validation:
- Notes:
  - record the final HIR-to-codegen receiver-mutability contract that replaced the discarded `_is_mutating` path

## Phase Closeout

- Final full sweep result:
- Final local validation:
- Remaining explicitly reclassified items:
- Merged PR links:
