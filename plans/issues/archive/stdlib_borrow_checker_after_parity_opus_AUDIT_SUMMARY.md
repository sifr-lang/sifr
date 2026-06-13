# Borrow/Ownership Design vs. Stdlib Implementation Audit

Generated: 2026-02-17

## Scope and Method

- **Architecture reference:** `internal_docs/architecture.md` — Contracts #2 (Borrow and Lifetime Strategy), #3 (Error Semantics Matrix), #6 (Slice and Collection Semantics), #7 (String Semantics), #9 (Destruction and Cleanup), #10 (Auto-Derived Traits)
- **Stdlib surface:** All 37 modules in `lib/sifr/*.sifr`
- **Compiler implementation:** `crates/sifr_hir/src/scope.rs` (ownership tracking), `crates/sifr_hir/src/lower.rs` (borrow checking), `crates/sifr_codegen/src/lib.rs` (Rust code generation), `crates/sifr_hir/src/stdlib.rs` (intrinsic type signatures), `crates/sifr_type_system/src/types.rs` (OwnershipKind, ParamConvention)
- **Existing audit baseline:** `audits/borrowing/REPORT.md` (50 borrow/ownership test files)

## Headline Findings

| Metric | Value |
| --- | --- |
| Total stdlib modules examined | 37 |
| Contradictions with architecture found | 10 |
| Critical severity | 1 (I/O `.unwrap()` across 6+ modules) |
| High severity | 4 (list/set/min/max panics, subscript-assign unsafe) |
| Medium severity | 4 (sentinel returns, generator limitations, `mut` untested) |
| Low severity | 1 (Counter JSON workaround) |
| Stdlib functions using `mut` parameters | 0 of ~150+ |
| Stdlib functions using `own` parameters | 0 of ~150+ |
| Codegen `.unwrap()` calls in intrinsic emission | 25+ |
| Architecture "no panics" promise | Systematically violated |

## Core Question: Does the Ownership Model Cover the Stdlib?

**The borrow-by-default read path is well covered. The safety contract and mutation paths are not.**

The ownership model has three layers:
1. **Borrow-by-default (read)** — Works correctly. The vast majority of stdlib functions only read their parameters via `&T`. This path is sound.
2. **Mutable borrowing (`mut`)** — Completely untested by stdlib. Zero functions use `mut` parameters. The stdlib avoids in-place mutation entirely.
3. **Ownership transfer (`own`)** — Completely untested by stdlib. Zero functions use `own` parameters. The stdlib never consumes arguments.

Meanwhile, the architecture's **safety contract** (Result/Option for all fallible operations, no panics) is **systematically violated** by the codegen layer, which uses `.unwrap()` for I/O, collection methods, and built-in functions.

## Contradiction Summary Table

| # | Contradiction | Architecture Clause | Severity | Affected Modules |
| --- | --- | --- | --- | --- |
| 1 | I/O operations use `.unwrap()` instead of `Result` | Safety Philosophy; Contract #3 | **Critical** | io, pathlib, shutil, os, tempfile, tomllib |
| 2 | `list.remove()`/`list.index()` panic on missing value | Safety Philosophy; Contract #7 | **High** | Any code using list methods |
| 3 | `min()`/`max()` panic on empty lists | Safety Philosophy; Contract #3 | **High** | Built-in functions |
| 4 | `SubscriptAssign` (`x[i] = val`) bypasses safe indexing | Contract #7 (safe indexing) | **High** | graphlib, any index-write code |
| 5 | `set.pop()` panics on empty set | Safety Philosophy | **High** | Any code using set methods |
| 6 | Statistics functions return `0.0` sentinel instead of `Result` | Safety Philosophy; Contract #3 | **Medium** | sifr.statistics |
| 7 | `heappop()` returns `0` sentinel instead of `Option` | Safety Philosophy; Contract #3 | **Medium** | sifr.heapq |
| 8 | No `mut` parameters in entire stdlib | Contract #2 (borrow strategy) | **Medium** | Design coverage gap |
| 9 | Generator codegen can't handle borrowed params in conditions | Contract #2; Contract #12 (Iterator) | **Medium** | sifr.itertools, lazy iteration |
| 10 | `Counter` uses JSON string to avoid ownership complexity | Contract #2 (borrow strategy) | **Low** | sifr.collections |

## Ownership Model Coverage Matrix

| Ownership Feature | Architecture Status | Stdlib Usage | Compiler Enforcement |
| --- | --- | --- | --- |
| Borrow-by-default (`&T`) | Defined in Contract #2 | Used everywhere | Codegen emits `&T` correctly |
| Mutable borrow (`mut` / `&mut T`) | Defined in Contract #2 | **Never used** | Codegen exists but untested by stdlib |
| Ownership transfer (`own` / `T`) | Defined in Contract #2 | **Never used** | Codegen exists but untested by stdlib |
| Copy types pass by value | Defined in Contract #2 | Used correctly | `int`, `float`, `bool` bypass borrows |
| Move-on-assignment | Defined in Ownership Model | Implicitly used | Scope tracking works |
| Method receiver inference (`&self`/`&mut self`) | Defined in Contract #2 | Used in classes | Body analysis determines receiver |
| Closure capture inference | Deferred to milestone_generics | Not applicable | Not implemented |
| Escape analysis | Defined in Contract #2 | Not exercised | Partial implementation |
| No lifetime annotations | Defined in Contract #2 | Correct | Lifetimes inferred |
| Shared mutable state (`Rc<RefCell<T>>`) | Deferred post-protocols | Not applicable | Not implemented |
| `for` loop borrows collection | Defined in Contract #6 | Used correctly | `.iter().cloned()` codegen |
| Slice copies (not views) | Defined in Contract #6 | Used correctly | `.to_vec()` / `.chars()` codegen |

## Borrow Checker Test Suite Status

From `audits/borrowing/REPORT.md` (50 tests):

| Status | Count | % |
| --- | --- | --- |
| PASS | 29 | 58% |
| Fail (Sifr compile — correct rejections) | 12 | 24% |
| Fail (Rust compile — codegen bugs) | 7 | 14% |
| Fail (Runtime) | 2 | 4% |

The 7 Rust compile failures indicate **codegen regressions** in the borrow-by-default path:
- `&String` vs `String` comparison mismatches (3 tests)
- Class field destructuring generates use-after-move (1 test)
- Method `&mut self` type mismatches (1 test)
- Generic function signature mismatches (1 test)
- `dyn Any` missing trait bounds on chained methods (1 test)

These codegen bugs affect patterns that the stdlib would use if it exercised `mut`/`own` paths.

## Detailed Reports

| Report | Contents |
| --- | --- |
| [SAFETY_VIOLATIONS.md](./SAFETY_VIOLATIONS.md) | Every `.unwrap()` and panic path in codegen, with line references |
| [OWNERSHIP_COVERAGE_GAPS.md](./OWNERSHIP_COVERAGE_GAPS.md) | Analysis of `mut`/`own` avoidance patterns in stdlib |
| [CODEGEN_EVIDENCE.md](./CODEGEN_EVIDENCE.md) | Exact code references for each contradiction |
| [RECOMMENDATIONS.md](./RECOMMENDATIONS.md) | Prioritized fix plan with effort estimates |
