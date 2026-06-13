# Review pass 1 — `milestone_diag_8` slice 2: if/while control-flow condition migration to `SIFR-FLOW-0005`

## Summary

Reviewed the uncommitted implementation that migrates the `if`/`while` condition type-validation diagnostic from raw `ctx.error(...)` transport to a dedicated `SIFR-FLOW-0005` code with a domain helper, registry entry, generated docs, internal index entry, inventory entry, HIR regression assertions for both keywords, an e2e fail fixture, and the in-progress entry on the phase issue tracker.

**Result: satisfied.** No blockers. No required nits. Implementation is correct, in-scope, internally consistent with the slice‑1 pattern, and introduces no fallback or compatibility paths.

## Files inspected

- [crates/sifr_diagnostics/src/codes.rs](crates/sifr_diagnostics/src/codes.rs:75) — constant, registry entry, active-list inclusion
- [crates/sifr_hir/src/lower/flow_diagnostics.rs](crates/sifr_hir/src/lower/flow_diagnostics.rs:32) — new `invalid_condition_type` helper
- [crates/sifr_hir/src/lower/control_flow_conditions.rs](crates/sifr_hir/src/lower/control_flow_conditions.rs:25) — migrated emission site
- [crates/sifr_hir/src/lower/statements.rs:1694](crates/sifr_hir/src/lower/statements.rs:1694), [crates/sifr_hir/src/lower/statements.rs:1982](crates/sifr_hir/src/lower/statements.rs:1982) — unchanged callers (`"if"` / `"while"` keyword passes)
- [crates/sifr_hir/src/lower/expressions_tests.rs:674](crates/sifr_hir/src/lower/expressions_tests.rs:674), [crates/sifr_hir/src/lower/expressions_tests.rs:686](crates/sifr_hir/src/lower/expressions_tests.rs:686) — strengthened HIR regression tests
- [crates/sifr/tests/e2e/fail/if_condition_numeric_truthiness.sifr](crates/sifr/tests/e2e/fail/if_condition_numeric_truthiness.sifr) — new e2e fail fixture
- [docs/errors/SIFR-FLOW-0005.md](docs/errors/SIFR-FLOW-0005.md) — generated per-code page
- [docs/errors/diagnostic-codes.md:84](docs/errors/diagnostic-codes.md:84) — public index updated
- [internal_docs/diagnostic_codes.md:108](internal_docs/diagnostic_codes.md:108) — internal index updated
- [internal_docs/diagnostic_emission_inventory.md:324](internal_docs/diagnostic_emission_inventory.md:324) — inventory updated
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:89](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:89) — slice‑2 in-progress entry

## Correctness

- **Constant** ([codes.rs:75](crates/sifr_diagnostics/src/codes.rs:75)): `FLOW_INVALID_CONDITION_TYPE` declared as `Self::new("SIFR-FLOW-0005", Severity::Error)`. Sits between `FLOW_MISSING_RETURN_VALUE` (FLOW‑0004) and `FLOW_UNREACHABLE_STATEMENT` (FLOW‑0901), preserving the family's ordered declaration block.
- **Registry entry** ([codes.rs:925‑935](crates/sifr_diagnostics/src/codes.rs:925)): family `FLOW`, summary `"Control-flow condition has an unsupported type."`, error severity, owner `sifr_hir::lower::control_flow_conditions` (this is the actual emitting module per [control_flow_conditions.rs:27](crates/sifr_hir/src/lower/control_flow_conditions.rs:27)), representative fixture path matches the new fixture, message template uses `{keyword}` and `{actual}` placeholders, declared args list both as `MessageAndJson`, dedupe args list both. Inserted between the `SIFR-FLOW-0004` and `SIFR-FLOW-0901` entries so the registry stays lexicographically ordered.
- **Active list** ([codes.rs:1385](crates/sifr_diagnostics/src/codes.rs:1385)): `FLOW_INVALID_CONDITION_TYPE` inserted between `FLOW_MISSING_RETURN_VALUE` and `FLOW_UNREACHABLE_STATEMENT`, matching declaration order.
- **Helper** ([flow_diagnostics.rs:32‑37](crates/sifr_hir/src/lower/flow_diagnostics.rs:32)): plain wrapper around `ctx.error_with_code(DiagnosticCode::FLOW_INVALID_CONDITION_TYPE, …)`. Format string `"{keyword} condition must be bool or collection/string truthiness, got '{actual}'"` is byte-identical to the registry `message_template` after `{keyword}` and `{actual}` substitution. Helper signature mirrors the slice‑1 `missing_return_value` style (`&mut LowerCtx`, two `&str` operands).
- **Emission site** ([control_flow_conditions.rs:25‑28](crates/sifr_hir/src/lower/control_flow_conditions.rs:25)): the only previous `ctx.error(format!(...))` call in this module is fully replaced by `super::flow_diagnostics::invalid_condition_type(ctx, keyword, actual.as_str())`. The owned `String` returned by `Type::display_name()` is bound to `actual` so `.as_str()` is valid for the lifetime of the call. No parallel uncoded emission and no behavior change to the supported‑type matcher above it ([control_flow_conditions.rs:10‑24](crates/sifr_hir/src/lower/control_flow_conditions.rs:10)).
- **Caller invariants preserved.** Both callers — [statements.rs:1694](crates/sifr_hir/src/lower/statements.rs:1694) (`if`) and [statements.rs:1982](crates/sifr_hir/src/lower/statements.rs:1982) (`while`) — pass `"if"` / `"while"` literal keywords unchanged, so the formatted human text and the `{keyword}` placeholder substitution remain stable.
- **HIR regressions** ([expressions_tests.rs:674‑684](crates/sifr_hir/src/lower/expressions_tests.rs:674), [:686‑696](crates/sifr_hir/src/lower/expressions_tests.rs:686)): both assertions now require the message substring AND `e.code == Some(DiagnosticCode::FLOW_INVALID_CONDITION_TYPE)`. The bare‑import form matches the slice‑1 final convention (post pass‑2 nit fix). The neighboring `test_class_truthiness_allowed_in_if_while_and_boolop` ([:698](crates/sifr_hir/src/lower/expressions_tests.rs:698)) is correctly left as-is — it is a happy‑path test that asserts `result.is_ok()` and has no diagnostic to pin.

## Registry / docs consistency

- [docs/errors/SIFR-FLOW-0005.md](docs/errors/SIFR-FLOW-0005.md) is the generator-shaped page (banner reads `<!-- Generated by cargo run -p sifr_diagnostics --bin gen-error-docs. Do not edit by hand. -->`, all fields populated, no manual editing).
- The public index row ([docs/errors/diagnostic-codes.md:84](docs/errors/diagnostic-codes.md:84)) and internal index row ([internal_docs/diagnostic_codes.md:108](internal_docs/diagnostic_codes.md:108)) match the registry: id, family, state Active, severity Error, docs path, fixture path, owner module, message template, declared/dedupe args, no severity override, `fix_all_eligible=false`. Both rows are inserted between the `SIFR-FLOW-0004` and `SIFR-FLOW-0901` rows, mirroring the registry order.
- The inventory row ([internal_docs/diagnostic_emission_inventory.md:324](internal_docs/diagnostic_emission_inventory.md:324)) maps `SIFR-FLOW-0005` to "unsupported control-flow condition type / if/while condition validation / new fixture path", and lands in the correct family-grouped section between `SIFR-FLOW-0004` and `SIFR-MATCH-0001`.
- Registry-internal validators carry through: the `arg!` macro entries declare `MessageAndJson` format for both placeholders ([codes.rs:933](crates/sifr_diagnostics/src/codes.rs:933)) so the placeholder/arg parity check passes, and `dedupe_args` is a subset of `declared_args` ([codes.rs:934](crates/sifr_diagnostics/src/codes.rs:934)). Both invariants are runtime-asserted by `cargo test -p sifr_diagnostics`, which the user reported passing.

## Fixture validity

- [crates/sifr/tests/e2e/fail/if_condition_numeric_truthiness.sifr](crates/sifr/tests/e2e/fail/if_condition_numeric_truthiness.sifr) is minimal and on‑purpose:
  ```
  # expect-error: SIFR-FLOW-0005

  def main():
      if 1:
          pass
  ```
  This is the smallest function shape that exercises the migrated path: a numeric literal (`Type::LiteralInt(1)`, see [crates/sifr_type_system/src/types.rs:47](crates/sifr_type_system/src/types.rs:47)) reaches `validate_control_flow_condition` and fails the supported-type matcher (LiteralInt is not in the `Bool | LiteralBool | List | Set | Dict | Tuple | Str | Bytes | Class | Protocol | Any | Unknown` set, nor a None-bearing union). The `# expect-error: SIFR-FLOW-0005` marker is the canonical unqualified form per `validate_expected_error_code` ([e2e.rs:757](crates/sifr/tests/e2e.rs:757)) — bare canonical code, no message substring, registry state must be Active (it is).
- The user's CLI cross-check (`cargo run -q -p sifr -- check …`) reports exit 1 with the expected condition diagnostic text, which is consistent with the helper format and the `if`-keyword caller.
- Fixture path string in the registry exactly matches the on-disk path. No drift.

## Family choice

`FLOW` is the right family for this diagnostic. Three reinforcing signals:

1. The diagnostic is emitted strictly from the lowering of two control‑flow constructs (`if` / `while`). The check itself lives in `control_flow_conditions.rs`. It does not fire from comprehensions, attribute access, callable resolution, or anywhere outside CFG-shaped lowering.
2. The sibling pattern is already in place: match‑guard‑not‑bool is `SIFR-MATCH-0002` ([codes.rs:79](crates/sifr_diagnostics/src/codes.rs:79)), filed under `MATCH` even though it is also a "type of condition" error. Filing the if/while analogue under `FLOW` is the parallel decision and keeps the code‑family ↔ construct‑family mapping invertible.
3. There is no allocation conflict on `SIFR-FLOW-0005`: the only prior mention of that id is a non-binding brainstorming bullet in [reviews/semantic-diagnostic-code-taxonomy-diag-4a-flow-diagnostics-review-pass-1.md:80](reviews/semantic-diagnostic-code-taxonomy-diag-4a-flow-diagnostics-review-pass-1.md:80), which proposed splitting FLOW‑0003 sub-cases into per-cause codes and never landed. Slice 1 already used FLOW‑0004 outside that brainstormed split, so the `FLOW-000d` numbering is being driven by actual migration order, not by the abandoned proposal.

A `TYPE-*` allocation would have been a defensible alternative on "this is a type mismatch" grounds, but it would diverge from MATCH‑0002's family convention and would split a tightly bounded checker (`validate_control_flow_condition`) away from the construct that triggers it. Sticking with `FLOW` is the more consistent call.

## Fallback / compatibility check

No fallback or compatibility shim is introduced. Specifically:

- The helper does not gate the code on any feature flag, optional context, or "legacy text" branch — it unconditionally calls `ctx.error_with_code(DiagnosticCode::FLOW_INVALID_CONDITION_TYPE, …)`.
- The migrated emission site directly replaces `ctx.error(format!(...))` with the coded helper. There is no parallel uncoded emission of the same text anywhere in the workspace (`grep -rn "condition must be bool"` returns only the helper, the registry template, and the two test substring assertions).
- The supported-type matcher above the call ([control_flow_conditions.rs:10‑24](crates/sifr_hir/src/lower/control_flow_conditions.rs:10)) is unchanged — same accepted shapes, same `union_contains_none` allowance — so this slice does not silently widen or narrow what passes.
- The `LoweringError`-via-`ctx.error` legacy transport remains accessible for other (out-of-scope) call sites, but no new uncoded emission of the condition‑type text is added; the previous one is removed in the same change.

## Scope discipline

The slice touches exactly the surfaces described in the milestone scope:

- One new constant, one new registry entry, one active-list insertion.
- One new helper in the existing `flow_diagnostics.rs` module (no new file, consistent with slice 1's placement of `missing_return_value`).
- One emission-site rewrite, with the supported-type matcher untouched.
- Two HIR regression tests upgraded from message-only to message+code pinning.
- One generated docs page, three index/inventory rows, one e2e fixture, one issue-tracker bullet.
- No collateral edits to other diagnostic families, no unrelated reformatting, no churn in `statements.rs`. The two `validate_control_flow_condition` callers are deliberately unchanged.

## Issue tracker placement

The new bullet on [issues/…:89](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:89) ("`milestone_diag_8` slice 2 in progress: …") sits immediately after the slice‑1 reviewer-satisfied bullet, which matches the chronological "Execution Status" ordering. Wording correctly identifies this as the if/while condition migration targeting `SIFR-FLOW-0005`. After PR merge and review-satisfied confirmation, this bullet should be flipped to `[x] … implementation complete and reviewer-satisfied: … PR: …` and a paired `[x] Claude implementation review for milestone_diag_8 slice 2 …` bullet should be appended (mirroring the slice‑1 pair pattern at lines 87–88), with the final `report_signature`/`wall_time` from `scripts/run_all_tests.sh --profile quick` recorded. That follow-up is not a blocker for this review.

## Validation cross-check

The user reported the following local validation as passing:

- `cargo run -q -p sifr_diagnostics --bin gen-error-docs`
- `cargo fmt --check`
- `cargo test -p sifr_diagnostics`
- `cargo test -p sifr_hir condition_rejects_numeric_truthiness -- --nocapture` (covers both `test_if_condition_rejects_numeric_truthiness` and `test_while_condition_rejects_numeric_truthiness` via the shared substring)
- `cargo test -p sifr --test e2e test_e2e_fail -- if_condition_numeric_truthiness --nocapture` (243 fail tests; ok; pre-existing two CFG panic lines printed by the unrelated fail corpus)
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/if_condition_numeric_truthiness.sifr` (exits 1 with expected condition diagnostic text)
- `cargo clippy -p sifr_diagnostics -p sifr_hir --no-deps -- -D warnings`

I did not re-run them; the implementation is small, in-scope, and the listed validations cover the relevant gates: registry tests (template/declared-args/dedupe-args/markdown-safety), generated docs drift, the HIR regressions that pin both message and code for both `if` and `while`, the full e2e fail corpus including the new fixture, and the CLI smoke-check that confirms exit code and human text.

Per CLAUDE.md, `scripts/run_all_tests.sh --profile quick` is the authoritative pre-PR gate. That isn't listed in the validation summary and would be the recommended next step before opening the PR — same expectation as slice 1, where the quick profile produced the canonical `report_signature=e1bf653aaa770517` fingerprint that subsequent slice entries record. Not a review blocker.

## Findings

### Blockers

None.

### Required nits

None.

### Non-blocking observations (no action required for this slice)

1. **E2E coverage covers `if` only.** The e2e fixture exercises the `"if"` keyword path; the `"while"` path is exercised only via the HIR unit test `test_while_condition_rejects_numeric_truthiness`. This mirrors slice 1, where the missing-return fixture covers the single migrated branch. A `while_condition_numeric_truthiness.sifr` companion fixture would round out e2e coverage of the `keyword="while"` substitution but is not strictly required, since the `{keyword}` substitution is a literal string interpolation with no branching logic — a regression in the `while` formatter would manifest only as a copy/paste mistake in the caller, which the unit test catches. Adding a paired fixture would be a small, mechanical follow-up.
2. **Helper placement vs. nonlocal-wrapper grouping.** `invalid_condition_type` is inserted between `missing_return_value` (FLOW‑0004) and the FLOW‑0003 nonlocal wrapper functions (`nonlocal_requires_enclosing_binding`, …). This continues the convention slice 1 established when it placed `missing_return_value` before the nonlocal wrappers. The trade-off is "code-numeric ordering of top-level emit helpers" vs. "physical co-location of FLOW‑0003 wrappers near the `invalid_nonlocal` parent". Slice 2 stays consistent with slice 1; flipping conventions later would require touching all three placements. No action.
3. **Span attachment.** The new helper does not attach `LoweringError.line`/`col`. Same as the existing pre-migration code path and same as the other FLOW helpers, so this is not a regression. Span attachment for HIR-emitted diagnostics is a separate cross-cutting gap.
4. **Untracked verification snapshots.** [verification/leetcode/](verification/leetcode/) is untracked and may contain stale `"if/while condition must be bool…"` substrings keyed without `SIFR-FLOW-0005`. Out of scope for this slice; if those snapshots are committed in a later slice, they should be re-keyed to `SIFR-FLOW-0005`.

## Verdict

**Satisfied — clear to PR.** The slice is correctly scoped, registry/docs are internally consistent, the helper format string matches the registry template byte-for-byte after substitution, the `if` and `while` HIR regressions now pin both the message and the structured code, the e2e fixture exercises exactly the migrated path, and no fallback or compatibility path was introduced. `FLOW` is the right family and `SIFR-FLOW-0005` is an unallocated, in-sequence id. Recommended next step before opening the PR: run `scripts/run_all_tests.sh --profile quick` so the slice's reviewer-satisfied entry can record the standard `report_signature`/`wall_time` fingerprint, mirroring the slice‑1 entry on [issues/…:88](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:88).
