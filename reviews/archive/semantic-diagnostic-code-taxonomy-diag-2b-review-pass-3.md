# Review: milestone_diag_2b — Diagnostic Registry Population (Pass 3)

Branch: `codex/semantic-diagnostics-diag-2b`
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Inventory: [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md)
Prior reviews: [reviews/semantic-diagnostic-code-taxonomy-diag-2b-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-2b-review-pass-1.md), [reviews/semantic-diagnostic-code-taxonomy-diag-2b-review-pass-2.md](semantic-diagnostic-code-taxonomy-diag-2b-review-pass-2.md)
Validation evidence reported: `python3 scripts/check_diagnostic_docs_sync.py`, `cargo test -p sifr_diagnostics --lib`, full `scripts/run_all_tests.sh --profile quick` carry-over from pre-edit state with report signature `e1bf653aaa770517`.

## Verdict

**Approve.** The single residual should-fix from pass 2 (R1 — issue line 927 still describing `SIFR-PARSE-0001` as "reserved opaque-parser-error") is resolved, and registry/docs/inventory consistency holds. No new blocking or should-fix items.

I re-ran the local consistency gates after the edit:

- `python3 scripts/check_diagnostic_docs_sync.py` — clean.
- `python3 scripts/check_diagnostic_schema_sync.py` — clean.
- `cargo run -q -p sifr_diagnostics --bin gen-error-docs -- --check` — clean, no drift.
- `cargo test -p sifr_diagnostics --lib` — 27 passed, 0 failed.

The change between pass 2 and pass 3 is a single-line documentation edit inside the `milestone_diag_7` scope block of the issue file; it touches nothing the registry, generator, or guardrails enforce, so the prior `--profile quick` signature (`e1bf653aaa770517`) remains representative.

## Pass-2 R1 verification

[issues/…-diagnostics.md:927](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:927) now reads:

> Keep `SIFR-PARSE-0001` retired; parser diagnostics use category-specific `SIFR-PARSE-0002..0009` codes and a `parser_category` JSON arg for upstream recovery context.

This matches the suggested edit from pass-2 R1 in substance: it drops the "reserved opaque-parser-error" framing and aligns with the registry's retired entry plus the `parser_category` JSON arg already declared on `SIFR-PARSE-0002..0009`. Verified end-to-end:

- `rg "reserved opaque-parser"` across `issues/`, `docs/`, `internal_docs/`, `crates/` — zero hits.
- `rg "opaque parser|opaque-parser"` returns only the (correct) retired-state phrasing:
  - [issues/…-diagnostics.md:214](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:214) — "Retired as the legacy opaque parser phase bucket."
  - [crates/sifr_diagnostics/src/codes.rs:395](../crates/sifr_diagnostics/src/codes.rs:395) — `retired_entry!` description "Retired opaque parser phase bucket."
  - [docs/errors/diagnostic-codes.md:141](../docs/errors/diagnostic-codes.md:141) — "Retired opaque parser phase bucket." in the public Retired Codes table.
- `rg "SIFR-PARSE-0001"` returns no remaining "reserved" wording. The remaining hits split cleanly into:
  - registry retired entry and replacement note ([codes.rs:393-397](../crates/sifr_diagnostics/src/codes.rs:393)),
  - public/internal docs Retired Codes rows,
  - issue file (lines 214, 927, 1201) — all retired-aligned,
  - inventory historical-state notes (lines 49, 53, 89, 111, 118, 137, 144) — describe the legacy mapping that `milestone_diag_4a`/`6` will remove,
  - legacy driver code/tests still emitting it via `CompilePhase::Parse` ([crates/sifr_driver/src/diagnostics.rs:135](../crates/sifr_driver/src/diagnostics.rs:135), [crates/sifr_driver/src/tests/diagnostics.rs:13](../crates/sifr_driver/src/tests/diagnostics.rs:13), [crates/sifr/src/main.rs:1334](../crates/sifr/src/main.rs:1334), [crates/sifr/tests/e2e.rs:2729](../crates/sifr/tests/e2e.rs:2729)) — all explicitly scheduled for `milestone_diag_4a`/`5`/`6` and out of scope here.

The registry's `SIFR-PARSE-0001` retired entry ([codes.rs:392-397](../crates/sifr_diagnostics/src/codes.rs:392)) carries description "Retired opaque parser phase bucket." with replacement "replaced by active PARSE category codes", and the active replacements `SIFR-PARSE-0002..0009` ([codes.rs:417-447](../crates/sifr_diagnostics/src/codes.rs:417) and following) all declare `json_arg!("parser_category")`. Issue line 927 is now an accurate statement of registry truth.

The three issue contradiction sites originally identified (table line 214, scope line 927, hard rule line 1201) are now mutually consistent and consistent with the registry, the generator-rendered Retired Codes tables, and the inventory.

## Registry/docs consistency spot-check

- `git diff --stat` confirms only six files touched on the branch, with no surprise modifications since pass 2: [crates/sifr_diagnostics/src/bin/gen-error-docs.rs](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs), [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs), [docs/errors/diagnostic-codes.md](../docs/errors/diagnostic-codes.md), [internal_docs/diagnostic_codes.md](../internal_docs/diagnostic_codes.md), [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md), and the issue file.
- The pass-2 invariants still hold: the registry-skeleton test enforces fixture-presence for every Active entry and bidirectional set-equality between `ACTIVE_DIAGNOSTIC_CODES` and active registry IDs; the docs-page existence-with-exact-casing invariant enforces 78 `docs/errors/SIFR-*.md` files matching the 78 Active entries; the markdown-safety guardrail covers the `replacement` field.
- `gen-error-docs --check` returns clean — generated `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` are still in sync with the registry after the issue-only edit.

## Findings

### Blocking

None.

### Should-fix in this PR

None. Pass-2 R1 is resolved; no new should-fix surfaced.

### Non-blocking carry-over (unchanged from pass 2)

- **R2** (pass-2): `SIFR-INTERNAL-0002` `owner_module` is still descriptive prose ([codes.rs:1291](../crates/sifr_diagnostics/src/codes.rs:1291)). Reserved entries are not subject to the active-entry guardrail; folds naturally into `milestone_diag_10` activation.
- **R3** (pass-2 / pass-1 N4): two-arg/dedupe sweep for `SIFR-DECIMAL-0004` and `SIFR-RESULT-0001`, deferred to `milestone_diag_8` / `milestone_diag_10`.

Both are restated only so the downstream-milestone reviewers see the trail; neither blocks `milestone_diag_2b`.

## Summary

Pass 3 is narrowly the pass-2 residual line-927 fix. The edit lands the same retired/category-codes phrasing already used at issue line 214, removes the last "reserved opaque-parser-error" wording from the repository, and leaves the registry, generator, public docs index, internal registry table, inventory, and validation gates internally consistent. **Approve to merge `milestone_diag_2b`.**
