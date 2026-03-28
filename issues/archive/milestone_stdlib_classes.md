## Product Requirements & Solution Design

---

### 1. Product Requirements

#### **Title**

milestone_stdlib_classes: Class-Based APIs in Sifr Standard Library

---

#### **Objective / Problem Statement**

The CPython parity audit identified **class-based APIs as the single biggest blocker** — 12+ stdlib modules need classes to reach meaningful parity with CPython. While Sifr's compiler already supports user-defined classes (constructors, methods, `&self`/`&mut self` inference, inheritance, protocols, `isinstance`), **no stdlib `.sifr` module currently defines a class**. The infrastructure to compile, export, and import stdlib classes exists in the driver but has never been exercised end-to-end.

This milestone proves that stdlib classes work by implementing `collections.Counter` as a proof-of-concept, replacing the existing procedural/intrinsic-based Counter API (`counter_from_list`, `counter_get`, `counter_most_common`) with a proper class. This unblocks all 12+ modules that need class-based APIs.

**Why `Counter`?**
1. The existing `_sifr.collections` already has Counter intrinsics (`counter_from_list`, `counter_get`, `counter_most_common`) — so the Rust backing code exists.
2. Counter uses only primitive fields (`dict[str, int]` internally, exposed as `str` JSON-encoded) — no `Callable`-as-struct-field needed.
3. Counter exercises the full pipeline: class definition in `.sifr` → HIR lowering → export via `ExternalDefs.classes` → import in user code → codegen with `pub struct` + `pub fn new()` + methods.
4. Counter is a well-known CPython class with a clear, testable API.
5. Counter methods exercise both `&self` (read-only: `get`, `most_common`, `total`) and `&mut self` (mutating: `increment`, `subtract`) receiver inference.

---

#### **Constraints**

| Constraint | Rationale |
| --- | --- |
| No new parser features | Classes already parse correctly |
| No `Callable`-as-struct-field fix | Counter doesn't need `Callable` fields; that fix is deferred |
| No operator overloading on stdlib classes | `__add__`/`__sub__` for Counter deferred (works for user classes but needs stdlib export testing) |
| Backward-compatible | Old procedural intrinsics (`counter_from_list` etc.) remain available; new class is additive |
| Single PoC class | Only `Counter`; other classes (Path, Logger, ArgumentParser, etc.) follow in future milestones |

---

#### **Business Goals & Success Criteria (KPIs)**

| Metric | Baseline (Today) | Target (Post-launch) |
| --- | --- | --- |
| Stdlib modules with class-based APIs | 0 | 1 (`collections.Counter`) |
| `sifr.collections` CPython parity | ~25% (procedural only) | ~40% (Counter class + existing functions) |
| Stdlib class import pipeline proven | Untested | Fully exercised end-to-end |
| Modules unblocked for future class work | 0 | 12+ (argparse, csv, logging, pathlib, etc.) |

---

#### **Scope**

##### ✅ Features In

1. **Define `Counter` class in `lib/sifr/collections.sifr`** — with `__init__`, `get`, `most_common`, `total`, `increment`, `values`, `keys`, `items` methods
2. **Add `_sifr.collections` intrinsics** for Counter's internal operations that need Rust backing (HashMap operations)
3. **Verify stdlib class export pipeline** — ensure `ExternalDefs.classes` correctly propagates `Counter` type to user code
4. **E2E pass tests** — import `Counter` from `sifr.collections`, construct, call methods, verify output
5. **E2E fail tests** — wrong constructor args, calling nonexistent methods, type mismatches
6. **Demo** — `demos/milestone_stdlib_classes_demo.sifr` showcasing Counter usage
7. **Update parity report** with new metrics

##### ❌ Features Out

| Feature | Reason for Exclusion |
| --- | --- |
| `Callable`-as-struct-field (`Box<dyn Fn>` fix) | Not needed for Counter; separate codegen task for future milestone |
| Operator overloading on Counter (`c1 + c2`) | Works for user classes but needs stdlib export testing; defer to avoid scope creep |
| `defaultdict` class | Requires `Callable` field for default factory; blocked by `Box<dyn Fn>` fix |
| `deque`, `OrderedDict`, `ChainMap`, `namedtuple` | Out of scope for PoC; follow-on work once pattern is proven |
| Generic Counter (`Counter[T]`) | Current Counter is `Counter` over `str` keys (matching existing intrinsics); generic version deferred |
| Iterator protocol on Counter | Requires iterator support; deferred |
| `Path`, `Logger`, `ArgumentParser`, etc. | Other modules' classes follow once this PoC proves the pipeline |

---

#### **Users / Stakeholders, Use-Cases & Dependencies**

| Persona | Use-Case / Benefit | Dependencies | AC-ID |
| --- | --- | --- | --- |
| Sifr developer | Use `Counter` class with familiar CPython-like API | Existing class compiler infrastructure | AC-1, AC-2 |
| Sifr developer | Import stdlib class and call methods | `ExternalDefs.classes` export pipeline | AC-3 |
| Compiler team | Proven pattern for adding more stdlib classes | This milestone's PoC | AC-4 |

---

### **Acceptance Criteria**

| **AC-ID** | Criterion |
| --- | --- |
| AC-1 | `Counter` class is defined in `lib/sifr/collections.sifr` with `__init__(self, data: str)`, `get(self, key: str) -> int`, `most_common(self, n: int) -> str`, `total(self) -> int`, `values(self) -> list[int]`, `keys(self) -> list[str]`, `items(self) -> str` methods |
| AC-2 | `Counter` methods correctly delegate to `_sifr.collections` intrinsics for HashMap operations |
| AC-3 | User code can `from sifr.collections import Counter` and construct/use it — the full export/import pipeline works |
| AC-4 | At least 2 E2E pass tests: (a) basic Counter construction + method calls, (b) Counter with mutation (`increment`) |
| AC-5 | At least 1 E2E fail test: wrong argument type to Counter constructor |
| AC-6 | Demo file `demos/milestone_stdlib_classes_demo.sifr` compiles and runs correctly |
| AC-7 | All existing tests pass (zero regressions) |
| AC-8 | Parity report updated |

---

## 2. Solution Design

### 2.1 Functional Requirements

**The `Counter` class API (in `lib/sifr/collections.sifr`):**

```python
class Counter:
    data: str  # JSON-encoded dict[str, int] (reuses existing intrinsic encoding)

    def __init__(self, data: str):
        self.data = data

    def get(self, key: str) -> int:
        return counter_get(self.data, key)

    def most_common(self, n: int) -> str:
        return counter_most_common(self.data, n)

    def total(self) -> int:
        return counter_total(self.data)

    def values(self) -> list[int]:
        return counter_values(self.data)

    def keys(self) -> list[str]:
        return counter_keys(self.data)

    def items(self) -> str:
        return counter_items(self.data)

    def increment(self, key: str) -> None:
        self.data = counter_increment(self.data, key)
```

**Factory function (convenience, matches CPython's `Counter(iterable)`):**

```python
def from_list(items: list[str]) -> Counter:
    return Counter(counter_from_list(items))
```

**New `_sifr.collections` intrinsics needed:**

| Intrinsic | Signature | Rust Implementation |
| --- | --- | --- |
| `counter_total` | `(counter: str) -> int` | Parse JSON HashMap, sum values |
| `counter_values` | `(counter: str) -> list[int]` | Parse JSON HashMap, return values as Vec |
| `counter_keys` | `(counter: str) -> list[str]` | Parse JSON HashMap, return keys as Vec |
| `counter_items` | `(counter: str) -> str` | Parse JSON HashMap, return JSON array of [key, count] pairs |
| `counter_increment` | `(counter: str, key: str) -> str` | Parse JSON HashMap, increment key, re-encode |

**Existing intrinsics reused (no changes needed):**
- `counter_from_list(items: list[str]) -> str`
- `counter_get(counter: str, key: str) -> int`
- `counter_most_common(counter: str, n: int) -> str`

---

### 2.2 Non-Functional Requirements

| ID | Requirement |
| --- | --- |
| NFR-1 | No performance regression on existing E2E test suite |
| NFR-2 | Counter class compiles to efficient Rust struct + impl (no heap allocation beyond the JSON string) |
| NFR-3 | The pattern established here must be replicable for future stdlib classes without compiler changes |

---

### 2.3 High-Level Architecture

```
lib/sifr/collections.sifr (defines Counter class)
    ↓ imports from
_sifr.collections intrinsics (Rust-backed HashMap operations)
    ↓ compiled by
sifr_driver compile_stdlib() (two-phase compilation)
    ↓ exports via
ExternalDefs.classes["sifr.collections"]["Counter"]
    ↓ imported by
User .sifr code: `from sifr.collections import Counter`
    ↓ compiled to
Rust: `use sifr_collections::Counter;` + method calls
```

**Key pipeline stages:**

1. **HIR lowering** (`lower.rs`): The `Counter` class in `collections.sifr` is lowered to `HirClass` with fields, methods, and receiver inference (`&self` vs `&mut self`).
2. **Driver export** (`driver/lib.rs`): `compile_stdlib()` already iterates `result.module.classes` and inserts into `ExternalDefs.classes`. The `has_pure_sifr_code` check already includes `!result.module.classes.is_empty()` (fixed in `milestone_stdlib_polish`).
3. **User import resolution** (`lower.rs`): When user code does `from sifr.collections import Counter`, the lowering phase looks up `externals.classes["sifr.collections"]["Counter"]` and registers the class type + constructor.
4. **Codegen** (`codegen/lib.rs`): The `Counter` struct and impl block are emitted with `pub` visibility (via `pub_mode`). User code references the struct and calls methods.

---

### 2.4 Detailed Component Design

**📦 `_sifr.collections` intrinsics (stdlib.rs + codegen)**

New intrinsic type signatures in `sifr_hir/src/stdlib.rs`:
- `counter_total(counter: str) -> int`
- `counter_values(counter: str) -> list[int]`
- `counter_keys(counter: str) -> list[str]`
- `counter_items(counter: str) -> str`
- `counter_increment(counter: str, key: str) -> str`

New codegen in `sifr_codegen/src/lib.rs` for each intrinsic:
- All use the existing JSON-encoded HashMap pattern (parse with `serde_json`, operate, re-encode)
- `counter_increment` returns a new JSON string (immutable-style, since Sifr strings are owned)

**📦 `lib/sifr/collections.sifr` (stdlib module)**

- Add `Counter` class definition with `__init__`, `get`, `most_common`, `total`, `values`, `keys`, `items`, `increment` methods
- Add `from_list` factory function
- Keep existing procedural functions (`new_set`, `set_from_list`, etc.) unchanged for backward compatibility

**📦 Driver pipeline verification**

The driver already handles class exports. The key verification is that:
1. `compile_stdlib()` correctly populates `ExternalDefs.classes["sifr.collections"]["Counter"]` with the right `Type::Class` variant
2. The `StdlibCode` for `sifr.collections` includes the compiled Rust struct + impl block
3. User code can reference `Counter` as a type and call its methods

---

### 2.5 Files to Change

| File | Change |
| --- | --- |
| `crates/sifr_hir/src/stdlib.rs` | Add 5 new intrinsic signatures to `intrinsic_collections()` |
| `crates/sifr_codegen/src/lib.rs` | Add codegen for 5 new `_sifr.collections` intrinsics |
| `lib/sifr/collections.sifr` | Add `Counter` class definition + `from_list` factory function |
| `crates/sifr/tests/e2e/pass/stdlib_collections_counter.sifr` | New: basic Counter construction + method calls |
| `crates/sifr/tests/e2e/pass/stdlib_collections_counter_mutate.sifr` | New: Counter mutation via `increment` |
| `crates/sifr/tests/e2e/fail/stdlib_counter_wrong_type.sifr` | New: wrong argument type to Counter |
| `demos/milestone_stdlib_classes_demo.sifr` | New: demo showcasing Counter class usage |
| `audits/STDLIB_PARITY_MASTER_REPORT.md` | Update metrics |

---

### 2.6 Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Class export pipeline has untested edge cases | Medium | High | This milestone is specifically designed to exercise and validate this path |
| Method receiver inference (`&self` vs `&mut self`) incorrect for stdlib classes | Low | Medium | Already proven in user-defined class E2E tests; `increment` tests `&mut self` |
| JSON-encoded HashMap is a performance bottleneck | Low | Low | Acceptable for PoC; future milestones can switch to native dict type |
| Existing procedural Counter tests break | Low | Low | Old functions remain; new class is additive |
| `pub_mode` doesn't apply correctly to class methods | Low | High | Codegen already handles `pub_mode` for struct, impl, and methods (verified in code review) |

---

### 2.7 Trade-offs & Alternatives

| Option Considered | Pros | Cons | Rationale for Final Choice |
| --- | --- | --- | --- |
| **A: Counter as PoC** (chosen) | Existing intrinsics, no Callable-in-struct needed, exercises full pipeline, well-known API | JSON-encoded internal state is not ideal long-term | Best PoC: minimal risk, maximum pipeline coverage |
| B: Path as PoC | High-value class, many modules need it | Needs operator overloading (`/` for path join), more complex | Too many unknowns for a first PoC |
| C: deque as PoC | Pure data structure, no intrinsic dependency | Needs generics, less pipeline coverage | Doesn't test intrinsic delegation pattern |
| D: Fix `Callable`-as-struct-field first | Unblocks defaultdict, ArgumentParser, Timer | Codegen change is orthogonal to stdlib class pipeline | Can be done in parallel or after; Counter doesn't need it |

---

### 2.8 Testing Strategy

| **AC-ID** | Test Layer | Happy-Path Check | Non-Happy / Edge Check |
| --- | --- | --- | --- |
| AC-1 | Code review | Counter class has all specified methods | N/A |
| AC-2 | E2E pass | Methods return correct values | N/A |
| AC-3 | E2E pass | `from sifr.collections import Counter` works | N/A |
| AC-4 | E2E pass | (a) construct + get/most_common/total, (b) increment mutates state | Empty counter, single-element counter |
| AC-5 | E2E fail | N/A | Wrong type to constructor triggers compile error |
| AC-6 | E2E pass | Demo compiles and runs | N/A |
| AC-7 | cargo test | All existing 340+ tests pass | N/A |
| AC-8 | Manual | Report updated with new numbers | N/A |

---

### 2.9 Future Work (Unlocked by This Milestone)

Once `Counter` proves the stdlib class pipeline works end-to-end, the following become straightforward:

| Next Class | Module | Additional Compiler Work Needed |
| --- | --- | --- |
| `Path` | `sifr.pathlib` | Operator overloading export from stdlib (already works for user classes) |
| `Match` | `sifr.re` | None — wraps existing regex intrinsics |
| `NamedTemporaryFile` | `sifr.tempfile` | Context manager (`with` statement) for auto-cleanup |
| `Logger` | `sifr.logging` | None — wraps existing logging intrinsics |
| `ArgumentParser` | `sifr.argparse` | `Callable`-as-struct-field fix (`Box<dyn Fn>`) |
| `defaultdict` | `sifr.collections` | `Callable`-as-struct-field fix (`Box<dyn Fn>`) |
| `DictReader`/`DictWriter` | `sifr.csv` | Iterator protocol |
| `TopologicalSorter` | `sifr.graphlib` | None — pure algorithmic class |
| `datetime`/`timedelta` | `sifr.datetime` | Operator overloading for arithmetic |
