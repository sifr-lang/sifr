# Sifr Compiler: Milestone Reorder & Gap Analysis

**Date:** 2026-02-14
**Scope:** `milestone_ergonomics` onward (first 3 milestones completed)
**Current plan version:** 22 milestones across 5 phases

---

## Part 1: What the Plan Got Right

The phased structure already addresses the most critical sequencing issues:

1. **`milestone_classes` before `milestone_error_handling`** — correct. Typed error hierarchies need classes.
2. **`milestone_ergonomics` separates concrete-return methods from Option-returning methods** — correct. Concrete methods in `milestone_ergonomics`, `Option`-returning methods in `milestone_safe_indexing`.
3. **`milestone_error_handling` before `milestone_safe_indexing`** — correct. `?` and `match` must exist before indexing returns `Option`.
4. **Integer overflow uses Rust defaults** (panic debug / wrap release) instead of `Result` wrapping — correct. Avoids making every arithmetic expression require error handling.
5. **`milestone_decorators` before `milestone_web_db`** — correct. Resolves the decorator dependency for `@app.get("/")`.
6. **`milestone_package_mgmt` split from `milestone_imports`** — correct. Package infra deferred to when it's actually needed.
7. **Comprehensions merged into `milestone_generics`** — correct. They're trivial iterator sugar.
8. **`milestone_decorators` now includes `*args`/`**kwargs`** — correct. Generic decorators need variadics to wrap arbitrary-signature functions.
9. **Previously missing features now included** — augmented assignment, `pass`, chained comparisons, string multiplication, star unpacking, walrus operator, `for`/`while`...`else`, tuple slicing, `del` statement, power operator codegen, multiple return values.

---

## Part 2: Remaining Gaps

### 2.1 `milestone_ergonomics` is Too Large

`milestone_ergonomics` bundles 20+ distinct features:

| Feature | Complexity | Notes |
|---|---|---|
| Augmented assignment (`+=` etc.) | Low | Simple desugaring |
| Conditional expressions (ternary) | Low | Expression-level if/else |
| Keyword arguments + defaults | **Medium-High** | Call-site resolution, default insertion, keyword-only enforcement |
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
| Walrus operator (`:=`) | **Medium-High** | Expression-level assignment, scope changes |
| Power operator codegen | Low | |
| Multiple return values | Low | Already works via tuples |
| `for`/`while`...`else` | Medium | Boolean flag codegen |

**Risk:** This is 3-4 weeks of work in one milestone. If any feature blocks, the entire milestone stalls.

**Recommendation:** Split into two sub-milestones:

- **`milestone_ergonomics` part 1 (Quick Wins):** Augmented assignment, ternary, `pass`, string multiply, power operator codegen, multiple return values, for-loop borrow, list slice copy, `for`/`while`...`else` — ~10 features, all low complexity, deliverable in days
- **`milestone_ergonomics` part 2 (Methods + Call Ergonomics):** Keyword args, defaults, keyword-only params, negative indexing, step slicing, tuple slicing, UTF-8 string fixes, all method suites, built-in functions, chained comparisons, star unpacking, walrus operator

This lets quick wins land fast and make the language immediately more usable, while heavier features get proper time.

### 2.2 `milestone_error_handling` Has a Subtle Bootstrapping Problem

`milestone_error_handling` defines typed error hierarchies using class inheritance syntax:

```python
class AppError(Error):
    message: str
class ValueError(AppError):
    pass
```

But `milestone_classes` only provides flat classes — no inheritance. Inheritance is in `milestone_inheritance`. The plan says `milestone_error_handling` uses "classes that implement an Error protocol," but:

- **Protocols** don't exist until `milestone_protocols`.
- **Inheritance** doesn't exist until `milestone_inheritance`.
- The codegen generates error types as Rust enums (`enum AppError { ValueError(ValueError), IOError(IOError) }`), which works, but the Sifr-level syntax `class ValueError(AppError)` looks like inheritance.

**Gap:** The plan needs to clarify how error type hierarchies work with only `milestone_classes`'s flat classes. Options:

1. **Error types are special-cased** — the compiler recognizes `class Foo(Error)` as "this is an error type" without full inheritance. The `(Error)` is a marker, not real inheritance. Simplest path.
2. **Move minimal inheritance into `milestone_classes`** — just enough for `class Child(Parent)` with field inheritance. Bloats `milestone_classes`.
3. **Error types use a different syntax** — e.g., a dedicated `error ValueError: message: str` construct. Cleaner but diverges from Python.

**Recommendation:** Option 1. Document that `class Foo(Error)` in `milestone_error_handling` is a special-cased error declaration, not general inheritance. Full inheritance semantics come in `milestone_inheritance`.

### 2.3 `milestone_protocols` Depends on `milestone_generics` for Full Usefulness

`milestone_protocols` introduces protocols (traits), but protocols without generics are limited. You can define `Protocol Printable: def __str__(self) -> str`, but you can't write `def process[T: Printable](items: list[T])` until `milestone_generics`.

This means `milestone_protocols`'s protocols are only usable via dynamic dispatch (`Box<dyn Trait>`) until `milestone_generics` adds generic bounds. Most real-world protocol usage is via generic bounds, not trait objects.

**Not a blocker** — protocols still enable operator overloading and discriminated unions. But the plan should note that protocol-as-bound is a `milestone_generics` feature, and `milestone_protocols` protocols are primarily for:
- Operator overloading (`__add__`, `__eq__`, etc.)
- Discriminated union tag narrowing
- Dynamic dispatch (`fn process(x: Printable)` → `Box<dyn Trait>`)

### 2.4 `milestone_imports` Can Move Earlier

`milestone_imports` is positioned after `milestone_protocols` in the dependency graph (`milestone_protocols → milestone_imports → milestone_generics`). But multi-file compilation doesn't technically need protocols — it needs classes (`milestone_classes`) for importing class definitions across files, and error handling (`milestone_error_handling`) for importing error types.

**Gap:** The dependency `milestone_protocols → milestone_imports` is too conservative. The rationale says "Protocols (traits) are needed for meaningful multi-file programs," but you can write useful multi-file programs with just classes and functions.

**Recommendation:** Consider `milestone_safe_indexing → milestone_imports` instead of `milestone_protocols → milestone_imports`. This lets multi-file compilation land earlier, which benefits all subsequent milestones (stdlib, test runner, etc.). `milestone_protocols` and `milestone_inheritance` can then be done after or in parallel with `milestone_imports`.

### 2.5 `milestone_decorators` Can Move Earlier

`milestone_decorators` is positioned after `milestone_async` (`milestone_async → milestone_decorators → milestone_web_db`). But decorators are fundamentally higher-order functions — a decorator takes a function and returns a function. This requires closures/lambdas from `milestone_generics`. Decorators do NOT need async.

The `*args`/`**kwargs` part also doesn't need async — it needs the type system to handle variadic types, which is a `milestone_generics`-level feature.

**Gap:** `milestone_decorators` could move right after `milestone_generics`. This would make decorators available for `milestone_core_stdlib` and `milestone_test_runner`, not just `milestone_web_db`.

**Recommendation:** Change the dependency from `milestone_async → milestone_decorators` to `milestone_generics → milestone_decorators`. The chain becomes: `milestone_generics → milestone_decorators → milestone_web_db`. `milestone_async` remains a dependency of `milestone_web_db` but not of `milestone_decorators`.

### 2.6 `milestone_data_processing` Doesn't Need `milestone_web_db`

The dependency graph shows `milestone_web_db → milestone_data_processing` and `milestone_generics → milestone_data_processing`. But polars DataFrames don't need a web framework — they need generics (`milestone_generics`) and I/O (`milestone_core_stdlib`).

**Recommendation:** Remove `milestone_web_db → milestone_data_processing`. Keep `milestone_generics → milestone_data_processing` and add `milestone_core_stdlib → milestone_data_processing`. This lets data processing work happen in parallel with web development.

### 2.7 `milestone_inheritance` Parallelism

`milestone_inheritance` depends on `milestone_protocols` (inheritance via trait delegation) but doesn't depend on `milestone_imports` or `milestone_generics`. The dependency graph shows `milestone_protocols → milestone_inheritance` as a side branch, which is correct.

This means `milestone_inheritance` can be done **in parallel** with `milestone_imports` and `milestone_generics`. The plan should explicitly note this parallelism opportunity.

### 2.8 `milestone_generators` `with` Statement Needs `milestone_protocols`

`milestone_generators` defines the `ContextManager` protocol for the `with` statement. But protocols are introduced in `milestone_protocols`. The dependency chain `milestone_generics → milestone_generators` skips `milestone_protocols`, but `milestone_generators` actually needs `milestone_protocols`'s protocol infrastructure.

Looking at the graph: `milestone_protocols → milestone_imports → milestone_generics → milestone_generators`. So `milestone_protocols` IS a transitive dependency. This is fine — but it should be explicitly noted that `with` requires the protocol system from `milestone_protocols`.

### 2.9 Missing: `not in` Operator

The plan mentions `in` operator (`milestone_control_flow`) but never explicitly mentions `not in`. This is a common Python idiom (`if key not in dict:`). It likely works via the parser fork, but should be explicitly tested.

### 2.10 Missing: Multiline String Literals

Triple-quoted strings (`"""..."""` and `'''...'''`) for multiline strings are not mentioned. The parser fork from ruff handles them, but codegen should be verified.

### 2.11 Missing: `== None` Diagnostic

The plan covers `is None` and `is not None` for narrowing. But `== None` (which Python allows but linters discourage) should be explicitly rejected or handled. The compiler should emit a diagnostic: "use `is None` instead of `== None`".

### 2.12 Missing: Empty Collections Type Inference

What type does `x = []` infer? Or `x = {}`? The plan doesn't specify. Options:
- Compile error: "cannot infer element type for empty collection"
- Infer from first usage (complex)
- Require annotation: `x: list[int] = []`

**Recommendation:** Require annotation for empty collections. Simplest and most explicit.

### 2.13 Missing: `str()` on Collections

`str([1, 2, 3])` should produce `"[1, 2, 3]"`. `print([1, 2, 3])` should work. The plan mentions `str(x)` for "any type" in `milestone_error_handling` but doesn't specify how collections format. This needs `Debug` or `Display` trait derivation on generated types.

### 2.14 Cross-cutting Contract #6 Contradicts `milestone_ergonomics`

Cross-cutting contract #6 (Slice and Collection Semantics) says "Dict/tuple: not sliceable." But `milestone_ergonomics` adds tuple slicing. The contract needs updating.

---

## Part 3: Dependency Graph Analysis

### Current Dependency Graph (from plan)

```
milestone_type_system → milestone_ergonomics → milestone_classes → milestone_error_handling → milestone_safe_indexing
milestone_safe_indexing → milestone_protocols → milestone_imports → milestone_generics
milestone_protocols → milestone_inheritance (branch)
milestone_generics → milestone_generators → milestone_core_stdlib → milestone_test_runner
milestone_core_stdlib → milestone_ext_collections → milestone_ext_stdlib
milestone_core_stdlib → milestone_ext_stdlib
milestone_ext_stdlib → milestone_async → milestone_decorators → milestone_web_db
milestone_test_runner → milestone_async
milestone_web_db → milestone_data_processing
milestone_generics → milestone_data_processing
milestone_web_db → milestone_metaprogramming → milestone_ffi → milestone_package_mgmt → milestone_dev_tooling → milestone_ecosystem
```

### True Minimum Dependencies (What Actually Blocks What)

```
milestone_ergonomics     needs: milestone_type_system (unions for type checking methods)
milestone_classes        needs: milestone_ergonomics (kwargs for __init__, method infra)
milestone_error_handling needs: milestone_classes (classes for error types), milestone_type_system (unions for Result/Option)
milestone_safe_indexing  needs: milestone_error_handling (Option/Result must be handleable)
milestone_protocols      needs: milestone_classes (classes for protocols), milestone_safe_indexing (Option for pattern matching)
milestone_inheritance    needs: milestone_protocols (protocols for trait delegation)
milestone_imports        needs: milestone_classes (classes to import), milestone_error_handling (errors across modules)
milestone_generics       needs: milestone_protocols (protocols for type bounds)
milestone_generators     needs: milestone_generics (iterators for generators), milestone_protocols (ContextManager protocol)
milestone_core_stdlib    needs: milestone_imports (imports for stdlib modules), milestone_generators (with statement for files)
milestone_test_runner    needs: milestone_core_stdlib (I/O for test discovery)
milestone_ext_collections needs: milestone_core_stdlib (stdlib pattern)
milestone_ext_stdlib     needs: milestone_core_stdlib (stdlib pattern)
milestone_async          needs: milestone_generics (closures for async), milestone_ext_stdlib (stdlib for networking)
milestone_decorators     needs: milestone_generics (closures for decorator wrapping)
milestone_web_db         needs: milestone_async (async for web), milestone_decorators (decorators for routing)
milestone_data_processing needs: milestone_generics (generics for DataFrames), milestone_core_stdlib (I/O for CSV)
milestone_metaprogramming needs: milestone_decorators (extends decorators), milestone_classes (classes for @dataclass)
milestone_ffi            needs: milestone_imports (imports for extern crate)
milestone_package_mgmt   needs: milestone_imports (module system for packages)
milestone_dev_tooling    needs: milestone_imports (multi-file for LSP)
milestone_ecosystem      needs: milestone_package_mgmt, milestone_dev_tooling
```

### Key Insight: `milestone_imports` Can Move Earlier

`milestone_imports` only needs `milestone_classes` and `milestone_error_handling`. It does NOT need `milestone_safe_indexing` or `milestone_protocols`. Moving it after `milestone_safe_indexing` (or even after `milestone_error_handling`) would:

- Let `milestone_core_stdlib` start sooner
- Let `milestone_test_runner` start sooner
- Enable dogfooding earlier

### Key Insight: `milestone_decorators` Can Move Much Earlier

`milestone_decorators` only needs `milestone_generics` (closures for wrapping, type system for variadics). It does NOT need `milestone_async`. Moving it right after `milestone_generics` means:

- Decorators available for stdlib modules
- `@property` (currently `milestone_inheritance`) could use decorator infrastructure
- `milestone_web_db` still works since `milestone_async → milestone_web_db` is the async dependency

---

## Part 4: Proposed Optimizations

### 4.1 Split `milestone_ergonomics` into Two Parts

**Part 1 (Quick Wins — ~1 week):**
- Augmented assignment (`+=`, `-=`, etc.)
- Conditional expressions (ternary)
- `pass` statement
- String multiplication (`"abc" * 3`)
- Power operator codegen (`**`)
- Multiple return values (tuple packing)
- For-loop borrow semantics
- List slice copy semantics
- `for`/`while`...`else` clauses

**Part 2 (Methods + Call Ergonomics — ~2-3 weeks):**
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

### 4.2 Move `milestone_imports` Earlier

Change: `milestone_protocols → milestone_imports` to `milestone_safe_indexing → milestone_imports` (parallel with `milestone_protocols`)

`milestone_imports` doesn't need protocols. It needs classes (`milestone_classes`) and error handling (`milestone_error_handling`) for cross-module imports. Moving it after `milestone_safe_indexing`:

```
milestone_safe_indexing → milestone_imports → milestone_generics (imports still before generics)
milestone_safe_indexing → milestone_protocols → milestone_generics (protocols still before generics)
milestone_protocols → milestone_inheritance (unchanged)
```

This means `milestone_imports` and `milestone_protocols`/`milestone_inheritance` can develop **in parallel**.

### 4.3 Move `milestone_decorators` Earlier

Change: `milestone_async → milestone_decorators` to `milestone_generics → milestone_decorators`

`milestone_decorators` needs closures (`milestone_generics`), not async (`milestone_async`). Moving it:

```
milestone_generics → milestone_decorators (decorators available earlier)
milestone_generics → milestone_generators (generators continue)
milestone_decorators → milestone_web_db (web still works)
milestone_async → milestone_web_db (async still required for web)
```

### 4.4 Remove `milestone_web_db → milestone_data_processing`

`milestone_data_processing` needs `milestone_generics` and `milestone_core_stdlib`, not `milestone_web_db`. Polars DataFrames have nothing to do with web frameworks.

```
milestone_generics → milestone_data_processing
milestone_core_stdlib → milestone_data_processing
```

This lets data processing develop in parallel with `milestone_async`/`milestone_web_db`.

### 4.5 Parallelize `milestone_ext_collections`/`milestone_ext_stdlib` with `milestone_test_runner`

All three depend on `milestone_core_stdlib` but not on each other. They can develop in parallel:

```
milestone_core_stdlib → milestone_test_runner
milestone_core_stdlib → milestone_ext_collections  } parallel
milestone_core_stdlib → milestone_ext_stdlib        }
```

The plan already notes this for `milestone_ext_stdlib` but should make it explicit for `milestone_ext_collections` too.

---

## Part 5: Proposed Optimized Sequence

### Linear Critical Path (what must be sequential)

```
milestone_type_system → milestone_ergonomics → milestone_classes → milestone_error_handling → milestone_safe_indexing
→ milestone_imports → milestone_generics → milestone_generators → milestone_core_stdlib
→ milestone_test_runner → milestone_async → milestone_web_db
```

### Parallel Tracks

```
After milestone_safe_indexing:
  TRACK A: milestone_imports (parallel with Track B)
  TRACK B: milestone_protocols → milestone_inheritance

After milestone_protocols + milestone_imports:
  milestone_generics (needs both)

After milestone_generics:
  TRACK C: milestone_generators → milestone_core_stdlib → ...
  TRACK D: milestone_decorators (parallel with Track C)
  TRACK E: milestone_data_processing (needs milestone_generics + milestone_core_stdlib)

After milestone_core_stdlib:
  TRACK F: milestone_test_runner
  TRACK G: milestone_ext_collections  } all parallel
  TRACK H: milestone_ext_stdlib       }

After milestone_ext_stdlib + milestone_test_runner:
  milestone_async → milestone_web_db (also needs milestone_decorators from Track D)

After milestone_web_db:
  milestone_metaprogramming → milestone_ffi

After milestone_imports:
  TRACK I: milestone_package_mgmt (can start early)
  TRACK J: milestone_dev_tooling (can start early)

After milestone_package_mgmt + milestone_dev_tooling:
  milestone_ecosystem
```

### Visual Diagram

```
                    milestone_type_system (done)
                              │
                    milestone_ergonomics
                              │
                      milestone_classes
                              │
                   milestone_error_handling
                              │
                    milestone_safe_indexing
                         ╱          ╲
              milestone_imports    milestone_protocols
                         ╲          ╱        ╲
                    milestone_generics    milestone_inheritance
                     ╱      │       ╲
    milestone_decorators    │    milestone_data_processing
           │        milestone_generators
           │                │
           │       milestone_core_stdlib
           │        ╱       │       ╲
           │  milestone_   milestone_  milestone_
           │  test_runner  ext_coll.   ext_stdlib
           │       ╲                    ╱
           │        milestone_async ◄──╯
           │                │
           ╰──────► milestone_web_db
                            │
                   milestone_metaprogramming
                            │
                      milestone_ffi
                            │
                   milestone_package_mgmt
                            │
                    milestone_dev_tooling
                            │
                    milestone_ecosystem
```

### Net Effect of Proposed Changes

| Change | Effect |
|---|---|
| Split `milestone_ergonomics` | Quick wins land in ~1 week instead of ~3-4 weeks |
| Move `milestone_imports` earlier | Stdlib and test runner unblocked sooner; `milestone_imports` and `milestone_protocols` develop in parallel |
| Move `milestone_decorators` earlier | Decorators available ~4 milestones sooner; usable in stdlib, not just web |
| Remove `milestone_web_db → milestone_data_processing` | Data processing not blocked by web; can develop in parallel |

---

## Part 6: Risk Assessment

### High Risk: `milestone_classes` Class Codegen

Generating correct Rust structs with method receiver inference (`&self` vs `&mut self` vs `self`) from body analysis is the hardest single feature in Phase 1. Edge cases around mutable field access, method chaining, and self-consumption will require careful testing.

**Mitigation:** Start with `&self` for all methods. Add `&mut self` inference as a follow-up within `milestone_classes`. Defer `self` (move) receiver to `milestone_protocols` or later.

### Medium Risk: `milestone_error_handling` Error Type Bootstrapping

As noted in 2.2, error types need some form of type hierarchy before inheritance (`milestone_inheritance`) exists. The special-casing approach works but needs clear documentation so implementers don't try to build full inheritance.

### Medium Risk: `milestone_safe_indexing` Ergonomics

Making `list[i]` return `Option[T]` is the biggest UX change in the language. Every existing program that does `x = items[0]` will need `x = items[0]?` or `x = items[0].unwrap()`. This will feel hostile to Python developers.

**Mitigation:** Provide excellent error messages. Ship `.unwrap_or(default)` and `.expect("msg")` as day-one methods on Option. Consider a `--unchecked` flag for prototyping that auto-unwraps Options with panics.

### Low Risk: `milestone_ergonomics` Size

Even split into two parts, part 2 is still large. But the features are independent — each method/builtin can be implemented and tested in isolation. The risk is timeline, not correctness.

---

## Part 7: Summary of Recommendations

| # | Change | Impact | Effort |
|---|---|---|---|
| 1 | Split `milestone_ergonomics` into quick wins + methods/calls | Faster first deliverable | Minimal (scoping only) |
| 2 | Move `milestone_imports` after `milestone_safe_indexing` (not `milestone_protocols`) | Stdlib and test runner land sooner | Minimal (remove one dep) |
| 3 | Move `milestone_decorators` after `milestone_generics` (not `milestone_async`) | Decorators available ~4 milestones sooner | Minimal (remove one dep) |
| 4 | Remove `milestone_web_db → milestone_data_processing` dependency | Data processing not blocked by web | Minimal (remove one dep) |
| 5 | Clarify `milestone_error_handling` error type bootstrapping | Avoid confusion during implementation | Documentation only |
| 6 | Update cross-cutting contract #6 for tuple slicing | Contract contradicts `milestone_ergonomics` | Documentation only |
| 7 | Add empty collection inference rule | Prevent ambiguous type errors | Small design decision |
| 8 | Add `not in` operator to test suite | Verify it works | Trivial |
| 9 | Add `== None` diagnostic | Guide users to `is None` | Small |
| 10 | Note `milestone_protocols` limitations pre-`milestone_generics` | Set expectations for protocol usage | Documentation only |
