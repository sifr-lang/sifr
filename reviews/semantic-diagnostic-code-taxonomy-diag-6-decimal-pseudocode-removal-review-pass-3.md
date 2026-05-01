# milestone_diag_6 slice 1 review (pass 3)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-next-from-rendered` against `main`, focused on the doc-only delta added since [reviews/semantic-diagnostic-code-taxonomy-diag-6-decimal-pseudocode-removal-review-pass-2.md](semantic-diagnostic-code-taxonomy-diag-6-decimal-pseudocode-removal-review-pass-2.md). Slice intent unchanged: drop `[E25xx]` text from decimal diagnostics, fixtures, and verification baselines, while preserving the eight top-level `SIFR-DECIMAL-000x` codes.

## Status of pass-2 finding

| # | Finding | Pass-3 status | Evidence |
| --- | --- | --- | --- |
| 1 | Inventory `Current public-code mechanisms to remove` row at line 120 still described pre-slice state in both clauses (`keeps [E25xx] as text` and `inside a broader SIFR-TYPE-0001 diagnostic`) | **Resolved** | [internal_docs/diagnostic_emission_inventory.md:120](internal_docs/diagnostic_emission_inventory.md:120) now reads `removed in milestone_diag_6 slice 1; decimal diagnostics now carry top-level SIFR-DECIMAL-* codes with no secondary message code`, with the Replacement cell unchanged at `keep top-level SIFR-DECIMAL-* identity and no message-embedded pseudo-code` |
| 2 | Slice scope alignment — no other gaps (pass 2) | **Re-verified** | See "Re-verification" below |
| 3 | Guardrail surface area observations (pass 2) | **N/A** | Non-blocking observations from pass 2; no code change required |

## Verification of the pass-3 delta

### Diff of the doc-only patch

`git diff internal_docs/diagnostic_emission_inventory.md` shows a one-line change to the row, replacing only the "Current effect" cell. The Mechanism, Current owner, and Replacement cells are byte-identical to pass 2:

```
-| Message-embedded pseudo-code | decimal/type-system messages and fixture expectations | keeps `[E25xx]` as text inside a broader `SIFR-TYPE-0001` diagnostic | top-level `SIFR-DECIMAL-*` diagnostic code and no secondary message code |
+| Message-embedded pseudo-code | decimal/type-system messages and fixture expectations | removed in `milestone_diag_6` slice 1; decimal diagnostics now carry top-level `SIFR-DECIMAL-*` codes with no secondary message code | keep top-level `SIFR-DECIMAL-*` identity and no message-embedded pseudo-code |
```

This is the exact wording suggested in pass-2 finding 1 ([reviews/semantic-diagnostic-code-taxonomy-diag-6-decimal-pseudocode-removal-review-pass-2.md:80](reviews/semantic-diagnostic-code-taxonomy-diag-6-decimal-pseudocode-removal-review-pass-2.md:80)), with a stylistic substitution of "no secondary message code" for "no message-embedded pseudo-code". Both phrasings describe the same property; the patch's choice of "no secondary message code" mirrors the original Replacement-cell wording at this same row, which keeps the Current/Replacement columns linguistically aligned.

### Convention match with sibling rows

The two retired-mechanism rows that pass 2 cited as the convention to follow are unchanged:

- [internal_docs/diagnostic_emission_inventory.md:117](internal_docs/diagnostic_emission_inventory.md:117) — `removed; diagnostics now carry active code strings, and human labels are code-derived`.
- [internal_docs/diagnostic_emission_inventory.md:118](internal_docs/diagnostic_emission_inventory.md:118) — `removed before milestone_diag_4b; workspace identities are explicit at construction`.

The new line-120 cell starts with the same `removed [in <milestone>]; <postcondition>` shape, scopes the milestone to `milestone_diag_6` slice 1 (matching the actual change set), and states the postcondition in the same active voice. Convention is preserved.

### Non-stale-doc accuracy

Both clauses pass 2 flagged are now true:

- `removed in milestone_diag_6 slice 1` — verified by the slice's emission-site changes at [crates/sifr_hir/src/lower/decimal_methods.rs:12-18](crates/sifr_hir/src/lower/decimal_methods.rs:12), [crates/sifr_hir/src/lower/expressions.rs:998-1013](crates/sifr_hir/src/lower/expressions.rs:998), and [crates/sifr_type_system/src/check.rs:31-46,370-391](crates/sifr_type_system/src/check.rs:31), plus `grep -rn "\[E25" crates/sifr_hir/ crates/sifr_type_system/` returning empty.
- `top-level SIFR-DECIMAL-* codes with no secondary message code` — verified by `grep -h "expect-error: SIFR-DECIMAL" crates/sifr/tests/e2e/fail/*.sifr` returning 13 distinct lines (15 fixtures, two `SIFR-DECIMAL-0002` and two `SIFR-DECIMAL-0004` and two `SIFR-DECIMAL-0006` entries appearing more than once), each in canonical `SIFR-DECIMAL-000X: <msg>` form with no `[E25xx]` substring; and the verification baselines at [crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-compact.stderr.txt:2](crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-compact.stderr.txt:2), `check-human.stderr.txt:1`, and `check-json.stderr.txt` all carrying `SIFR-DECIMAL-0001` only, with no `[E2501]` text in the rendered message or the JSON `args.message.value` field.

### Re-verification of pass-2 broad-scope checks

I re-ran the same searches pass 2 used to confirm no other doc surface drifted with the pass-3 patch:

- `grep -rn "\[E25" crates/ demos/ docs/ internal_docs/ verification/ scripts/ issues/ --include="*.rs" --include="*.sifr" --include="*.md" --include="*.txt"` (excluding `reviews/`, `target/`, `.git/`, `issues/archive/`): 22 hits, all in scope:
  - `crates/sifr/tests/e2e.rs:2607,2759,2773,2788` — the new guardrail's own assertion text and harness grammar samples, deferred to `milestone_diag_5`.
  - `internal_docs/diagnostic_emission_inventory.md:110,136,144-151` — manual `RenderedDiagnostic` rows whose `[E2507]`-style content lives in CLI/driver test surfaces (deferred to `milestone_diag_5`), the harness-grammar paragraph that explicitly notes the deferral, and the historical migration ledger.
  - `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:73,225,273,344,371,1003,1035,1042` — the in-progress slice status entry, design/scope context, and required negative-test specifications. None of these are stale state descriptions; they are migration plan and historical context.
- `grep -rln "expect-error" demos/` returns only `demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr`, still using canonical `SIFR-DECIMAL-0005:` form.
- `grep -rn "\[E25" crates/sifr_hir/ crates/sifr_type_system/` returns empty — no regression in the message-text removal.

The 15 fail-fixture coverage table from pass 2 is unchanged; each `SIFR-DECIMAL-000X` code is still exercised by at least one fixture, and the new guardrail at [crates/sifr/tests/e2e.rs:2590-2616](crates/sifr/tests/e2e.rs:2590) still walks all `tests/e2e/fail/*.sifr`, filters to those containing `SIFR-DECIMAL-`, and asserts neither `failure.code.starts_with("E25")` nor `failure.message.contains("[E25")` for any of them.

### Validation matrix

The user re-ran the slice's full validation matrix and reports green on:

- `cargo test -p sifr_type_system` — exercises the four `check.rs` decimal sites.
- `cargo test -p sifr --test e2e test_e2e_fail` — exercises the 15 modified fail fixtures and the new guardrail.
- `python3 scripts/run_verification_hardening.py --suite diagnostics` — exercises the three `decimal_invalid_literal` baselines.
- `cargo test -p sifr_hir decimal` — exercises the lowering sites in `decimal_methods.rs` and `expressions.rs`.
- `cargo test -p sifr test_decimal_fail_fixtures_do_not_emit_legacy_pseudo_codes` — runs the new guardrail in isolation.

I executed the three post-doc-patch validators directly:

- `cargo fmt --check` — clean (no output).
- `git diff --check` — clean (no output).
- `cargo clippy --workspace -- -D warnings` — clean (`Finished dev profile [unoptimized + debuginfo] target(s) in 3m 30s`, no warnings) after `cargo clean -p sifr` to force a full re-check.

No new lint surface was introduced by the doc-only delta, and the previously-validated code surfaces are unchanged.

## Findings (pass 3)

None. Pass-2 finding 1 is resolved cleanly with no new defects.

## What looks correct

- The pass-3 patch is genuinely doc-only — `git diff --stat` shows only `internal_docs/diagnostic_emission_inventory.md` changed since pass 2, with a one-line table-cell edit. No code surface, fixture, baseline, demo, phase doc, or test was touched in this pass.
- The updated cell is consistent with the surrounding table's convention and accurately describes both the migration moment (`milestone_diag_6` slice 1) and the resulting postcondition (`top-level SIFR-DECIMAL-* codes with no secondary message code`).
- The Replacement column at line 120 now states a target that matches the current code state, which is the correct shape for a row whose Current effect is "removed". This mirrors how lines 117 and 118 are framed.
- All other surfaces touched earlier in this slice (15 fail fixtures, 3 verification baselines, 3 demo files plus 3 idiomatic siblings, Phase 28 doc, the new guardrail at `crates/sifr/tests/e2e.rs:2590-2616`, and the four HIR/type-system emission sites) remain in the state pass 2 verified.
- No further `[E25xx]`-mention drift in the in-scope code, demo, baseline, or live-state-description doc surfaces — every remaining grep hit is either harness grammar deferred to `milestone_diag_5`, an historical migration ledger, the in-progress status entry, or design/scope context in the issue file.
- Slice contract end-to-end: `[E25xx]` text removed from decimal diagnostics and fixtures, top-level `SIFR-DECIMAL-000x` codes preserved at every emission site, no compatibility shim added, and a regression guardrail in place.

## Recommendation

Pass 2's only finding is resolved with a minimal, conventional one-line patch. No new findings. No further validation runs needed beyond the matrix the user already executed plus the three doc-patch validators (`cargo fmt --check`, `git diff --check`, `cargo clippy --workspace -- -D warnings`), all of which I confirmed clean.

The slice is ready for PR. After merge, the residual cleanups for the harness-grammar `[E2507]` acceptance and the manual `RenderedDiagnostic` test surfaces remain owned by `milestone_diag_5`, as already documented at [internal_docs/diagnostic_emission_inventory.md:110-111,136](internal_docs/diagnostic_emission_inventory.md:110).
