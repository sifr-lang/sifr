# Phase Review: Ad Hoc Stateful RNG, Crypto, and Polish Parity Expansion

## Review Scope

Focus: `wave_psp_rng_1` deterministic RNG state/object model closure quality, parity claims, governance docs, and test completeness.

Reference:
- Planning doc: `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`
- Execution ledger: `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-execution.md`
- Architecture lock: `verification/stdlib/phase_psp_rng_architecture_lock.md`
- Wave traceability: `verification/stdlib/wave_psp_rng_0_cpython_traceability.md`, `verification/stdlib/wave_psp_rng_1_cpython_traceability.md`

## Executive Summary

**Status**: `wave_psp_rng_0` completed; `wave_psp_rng_1` implementation + validation completed, pending PR/review/merge loop.

The deterministic RNG state/object model implementation (`wave_psp_rng_1`) is well-structured and achieves material closure of the stateful random surface. The implementation follows MT19937-compatible semantics, provides typed state containers, and correctly maintains the distinction between deterministic `Random` and non-deterministic `SystemRandom`. Governance documentation is complete and traceability is established.

## wave_psp_rng_1 Implementation Quality

### Deterministic RNG State Model

The implementation correctly ships a typed deterministic state model:

| Component | Status | Assessment |
|-----------|--------|------------|
| `RandomState` class | Shipped | Correctly typed with `version: int`, `state_words: list[int]`, `index: int`, `gauss_next: float \| None` |
| `Random` class | Shipped | MT19937-compatible implementation with proper state management |
| `seed()` | Shipped | Normalizes input and seeds MT19937 state correctly |
| `getstate()` / `setstate()` | Shipped | Correct state round-trip with typed `Result` boundaries |
| `randbytes()` | Shipped | Returns first-class `bytes` via raw-byte-backed storage |

**MT19937 Constants Verified**:
- `_MT_N = 624` (state word count) ✓
- `_MT_M = 397` (twist parameter) ✓
- `_MT_MATRIX_A = 2567483615` (twist matrix) ✓
- `_MT_F = 1812433253` (seed generation) ✓

### Module-Global Delegation

The module-level functions correctly delegate to one shared module-global `Random` instance:
- `seed()`, `getstate()`, `setstate()`, `randrange()`, `randint()`, `random()`, `choice()`, `choices()`, `sample()`, `shuffle()`, `gauss()`, `uniform()`, `randbytes()`

**Verified in source** (`lib/sifr/random.sifr`):
- `_build_state_from_module_storage()` reads from module state
- `_store_state_into_module_storage()` writes to module state
- `_ensure_module_state_initialized()` bootstraps on first use

### Codegen Quality

The Rust codegen (`crates/sifr_codegen/src/intrinsics/random.rs`) correctly:

1. **State Management Intrinsics**:
   - `lower_random_module_state_words()` - reads state words
   - `lower_random_module_state_index()` - reads state index
   - `lower_random_module_state_gauss_next()` - reads gauss_next
   - `lower_random_module_set_state()` - validates and sets state

2. **Validation in Set State**:
   - Index bounds check: `[0, 624]`
   - State words length check: exactly 624
   - Returns `Result[None, ValueError]` for invalid inputs

3. **Thread Safety**:
   - Uses `__SIFR_RANDOM_MODULE_STATE.lock()` for module-level state
   - Proper mutex handling with `unwrap_or_else()`

## Parity Claims Assessment

### Shipped (Adapted)

| CPython Surface | Sifr Direction | State |
|-----------------|----------------|-------|
| `Random` class | typed deterministic state container | ✓ Shipped |
| `RandomState` class | `version, state_words, index, gauss_next` | ✓ Shipped |
| `seed()`, `getstate()`, `setstate()` | module-level delegation | ✓ Shipped |
| `randbytes(n)` | first-class `bytes` return | ✓ Shipped |
| `randrange`, `randint`, `random`, `choice`, `sample`, `shuffle`, `gauss`, `uniform` | deterministic module-level helpers | ✓ Shipped |

### Explicitly Unsupported

| CPython Surface | State | Rationale |
|-----------------|-------|-----------|
| `choices(weights=...)` | `unsupported` | Weighted distribution requires additional implementation; correctly left out of wave 1 |
| `SystemRandom.getstate()` / `setstate()` | `unsupported` | Host-random is non-deterministic by design; correctly not claimed |

**Parity Claims**: ✓ Correctly classified. Both unsupported surfaces have negative test coverage.

## Test Completeness

### Positive Coverage

| Test | Purpose | Status |
|------|---------|--------|
| `phase_psp_rng_1_stateful_random_object_model.sifr` | Deterministic state replay, state words validation, randbytes | ✓ Present |
| `ad_hoc_rng_wave1_stateful_object_model_demo.sifr` | End-to-end demo of state round-trip | ✓ Present |

**Test assertions verified**:
- Deterministic replay: `assert_eq(second, replay_second)` ✓
- State structure: `assert_eq(len(checkpoint.state_words), 624)` ✓
- Index bounds: `assert_true(checkpoint.index >= 0 and checkpoint.index <= 624)` ✓
- Module-level delegation replay: `assert_eq(module_a3, module_a3_replay)` ✓
- randbytes determinism: `assert_eq(str(bytes_a), str(bytes_b))` ✓

### Negative Coverage

| Test | Purpose | Status |
|------|---------|--------|
| `phase_psp_rng_1_system_random_state_unsupported.sifr` | SystemRandom.getstate() rejection | ✓ Present |
| `phase_psp_b2_random_choices_weights_unsupported.sifr` | Weighted choices rejection | ✓ Present (regression) |

### Validation Evidence (from execution doc)

```
- positive path: cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_rng_1_stateful_random_object_model.sifr -> PASS
- positive path: cargo run -q -p sifr -- run demos/ad_hoc_rng_wave1_stateful_object_model_demo.sifr -> PASS
- negative path: cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_rng_1_system_random_state_unsupported.sifr -> expected compile failure (PASS)
- negative regression: cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_random_choices_weights_unsupported.sifr -> expected compile failure (PASS)
- unit lane: cargo test -p sifr -- --skip test_e2e_pass -> PASS
- wave gate: $(pwd)/scripts/run_all_tests.sh -> PASS (2026-03-21)
```

**Test Completeness**: ✓ Complete. Both positive and negative coverage are present.

## Governance Documentation Quality

### Architecture Lock (`phase_psp_rng_architecture_lock.md`)

| Section | Status | Assessment |
|---------|--------|------------|
| Locked Public Contract | ✓ Present | Correctly documents `RandomState` shape, bytes API, SystemRandom boundary |
| Baseline Fractures | ✓ Present | Documents pre-wave state (stateless wrappers) |
| Permanent Sifr-Safe Diffs | ✓ Present | Buffer protocol, memoryview, SystemRandom state, decimal/fraction statistics |
| CPython Family Mapping | ✓ Present | Correct wave ownership assigned |

### Wave Traceability

| Document | Status |
|----------|--------|
| `wave_psp_rng_0_cpython_traceability.md` | ✓ Present |
| `wave_psp_rng_1_cpython_traceability.md` | ✓ Present |

Both traceability matrices correctly map CPython families to Sifr surface directions with state classifications.

### Milestone Inventory Update

Verified in `milestone_psp_7_parity_governance_inventory.md`:
- `random` module now lists: `wave_psp_b2 + wave_psp_rng_1`
- Residual waiver family correctly updated: "Deterministic mutable-state random object parity (`RandomState`, `Random`, module-level delegation, and `randbytes`) is now shipped by `wave_psp_rng_1`"

**Governance**: ✓ Complete and correct.

## Issues and Findings

### Finding 1: No CPython Test Porting Evidence

**Severity**: Low (informational)

The phase document (`ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`) lists CPython test porting targets:
- `Lib/test/test_random.py`

However, there is no explicit evidence in the traceability or execution docs showing which specific test functions from `test_random.py` were ported or adapted. The current evidence shows:
- Positive fixture coverage for deterministic behavior
- Module-level function coverage

**Recommendation**: Consider adding a CPython test mapping table to `wave_psp_rng_1_cpython_traceability.md` that explicitly lists which test cases from `Lib/test/test_random.py` are covered by the positive fixtures. This would strengthen the traceability claim.

### Finding 2: `gauss_next` Serialization

**Severity**: Low (acceptable design decision)

The implementation stores `gauss_next: float | None` in `RandomState`. This is correct for Gaussian distribution state (the Box-Muller polar method stores one precomputed value). However, the codegen `lower_random_module_state_gauss_next()` clones this value:

```rust
receiver: Box::new(RustExpr::MethodCall {
    receiver: Box::new(RustExpr::Field { ... }),
    method: "clone".to_string(),
    ...
})
```

This works but could be simplified since `float` implements `Copy` in Rust. This is a minor optimization and not a correctness issue.

**Status**: Acceptable. No action required.

### Finding 3: Wave 1 Pending PR Merge

**Severity**: Informational

The execution doc shows `wave_psp_rng_1` status as "implementation + validation completed (pending PR/review/merge loop)". This is appropriate - the implementation is complete but not yet merged to main.

**Status**: Expected state for this phase of work.

## Review Verdict

| Criterion | Status | Notes |
|-----------|--------|-------|
| Deterministic RNG state model | ✓ Complete | MT19937-compatible, typed `RandomState` |
| Object model quality | ✓ Complete | Proper state encapsulation, typed boundaries |
| Parity claims | ✓ Correct | Shipped/unsupported correctly classified |
| Test coverage | ✓ Complete | Positive + negative fixtures present |
| Governance docs | ✓ Complete | Traceability, inventory, execution ledger all updated |
| Local validation | ✓ Pass | Full test suite passes |

### Recommendation

**APPROVED** for production-grade review and merge pending.

The `wave_psp_rng_1` implementation demonstrates:
1. Correct MT19937 deterministic state model
2. Proper typed boundaries with `Result` error handling
3. Complete test coverage (positive + negative)
4. Governance documentation is complete and accurate

The residual unsupported items (`choices(weights=...)`, `SystemRandom` state) are correctly classified and have negative test guards.

### Post-Merge Action Items

After `wave_psp_rng_1` merges:
1. Update phase execution doc to mark wave 1 as merged
2. Proceed to `wave_psp_rng_2` (hashlib bytes-native expansion) when ready

---

*Review completed: 2026-03-21*
*Reviewer: agent*
*Phase: ad-hoc-stateful-rng-crypto-and-polish-parity-expansion*
*Wave: wave_psp_rng_1*
