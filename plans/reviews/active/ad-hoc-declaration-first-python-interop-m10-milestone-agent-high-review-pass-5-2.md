All the evidence is in. Here is the pass-5 review.

---

# M10 Milestone Review — Pass 5 (branch `codex/m10-milestone-closure` + working tree, PR #2990, agent high, read-only)

**Summary: every prior blocker from passes 2–4 is verifiably closed and every validation claim independently reproduces — but the pass-4 remediation for the silent mutation-loss regression is incomplete. Two adjacent, user-reachable shapes of the exact same defect remain in the working tree, one of them a hard regression from correct pre-M10 behavior. The milestone is not ready to close.**

**Methodology.** I read the milestone issue, the pass-2/3/4 review artifacts, the full working-tree remediation diff (26 files, ~396 insertions, 2 new source files, 7 new fixtures) and the surrounding implementation (`field_and_stdlib_rewrites.rs`, `class_field_emitter.rs`, `error_contracts.rs`, `try_error_carrier.rs`, `class_upcasts.rs`, `union_identity.rs`, `source_names.rs`, `python_interop_common.rs`, lowering dispatch/diagnostics, driver tests). The committed `origin/main...HEAD` surface was whole-diff-reviewed in passes 2–4; I re-inspected every area the current remediation touches. I rebuilt debug and release compilers from the exact working tree, ran all claimed test suites and gates myself, and ran ~12 fresh behavioral probes in `/tmp/m10rev5` with the pre-M10 installed `~/.sifr/bin/sifr` 0.1.0-beta.14 as reference. No repository file was modified; the dirty `third_party/ruff` submodule was left untouched.

## Prior-blocker verification — all closed as claimed

- **Pass-4 blocker (mutating calls on move-valued fields of `mut`-borrowed parameters)** — fixed *for the reported shape*. The pass-4 repro now prints 3; `extend`/`pop` preserve mutations and return values; the new fixture `mut_borrowed_parameter_field_mutation.sifr` passes natively; both prescribed unit tests exist (`field_read_from_borrowed_parameter_clones_move_value`, `mutating_field_receiver_from_mut_borrowed_parameter_does_not_clone`). The pass-3 read fix is intact (no E0507; borrowed reads clone), and mutation through an immutably-borrowed parameter is still rejected at check (SIFR-OWN-0005). **However, the fix is keyed too narrowly — see the blocker below.**
- **Pass-3 blockers 1–2 and pass-2 findings** — still closed: all seven new fixtures (`try_union_error_channel`, `try_union_builtin_error_channel`, `try_union_error_alias`, `imported_error_not_catch_all`, `python_error_contract_without_interop`, `python_error_source_name`, plus the mut-param fixture) build and run natively; the ignored package-mode `PythonError` identity-collision test passes explicitly (1 passed); catch-all classification now keys on `is_builtin_error_base` identity (with negative unit test `same_basename_import_is_not_a_catch_all`); the basename tail-match upcast fallback is deleted with the prescribed negative test; `_sifr.python.PythonError` is off the global-identity exemption list with the `assert_ne!` guard in `source_names.rs`; union identity keys are order-insensitive (`unordered_sequence`) with unit coverage; runtime-python detection covers both namespaces.

## Validation claims — all independently confirmed

`sifr_codegen` 863, `sifr_lowering` 779 (+1 ignored), `sifr_type_system` 114, `sifr_driver` 358 (+33 ignored), the explicit ignored identity test 1 passed; `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, HIR maintainability guardrails, and the file-size guardrail (PASS, 2714 files, 900-line limit) all pass; the native fixture and pass-4 repros preserve `append`/`extend`/`pop` mutations. `class_emitter.rs` is at 786 lines after the split.

## Blocking finding

### 1. HIGH — Silent mutation loss persists for two shapes adjacent to the pass-4 fix: nested field chains and `mut` class-method parameters

The pass-4 remediation (`field_and_stdlib_rewrites.rs:41-46,150-166`) arms clone suppression only when the mutating receiver's parent is **directly a `Name` in `mut_borrowed_params`**, while the new clone condition fires for **any** field access whose object is a name in `borrowed_params` *or* `mut_borrowed_params`. Two reachable shapes fall in the gap:

- **(a) Nested field chain through a `mut` parameter — regression from correct pre-M10 behavior.**
  ```python
  class Inner:  items: list[int]
  class Outer:  inner: Inner
  def add(mut o: Outer) -> None:
      o.inner.items.append(7)
  ```
  `check` passes, `build` passes, prints **2** (append lost). Emitted Rust: `o.inner.clone().items.push(7_i64)` — the inner access `o.inner` takes the new borrowed-parameter clone, and no suppression reaches it (the receiver's parent is a `FieldAccess`, not a `Name`). The pre-M10 installed release accepts the same program and prints the correct **3**.
- **(b) `mut` parameter of a class method — fail-closed became silently wrong.**
  ```python
  class Crate:
      items: list[int]
      def merge(self, mut other: Crate) -> None:
          other.items.append(1)
  ```
  Prints **1** (append lost). Emitted Rust: `fn merge(&self, other: &Crate) { other.items.clone().push(1_i64); }` — method `mut` parameters are emitted as `&T` and tracked in `borrowed_params`, not `mut_borrowed_params`, so the suppression never arms while the clone condition (which includes `borrowed_params`) fires. Pre-M10 this failed closed at build (SIFR-BUILD-0005).

**Why blocking:** identical failure class to the pass-4 blocker — check passes, build passes, wrong answer at runtime — the most severe class this project defines ("if it compiles, it works"). Shape (a) is a strict regression versus pre-M10 correct behavior; shape (b) converts a fail-closed build error into silent data loss. Both are ordinary three-line programs introduced by the exact working-tree diff under review.

**Remediation direction:** key both the arm predicate (`method_call_needs_field_clone_suppression`) and the consumption on the *root* of the receiver chain (walk `FieldAccess` parents to the base `Name`) rather than the immediate parent, and ensure suppression covers every intermediate field access in a mutating receiver chain (the single-decrement counter only reaches the outermost access). For method `mut` parameters, either lower them into `mut_borrowed_params`/`&mut` like free-function parameters, or reject the mutation at check to restore fail-closed behavior. Add native fixtures for both shapes with asserted post-mutation values — today no fixture or unit test covers a receiver chain deeper than one field or a method-scope `mut` parameter, which is why the fully green suite coexists with the defect.

## Non-blocking findings (fix-forward)

2. **MEDIUM (pre-existing, urgent follow-up issue recommended)** — the same nested silent loss exists through `self`: `self.inner.items.append(7)` prints 2 on **both** the working tree and pre-M10, so it predates M10 and is out of milestone scope — but it is the same core-guarantee violation and should be fixed together with blocker 1(a) since they share a root cause.
3. **LOW** — `is_builtin_error_base` (`error_contracts.rs:9`) is a pure shape match (identity `None`, name `Error`, single `message: str`, no parent); a user-defined `class Error: message: str` type-checks and would be classified as a catch-all handler. Marginal, since such a class shadows the builtin anyway, but an identity-based tag would be more robust than shape.
4. **LOW** — `crates/sifr_runtime/src/python.rs` remains at exactly the 900-line cap (pass-3/4 finding, still unaddressed); the next touch forces a mid-change refactor.
5. **LOW** — same-basename diagnostic qualification remains partial (pass-4 finding 2 unchanged): `format_type_name` qualifies duplicates within one rendered union, but SIFR-RESULT-0005 and type-checker-formatted paths still print bare `Error`.

## Verdict

The milestone's declared remit — nominal identity, union carriers, catch-all identity, `PythonError` collision behavior, check/build parity, feature gating, decomposition — is in good shape, and every previously recorded blocker is genuinely closed with fixtures and tests. But the pass-4 fix stops one syntactic level short of the defect it was meant to close: nested and method-parameter variants of the same silent mutation loss remain, one of them a regression from correct pre-M10 output. Per the standing bar (no unresolved correctness blocker), this must be fixed, covered by the missing nested/method-shape regressions, and the authoritative gate rerun before closure.

VERDICT: NOT SATISFIED
