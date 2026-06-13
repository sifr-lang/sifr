# Review: milestone_diag_2a — Diagnostic Registry Skeleton (Pass 2)

Branch: `codex/semantic-diagnostics-diag-2a`
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Prior review: [reviews/semantic-diagnostic-code-taxonomy-diag-2a-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-2a-review-pass-1.md)
Validation evidence reported since pass 1: `cargo test -p sifr_diagnostics`, `python3 scripts/check_diagnostic_docs_sync.py`, `cargo fmt --check`, `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings`.

## Scope reviewed

- Hygiene cleanup: `crates/sifr_diagnostics/src/codes/` empty leftover directory removed.
- Registry validation tightening in [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs):
  - JSON-only declared args must not appear as message-template placeholders ([codes.rs:475-497](../crates/sifr_diagnostics/src/codes.rs:475)).
  - Reserved family-base entries (`SIFR-<FAMILY>-0000`) must not declare severity ([codes.rs:355-365](../crates/sifr_diagnostics/src/codes.rs:355)).
  - Registry strings emitted as inline code in generated docs must not contain backticks ([codes.rs:499-534](../crates/sifr_diagnostics/src/codes.rs:499)).
- Drift-check tightening in [crates/sifr_diagnostics/src/bin/gen-error-docs.rs](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs):
  - `--check` now enumerates `docs/errors/*.md` and reports any markdown file that is not in the expected set as an "orphan generated diagnostic page" ([gen-error-docs.rs:102-139](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:102)).
- Generated outputs and surrounding wiring (model/render test redirects, `scripts/check_diagnostic_docs_sync.py`, `scripts/run_all_tests.sh`, phase tracker) re-checked end-to-end.

## Verdict

The registry skeleton now meets every facet of the `milestone_diag_2a` definition of done. All actionable pass-1 findings (F1, F2, F3, F4) are resolved or properly scoped. No blocking correctness defects remain. **Approve to proceed to PR.**

The only remaining concern is non-blocking: F5 (the `#[cfg(test)]` gating on `DiagnosticCode::new`) still applies and will need to flip together with the active-code population in `milestone_diag_2b`. That is correctly noted as a 2b boundary item rather than a 2a defect.

## Pass-1 findings — resolution status

| Pass-1 finding | Status | Evidence |
| --- | --- | --- |
| F1a — JSON-only arg declarations have no behavioral assertion | ✅ Resolved | `assert_template_placeholders_are_declared` now asserts the matching declaration's `format == MessageAndJson` for every placeholder, panicking with `"json-only arg {placeholder} must not appear in the message template for {id}"` ([codes.rs:490-496](../crates/sifr_diagnostics/src/codes.rs:490)). Vacuous today (zero `JsonOnly` declarations) but the framework is no longer empty. |
| F1b — Reserved family-base severity invariant | ✅ Resolved | Per-family loop now asserts `base.declared_severity == None` for every `SIFR-<FAMILY>-0000` entry ([codes.rs:360-364](../crates/sifr_diagnostics/src/codes.rs:360)). Cross-checked: `reserved_family_base()` helper sets `declared_severity: None` ([codes.rs:266-281](../crates/sifr_diagnostics/src/codes.rs:266)), and `SIFR-INTERNAL-0001`/`-0002` are non-base reserved entries (not iterated by this loop) so they correctly continue to declare severity. |
| F2 — Drift check has a blind spot for orphan files | ✅ Resolved | `check_active_doc_casing` now builds an expected-set of `{ "diagnostic-codes.md" } ∪ { "<id>.md" : id ∈ active_registry_entries() }` and reports any other `.md` file in `docs/errors/` as an orphan ([gen-error-docs.rs:118-138](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:118)). Extension match is case-insensitive; filename match is case-sensitive, so the orphan check also doubles as a casing-mismatch detector. |
| F3 — Empty leftover `crates/sifr_diagnostics/src/codes/` directory | ✅ Resolved | Directory is gone (`ls` shows only the file `codes.rs`); `git status` no longer references the deleted `codes/mod.rs` parent. |
| F4 — `escape_table` only escapes `|`, leaving inline-code spans vulnerable to backtick content | ✅ Resolved (option-a path) | `assert_registry_strings_are_markdown_safe` ([codes.rs:499-534](../crates/sifr_diagnostics/src/codes.rs:499)) asserts no backtick in `entry.id`, `entry.family`, `entry.docs_path`, `entry.summary`, `entry.owner_module`, `entry.message_template`, `entry.representative_fixture_path`, declared arg names, dedupe args, and `tooling.tool_actions`. This is the option-(a) path explicitly offered by pass 1 ("document and assert these registry strings must not contain backticks"). `escape_table` continues to handle pipes; backticks are forbidden by construction. |
| F5 — `DiagnosticCode::new` is `#[cfg(test)]` only | ⚠️ Carries over to 2b | Constructor remains test-gated ([codes.rs:17-23](../crates/sifr_diagnostics/src/codes.rs:17)); intentional for the skeleton phase, must flip with active-code population in 2b. Not a 2a defect. |

## DoD coverage — re-checked

| DoD bullet | Status | Notes vs. pass 1 |
| --- | --- | --- |
| Registry skeleton (families, per-family numbering, state machine, reserved bases) | ✅ | Unchanged. |
| `SIFR-INTERNAL-0001` / `SIFR-INTERNAL-0002` reserved with correct severities | ✅ | Unchanged; still `Reserved` with `Error`/`Note` declared severity. |
| Registry and code constants cannot silently diverge | ✅ | Bidirectional `BTreeSet` equality between `active_registry_entries()` ids and `ACTIVE_DIAGNOSTIC_CODES` ids ([codes.rs:367-385](../crates/sifr_diagnostics/src/codes.rs:367)). Unchanged. |
| Registry record shape | ✅ | Unchanged. |
| `DiagnosticCode::code()` is the only canonical accessor | ✅ | Unchanged. |
| Tooling metadata defaults documented | ✅ | Default prose is in both generated docs ([gen-error-docs.rs:146,202](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:146)). |
| Docs generator writes three outputs (`docs/errors/<CODE>.md`, `docs/errors/diagnostic-codes.md`, `internal_docs/diagnostic_codes.md`) | ✅ | Unchanged. |
| Build-time validation test checks (1) template ⊆ declared args, (2) **JSON-only arg declarations**, (3) docs-page presence for active codes, (4) constant/registry sync, (5) canonical code forms, (6) registry state validity, (7) **registry-declared severity constraints** | ✅ now fully covered | Pass 1 marked (2) and (7) as partial. Both are now covered: (2) by the `MessageAndJson`-only placeholder check, and (7) by the reserved-family-base no-severity check (in addition to the pre-existing "active entries declare severity" assertion). |
| Skeleton validation passes with zero active codes | ✅ | `cargo test -p sifr_diagnostics` reportedly passes, and the new assertions are vacuous on the current registry (no `JsonOnly` args, no backticks in any string, family bases have `None` severity). |
| Generator and drift detection wired into local validation | ✅ | `scripts/check_diagnostic_docs_sync.py` invokes `gen-error-docs --check` and is run unconditionally in `scripts/run_all_tests.sh` before `cargo test -p sifr_diagnostics` ([run_all_tests.sh:101-103](../scripts/run_all_tests.sh:101)). The `--check` path now also flags orphan pages, closing the F2 gap. |

The DoD's literal phrasing was "fail on drift with `git diff --exit-code`". The implementation continues to use an in-memory comparison instead, but pass 1 already accepted that as functionally equivalent (and arguably more robust, since it works in dirty trees and now also detects orphans that `git diff` of generated paths would also miss). Not a regression and not a defect.

## Findings

### Blocking

None.

### Non-blocking — observations on the new validations

#### N1. `assert_registry_strings_are_markdown_safe` does not check pipes in non-backtick-wrapped values

`entry.summary` is emitted as raw text in markdown table cells in both `public_index` and `internal_reference` ([gen-error-docs.rs:152-155, 184-186](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:152)) without going through `escape_table`. If a future summary string ever contained `|`, the table cell would split. The new assertion does not cover this case (it only forbids backticks).

`entry.id`, `entry.family`, and `entry.docs_path` in the internal-reference registry table are emitted as backtick-wrapped raw values without `escape_table` ([gen-error-docs.rs:218-224](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:218)); a `|` inside an inline-code span still breaks markdown table layout in most renderers. Vacuous today: the canonical code form (`SIFR-<FAMILY>-NNNN`) and family-name regex (uppercase ASCII letters) make pipes structurally impossible for `id` and `family`, and `docs_path` is currently a constant. So the only meaningful exposure is `entry.summary`.

This is non-blocking and arguably out of scope for 2a, since no current registry string contains a pipe and the milestone is explicit that active-code rigor lands in 2b. Worth flagging for the 2b boundary alongside F5: when adding the first active codes whose summaries are author-controlled prose, also assert "no `|` in `summary`" (or pipe-escape it in the renderer).

#### N2. The orphan-page detection is correct, but the cargo-test side has overlap with the binary-side check

`active_diagnostic_docs_pages_exist_with_exact_casing` ([codes.rs:388-416](../crates/sifr_diagnostics/src/codes.rs:388)) and `gen-error-docs --check`'s `check_active_doc_casing` ([gen-error-docs.rs:102-139](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:102)) both enumerate `docs/errors/` to assert active-code docs exist with exact casing. The cargo test does not check orphans; the binary `--check` does both presence and orphans. The two paths are complementary (different consumers run them), but they re-implement directory enumeration and BTreeSet construction.

This is purely a style / minor-duplication observation, not a defect — both paths are correct. If consolidation is desired later, lifting the shared logic into `crates/sifr_diagnostics/src/codes.rs` and calling it from both sites would eliminate the duplication. Not worth doing in this PR; flag it for cleanup once the binary's `--check` mode is the established source of truth.

#### N3. F5 carries forward unchanged

`DiagnosticCode::new` remains `#[cfg(test)]`-gated. The skeleton phase has no production constructor and `ACTIVE_DIAGNOSTIC_CODES` is empty, so this is correct today and will need to be ungated (or replaced by an internal const constructor) at the same time the first active entries are added. Per pass 1: "load-bearing detail to remember at the 2b boundary, not a defect". Reaffirmed.

## Spot checks

- **Pass-1 hygiene cleanup landed cleanly.** `crates/sifr_diagnostics/src/codes/` is gone. `git status` shows the deleted `codes/mod.rs` and the new `codes.rs` sibling, but the parent directory itself no longer exists on disk. ✅
- **No external consumers regressed.** Only test-only files (`crates/sifr_diagnostics/src/model/mod.rs`, `crates/sifr_diagnostics/src/render/mod.rs`) reference `DiagnosticCode::*` and they all use the `TEST_*` constants. `grep` across the repo confirms no production code references any registry symbol. ✅
- **Severity constants stay aligned.** `SIFR-INTERNAL-0001` declares `Error` ([codes.rs:234](../crates/sifr_diagnostics/src/codes.rs:234)), `SIFR-INTERNAL-0002` declares `Note` ([codes.rs:248](../crates/sifr_diagnostics/src/codes.rs:248)); both remain `Reserved`. The new "reserved family base must not declare severity" assertion targets `SIFR-<FAMILY>-0000` entries specifically and does not regress these `0001/0002` reservations (they are not family bases). ✅
- **Generated docs are consistent with registry.** Spot-comparing [docs/errors/diagnostic-codes.md](../docs/errors/diagnostic-codes.md) and [internal_docs/diagnostic_codes.md](../internal_docs/diagnostic_codes.md) against the registry: 17 family rows, 17 reserved bases, 2 reserved non-base entries (`-0001`, `-0002`), zero active entries, severities and owners match the registry source. ✅
- **`docs/errors/` contains exactly the expected set.** `ls docs/errors/` shows only `diagnostic-codes.md`. With zero active entries the orphan check's expected set is `{ "diagnostic-codes.md" }`, so `--check` passes. ✅
- **Drift-check wiring runs in every lane.** `check_diagnostic_docs_sync.py` is invoked unconditionally in `scripts/run_all_tests.sh` before `cargo test -p sifr_diagnostics`, which means a docs-drift or orphan regression fails fast in `quick`, `pr`, `nightly`, and `release` profiles. ✅
- **Phase tracker reflects 2a state correctly.** Issue file lists registry-skeleton + docs-validation as completed, leaves "Claude review for 2a" and "PR opened and merged" unchecked, cites validation report signature `e1bf653aaa770517`, and updates the wave label to `milestone_diag_2a`. ✅
- **JSON schema still unaffected.** `RenderedDiagnostic.code` remains a plain string in the schema; the registry tightening does not perturb the wire format, and `check_diagnostic_schema_sync.py` continues to be a no-op for this milestone. ✅

## Suggested next actions

1. (None blocking.) Open the PR.
2. (For 2b.) When active codes are populated:
   - Ungate `DiagnosticCode::new` (F5) at the same boundary.
   - Tighten `assert_registry_strings_are_markdown_safe` to also forbid `|` in `entry.summary` (N1), or alternatively pipe-escape it in `public_index`/`internal_reference` rather than emitting it raw.
   - Consider lifting the duplicated `docs/errors` enumeration logic out of `codes.rs` and `gen-error-docs.rs` into a shared helper (N2) — purely a cleanup, not a correctness item.
3. Track the carry-forward items above against the existing 2b ticket; do not bundle them into 2a.

## Conclusion

Pass-1 blocking-adjacent gaps (F1a, F1b, F2, F4) are all resolved with assertions that are vacuous today but will catch regressions the moment active codes land in 2b. The empty leftover directory (F3) is gone. The `cfg(test)` constructor (F5) remains a deferred boundary concern for 2b, which is correct.

The registry skeleton is structurally complete, internally consistent, and properly fenced against the most plausible 2b authoring mistakes (JsonOnly leakage into templates, reserved-base severity drift, registry strings that would corrupt generated markdown, and orphan generated pages). All seven facets of the build-time validation DoD bullet are now non-trivially implemented. Local validation (cargo test, fmt, clippy, docs-sync, schema-sync) is wired into `scripts/run_all_tests.sh` and runs in every profile. **Approve to proceed to PR for `milestone_diag_2a`.**
