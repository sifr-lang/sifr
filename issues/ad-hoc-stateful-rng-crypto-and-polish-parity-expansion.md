# Ad Hoc Phase: Stateful RNG, Crypto, and Polish Parity Expansion

Status: open (documented 2026-03-17)
Context: final cleanup phase after the structured/class and runtime/file-object follow-ups
Execution readiness: planning-ready after the preceding follow-up phases close

## Objective

Close the remaining high-value parity debt for stateful randomness, advanced crypto surfaces, and a small set of already-strong modules that need targeted polish rather than broad redesign.

Primary module targets:

- `random`
- `hashlib`

Secondary targeted polish modules:

- `base64`
- `statistics`
- `textwrap`
- `html`

## Source of Truth

- canonical parity inventory:
  - `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
- relevant wave ledgers:
  - `verification/stdlib/wave_psp_b2_cpython_traceability.md`
  - `verification/stdlib/wave_psp_c2_cpython_traceability.md`
  - `verification/stdlib/wave_psp_e1_cpython_traceability.md`
- predecessor planning docs:
  - `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
  - `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
- architecture baseline:
  - `internal_docs/architecture.md`
- CPython source and tests:
  - `/Users/yaseralnajjar/work/sifr/cpython`
  - `/Users/yaseralnajjar/work/sifr/cpython/Lib/test`

Primary upstream families:

- `Lib/test/test_random.py`
- `Lib/test/test_hashlib.py`
- `Lib/test/test_base64.py`
- `Lib/test/test_statistics.py`
- `Lib/test/test_textwrap.py`
- `Lib/test/test_html.py`

## Why This Needs Its Own Phase

This work shares a different root cause from the earlier phases:

- deterministic mutable RNG state,
- algorithm inventory expansion and bytes-oriented crypto boundaries,
- and final polish on modules that are already near closure but still carry a narrow advanced-feature waiver set.

It should therefore execute after the broader object-model and runtime work, not before it.

## Depends on

- milestone-7 canonical closure inventory remains the baseline
- prior follow-up phases should already have reduced broader parser, class, and runtime debt
- Phase 27 non-regression invariants remain mandatory
- Phase 29 local-first validation contract remains mandatory

## Scope

This phase owns:

- `random` stateful generator parity such as `seed`, `getstate`, `setstate`, and selected `Random` / `SystemRandom` behavior,
- `hashlib` advanced algorithm and bytes-oriented digest parity where the runtime can support it honestly,
- `base64` only for any residual non-core codec or bytes-boundary polish that remains after earlier phases,
- `statistics` only for narrow advanced numeric-surface cleanup,
- `textwrap` and `html` only for any residual top-level polish waivers not already closed by the structured/class phase.

This phase does not own:

- first-class bytes object-model design from zero,
- broad parser/class expansion,
- stream/file-object lifecycle work,
- reflection-heavy callable wrappers,
- async runtime or host platform expansion.

## Non-goals

- pretending deterministic RNG object parity exists without a documented state model,
- claiming bytes-native crypto parity while still routing through string-only stand-ins,
- reopening already-closed low-value polish areas unless they shrink a specific waiver entry.

## Priority Targets

### priority_1: Stateful RNG parity

Modules:

- `random`

Required closure direction:

- add deterministic state APIs,
- define the supported `Random` / `SystemRandom` object model,
- preserve panic-free typed domain behavior for all invalid-state and invalid-argument paths.

### priority_2: Advanced crypto parity

Modules:

- `hashlib`

Required closure direction:

- ship the next algorithm tranche such as SHA3/SHAKE only when the runtime support is real,
- close bytes-oriented digest/object gaps only where the bytes boundary is explicit and safe,
- keep unsupported families explicitly classified if first-class bytes parity is still absent.

### priority_3: Near-closure polish modules

Modules:

- `base64`
- `statistics`
- `textwrap`
- `html`

Required closure direction:

- close any remaining narrow advanced-feature waivers that are low-risk and high-signal,
- avoid turning this phase into a broad text or parser redesign.

## Waves

### wave_psp_rng_1: Deterministic RNG State and Object Model

Scope:

- `random`

Definition of done:

- stateful RNG parity is materially stronger,
- `seed` / `getstate` / `setstate` and the approved generator object model are shipped or sharply waived,
- local coverage proves deterministic behavior and typed failure boundaries.

### wave_psp_rng_2: Advanced Hash Surface Expansion

Scope:

- `hashlib`

Definition of done:

- the next algorithm/object tranche is closed or explicitly re-waived with concrete runtime blockers,
- bytes-oriented behavior is not overclaimed,
- traceability and negative coverage match the shipped surface.

### wave_psp_rng_3: Final Polish Waiver Reduction

Scope:

- `base64`
- `statistics`
- `textwrap`
- `html`

Definition of done:

- the remaining low-risk polish waivers are materially reduced,
- no owned module remains in a vague “already strong but partial” state,
- surviving waivers are narrow and explicit.

## CPython Test Porting Targets

- `Lib/test/test_random.py`
- `Lib/test/test_hashlib.py`
- `Lib/test/test_base64.py`
- `Lib/test/test_statistics.py`
- `Lib/test/test_textwrap.py`
- `Lib/test/test_html.py`

## Quality Contract

- Determinism claims must be real and testable.
- Crypto and digest surfaces must not introduce panic-prone or fake bytes compatibility.
- Every wave must update the waiver ledger before merge.
- No module may be called “finished” in this phase unless its surviving waiver set is explicit and small.

## Local Validation Commands

- quick gate:
  - `scripts/run_all_tests.sh --profile quick`
- full gate:
  - `scripts/run_all_tests.sh`
- targeted:
  - `cargo test -p sifr -- <test_name>`
  - `cargo run -q -p sifr -- run demos/<rng-crypto-demo>.sifr`

## Exit Gate

This phase is complete only when:

- the `random` stateful-object waiver family is materially reduced,
- `hashlib` advanced algorithm and digest waivers are materially reduced or sharply reclassified,
- targeted polish modules no longer carry vague advanced-feature debt,
- the full validation suite is green,
- external review confirms production-grade closure for the documented scope.
