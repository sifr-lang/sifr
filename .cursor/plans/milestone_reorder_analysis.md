# Sifr Compiler: Milestone Reorder & Gap Analysis

**Date:** 2026-02-14
**Scope:** M4 onward (M1, M2, M3 completed)
**Current plan version:** 25 milestones across 5 phases

---

## Part 1: What the Updated Plan Got Right

The restructured plan (M4-M25) already addresses several major issues:

1. **M5 (Basic Classes) before M6 (Error Handling)** — correct. Typed error hierarchies need classes.
2. **M4 (Language Ergonomics) separated from safe indexing** — correct. Concrete-return methods land in M4, `Option`-returning methods in M7.
3. **M6 before M7** — correct. `?` and `match` must exist before indexing returns `Option`.
4. **Integer overflow uses Rust defaults** (panic debug / wrap release) instead of `Result` wrapping — correct. Avoids making every arithmetic expression require error handling.
5. **M18 (Basic Decorators) before M19 (Web)** — correct. Resolves the decorator dependency for `@app.get("/")`.
6. **M23 (Package Management) split from M10 (Imports)** — correct. Package infra deferred to when it's actually needed.
7. **Comprehensions merged into M11** — correct. They're trivial iterator sugar.
8. **Missing features from prior plan now included** — augmented assignment, `pass`, chained comparisons, string multiplication, star unpacking, walrus operator, `for`/`while`...`else`, tuple slicing, `del` statement, power operator codegen, multiple return values.

The phased structure (Foundations → Type Power → Pythonic → Ecosystem → Polish) is sound.

---

## Part 2: Remaining Gaps

### 2.1 M4 is Still Too Large

M4 ("Language Ergonomics") bundles 20+ distinct features:

| Feature | Complexity | Notes |
|---|---|---|
| Augmented assignment (`+=` etc.) | Low | Simple desugaring |
| Conditional expressions (ternary) | Low | Expression-level if/else |
| Keyword arguments + defaults | **Medium-High** | Requires call-site resolution, default value insertion, keyword-only enforcement |
| For-loop borrow semantics | Low | Codegen change |
| List slice copy | Low | Already partially works |
| Negative indexing | Medium | Runtime index resolution |
| Step slicing | Medium | Complex iterator logic |
| Tuple slicing | Medium | Compile-time index resolution |
| UTF-8 string indexing fixes | Medium | Changes existing codegen |
| 9 list methods | Medium | Each needs type checking + codegen |
| 8 dict methods | Medium | Same |
| 15+ string methods | Medium | Same |
| Tuple methods | Low | |
| Built-in functions (len, abs, round, repr) | Low-Medium | |
| Chained comparisons | Medium | Desugaring with temp variables |
| String multiplication | Low | |
| `pass` statement | Trivial | |
| Star unpacking | Medium | Slice operations on Vec |
| Walrus operator (`:=`) | **Medium-High** | Requires expression-level assignment, scope changes |
| Power operator codegen | Low | |
| Multiple return values | Low | Already works via tuples |
| `for`/`while`...`else` | Medium | Boolean flag codegen |

**Risk:** This is 3-4 weeks of work presented as one milestone. If any feature blocks, the entire milestone stalls.

**Recommendation:** Split M4 into two sub-milestones:

- **M4a (Quick Wins):** Augmented assignment, ternary, `pass`, string multiply, power operator codegen, multiple return values, for-loop borrow, list slice copy, `for`/`while`...`else` — ~10 features, all low complexity, can be done in a few days
- **M4b (Call Ergonomics + Methods):** Keyword args, defaults, keyword-only params, negative indexing, step slicing, tuple slicing, UTF-8 string fixes, all method suites, built-in functions, chained comparisons, star unpacking, walrus operator — the heavier features

This lets M4a land fast and make the language immediately more usable, while M4b can take the time it needs.

### 2.2 M6 Error Handling Has a Subtle Bootstrapping Problem

M6 defines typed error hierarchies using single-level class inheritance:

```python
class AppError(Error):
    message: str
class ValueError(AppError):
    pass
```

But M5 (Basic Classes) only provides flat classes — no inheritance. Inheritance is in M9. The plan says M6 uses "classes that implement an Error protocol," but:

- **Protocols** don't exist until M8.
- **Inheritance** doesn't exist until M9.
- M6 generates error types as Rust enums (`enum AppError { ValueError(ValueError), IOError(IOError) }`), which is fine for codegen, but the Sifr-level syntax `class ValueError(AppError)` looks like inheritance.

**Gap:** The plan needs to clarify how error type hierarchies work with only M5's flat classes. Options:

1. **Error types are special-cased in M6** — the compiler recognizes `class Foo(Error)` as "this is an error type" without full inheritance. The `(Error)` is a marker, not real inheritance. This is the simplest path.
2. **Move minimal inheritance into M5** — just enough for `class Child(Parent)` with field inheritance. But this bloats M5.
3. **Error types use a different syntax** — e.g., `error ValueError: message: str` as a dedicated construct. Cleaner but diverges from Python.

**Recommendation:** Option 1. Document that `class Foo(Error)` in M6 is a special-cased error declaration, not general inheritance. The compiler treats `Error` as a built-in marker base. Full inheritance semantics come in M9.

### 2.3 M8 Protocols Depend on M11 Generics for Full Usefulness

M8 introduces protocols (traits), but protocols without generics are limited. You can define `Protocol Printable: def __str__(self) -> str`, but you can't write `def process[T: Printable](items: list[T])` until M11.

This means M8's protocols are only usable via dynamic dispatch (`Box<dyn Trait>`) until M11 adds generic bounds. That's a significant limitation — most protocol usage in real code is via generic bounds, not trait objects.

**Not a blocker** — protocols still enable operator overloading and discriminated unions in M8. But the plan should note that protocol-as-bound is an M11 feature, and M8 protocols are primarily for:
- Operator overloading (`__add__`, `__eq__`, etc.)
- Discriminated union tag narrowing
- Dynamic dispatch (`fn process(x: Printable)` → `Box<dyn Trait>`)

### 2.4 M9 Inheritance Timing

M9 (Inheritance) depends on M8 (Protocols) because inheritance is implemented via trait delegation. But M9 doesn't depend on M10 (Imports) or M11 (Generics). The dependency graph shows `M8 → M9` as a side branch, which is correct.

However, the plan shows `M8 → M10 → M11` as the main path, with M9 branching off M8. This means M9 could be done **in parallel** with M10 or M11. The plan should explicitly note this parallelism opportunity.

### 2.5 M10 Imports Need M5 Classes But Not M8 Protocols

M10 (Multi-file + Imports) is positioned after M8 (Protocols). But multi-file compilation doesn't technically need protocols — it needs classes (M5) for importing class definitions across files, and error handling (M6) for importing error types.

**Gap:** M10 could potentially move earlier, right after M7. The dependency `M8 → M10` in the graph seems too conservative. The rationale says "Protocols (traits) are needed for meaningful multi-file programs," but you can write useful multi-file programs with just classes and functions.

**Recommendation:** Consider `M7 → M10` instead of `M8 → M10`. This lets multi-file compilation land earlier, which benefits all subsequent milestones (testing, stdlib, etc.). M8 and M9 can then be done after M10 if desired.

### 2.6 M12 `with` Statement Needs Protocols from M8

M12 (Generators + With) defines the `ContextManager` protocol. But protocols are introduced in M8. The dependency chain `M11 → M12` skips M8, but M12 actually needs M8's protocol infrastructure.

Looking at the graph more carefully: `M8 → M10 → M11 → M12`. So M8 IS a transitive dependency of M12. This is fine — but it should be explicitly noted that `with` requires the protocol system from M8.

### 2.7 M18 Basic Decorators Need Closures from M11

M18 (Basic Decorators) is positioned after M17 (Async). But decorators are fundamentally higher-order functions — a decorator takes a function and returns a function. This requires closures/lambdas from M11.

The dependency graph shows `M16 → M17 → M18`, but the true dependency is `M11 → M18` (closures needed for decorator wrapping). M18 doesn't need async (M17) at all.

**Gap:** M18 could move earlier — right after M11. This would let decorators be available for M13 (stdlib) and M14 (test runner), not just M19 (web).

**Recommendation:** Move M18 to right after M11 (or parallel with M12). The dependency chain becomes: `M11 → M18 → M19`. This also means `@property` decorators (currently in M9) could use M18's decorator infrastructure.

### 2.8 Missing: `not in` Operator

The plan mentions `in` operator (M2) but never explicitly mentions `not in`. This is a common Python idiom (`if key not in dict:`). It likely works via the parser fork, but should be explicitly tested.

### 2.9 Missing: Multiline String Literals

Triple-quoted strings (`"""..."""` and `'''...'''`) for multiline strings are not mentioned. The parser fork from ruff handles them, but codegen should be verified.

### 2.10 Missing: `None` Comparisons Beyond `is`/`is not`

The plan covers `is None` and `is not None` for narrowing. But `== None` (which Python allows but linters discourage) should be explicitly rejected or handled. The compiler should emit a diagnostic: "use `is None` instead of `== None`".

### 2.11 Missing: Empty Collections Type Inference

What type does `x = []` infer? Or `x = {}`? The plan doesn't specify. Options:
- Compile error: "cannot infer element type for empty collection"
- Infer from first usage (complex)
- Require annotation: `x: list[int] = []`

**Recommendation:** Require annotation for empty collections. This is the simplest and most explicit approach.

### 2.12 Missing: `str()` on Collections

`str([1, 2, 3])` should produce `"[1, 2, 3]"`. `print([1, 2, 3])` should work. The plan mentions `str(x)` for "any type" in M6 but doesn't specify how collections format. This needs `Debug` or `Display` trait derivation on generated types.

### 2.13 Cross-cutting Contract Gap: Tuple Slicing vs Contract #6

Cross-cutting contract #6 says "Dict/tuple: not sliceable." But M4 adds tuple slicing. The contract needs updating.

---

## Part 3: Dependency Graph Analysis

### Current Dependency Graph (from plan)

```
M3 → M4 → M5 → M6 → M7
M7 → M8 → M10 → M11
M8 → M9 (branch)
M11 → M12 → M13 → M14
M13 → M15 → M16
M13 → M16
M16 → M17 → M18 → M19
M14 → M17
M19 → M20
M11 → M20
M19 → M21
M21 → M22 → M23 → M24 → M25
```

### True Minimum Dependencies (What Actually Blocks What)

```
M4  needs: M3 (unions for type checking methods)
M5  needs: M4 (kwargs for __init__, methods need the method infra)
M6  needs: M5 (classes for error types), M3 (unions for Result/Option)
M7  needs: M6 (Option/Result must be handleable)
M8  needs: M5 (classes for protocols), M7 (Option for pattern matching)
M9  needs: M8 (protocols for trait delegation)
M10 needs: M5 (classes to import), M6 (errors across modules)
M11 needs: M8 (protocols for type bounds)
M12 needs: M11 (iterators for generators), M8 (protocols for ContextManager)
M13 needs: M10 (imports for stdlib modules), M12 (with statement for files)
M14 needs: M13 (I/O for test discovery)
M15 needs: M13 (stdlib pattern)
M16 needs: M13 (stdlib pattern)
M17 needs: M11 (closures for async), M16 (stdlib for networking)
M18 needs: M11 (closures for decorator wrapping)
M19 needs: M17 (async for web), M18 (decorators for routing)
M20 needs: M11 (generics for DataFrames), M13 (I/O for CSV)
M21 needs: M18 (extends decorators), M5 (classes for @dataclass)
M22 needs: M10 (imports for extern crate)
M23 needs: M10 (module system for packages)
M24 needs: M10 (multi-file for LSP)
M25 needs: M23 (package management), M24 (tooling)
```

### Key Insight: M10 Can Move Earlier

M10 (imports) only needs M5 (classes) and M6 (errors). It does NOT need M7 (safe indexing) or M8 (protocols). Moving M10 earlier would:

- Let M13 (stdlib) start sooner
- Let M14 (test runner) start sooner
- Enable dogfooding earlier

### Key Insight: M18 Can Move Much Earlier

M18 (basic decorators) only needs M11 (closures). It does NOT need M17 (async). Moving M18 right after M11 means:

- Decorators available for stdlib modules
- `@property` (currently M9) could use decorator infrastructure
- Web framework (M19) still works since M17 → M19 is the async dependency

---

## Part 4: Proposed Optimizations

### 4.1 Split M4 into M4a/M4b

**M4a (Quick Wins — 1 week):**
- Augmented assignment (`+=`, `-=`, etc.)
- Conditional expressions (ternary)
- `pass` statement
- String multiplication (`"abc" * 3`)
- Power operator codegen (`**`)
- Multiple return values (tuple packing)
- For-loop borrow semantics
- List slice copy semantics
- `for`/`while`...`else` clauses

**M4b (Methods + Call Ergonomics — 2-3 weeks):**
- Keyword arguments + defaults + keyword-only params
- Negative indexing
- Step slicing
- Tuple slicing
- UTF-8 string indexing fixes
- All list/dict/string/tuple method suites (concrete returns)
- Built-in functions (len, abs, round, repr)
- Chained comparisons
- Star unpacking
- Walrus operator

### 4.2 Move M10 Earlier

Change: `M7 → M8 → M10` to `M7 → M10` (parallel with M8)

M10 doesn't need protocols. It needs classes (M5) and error handling (M6) for cross-module imports. Moving it after M7 means:

```
M7 → M10 → M13 → M14 (test runner lands sooner)
M7 → M8 → M9 (OOP features continue in parallel)
M8 → M11 (generics still need protocols)
```

### 4.3 Move M18 Earlier

Change: `M17 → M18` to `M11 → M18`

M18 (basic decorators) needs closures (M11), not async (M17). Moving it:

```
M11 → M18 (decorators available earlier)
M11 → M12 (generators continue)
M18 → M19 (web still works)
```

### 4.4 Parallelize M15/M16 with M14

M15 (extended collections) and M16 (extended stdlib) both depend on M13 but not on M14 (test runner). They can be developed in parallel:

```
M13 → M14 (test runner)
M13 → M15 (extended collections)  } parallel
M13 → M16 (extended stdlib)       }
```

The plan already notes this for M16 but not for M15.

### 4.5 M20 Can Parallel M19

M20 (data processing) needs M11 (generics) and M13 (I/O), but NOT M19 (web). The current graph shows `M19 → M20` and `M11 → M20`. The M19 dependency seems unnecessary — polars DataFrames don't need a web framework.

Change: Remove `M19 → M20`. Keep `M11 → M20` and `M13 → M20`.

---

## Part 5: Proposed Optimized Sequence

### Linear Critical Path (what must be sequential)

```
M3 → M4a → M4b → M5 → M6 → M7 → M10 → M11 → M12 → M13 → M14 → M17 → M19
```

### Parallel Tracks (can be developed alongside the critical path)

```
After M7:  M8 → M9 (OOP features, parallel with M10)
After M8:  M11 (needs M8 for type bounds)
After M11: M18 (basic decorators, parallel with M12)
After M13: M15, M16 (parallel with M14)
After M13: M20 (parallel with M17, needs M11 + M13)
After M19: M21 → M22
After M10: M23, M24 (can start early, parallel with ecosystem)
After M23+M24: M25
```

### Visual Sequence

```
CRITICAL PATH:
  M4a → M4b → M5 → M6 → M7 → M10 → M11 → M12 → M13 → M14 → M17 → M19

PARALLEL TRACK A (OOP):
                              M7 → M8 → M9
                                    ↓
                                   M11

PARALLEL TRACK B (Decorators):
                                        M11 → M18 → M19

PARALLEL TRACK C (Stdlib):
                                                    M13 → M15
                                                    M13 → M16 → M17

PARALLEL TRACK D (Data):
                                                    M13 → M20 (also needs M11)

PARALLEL TRACK E (Polish):
                                                              M19 → M21 → M22
                                              M10 → M23
                                              M10 → M24
                                                    M23 + M24 → M25
```

### Net Effect

- **M10 (imports) lands ~2 milestones earlier** — unblocks stdlib and test runner sooner
- **M18 (decorators) lands ~5 milestones earlier** — available for stdlib, not just web
- **M20 (data) no longer blocked by M19** — can develop in parallel
- **M4 split** — quick wins land immediately, heavier features get proper time

---

## Part 6: Risk Assessment

### High Risk: M5 Class Codegen

Generating correct Rust structs with method receiver inference (`&self` vs `&mut self` vs `self`) from body analysis is the hardest single feature in Phase 1. Edge cases around mutable field access, method chaining, and self-consumption will require careful testing.

**Mitigation:** Start with `&self` for all methods. Add `&mut self` inference as a follow-up within M5. Defer `self` (move) receiver to M8 or later.

### Medium Risk: M6 Error Type Bootstrapping

As noted in 2.2, error types need some form of type hierarchy before inheritance (M9) exists. The special-casing approach works but needs clear documentation.

### Medium Risk: M7 Safe Indexing Ergonomics

Making `list[i]` return `Option[T]` is the biggest UX change in the language. Every existing program that does `x = items[0]` will need `x = items[0]?` or `x = items[0].unwrap()`. This will feel hostile to Python developers.

**Mitigation:** Provide excellent error messages. Consider `.unwrap_or(default)` and `.expect("msg")` as day-one methods on Option. Consider a `--unchecked` flag for prototyping that auto-unwraps Options with panics.

### Low Risk: M4 Size

Even split into M4a/M4b, M4b is still large. But the features are independent — each method/builtin can be implemented and tested in isolation. The risk is timeline, not correctness.

---

## Part 7: Summary of Recommendations

| # | Change | Impact | Effort |
|---|---|---|---|
| 1 | Split M4 into M4a (quick wins) + M4b (methods/calls) | Faster first deliverable | Minimal (just scoping) |
| 2 | Move M10 after M7 (not M8) | Stdlib and test runner land sooner | Minimal (remove one dep) |
| 3 | Move M18 after M11 (not M17) | Decorators available for stdlib | Minimal (remove one dep) |
| 4 | Remove M19 → M20 dependency | Data processing not blocked by web | Minimal (remove one dep) |
| 5 | Clarify M6 error type bootstrapping | Avoid confusion during implementation | Documentation only |
| 6 | Update cross-cutting contract #6 | Tuple slicing now exists in M4 | Documentation only |
| 7 | Add empty collection inference rule | Prevent ambiguous type errors | Small design decision |
| 8 | Add `not in` operator to test suite | Verify it works | Trivial |
| 9 | Add `== None` diagnostic | Guide users to `is None` | Small |
| 10 | Note M8 protocol limitations pre-M11 | Set expectations | Documentation only |
