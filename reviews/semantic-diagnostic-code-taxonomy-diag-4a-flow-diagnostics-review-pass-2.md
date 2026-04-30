# `milestone_diag_4a` slice 2b.16 — flow diagnostics migration — review pass 2

Branch: `codex/semantic-diagnostics-diag-4a-flow-diagnostics`
Predecessor review: [reviews/semantic-diagnostic-code-taxonomy-diag-4a-flow-diagnostics-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-flow-diagnostics-review-pass-1.md)
Tracker: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)

## Verdict

**Reviewer satisfied / approved.** No blockers. Pass 1's three cosmetic registry/docs notes have been closed cleanly; the remaining pass 1 follow-ups (FLOW-0003 sub-case e2e fixtures and span attachment) were already classified as out of scope and remain non-blocking. The slice is ready to land.

## Pass 1 follow-ups — confirmed addressed

### Note 1: FLOW-0001 / FLOW-0002 template quoting

Template strings now match the runtime emission verbatim.

- [crates/sifr_diagnostics/src/codes.rs:877](crates/sifr_diagnostics/src/codes.rs:877) — `'break' outside of loop` (was `break outside of loop`)
- [crates/sifr_diagnostics/src/codes.rs:888](crates/sifr_diagnostics/src/codes.rs:888) — `'continue' outside of loop` (was `continue outside of loop`)
- Helper bodies in [crates/sifr_hir/src/lower/flow_diagnostics.rs:8](crates/sifr_hir/src/lower/flow_diagnostics.rs:8) and [:15](crates/sifr_hir/src/lower/flow_diagnostics.rs:15) emit those exact strings.
- Catalog rows in [internal_docs/diagnostic_codes.md:105-106](internal_docs/diagnostic_codes.md:105) and the per-code pages [docs/errors/SIFR-FLOW-0001.md:13](docs/errors/SIFR-FLOW-0001.md:13) / [docs/errors/SIFR-FLOW-0002.md:13](docs/errors/SIFR-FLOW-0002.md:13) are regenerated and consistent.

### Note 2: FLOW-0003 template/arg mismatch

Replaced the stale generic `invalid nested function flow: {reason}` template with a real emission shape, swapping the unused `reason` arg for the actually-populated `function` arg.

- [crates/sifr_diagnostics/src/codes.rs:899](crates/sifr_diagnostics/src/codes.rs:899) — template is now `recursive nested function '{function}' cannot mutate captured state with nonlocal yet`.
- [crates/sifr_diagnostics/src/codes.rs:900-901](crates/sifr_diagnostics/src/codes.rs:900) — `declared_args = [arg!("function")]`, `dedupe_args = ["function"]`.
- The placeholder validator at [crates/sifr_diagnostics/src/codes.rs:1606-1627](crates/sifr_diagnostics/src/codes.rs:1606) accepts the new template since `function` is declared as `MessageAndJson` — `cargo test -p sifr_diagnostics` passes 31/31 locally.
- The representative fixture [crates/sifr/tests/e2e/fail/nested_function_recursive_nonlocal_unsupported.sifr](crates/sifr/tests/e2e/fail/nested_function_recursive_nonlocal_unsupported.sifr) drives that exact shape, so the docs no longer advertise a never-emitted message.

#### Backtick caveat — intentional, documented

Note that the runtime helper [crates/sifr_hir/src/lower/flow_diagnostics.rs:60-67](crates/sifr_hir/src/lower/flow_diagnostics.rs:60) emits with backticks around `\`nonlocal\``, while the registry template stores it as plain `nonlocal`. This is forced by the registry's "no backticks in registry strings" guard at [crates/sifr_diagnostics/src/codes.rs:1630-1666](crates/sifr_diagnostics/src/codes.rs:1630), which would assert if the template kept the markdown-fenced form. Pass 2 description correctly calls this out as the registry-allowed form. Worth leaving a sentence in the FLOW-0003 doc explaining the divergence to anyone copy-pasting from the docs into a regex test, but it is a strict downstream-of-this-PR docs task and not a blocker.

### Docs regeneration

Regenerated locally to confirm zero drift:

```
cargo run -q -p sifr_diagnostics --bin gen-error-docs
git status -s docs/errors internal_docs/diagnostic_codes.md
```

Produces no additional changes beyond the four files already in the working tree — the docs are a fixed point of the regenerator on the current registry state.

## Validation re-confirmation

Locally re-ran the gates from the PR description against the current working tree:

- `cargo run -q -p sifr_diagnostics --bin gen-error-docs` — clean, no drift after.
- `cargo fmt --check` — clean.
- `python3 scripts/check_diagnostic_docs_sync.py` — clean.
- `python3 scripts/check_diagnostic_schema_sync.py` — clean.
- `cargo test -p sifr_diagnostics` — 31 passed.
- `python3 scripts/check_hir_maintainability_guardrails.py` — `HIR maintainability guardrails: PASS`.
- `cargo test -p sifr_hir test_break_outside_loop` — pass.
- `cargo test -p sifr_hir test_continue_outside_loop` — pass.
- `cargo test -p sifr_hir nonlocal` — 7/7 pass, including the three new code-equality tests (`test_top_level_nonlocal_requires_enclosing_binding_code`, `test_unresolved_nonlocal_has_flow_code`, `test_nonlocal_current_binding_conflict_has_flow_code`) at [crates/sifr_hir/src/lower/nested_function_tests.rs:163-198](crates/sifr_hir/src/lower/nested_function_tests.rs:163).
- `cargo test -p sifr_hir diagnostic_transport_tests` — 2 passed.
- `cargo test -p sifr --test e2e -- test_e2e_fail` — 1/1 pass (matches re-keyed FLOW fixtures end-to-end through `code == expected.code` at [crates/sifr/tests/e2e.rs:2561](crates/sifr/tests/e2e.rs:2561)).
- `cargo clippy --workspace -- -D warnings` — clean.

Pass 1's `scripts/run_all_tests.sh --profile quick` PASS at `report_signature=e1bf653aaa770517` is unchanged — pass 2 only edits registry strings, regenerated docs, and minor test guards, none of which affect the report signature inputs.

## Status of pass 1 non-blocking notes that are NOT closed

These were already accepted as out of scope; flagging here only so the audit trail is complete.

3. **E2E coverage of FLOW-0003 sub-cases is still limited to the recursive-helper fixture.** Five of six FLOW-0003 paths still rely on HIR-only assertions (now strengthened with structured-code checks, but no fixture). Recommend a small follow-up slice (one fixture per sub-case) before SIFR-TYPE-0001 is removed.

4. **`LoweringError.line/col` remain `None` for flow helpers.** Same as pre-migration. Span attachment for HIR-emitted diagnostics is the obvious next gap, separate from the code-taxonomy work.

## Scope check

Branch is rebased atop `main` with 21 commits; the only unstaged changes are the slice files. No unrelated edits crept in. Tracker entry [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:50-51](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:50) cleanly closes 2b.15 and opens 2b.16 (PR `pending`).

## Recommendation

Land this slice. The two pass 1 notes that remain open are explicitly out of the slice scope and have natural follow-ups (per-sub-case fixtures, span attachment) tracked elsewhere.
