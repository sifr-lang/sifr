# Review: milestone_diag_2b — Diagnostic Registry Population (Pass 2)

Branch: `codex/semantic-diagnostics-diag-2b`
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Inventory: [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md)
Prior review: [reviews/semantic-diagnostic-code-taxonomy-diag-2b-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-2b-review-pass-1.md)
Validation evidence reported: `cargo test -p sifr_diagnostics`, `python3 scripts/check_diagnostic_docs_sync.py`, `python3 scripts/check_diagnostic_schema_sync.py`, `cargo fmt --check -p sifr_diagnostics`, `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings`, `scripts/run_all_tests.sh --profile quick` (signature `e1bf653aaa770517`).

## Verdict

**Approve to proceed to PR.** Every pass-1 should-fix item is resolved at the registry/docs/inventory level, with a single residual line in the issue text (line 927) that still describes `SIFR-PARSE-0001` as a reserved opaque-parser-error code rather than retired. That residual fragment is a documentation echo of pass-1 B1 and does not affect the registry, the generator, the inventory, or any guardrail; it is a non-blocking should-fix that can land in this PR or roll into the `milestone_diag_7` scope where it lives.

I re-ran the local validation gates:

- `cargo run -q -p sifr_diagnostics --bin gen-error-docs -- --check` — clean, no drift.
- `cargo test -p sifr_diagnostics --lib` — 27 passed, 0 failed.
- `python3 scripts/check_diagnostic_docs_sync.py` — clean.
- `python3 scripts/check_diagnostic_schema_sync.py` — clean.
- `cargo fmt --check -p sifr_diagnostics` — clean.
- `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings` — clean.

## Pass-1 follow-up status

| Pass-1 finding | Status | Evidence |
| --- | --- | --- |
| **B1** Issue text contradicts the chosen `SIFR-PARSE-0001` retirement. | ⚠️ Mostly fixed — line 214 row updated to "Retired as the legacy opaque parser phase bucket…" ([issue line 214](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:214)) and the Hard Rule re-anchored to retired catch-all codes plus "any active non-`INTERNAL` `0001` code" ([issue line 1201](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1201)). One residual sentence remains at [issue line 927](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:927) inside the `milestone_diag_7` scope: "Keep `SIFR-PARSE-0001` only for the reserved opaque-parser-error meaning, and guardrail it against use as a default parser code." See R1 below. |
| **B2** `panic_boundary.rs` fixture re-points to legacy `SIFR-CODEGEN-0001` test. | ✅ Fixed via planned-anchor syntax. `SIFR-INTERNAL-0001` now points at [`crates/sifr_driver/src/tests/panic_boundary.rs::planned_internal_0001`](../crates/sifr_diagnostics/src/codes.rs:1276) and `SIFR-CODEGEN-0002` at [`crates/sifr_driver/src/tests/panic_boundary.rs::planned_codegen_0002`](../crates/sifr_diagnostics/src/codes.rs:1210). The legacy panic-boundary file [crates/sifr_driver/src/tests/panic_boundary.rs](../crates/sifr_driver/src/tests/panic_boundary.rs) still exercises only `CompilePhase::Codegen → SIFR-CODEGEN-0001` behavior; the new test anchors are unambiguously planned migrations, matching the DoD's "fixture file may land in the milestone that migrates the emitting family." |
| **N1** Four prose `owner_module` values. | ✅ Fixed for all four. `SIFR-IMPORT-0001`/`0002` now `sifr_hir::lower` ([codes.rs:555](../crates/sifr_diagnostics/src/codes.rs:555), [codes.rs:566](../crates/sifr_diagnostics/src/codes.rs:566)); `SIFR-CODEGEN-0002` and `SIFR-INTERNAL-0001` now `sifr_driver::diagnostics` ([codes.rs:1212](../crates/sifr_diagnostics/src/codes.rs:1212), [codes.rs:1278](../crates/sifr_diagnostics/src/codes.rs:1278)). Carry-over noted at R2 below for the still-prose Reserved entry. |
| **N2** Retired entries reuse `owner_module` as replacement note. | ✅ Fixed via dedicated field. `DiagnosticRegistryEntry` gains `pub replacement: Option<&'static str>` ([codes.rs:218](../crates/sifr_diagnostics/src/codes.rs:218)); `retired_entry!` populates `replacement` and leaves `owner_module: None` ([codes.rs:354-371](../crates/sifr_diagnostics/src/codes.rs:354)). The public docs index now renders a Retired Codes table with a Replacement column ([gen-error-docs.rs:189-208](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:189)); the internal registry table inserts a Replacement column between Docs path and Fixture ([gen-error-docs.rs:236-256](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:236)); the markdown-safety guardrail covers the new field ([codes.rs:1635](../crates/sifr_diagnostics/src/codes.rs:1635)). |
| **N3** Stale "fixture pending in `milestone_diag_2b`" notes in the inventory. | ✅ Fixed for all three flagged rows. `SIFR-TYPE-0004` (parser-error mappings table at [inventory line 77](../internal_docs/diagnostic_emission_inventory.md:77) and registry block at [inventory line 306](../internal_docs/diagnostic_emission_inventory.md:306)), `SIFR-TYPE-0007` ([inventory line 309](../internal_docs/diagnostic_emission_inventory.md:309)), and `SIFR-TYPE-0008` ([inventory line 310](../internal_docs/diagnostic_emission_inventory.md:310)) now read "fixture pending in `milestone_diag_7`". |
| **N4** Two-arg/dedupe sweeps for `SIFR-DECIMAL-0004` and `SIFR-RESULT-0001`. | Carry-over for `milestone_diag_8`/`milestone_diag_10` per the original framing. No change required for diag_2b. |
| **N5** `SIFR-INTERNAL-0002` `docs_path` anchor. | ✅ Fixed. `docs_path: "docs/errors/diagnostic-codes.md"` ([codes.rs:1287](../crates/sifr_diagnostics/src/codes.rs:1287)) — anchor stripped, matches the family-base reserved entries. |

## Definition-of-done coverage (revalidated)

| DoD bullet | Status | Evidence |
| --- | --- | --- |
| Every emitted code exists in the registry. | ✅ (vacuous) | No new emission sites added; legacy `SIFR-WORKSPACE-0001..0103` strings in [sifr_driver/src/diagnostics.rs:96-128](../crates/sifr_driver/src/diagnostics.rs:96) are all active in the registry. |
| Every active registry code records a representative fixture path; reserved codes exempt. | ✅ | The registry-skeleton test enforces `representative_fixture_path.is_some()` for every Active entry ([codes.rs:1464-1468](../crates/sifr_diagnostics/src/codes.rs:1464)). All 78 Active entries pass; reserved entries (family bases + `SIFR-INTERNAL-0002`) and the four Retired entries leave it `None`. |
| Every active code has a docs page under `docs/errors/<CODE>.md`. | ✅ | `ls docs/errors/SIFR-*.md` returns 78 files, exactly the 78 Active entries. Build-time check at [codes.rs:1519-1547](../crates/sifr_diagnostics/src/codes.rs:1519) and drift-check parity at [gen-error-docs.rs:102-139](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:102) both green. Spot-checked [docs/errors/SIFR-INTERNAL-0001.md](../docs/errors/SIFR-INTERNAL-0001.md), [docs/errors/SIFR-CODEGEN-0002.md](../docs/errors/SIFR-CODEGEN-0002.md), [docs/errors/SIFR-IMPORT-0001.md](../docs/errors/SIFR-IMPORT-0001.md). |
| Every active code has a `DiagnosticCode` constant; retired codes do not. | ✅ | `ACTIVE_DIAGNOSTIC_CODES` ([codes.rs:1299-1378](../crates/sifr_diagnostics/src/codes.rs:1299)) declares 78 constants; bidirectional set-equality with active registry entry ids is asserted at [codes.rs:1498-1506](../crates/sifr_diagnostics/src/codes.rs:1498). The four Retired entries (`SIFR-PARSE-0001`, `SIFR-TYPE-0001`, `SIFR-CODEGEN-0001`, `SIFR-BUILD-0001`) and the Reserved `SIFR-INTERNAL-0002` have no constant. |
| Domain diagnostic helpers exist only for active codes. | ✅ (vacuous) | Helpers are introduced family-by-family in `milestone_diag_4a`/`7`/`8`; the constants are now visible to compiler crates as required. |
| Registry population matches the inventory. | ✅ | All carryover discrepancies from pass-1 are reconciled. Workspace review block at [inventory lines 124-133](../internal_docs/diagnostic_emission_inventory.md:124) matches the registry's keep-all + add-`0104` decision. |
| Existing workspace codes either active with precise rule + docs page or retired with replacement. | ✅ | All seven `SIFR-WORKSPACE-0001..0103` are Active with precise templates and per-code docs pages; `SIFR-WORKSPACE-0104` is added Active with template `workspace import cycle detected: {cycle}`. None retired. |

Note on count: the pass-1 review reported 75 Active entries; the actual count is 78 (verified by `grep -c '^    active_entry!'` and the exact length of `ACTIVE_DIAGNOSTIC_CODES`). The discrepancy is a counting artifact in pass-1, not a registry change between passes — `git log` shows no Active-entry additions on this branch since pass-1.

## Generated docs spot-checks

- [docs/errors/diagnostic-codes.md:137-145](../docs/errors/diagnostic-codes.md:137) — new "Retired Codes" table with Replacement column rendered cleanly: four rows for `SIFR-PARSE-0001`, `SIFR-TYPE-0001`, `SIFR-CODEGEN-0001`, `SIFR-BUILD-0001`, each with the registry-supplied replacement note.
- [internal_docs/diagnostic_codes.md:41-42](../internal_docs/diagnostic_codes.md:41) — header row now includes `Replacement` between `Docs path` and `Fixture`; 13-column rule row matches.
- [internal_docs/diagnostic_codes.md:60-63](../internal_docs/diagnostic_codes.md:60) — retired rows render Replacement and `n/a` for Owner; no overload of the Owner column with replacement prose.
- [internal_docs/diagnostic_codes.md:135](../internal_docs/diagnostic_codes.md:135) — `SIFR-CODEGEN-0002` row carries planned-anchor fixture path and `sifr_driver::diagnostics` owner.
- [internal_docs/diagnostic_codes.md:141](../internal_docs/diagnostic_codes.md:141) — `SIFR-INTERNAL-0001` row similarly clean.
- [internal_docs/diagnostic_codes.md:142](../internal_docs/diagnostic_codes.md:142) — `SIFR-INTERNAL-0002` Reserved row shows `docs/errors/diagnostic-codes.md` (no anchor) and the still-prose owner `diagnostic recovery cap` (see R2 below).

The active-code page template now includes a "Representative fixture" row; spot-checked pages match the registry's `representative_fixture_path`.

## Findings

### Blocking

None.

### Should-fix in this PR

#### R1. Issue line 927 still says `SIFR-PARSE-0001` is "reserved opaque-parser-error" inside the `milestone_diag_7` scope

[issues/…-diagnostics.md:927](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:927):

> Keep `SIFR-PARSE-0001` only for the reserved opaque-parser-error meaning, and guardrail it against use as a default parser code.

Pass-1 B1 called out two contradiction sites (renumbering table line 214, Hard Rule line 1201); both are now fixed. This third site lives in the `milestone_diag_7: Parser, Name, Import, Type, and Call Diagnostics` scope block and was not on the pass-1 list, but it perpetuates the same contradiction the pass-1 review flagged: the issue describes `SIFR-PARSE-0001` as Reserved (with a guardrail to enforce that meaning), while the registry has retired it and uses planned `SIFR-PARSE-0002..0009` plus a `parser_category` JSON arg for the upstream-recovery case.

Suggested edit (mirrors the new line 214 phrasing):

> Retire `SIFR-PARSE-0001`; parser diagnostics use category-specific `SIFR-PARSE-0002..0009` codes and the `parser_category` JSON arg for upstream-recovery context. Guardrail forbids any non-`INTERNAL` `0001` code from being used as a family-default catch-all.

This is a one-line documentation edit; the registry, generator, and gates already enforce the retired meaning. Treating as should-fix because the diag_7 reviewer will hit the same conflict; if the team prefers, it is also fine to roll the line-927 update into the diag_7 PR scope and mark it acknowledged.

### Non-blocking

#### R2. `SIFR-INTERNAL-0002` `owner_module` is still descriptive prose

[crates/sifr_diagnostics/src/codes.rs:1291](../crates/sifr_diagnostics/src/codes.rs:1291) sets `owner_module: Some("diagnostic recovery cap")`. After pass-1 N1, every other registry entry uses a Rust module path (`sifr_driver::diagnostics`, `sifr_hir::lower::statements`, etc.). `SIFR-INTERNAL-0002` is Reserved, so the active-entry guardrail does not require a module path, and the user's pass-2 update note acknowledges this. Still, when the entry is activated in `milestone_diag_10` the owner will need a real module path — natural candidate is `sifr_diagnostics::recovery` or whatever module emits the recovery-cap omission summary. Worth folding into the diag_10 activation rather than touching it now; flagging here for visibility so it is not forgotten.

#### R3. `SIFR-RESULT-0001` and `SIFR-DECIMAL-0004` carry-over from pass-1 N4

Unchanged from pass-1; both deferred to `milestone_diag_8` / `milestone_diag_10`. Restating here only so the diag_8 reviewer can see the trail.

### Style-only / non-actionable

- Active-code pages now have eight metadata rows (Code, Family, Severity, Owner, Message template, Representative fixture, Declared args, Dedupe args). The added "Representative fixture" row makes per-code pages self-contained.
- The internal registry table is now 13 columns. Rendering width is borderline but every cell is single-line and markdown-safe (asserted at [codes.rs:1630-1666](../crates/sifr_diagnostics/src/codes.rs:1630)).

## Summary

`milestone_diag_2b` pass 2 cleanly addresses every pass-1 should-fix item:

- `SIFR-PARSE-0001` retirement is reflected in the renumbering table and the Hard Rule (one residual sentence at line 927 still needs the same edit — R1).
- `SIFR-INTERNAL-0001` and `SIFR-CODEGEN-0002` planned fixture paths use `panic_boundary.rs::planned_*` anchors, distinguishing them from the legacy `SIFR-CODEGEN-0001` test that still occupies the same file.
- The four flagged owner modules are now Rust module paths; one Reserved holdout (`SIFR-INTERNAL-0002`, R2) is non-blocking and naturally lands with diag_10 activation.
- Retired entries no longer overload `owner_module` for replacement notes — a dedicated `replacement: Option<&'static str>` field is wired through registry, public docs index Retired Codes table, internal registry Replacement column, and markdown-safety guardrail.
- Inventory `fixture pending in milestone_diag_2b` notes for `SIFR-TYPE-0004/0007/0008` are updated to `milestone_diag_7`.
- `SIFR-INTERNAL-0002` `docs_path` is the index without the dead anchor.

Validation gates locally green: `cargo test`, both `check_diagnostic_*_sync.py` scripts, `cargo fmt --check`, `cargo clippy -D warnings`, and the `gen-error-docs --check` drift gate. Registry has 78 Active entries with constants, 4 Retired entries with replacement metadata, and 18 Reserved entries (17 family bases + `SIFR-INTERNAL-0002`); the bidirectional active-set/constants invariant and the docs-page existence-with-exact-casing invariant are both enforced at build time and at drift-check time.

No blocking correctness defects. **Approve to open the `milestone_diag_2b` PR**, with R1 (one-line edit at issue line 927) either landed in this PR or acknowledged for the `milestone_diag_7` PR.
