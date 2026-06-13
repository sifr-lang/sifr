# milestone_diag_6 slice 1 review (pass 1)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-next-from-rendered` against `main`, against the milestone_diag_6 slice 1 contract in [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md).

Slice intent (from the issue, lines 1030–1044): convert decimal pseudo-codes to top-level `SIFR-DECIMAL-000x` codes, drop `[E25xx]` from messages, update decimal e2e fixtures and verification baselines in this milestone. The harness is allowed to keep accepting `[Edddd]` until milestone_diag_5.

## What the slice changes

- HIR decimal lowering ([crates/sifr_hir/src/lower/decimal_methods.rs](crates/sifr_hir/src/lower/decimal_methods.rs)): drops `[E2501]`/`[E2502]`/`[E2505]`/`[E2506]`/`[E2507]`/`[E2508]` from 13 message strings; deletes the now-unused `decimal_diag_code` helper. `DiagnosticCode::DECIMAL_*` codes remain at every emission site via `ctx.error_with_code(...)`.
- Float-conversion guard in `lower_call` ([crates/sifr_hir/src/lower/expressions.rs:1001-1009](crates/sifr_hir/src/lower/expressions.rs:1001)): drops `[E2505]`/`[E2506]` from `float(decimal_value)` and `float(bigdecimal_value)` rejection messages; codes preserved.
- Type-system mixed-arithmetic / mixed-comparison checks ([crates/sifr_type_system/src/check.rs:33,46,372,385](crates/sifr_type_system/src/check.rs:33)): drops `[E2503]`/`[E2504]` from four `TypeError.message` fields; `code: Some(DiagnosticCode::DECIMAL_*)` preserved on every site.
- E2E fail fixtures (15 files under [crates/sifr/tests/e2e/fail/](crates/sifr/tests/e2e/fail/)): rewrites `# expect-error: [SIFR-DECIMAL-000X] [E25XX] <msg>` to `# expect-error: SIFR-DECIMAL-000X: <msg>`.
- Verification baselines for the decimal_invalid_literal fixture ([crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-{compact,human,json}.stderr.txt](crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/)): drops `[E2501]` from the rendered message body; top-level code (`SIFR-DECIMAL-0001`) and URL unchanged.
- Inventory note ([internal_docs/diagnostic_emission_inventory.md:136-138](internal_docs/diagnostic_emission_inventory.md:136)): rewrites the E2E-expectation-surface paragraph to say decimal messages and fixtures stop carrying `[E25xx]` in milestone_diag_6 while the harness still accepts `[E2507]` until milestone_diag_5.
- Issue status ([issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:11,73](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:11)): wave bumped to milestone_diag_6; "in progress" entry added for slice 1.

## Verification of the slice contract

- `No decimal diagnostic message embeds [E25xx]`: verified. `grep -rn "\[E25" crates/sifr_hir/ crates/sifr_type_system/` is empty after the change. The remaining `[E2507]` hits in [crates/sifr/tests/e2e.rs:2730,2744,2759,2761](crates/sifr/tests/e2e.rs:2730) are harness self-tests that exercise the legacy expectation grammar — explicitly in scope only for milestone_diag_5.
- `No decimal diagnostic emits SIFR-TYPE-0001`: still holds (no regression from this slice — that was already the post-milestone_diag_4a slice 2b.1 state).
- `Decimal e2e fixtures and verification baselines updated in this milestone`: all 15 fail fixtures and the single decimal verification fixture's three stderr baselines are updated. Exit-code and stdout baselines correctly untouched (no message text in them).
- `Existing decimal negative cases preserve message clarity`: spot-checked all 13 messages — all read sensibly without the bracketed prefix (e.g. `decimal.round() scale must be between 0 and 28, got 29`, `cannot mix 'decimal' and 'bigdecimal' in arithmetic; …`). No information loss; the actionable hints (`use Decimal("...") for exact construction`, `use explicit Decimal(...) or BigDecimal(...) conversion`) are intact.
- Top-level `SIFR-DECIMAL-*` identity preservation: each migrated emission site still passes a `DiagnosticCode::DECIMAL_*` constant (`DECIMAL_INVALID_LITERAL`, `DECIMAL_BIGDECIMAL_INVALID_LITERAL`, `DECIMAL_FLOAT_MIXED`, `DECIMAL_MIXED_WITH_BIGDECIMAL`, `DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN`, `DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN`, `DECIMAL_SCALE_INVALID`, `DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID`). All eight constants exist in [crates/sifr_diagnostics/src/codes.rs:44-54](crates/sifr_diagnostics/src/codes.rs:44).
- Harness compatibility with the new fixture format: traced `parse_expected_error` ([crates/sifr/tests/e2e.rs:595](crates/sifr/tests/e2e.rs:595)) by hand. For input `SIFR-DECIMAL-0001: Decimal() received invalid exact literal '12.34.56'`, the parser splits on the first `:`, returns code `SIFR-DECIMAL-0001` and `message_contains` set to the trailing message; `is_diagnostic_code` accepts the code. Match logic at [crates/sifr/tests/e2e.rs:2560-2566](crates/sifr/tests/e2e.rs:2560) does substring containment, which the new emitted message satisfies.
- No compatibility fallback was added. The `decimal_diag_code` helper was simply deleted; there is no shim retaining the bracketed prefix. ✓

## Findings

### 1. Stale demo expectation now references a code that the diagnostic no longer emits

[demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr:3](demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr:3) still says:

```
# expect-error: [E2505] Decimal(float_value) is not allowed; use Decimal("...") for exact construction
```

If this fixture were ever run through the e2e harness, `parse_expected_error` would set `code = "E2505"` (the legacy `is_message_error_code` path), then the fail-matcher at [crates/sifr/tests/e2e.rs:2561](crates/sifr/tests/e2e.rs:2561) would compare `failure.code == "E2505"` — but the diagnostic now emits `SIFR-DECIMAL-0005`. The expectation would no longer match.

Today this demo is not auto-validated (no script under `scripts/` and no test under `crates/` references it; `grep -rn "demos/decimal_types\|forbidden_float_constructor" crates/ scripts/` is empty), so the slice's local validation passes. But the file is now factually inconsistent with the diagnostic surface, and the inventory paragraph that the slice rewrote ([internal_docs/diagnostic_emission_inventory.md:136-138](internal_docs/diagnostic_emission_inventory.md:136)) says decimal "fail-fixture expectations stop carrying `[E25xx]` pseudo-code text in `milestone_diag_6`". This demo is the only `expect-error` annotation under `demos/` (`grep -rln "expect-error" demos/` returns just this file), and it's still carrying `[E2505]`. Either include it in this slice (one-line edit, mirrors the e2e fixture form) or carve out an explicit "demos are out of scope here" note in the issue / inventory so the contradiction is intentional.

### 2. Inventory `Verification Baseline Surface` row is now stale

[internal_docs/diagnostic_emission_inventory.md:262](internal_docs/diagnostic_emission_inventory.md:262) still says:

```
| `diagnostics/decimal_invalid_literal` | `SIFR-TYPE-0001` plus message-embedded `[E2501]` in compact/json/human output | `SIFR-DECIMAL-0001`; regenerate in decimal migration and renderer integration |
```

Both clauses on the left are no longer true: the baseline already emits `SIFR-DECIMAL-0001` (since milestone_diag_4a slice 2b.1) and after this slice it no longer carries `[E2501]`. The migration described on the right is the work this slice just finished. The slice updated the E2E-expectation paragraph above (line 136) but missed this row. Either drop the row, mark it "Done in milestone_diag_6 slice 1", or rewrite it to describe the now-current state. Leaving it as a "Current baseline markers" row is misleading for future readers.

### 3. Demo string literal claims a now-retired diagnostic range

[demos/decimal_diagnostics/main.sifr:14](demos/decimal_diagnostics/main.sifr:14) prints:

```
"diagnostic range E2501-E2508 is reserved and enforced"
```

The matching `idiomatic.rs` / `emitted.rs` siblings ([demos/decimal_diagnostics/idiomatic.rs:34](demos/decimal_diagnostics/idiomatic.rs:34), [demos/decimal_diagnostics/emitted.rs:19](demos/decimal_diagnostics/emitted.rs:19)) print the same line. After this slice the codes have moved to `SIFR-DECIMAL-0001..0008` and there is no `E25xx` range reserved anywhere except in archive/historical phase docs. The demo's positive output is now a falsehood.

This is not a correctness regression (the demo's stdout baseline, if any, will still match the literal), but it is a stale user-facing surface that the slice plausibly should have refreshed. At minimum, flag it as deferred so milestone_diag_6 doesn't claim the decimal pseudo-code surface is fully retired while a demo still announces it.

### 4. Stale `[E25xx]` mentions in demo doc comments

Three doc comments still reference the retired pseudo-codes:

- [demos/decimal_diagnostics/negative_cases/decimal_round_scale_out_of_range/idiomatic.rs:7](demos/decimal_diagnostics/negative_cases/decimal_round_scale_out_of_range/idiomatic.rs:7) — "deterministic `[E2507]` error"
- [demos/decimal_verification/negative_cases/forbidden_mixed_arithmetic/idiomatic.rs:7](demos/decimal_verification/negative_cases/forbidden_mixed_arithmetic/idiomatic.rs:7) — "deterministic `[E2504]` diagnostic"
- [demos/decimal_verification/negative_cases/forbidden_float_conversion/idiomatic.rs:7](demos/decimal_verification/negative_cases/forbidden_float_conversion/idiomatic.rs:7) — "deterministic `[E2505]` diagnostic"

Pure documentation drift — does not affect correctness or any test — but they directly contradict the slice's stated objective. Consider sweeping in this pass or filing as the next slice's residual cleanup.

### 5. Phase document still reserves the `E25xx` range as authoritative

[internal_docs/phases/28_decimal_type_and_exact_numeric_semantics.md:187-196](internal_docs/phases/28_decimal_type_and_exact_numeric_semantics.md:187) still lists `E2501-E2508` as the reserved diagnostic range with one-line meanings. The canonical registry in [crates/sifr_diagnostics/src/codes.rs](crates/sifr_diagnostics/src/codes.rs) has superseded that claim, but the phase doc reads as a standing requirement. If phase docs are treated as historical/archival, an inline note pointing at `SIFR-DECIMAL-*` would prevent confusion. If they are treated as live, this needs an actual rewrite. This was not in the slice's scope as written, but it is the same surface category the slice claims to clean up; worth at least a forward-pointer.

### 6. No guardrail asserting `[E25` is gone from the decimal emission surface

The verification baseline for `decimal_invalid_literal` does pin the rendered text exactly, so `SIFR-DECIMAL-0001` is regression-protected against re-introducing `[E2501]`. The remaining seven decimal codes (`-0002..-0008`) have no analogous exact-text baseline — they are checked only by e2e fail fixtures, which use substring matching that would tolerate a re-introduced `[E25xx]` prefix.

A single unit test along the lines of "for every decimal e2e fail fixture, run the compiler and assert no rendered diagnostic message contains `"[E25"`" would close that gap cheaply. Given the slice is named after this exact removal, a regression-prevention test seems appropriate. Optional but recommended for the next pass.

### 7. Status-entry phrasing

Issue line 73 (the new entry) lists this as `[ ] in progress` whereas the prior shipped entries follow the pattern `[x] … implementation complete and reviewer-satisfied … PR: <url>`. That phrasing is correct for an unmerged slice, but means whoever flips the box on merge needs to remember to convert the wording too. Not a defect — a process pointer.

## What looks correct

- Top-level `SIFR-DECIMAL-*` identity is preserved everywhere; not a single decimal emission lost its `code: Some(...)` or its `error_with_code(DiagnosticCode::DECIMAL_*, …)` argument.
- The deleted `decimal_diag_code` helper has zero remaining callers (`grep -rn "decimal_diag_code" crates/` is empty); the surviving `decimal_scale_diagnostic_code` helper correctly still returns a `DiagnosticCode`, not a bracketed string.
- All 15 e2e fail fixtures use a uniform new format `SIFR-DECIMAL-000X: <msg>` — no fixture was missed and no fixture was rewritten with a different shape.
- Three baseline files (compact/human/json) for the one decimal verification fixture were all touched; the JSON `args.message` field was updated alongside the top-level `message` field, keeping the JSON shape internally consistent.
- The slice does not touch later milestone_diag_5 harness grammar — the harness still accepts `[Edddd]` and the harness self-tests still exercise that path, exactly as the issue requires.
- The slice introduces no compatibility fallback or shim. The pseudo-code prefixes are gone, not aliased.
- No spurious changes to the public API surface, no churn outside the decimal family, no unrelated formatting drift.

## Recommendation

The slice's core goal is met for the **emission** and **e2e/verification** surfaces. To close the slice cleanly I would either fold in or explicitly defer:

1. Update or remove [demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr:3](demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr:3). It is the only remaining `expect-error: [E25xx]` in the working tree outside the harness self-tests, and it directly contradicts the slice's inventory rewrite.
2. Update [internal_docs/diagnostic_emission_inventory.md:262](internal_docs/diagnostic_emission_inventory.md:262) so the `decimal_invalid_literal` row reflects post-migration state, matching the paragraph rewrite above it.
3. Decide whether the `demos/decimal_diagnostics/*` "diagnostic range E2501-E2508 is reserved" string and the three `idiomatic.rs` doc comments are in scope. Either fix them here or note the deferral in the slice status entry.
4. Optional but recommended: add a guardrail test asserting decimal diagnostic messages contain no `"[E25"` substring, so the seven non-baselined decimal codes can't regress.

Items 1 and 2 are the smallest concrete loose ends that match the slice's stated objective; the remainder are optional polish. Validation evidence the user already collected (`cargo test -p sifr_type_system`, `cargo test -p sifr --test e2e test_e2e_fail`, `python3 scripts/run_verification_hardening.py --suite diagnostics`) covers the surfaces this slice changes — no additional cargo/test invocation is needed to clear pass 1.
