# Sifr Compiler Plan — Recheck Analysis (Post-Update #2)

**Date:** 2026-02-14
**Plan reviewed:** `sifr_compiler_architecture_fa3c10ee.plan.md` (3546 lines)
**Scope:** milestone_ergonomics onward (all unstarted milestones)
**Constraint:** Strictly sequential execution — no parallel milestones.

---

## What Was Fixed Since Last Recheck

Two of the previous issues have been addressed:

1. **`@staticmethod` in `milestone_protocols` newtype example — FIXED.** The example now uses a module-level factory function `make_port()` (line 1503) and includes an explicit forward-reference note (line 1516): "this example uses a module-level factory function because `@staticmethod` is not available until milestone_inheritance."

2. **Panic-based indexing in `milestone_ergonomics` — CLARIFIED.** A safety staging note (line 767) now explicitly states: "milestones before milestone_safe_indexing use panic-based indexing as a bootstrap mechanism. The global no-panic guarantee is fully enforced from milestone_safe_indexing onward." This resolves the apparent conflict between `milestone_ergonomics` indexing and the global safety contract.

---

## Remaining Issues (Strictly Sequential)

### Issue 1: `.sort()` in `milestone_ergonomics` Requires `Comparable` Protocol (HIGH)

**Location:** Line 837, line 1024

`milestone_ergonomics` includes `.sort()` as a concrete list method:

> `.sort()` -> in-place sort (requires `Comparable` -- basic version for primitive types)

The `Comparable` protocol is defined in `milestone_protocols` (5 milestones later). The parenthetical "basic version for primitive types" implies a hardcoded sort for primitives, but this is never explicitly defined anywhere in the plan.

Meanwhile, `milestone_generics` has a complete "Sorting Contract" (lines 1697-1706) covering `.sort()`, `sorted()`, key functions, reverse, and float rejection — all requiring `T: Comparable`.

**Problem:** `.sort()` appears in two milestones with different semantics — a vague "basic version" in `milestone_ergonomics` and a fully specified version in `milestone_generics`. This creates ambiguity about what the implementer should build in `milestone_ergonomics`.

**Recommendation:** Choose one:
- **(A) Clarify the primitive hardcode:** Add explicit text to `milestone_ergonomics` specifying that `.sort()` works only for `list[int]`, `list[str]`, `list[bool]` via direct `vec.sort()` codegen (Rust's `Ord` trait covers these). No protocol dispatch, no key functions, no float rejection. Then `milestone_generics` upgrades to the full generic version.
- **(B) Defer `.sort()` entirely to `milestone_generics`:** Remove it from `milestone_ergonomics` DoD (line 1024). This is cleaner — one milestone owns sorting completely.

---

### Issue 2: `@property` Duplicated Across Two Milestones (MEDIUM)

**Location:** Lines 1556, 1594 (`milestone_inheritance`) and lines 2308, 2337 (`milestone_metaprogramming`)

`@property` is listed as a feature and in the DoD of both milestones:

- `milestone_inheritance`: "Properties: `@property` maps to getter methods, `@property.setter` maps to setter methods." DoD: "`@property` getter/setter works"
- `milestone_metaprogramming`: "`@property`: getter/setter generation." DoD: "`@property` generates getter/setter methods"

**Problem:** An implementer reaching `milestone_inheritance` will build `@property`. When they reach `milestone_metaprogramming`, the DoD says to build it again. This is confusing.

**Recommendation:** `@property` is a method-level feature (getter/setter dispatch), not a compile-time AST transform. It belongs in `milestone_inheritance`. Remove `@property` from `milestone_metaprogramming`'s feature list (line 2308) and DoD (line 2337). If `milestone_metaprogramming` is meant to add *enhanced* property variants (cached properties, computed properties), state that explicitly instead.

---

### Issue 3: `str(x)` Codegen Requires `Display` Before It Exists (LOW)

**Location:** Line 1156

`milestone_error_handling` defines:
> `str(x)` for any type -> `str` -- string representation. Codegen: `format!("{}", x)` (requires `Display`)

The `Display` trait (`__str__` mapping) is formalized in `milestone_protocols` (3 milestones later). However, all classes auto-derive `Debug` from `milestone_classes`, so `format!("{:?}", x)` is always available.

**Recommendation:** Add a note: "Until `milestone_protocols` provides `Display` via `__str__`, `str(x)` uses `Debug` formatting (`format!(\"{:?}\", x)`). After `milestone_protocols`, user-defined `__str__` maps to `Display` and `str(x)` uses `format!(\"{}\", x)`."

---

### Issue 4: `class Port(int)` Newtype Uses Inheritance Syntax Before `milestone_inheritance` (LOW)

**Location:** Line 1500

The newtype example `class Port(int):` uses `(int)` syntax, which looks like inheritance. The plan already has a precedent for special-casing this pattern — `milestone_error_handling` (line 1140) explicitly notes that `class Foo(Error)` is a "special-cased error declaration" and "NOT general inheritance syntax."

**Recommendation:** Add the same kind of note to the newtype section: "Note: `class Port(int)` is a special-cased newtype declaration — the compiler recognizes primitive type parents (`int`, `float`, `str`, `bool`) and generates a Rust newtype struct (`struct Port(i64)`). This is NOT general inheritance syntax; full single inheritance comes in `milestone_inheritance`."

---

### Issue 5: Tuple Slicing Contract Inconsistency (LOW)

**Location:** Line 2741 vs lines 808-812

The cross-cutting slice contract (line 2741) says:
> **Tuple:** compile-time slicing supported (milestone_ergonomics) -- the compiler can statically verify tuple slice bounds and produce a new tuple type.

This is consistent with `milestone_ergonomics` (lines 808-812) which specifies compile-time tuple slicing with constant indices. No contradiction remains — this was flagged in the earlier recheck but appears to be resolved. The contract now correctly states tuple slicing is supported.

**Status:** Resolved. No action needed.

---

## Dependency Chain Verification (Sequential)

The current strictly sequential chain is:

```
milestone_ergonomics
  -> milestone_classes
    -> milestone_error_handling
      -> milestone_safe_indexing
        -> milestone_imports
          -> milestone_protocols
            -> milestone_inheritance
              -> milestone_generics
                -> milestone_generators
                  -> milestone_decorators
                    -> milestone_core_stdlib
                      -> milestone_test_runner
                        -> milestone_ext_collections
                          -> milestone_ext_stdlib
                            -> milestone_async
                              -> milestone_web_db
                                -> milestone_data_processing
                                  -> milestone_metaprogramming
                                    -> milestone_ffi
                                      -> milestone_package_mgmt
                                        -> milestone_dev_tooling
                                          -> milestone_ecosystem
```

**25 milestones total** (3 completed, 22 remaining).

### Dependency Audit (each arrow checked)

| From -> To | Real dependency? | Notes |
|---|---|---|
| milestone_ergonomics -> milestone_classes | Yes | Classes need ergonomic features (kwargs for `__init__`, methods, etc.) |
| milestone_classes -> milestone_error_handling | Yes | Error types need `class ValueError(Error)` |
| milestone_error_handling -> milestone_safe_indexing | Yes | Safe indexing returns `Option` which needs `?`/`match` from error handling |
| milestone_safe_indexing -> milestone_imports | Yes (soft) | Completes single-file safety before multi-file. Not a hard technical dependency but good sequencing. |
| milestone_imports -> milestone_protocols | Yes | Cross-module trait definitions need imports |
| milestone_protocols -> milestone_inheritance | Yes | Inherited classes should implement protocols immediately |
| milestone_inheritance -> milestone_generics | Weak | Generics only need protocols for type bounds, not inheritance. But keeping sequential avoids complexity. |
| milestone_generics -> milestone_generators | Yes | Generators need closures and iterators |
| milestone_generators -> milestone_decorators | Yes | Decorators need closures; `@contextmanager` pattern benefits from generators |
| milestone_decorators -> milestone_core_stdlib | Yes | Stdlib uses `@decorator` patterns |
| milestone_core_stdlib -> milestone_test_runner | Yes | Test runner needs `sifr.io` and `sifr.os` |
| milestone_test_runner -> milestone_ext_collections | Yes (soft) | Dogfooding: test extended collections with Sifr's own test runner |
| milestone_ext_collections -> milestone_ext_stdlib | Yes (soft) | Some stdlib modules use `bytes` and extended collections |
| milestone_ext_stdlib -> milestone_async | Yes | Async runtime benefits from full stdlib |
| milestone_async -> milestone_web_db | Yes | Web framework needs async/await |
| milestone_web_db -> milestone_data_processing | Weak | Data processing (polars, CSV) doesn't need web/DB. Needs generics + core stdlib + decorators. |
| milestone_data_processing -> milestone_metaprogramming | Yes (soft) | Metaprogramming comes after language is functional |
| milestone_metaprogramming -> milestone_ffi | Yes | FFI benefits from stable language surface |
| milestone_ffi -> milestone_package_mgmt | Yes | Package management benefits from FFI (access to Rust ecosystem) |
| milestone_package_mgmt -> milestone_dev_tooling | Yes | Tooling needs package infrastructure |
| milestone_dev_tooling -> milestone_ecosystem | Yes | Registry needs tooling quality |

**Verdict:** All dependencies are valid for sequential execution. The `milestone_web_db` -> `milestone_data_processing` link is the weakest (data processing doesn't technically need web/DB), but in a strictly sequential plan this doesn't matter — it just determines order, and doing web before data processing is a reasonable choice (web is higher priority for most users).

---

## Spec Fixes Needed (Prioritized)

| # | Priority | Issue | Recommended Fix |
|---|----------|-------|-----------------|
| 1 | HIGH | `.sort()` in `milestone_ergonomics` references `Comparable` from `milestone_protocols` | Either clarify primitive-only hardcode or defer to `milestone_generics` |
| 2 | MEDIUM | `@property` duplicated in `milestone_inheritance` and `milestone_metaprogramming` | Remove from `milestone_metaprogramming` |
| 3 | LOW | `str(x)` codegen says `Display` but `Display` doesn't exist yet | Add note: uses `Debug` fallback until `milestone_protocols` |
| 4 | LOW | `class Port(int)` newtype needs special-case note (like `class Foo(Error)`) | Add "special-cased newtype declaration" note |

---

## Overall Assessment

The plan is in **very good shape**. The two issues from the previous recheck that were flagged as MEDIUM have been fixed (`@staticmethod` newtype example, panic-based indexing clarification). The dependency chain is sound for strictly sequential execution.

The one genuinely problematic item is `.sort()` in `milestone_ergonomics` — it references a concept (`Comparable`) that doesn't exist for 5 more milestones and has ambiguous semantics ("basic version for primitive types" is never defined). Everything else is a minor clarification.
