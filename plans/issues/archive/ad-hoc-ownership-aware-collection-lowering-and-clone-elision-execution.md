# Ad Hoc Phase Execution Checklist (Ownership-Aware Collection Lowering and Clone Elision)

Status: closed (started 2026-03-21; `wave_clone_0` architecture lock/baseline, `wave_clone_1` iterator/comprehension ownership correction, `wave_clone_2` index/slice/star-unpack ownership correction, and `wave_clone_3` generic hardening/regression lock completed on 2026-03-21; wave-closure pass-1/pass-2, milestone-closure pass-1/pass-2, and phase-closure pass-1/pass-2 production-grade reviews approved on 2026-03-21)
Owner: ad_hoc_collection_lowering_clone_elision execution loop
Reference planning doc:
- `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [x] Entry baseline validated before wave 0
- [x] Scope remains constrained to active wave
- [x] Root cause is fixed without compatibility shims
- [x] Positive-path and negative-path validation recorded for each wave
- [x] Demo runs before opening each wave PR
- [x] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [x] PR opened/reviewed/merged before next wave starts
- [x] Docs + traceability + roadmap/issue state updated before moving on

## Full Phase To-Do Plan
1. [x] `wave_clone_0`: architecture lock, inventory, and execution-baseline capture
2. [x] `wave_clone_1`: iterator/comprehension clone-elision through shared planner
3. [x] `wave_clone_2`: indexing/slicing/star-unpack ownership correction
4. [x] `wave_clone_3`: generic hardening, regression lock, and phase closure
5. [x] wave-level extra completion review cycle done
6. [x] wave-level extra production-grade review cycle done
7. [x] milestone-level completion review cycle done
8. [x] milestone-level production-grade review cycle done
9. [x] phase-level completion review cycle done
10. [x] phase-level production-grade review cycle done

## Entry Baseline Evidence (2026-03-21)

Baseline commands:
- `scripts/run_all_tests.sh --profile quick`
- `cargo run -q -p sifr -- emit demos/milestone_generics_demo.sifr`
- `cargo run -q -p sifr -- emit demos/milestone_ergonomics_demo.sifr`
- `cargo run -q -p sifr -- emit demos/milestone_safe_indexing_demo.sifr`
- `cargo run -q -p sifr -- emit demos/milestone_control_flow_demo.sifr`

Required baseline records:
- generated-code evidence for current clone-heavy patterns:
  - `.clone().into_iter()`
  - `.iter().cloned()` on borrowed `Copy` collections
  - `.get(...).cloned()` on borrowed `Copy` collections
  - whole-source `clone()` before star-unpack
  - per-element `.clone()` in stepped slicing for `Copy` types
  - boxed range iteration such as `Box::new((range).clone().into_iter())`
- exact implementation ownership of those patterns in:
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
- explicit residual note that borrowed move-heavy collections remain out of scope for full CPython parity unless runtime representation changes later

Suggested baseline capture helpers:
- `cargo run -q -p sifr -- emit ... | grep -E '\\.clone\\(\\)\\.into_iter\\(|\\.iter\\(\\)\\.cloned\\(|\\.get\\([^)]*\\)\\.cloned\\(|Box::new\\(\\([^)]*\\.clone\\(\\)\\.into_iter\\('`
- keep the full emitted output alongside grep-filtered excerpts; grep is only for scanability

Observed baseline result before `wave_clone_1` edits:
- quick lane command: `scripts/run_all_tests.sh --profile quick`
- lane result: PASS
  - HIR + `sifr_driver` maintainability guardrails: PASS
  - `cargo test -p sifr -- --skip test_e2e_pass`: PASS (`37` tests)
  - e2e fail/runtime lane: PASS (`25` tests)
  - validation contract matrix (`frontend_mode_parity`, `phase23_graph_isolation`): PASS (`7` rows)
  - e2e pass quick suite: PASS (`24` fixtures, signature `e1bf653aaa770517`)
- clone-heavy emit evidence and ownership mapping locked in:
  - `verification/stdlib/wave_clone_0_codegen_traceability.md`
- residual boundary lock:
  - borrowed move-heavy parity for CPython-style reference semantics is out of scope for this phase unless runtime representation changes later

## Implementation Inventory

Primary code paths that must be reviewed in this phase:

- `crates/sifr_codegen/src/helpers.rs`
  - add shared planner / helper entry points
- `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `lower_comprehension_iter_for_ir`
  - `try_lower_for_iter_expr_for_ir`
  - `lower_iter_source_expr_for_ir`
  - iterator/enumerate special cases
  - boxed range iteration paths in structural range lowering (`for` / comprehensions / similar structural chains)
- `crates/sifr_codegen/src/lower_expr.rs`
  - simple `map` lowering
  - simple `filter` lowering
  - list/dict/generator comprehension simple lowering
  - iterator-chain simple lowering
- `crates/sifr_codegen/src/lower_stmt.rs`
  - `try_lower_simple_star_unpack_stmt`
  - simple indexing / safe-indexing / slicing helpers in scope, including dict/list `.get(...).cloned()` paths for `Copy` values
- `crates/sifr_codegen/src/ir_optimize.rs`
  - confirm it remains a narrow post-pass and does not become the primary semantic fix path

Secondary review paths:

- `crates/sifr_type_system/src/types.rs`
- `crates/sifr_codegen/src/function_emitter.rs`
- `crates/sifr_codegen/src/method_call_emitter.rs`

## Wave Progress

### wave_clone_0: Architecture Lock, Inventory, and Baseline
- Status: completed
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1394 (merged)
- Scope:
  - capture the generated-code baseline and inventory all targeted clone-heavy patterns
  - lock one canonical ownership-aware planner design before broad edits
  - define the concrete helper contract that classifies `HirExpr` into planner-facing value categories
  - record exact ownership semantics for:
    - place vs temporary
    - `Copy` vs `Move`
    - source preserve vs source consume
    - element yield mode (`Copy` / `Clone` / `Move` / `Borrow`)
    - conservative generic handling (`TypeVar`, `Any`, unions with move members)
  - confirm that both structured IR lowering and simple lowering must route through the same planner
- Deliverables:
  - phase execution ledger populated with baseline evidence
  - architecture comments / helper-module notes describing the planner contract
  - explicit `ValueCategory` classification rules recorded for the planner (`Place | Temporary`)
  - explicit `SourceAccessMode` and `YieldMode` planner contracts recorded
  - emitted-Rust before-state artifacts for representative demos
- Validation target:
  - `scripts/run_all_tests.sh --profile quick`
  - `cargo run -q -p sifr -- emit ...` for representative demos/fixtures
  - at least one positive demo and one negative ownership-safety fixture selected for later regression locking
- Validation evidence:
  - `scripts/run_all_tests.sh --profile quick` -> PASS
  - emit baseline captures:
    - `cargo run -q -p sifr -- emit demos/milestone_generics_demo.sifr`
    - `cargo run -q -p sifr -- emit demos/milestone_ergonomics_demo.sifr`
    - `cargo run -q -p sifr -- emit demos/milestone_safe_indexing_demo.sifr`
    - `cargo run -q -p sifr -- emit demos/milestone_control_flow_demo.sifr`
  - positive wave-lock fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/wave_clone_0_architecture_lock.sifr` -> PASS
  - positive wave-lock demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_clone_wave0_architecture_lock_demo.sifr` -> PASS (`ad_hoc_clone_wave0_architecture_lock_demo: pass`)
  - negative ownership-safety anchor:
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/borrow_escape_store.sifr` -> expected compile failure (PASS)
  - baseline artifact:
    - `verification/stdlib/wave_clone_0_codegen_traceability.md`

### wave_clone_1: Iterator and Comprehension Ownership Correction
- Status: completed
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1395 (merged)
- Scope:
  - implement the shared planner in `helpers.rs` or an equivalent focused support module
  - refactor iterator/comprehension lowering to derive decisions from planner output
  - remove `.clone().into_iter()` for owned temporary collection pipelines
  - remove boxed range clone paths where structural range iteration can lower natively without ownership noise
  - emit copy-oriented iteration for borrowed `Copy` element collections
  - keep borrowed move-element iteration semantically correct
- Required files:
  - `crates/sifr_codegen/src/helpers.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- Validation target:
  - positive: representative `for`, `map`, `filter`, and comprehension demos/fixtures run successfully
  - negative: named borrowed containers are not implicitly consumed
  - emit inspection: targeted outputs no longer contain `.clone().into_iter()` in these paths
  - emit inspection: targeted structural `Range` loops/comprehensions no longer emit `Box::new((range).clone().into_iter())`
  - emit inspection: borrowed `Copy` iteration no longer lowers through `.iter().cloned()` and instead uses copy-oriented iteration such as `iter().copied()` or an equivalent zero-clone shape
  - wave gate: `scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - wave fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/wave_clone_1_iterator_comprehension_ownership.sifr` -> PASS (`wave_clone_1_iterator_comprehension_ownership: pass`)
  - wave demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_clone_wave1_iterator_comprehension_demo.sifr` -> PASS (`ad_hoc_clone_wave1_iterator_comprehension_demo: pass`)
  - emit checks:
    - `cargo run -q -p sifr -- emit demos/milestone_control_flow_demo.sifr`
      - copy-oriented loop evidence: `for n in nums.iter().copied()`
      - structural range evidence: `for i in 1 as i64..n + (1 as i64)` (no boxed range clone path)
    - `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/wave_clone_1_iterator_comprehension_ownership.sifr`
      - temporary map source evidence: `vec![...].into_iter().map(...)` (no source clone-before-into_iter)
      - enumerate evidence: `Box::new((nums).iter().copied().enumerate().map(...))`
  - validation lanes:
    - `scripts/run_all_tests.sh --profile quick` -> PASS
    - `scripts/run_all_tests.sh` -> PASS
  - wave traceability artifact:
    - `verification/stdlib/wave_clone_1_iterator_codegen_traceability.md`
  - external review pass 1:
    - artifact: `reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-wave-clone-1-review-pass-1.md`
    - applied actions: added explicit planner regression tests for `YieldMode::Clone` and conservative `YieldMode::Borrow` fallback
    - deferred to `wave_clone_3`: tuple-literal `ValueCategory` classification hardening
  - external review pass 2 (production-grade check):
    - artifact: `reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-wave-clone-1-review-pass-2.md`
    - validated result: no high/medium wave-1 blockers; production-grade approved for `wave_clone_1`
    - deferred to `wave_clone_3` (optional generated-code polish): normalize redundant `.copied().collect()` chains

### wave_clone_2: Indexing, Safe Indexing, Slicing, and Star-Unpack Ownership Correction
- Status: completed
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1398 (merged)
- Scope:
  - refactor indexing and safe-indexing extraction to use ownership-aware plans
  - remove `Copy`-element `.clone()` / `.cloned()` in targeted indexing paths
  - remove whole-source clone in simple star-unpack lowering
  - preserve correct clone/copy behavior for move-element slices and unpacking
- Required files:
  - `crates/sifr_codegen/src/helpers.rs`
  - `crates/sifr_codegen/src/expr_render_helpers.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- Validation target:
  - positive: indexing / safe-indexing / slicing / star-unpack fixtures and demos run successfully
  - negative: ownership-safety regressions remain rejected
  - emit inspection: targeted outputs no longer contain whole-source star-unpack clone
  - emit inspection: `Copy`-element indexing no longer uses `.clone()` / `.cloned()` in targeted paths and instead uses copy-oriented extraction such as direct copy-out or `.copied()` where applicable
  - wave gate: `scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - wave fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/wave_clone_2_index_slice_unpack_ownership.sifr` -> PASS (`wave_clone_2_index_slice_unpack_ownership: pass`)
  - wave demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_clone_wave2_index_slice_unpack_demo.sifr` -> PASS (`ad_hoc_clone_wave2_index_slice_unpack_demo: pass`)
  - emit checks:
    - `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/wave_clone_2_index_slice_unpack_ownership.sifr`
      - copy-safe indexing evidence: `scores.get(\"alice\").copied()`
      - move-element indexing evidence: `__sifr_index_list.get(__sifr_index_norm).cloned()`
      - star-unpack evidence: `let _star_tmp = &nums;` and no `let _star_tmp = <source>.clone();`
      - stepped copy-slice evidence: `_result.push(*_el);`
  - wave traceability artifact:
    - `verification/stdlib/wave_clone_2_index_slice_unpack_traceability.md`
  - external review pass 1:
    - artifact: `reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-wave-clone-2-review-pass-1.md`
    - applied actions: updated stale unit-test expectations to assert copy-oriented behavior (`copied` and direct `Index`) instead of old clone-heavy shapes
    - validation: `cargo test -p sifr_codegen simple_compare_condition_wraps_proven_list_index_without_double_option`, `cargo test -p sifr_codegen test_self_field_clone_suppression_is_scoped_and_non_sticky`, `scripts/run_all_tests.sh`
  - external review pass 2 (production-grade check):
    - artifact: `reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-wave-clone-2-review-pass-2.md`
    - applied actions: updated `lowers_simple_for_with_else_and_name_iter` to assert `.copied()` for `list[int]` named-iterator lowering
    - deferred to `wave_clone_3`: tuple ownership hardening in `Type::ownership()` (copy tuples should classify as `OwnershipKind::Copy`)

### wave_clone_3: Generic Hardening, Regression Lock, and Closure
- Status: completed
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1402 (merged)
- Scope:
  - harden `TypeVar`, `Any`, and union cases under the shared planner
  - ensure no unsound `.copied()` lowering is emitted for conservative types
  - add regression coverage for generated Rust shape and runtime behavior
  - document residual move-heavy parity limits explicitly in architecture/phase docs
- Required files:
  - `crates/sifr_codegen/src/helpers.rs`
  - `crates/sifr_type_system/src/types.rs`
  - targeted tests under `crates/sifr_codegen`, `crates/sifr_hir`, and `crates/sifr/tests/e2e`
  - `crates/sifr/tests/e2e/pass/wave_clone_3_generic_hardening_ownership.sifr`
  - `demos/ad_hoc_clone_wave3_generic_hardening_demo.sifr`
  - `verification/stdlib/wave_clone_3_generic_hardening_traceability.md`
  - `internal_docs/architecture.md`
  - `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md`
- Validation target:
  - positive: generic-safe fixtures and demos behave correctly
  - negative: conservative generic ownership remains explicit and deterministic
  - targeted generated-code assertions cover current high-value clone-heavy patterns
  - `cargo fmt --check`
  - `cargo clippy --workspace -- -D warnings`
  - `python3 scripts/check_hir_maintainability_guardrails.py`
  - `scripts/run_all_tests.sh`
- Validation evidence:
  - targeted type-system ownership hardening:
    - `cargo test -p sifr_type_system test_tuple_ownership_all_copy_is_copy` -> PASS
    - `cargo test -p sifr_type_system test_tuple_ownership_with_move_is_move` -> PASS
  - targeted planner hardening:
    - `cargo test -p sifr_codegen -- helpers::tests` -> PASS
      - `iterator_plan_preserved_list_any_uses_borrow_not_clone` confirms `list[Any]` iteration stays borrow-based
      - `iterator_plan_copies_tuple_of_copy_elements` confirms copy-yield for `list[tuple[int, int]]`
  - wave fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/wave_clone_3_generic_hardening_ownership.sifr` -> PASS (`wave_clone_3_generic_hardening_ownership: pass`)
  - wave demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_clone_wave3_generic_hardening_demo.sifr` -> PASS (`ad_hoc_clone_wave3_generic_hardening_demo: pass`)
  - emit checks:
    - `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/wave_clone_3_generic_hardening_ownership.sifr`
      - tuple copy iteration evidence: `for pair in pairs.iter().copied()`
      - conservative `Any` iteration evidence: `for _v in anys.iter()` (no `.cloned()` / `.copied()`)
  - wave traceability artifact:
    - `verification/stdlib/wave_clone_3_generic_hardening_traceability.md`
  - external review pass 1:
    - artifact: `reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-wave-clone-3-review-pass-1.md`
    - applied actions: documented conservative-typing invariants on `is_conservative_element_type` so `Any`/`Unknown` handling and `TypeVar` separation are explicit for maintainers
    - validation: `scripts/run_all_tests.sh`
  - external review pass 2 (production-grade check):
    - artifact: `reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-wave-clone-3-review-pass-2.md`
    - applied actions: no wave-scoped code changes required; review approved production readiness and confirmed pass-1 follow-up closure
    - follow-up tracking (out of wave scope): pre-existing `phase_psp_iter_fix_7_user_defined_iterable_protocol` dangling-reference fix and pre-existing unrelated unit-test failures remain tracked separately
    - validation: `scripts/run_all_tests.sh`

## Wave Closure Review Cycles

- wave closure completion review: completed (`reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-wave-closure-review-pass-1.md`)
  - applied actions:
    - documented canonical ownership-aware collection lowering decision tree in `internal_docs/architecture.md`
    - recorded residual boundary note (clone-elision closure does not claim full CPython move-heavy runtime parity)
    - marked global gate `Root cause is fixed without compatibility shims` as complete
  - validation:
    - `scripts/run_all_tests.sh`
- wave closure production-grade review: completed (`reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-wave-closure-review-pass-2.md`)
  - applied actions:
    - confirmed pass-1 finding closure and architecture documentation completeness
    - retained phase/milestone closure status as in-progress pending mandated downstream closure cycles
  - validation:
    - `scripts/run_all_tests.sh`

## Milestone Closure Review Cycles

- milestone closure completion review: completed (`reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-milestone-closure-review-pass-1.md`)
  - applied actions:
    - confirmed milestone-level readiness with no open correctness findings
    - retained `closed` status flips for phase/roadmap/execution until milestone pass-2 and phase-level closure cycles complete
  - validation:
    - `scripts/run_all_tests.sh`
- milestone closure production-grade review: completed (`reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-milestone-closure-review-pass-2.md`)
  - applied actions:
    - confirmed milestone production-grade readiness with no open findings
    - retained phase-level `closed` status flips and roadmap closure wording until phase closure review cycles are completed
  - validation:
    - `scripts/run_all_tests.sh`

## Phase Closure Review Cycles

- phase closure completion review: completed (`reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-phase-closure-review-pass-1.md`)
  - applied actions:
    - confirmed phase-closure readiness with no open correctness findings
    - deferred final `closed` status flips (phase doc/execution/roadmap) to phase-closure production-grade pass for loop-consistent closure sequencing
  - validation:
    - `scripts/run_all_tests.sh`
- phase closure production-grade review: completed (`reviews/phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision-phase-closure-review-pass-2.md`)
  - applied actions:
    - finalized status-line closure updates in phase doc, execution ledger, and roadmap entry wording
    - marked phase-level production-grade cycle checklist item complete
  - validation:
    - `scripts/run_all_tests.sh`

## Suggested Regression Targets

Generated-code contract checks should explicitly cover:

- `demos/milestone_generics_demo.sifr`
- `demos/milestone_ergonomics_demo.sifr`
- `demos/milestone_safe_indexing_demo.sifr`
- `demos/milestone_control_flow_demo.sifr`

Behavioral fixtures should include:

- list iteration over `int`, `bool`, and `str`
- map/filter over named and temporary containers
- list/dict comprehension over named and temporary containers
- direct indexing and safe indexing over `Copy` and move-element containers
- star-unpack and stepped slicing
- at least one conservative generic case (`TypeVar` or `Any`)

## Closure Requirements

Before phase closure:

- update `internal_docs/architecture.md` with the canonical ownership-aware collection lowering rule
- update the phase doc with merged PR links and closure notes
- record residual-risk notes describing which remaining CPython gaps are lowering-bound vs representation-bound
