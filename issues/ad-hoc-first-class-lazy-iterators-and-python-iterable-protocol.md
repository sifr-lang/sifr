# Ad Hoc Phase: First-Class Lazy Iterators and Python Iterable Protocol

Status: open (documented 2026-03-17)
Context: ad hoc architecture phase captured in `issues/` before roadmap-phase promotion
Execution readiness: implementation-ready in sequence as the iterator architecture phase for the post-milestone-31.5 parity sequence; wave 1 still requires recorded entry-baseline evidence in the execution issue

## Objective

Make lazy iteration a first-class Sifr language and runtime feature using the CPython iterable/iterator split as the reference model, while preserving Sifr's safety contract instead of copying CPython's runtime mutation and exception behavior.

This phase is architectural, not cosmetic. The goal is not to add a few lazy helpers. The goal is to replace the current eager stand-ins with one coherent model where:

- collections remain concrete reusable values,
- collections are iterable,
- iterators are separate single-pass stateful values,
- generator functions return iterators,
- Python-shaped lazy APIs return iterators rather than eagerly materialized lists,
- and explicit collection materialization remains visible in source through `list(...)`, `tuple(...)`, `set(...)`, and `dict(...)`.

Backward compatibility is explicitly out of scope for this phase. If current eager APIs conflict with the correct iterator architecture, the architecture wins.

## Source of Truth

This phase must use the following as authoritative references:

- CPython source tree:
  - `/Users/yaseralnajjar/work/sifr/cpython`
- CPython test corpus:
  - `/Users/yaseralnajjar/work/sifr/cpython/Lib/test`
- Current Sifr architecture baseline:
  - `internal_docs/architecture.md`
  - `internal_docs/phases/02_type_system_power.md`
  - `internal_docs/phases/07_stdlib_parity.md`
- Existing parity governance and waivers:
  - `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
  - `verification/stdlib/wave_psp_b2_cpython_traceability.md`

The CPython implementation points already validated for this phase are:

- `Objects/abstract.c`
  - `PyObject_GetIter`
  - `PyIter_Check`
  - `PyIter_Next`
- `Python/bltinmodule.c`
  - `builtin_next`
  - `zip_new`
  - `zip_next`
- `Objects/listobject.c`
  - `list_iter`
  - `listiter_next`
- `Objects/rangeobject.c`
  - `range_iter`
  - `rangeiter_next`
- `Objects/dictobject.c`
  - `dict_iter`
  - `dictiter_new`
- `Objects/enumobject.c`
  - `enum_new_impl`
  - `enum_next`
  - `reversed_new_impl`
  - `reversed_next`
- `Objects/genobject.c`
  - `gen_iternext`

These sources establish the reference architecture:

1. `iter(x)` produces a separate iterator object.
2. `next(x)` accepts only iterators, not arbitrary iterables.
3. Collections and iterators are distinct runtime shapes.
4. Lazy builtins like `zip`, `enumerate`, `reversed`, and generators are concrete iterator objects that carry internal state.
5. CPython keeps a legacy sequence fallback for `iter(x)` when no iterator hook exists. Sifr should not copy that fallback in the first implementation because explicit iterable conformance is cleaner and safer.

## Why This Needs Its Own Phase

The earlier stdlib parity work deliberately deferred broad lazy iterator parity because the runtime and type-system contract was not closed. That was the correct deferment at the time, but it leaves a large architectural gap:

- generator functions are not fully modeled as iterators,
- lazy stdlib helpers are still implemented as eager collections,
- builtin iteration semantics are not yet defined around first-class iterator types,
- Python-shaped code that expects iterator-returning APIs still encounters architectural mismatch,
- and CPython test parity for iterator behavior is difficult to validate while the object model is still wrong.

This is not a module-by-module cleanup task. It is a language/runtime phase.

## Depends on

- Existing ownership and borrow-checking architecture remains in force.
- Phase 27 non-regression invariants remain mandatory.
- Phase 29 local-first validation contract remains mandatory.
- The completed ad hoc CPython surface-parity phase remains valid as the baseline; this phase extends architecture rather than reopening already-closed work without cause.

## Recommended Placement

- Execute as an ad hoc interstitial architecture phase before broader new stdlib parity waves that would otherwise build on the wrong eager foundation.
- Treat this phase as the prerequisite for any future attempt at full iterator/lazy-object parity across builtin and stdlib surfaces.

## Full-Closure Target

This phase is complete only when all of the following are true:

1. Sifr has first-class `Iterable[T]` and `Iterator[T]` concepts in the type system and runtime model.
2. `iter(x)` and `next(it)` exist as real builtin surfaces with coherent lowering, typing, and codegen.
3. `for` loops are expressed through the iterable/iterator protocol rather than ad hoc per-container lowering.
4. Generator functions return iterator values rather than eager collections.
5. Core lazy builtins and the initial `itertools` subset return iterators rather than lists.
6. CPython-derived iterator tests are ported or explicitly adapted/waived with rationale.
7. Remaining lazy-gap surfaces are classified explicitly rather than left vague.

## Scope

This ad hoc phase owns:

- first-class iterable and iterator protocol design,
- `iter(...)` builtin semantics,
- `next(...)` builtin semantics,
- `for` loop lowering through the protocol,
- generator lowering to lazy stateful iterator objects,
- conversion of selected builtins and `itertools` helpers from eager to lazy behavior,
- CPython iterator-parity test selection and adaptation,
- explicit classification of retained unsupported iterator families.

This ad hoc phase does not own:

- async iterators,
- exact CPython exception behavior,
- CPython's legacy sequence fallback for `iter(x)`,
- unrelated reflective stdlib surfaces,
- full conversion of every iterator-adjacent stdlib API in the same first spike.

## Non-goals

- preserving current eager return types when they conflict with the correct design,
- copying CPython's runtime mutation-during-iteration behavior,
- exposing `StopIteration` as a Sifr user-facing control-flow boundary,
- implementing every advanced iterator family in the same first milestone,
- keeping implicit iterator-to-list materialization behavior.

## Core Design Decisions

### 1. Python's iterable/iterator split is the correct surface model

- `list`, `tuple`, `dict`, `set`, `str`, and `range` remain concrete reusable value types.
- Those types become iterable.
- `Iterator[T]` is a separate first-class single-pass type.
- `iter(iterator)` is idempotent.

### 2. `Iterable[T]` and `Iterator[T]` must be first-class in the type system

The spike should not rely on ad hoc compiler special cases forever. The public model should be explicit:

- `Iterable[T]` provides `__iter__() -> Iterator[T]`
- `Iterator[T]` provides `__next__() -> Option[T]`
- `Iterator[T]` also satisfies the iterable contract

### 3. `next()` is iterator-only and returns `Option[T]`

CPython's `next()` accepts only iterator objects. Sifr should preserve that shape, but adapt the boundary from exceptions to `Option[T]`.

- `next(iterable)` should not type-check unless the value is already an iterator.
- exhaustion is represented as `None`, not `StopIteration`.

### 4. Explicit materialization is required

Iterator pipelines must not silently allocate collections.

- `zip(...)`, `enumerate(...)`, `reversed(...)`, generators, and lazy `itertools` helpers return iterators.
- materialization is explicit through `list(...)`, `tuple(...)`, `set(...)`, and `dict(...)`.

### 5. Sifr safety overrides CPython runtime quirks

CPython permits runtime mutation patterns that are either tolerated or rejected dynamically depending on the container. Sifr should not emulate those runtime differences.

The correct Sifr direction is:

- borrowed collection iterators participate in ownership and exclusivity rules,
- mutation while an incompatible live iterator borrow exists is rejected at compile time where possible,
- otherwise the unsupported boundary must be explicit and deterministic.

### 6. CPython sequence-fallback iteration is intentionally deferred

CPython allows `iter(x)` to fall back to sequence indexing if `__iter__` is absent. That compatibility rule exists for a dynamic runtime with decades of legacy behavior. It is not required for the first correct Sifr architecture and should remain out of scope unless later parity evidence proves it is worth adding.

## Execution Model

- This remains an ad hoc issue-driven phase until promoted into `internal_docs/phases/`.
- Work executes one wave at a time.
- Every wave must port or classify the relevant CPython tests before closure.
- Every wave must complete the same external review loop used in the recent parity work:
  - completion-gap review,
  - production-grade review,
  - validated fixes,
  - PR,
  - merge,
  - closure status update.
- No wave is complete just because the feature seems to work locally. Review confirmation and parity accounting are hard gates.

## Reviewer Gate

A wave is not complete when the implementer believes the iterator behavior is "good enough".

A wave is complete only when the reviewer explicitly confirms all of the following:

- the implementation matches the documented iterable/iterator surface,
- CPython parity gaps for the wave are either closed or explicitly waived,
- CPython test parity for the wave is either ported or explicitly waived,
- Sifr principles are preserved,
- no eager workaround architecture remains in the surfaces claimed by the wave,
- implementation quality is production-grade and deterministic.

## Internal Model Contract

- Collections remain reusable and multi-pass.
- Iterator values are single-pass and stateful.
- `for item in collection` must not consume the collection itself.
- `for item in iterator` advances the iterator state.
- Generator functions return iterators.
- Lazy builtins store iterator state internally rather than materializing full result collections.
- `next()` operates on iterators only and returns `Option[T]`.
- No implicit eager collection materialization is allowed in iterator-returning APIs.

## Waves

### wave_iter_1: Iterator Protocol and Type-System Contract

Scope:

- introduce `Iterable[T]` and `Iterator[T]` into the type system and HIR,
- define protocol contracts for `__iter__` and `__next__`,
- make `Iterator[T]` satisfy the iterable contract,
- update architecture docs and parity governance to replace the current lazy-iterator waiver with this new execution plan.

Definition of done:

- iterable and iterator types are first-class concepts rather than undocumented compiler special cases,
- architecture docs clearly define the intended surface and the intentional Sifr deviations from CPython,
- the repo has an explicit traceability and waiver model for this phase.

### wave_iter_2: Builtin Protocol Entry and `for` Lowering

Scope:

- add `iter(...)` builtin lowering, typing, and codegen,
- add `next(...)` builtin lowering, typing, and codegen,
- rewrite `for` lowering to use the iterable/iterator protocol,
- define and enforce borrow-safe behavior for collection-backed iterators.

Definition of done:

- `for` loops no longer depend on ad hoc collection-only lowering,
- `iter(...)` and `next(...)` work coherently across supported iterable and iterator values,
- invalid iterator misuse or borrow-conflicting mutation is rejected deterministically.

### wave_iter_3: Generator Rewrite

Scope:

- replace eager generator buffering with lazy state-machine or equivalent iterator lowering,
- change generator-return typing from eager collection to iterator,
- align generated Rust with a true iterator model.

Definition of done:

- generator functions are genuinely lazy,
- generator output is modeled as `Iterator[T]`,
- existing generator behavior is revalidated through the new iterator protocol.

### wave_iter_4: Core Builtin Lazy Parity

Scope:

- convert `zip(...)`,
- convert `enumerate(...)`,
- convert `reversed(...)`,
- revalidate builtin iteration semantics against CPython source and tests.

Definition of done:

- these builtins return iterators rather than eager lists,
- public behavior matches the documented Python-shaped surface under Sifr-safe adaptation,
- explicit materialization via `list(...)` or similar is the canonical eager path.

### wave_iter_5: Initial `itertools` Lazy Subset

Scope:

- convert `chain`,
- convert `repeat`,
- convert `islice`,
- convert `count`,
- classify retained advanced iterator families explicitly.

Definition of done:

- the initial high-value lazy stdlib subset runs on the canonical iterator architecture,
- retained unsupported families are tracked with rationale rather than left open.

### wave_iter_6: Parity Closure, Demo, and Governance Hardening

Scope:

- add dedicated iterator-architecture demo coverage,
- complete CPython traceability records,
- harden docs and governance inventory,
- close with completion and production-grade review cycles.

Definition of done:

- the new iterator architecture is documented, demoed, and review-approved,
- no wave-owned parity or test-parity surface remains undocumented.

## CPython Test Porting Targets

This phase should start by porting and adapting the most informative iterator tests from `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_iter.py`:

- `test_iter_basic`
- `test_iter_idempotency`
- `test_iter_for_loop`
- `test_iter_independence`
- `test_nested_comprehensions_iter`
- `test_iter_class_for`
- `test_iter_class_iter`

Then port targeted behavior from:

- `Lib/test/test_builtin.py` for `next`
- `Lib/test/test_generators.py` for single-pass generator behavior and exhaustion
- `Lib/test/test_yield_from.py` only for cases compatible with Sifr's `Option`-based boundary and current generator scope
- builtin-specific test families for `zip`, `enumerate`, and `reversed`

Every reviewed upstream test or test family must end in exactly one state:

- `adopted`
- `adapted`
- `waived`

`waived` requires an explicit rationale tied to one of:

- `intentional-diff`
- `unsupported`
- `host-limited`
- `cpython-implementation-detail`

## Quality Contract

### Entry criteria

- The current mainline test baseline is green before execution starts.
- Existing ownership and non-panic invariants remain green.
- Entry-baseline evidence is recorded in the eventual execution issue before wave 1 begins.
- The phase starts with at least one confirmed current eager-mismatch example for generators or lazy builtins recorded in the execution tracker.
- Entry-baseline evidence must also record:
  - the chosen first implementation strategy for generator lowering,
  - one concrete type-system spike target for `Iterable[T]` / `Iterator[T]`,
  - an initial CPython test-family inventory with explicit adopt/adapt/waive tracking for the first wave,
  - one concrete borrow-safety example that the compiler must reject or sharply waive.

### Phase-wide invariants

- No user-triggerable panic paths are introduced.
- No implicit iterator-to-collection materialization remains in surfaces this phase claims to own.
- Collections remain reusable values rather than becoming iterator objects.
- Iterator consumption semantics remain explicit and deterministic.
- Unsupported iterator families fail through explicit documented boundaries rather than silent eager fallback or `Any` leakage.
- Generated Rust remains reviewable and deterministic.

### Wave quality checks

- No fallback, migration, or compatibility shim is allowed as the final architecture.
- No lazy partial fixes are allowed; each wave must close the root cause in its scope.
- Every wave must include at least one positive-path and one negative-path validation case.
- Every wave must include CPython test-parity accounting for its owned surfaces.
- Validation evidence must be recorded in the execution issue before merge.
- No wave is complete if its claimed iterator surface still degrades to eager list semantics without explicit rationale.

## Local Validation Commands

- Full local suite:
  - `scripts/run_all_tests.sh`
- Quick local suite for PRs:
  - `scripts/run_all_tests.sh --profile quick`
- Targeted compiler checks:
  - `cargo test -p sifr -- <test_name>`
  - `cargo test -p sifr_hir -- <test_name>`
  - `cargo test -p sifr_codegen -- <test_name>`
- Demo execution:
  - `cargo run -q -p sifr -- run demos/<iterator_demo>.sifr`

## Review Loop

For each wave:

1. define the wave todo list and the exact CPython source and test families in scope,
2. implement the wave,
3. validate locally with demo and tests,
4. open a PR,
5. run an external completion review focused on:
   - iterator parity gaps,
   - CPython behavior mismatches,
   - CPython test parity gaps,
   - Sifr-principle violations,
6. validate the review findings and fix what makes sense,
7. merge the review-fix PR,
8. run an external production-grade review with the same focus,
9. validate the review findings and fix what makes sense,
10. merge the production-grade review-fix PR,
11. update docs, traceability, and execution status before moving to the next wave.

At wave closure:

- run an additional completion review cycle,
- run an additional production-grade review cycle,
- send the required notifications,
- then move to the next wave.

At phase closure:

- run another completion review cycle,
- run another production-grade review cycle,
- only then mark the phase complete.

## Exit Gate

- Sifr has first-class iterable and iterator protocol support.
- Generator functions and owned lazy builtins return iterators rather than eager collections.
- Core iterator semantics are covered by CPython-derived tests with explicit adopt/adapt/waive accounting.
- Remaining advanced lazy gaps are explicitly classified rather than left open.
- The full validation suite is green.
- External review confirms the phase is production-grade for its documented scope.
