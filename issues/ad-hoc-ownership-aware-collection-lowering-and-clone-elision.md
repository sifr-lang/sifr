# Ad Hoc Phase: Ownership-Aware Collection Lowering and Clone Elision

Status: closed (started 2026-03-21; `wave_clone_0` architecture lock/baseline, `wave_clone_1` iterator/comprehension ownership correction, `wave_clone_2` index/slice/star-unpack ownership correction, and `wave_clone_3` generic hardening/regression lock completed on 2026-03-21; wave-closure pass-1/pass-2, milestone-closure pass-1/pass-2, and phase-closure pass-1/pass-2 production-grade reviews approved on 2026-03-21)
Context: corrective post-closure phase after the full ad hoc parity follow-up sequence covering lazy iterators, waiver reduction, structured/class surfaces, bytes foundations, runtime/file objects, canonical iteration closure, and stateful RNG/crypto/polish expansion
Execution readiness: design-ready after completion of the following predecessor phases:
  - `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol.md`
  - `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`
  - `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
  - `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
  - `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
  - `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`
  - `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`
This phase is implementation-ready: execution ledger is active, wave-0 architecture/baseline lock is completed, wave-1 iterator/comprehension ownership correction is completed, wave-2 indexing/slicing/star-unpack ownership correction is completed, and wave-3 generic hardening/regression lock is completed.
Execution ledger: `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision-execution.md`

## Objective

Remove unnecessary `.clone()` insertion in generated Rust by fixing the root cause: collection, indexing, slicing, star-unpack, and iterator lowering currently collapse distinct ownership cases into clone-heavy fallback paths.

This phase does not treat clone removal as a cosmetic optimization pass. The goal is to make generated Rust reflect Sifr's ownership model correctly:

- `Copy` values should copy, not clone
- borrowed move values should only clone when Sifr semantics actually require an owned result
- temporary owned containers should be consumed directly instead of cloned before iteration
- named borrowed containers must not be silently consumed

Primary target area:

- compiler lowering and codegen for collection access and iteration

Primary surface targets:

- `for ... in ...`
- `iter`
- `map`
- `filter`
- list / dict / string indexing
- safe indexing
- slicing
- star-unpack
- list / dict / generator comprehensions

Secondary target area:

- generated-code quality and performance evidence for collection-heavy paths

## Source of Truth

- architecture baseline:
  - `internal_docs/architecture.md`
  - `internal_docs/phases/10_borrow_by_default.md`
  - `internal_docs/phases/30_reliability_parity_and_performance_budgets.md`
  - `internal_docs/phases/34_generated_code_quality_and_production_readiness.md`
- predecessor ad hoc sequence:
  - `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol.md`
  - `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`
  - `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
  - `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
  - `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
  - `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`
  - `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`
- relevant implementation hotspots:
  - `crates/sifr_type_system/src/types.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_hir/src/lower/statements.rs`
  - `crates/sifr_codegen/src/helpers.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/function_emitter.rs`
  - `crates/sifr_codegen/src/method_call_emitter.rs`
  - `crates/sifr_codegen/src/ir_optimize.rs`
- evidence and demos exposing current clone-heavy output:
  - `demos/milestone_generics_demo.rs`
  - `demos/milestone_ergonomics_demo.rs`
  - `demos/milestone_safe_indexing_demo.rs`
  - `demos/milestone_control_flow_demo.rs`
- CPython reference tree:
  - `/Users/yaseralnajjar/work/sifr/cpython`
- wave-0 baseline and lock artifact:
  - `verification/stdlib/wave_clone_0_codegen_traceability.md`

Primary upstream families for reference only:

- `Objects/listobject.c`
- `Objects/tupleobject.c`
- `Objects/dictobject.c`
- `Lib/test/test_iter.py`
- `Lib/test/test_list.py`
- `Lib/test/test_tuple.py`
- `Lib/test/test_dict.py`

## Why This Needs Its Own Phase

The current problem is architectural, not a set of isolated bad codegen lines.

Today, several lowering paths ask an underspecified question:

- "How do I produce an owned Rust value here?"

But the compiler should instead distinguish at least these cases:

- borrowed container + `Copy` element
- borrowed container + `Move` element
- owned temporary container + `Move` element
- named place expression that must not be implicitly consumed
- generic / `TypeVar` / `Any` cases where ownership must remain conservative

Because these cases are collapsed too early, the current implementation falls back to:

- `.clone().into_iter()`
- `.iter().cloned()`
- `.get(...).cloned()`
- `value.clone()` before star-unpack

even where Sifr already has enough type information to do better.

That makes this a compiler-semantics cleanup phase with performance wins, not a late peephole optimization pass.

It also explains why this phase belongs after the earlier ad hoc sequence instead of being folded into it retroactively:

- the lazy-iterator and canonical-iteration phases established the iterable model and lazy/eager boundary,
- the bytes/runtime/file-object and RNG/crypto/polish phases expanded higher-level surfaces on top of that model,
- and only after those phases shipped is the remaining debt clearly isolated as ownership-aware lowering quality rather than missing feature support.

## Depends on

- `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol.md`
- `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`
- `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
- `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
- `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
- `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`
  - iteration semantics must remain canonical; this phase refines ownership-sensitive lowering inside that model rather than redesigning it
- `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`
  - this phase is intentionally later and narrower than the broad parity-expansion sequence; it hardens generated-code quality after those surfaces are already in place
- Phase 27 non-regression invariants remain mandatory
- Phase 29 local-first validation contract remains mandatory
- Phase 30 performance-budget discipline remains mandatory
- earlier phases should already have closed broad feature-surface gaps, leaving ownership-aware lowering quality as the bounded remaining problem

## Problem Statement

Sifr already models ownership and borrowing well enough to avoid many of the currently emitted clones:

- `Type::ownership()` already distinguishes `Copy` vs `Move`
- default parameter convention logic already treats `Copy` values differently from move values
- parameter and call emission already avoid unnecessary borrows for `Copy` values

The gap is specifically in collection and iterator lowering, where ownership information is not propagated into a single explicit planning step.

That leads to three bad outcomes:

1. `Copy` element paths pay clone-shaped overhead even when a plain copy or `.copied()` is enough.
2. Temporary owned containers are cloned before iteration instead of consumed directly.
3. The code generator hides unresolved ownership distinctions behind cloning rather than making the ownership decision explicit.

## Language and Runtime Contract for This Phase

### Canonical lowering rule

Collection lowering in this phase must derive from an explicit ownership-aware plan rather than ad hoc local branching.

Illustrative internal shape:

```rust
enum ValueCategory {
    Place,
    Temporary,
}

enum SourceAccessMode {
    Preserve,
    Consume,
}

enum YieldMode {
    Copy,
    Clone,
    Move,
    Borrow,
}
```

The exact enum names are not important. The invariant is:

- classify the expression as `Place` or `Temporary`
- decide whether the source container must be preserved or may be consumed
- decide whether elements must be copied, cloned, moved, or only borrowed

Any concrete iterator/access plan should be derived from those axes rather than from one overloaded "owned vs borrowed vs consumed" switch.

### Required decision inputs

Every collection-read / iteration lowering decision in this phase must consider:

- the container expression category:
  - named place / field / reusable lvalue
  - temporary value produced in the current expression
- the source access contract:
  - preserve the source container
  - consume the source container
- the element ownership kind:
  - `Copy`
  - `Move`
- the element-yield contract:
  - copy the element out
  - clone the element out
  - move the element out
  - borrow the element only
- whether the type is concrete or conservative:
  - `TypeVar`
  - `Any`
  - unions containing move members

`ValueCategory` must be derived by one explicit helper rather than informal caller judgment. In particular, wave 0 must lock how the planner classifies at least:

- names and other reusable places,
- field / index / attribute-style place-like expressions where relevant,
- constructor calls and other one-shot temporaries,
- expressions whose values may not be implicitly consumed because later reuse is semantically observable.

This phase intentionally keeps `ValueCategory` minimal:

- `Place`
- `Temporary`

It does not introduce a separate `BorrowedPlace` variant. Borrowing vs consuming is a planner decision captured by `SourceAccessMode`, not by the category enum itself.

### Canonical iteration rules in this phase

- temporary owned containers may lower with direct `into_iter()` where that matches Sifr ownership semantics
- iterator-pipeline consumption and source-container consumption are separate decisions; exhausting `sum(...)`, `min(...)`, `max(...)`, or a `for` loop does not by itself imply that a named source container was consumed
- borrowed containers with `Copy` element types should lower to copy-oriented access:
  - `iter().copied()`
  - direct deref / copy-out
  - equivalent zero-clone Rust shapes
- borrowed containers with `Move` element types may lower to `iter().cloned()` only where the Sifr construct semantically yields owned values
- named place expressions must not be silently consumed just to avoid cloning

### Canonical indexing and slicing rules in this phase

- indexing a borrowed collection of `Copy` values must not emit `.clone()`
- safe indexing of borrowed collections of `Copy` values must prefer `.copied()` over `.cloned()`
- star-unpack must not clone the whole source container up front
- slicing must clone or copy only the elements required by the result shape
- contiguous `Copy`-element slices should prefer copy-oriented Rust fast paths where the backend type supports them

### Range-specific rule in this phase

- structural loop/comprehension lowering for `Range` must not emit ownership noise such as `.clone()` or unnecessary boxing
- this applies to structural iteration contexts such as:
  - `for`
  - comprehensions
  - direct map/filter-style structural lowering where the chain is not being exposed as a first-class iterator object
- this rule does not by itself ban boxed iterator representations for first-class `Iterator[T]` expression results where the backend intentionally uses type erasure

### Generic and conservative cases

- `TypeVar` and `Any` remain conservative in this phase
- the compiler must not emit `.copied()` for generic element types unless the type system can prove `Copy`
- if Sifr semantics require an owned generic move value from a borrowed container, this phase must keep the ownership requirement explicit rather than hiding it behind a performance shortcut

### Explicit non-claim

This phase removes unnecessary clones. It does not by itself guarantee CPython parity for move-heavy workloads such as borrowed `list[str]` iteration, because Sifr's current runtime representation for move types is still owned Rust data, not CPython-style refcounted object handles.

If future profiling shows that borrowed move-heavy iteration remains materially slower than CPython after this phase, that should become a separate runtime-representation phase rather than expanding this one beyond scope.

## Scope

This phase owns:

- ownership-aware planner/helpers for collection access and iteration lowering
- refactoring existing lowering sites to route through that planner
- refactoring both IR-oriented and simple-lowering paths so clone decisions stop diverging between `stmt_support_emitter.rs` and `lower_expr.rs`
- removal of whole-container clones in iterator/comprehension/map/filter paths where ownership does not require them
- removal of `Copy`-type `.clone()` emissions in indexing and safe-indexing paths
- star-unpack and slice lowering fixes that avoid cloning more than semantics require
- regression coverage and generated-code assertions proving these clone-heavy patterns are gone from targeted outputs

This phase does not own:

- broad runtime representation redesign for `str`, `list`, `dict`, or class instances
- new public ownership syntax
- async iteration changes
- unrelated stdlib expansion
- benchmark theater that ignores semantic correctness

## Non-goals

- introducing hidden container consumption to make generated Rust look smaller
- using peephole IR optimization as the primary fix for ownership mistakes made earlier in lowering
- weakening conservative generic ownership rules
- claiming CPython parity for borrowed move-heavy collections without a runtime-representation change
- adding fallback code paths that choose between clone-heavy and clone-light behavior dynamically at runtime

## Priority Targets

### priority_1: Central ownership-aware lowering planner

Modules / crates:

- `sifr_codegen`
- `sifr_type_system`

Required closure direction:

- define one internal planner for collection access and iteration
- make lowering decisions derive from ownership, value category, source access mode, and yield mode
- stop re-encoding the same ownership decision separately in each lowering function
- ensure the planner is shared by both:
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`

### priority_2: Iterator and comprehension clone elimination

Surfaces:

- `for`
- `map`
- `filter`
- list / dict / generator comprehensions

Required closure direction:

- replace `.clone().into_iter()` on temporary-owned containers with direct consumption
- replace borrowed-`Copy` `.iter().cloned()` with copy-oriented lowering
- keep borrowed move-element iteration correct without hidden container consumption

### priority_3: Indexing, slicing, and unpacking clone elimination

Surfaces:

- indexing
- safe indexing
- stepped slicing
- star-unpack

Required closure direction:

- remove `.clone()` on `Copy` element extraction
- remove whole-source clone in star-unpack
- clone or copy only the result elements actually required

### priority_4: Validation and generated-code contract hardening

Required closure direction:

- add tests that fail if targeted clone-heavy output patterns regress
- add demo / emit evidence showing before-vs-after generated Rust shapes
- record explicit caveats where move-heavy parity still depends on later runtime work

## Waves

### wave_clone_0: Architecture Lock

- inventory every runtime-relevant clone pattern currently emitted in targeted surfaces
- identify the exact lowering entry points that own those decisions
- create the execution ledger and lock the implementation checklist before wave 1 code changes begin
- define the concrete helper contract that classifies `HirExpr` into planner-facing value categories
- lock the invariants for:
  - place vs temporary
  - source preserve vs source consume
  - `Copy` vs `Move`
  - element yield mode (`Copy` / `Clone` / `Move` / `Borrow`)
  - generic conservative handling
- document the planner design before broad code changes begin

Definition of done:

- execution ledger exists and is populated with the wave plan
- one canonical planner design is documented in code comments and phase notes
- no new direct clone-heuristic branches are added outside that model

### wave_clone_1: Iterator and Comprehension Ownership Correction

- refactor `for` lowering, iterator adaptation, map/filter lowering, and comprehension lowering to use the planner
- cover both:
  - structured IR lowering in `stmt_support_emitter.rs`
  - simple-lowering paths in `lower_expr.rs`
- eliminate `.clone().into_iter()` where the input is an owned temporary
- emit copy-oriented iteration for borrowed `Copy` element containers

Definition of done:

- targeted demos no longer show `.clone().into_iter()` for those paths
- borrowed `Copy` iteration no longer lowers through `.iter().cloned()`

### wave_clone_2: Indexing, Slicing, and Star-Unpack Ownership Correction

- refactor direct indexing and safe indexing to use ownership-aware extraction plans
- refactor slicing and star-unpack to avoid whole-container clones
- preserve semantics for borrowed move-element extraction

Definition of done:

- targeted demos no longer show `Copy`-element `.clone()` for indexing and safe indexing
- star-unpack no longer begins with `value.clone()` solely for backend convenience

### wave_clone_3: Generic Hardening, Validation, and Closure

- harden `TypeVar` / `Any` / union cases
- add regression tests for generated Rust shape and runtime behavior
- validate local performance-sensitive examples with `emit`, `check`, and `run`
- document residual move-heavy parity limits explicitly

Definition of done:

- conservative generic handling is explicit and tested across both planner inputs and the lowering call sites that consume planner output
- docs record what this phase fixed and what remains a later runtime concern
- full local validation passes

## Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | collection and iterator lowering decisions in scope derive from one explicit ownership-aware planning path |
| AC-2 | generated Rust no longer emits `.clone().into_iter()` for owned temporary collection pipelines in the targeted surfaces |
| AC-3 | borrowed collection iteration over `Copy` element types no longer emits `.iter().cloned()` in targeted surfaces and instead lowers through copy-oriented access such as `iter().copied()` or an equivalent zero-clone Rust shape |
| AC-4 | direct indexing and safe indexing of borrowed `Copy` collections no longer emit `.clone()` / `.cloned()` in targeted surfaces and instead lower through copy-oriented access such as direct copy-out or `.copied()` where applicable |
| AC-5 | star-unpack no longer clones the full source collection solely for lowering convenience |
| AC-6 | borrowed move-element cases remain semantically correct and do not silently consume named containers |
| AC-7 | `TypeVar` / `Any` handling remains conservative and does not introduce unsound copy-oriented lowering |
| AC-8 | generated-code regression coverage exists for at least the current demo-backed clone-heavy patterns |
| AC-9 | local validation passes via `scripts/run_all_tests.sh --profile quick` and `scripts/run_all_tests.sh` |
| AC-10 | documentation explicitly states that this phase removes unnecessary clones but does not claim full CPython parity for move-heavy runtime representations |

## Validation Requirements

Minimum validation evidence for each wave:

- targeted `cargo run -q -p sifr -- emit ...` output captured for representative fixtures
- targeted `cargo run -q -p sifr -- run ...` demos proving semantics remain correct
- unit or e2e regression coverage for every root-cause clone pattern removed
- `scripts/run_all_tests.sh --profile quick`

Phase-exit validation:

- `scripts/run_all_tests.sh`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`

## Demo / Fixture Targets

Representative targets for this phase should include:

- a demo showing list iteration over `int`, `bool`, and `str`
- a demo showing map/filter/comprehension over temporary and named containers
- a demo showing safe indexing and direct indexing over `Copy` and move-element lists
- a demo showing star-unpack and slicing without whole-container cloning
- at least one negative test proving named borrowed containers are not implicitly consumed

## Exit Notes

Before closing this phase:

- update `internal_docs/architecture.md` with the canonical ownership-aware collection lowering rule
- update any relevant roadmap / phase trackers with merged PR links and closure notes
- record a short residual-risk section describing which CPython performance gaps remain representation-bound rather than lowering-bound
