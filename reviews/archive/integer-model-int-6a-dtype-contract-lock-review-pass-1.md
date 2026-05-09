

Review complete. All sentinels present, script passes, lanes wired correctly, design aligned. No blockers.

---

## INT-6A dtype contract lock review pass 1 — satisfied

### Scope confirmed

The slice adds exactly what INT-6A requires:

| Contract element | Status |
|---|---|
| Canonical dtype names (int8–int64, uint8–uint64, isize/usize) | Present in contract §1, design §2 and §5, inventory §2.5 |
| Default arithmetic: dtype-preserving, fallible (`Result[array[X], OverflowError]`) | Present in contract §3, design §8, inventory §2.5 |
| Explicit overflow policy APIs (checked/wrapping/saturating/overflowing/widen) | Present in contract §3, design §8, inventory §2.5 |
| `SIFR-INT-0008` future emission for missing overflow policy | Present in contract §4, design §9 (diagnostics table), inventory §2.5 |
| Explicit dtype required for `list[int]` → compact storage | Present in contract §2, design §8 ("Creating a column/tensor from `list[int]` requires an explicit dtype"), inventory §2.5 |
| Arrow/Parquet integer schema → matching fixed-width Sifr dtypes | Present in contract §5, design §10, inventory §2.5 |
| No silent widen of external integer columns | Present in contract §5, inventory §2.5 |
| Validation suite wired into quick/pr/nightly/release lanes | Present in `validation_contracts/manifest.json` §7 and `validation_lanes/manifest.json` matrix suites |
| Sentinel-gated check script | Present at `scripts/check_integer_dtype_contract.py` |

### Sentinel validation

All 6 REQUIRED_TEXT strings match the contract file:

| Sentinel | Contract location |
|---|---|
| `array[int32] + array[int32] -> Result[array[int32], OverflowError]` | contract.md:45, contract.md:101 |
| `array[int32] + array[int32]` cannot silently wrap | contract.md:50, contract.md:102 |
| `array[int32] + array[int32]` cannot accidentally widen to `array[int]` | contract.md:50, contract.md:103 |
| Constructing compact column, tensor, or array storage from `list[int]` requires an explicit dtype | contract.md:18, contract.md:104–105 |
| `SIFR-INT-0008` | contract.md:67, contract.md:106 |
| Arrow and Parquet integer columns map to matching fixed-width Sifr dtypes | contract.md:80, contract.md:107 |
| must not silently widen external integer columns to source-level | contract.md:93 |

`python3 scripts/check_integer_dtype_contract.py` exits 0. `git diff --check` is clean.

### Design alignment

- Contract §1 canonical dtype names match design §2 source types table.
- Contract §3 default arithmetic contract matches design §8 ("Array/tensor/dataframe arithmetic is a carve-out from scalar fixed-width promotion... returns `Result[array[int32], OverflowError]` by default").
- Contract §4 `SIFR-INT-0008` definition ("fixed-width dtype arithmetic operation without an explicit overflow policy emits SIFR-INT-0008") matches design §9 diagnostics table entry.
- Contract §5 Arrow/Parquet mapping table matches design §10 section.
- Design §8 explicitly cross-references the validation contract artifact: "The reviewable contract artifact for this deferred dtype surface is `verification/validation_contracts/integer_dtype_contract.md`".
- Inventory §2.5 correctly points at the contract and check script.

### Gate strength

The sentinel approach is sound for a pre-runtime lock:
- Any PR that removes or weakens the contract text fails the check script in quick/pr/nightly/release.
- The `Validation Sentinels` section (§7) explicitly documents that future runtime work can replace sentinels with executable fixtures only after owning data-science surfaces exist, and only if replacement still fails closed for silent wrapping or implicit widening.

The one structural observation (not a blocker): the replacement clause means INT-6B must produce an executable fixture that the validation suite can run, so the lane gate continues to enforce the contract even after the text-based sentinel approach is retired. This is already called out in the contract; INT-6B implementation should confirm it.

### Review artifact

This review is at `reviews/integer-model-int-6a-dtype-contract-lock-review-pass-1.md`. The phase tracker INT-6A checklist item (currently unchecked) should be updated after the PR lands with the review link and validation confirmation.
