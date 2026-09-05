# Ad Hoc Phase: Demo Closure and Compiler Correctness — Execution Ledger

Status: complete (created 2026-03-28, closed 2026-03-28)
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

status: completed

- PR: `https://github.com/sifr-lang/sifr/pull/1435`
- Validation:
  - `cargo test -p sifr_codegen test_generate_rust_iterator_return_consumes_local_list_binding -- --nocapture`
  - `cargo test -p sifr_codegen test_generate_rust_iterator_return_consumes_owned_param_binding -- --nocapture`
  - `cargo test -p sifr_codegen test_generate_rust_iterable_return_from_iterator_materializes_for_signature -- --nocapture`
  - `target/debug/sifr run` passes on the iterator-family phase demos:
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
- Notes:
  - Closure evidence came from live emitted Rust and direct `run` reruns.
  - Return-path iterator lowering now forces ownership consumption for escaping `iter(...)` returns backed by owned locals/temporaries, eliminating the invalid borrowed-local iterator shape.

### wave_2_filehandle_layout_closure

status: completed

- PR: `https://github.com/sifr-lang/sifr/pull/1435`
- Validation:
  - `cargo test -p sifr_codegen test_generate_rust_open_uses_canonical_filehandle_constructor -- --nocapture`
  - `target/debug/sifr run demos/advanced_class_libraries/main.sifr`
  - `target/debug/sifr run demos/class_libraries/main.sifr`
  - `target/debug/sifr run demos/system_tools/main.sifr`
- Notes:
  - Eliminated the builtin `open(...)` constructor-bypass path by routing generated success values through `FileHandle::new(...)`.

### wave_3_demo_adaptation_mut_sort

status: completed

- PR: `https://github.com/sifr-lang/sifr/pull/1435`
- Validation:
  - `target/debug/sifr run demos/mut_sort/main.sifr`
- Notes:
  - Ownership decision: intentional semantics. `mut values: list[int] -> list[int]` is a borrowed mutable parameter and cannot escape by return.
  - Canonical demo adaptation landed: `mut_sort` now uses `own mut` to model consume-mutate-return explicitly.

### wave_4_generated_borrow_mismatch

status: completed

- PR: `https://github.com/sifr-lang/sifr/pull/1435`
- Validation:
  - `cargo test -p sifr_codegen test_generate_rust_generator_clones_borrowed_params_into_owned_locals_before_calls -- --nocapture`
  - `cargo test -p sifr_codegen test_generate_rust_defaultdict_int_augassign_uses_entry_default -- --nocapture`
  - `target/debug/sifr run demos/defaultdict/main.sifr`
  - `target/debug/sifr run demos/regex_and_filesystem/main.sifr`
- Notes:
  - Preserved the `__compat_defaultdict_*` alias through HIR subscript-augassign validation so codegen can keep `defaultdict(int)` on the `entry(...).or_insert(0)` path.
  - Generator materialization now treats cloned borrowed params as owned locals during call adaptation, restoring borrowed wrapper-call emission such as `glob(&directory, &pattern)`.

### wave_5_recursive_surface_closure

status: completed

- PR: `https://github.com/sifr-lang/sifr/pull/1435`
- Validation:
  - `cargo test -p sifr_codegen test_generate_rust_recursive_constructor_argument_wraps_optional_box_field -- --nocapture`
  - `target/debug/sifr run demos/nested_recursive_helpers/main.sifr`
  - `target/debug/sifr run demos/recursive_records/main.sifr`
- Notes:
  - Compiler closure: registry-backed constructor lowering now applies the same signature-aware recursive optional boxing as the structured plain-call path.
  - Demo closure: `recursive_records` was normalized to explicit option-typed locals instead of constructing `Some(record)` temporaries multiple times. That preserves borrow-by-default semantics and avoids accidental moves.

### wave_6_tuple_assignment_codegen_closure

status: completed

- PR: `https://github.com/sifr-lang/sifr/pull/1435`
- Validation:
  - `cargo test -p sifr_codegen test_generate_rust_tuple_field_assignment_emits_mutable_self_receiver -- --nocapture`
  - `target/debug/sifr run demos/tuple_assignment/main.sifr`
- Notes:
  - Receiver-mutability detection in codegen now treats tuple-unpack field targets as self-field mutation, so methods such as `rotate` lower to `&mut self` instead of `&self`.

## Phase Closeout

- Final full sweep result:
  - Former unresolved phase set rerun: `21/21` pass (`target/debug/sifr run` on every formerly failing renamed demo after `local_shadowing` had already been fixed outside this phase).
- Final local validation:
  - `scripts/run_all_tests.sh` passed on 2026-03-28.
- Remaining explicitly reclassified items:
  - `demos/mut_sort/main.sifr` was confirmed as a canonical demo adaptation to `own mut`, not a compiler bug.
  - `demos/recursive_records/main.sifr` retained the compiler-side recursive constructor fix, but its final call-site shape was also normalized to explicit option-typed locals to avoid repeated temporary ownership moves.
- Merged PR links:
- `https://github.com/sifr-lang/sifr/pull/1435`
- final implementation review: `tmp/agent_phase_final_impl_review.md` (`no actionable findings`)
