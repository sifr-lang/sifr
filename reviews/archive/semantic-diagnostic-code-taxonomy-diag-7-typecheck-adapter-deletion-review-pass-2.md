# Review Pass 2 — milestone_diag_7 slice 3: TypeCheckDiagnostic Adapter Deletion

Scope under review: the same working-tree slice as pass 1, plus the non-blocking polish folded in afterwards. The slice still deletes `sifr_type_system::TypeCheckDiagnostic` (struct + `Display`/`Error` impls), removes the `LowerCtx::type_check_diagnostic` shim, and rewrites the four operator helpers to return `(DiagnosticCode, String)` directly. Pass 2 inspects the deltas the author landed on top of pass 1's SATISFIED verdict.

Pass-2 deltas inspected:

- [crates/sifr_hir/src/lower/expressions_tests.rs](../crates/sifr_hir/src/lower/expressions_tests.rs) — two pre-existing tests strengthened (`test_attribute_subscript_augassign_lowers_for_class_fields`, `test_nested_subscript_augassign_lowers_for_name_targets`) and two new tests added (`test_list_subscript_augassign_type_error_keeps_code`, `test_dict_subscript_augassign_type_error_keeps_code`).
- [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md) — TYPE_MISMATCH inventory row now says "equality comparison mismatch from operator helpers; broader expected/actual mismatch from HIR sites" in the message-category column and "...; helper-specific comparison fixture pending" in the fixture column; column header softened from "Representative fixture" to "Representative code fixture".

The rest of the slice (helper return-type rewrite, four behavioral upgrades, adapter deletion, shim deletion, `TypeCheckResult` private alias) is unchanged from pass 1.

---

## Verdict: SATISFIED — mergeable

The pass-2 polish is good. F2/N1 (TYPE_MISMATCH inventory row imprecision) is now documented honestly. F3/N2 (lost dedicated boundary test for the operator-helper → HIR transport edge) is now covered, and the four formerly-uncoded sites identified in pass 1 each have HIR-level regression coverage. No new blockers. No behavioral regressions. No residual adapters re-introduced. No fallback paths or `Option<DiagnosticCode>` edges.

I remain satisfied; this is mergeable.

---

## What Holds Up After Pass 2

### F3/N2 is now covered — destructure-and-route step is locked at the HIR boundary

Pass 1 N2 noted that no single test asserted "operator-helper `Err` ⇒ `LoweringError` with `Some(code)`" at the HIR call site. Pass 2 adds two such tests directly and strengthens two more, giving each of the four pass-1 behaviorally-upgraded sites its own HIR-level guardrail:

| Behaviorally-upgraded site (pass 1) | Pass-2 test that locks it |
| --- | --- |
| [aug_assign_lowering.rs:130-135](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:130) (nested subscript `obj[a][b] += rhs`) | [`test_nested_subscript_augassign_lowers_for_name_targets`](../crates/sifr_hir/src/lower/expressions_tests.rs:485) — strengthened to assert `code == Some(TYPE_UNSUPPORTED_OPERATOR)` |
| [aug_assign_lowering.rs:198-204](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:198) (attribute-subscript `obj.field[i] += rhs`) | [`test_attribute_subscript_augassign_lowers_for_class_fields`](../crates/sifr_hir/src/lower/expressions_tests.rs:458) — strengthened to assert `code == Some(TYPE_UNSUPPORTED_OPERATOR)` |
| [container_literal_specialization.rs:140-141](../crates/sifr_hir/src/lower/container_literal_specialization.rs:140) (list element `xs[i] += rhs`) | [`test_list_subscript_augassign_type_error_keeps_code`](../crates/sifr_hir/src/lower/expressions_tests.rs:511) — new test; `xs[0] += "x"` with `xs: list[int]` |
| [container_literal_specialization.rs:153-168](../crates/sifr_hir/src/lower/container_literal_specialization.rs:153) (dict value, non-empty-specialization branch) | [`test_dict_subscript_augassign_type_error_keeps_code`](../crates/sifr_hir/src/lower/expressions_tests.rs:524) — new test; `data["x"] += "x"` after `data["x"] = 1` with `data: dict[str, int]` |

I confirmed the four tests run and pass: `cargo test -p sifr_hir --lib augassign -- --nocapture` reports 4 of the matching tests as `ok` (alongside two unrelated augassign tests). Each assertion is the canonical pair-shape `error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR) && error.message.contains("unsupported operand type(s) for +")` — i.e., the test will fail if a future change re-introduces the "drop the code" bug at any of the four sites. The tests also outlive future helper-signature restructuring because they exercise the destructure-and-route step from HIR, not the helper return type directly.

The dict test is correctly aimed at the *non-empty-specialization* else branch in `validate_subscript_augassign_target`: typing `data: dict[str, int]` (no empty-dict literal) means the dict never enters `ctx.empty_dict_specializations`, so `type_check_binary_op(int, "+", str)` failure flows through the `else { ctx.error_with_code(code, message) }` arm — which is exactly the upgraded site at [container_literal_specialization.rs:166](../crates/sifr_hir/src/lower/container_literal_specialization.rs:166). The first `data["x"] = 1` is incidental setup and does not specialize anything.

The list test similarly avoids the "augmented subscript assignment target must be a simple name" early-error path by using a name target (`xs[0]`), so the failure unambiguously originates from the `type_check_binary_op` call at [container_literal_specialization.rs:140](../crates/sifr_hir/src/lower/container_literal_specialization.rs:140).

These two new tests, plus the two strengthened ones, satisfy F3 entirely. Pass-1 follow-up F1 (e2e fail fixtures specifically) is technically a different mitigation, but the HIR-unit-test substitution provides equivalent regression coverage with substantially less e2e fixture cost — and unit tests are tighter on what they assert (`code == Some(...)` directly, vs. e2e expectation strings). I read this as a deliberate and reasonable trade.

### F2/N1 is now documented honestly

The pass-1 finding was that the TYPE_MISMATCH inventory row claimed `crates/sifr/tests/e2e/fail/type_mismatch.sifr` and `union_type_mismatch.sifr` as fixtures for the row, even though those fixtures actually exercise the assignment-shape `TYPE_MISMATCH` from HIR lowering paths — not the operator-helper's equality-comparison fallback at [check.rs:381-388](../crates/sifr_type_system/src/check.rs:381).

Pass 2 lands two coordinated edits at [inventory.md:64-74](../internal_docs/diagnostic_emission_inventory.md:64):

- The column header is softened to "Representative code fixture" (less of a claim than "fixtures that exercise this exact emission site").
- The TYPE_MISMATCH row's message-category column now reads "equality comparison mismatch from operator helpers; broader expected/actual mismatch from HIR sites" — explicitly partitioning the two emission populations.
- The TYPE_MISMATCH row's fixture column appends "; helper-specific comparison fixture pending" — explicitly admitting the gap.

This is the (b)-style fix called out in pass 1's N1 (relaxing the column header to acknowledge the fixtures cover the SIFR code in general). It is honest documentation: a future reader will not mistake the listed fixtures for guards on the helper's comparison-fallback emission, and the "pending" tag flags the gap for whoever lands the comparison fixture later. The rest of the table rows are unchanged and remain accurate (each canonical code is still constructed by an active `Err((<const>, ...))` site in [check.rs](../crates/sifr_type_system/src/check.rs)).

### No regression in the symbol-deletion claim

`rg -n 'TypeCheckDiagnostic|type_check_diagnostic' crates/ -g '*.rs'` returns nothing — adapter struct, `Display`/`Error` impls, and the `LowerCtx::type_check_diagnostic` shim remain physically gone. The pass-2 changes did not re-introduce any of them. The pass-2 strengthened tests reference only `LoweringError`, `DiagnosticCode`, and `lower_module` — they do not transitively re-import the deleted symbol (the relevant `use sifr_type_system::TypeCheckDiagnostic` from `diagnostic_transport_tests.rs` was deleted in pass 1 and stays deleted).

### No regression in the helper return shape

`rg -n 'TypeCheckResult' crates/ -g '*.rs'` returns the four expected hits in [check.rs](../crates/sifr_type_system/src/check.rs): one `type` definition and three signatures. The alias remains private (no `pub`), still expands to bare `Result<Type, (DiagnosticCode, String)>`, and the call sites in `sifr_hir` continue to destructure the bare tuple. Pass-2 polish did not promote it to public, did not sneak it into a shared crate, and did not turn it into a wrapper struct. Compliance with "no residual adapter" holds.

### The discard site at `nested_function_inference.rs:1440` is still safe

[nested_function_inference.rs:1440-1441](../crates/sifr_hir/src/lower/nested_function_inference.rs:1440) `type_check_binary_op(...).unwrap_or_else(|_| infer_numeric_result_type(...))` is unchanged from pass 1. The closure binds the whole error tuple as `_` — type-inference fallback, not error reporting — and is unaffected by the new tuple shape. Pass 2 did not touch this site. Still correct.

### Validation cross-check (re-run by reviewer)

I re-ran on the working tree to confirm the author's reported lane:

- `cargo test -p sifr_hir --lib augassign -- --nocapture` — 6 tests passed, including the four pass-2 boundary tests.
- `cargo test -p sifr_hir --lib diagnostic_transport_tests -- --nocapture` — 2 tests passed (transport edge unchanged).
- `cargo test -p sifr_type_system` — 85 passed (helper-side unit tests still locking the canonical codes via `.0`).

These corroborate the author's stated validation lane (`cargo fmt --check`, `cargo test -p sifr_hir augassign_type_error_keeps_code -- --nocapture`, `cargo test -p sifr_hir augassign_lowers -- --nocapture`, plus the prior slice's `cargo test -p sifr_type_system`, `cargo test -p sifr_hir diagnostic_transport_tests -- --nocapture`, `cargo clippy -p sifr_type_system -p sifr_hir --no-deps -- -D warnings`, `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture`). I would still recommend running `scripts/run_all_tests.sh --profile quick` — the AGENTS.md authoritative gate — before opening the PR if it has not been re-run since the pass-2 additions; the test-only delta is small enough that a regression is unlikely, but the gate is fast.

---

## Findings

### Blockers

None.

### Non-blockers

#### NB1 — F1 was satisfied via HIR unit tests rather than e2e fail fixtures (intentional substitution worth noting in the PR description)

Pass-1 follow-up F1 specifically asked for e2e fail fixtures asserting the canonical `SIFR-TYPE-0005` line on each of the four upgraded sites. Pass 2 provides equivalent regression coverage as HIR unit tests in `expressions_tests.rs` instead. I think the substitution is fine — the unit tests are tighter on what they assert (direct `code == Some(...)` comparison vs. e2e expectation parsing), faster to run, and located alongside the lowering code that owns the upgraded paths. The e2e fail-fixture suite is also already exercising the helper's `TYPE_UNSUPPORTED_OPERATOR` code via `optional_arithmetic_without_narrowing.sifr` and the bigint/decimal mixing fixtures, so a regression that flipped a code at the renderer boundary would still be caught by the existing fail fixtures.

The case for adding e2e fixtures anyway: they exercise the full end-to-end render path (renderer + `# expect-error` grammar) at the upgraded sites, which the HIR unit tests do not. The case against: marginal, given the existing per-code fail fixtures already lock the renderer boundary.

I would call this out in the PR description so a future reviewer doesn't read "F1 unaddressed" — the right framing is "F1 was substituted with HIR unit tests covering the same four sites; e2e fixture coverage left to a future cleanup if/when desired." Non-blocking.

#### NB2 — TYPE_MISMATCH "helper-specific comparison fixture pending" is now a documented commitment

The pass-2 inventory note "helper-specific comparison fixture pending" creates an implicit follow-up: someone needs to either (a) add a fixture exercising `int == str` (or similar) so the helper's [check.rs:381-388](../crates/sifr_type_system/src/check.rs:381) emission has a fixture-level guardrail, or (b) decide that helper-comparison `TYPE_MISMATCH` doesn't need its own fixture and remove the "pending" annotation.

This is also non-blocking — the row is now accurate either way — but the next slice in this milestone wave should pick (a) or (b) rather than letting "pending" linger.

#### NB3 — `TypeCheckResult` private alias readability (carried over from pass 1)

Pass-1 N3 flagged the private cosmetic alias as a wash readability-wise. Pass 2 did not change it. Still fine; not a blocker. If the next slice (F4 from pass 1 — uplifting helper failures to canonical `SifrDiagnostic` with declared args + spans) restructures these helpers anyway, the alias either disappears or transforms naturally; no need to act on it now.

---

## Correctness, Stale-Doc, And Behavioral-Regression Sweep

- **Correctness.** Each new HIR test asserts both the canonical code (`Some(TYPE_UNSUPPORTED_OPERATOR)`) and the message substring (`"unsupported operand type(s) for +"`). The pair is the right contract: the message guards against a future regression that swaps the helper for a different (still-coded) emission, and the code guards against a regression that drops the code. The test fixtures route through the four upgraded paths exactly once each, and I confirmed by inspection that nothing else in the lowering pipeline emits `TYPE_UNSUPPORTED_OPERATOR` for these specific operand shapes (the operator helpers are the only producer of that constant in the lowering crates). No false-pass risk.
- **Stale docs.** The pass-2 inventory edits are internally consistent: the surface-section table, the message-category column, and the column header all describe the new `(DiagnosticCode, String)` shape. The "Public-code mechanisms to remove" row at [inventory.md:113](../internal_docs/diagnostic_emission_inventory.md:113) was already updated in pass 1 to mention `error_with_code`; pass 2 did not change it and it remains accurate. The span/related-span note at [inventory.md:369](../internal_docs/diagnostic_emission_inventory.md:369) is still correct. The issue checkbox at [issues/...md:81](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:81) remains the in-progress marker; no edit needed.
- **Behavioral regressions.** Zero. The pass-2 polish is test-only and doc-only; the production paths (helper returns, destructure call sites, `LowerCtx::error_with_code`) are byte-identical to the pass-1 state.
- **No-fallback / no-residual-adapter compliance.** Holds. The four helper signatures still return `Result<Type, (DiagnosticCode, String)>` (via the private `TypeCheckResult` alias). No `Option<DiagnosticCode>` anywhere on the operator-helper boundary. No message-substring classifier. No new transport struct. No `pub use` re-export of a deleted symbol. The four behaviorally-upgraded sites still route the canonical code through `error_with_code` exactly as they did in pass 1.

---

## Bottom Line

Pass-1's three optional follow-ups are now landed (F2 documented honestly, F3 covered with HIR boundary tests, F1 satisfied via HIR unit tests in lieu of e2e fixtures). The slice continues to satisfy the literal contract — `TypeCheckDiagnostic` and `type_check_diagnostic` are gone, the operator helpers return canonical `(DiagnosticCode, String)` directly, the four formerly-uncoded subscript / attribute-subscript aug-assign sites now carry their canonical codes through to the renderer, and the destructure-and-route step at every HIR call site is now locked by either an in-crate type-system unit test (for the helper-side contract) or an HIR `expressions_tests` test (for the boundary contract).

Reviewer remains satisfied. Mergeable. Two items worth mentioning in the PR description for reviewer hygiene: (NB1) the F1 substitution from e2e fixtures to HIR unit tests is intentional, and (NB2) the inventory's "helper-specific comparison fixture pending" annotation is a deliberate commitment for a future small slice (or an explicit decision to drop the "pending" tag). Neither is a blocker.
