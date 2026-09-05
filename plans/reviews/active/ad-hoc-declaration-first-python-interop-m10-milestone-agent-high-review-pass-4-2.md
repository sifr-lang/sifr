All verification is complete. Here is the pass-4 review.

---

# M10 Milestone Review — Pass 4 (working tree at `codex/m10-milestone-closure`, agent high, whole-diff, read-only)

**Summary: every pass-2 and pass-3 blocker is verifiably fixed, and all validation claims check out exactly — but the pass-3 remediation itself introduced one new HIGH silent-correctness regression, so the milestone is not ready for the merge gate.**

**Methodology.** I read the milestone issue (`plans/issues/active/ad-hoc-declaration-first-python-interop.md`), both prior review artifacts, the complete working-tree remediation diff (25 modified files, 2 new source files, 6 new fixtures, ~358 insertions), and the surrounding implementation (`class_field_emitter.rs`, `error_contracts.rs`, `class_trait_capabilities.rs`, `try_error_carrier.rs`, `union_identity.rs`, `python_error.rs`, receiver-clone suppression machinery). The committed `origin/main...HEAD` surface was already whole-diff-reviewed in passes 2–3; I re-inspected the areas the remediation touches. I rebuilt the debug compiler from the exact working tree and ran ~15 fresh probes in `/tmp/m10rev4` plus the pass-3 probes in `/tmp/m10rev3` (forcing regeneration by deleting cached outputs), with the installed pre-M10 `~/.sifr/bin/sifr` as behavioral reference. No repository file was modified; the dirty `third_party/ruff` submodule was left untouched. (Note: an empty placeholder `plans/reviews/active/...pass-4.md` already exists untracked; I did not write to it.)

## Prior-blocker verification — all fixed

- **Pass-3 blocker 1 (builtin/mixed union error channels)** — **fixed.** `ValueError | KeyError`, mixed `CsvError | ValueError`, and aliased `type Failures = CsvError | CfgError` channels all check, build natively, and dispatch to the correct handler (verified with fresh probes and the `try_union_builtin_error_channel` / `try_union_error_alias` / `try_union_error_channel` fixtures). Builtin error structs now derive `PartialEq/Eq/Hash` (`preamble/types_and_errors.rs:127-133`), closing the enum/struct derive parity gap.
- **Pass-3 blocker 2 (canonical `PythonError` identity collision)** — **fixed.** `_sifr.python.PythonError` was removed from `GLOBAL_RUST_NOMINAL_IDENTITIES`; the canonical class now emits as `__SifrStdlib___sifr_x2epython_x2ePythonError`, distinct from a local `struct PythonError`. The derive on the canonical struct is sound because runtime `PythonError` carries manual `PartialEq/Eq/Hash` impls (`crates/sifr_runtime/src/python/python_error.rs:146-166`). Verified via single-file `emit`, a forced-fresh package-mode rebuild of the pass-3 probe (prints `x`), and the new ignored driver test, which I ran explicitly: 1 passed in 18.6s. An adversarial probe with a user class spelling the exact mangled name is escaped injectively (`__SifrSource_____SifrStdlib______sifr__x2epython__x2ePythonError`) — no collision.
- **Pass-3 finding 3 (alias channel + leaked debug dump)** — fixed; invalid alias unions now produce a clean SIFR-RESULT-0002 rendering the alias name.
- **Pass-3 finding 4 (order-sensitive duplicate union enums)** — fixed; `A | B` and `B | A` now share one enum (`unordered_sequence` in `union_identity.rs`; probe emitted exactly one `__SifrUnion` enum and ran correctly). Degenerate `E | ValueError` (alias duplicating a member) also builds and runs.
- **Pass-3 finding 5 (missing upcast negative test)** — added (`class_upcasts.rs` `basename_only_ancestor_match_does_not_emit_an_upcast`).
- **Pass-2 findings 3/4 (catch-all identity, basename tail-match)** — still fixed; builtin `except Error` catch-all works, `imported_error_not_catch_all` passes natively, immutable-borrow mutation is rejected at check (SIFR-OWN-0005).

## Blocking finding

### 1. HIGH — Mutating method calls on move-valued fields of `mut`-borrowed class parameters are silently discarded (new regression from the pass-3 remediation)

- **Files:** `crates/sifr_codegen/src/expr_render_helpers/field_and_stdlib_rewrites.rs:152-158` (the new `is_borrowed_parameter` clone condition), interacting with the receiver-clone suppression at `field_and_stdlib_rewrites.rs:144-151`.
- **Repro (verified with the working-tree compiler):**
  ```python
  class Crate:
      items: list[int]
  def add(mut item: Crate) -> None:
      item.items.append(7)
  def main() -> None:
      c: Crate = Crate([1, 2])
      add(c)
      print(len(c.items))
  ```
  `check` passes, `build` passes, and the program prints **2** — the append is lost. Emitted Rust: `fn add(item: &mut Crate) { item.items.clone().push(7_i64); }`. The pre-M10 installed release prints the correct **3**. `.extend` and `.pop` fail the same way (probe expected `3, 2`, got `1, 1`), so the entire mutating-method family (`append`, `extend`, `pop`, `sort`, `insert`, `remove`, `clear`, dict/set mutators) is affected. Direct field assignment (`h.label = ...`) and index assignment (`h.table["k"] = 42`) still work.
- **Root cause:** the pass-3 fix for borrowed move-valued field *reads* added `borrowed_params`/`mut_borrowed_params` to the `needs_clone` condition. But the existing receiver-clone suppression (`pending_self_field_clone_suppression`, armed by `method_call_needs_field_clone_suppression` before lowering a mutating call's receiver) is only *consumed* when `is_self_access || is_recursive_field` — so `self.items.append(...)` correctly skips the clone (verified: prints 3), while a borrowed-parameter receiver clones anyway and the mutation lands on a temporary.
- **Why blocking:** this is a silent wrong-answer in compiled output — strictly worse than the fail-closed E0507 it replaced, and a direct violation of the "if it compiles, it works" core guarantee. It ships in the exact diff under review and is user-reachable with three lines of ordinary code.
- **Remediation direction:** extend the suppression-consumption predicate at `field_and_stdlib_rewrites.rs:144-151` to also fire for borrowed-parameter receivers (or gate the `is_borrowed_parameter` clone to non-receiver positions). Add (a) a native pass fixture exercising `mut`-param field mutation via `append`/`extend`/`pop` with asserted lengths, and (b) a unit test mirroring `field_read_from_borrowed_parameter_clones_move_value` that asserts *no* clone is emitted for a mutating-call receiver. Today no fixture, demo, or unit test covers this shape (the only class-typed `mut`-param demo, `tuple_assignment`, uses field assignment), which is why the fully green validation suite coexisted with the regression.

## Non-blocking findings (fix-forward)

2. **LOW — Same-basename diagnostics remain unqualified on the common paths.** The remediation claim "same-basename union diagnostics qualify exact identities" is only partially realized: the new qualification in `format_type_name` (`lower/diagnostics.rs:94-122`) fires only when duplicates appear inside one rendered union. `SIFR-TYPE-0002` still prints `expected 'str', got 'Result[int, Error | Error]'`… wait — that path *does* have duplicates and still prints `Error | Error` (it uses the type checker's own formatter, not `format_type_name`), and `SIFR-RESULT-0005` reports the uncovered member as bare `Error` when the program has two same-basename imports. Both verified with probes. Cosmetic, fail-closed, but confusing for exactly the same-basename programs M10 legitimizes.
3. **LOW — `crates/sifr_runtime/src/python.rs` still sits at exactly the 900-line cap** (pass-3 finding 7, unaddressed). The next touch forces a mid-change refactor.
4. **LOW (pre-existing, recommend follow-up issue) — attribute augmented assignment through a `mut` parameter leaks an internal error.** `h.label += "!"` fails the build with `compile_error!("structured statement emission missing for production path: AttributeAugAssign {...}")` — fail-closed on both pre-M10 and this branch, but the raw internal dump surfaces in user-facing build output.

## Validation claims — all independently confirmed

`sifr_type_system` 114, `sifr_codegen` 862, `sifr_lowering` 779 (+1 ignored), `sifr_driver` 358 (+33 ignored) all pass; the ignored `test_build_package_keeps_runtime_and_source_python_error_identities_distinct` passes explicitly; all six new fixtures build and run natively (exit 0, assert-based); `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, HIR maintainability guardrails, and the file-size guardrail (2713 files, 900-line limit) all pass. No new panic paths, fallback paths, or dependency-gating gaps found in the remediation; runtime-python feature detection covers both namespaces with unit coverage.

## Verdict

The milestone's own remit (nominal identity, union carriers, check/build parity) is now in good shape — every previously recorded blocker is closed with fixtures and probes to show for it. But the borrowed-field clone remediation trades a fail-closed build error for silent data loss on `mut`-parameter field mutations, which is the most severe failure class this project defines. That must be fixed, covered by the missing regression tests, and the authoritative gate rerun before closure.

VERDICT: NOT SATISFIED
