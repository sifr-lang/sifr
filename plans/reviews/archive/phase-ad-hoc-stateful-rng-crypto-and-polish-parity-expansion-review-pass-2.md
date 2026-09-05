# Phase Review: Ad Hoc Stateful RNG, Crypto, and Polish Parity Expansion

## Review Metadata

- **Phase**: `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`
- **Wave Under Review**: `wave_psp_rng_1` (Deterministic RNG State and Object Model)
- **Review Pass**: 2
- **Date**: 2026-03-21
- **Reviewer**: agent (automated assessment)

## Executive Summary

**wave_psp_rng_1 is NOT production-ready.** The wave has not started implementation. The deterministic RNG state model (`RandomState`, `Random`, `SystemRandom`, `seed`/`getstate`/`setstate`, `randbytes`) is completely absent from the codebase. The architecture lock validation required by the phase document has not been completed.

---

## Current Implementation State

### Random Module (`lib/sifr/random.sifr`)

**Shipped Surface:**
- `randint(min: int, max: int) -> Result[int, ValueError]`
- `random() -> float`
- `uniform(min: float, max: float) -> float`
- `shuffle[T](mut items: list[T]) -> None`
- `sample[T](items: list[T], k: int) -> Result[list[T], ValueError]`
- `randrange(start: int, stop: int | None = None, step: int = 1) -> Result[int, ValueError]`
- `getrandbits(k: int) -> Result[int, ValueError]`
- `gauss(mu: float, sigma: float) -> float`
- `choice[T](items: list[T]) -> Result[T, ValueError]`
- `choices[T](items: list[T], k: int = 1) -> Result[list[T], ValueError]`

**NOT Shipped (as defined by wave_psp_rng_1 scope):**
- `RandomState` - typed value object with fields: `version`, `state_words`, `index`, `gauss_next`
- `Random` - MT19937-compatible stateful generator
- `SystemRandom` - non-deterministic host-backed RNG
- `seed(...)` - deterministic seeding
- `getstate() -> RandomState` - state serialization
- `setstate(state: RandomState) -> None` - state deserialization
- `randbytes(n: int) -> Result[bytes, ValueError]` - raw byte generation

### Intrinsics Layer (`crates/sifr_codegen/src/intrinsics/random.rs`)

The current implementation uses Rust's `rand` crate with `thread_rng()`:
- Non-deterministic, thread-local RNG
- No MT19937 state model
- No state serialization/deserialization paths
- No `randbytes` intrinsic

### Waiver Ledger Status

From `verification/stdlib/milestone_psp_7_parity_governance_inventory.md` (line 159):

| Surface | Terminal State | Rationale |
| --- | --- | --- |
| Weighted/stateful random families (`choices(weights=...)`, `seed`/`getstate`/`setstate`) | `unsupported` | Current randomness layer intentionally avoids deterministic mutable RNG object model. |

This waiver remains active - wave_psp_rng_1 has not yet addressed it.

---

## Missing Parity for wave_psp_rng_1

### Core Typed State Model (REQUIRED)

1. **`RandomState` type** - Not implemented
   - Fields: `version: int`, `state_words: list[int]`, `index: int`, `gauss_next: float | None`
   - Must be a typed value object, NOT a raw Python tuple

2. **`Random` class** - Not implemented
   - MT19937-compatible state semantics
   - Methods: `random()`, `randint()`, `randrange()`, `choice()`, `choices()`, `sample()`, `shuffle()`, `gauss()`, `uniform()`, `getstate()`, `setstate()`, `seed()`, `randbytes()`

3. **`SystemRandom` class** - Not implemented
   - Non-deterministic, host-backed
   - Must NOT support `getstate`/`setstate`

4. **Module-level delegation** - Not implemented
   - `seed()`, `getstate()`, `setstate()` delegating to module-global `Random` instance

5. **`randbytes(n: int)`** - Not implemented
   - Must return canonical raw-byte-backed `bytes`
   - Must NOT materialize widened integer storage internally

### Architecture Lock Validation (REQUIRED BEFORE IMPLEMENTATION)

Per the phase document (lines 279-292), these architecture lock items must exist BEFORE wave_psp_rng_1 begins:

| Item | Status |
| --- | --- |
| Implementation note mapping `RandomState` fields to MT19937 internal state | NOT DONE |
| Implementation note defining `SystemRandom` host-boundary contract | NOT DONE |
| Sifr demo covering typed `RandomState` and module-global RNG proxy model | NOT DONE |
| Sifr demo covering bytes-native digest model | NOT DONE (hashlib wave) |
| Implementation note proving bytes-native RNG paths consume raw-byte-backed `bytes` | NOT DONE |
| Negative-path test for every newly explicit permanent divergence | NOT DONE |
| CPython-family mapping table (adopted/adapted/permanently waived) | NOT DONE |
| Explicit phase test families covering `test_random` | NOT DONE |
| Compile-time rejection or negative runtime case for typed surface divergence | NOT DONE |
| Dependency audit note for SHA3/SHAKE families in Rust dependency stack | NOT DONE |

---

## Missing Tests (CPython Itertools/Random-Related)

### CPython Test Adaptation

The phase document specifies these CPython test targets (lines 262-269):

| CPython Test File | Status |
| --- | --- |
| `Lib/test/test_random.py` | NOT PORTED |
| `Lib/test/test_hashlib.py` | NOT PORTED |
| `Lib/test/test_base64.py` | NOT PORTED |
| `Lib/test/test_statistics.py` | NOT PORTED |
| `Lib/test/test_textwrap.py` | NOT PORTED |
| `Lib/test/test_html.py` | NOT PORTED |

### Current Test Coverage

**Existing tests:**
- `crates/sifr/tests/e2e/pass/cpython_random_subset.sifr` - Basic functional tests for non-stateful functions
- `crates/sifr/tests/e2e/pass/stdlib_random.sifr` - Likely similar basic coverage

**Missing test coverage:**
- No MT19937 state serialization tests (`getstate`/`setstate` round-trip)
- No deterministic seeding tests (`seed()` reproducibility)
- No `Random` / `SystemRandom` object model tests
- No `randbytes` tests
- No weighted `choices(weights=...)` tests (explicitly out of scope per phase doc line 97)
- No CPython `test_random.py` family adaptation

---

## Production-Readiness Blockers

### Critical Blockers (MUST FIX BEFORE IMPLEMENTATION)

1. **Architecture lock validation incomplete**
   - None of the 10 required architecture lock items (lines 279-292) have been completed
   - The phase explicitly states these must exist BEFORE implementation begins

2. **No MT19937 implementation**
   - Current implementation uses `rand::thread_rng()` which is non-deterministic
   - No stateful PRNG with serialization support

3. **Waiver not updated**
   - The `milestone_psp_7_parity_governance_inventory.md` still lists stateful RNG as `unsupported`
   - No migration path from `unsupported` to `parity-closed`

4. **No CPython test adaptation**
   - `test_random.py` has not been ported or adapted
   - No traceability between upstream test cases and Sifr implementation

### High-Priority Blockers (MUST FIX FOR PRODUCTION GRADE)

5. **No `randbytes` implementation**
   - Required by phase spec (line 95-96)
   - Must return raw-byte-backed `bytes` without integer widening

6. **No typed `RandomState` type**
   - The waiver explicitly mentions "typed value object" requirement
   - Current implementation has no state representation

7. **No `SystemRandom` boundary definition**
   - Required for production-grade crypto randomness
   - Host-boundary contract not documented

---

## Governance Assessment

### Phase Exit Gate Status

Per the phase document (lines 304-312), exit requires:

| Exit Gate Criterion | Status |
| --- | --- |
| `random` stateful-object waiver family materially reduced | NOT STARTED - waiver still `unsupported` |
| `hashlib` advanced algorithm/digest waivers reduced | NOT STARTED |
| Targeted polish modules no longer carry vague debt | NOT STARTED |
| Full validation suite is green | UNKNOWN - not implemented |
| External review confirms production-grade closure | NOT DONE |

### Wave Completion Criteria

Per the phase document (lines 228-233), wave_psp_rng_1 definition of done:

| Criterion | Status |
| --- | --- |
| Stateful RNG parity materially stronger | NOT STARTED |
| `seed`/`getstate`/`setstate` shipped or sharply waived | NOT STARTED - still waived |
| Generator object model shipped or sharply waived | NOT STARTED |
| Local coverage proves deterministic behavior | NOT STARTED |
| Typed failure boundaries validated | NOT STARTED |

---

## Recommendations

### Before Implementation Begins

1. **Complete architecture lock validation** - All 10 items from lines 279-292 must be documented
2. **Define MT19937 implementation approach** - Choose between:
   - Pure Rust implementation (no external dependency)
   - Rust crate with state exposure (e.g., `rand` with `StdRng` + `SeedableRng`)
3. **Finalize `RandomState` type definition** - Map to internal MT19937 state words
4. **Define `SystemRandom` host boundary** - Document entropy source and failure modes

### Implementation Sequence

1. **Phase 1**: Implement `RandomState` type and `Random` class with MT19937
2. **Phase 2**: Add module-level `seed`/`getstate`/`setstate` delegation
3. **Phase 3**: Implement `SystemRandom` with host entropy
4. **Phase 4**: Add `randbytes` with raw-byte-backed `bytes`
5. **Phase 5**: Port CPython `test_random.py` coverage
6. **Phase 6**: Update waiver ledger and governance inventory

### Test Coverage Requirements

- Deterministic seeding reproducibility tests
- State serialization round-trip tests (`getstate` -> Python -> Sifr -> `setstate`)
- `randbytes` output validation (correct length, raw bytes)
- Negative path tests for invalid state
- CPython behavioral equivalence tests (where deterministic)

---

## Conclusion

**wave_psp_rng_1 is not production-ready and has not started implementation.** The deterministic RNG state model defined in the phase document (`RandomState`, `Random`, `SystemRandom`, stateful functions) is completely absent from the codebase. The architecture lock validation required before implementation has not been completed.

The current random module implementation uses non-deterministic `thread_rng()` and has no stateful object model. The existing waiver in `milestone_psp_7_parity_governance_inventory.md` classifying stateful RNG as `unsupported` remains active.

**Action Required**: Complete architecture lock validation items before beginning implementation. This is a prerequisite per the phase document and must not be skipped.
