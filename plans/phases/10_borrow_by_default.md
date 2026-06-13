# Borrow-by-Default

**Why now:** Safety remediation is done. The codegen no longer panics. Error types have been refined into a CPython-aligned subclass hierarchy with compile-time exhaustiveness checking. Now we can safely change the parameter passing convention without worrying about interacting with broken safety paths or incomplete error type infrastructure.

This phase changes Sifr's function parameter passing from move-by-default to borrow-by-default. Function arguments are immutably borrowed (`&T`) by default, with opt-in `mut` (mutable borrow, `&mut T`) and `own` (ownership transfer, `T`) keywords.

---

## milestone_borrow_default: Borrow-by-Default Parameter Passing

status: completed

**Goal:** Change Sifr's function parameter passing from move-by-default to borrow-by-default. This is the core language change.

**Depends on:** milestone_error_subclasses (Phase 09 safety remediation and error hierarchy must be complete)

As defined in [05_borrow_by_default.md](05_borrow_by_default.md) (original detailed plan):

- Add `ParamConvention` enum (`Borrow`, `MutBorrow`, `Own`)
- Parse `mut`/`own` soft keywords
- Delete `borrows_args` hardcoded list
- Update all call paths (regular, Callable, method) to convention-aware logic
- Codegen emits `&T` / `&mut T` / `T` based on convention

### Definition of Done (milestone_borrow_default)

- `ParamConvention` enum exists in the type system
- `FunctionType.params` carries conventions; `Callable` type variant carries conventions
- `mut`/`own` keywords parse correctly on function parameters
- Convention propagates from AST through HIR to codegen
- User-defined functions emit `&T` by default for Move-type params
- `borrows_args` hardcoded list is deleted
- All call paths (regular, Callable, method) use convention-aware move tracking
- Call sites emit `&arg`/`&mut arg`/`arg` based on callee conventions
- Borrowed parameter escape (return/store) produces a compiler error, not silent `.clone()`
- Existing E2E tests pass (with necessary adjustments for new semantics)

---

## milestone_borrow_hardening: Exclusivity, Escape Analysis, and Diagnostics

status: completed

**Goal:** Harden the borrow-by-default model with exclusivity enforcement, escape analysis, consuming-self method receivers, for-loop element semantics, clear error messages, and comprehensive tests.

**Depends on:** milestone_borrow_default (borrow-by-default must be working)

### Work Items

- Mutable borrow exclusivity tracking
- Escape analysis enforcement: returning or storing a borrowed parameter produces a compiler error with actionable diagnostic ("use `own` or `.clone()`"), not a silent `.clone()` insertion
- Validate consuming-self method receiver patterns: methods that consume `self` (builder pattern) must correctly emit `self` (move) in codegen, and calling a consuming method on a variable must invalidate it
- **For-loop element semantics:** Resolve and document whether `for x in items` borrows elements (Rust-like `&T` iteration) or clones them (Python-like independent copies). Architecture contract #2 says "for-loop elements: borrow by default" — implement this, and add E2E tests proving that mutating `x` inside the loop does not mutate the collection, and that storing `x` beyond the loop body produces a compiler error (escape analysis). Document the final semantics in `architecture.md` under the Borrow/Lifetime contract.
- Clear error messages for all borrow violations
- Update all 50 borrowing audit tests
- New E2E pass/fail tests (including escape-analysis, consuming-self, and for-loop element cases)
- Multi-module convention tests
- Fix the 7 known codegen regressions from the borrowing audit

### Definition of Done (milestone_borrow_hardening)

- Exclusivity errors caught by `sifr check` with clear error messages
- Escape analysis enforcement: no silent `.clone()` insertion
- Consuming-self method receivers work correctly in codegen
- For-loop element semantics resolved, implemented, documented, and tested
- All 50 borrowing audit tests updated and passing/failing correctly
- New E2E pass/fail tests for borrowed_parameters, mut_param, own_param, exclusivity, escape-analysis, consuming-self, for-loop elements
- Parser snapshot tests cover `mut`/`own` soft keyword edge cases
- Multi-module convention tests verify `FunctionType`/`Callable` convention propagation across imports
- 7 known codegen regressions from the borrowing audit fixed
- Architecture documentation updated

---

## milestone_borrow_stdlib: Stdlib Ownership Patterns

status: completed

**Goal:** Exercise `mut` and `own` in the stdlib to prove the model works in real code.

**Depends on:** milestone_borrow_hardening (ownership model must be fully hardened)

### Work Items

- Convert `heapq` to use `mut` parameters (in-place mutation, O(n) heapify)
- Convert `bisect.insort_*` to use `mut` parameters
- Add at least one `own` parameter stdlib function (e.g., `itertools.chain`)
- Fix generator + borrow interaction (generator state machine captures borrowed parameters)
- Replace `Counter` JSON workaround with native `dict[str, int]` field

### Definition of Done (milestone_borrow_stdlib)

- `heapq` functions use `mut` parameters for in-place mutation
- `bisect.insort_*` functions use `mut` parameters
- At least one stdlib function uses `own` parameter
- Generator + borrow interaction works correctly
- `Counter` uses native `dict[str, int]` field instead of JSON workaround
- E2E tests proving `mut`/`own` work correctly in stdlib context

---

## Milestone Ordering

- **milestone_borrow_default first:** The core language change must land before hardening can begin.
- **milestone_borrow_hardening second:** Exclusivity checking, escape analysis, consuming-self, for-loop semantics, and regression fixes build on the working borrow-by-default codegen.
- **milestone_borrow_stdlib third:** Stdlib ownership patterns exercise the fully hardened model in real code, proving it works before Phase 11 adds 50+ new functions.
