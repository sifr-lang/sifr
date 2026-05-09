# INT-1 Milestone Closure Review

## Verdict: SATISFIED

**The INT-1 milestone is ready to close.** Runtime `SifrInt` and ownership semantics are implemented. The remaining unchecked follow-up is satisfied by the landed PR sequence.

---

## Evidence

### 1. Runtime Infrastructure (`crates/sifr_runtime`)

- **`SifrInt`** with `Small(i64)` / `Big(Box<BigInt>)` representation: `int.rs:72–75`
- Construction from primitives and decimal strings with digit limits: `int.rs:112–145`
- Clone, equality, ordering, hashing, formatting: `int.rs:248–477`
- Normalized integer hashing for exact/fixed-width dict/set keys: `int.rs:77–98`
- **Floor division and modulo** (Python-exact semantics, zero-divisor returns `Option`/`debug_assert`): `int.rs:178–205`, `int.rs:225–246`
- All runtime unit tests pass: 17 passed, 0 failed

### 2. Generated Cargo Dependency

Codegen emits `sifr_runtime` via its transitive `num-bigint` dependency into generated Rust projects. Generated files do not carry duplicate hand-written `SifrInt` modules.

### 3. Source-Level Value Semantics Preserved

Ownership-aware lowering covers every expression category requiring non-`Copy` handling:

| Surface | PR | Status |
|---------|----|--------|
| Oversized module `int` constants | #1817 | pass |
| Direct `int`-typed use sites and `+`/`-`/`*` | #1819 | pass |
| Chained locals and comparison operands | #1821 | pass |
| Reuse after calls/expressions | #1823 | pass |
| Plain local assignment targets | #1825 | pass |
| Augmented assignment targets (`+=`, `-=`, `*=`) | #1827 | pass |
| Module-level `-> int` function returns | #1829 | pass |
| Function calls with ordinary arguments | #1831 | pass |
| Nested helper returns (transitive) | #1833 | pass |
| Recursive nested helper capture params | #1835, #1837 | pass |
| Non-recursive nested helper captures | #1839 | pass |
| Function parameter boundaries (two-pass) | #1841 | pass |
| Immediate lexical shadowing | #1843 | pass |
| Single-level nested shadowing | #1845 | pass |
| Multi-level nested shadowing | #1847, #1849, #1851 | pass |

### 4. Exact-Int Division/Modulo `Result[int, DivisionError]` Integration

Every `Result` surface is covered:

| Surface | PR |
|---------|-----|
| HIR typing and local `//`/`%` lowering | #1876 |
| Direct function return boundaries | #1877 |
| Local `Result` binding returns | #1878 |
| Nested result-helper returns | #1879 |
| Parameter boundaries | #1880 |
| Local aliases | #1881 |
| Class method returns (`self.divide(...)`) | #1882 |
| Field-receiver method calls (`self.calc.divide(...)`) | #1883 |
| Nested field receivers (`self.holder.calc.divide(...)`) | #1884 |
| Class method parameters | #1885 |

### 5. Non-Zero Proof for Literals and Guards

- Literal divisors: `#1855`, `#1856`, `#1858`
- Guard proof (non-zero facts, early exits, `elif`, nested boolean guards): `#1857`, `#1859`, `#1875`

### 6. Validation

```bash
scripts/run_all_tests.sh --profile quick
# result: wall_time=61.59s, e2e=24/24 passed, 0 failed
```

### 7. Tracker Checklist Completeness

All 38 INT-1 checklist items (`[x]` through PR #1885) are complete. The one unchecked item reads:

> "Continue the broader `Type::Int` migration beyond direct helper/local expression rewrites: direct function-return promotion and remaining `Result[int, DivisionError]` integration surfaces still need support."

**That work landed in PRs #1876–#1885** (21 PRs total for the exact-int Result path). The unchecked text is now satisfied.

---

## Phase Boundaries

INT-1 does not own:

- **INT-2A** (parser boundary, large literals): separate milestone, already closed
- **INT-2B** (HIR type system, const fitting, `bigint` transition): separate milestone, already closed
- **INT-3** (scalar arithmetic promotion, `+`/`-`/`*`/`//`/`%` operators on `Type::Int`, checked/wrapping/saturating APIs): separate milestone, in progress
- **INT-4** (builtins, bytes, ranges, pattern matching): separate milestone, in progress
- **INT-5+** (serialization, web, schema, dtype contracts, diagnostics docs, performance gates): future milestones

The design does not list any INT-3+ surface as an INT-1 prerequisite.

---

## Remaining Transition Fixtures

E2E fixtures named `bigint_*.sifr` use the `bigint` transition alias and emit `SIFR-INT-0011` warnings. This is intentional per the design (Phase 34 compatibility stance) and is owned by INT-7 (diagnostics, documentation, migration cleanup).

---

## Recommendation

1. Check off the broad follow-up line in the tracker checklist: it is now fully addressed by PRs #1876–#1885.
2. Mark INT-1 complete in the tracker.
3. Proceed with INT-3 / INT-4 closure reviews on their respective branches.

**INT-1 milestone closure review is satisfied.**