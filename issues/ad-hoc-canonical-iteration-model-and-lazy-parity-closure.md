# Ad Hoc Phase: Canonical Iteration Model and Lazy Parity Closure

Status: completed (started 2026-03-20; `wave_psp_iter_fix_0` through `wave_psp_iter_fix_8` implementation/review cycles merged; wave/milestone/phase closure review cycles approved; final phase production-grade pass remediated clippy findings in `sifr_hir` lowering paths; post-closure add-on ports related CPython `test_itertools` coverage for shipped `sifr.itertools` surfaces; post-closure add-on review pass 1 + pass 2 approved)
Context: corrective follow-up phase inserted after runtime/file-object parity expansion and before stateful RNG/crypto/polish parity expansion
Execution readiness: implementation-ready in sequence after `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`; this phase should execute before `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md` so later stream-style, binary, and stdlib work inherits one coherent iterable model
Execution ledger: `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure-execution.md`

## Objective

Close the remaining root-cause iterator and iterable debt so Sifr has one canonical iteration model from type system through HIR lowering, codegen, generators, builtins, and stdlib adapters.

This is not a cosmetic parity pass. The goal is to remove the current split where:

- `Iterable[T]` and `Iterator[T]` exist at the type level,
- iteration builtins lower through ad hoc builtin-call paths,
- codegen still assumes concrete containers in too many places,
- and lazy surfaces sometimes type-check but then eagerly materialize or fail once lowered.

Primary target area:

- compiler iteration semantics across type system, HIR, and codegen

Primary surface targets:

- `iter`
- `next`
- `reversed`
- `map`
- `filter`
- `zip`
- `enumerate`
- generator expressions
- generator functions
- collection materialization boundaries
- `sifr.itertools`

Secondary target area:

- user-defined iterable protocol participation
  - only after the canonical builtin and stdlib model is stable

## Source of Truth

- architecture baseline:
  - `internal_docs/architecture.md`
  - `internal_docs/phases/02_type_system_power.md`
  - `internal_docs/phases/07_stdlib_parity.md`
- parity governance and waiver inventory:
  - `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
- predecessor planning docs:
  - `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol.md`
  - `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`
  - `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
  - `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
  - `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
- successor planning doc:
  - `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`
- implementation hotspots:
  - `crates/sifr_type_system/src/types.rs`
  - `crates/sifr_hir/src/hir_nodes.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_hir/src/lower/statements.rs`
  - `crates/sifr_hir/src/lower/builtin_calls.rs`
  - `crates/sifr_hir/src/lower/function_flow.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/function_emitter.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
  - `crates/sifr_codegen/src/operator_protocol_emitters.rs`
  - `lib/sifr/itertools.sifr`
- CPython source and tests:
  - `/Users/yaseralnajjar/work/sifr/cpython`
  - `/Users/yaseralnajjar/work/sifr/cpython/Lib/test`

Primary upstream families:

- `Lib/test/test_iter.py`
- `Lib/test/test_filter.py`
- `Lib/test/test_enumerate.py`
- `Lib/test/test_zipfile.py`
  - only where runtime/file-object iterator-returning surfaces depend on correct iteration semantics
- `Lib/test/test_generators.py`
- `Lib/test/test_itertools.py`
- `Lib/test/test_tuple.py`
  - only for tuple iteration behavior relevant to the final contract

Initial CPython validation focus for this phase:

- `test_iter` coverage must explicitly account for:
  - basic `iter(...)` / `next(...)` behavior,
  - iterator-vs-iterable separation,
  - collection re-iteration vs single-pass iterator exhaustion
- `test_generators` coverage must explicitly account for:
  - simple lazy yield behavior,
  - filtered generator-expression parity where adapted,
  - intentionally unsupported generator shapes with explicit Sifr-safe diagnostics
- `test_itertools` coverage must explicitly account for:
  - lazy adapter composition,
  - materialization boundaries,
  - explicitly buffered helpers that remain documented intentional differences

## Why This Needs Its Own Phase

The earlier lazy-iterator phase established surface support for iterators, generators, and selected lazy helpers, but it did not fully close the implementation architecture.

The remaining defects share one root cause:

- `Iterator[T]` semantics still fracture once values reach codegen,
- builtin iterator operations are still lowered through ad hoc builtin-call shapes rather than dedicated iterator IR,
- erased boxed iterator lowering loses capability information such as reversibility and multi-pass behavior,
- and stdlib lazy helpers still mix generic iterable claims with collection-only or eagerly buffered implementations.

That makes this a compiler/runtime semantics phase, not a module-by-module parity cleanup.

## Depends on

- `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol.md`
  - this phase is corrective follow-up work, not a replacement for the original phase
- `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
  - bytes iteration and binary stream surfaces must inherit the final canonical iterator model
- `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
  - runtime/file APIs now expose iterator-returning surfaces that need a stable backend contract
- milestone-7 parity governance remains the baseline
- Phase 27 non-regression invariants remain mandatory
- Phase 29 local-first validation contract remains mandatory

## Language Contract

### Canonical iteration types

- `Iterable[T]`: a value that can produce iteration over `T`
- `Iterator[T]`: a stateful single-pass iterable whose `next()` yields `T | None`
- `Reversible[T]`: an iterable that supports reverse iteration
- `Iterator[T]` satisfies `Iterable[T]`
- not every iterable is reversible
- not every iterable is multi-pass

### Canonical collection iteration rules

- `list[T]` iterates `T`
- `set[T]` iterates `T`
- `dict[K, V]` iterates keys of type `K`
- `str` iterates one-character `str`
- `bytes` iterates `int`
- `range` iterates `int`
- tuples are iterable only when the compiler can prove one statically valid element type for the whole tuple
- homogeneous tuples therefore iterate their shared element type
- heterogeneous tuples do not gain implicit union-yield iteration in this phase; if no single common element type can be proven, tuple iteration is rejected explicitly

### Internal capability model

Compiler layers in this phase must reason about iterator capabilities explicitly rather than recovering them from erased backend types.

Illustrative target shape:

```rust
enum IteratorCapability {
    SinglePass,
    MultiPass,
    DoubleEnded,
    ExactSize,
}
```

The implementation does not need to expose this enum as a public language surface, but type-system, HIR, and codegen work in this phase must agree on an equivalent internal model.

### Canonical lazy vs eager rules

Lazy:

- `iter`
- `next`
- `reversed`
- `map`
- `filter`
- `zip`
- `enumerate`
- generator expressions
- generator functions
- lazy portions of `sifr.itertools`

Eager:

- `list(...)`
- `set(...)`
- `dict(...)`
- `tuple(...)`
- `sorted(...)`
- list/set/dict comprehensions
- any API whose documented semantics inherently require buffering or collection materialization

### Canonical safety rules

- `next(it)` returns `T | None`; `StopIteration` is not user-facing
- single-pass iterators must not be silently cloned to fake reusability
- invalid reuse of single-pass or uniquely borrowed iterators must be rejected statically when the ownership model can prove it; this phase does not treat `.clone()` as a semantic escape hatch for iterator correctness
- collection mutation while an incompatible live iterator exists remains statically controlled
- no silent eager fallback is allowed in lazy-returning APIs

## Scope

This phase owns:

- capability-aware iteration semantics in the type system
- canonical iterator HIR for builtin and lowering-level iteration operations
- concrete iterator-pipeline codegen for lazy chains
- generator backend unification with the canonical iterator model
- builtin lazy/eager boundary cleanup
- `sifr.itertools` rewrite around `Iterable[...]` where semantically valid
- user-defined iterable protocol participation after builtin/runtime semantics are stable

This phase does not own:

- async iteration
- unrelated parser or collection redesign
- broad new stdlib expansion unrelated to iteration correctness
- RNG or crypto feature expansion beyond what depends on iterator correctness

## Non-goals

- preserving incorrect eager behavior for compatibility
- keeping `Iterator[T]` semantically defined by erased Rust trait objects
- claiming lazy parity while buffering eagerly in hidden implementation paths
- introducing fallback shims that mask capability or ownership problems instead of fixing them
- broad user-defined dunder/protocol expansion unrelated to iteration conformance

## Priority Targets

### priority_1: Canonical compiler iteration semantics

Modules / crates:

- `sifr_type_system`
- `sifr_hir`
- `sifr_codegen`

Required closure direction:

- define capabilities explicitly,
- stop stringly builtin lowering for iterator operations,
- preserve concrete iterator semantics through codegen,
- and make one lowering path own collection iteration, iterator adapters, and collection materialization.

### priority_2: Builtin lazy/eager parity closure

Surfaces:

- `iter`
- `next`
- `reversed`
- `map`
- `filter`
- `zip`
- `enumerate`
- `sorted`
- collection constructors as explicit collectors

Required closure direction:

- builtins must agree across typing, lowering, and execution,
- `filter` must become truly lazy,
- and capability-gated operations such as `reversed` must be checked and lowered correctly.

### priority_3: Generator and stdlib iterator closure

Surfaces:

- generator functions
- generator expressions
- `sifr.itertools`
- iterator-returning runtime/file surfaces from earlier phases

Required closure direction:

- generator outputs must behave like first-class iterators,
- filtered and composed generator shapes must lower coherently,
- stdlib lazy helpers must accept general iterables where valid and document buffering where unavoidable.

### priority_4: User-defined iterable participation

Surfaces:

- user classes implementing iteration methods

Required closure direction:

- user-defined iterables must participate in the same type-checking, lowering, and codegen pipeline as builtin iterables,
- with precise diagnostics for protocol conformance failures.

## Waves

### wave_psp_iter_fix_0: Contract Freeze and Governance Lock

Scope:

- freeze canonical iteration semantics for `Iterable[T]`, `Iterator[T]`, `Reversible[T]`
- freeze lazy vs eager boundaries
- freeze tuple iteration contract
- classify permanent divergences and unsupported iterator families before implementation proceeds
- update architecture and waiver governance so later waves do not invent semantics ad hoc

Definition of done:

- the language contract in this document is reflected in architecture and parity governance,
- tuple behavior is explicitly classified,
- and all later waves have one semantic target.

### wave_psp_iter_fix_1: Type-System Capability Layer

Scope:

- keep `Iterable[T]` / `Iterator[T]` first-class
- add reversible capability support
- add internal iteration capability metadata for lowering and codegen
- align assignability and tuple iterability with the frozen contract
- make the tuple rule explicit: one statically provable element type or no tuple iteration support

Definition of done:

- reversibility is type-checked explicitly,
- type-level iteration semantics no longer depend on erased backend assumptions,
- and tuple iteration behavior is internally consistent.

### wave_psp_iter_fix_2: Canonical Iterator HIR

Scope:

- add dedicated iterator HIR for protocol entry, adapters, and explicit collection
- lower `for`, `iter`, `next`, `reversed`, `map`, `filter`, `zip`, `enumerate`, generator expressions, and comprehension sources through the same iterator IR family
- remove generic builtin-call lowering for iterator operations where canonical HIR exists

Definition of done:

- iterator semantics are represented structurally in HIR,
- `for` and builtin lazy operations share one lowering path,
- and HIR snapshots can directly assert iterator semantics.

### wave_psp_iter_fix_3: Concrete Iterator Codegen Pipelines

Scope:

- emit concrete Rust iterator chains for lazy pipelines
- centralize collection-to-iterator lowering
- remove clone-based fake re-iteration of true iterators
- preserve reversible / double-ended behavior where the contract allows it
- ensure materialization occurs only at explicit eager boundaries

Definition of done:

- iterator pipelines compile end-to-end without collection-only assumptions,
- generated Rust no longer calls `.iter()` or `.clone()` on iterator values unless semantically valid,
- `any(iter(xs))`, `filter(..., iter(xs))`, and `sorted(iter(xs))` are required closure targets,
- `reversed(iter(xs))` closes only for reversible / double-ended inputs and otherwise must fail through explicit capability-aware typing or diagnostics,
- and any residual non-closure must be a documented intentional divergence rather than an accidental backend limitation.

### wave_psp_iter_fix_4: Generator Backend Unification

Scope:

- replace the current narrow generator backend shape restrictions with a canonical iterator-producing backend
- align generator functions and generator expressions with the same iterator semantics
- support legal filtered and composed generator shapes through the canonical iteration path
- eliminate the current implementation restriction that only supports a single top-level `while` loop with exactly one direct yield site or one direct if-guarded yield site

Definition of done:

- generator outputs behave as first-class iterators,
- filtered generator expressions compile lazily,
- unsupported generator shapes fail with precise diagnostics instead of backend mismatch or codegen panic,
- and no supported generator form depends on the current ad hoc top-level-while-loop shape contract.

### wave_psp_iter_fix_5: Builtin Surface Cleanup

Scope:

- make `filter` return `Iterator[T]`
- ensure `map`, `zip`, `enumerate`, and `reversed` consume canonical iterable inputs and return lazy iterators
- ensure `any`, `all`, `sum`, `min`, `max`, `list`, `set`, `dict`, `tuple`, and `sorted` consume general iterables according to the final contract

Definition of done:

- builtin typing, lowering, and execution semantics match,
- eager boundaries are explicit and consistent,
- and builtin iterator consumers no longer assume concrete collection methods on iterator values.

### wave_psp_iter_fix_6: `sifr.itertools` and Iterator-Returning Stdlib Closure

Scope:

- rewrite `sifr.itertools` around `Iterable[...]` where semantically valid
- preserve laziness unless buffering is part of the documented algorithm
- align iterator-returning runtime/file helpers from earlier phases with the final canonical model

Definition of done:

- stdlib lazy helpers compose correctly with builtin iterators and collectors,
- list-only assumptions are removed where not semantically required,
- and buffered combinatoric helpers are documented explicitly instead of pretending to stream.

### wave_psp_iter_fix_7: User-Defined Iterable Protocol Participation

Scope:

- define user-facing iterable protocol participation for classes
- validate `__iter__`, `__next__`, and `__reversed__` conformance
- add user-defined iterable positive and negative coverage

Definition of done:

- user-defined iterable classes participate in the canonical pipeline,
- and protocol violations get precise diagnostics.

### wave_psp_iter_fix_8: Downstream Phase Alignment and Final Closure

Status:

- implementation/reviews merged with production-grade approval
- key wave-8 artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_8_downstream_alignment_closure.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_8_reversed_runtime_iterator_not_reversible.sifr`
  - `demos/ad_hoc_iter_fix_wave8_downstream_alignment_demo.sifr`
  - `verification/stdlib/wave_psp_iter_fix_8_cpython_traceability.md`

Scope:

- audit iterator-sensitive surfaces inherited from:
  - `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`
  - `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
  - `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
  - `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
- revalidate bytes iteration semantics against the final canonical iterator contract
- revalidate runtime/file iterator-returning APIs against the final canonical iterator contract
- close collection-only iterator assumptions that remain in earlier-phase stdlib surfaces
- update parity governance, waivers, and closure demos so inherited iterator behavior is classified here rather than retroactively changing earlier phase claims

Definition of done:

- no iterator-sensitive surface shipped by the earlier implemented phases contradicts the canonical iteration model,
- residual differences are documented as intentional divergences rather than accidental backend fallout,
- the milestone demo and final negative-case coverage reflect inherited runtime, bytes, and stdlib iterator composition,
- and the phase closes with no separate builtin-only or phase-local iterator semantics left behind.

## Required Validation

### Compiler validation

- type-system tests for capability assignability and tuple iteration consistency
- HIR lowering snapshots for canonical iterator forms
- codegen tests for concrete iterator chains and explicit materialization boundaries
- generator lowering tests for legal and intentionally unsupported shapes
- diagnostics tests for invalid reversibility, reuse, and protocol conformance

### End-to-end positive coverage

- `for` over list, set, dict, str, bytes, range, tuple, and user-defined iterable values
- `iter` and `next`
- `map`, `filter`, `zip`, `enumerate`, `reversed`
- `sorted` over collections and iterator values
- generator expressions
- generator functions
- `list`, `set`, `dict`, and `tuple` collecting from iterables
- runtime/file iterator-returning APIs from earlier phases composing with builtins
- `sifr.itertools` lazy subset composition

### End-to-end negative coverage

- `reversed` on non-reversible single-pass iterators
- assigning lazy iterator results directly to concrete collection-typed values without explicit materialization
- invalid iterator reuse where ownership semantics disallow it
- unsupported heterogeneous tuple iteration if still out of contract
- malformed user-defined iterable protocol implementations
- unsupported generator backend shapes with precise diagnostics

## Demo Requirement

Add a closure demo under `demos/` that shows:

- builtin collection iteration,
- lazy builtin adapter chains,
- explicit collection materialization,
- generator expressions,
- generator functions,
- `sifr.itertools` composition,
- runtime/file iterator composition inherited from earlier phases,
- user-defined iterable participation,
- and at least one negative-case safety assertion.

## Exit Criteria

This phase is complete only when all of the following are true:

1. Sifr has one canonical iteration semantics path from type system through codegen.
2. builtin iterator operations are consistent across typing, lowering, and execution.
3. `Iterator[T]` no longer breaks correctness because erased default lowering loses required capabilities.
4. `filter` is truly lazy and composes correctly.
5. `reversed` is capability-correct.
6. tuple iteration is internally consistent with the final language contract.
7. generator expressions and generator functions behave as first-class iterator producers.
8. `sifr.itertools` and iterator-returning stdlib APIs interoperate with builtin iterator consumers and collectors.
9. user-defined iterable protocol participation works through the same canonical path.
10. all targeted validation lanes pass locally.

## Follow-On Placement

Once this phase is complete, `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md` can execute on top of a stable iterable model rather than carrying iterator capability debt into future stream-style or bytes-native APIs.
