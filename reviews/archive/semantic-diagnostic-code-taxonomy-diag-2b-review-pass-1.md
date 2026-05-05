# Review: milestone_diag_2b — Diagnostic Registry Population (Pass 1)

Branch: `codex/semantic-diagnostics-diag-2b`
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Inventory: [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md)
Prior reviews: 2a pass 2 (`reviews/semantic-diagnostic-code-taxonomy-diag-2a-review-pass-2.md`), 3 pass 2 (`reviews/semantic-diagnostic-code-taxonomy-diag-3-review-pass-2.md`)
Validation evidence reported: `cargo test -p sifr_diagnostics`, `python3 scripts/check_diagnostic_docs_sync.py`, `python3 scripts/check_diagnostic_schema_sync.py`, `cargo fmt --check -p sifr_diagnostics`, `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings`, `scripts/run_all_tests.sh --profile quick` (signature `e1bf653aaa770517`).

## Scope reviewed

- [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs) — 75 active `DiagnosticCode` constants, 75 active registry entries, 17 reserved family bases, 1 reserved `SIFR-INTERNAL-0002`, 4 retired entries, the `active_entry!` / `retired_entry!` macros, and the existing constant/registry sync, declared-arg, dedupe-arg, template-placeholder, family-name, and markdown-safety guardrail tests.
- [crates/sifr_diagnostics/src/bin/gen-error-docs.rs](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs) — generator and `--check` drift detection, including `check_active_doc_casing` orphan/case enforcement.
- Generated outputs: [docs/errors/diagnostic-codes.md](../docs/errors/diagnostic-codes.md), [internal_docs/diagnostic_codes.md](../internal_docs/diagnostic_codes.md), and 75 generated active-code pages under `docs/errors/SIFR-*.md`.
- Phase tracker delta in [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) (status flip `3 → 2b`, three new checked DoD bullets, two unchecked, validation evidence section).

I re-ran the validation gates locally:

- `cargo run -q -p sifr_diagnostics --bin gen-error-docs -- --check` — clean, no drift.
- `cargo test -p sifr_diagnostics --lib` — 27 passed, 0 failed.
- `python3 scripts/check_diagnostic_schema_sync.py` — clean.

## Verdict

The registry population is internally consistent, the generator/drift gate is green, and every `milestone_diag_2b` definition-of-done bullet has a concrete artifact behind it. No blocking correctness defects. **Approve to proceed to PR**, contingent on either (a) acknowledging or fixing the four nits below in this PR or (b) acknowledging them in writing for `milestone_diag_4a`/`milestone_diag_7` follow-ups.

The pass-1 carry-over from 2a (F5 — `DiagnosticCode::new` was `#[cfg(test)]` only) has been resolved by this milestone: `new` is now `const` and `pub`-visible inside the crate via the constants, and 75 active constants exist outside the `#[cfg(test)]` test-only triplet ([codes.rs:128-133](../crates/sifr_diagnostics/src/codes.rs:128)). Active codes can now be referenced from non-test compiler source as required by the diag_4a+ milestones.

## Definition-of-done coverage

| DoD bullet | Status | Evidence |
| --- | --- | --- |
| Every emitted code exists in the registry | ✅ (vacuous) | No new emission sites in this milestone; the legacy `CompileError::workspace_diagnostic_code` prefix classifier ([sifr_driver/src/diagnostics.rs:96-128](../crates/sifr_driver/src/diagnostics.rs:96)) still uses raw string codes `SIFR-WORKSPACE-0001..0103` and all of those are now active in the registry. The decidable global emission-presence guardrail is correctly deferred to `milestone_diag_11` per the issue. |
| Every active registry code records a representative fixture path; reserved codes exempt | ✅ | All 75 `DiagnosticState::Active` entries set `representative_fixture_path: Some(...)`; the active assertion in `registry_skeleton_is_internally_consistent` ([codes.rs:1459-1463](../crates/sifr_diagnostics/src/codes.rs:1459)) enforces this. Reserved (family bases, `SIFR-INTERNAL-0002`) and retired entries set it to `None`. The DoD explicitly allows the fixture *file* to land in the migrating milestone ("The fixture file itself may land in the milestone that migrates the emitting family"); see N3 below for the inventory of paths that don't yet exist on disk. |
| Every active code has a docs page under `docs/errors/<CODE>.md` | ✅ | `ls docs/errors/SIFR-*.md` returns 78 files: 75 active pages + 3 new reserved/retired pages (none for retired/reserved actually; 75 active match constants). The active-page existence and exact casing are enforced both at build time by `active_diagnostic_docs_pages_exist_with_exact_casing` ([codes.rs:1514-1542](../crates/sifr_diagnostics/src/codes.rs:1514)) and at drift-check time by `check_active_doc_casing` ([gen-error-docs.rs:102-139](../crates/sifr_diagnostics/src/bin/gen-error-docs.rs:102)). Spot-checked [docs/errors/SIFR-NAME-0001.md](../docs/errors/SIFR-NAME-0001.md), [docs/errors/SIFR-WORKSPACE-0101.md](../docs/errors/SIFR-WORKSPACE-0101.md), [docs/errors/SIFR-INTERNAL-0001.md](../docs/errors/SIFR-INTERNAL-0001.md), [docs/errors/SIFR-CODEGEN-0002.md](../docs/errors/SIFR-CODEGEN-0002.md). |
| Every active code has a `DiagnosticCode` constant; retired codes do not | ✅ | `ACTIVE_DIAGNOSTIC_CODES` ([codes.rs:1295-1374](../crates/sifr_diagnostics/src/codes.rs:1295)) lists 75 constants. Bidirectional set-equality between `active_registry_entries()` ids and `ACTIVE_DIAGNOSTIC_CODES` ids is asserted at [codes.rs:1493-1502](../crates/sifr_diagnostics/src/codes.rs:1493). Retired entries (`SIFR-PARSE-0001`, `SIFR-TYPE-0001`, `SIFR-CODEGEN-0001`, `SIFR-BUILD-0001`) and reserved entries (`SIFR-INTERNAL-0002`, family bases) have no constants. |
| Domain diagnostic helpers may exist only for active codes | ✅ (vacuous) | No domain helpers added in this milestone — they are introduced family-by-family in `milestone_diag_4a`/`7`/`8`. The constants are now visible to compiler source crates, which is the prerequisite. |
| Registry population matches the checked-in inventory | ⚠️ Mostly — see B1 below for two divergences from the inventory text that should be reconciled in this PR. |
| Every existing workspace code has either an active registry entry with a precise rule and docs page, or a retired entry with replacement code recorded | ✅ | All seven existing workspace codes (`0001..0004`, `0101..0103`) are active with precise rules and docs pages, matching the inventory's "keep" decision (lines 126–133 of the inventory). `SIFR-WORKSPACE-0104` is added active per inventory line 133. None of the existing workspace codes was retired, which is consistent with the inventory's stated review outcome. Templates (`could not parse workspace manifest at {path}: {reason}`, `source root {path} escapes the workspace root`, `module {module} is ambiguous in workspace {workspace}`, etc.) are precise and align with the legacy prefix-classifier semantics in [sifr_driver/src/diagnostics.rs:96-128](../crates/sifr_driver/src/diagnostics.rs:96). |

## Focus-area review

### Active vs reserved vs retired, especially the called-out codes

| Code | Registry state | Severity | Verdict |
| --- | --- | --- | --- |
| `SIFR-PARSE-0001` | Retired | n/a | Matches inventory ("retired legacy phase bucket", inventory line 53), but **diverges from the issue's "Existing code renumbering" table** (issue line 214: "Reserved meaning only: opaque parser error with no upstream classification"). See B1 below — this is a deliberate decision in `milestone_diag_3` and is allowed by the issue's "These exact numbers are the proposed starting point. They can be adjusted during `milestone_diag_2b`" caveat (issue line 1137), but the issue text should be updated to match, otherwise diag_4a/7 reviewers will hit the same conflict. |
| `SIFR-TYPE-0001` | Retired | n/a | Correct per issue line 215 and inventory. Replacement note "replaced by active semantic family codes" is accurate. |
| `SIFR-CODEGEN-0001` | Retired | n/a | Correct per issue line 216 and inventory. Replacement note "replaced by SIFR-CODEGEN-0002 or INTERNAL codes" is accurate. |
| `SIFR-BUILD-0001` | Retired | n/a | Correct per issue line 217 and inventory. Replacement note "replaced by active BUILD operation codes" is accurate. |
| `SIFR-INTERNAL-0001` | Active | Error | Correct per issue line 733 ("ICE-class internal diagnostics such as `SIFR-INTERNAL-0001` declare `Error`"). State flip from Reserved → Active is the right milestone-2b transition; previously the entry had `docs_path: docs/errors/diagnostic-codes.md#sifr-internal-0001` and is now `docs/errors/SIFR-INTERNAL-0001.md` with a generated page. |
| `SIFR-INTERNAL-0002` | Reserved | Note | Correct per issue line 733 and 788 ("`SIFR-INTERNAL-0002` remains `Reserved` until activation in `milestone_diag_10`"). Severity declared but no constant, no fixture, no template — exactly the reserved shape. |

### Existing workspace codes (`SIFR-WORKSPACE-0001..0103`)

All retained as active with precise per-code rules:

- `0001` — malformed sifr.toml; template `could not parse workspace manifest at {path}: {reason}`. Precise.
- `0002` — source root escapes via `..`; template `source root {path} escapes the workspace root`. Precise.
- `0003` — source root not a directory; template `source root {path} is not a directory`. Precise.
- `0004` — invalid source root entry shape/path; template `invalid source root entry {entry}`. Precise.
- `0101` — unresolved import; template `could not resolve import {module}` with json-only `searched_paths`. Precise.
- `0102` — ambiguous import; template `module {module} is ambiguous in workspace {workspace}` with json-only `candidate_paths`. Precise.
- `0103` — namespace collision; template `module {module} collides with namespace path {path}`. Precise.

The decision to keep all seven matches the inventory's `milestone_diag_2b` review block (inventory lines 126–133). `SIFR-WORKSPACE-0104` is correctly added new for workspace import cycle, with template `workspace import cycle detected: {cycle}` and a planned fixture at [crates/sifr_driver/src/tests/project_graph.rs](../crates/sifr_driver/src/tests/project_graph.rs). The inventory left ownership ambiguous ("`SIFR-WORKSPACE-0104` or `SIFR-IMPORT-0004`", inventory line 93); the registry chose `WORKSPACE` because `compile_order.rs` is workspace-graph driven. That is consistent with the policy in the issue line 201 ("Module resolution diagnostics use `SIFR-IMPORT-*` when the failure is about import statement form … `SIFR-WORKSPACE-*` when the failure is about workspace/project layout, module graph construction…").

### Message templates, declared args, dedupe args, owner modules, fixture plans

Templates are validated by `assert_template_placeholders_are_declared` ([codes.rs:1601-1623](../crates/sifr_diagnostics/src/codes.rs:1601)) — every `{placeholder}` must have a `MessageAndJson` declared arg, and json-only args must not appear in the template. Dedupe args are validated by `assert_dedupe_args_are_declared` ([codes.rs:1586-1599](../crates/sifr_diagnostics/src/codes.rs:1586)) — every dedupe arg must be declared. Active entries assert severity, owner_module, message_template, fixture path, and canonical docs path ([codes.rs:1442-1469](../crates/sifr_diagnostics/src/codes.rs:1442)). All 75 active entries pass.

The 75 declared severities partition cleanly: 71 Error, 2 Warning (`SIFR-TYPE-0901`, `SIFR-FLOW-0901`), 2 Note (`SIFR-TYPE-0902` + reserved `SIFR-INTERNAL-0002`). Templates avoid backticks (asserted by `assert_registry_strings_are_markdown_safe`, [codes.rs:1625-1660](../crates/sifr_diagnostics/src/codes.rs:1625)), so generated inline-code spans are safe.

Owner modules generally use Rust-module-path form (`sifr_hir::lower::statements`, `sifr_driver::project::discovery`, etc.) but four entries do not — see N1 below.

Dedupe args reasonably mirror declared args for most entries. Two minor observations folded into N4 below.

### Generated docs and docs-sync behavior

- `docs/errors/diagnostic-codes.md` — public index, lists 75 active codes (table rows 35–112), 19 reserved codes (17 family bases + `SIFR-INTERNAL-0002` + the family base count includes `INTERNAL-0000`), and 4 retired codes with replacement text.
- `internal_docs/diagnostic_codes.md` — full registry table, 100 rows total: 17 family bases (Reserved) + 4 retired + 75 active + `SIFR-INTERNAL-0002` Reserved + (the legacy `SIFR-INTERNAL-0001` Reserved row from 2a is now replaced by an Active row, correctly).
- 75 individual `docs/errors/SIFR-*.md` pages, each containing summary + a 7-row metadata table (Code, Family, Severity, Owner, Message template, Representative fixture, Declared args, Dedupe args).
- The docs-sync drift check (`scripts/check_diagnostic_docs_sync.py`) is a thin wrapper that runs `cargo run -p sifr_diagnostics --bin gen-error-docs -- --check`, which compares in-memory generated content against the on-disk file (no `git diff` dependency, works in dirty trees), and additionally calls `check_active_doc_casing` to detect orphan or wrong-cased `.md` files in `docs/errors/`. I verified locally that the drift check is green.

The schema-sync gate is unchanged from `milestone_diag_1`/`2a` (no model surface change in this milestone) and is reported passing.

## Findings

### Blocking

None.

### Should-fix in this PR

#### B1. SIFR-PARSE-0001 retirement diverges from the issue's existing-code renumbering table

The issue's "Existing code renumbering" table (issue line 214) reads:

> `SIFR-PARSE-0001` | Reserved meaning only: opaque parser error with no upstream classification. It must not be used when a more specific parser condition is detectable, and guardrails must reject it as a default parser emission code.

This describes a `Reserved`-state code with a precise opaque-parser-error semantic, not a Retired one. The issue's "Hard Rules" reinforce the same intent (issue line 1201): "Do not use `SIFR-PARSE-0001`, `SIFR-CODEGEN-0001`, `SIFR-BUILD-0001`, or any other `0001` code as a family-default catch-all unless the registry gives it a precise, guardrailed meaning."

The inventory (line 49 and line 53) and the registry instead retire `SIFR-PARSE-0001` ("retired legacy phase bucket"). This is allowed by the issue's "These exact numbers are the proposed starting point. They can be adjusted during `milestone_diag_2b`" caveat (issue line 1137), and Retired is defensible — the issue's `milestone_diag_4a` already deletes the only existing emission path (`CompilePhase::Parse → SIFR-PARSE-0001`), and there is no concrete inventory entry that needs a "Reserved opaque parser error" code today.

But the milestone's job is to populate the registry to match the inventory *and* the broader phase contract. The issue text now contradicts the registry on this point, which will recur in every later parser-migration review. Two acceptable paths:

1. (Preferred, smaller surface) Update the issue's existing-code-renumbering table row for `SIFR-PARSE-0001` from "Reserved meaning only" to "Retired; replaced by `SIFR-PARSE-0002..0009`. The opaque-parser-error use case folds into `SIFR-PARSE-0002` (expected token / generic recovery) plus the parser-category json arg." Update the Hard Rule on line 1201 to either drop the `SIFR-PARSE-0001` reference or re-anchor it to "any active `0001` code in a non-INTERNAL family."
2. Flip the registry to mark `SIFR-PARSE-0001` Reserved (with the active-emission guardrail planned for `milestone_diag_7`) and keep an empty docs page reference. This is more churn and would need a new constant `Reserved` exemption in the constants/registry sync test.

Path 1 keeps the inventory's intent intact and just synchronizes the issue text. Either way, the contradiction should not be left to be discovered by the diag_7 reviewer.

#### B2. `SIFR-INTERNAL-0001` claims a `crates/sifr_driver/src/tests/panic_boundary.rs` fixture, but the panic-boundary tests still assert legacy `SIFR-CODEGEN-0001` behavior

`SIFR-INTERNAL-0001` and `SIFR-CODEGEN-0002` both record `representative_fixture_path: "crates/sifr_driver/src/tests/panic_boundary.rs"`. The file exists, but as of this branch it still tests the legacy `CompilePhase::Codegen → "SIFR-CODEGEN-0001"` mapping, not `SIFR-INTERNAL-0001`/`SIFR-CODEGEN-0002`. Because `SIFR-INTERNAL-0001` is now `Active` (not `Reserved`), the registry asserts a fixture path that does not actually lock the new code's behavior.

This is acceptable per the DoD "fixture file itself may land in the milestone that migrates the emitting family" *for representative fixtures of paths that don't yet exist*. But here the path *does* exist and locks a different code, which is more confusing than a missing fixture file. Two options:

1. Mark `SIFR-INTERNAL-0001` `Reserved` until `milestone_diag_4a` migrates `panic_boundary.rs`. This needs the bidirectional active-set-vs-constants invariant to relax for one entry, and changes the issue's "ICE-class internal diagnostics such as `SIFR-INTERNAL-0001` declare `Error`" — but Reserved entries can still declare severity (the registry already does this for `SIFR-INTERNAL-0002` Note).
2. Keep `SIFR-INTERNAL-0001` Active and point its `representative_fixture_path` at a not-yet-existing path (e.g., `crates/sifr_driver/src/tests/panic_boundary.rs::ice_emits_internal_0001`) so it is unambiguously a planned fixture rather than a re-pointed one. The DoD explicitly allows the file to land later.

Option 2 is the lighter touch and closer to how `SIFR-PARSE-000x` and `SIFR-TYPE-0004/0007/0008` are already handled (planned paths that don't exist yet — see N3).

`SIFR-CODEGEN-0002` has the same shape as `SIFR-INTERNAL-0001` and the same option-2 treatment is warranted.

### Non-blocking

#### N1. Four `owner_module` values are descriptive prose, not Rust module paths

The other 71 active entries set `owner_module` to a Rust module path: `sifr_hir::lower`, `sifr_hir::lower::statements`, `sifr_driver::workspace`, `sifr_type_system`, etc. Four entries break this convention:

- `SIFR-IMPORT-0001`, `SIFR-IMPORT-0002` — owner `sifr_hir::lower::mod`. The Rust module path for `crates/sifr_hir/src/lower/mod.rs` is **`sifr_hir::lower`** (a `mod.rs` file is not addressed as `::mod`). Either drop `::mod` (consistent with other lowering entries that target `mod.rs`, e.g., `SIFR-NAME-0001`/`0002`/`0004` which all set `sifr_hir::lower`) or move the import-resolution helper to a dedicated `sifr_hir::lower::imports` module and update the path. The inventory line 22 already calls out this file as `crates/sifr_hir/src/lower/mod.rs`; the natural module path is `sifr_hir::lower`.
- `SIFR-CODEGEN-0002` — owner `sifr_driver::codegen boundary` (with a space). The actual codegen panic boundary lives in `sifr_driver::diagnostics::run_codegen_with_boundary` ([sifr_driver/src/diagnostics.rs:255](../crates/sifr_driver/src/diagnostics.rs:255)). Suggest `sifr_driver::diagnostics`.
- `SIFR-INTERNAL-0001` — owner `compiler panic boundary` (descriptive prose). Inventory line 351 also uses descriptive prose here, but every other entry uses a module path. Suggest `sifr_driver::diagnostics` (same as the codegen boundary), or invent a dedicated `sifr_diagnostics::internal` namespace if internal codes will be emitted from multiple call sites.

Tightening these reduces inconsistency in the registry table and makes the future emission-presence guardrail (`milestone_diag_11`) trivial to implement as a "module-path string in repo" check. None of the values trip the markdown-safety guardrail (no backticks), but the registry's emergent contract for `owner_module` should stay as "Rust module path."

#### N2. Retired entries reuse `owner_module` to carry the replacement note

`retired_entry!` ([codes.rs:352-369](../crates/sifr_diagnostics/src/codes.rs:352)) puts the replacement description in the `owner_module: Some($replacement)` field, and the public docs index renders it under a "Replacement" column. The registry table in `internal_docs/diagnostic_codes.md` renders the same string under "Owner". This is dual-semantic — readers of the internal table see e.g. "Owner = `replaced by active PARSE category codes`" for retired rows, which is misleading.

Either:

1. Add a real `replacement: Option<&'static str>` field to `DiagnosticRegistryEntry`, set it for retired entries, render it in the public retired table, and stop overloading `owner_module`. Reserved/active entries leave `replacement: None`.
2. Keep the overload but rename the public-doc column from "Replacement" to match — and have the internal-doc renderer show "Replacement" for retired rows specifically.

Option 1 is cleaner; the field is a leaf addition with a one-line addition to `assert_registry_strings_are_markdown_safe`.

#### N3. Several active fixture paths point to files that do not yet exist

The DoD explicitly allows planned fixture paths to land in the migrating milestone. For full transparency, the active codes whose `representative_fixture_path` is currently a non-existent file are:

- `SIFR-PARSE-0002..0009` — eight planned fixtures under `crates/sifr/tests/e2e/fail/parser_*.sifr`. Inventory says these land in `milestone_diag_7` — consistent.
- `SIFR-TYPE-0004` — `crates/sifr/tests/e2e/fail/missing_type_annotation.sifr`. Inventory line 306: "fixture pending in `milestone_diag_2b`" — that text is now stale; this PR did not add the fixture. Acceptable per the DoD ("may land in the milestone that migrates the emitting family"), but the inventory note should be updated to "pending in `milestone_diag_7`" to remove the contradiction.
- `SIFR-TYPE-0007` — `crates/sifr/tests/e2e/fail/invalid_type_annotation.sifr`. Same as above, inventory line 309 "pending in `milestone_diag_2b`".
- `SIFR-TYPE-0008` — `crates/sifr/tests/e2e/fail/container_literal_type_conflict.sifr`. Same as above, inventory line 310 "pending in `milestone_diag_2b`".
- `SIFR-TYPE-0901` — `crates/sifr/tests/e2e/pass/arithmetic_overflow_warning.sifr` (warning). Lands with type-system migration.
- `SIFR-TYPE-0902` — `crates/sifr/tests/e2e/pass/reveal_type.sifr` (note). Lands with type-system migration.
- `SIFR-FLOW-0901` — `crates/sifr/tests/e2e/fail/unreachable_statement_warning.sifr` (warning). Lands with HIR statement migration.

Total: ~14 planned-but-missing fixtures, none blocking per the DoD. Two adjustments would make the worklist clean:

- Update inventory lines 306/309/310 to drop the "pending in `milestone_diag_2b`" text and replace with "pending in `milestone_diag_7`" (or `milestone_diag_8` for the FLOW-0901 case).
- Optionally have the `active_diagnostic_docs_pages_exist_with_exact_casing` test add a parallel "fixture file existence" assertion gated by `cfg(test)` once the migration milestones land, so a misnamed planned fixture is caught before the migration PR.

#### N4. Two minor template / dedupe inconsistencies worth a sweep before diag_7

- `SIFR-DECIMAL-0004` declares zero args and zero dedupe args; template is the literal string "cannot mix Decimal and BigDecimal". That is fine for `milestone_diag_2b` (every error of this kind is identical), but `milestone_diag_10` recovery deduplication uses `(code, message_template, primary SourceSpan range, dedupe args)` (issue line 619). With zero dedupe args, two distinct mixed-arithmetic errors at the same span are deduped to one — desired — but two distinct errors at *different* spans are not deduped — also desired. Confirm during diag_10 design that the empty dedupe-key choice is intentional.
- `SIFR-RESULT-0001` declares zero args and template "unused Result value". Acceptable, but the inventory's recovery sketch (inventory line 370: "code + result/error type + primary span") implies a `result_type` arg. Worth a sweep at diag_8 to add `result_type` as an arg if it materially helps the LSP / tooling story.

Both are diag_8/diag_10 concerns, not diag_2b defects.

#### N5. `SIFR-INTERNAL-0002`'s `docs_path` points to an anchor that doesn't exist

`SIFR-INTERNAL-0002` sets `docs_path: "docs/errors/diagnostic-codes.md#sifr-internal-0002"`, but the generated `docs/errors/diagnostic-codes.md` has no `#sifr-internal-0002` heading — the reserved codes table just lists the row. Following the URL anchors at the top of the page. Other reserved entries (family bases) use `docs/errors/diagnostic-codes.md` without an anchor. Either drop the `#sifr-internal-0002` suffix (consistent with family bases) or make `internal_reference()` emit per-reserved-code anchors.

Drop is the simpler fix; anchors aren't load-bearing for any consumer in this milestone.

### Style-only / non-actionable

- The macros `active_entry!` / `retired_entry!` ([codes.rs:333-369](../crates/sifr_diagnostics/src/codes.rs:333)) are readable but force every active entry to declare `tooling: DiagnosticTooling::DEFAULT`. Once any entry needs non-default tooling, the macro will need an extra arg. Not relevant in `milestone_diag_2b`.
- `internal_docs/diagnostic_codes.md` is a 12-column table; rendering width is borderline but `assert_registry_strings_are_markdown_safe` keeps each cell single-line. Fine for now.

## Summary

`milestone_diag_2b` lands a complete, internally consistent diagnostic registry: 75 active codes with constants, message templates, declared args, dedupe args, owner modules, severities, fixture plans, and per-code generated docs pages; 4 retired catch-all codes; 18 reserved entries (17 family bases + `SIFR-INTERNAL-0002`); a green generator/drift gate; and validation evidence for every gate the issue requires at this milestone. The registry/constants sync invariant is enforced bidirectionally and the docs-page existence + casing invariant is enforced both at build time and at drift-check time.

Two should-fix items (B1: reconcile the issue's `SIFR-PARSE-0001` text with the chosen retirement; B2: clarify that `panic_boundary.rs` is a planned fixture for `SIFR-INTERNAL-0001`/`SIFR-CODEGEN-0002`, not the current locked one) plus five non-blocking nits (N1: four prose owner modules; N2: retired-entry replacement field overload; N3: stale inventory "pending in 2b" notes for fixtures that now land in 7/8; N4: two args/dedupe sweeps for diag_8/10; N5: stale anchor in `SIFR-INTERNAL-0002` docs_path).

None of those gate the PR. Approve to open `milestone_diag_2b` PR after either landing the should-fix items or recording acknowledgement that they roll into diag_4a/7.
