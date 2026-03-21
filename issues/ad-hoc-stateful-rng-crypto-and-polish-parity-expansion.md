# Ad Hoc Phase: Stateful RNG, Crypto, and Polish Parity Expansion

Status: closed (documented 2026-03-18; sequencing revised 2026-03-20; wave `wave_psp_rng_0` architecture lock completed 2026-03-21; wave `wave_psp_rng_1` merged via PR #1376 on 2026-03-21; wave `wave_psp_rng_2` closed via PRs #1379/#1380 with production-grade review pass-2 approval on 2026-03-21; wave `wave_psp_rng_3` merged via PRs #1382/#1383 with production-grade review pass-2 approval on 2026-03-21; milestone closure review passes 1/2 approved on 2026-03-21; phase closure review passes 1/2 approved and production-grade closed on 2026-03-21; post-closure CPython adaptation pass completed on 2026-03-21; post-closure external review remediation pass-1 completed on 2026-03-21; post-closure external review remediation pass-2 completed on 2026-03-21)
Context: final cleanup phase after the structured/class, extended bytes-foundation, runtime/file-object, and canonical iteration-model follow-ups
Execution readiness: implementation-ready in sequence after `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`; predecessor bytes-phase extension waves `wave_psp_bytes_4` and `wave_psp_bytes_5` are completed, so crypto and RNG surfaces inherit the final raw-byte-backed `bytes` contract, stable iterator semantics, and successor governance baseline
Execution ledger: `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-execution.md`

## Objective

Close the remaining high-value parity debt for stateful randomness, advanced crypto surfaces, and a small set of already-strong modules that need targeted polish rather than broad redesign.

Primary module targets:

- `random`
- `hashlib`

Secondary targeted polish modules:

- `base64`
- `statistics`

Conditional residual polish targets:

- `textwrap`
- `html`
  - only if earlier phases still leave explicit residual waivers for them

## Source of Truth

- canonical parity inventory:
  - `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
- relevant wave ledgers:
  - `verification/stdlib/wave_psp_b2_cpython_traceability.md`
  - `verification/stdlib/wave_psp_rng_1_cpython_traceability.md`
  - `verification/stdlib/wave_psp_rng_2_cpython_traceability.md`
  - `verification/stdlib/wave_psp_rng_3_cpython_traceability.md`
  - `verification/stdlib/wave_psp_c2_cpython_traceability.md`
  - `verification/stdlib/wave_psp_e1_cpython_traceability.md`
- predecessor planning docs:
  - `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
  - `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
  - `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
  - `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`
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
- algorithm inventory expansion and binary crypto boundaries,
- and final polish on modules that are already near closure but still carry a narrow advanced-feature waiver set.

It should therefore execute after the broader object-model and runtime work, not before it.

The phase design is fixed in this document. Waves 0/1/2/3 are implementation-complete, wave-level/milestone-level/phase-level review loops are closed, and the phase is production-grade closed.

## Depends on

- `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
- `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`
- `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`
- milestone-7 canonical closure inventory remains the baseline
- prior follow-up phases should already have reduced broader parser, class, and runtime debt
- Phase 27 non-regression invariants remain mandatory
- Phase 29 local-first validation contract remains mandatory

## Public Surface Contract

### `random`

- Use a deterministic explicit PRNG object model instead of the current stateless intrinsic-backed model.
- Add:
  - `RandomState`
  - `Random`
  - `SystemRandom`
- `Random` uses MT19937-compatible state semantics.
- `RandomState` is a typed value object rather than a raw Python tuple.
- `RandomState` fields:
  - `version: int`
  - `state_words: list[int]`
  - `index: int`
  - `gauss_next: float | None`
- Module-level `seed`, `getstate`, `setstate`, `randrange`, `randint`, `random`, `choice`, `choices`, `sample`, `shuffle`, `gauss`, `uniform`, and `randbytes` delegate to one module-global `Random` instance.
- `SystemRandom` remains non-deterministic and does not support `getstate` or `setstate`.
- `randbytes(n: int) -> Result[bytes, ValueError]` is in scope once the deterministic object model is stable.
- `randbytes` must return canonical raw-byte-backed `bytes` directly; it must not materialize widened integer storage internally.
- `choices(weights=...)` remains out of scope and explicitly unsupported in this phase unless a later wave explicitly widens scope.

### `hashlib`

- Keep `str` input support.
- Use first-class `bytes` as the canonical binary carrier.
- `HashObject` gains:
  - `update(data: str) -> None`
  - `update_bytes(data: bytes) -> None`
  - `digest() -> bytes`
  - `digest_bytes() -> bytes`
  - `hexdigest() -> str`
- `digest_bytes()` is an explicit alias for `digest()` for API clarity.
- Add:
  - `new_bytes(name: str, data: bytes = bytes()) -> Result[HashObject, ValueError]`
- Typed bytes-native crypto paths must operate on canonical raw-byte-backed `bytes` without per-element range validation or `i64` widening/narrowing on already-typed `bytes` values.
- Add SHA3 / SHAKE only for algorithms already supported by the Rust dependency stack when implementation begins.
- SHAKE APIs require explicit output length parameters and return `bytes`.

### `base64`

- Any remaining advanced codec work uses first-class `bytes` as the binary carrier.
- Existing text-friendly helpers may remain, but parity claims for binary-oriented surfaces attach to the bytes-based APIs.

### `statistics`

- Only close narrow advanced surfaces that do not require decimal, fraction, or context-sensitive semantics.
- `NormalDist` did not meet the bounded wave scope and remains explicitly unsupported for this phase (tracked with a negative fixture and waiver entry).

### `textwrap` / `html`

- Keep them in this phase only if phase 3 still leaves explicit residual waivers.
- Otherwise remove them from the execution checklist for this phase and leave them closed in the earlier phase docs.

## Permanent Sifr-Safe Diffs

The following are intentionally not part of this phase’s execution target:

- full CPython buffer protocol parity,
- `memoryview`,
- `bytearray`-driven binary mutability ecosystems,
- non-deterministic state export for `SystemRandom`,
- decimal / fraction / context-aware `statistics` semantics.

If these remain unsupported at phase exit, they must be explicit and narrow in the waiver inventory.

## Scope

This phase owns:

- `random` stateful generator parity such as `seed`, `getstate`, `setstate`, and the typed `Random` / `SystemRandom` object model,
- `hashlib` advanced algorithm and bytes-native digest parity on top of the predecessor `bytes` phase,
- `base64` residual binary-surface polish using first-class `bytes`,
- `statistics` narrow advanced-surface cleanup,
- residual `textwrap` / `html` polish only if earlier phases leave explicit waivers open.

This phase does not own:

- broad parser/class expansion,
- stream/file-object lifecycle work,
- reflection-heavy callable wrappers,
- async runtime or host platform expansion.

## Non-goals

- pretending deterministic RNG object parity exists without the typed state model defined above,
- claiming bytes-native crypto parity while still routing through helper-only stand-ins,
- reopening already-closed low-value polish areas unless they shrink a specific waiver entry.

## Priority Targets

### priority_1: Stateful RNG parity

Modules:

- `random`

Required closure direction:

- land the deterministic `RandomState` model,
- define the module-global proxy behavior,
- add `randbytes` on the approved bytes-backed object model,
- preserve panic-free typed domain behavior for all invalid-state and invalid-argument paths.

### priority_2: Advanced crypto parity

Modules:

- `hashlib`
- `base64`

Required closure direction:

- ship the next algorithm tranche only when the runtime support is real,
- close binary digest/object gaps using first-class `bytes`,
- keep unsupported families explicitly classified if buffer/view parity is still absent.

### priority_3: Near-closure polish modules

Modules:

- `statistics`
- `textwrap`
- `html`

Required closure direction:

- close any remaining narrow advanced-feature waivers that are low-risk and high-signal,
- avoid turning this phase into a broad text or parser redesign.

## Waves

### wave_psp_rng_0: Architecture Lock

Scope:

- `RandomState`
- module-global RNG proxy behavior
- bytes-native crypto boundary

Definition of done:

- the typed RNG state model and bytes-native crypto rules in this document are reflected in traceability and waivers,
- no later wave needs to invent state serialization or binary-carrier semantics,
- permanently deferred families are explicitly classified before implementation proceeds.

### wave_psp_rng_1: Deterministic RNG State and Object Model

Scope:

- `random`

Definition of done:

- stateful RNG parity is materially stronger,
- `seed` / `getstate` / `setstate` and the approved generator object model are shipped or sharply waived,
- local coverage proves deterministic behavior and typed failure boundaries.

### wave_psp_rng_2: Advanced Hash and Binary Surface Expansion

Scope:

- `hashlib`
- `base64`

Definition of done:

- the next algorithm and binary-surface tranche is closed or explicitly re-waived with concrete runtime blockers,
- bytes-native behavior is not overclaimed,
- traceability and negative coverage match the shipped surface.

### wave_psp_rng_3: Final Polish Waiver Reduction

Scope:

- `statistics`
- residual `textwrap`
- residual `html`

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
- Crypto and digest surfaces must not introduce panic-prone or fake binary compatibility.
- Every wave must update the waiver ledger before merge.
- No module may be called “finished” in this phase unless its surviving waiver set is explicit and small.
- Non-deterministic host-backed APIs such as `SystemRandom` are allowed only when their boundaries are explicit, non-panicking, and excluded from deterministic state-serialization claims.

## Architecture Lock Validation

Before `wave_psp_rng_1` began implementation, the phase added:

- one implementation note mapping the approved `RandomState` fields to the chosen MT19937-compatible internal state model,
- one implementation note defining the exact host-boundary contract for `SystemRandom`,
- one Sifr demo covering the typed `RandomState` and module-global RNG proxy model,
- one Sifr demo covering the bytes-native digest model,
- one implementation note proving bytes-native RNG/crypto paths consume raw-byte-backed `bytes` directly rather than compensating around widened integer storage,
- one negative-path test for every newly explicit permanent divergence,
- one CPython-family mapping table proving which upstream cases are adopted, adapted, or permanently waived,
- explicit phase test families covering `test_random`, `test_hashlib`, `test_base64`, and `test_statistics`,
- one compile-time rejection or negative runtime case for every new typed surface that proves the remaining Sifr-safe divergence is explicit rather than accidental,
- one dependency audit note recording which SHA3 / SHAKE families are actually available in the Rust dependency stack at execution start.

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
- `hashlib` advanced algorithm and binary digest waivers are materially reduced or sharply reclassified,
- targeted polish modules no longer carry vague advanced-feature debt,
- the full validation suite is green,
- external review confirms production-grade closure for the documented scope.
