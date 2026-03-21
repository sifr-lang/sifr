# Ad Hoc Phase Execution Checklist (Ownership-Aware Collection Lowering and Clone Elision)

Status: proposed on 2026-03-21
Owner: ad_hoc_collection_lowering_clone_elision execution loop
Reference planning doc:
- `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [ ] Entry baseline validated before wave 0
- [ ] Scope remains constrained to active wave
- [ ] Root cause is fixed without compatibility shims
- [ ] Positive-path and negative-path validation recorded for each wave
- [ ] Demo runs before opening each wave PR
- [ ] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [ ] PR opened/reviewed/merged before next wave starts
- [ ] Docs + traceability + roadmap/issue state updated before moving on

## Full Phase To-Do Plan
1. [ ] `wave_clone_0`: architecture lock, inventory, and execution-baseline capture
2. [ ] `wave_clone_1`: iterator/comprehension clone-elision through shared planner
3. [ ] `wave_clone_2`: indexing/slicing/star-unpack ownership correction
4. [ ] `wave_clone_3`: generic hardening, regression lock, and phase closure
5. [ ] wave-level extra completion review cycle done
6. [ ] wave-level extra production-grade review cycle done
7. [ ] milestone-level completion review cycle done
8. [ ] milestone-level production-grade review cycle done
9. [ ] phase-level completion review cycle done
10. [ ] phase-level production-grade review cycle done

## Entry Baseline Evidence (to capture before wave 0 starts)

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
- Status: not started
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

### wave_clone_1: Iterator and Comprehension Ownership Correction
- Status: not started
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
- Validation target:
  - positive: representative `for`, `map`, `filter`, and comprehension demos/fixtures run successfully
  - negative: named borrowed containers are not implicitly consumed
  - emit inspection: targeted outputs no longer contain `.clone().into_iter()` in these paths
  - emit inspection: targeted structural `Range` loops/comprehensions no longer emit `Box::new((range).clone().into_iter())`
  - emit inspection: borrowed `Copy` iteration no longer lowers through `.iter().cloned()` and instead uses copy-oriented iteration such as `iter().copied()` or an equivalent zero-clone shape
  - wave gate: `scripts/run_all_tests.sh --profile quick`

### wave_clone_2: Indexing, Safe Indexing, Slicing, and Star-Unpack Ownership Correction
- Status: not started
- Scope:
  - refactor indexing and safe-indexing extraction to use ownership-aware plans
  - remove `Copy`-element `.clone()` / `.cloned()` in targeted indexing paths
  - remove whole-source clone in simple star-unpack lowering
  - preserve correct clone/copy behavior for move-element slices and unpacking
- Required files:
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/helpers.rs`
- Validation target:
  - positive: indexing / safe-indexing / slicing / star-unpack fixtures and demos run successfully
  - negative: ownership-safety regressions remain rejected
  - emit inspection: targeted outputs no longer contain whole-source star-unpack clone
  - emit inspection: `Copy`-element indexing no longer uses `.clone()` / `.cloned()` in targeted paths and instead uses copy-oriented extraction such as direct copy-out or `.copied()` where applicable
  - wave gate: `scripts/run_all_tests.sh --profile quick`

### wave_clone_3: Generic Hardening, Regression Lock, and Closure
- Status: not started
- Scope:
  - harden `TypeVar`, `Any`, and union cases under the shared planner
  - ensure no unsound `.copied()` lowering is emitted for conservative types
  - add regression coverage for generated Rust shape and runtime behavior
  - document residual move-heavy parity limits explicitly in architecture/phase docs
- Required files:
  - `crates/sifr_codegen/src/helpers.rs`
  - targeted tests under `crates/sifr_codegen`, `crates/sifr_hir`, and `crates/sifr/tests/e2e`
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
