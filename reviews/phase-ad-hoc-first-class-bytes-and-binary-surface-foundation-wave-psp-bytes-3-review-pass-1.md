# Review: wave_psp_bytes_3 Downstream Contract Adoption (Review Pass 1: Completion-Gap)

**Wave**: `wave_psp_bytes_3` (Downstream Contract Adoption and Governance Closeout)
**Phase**: `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
**Reviewer**: External completion-gap review
**Date**: 2026-03-19

---

## Scope of Review

This review examines the completion status of `wave_psp_bytes_3`, which is responsible for:
1. Migrating binary file I/O boundaries to first-class `bytes`
2. Adding fixture/demo coverage proving downstream contract adoption
3. Updating governance ledgers to reflect the new binary carrier contract

## Review Areas

### 1. Downstream Contract Adoption to First-Class Bytes

**Finding**: ✅ **COMPLETE** - No blockers identified.

**Evidence**:
- `lib/sifr/io.sifr` (lines 30-34) has been properly updated:
  - `read_bytes() -> Result[bytes, IOError]`
  - `write_bytes(data: bytes) -> Result[int, IOError]`
- Compile-time type checking correctly rejects `list[int]` as demonstrated by fail fixtures:
  - `phase_psp_bytes_3_write_bytes_rejects_int_list.sifr`: produces compile error "expected 'bytes', got 'list[int]'"
  - `phase_psp_bytes_3_read_bytes_not_list.sifr`: produces compile error "expected 'list[int]', got 'Result[bytes, IOError]'"

**Verification**:
```bash
$ cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_3_write_bytes_rejects_int_list.sifr
type error: argument 1 ('data') of FileHandle.write_bytes(): expected 'bytes', got 'list[int]'

$ cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_3_read_bytes_not_list.sifr
type error: type mismatch: expected 'list[int]', got 'Result[bytes, IOError]'
```

### 2. Fixture and Demo Coverage

**Finding**: ✅ **COMPLETE** - Comprehensive coverage verified.

**Positive-path fixtures**:
- `crates/sifr/tests/e2e/pass/phase_psp_bytes_3_downstream_contract_alignment.sifr` - Core downstream contract test
- `crates/sifr/tests/e2e/pass/open_binary_read.sifr` - Binary read fixture
- `crates/sifr/tests/e2e/pass/open_binary_write.sifr` - Binary write fixture
- `crates/sifr/tests/e2e/pass/cpython_io_subset.sifr` - CPython IO subset
- `crates/sifr/tests/e2e/pass/stdlib_io_consolidated.sifr` - Stdlib IO consolidation

**Negative-path fixtures**:
- `crates/sifr/tests/e2e/fail/phase_psp_bytes_3_write_bytes_rejects_int_list.sifr`
- `crates/sifr/tests/e2e/fail/phase_psp_bytes_3_read_bytes_not_list.sifr`

**Demo**:
- `demos/ad_hoc_bytes_wave3_downstream_contract_adoption_demo.sifr` - Demonstrates bytes file-handle roundtrip with `to_ints()` conversion

**Verification**:
```bash
$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_bytes_3_downstream_contract_alignment.sifr
# PASS

$ cargo run -q -p sifr -- run demos/ad_hoc_bytes_wave3_downstream_contract_adoption_demo.sifr
# PASS
```

### 3. Governance Ledger Consistency

**Finding**: ✅ **COMPLETE** - Ledger properly updated.

**Evidence**:

**a) milestone_psp_7_parity_governance_inventory.md**:
- Line 49: `bytes` (first-class immutable surface) now references `wave_psp_bytes_3_cpython_traceability.md`
- Line 58: `bytes` module closure includes `wave_psp_bytes_3` in the closure wave chain
- Line 127: Canonical CPython Adopt/Adapt/Waive ledger entry for `wave_psp_bytes_3`

**b) wave_psp_bytes_3_cpython_traceability.md**:
- Created and populated with adopt/adapt/waive matrix
- Lists pass/fail fixtures and demos
- Classifies remaining binary waivers (bytearray, memoryview, buffer protocol, non-UTF-8 codecs, hashlib bytes-native APIs, base64 bytes entrypoints)

**c) Successor phase alignment**:

- `issues/ad-hoc-runtime-and-file-object-parity-expansion.md` (lines 101-105):
  - Explicitly requires binary surfaces to use first-class `bytes` rather than `list[int]`
  - `read_bytes() -> Result[bytes, IOError]`
  - `write_bytes(data: bytes) -> Result[int, IOError]`

- `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`:
  - Line 93: `randbytes(n: int) -> Result[bytes, ValueError]`
  - Line 99: "Use first-class `bytes` as the canonical binary carrier"
  - Lines 102-104: `update_bytes(data: bytes)`, `digest() -> bytes`
  - Line 108: `new_bytes(name: str, data: bytes = bytes())`

**d) Remaining waiver classification**:
The governance ledger correctly maintains narrow waivers for:
- `bytearray` mutable object-model (deferred)
- `memoryview` and buffer protocol (deferred)
- Non-UTF-8 codec matrices (out of scope)
- `hashlib` bytes-native digest families (deferred to RNG/crypto phase)
- Direct bytes-oriented base64 entrypoints (deferred)

---

## Summary

| Review Area | Status | Notes |
|-------------|--------|-------|
| Downstream contract adoption | ✅ Complete | `io.sifr` properly typed with `bytes` |
| Compile-time rejection of `list[int]` | ✅ Complete | Verified with fail fixtures |
| Fixture coverage (positive) | ✅ Complete | 5 pass fixtures verified |
| Fixture coverage (negative) | ✅ Complete | 2 fail fixtures verified |
| Demo coverage | ✅ Complete | Single comprehensive demo |
| Governance ledger (milestone inventory) | ✅ Complete | Updated with wave_3 references |
| Governance ledger (traceability) | ✅ Complete | `wave_psp_bytes_3_cpython_traceability.md` created |
| Successor phase alignment | ✅ Complete | Both successors reference `bytes` |
| Waiver classification | ✅ Complete | Remaining waivers correctly narrowed |

---

## Verification Commands Run

```bash
# Positive path tests
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_bytes_3_downstream_contract_alignment.sifr  # PASS
cargo run -q -p sifr -- run demos/ad_hoc_bytes_wave3_downstream_contract_adoption_demo.sifr                 # PASS
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/open_binary_read.sifr                               # PASS
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/open_binary_write.sifr                             # PASS
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_io_subset.sifr                             # PASS
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_io_consolidated.sifr                        # PASS

# Negative path tests (expected compile failures)
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_3_write_bytes_rejects_int_list.sifr  # Type error: expected 'bytes', got 'list[int]'
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_3_read_bytes_not_list.sifr          # Type error: expected 'list[int]', got 'Result[bytes, IOError]'

# Unit tests
cargo test -p sifr -- --skip test_e2e_pass  # 25 passed, 0 failed
```

---

## Recommendation

**Status**: ✅ **APPROVED** - No remediation changes required.

The wave implementation is complete and meets all completion-gap criteria:
1. Binary I/O contracts properly migrated to first-class `bytes`
2. Comprehensive fixture and demo coverage in place
3. Governance ledgers consistent and properly updated
4. Successor phases correctly anchored on `bytes` as canonical binary carrier

The wave is ready for production-grade review (review pass 2).
