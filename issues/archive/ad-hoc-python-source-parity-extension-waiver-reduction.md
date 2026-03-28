# Ad Hoc Phase: Python Source Parity Extension and Waiver Reduction

Status: open (documented 2026-03-17)
Context: continuation phase after `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol.md`
Execution readiness: implementation-ready after the predecessor phase closes its protocol, builtin-lazy, and initial-`itertools` waves

## Objective

Use the first-class iterable/iterator architecture from the lazy-iterator phase to retire parity waivers and eager adaptations that were previously accepted only because Sifr did not yet have a correct iterator model.

This is not a fresh broad parity sweep. It is a focused continuation phase with three concrete goals:

1. replace eager list-backed compatibility adaptations with true iterator-returning behavior where CPython defines iterator surfaces,
2. close previously waived iterator-returning APIs in already shipped modules,
3. update the parity-governance ledgers so the repo no longer claims closure through stale eager semantics.

The closed phase in `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md` correctly deferred these items at the time. This follow-up exists because the deferment should now be consumed, not left as permanent waiver debt.

## Source of Truth

This phase must use the following as authoritative references:

- predecessor architecture phase:
  - `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol.md`
- closed baseline parity phase:
  - `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`
  - `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface-execution.md`
- canonical parity governance and waiver inventories:
  - `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
  - `verification/stdlib/wave_psp_a1_cpython_traceability.md`
  - `verification/stdlib/wave_psp_b2_cpython_traceability.md`
  - `verification/stdlib/wave_psp_d1_cpython_traceability.md`
  - `verification/stdlib/wave_psp_e1_cpython_traceability.md`
- parity audits and baseline matrices:
  - `issues/stdlib_gaps_cpython_module_by_module_audit_2026-03-14.md`
  - `verification/stdlib/phase30_parity_matrix.md`
- current architecture baseline:
  - `internal_docs/architecture.md`
  - `internal_docs/phases/07_stdlib_parity.md`
  - `internal_docs/phases/30_reliability_parity_and_performance_budgets.md`
  - `internal_docs/phases/31_algorithmic_compatibility_and_leetcode_coverage.md`
- CPython source and tests:
  - `/Users/yaseralnajjar/work/sifr/cpython`
  - `/Users/yaseralnajjar/work/sifr/cpython/Lib/test`

Primary upstream CPython families for this continuation are:

- `Lib/test/test_iter.py`
- `Lib/test/test_builtin.py`
- `Lib/test/test_itertools.py`
- `Lib/test/test_re.py`
- `Lib/test/test_glob.py`
- `Lib/test/test_pathlib/`

The predecessor phase already identified the core implementation references for `iter`, `next`, `zip`, `enumerate`, `reversed`, collection iterators, and generator iterators. This phase must consume those architectural decisions rather than reopening them.

## Why This Needs Its Own Follow-up Phase

The closed parity phase intentionally accepted several lazy-surface compromises:

- `reversed(...)` was marked parity-closed while still materializing a list,
- `zip(...)`, `map(...)`, and `enumerate(...)` closed call-shape parity before true iterator-return parity existed,
- `itertools` closed a useful eager subset while the canonical lazy object model was explicitly waived,
- `glob` left `iglob` out of scope,
- `re` left `finditer` out of scope,
- and iterator-returning filesystem helpers were accepted through adapted eager surfaces instead of true iterator contracts.

That was correct then, because the runtime did not yet have first-class iterator values, `iter(...)` / `next(...)`, or generator-based lazy lowering.

After the predecessor phase lands, keeping those waivers in place becomes technical debt instead of prudent scoping. This continuation phase is the point where that debt must be retired.

## Depends on

- `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol.md`
  - hard dependency on its protocol, builtin-lazy, generator, and initial-`itertools` waves
- `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`
  - remains the baseline inventory this phase is extending and correcting
- Phase 27 non-regression invariants remain mandatory
- Phase 29 local-first validation contract remains mandatory

## Recommended Placement

- Execute immediately after the lazy-iterator/protocol phase reaches wave closure for:
  - builtin iterator return-shape parity,
  - generator iterator lowering,
  - initial lazy `itertools` subset
- Treat this phase as the parity-consumption layer for that architecture.
- Do not start unrelated parity-expansion work that depends on lazy iterator return shapes before this phase closes.

## Scope

This phase owns:

- re-closing builtin iterator-returning surfaces so they use true iterators instead of eager materialization,
- replacing the broad `itertools` lazy-model waiver with concrete shipped iterator behavior and a smaller explicit residual waiver set,
- adding missing iterator-returning APIs in already shipped modules where the missing architecture was the primary blocker,
- re-auditing already shipped iterator-returning module APIs for return-shape parity,
- updating the canonical traceability and waiver ledgers to reflect post-iterator reality.

This phase does not own:

- async iterators,
- reflection-by-string factories such as `operator.attrgetter` and `operator.methodcaller`,
- callable-wrapper families such as `functools.partial`, `cmp_to_key`, cache decorators, or `wraps`,
- deterministic mutable RNG object families such as `random.Random`, `seed`, `getstate`, and `setstate`,
- bytes-native parity expansions in `hashlib`, `base64`, or `secrets`,
- unrelated class-hierarchy expansion in `io`, `logging`, `subprocess`, `zipfile`, or `configparser`,
- dynamic callback-hook parity for `json` and `tomllib`.

## Non-goals

- preserving eager list-backed behavior for APIs that should now return iterators,
- keeping stale "parity-closed" labels when the closure still depended on eager adaptation,
- reopening waivers that are blocked by other root causes unrelated to iterable/iterator architecture,
- introducing fallback shims or duplicate compatibility entry points instead of fixing the real return-shape model,
- copying CPython exception control flow instead of using Sifr-safe typed boundaries.

## Waiver-Retirement Model

This phase should be driven by waiver retirement, not by module collection.

### 1. Retire stale eager builtin adaptations

The closed parity inventory currently records builtin closure while still carrying eager behavior:

- `reversed(...)`
- `enumerate(...)`
- `zip(...)`
- `map(...)`

After the predecessor phase, these surfaces should no longer be considered "closed enough" through eager list materialization. They must be re-closed on the canonical iterator runtime and their CPython traceability notes must be updated accordingly.

### 2. Consume the `itertools` lazy waiver

The current waiver ledger explicitly says:

- iterator-object / broad lazy parity families remain `unsupported`
- revisit when the lazy-iterator runtime architecture exists

That revisit trigger is exactly what the predecessor phase provides. This continuation must therefore replace the broad waiver with:

- real shipped lazy behavior for the approved `itertools` surface,
- narrower residual waivers only for families still blocked by separate typing or object-model constraints.

### 3. Close missing iterator-returning module APIs

The audit and traceability docs already identify iterator-returning gaps that were left open because the repo lacked a correct lazy model. The main continuation targets are:

- `re.finditer(...)`
- `Pattern.finditer(...)`
- `glob.iglob(...)`
- `Path.iterdir()`
- `Path.glob(...)`
- `Path.rglob()`

If any of these currently return eager collections or helper-specific adapted shapes, this phase must either:

- convert them to real iterator-returning behavior,
- or record an explicit remaining waiver with a concrete non-iterator blocker.

### 4. Keep non-iterator waivers explicit

This continuation should not pretend that the lazy-iterator architecture unlocks everything.

The following waiver families stay out of scope unless a new root cause is explicitly closed:

- `functools.partial` / `cmp_to_key`
- `operator.attrgetter` / `methodcaller`
- stateful RNG object parity
- bytes/object-model parity families
- dynamic callback injection surfaces
- host-limited runtime/platform families

## Priority Closure Targets

The order below is mandatory. Builtin and `itertools` re-closure comes first because downstream module APIs should reuse the same canonical iterator behavior.

### priority_1: Builtin iterator-return shape re-closure

Targets:

- `reversed(...)`
- `enumerate(...)`
- `zip(...)`
- `map(...)`

Required outcome:

- no builtin in this set returns eager materialized collections as its claimed parity behavior,
- explicit materialization happens only when the user writes `list(...)`, `tuple(...)`, `set(...)`, or `dict(...)`,
- any currently recorded `strict=` waivers for `zip` / `map` remain explicit unless closed separately, and this phase must revalidate that both waiver entries still correspond to real upstream-supported surfaces before phase exit.

### priority_2: `itertools` waiver replacement

Targets:

- predecessor-carried lazy subset:
  - `chain`
  - `repeat`
  - `islice`
  - `count`
- already shipped high-value eager helpers that should now migrate to real iterators:
  - `accumulate`
  - `compress`
  - `dropwhile`
  - `takewhile`
  - `filterfalse`
  - `zip_longest`
  - `cycle`
  - `starmap`
  - `product`
  - `permutations`
  - `combinations`
  - `combinations_with_replacement`

Residual waiver candidates after this continuation may still include:

- `tee`
- `groupby`
- any family that still requires separate object-lifetime or callable-typing work

Those residual waivers must be narrow and specific. The broad "lazy iterator object families" waiver should not survive phase exit.

### priority_3: Iterator-returning module API expansion

Targets:

- `re.finditer(...)`
- `Pattern.finditer(...)`
- `glob.iglob(...)`
- `Path.iterdir()`
- `Path.glob(...)`
- `Path.rglob()`

Required outcome:

- these APIs are either shipped as real iterator-returning surfaces,
- or explicitly waived with a blocker that is not merely "Sifr had no iterator architecture".

### priority_4: Governance and public-claim correction

Targets:

- `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
- relevant `wave_psp_*_cpython_traceability.md` ledgers
- successor parity inventories created for this continuation
- architecture and public parity wording where eager behavior was previously described as closure

Required outcome:

- no stale ledger entry continues to imply full closure through eager adaptation when the new iterator runtime exists,
- every surviving non-parity entry includes an owner, rationale, revisit rule, and precise blocker.

## Waves

### wave_psp_ext_1: Builtin Iterator Re-Closure

Scope:

- port predecessor builtin-iterator architecture into the legacy parity ledgers,
- convert `reversed`, `enumerate`, `zip`, and `map` to true iterator-returning semantics where they are still eager,
- revalidate `list(...)`, `tuple(...)`, `set(...)`, and `dict(...)` as the canonical materialization boundary.

Definition of done:

- the builtin iterator helpers above no longer rely on eager collection return shapes,
- CPython-derived tests and demos cover iterator exhaustion and explicit materialization,
- the old eager-adaptation wording is removed from the canonical ledgers.

### wave_psp_ext_2: `itertools` Lazy Surface Closure

Scope:

- replace the broad `itertools` lazy waiver with real shipped iterator behavior,
- migrate previously eager `itertools` helpers onto the canonical iterator runtime,
- tighten residual waivers to only the families still blocked by non-iterator root causes.

Definition of done:

- the shipped `itertools` subset now behaves as actual iterator-returning APIs,
- no traceability note still describes the approved `itertools` surface as fundamentally eager,
- any remaining unsupported families are narrowly justified.

### wave_psp_ext_3: Regex and Filesystem Iterator Surfaces

Scope:

- add `re.finditer(...)`,
- add `Pattern.finditer(...)`,
- add `glob.iglob(...)`,
- re-audit `Path.iterdir`, `Path.glob`, and `Path.rglob` for iterator return-shape parity.

Definition of done:

- the high-value iterator-returning stdlib APIs above are shipped or explicitly re-waived with a non-iterator blocker,
- positive and negative coverage exists for exhaustion, explicit materialization, and invalid input handling,
- the wave does not regress existing deterministic filesystem and regex safety behavior.

### wave_psp_ext_4: Waiver Ledger Reduction and Exit Closure

Scope:

- publish the post-iterator successor governance inventory,
- update all affected wave ledgers,
- shrink or replace the old lazy waivers,
- align architecture/public wording with actual post-phase behavior.

Definition of done:

- no broad lazy-iterator waiver remains where the predecessor architecture already removed the root cause,
- no affected surface remains in `open`,
- the repo has one clear post-phase account of what is now parity-closed, what remains intentionally different, and why.

## CPython Test Porting Targets

This continuation should harvest and map at least the following upstream families:

- `Lib/test/test_iter.py`
  - iterator identity, exhaustion, independence, and nested-loop behavior
- `Lib/test/test_builtin.py`
  - `enumerate`, `zip`, `map`, `reversed`, `iter`, and `next`
- `Lib/test/test_itertools.py`
  - shipped combinator families plus explicit waiver accounting for any retained exclusions
- `Lib/test/test_re.py`
  - `finditer` behavior and exhaustion semantics
- `Lib/test/test_glob.py`
  - `iglob` behavior and pattern/empty-root boundaries
- `Lib/test/test_pathlib/`
  - `iterdir`, `glob`, and `rglob` iterator-returning behavior

Every reviewed upstream test or family must end in exactly one state:

- `adopted`
- `adapted`
- `waived`

`waived` requires explicit rationale tied to:

- `intentional-diff`
- `unsupported`
- `host-limited`
- `cpython-implementation-detail`

## Quality Contract

### Entry criteria

- the predecessor lazy-iterator phase has merged the protocol and builtin-lazy waves this continuation depends on,
- the mainline test baseline is green before implementation starts,
- current eager/lazy mismatch evidence is recorded for every target wave before code changes begin.

### Phase-wide invariants

- no user-triggerable panic paths are introduced,
- no claimed iterator-returning API silently materializes a collection without an explicit source-level materialization call,
- collections remain reusable values and iterators remain single-pass stateful values,
- compile-time ownership and exclusivity guarantees remain in force for iterator-backed mutation scenarios,
- remaining unsupported iterator families fail through explicit, documented boundaries.

### Wave quality checks

- every wave must include at least one CPython-derived positive-path and one negative-path validation case,
- every wave must update the relevant traceability ledger before merge,
- no wave may leave a claimed surface in a partially eager / partially lazy undocumented state,
- no wave is complete if it merely changes implementation without shrinking or correcting the waiver/governance debt.

## Local Validation Commands

- quick gate:
  - `scripts/run_all_tests.sh --profile quick`
- full gate:
  - `scripts/run_all_tests.sh`
- targeted compiler checks:
  - `cargo test -p sifr -- <test_name>`
  - `cargo test -p sifr_hir -- <test_name>`
  - `cargo test -p sifr_codegen -- <test_name>`
- targeted demos:
  - `cargo run -q -p sifr -- run demos/<iterator-parity-demo>.sifr`

## Review Loop

For each wave:

1. define the exact waived/adapted surfaces being retired,
2. enumerate the relevant CPython source and test families,
3. implement the wave,
4. validate locally,
5. open a PR,
6. run completion review,
7. validate and fix actionable findings,
8. merge,
9. run production-grade review,
10. validate and fix actionable findings,
11. merge,
12. update parity ledgers and phase status before moving on.

At wave closure:

- run one additional completion review,
- run one additional production-grade review,
- update the successor waiver inventory,
- then move to the next wave.

At phase closure:

- run the same two review loops again,
- ensure the final governance inventory is published,
- ensure all affected stale eager-parity claims are corrected,
- only then mark the phase complete.

## Exit Gate

This phase is complete only when all of the following are true:

- builtin iterator-returning surfaces no longer depend on eager compatibility behavior,
- the broad `itertools` lazy waiver from the earlier parity inventory has been retired or reduced to a narrow residual set,
- `re.finditer`, `Pattern.finditer`, `glob.iglob`, and iterator-returning `pathlib` helpers are either shipped or explicitly re-waived with non-iterator blockers,
- the canonical governance inventory no longer overstates closure through stale eager semantics,
- the full validation suite is green,
- external review confirms production-grade closure for the documented scope.
