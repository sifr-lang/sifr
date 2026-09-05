# Review (Round 3): Ad-Hoc Phase — Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics

Reviewer: agent
Date: 2026-04-29
Source: `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`
Repo state at review: branch `main`, commit `38b1f9c9`
Prior reviews: `reviews/semantic-diagnostic-code-taxonomy-proposal-review.md` (round 1), `reviews/semantic-diagnostic-code-taxonomy-proposal-review-2.md` (round 2)
Lens: principal-engineer / compiler architecture

## Verdict: READY — no remaining material blockers

The round 2 amendments have been applied cleanly and the proposal is now directly implementable. A `milestone_diag_1` PR can begin without follow-up clarifications. The findings below are residual, non-blocking risks that are appropriate to land *during* implementation rather than as gating amendments.

Severity tags: 🔴 blocker, 🟠 must-fix, 🟡 should-fix, 🟢 polish. **No 🔴 or 🟠 findings remain.**

---

## 1. Round 2 amendment verification

For traceability, the 17 Y-amendments from round 2 map onto the current document as follows. All are addressed.

| Round-2 ref | Topic | Status | Verification |
|---|---|---|---|
| Y1 🔴 | Numbering convention vs `SIFR-PARSE-0001` | ✅ Resolved | PARSE family now `0000..0999`, base `0000` reserved, `0001` is base+1 active. Convention holds without exception. |
| Y2 🔴 | Registry/inventory ordering | ✅ Resolved | diag_2 split into `diag_2a` (skeleton) → `diag_3` (inventory) → `diag_2b` (population). Mermaid graph reflects new order. |
| Y3 🟠 | Accumulator vs `Result<...>` shape | ✅ Resolved | `LoweringOutcome { result, diagnostics }` with `LowerCtx::emit(...)` accumulator and explicit driver-side severity check. |
| Y4 🟠 | Codegen-with-span loophole | ✅ Resolved | "Codegen diagnostics with source mappings are `SourceDiagnostic` values. Codegen failures without a source mapping are treated as internal failures and use `SIFR-INTERNAL-*`…" |
| Y5 🟠 | Template substitution model | ✅ Resolved | `args: BTreeMap<String, serde_json::Value>` field added; `{name}` braces with `{{`/`}}` escaping pinned. |
| Y6 🟠 | Generator binary names | ✅ Resolved | `cargo run -p sifr_diagnostics --bin gen-error-docs`, `scripts/check_diagnostic_docs_sync.py`, and output paths pinned. |
| Y7 🟡 | Single envelope version | ✅ Resolved | "The envelope version is the only schema version; individual diagnostics do not carry a second version number." `schema_version` removed from per-diagnostic structs. |
| Y8 🟡 | Synthesized HIR span policy | ✅ Resolved | "Synthesized HIR nodes inherit the `SourceSpan` of their nearest parser-origin ancestor… Diagnostics that truly have no real source mapping are internal compiler diagnostics and use `SIFR-INTERNAL-*`; do not fabricate a source span." |
| Y9 🟡 | Severity enum locked | ✅ Resolved | milestone_diag_1 DoD: "Define the canonical `Severity` enum exactly as `Error \| Warning \| Note \| Help`; internal diagnostics use `Severity::Error`." |
| Y10 🟡 | Warning accounting | ✅ Resolved | "Non-Error Diagnostics" section pins shared envelope, shared compact grouping, exit-code policy, and 50-cap behavior. |
| Y11 🟡 | Generic-bound vs type-mismatch examples | ✅ Resolved | Three concrete generic examples added under "Family ownership rules". |
| Y12 🟡 | Phase 27 amend, Phase 39 anchor | ✅ Resolved | "Do not reopen Phase 27" stated. Stability Policy anchors post-1.0 to "Phase 39 stable-channel GA". |
| Y13 🟡 | milestone_diag_11 reframe | ✅ Resolved | "Required guardrails" subsection frames duplicates as enforcement assertions, not redo'd work. |
| Y14 🟢 | Non-error path chosen | ✅ Resolved | "This phase uses one diagnostic stream for errors, warnings, notes, and help." Alternative dropped. |
| Y15 🟢 | `SourceId` opaque | ✅ Resolved | `pub struct SourceId; // Opaque, cheaply cloneable implementation detail.` |
| Y16 🟢 | Exact workspace renumbering | ✅ Resolved | "Exact workspace renumbering" table pins `0001..0004` → `6001..6004`, `0101..0103` → `6101..6103` (split preserved). |
| Y17 🟢 | Re-export deletion deadline | ✅ Resolved | "Any re-exports must be removed by `diag_4b`." |

The no-fallback / no-historical-compatibility stance is preserved. All round-1 strengths (semantic-vs-phase identity, family overlap rules, hard guardrails, per-milestone fixture ownership, ChildSeverity discipline) remain intact.

---

## 2. Residual non-blocking risks

These are flagged for awareness during implementation. None of them prevent `milestone_diag_1` from starting.

### R1. 🟡 Migration coverage for non-HIR surfaces is implicit

The "Existing Surface Inventory" enumerates eleven emission surfaces, including:

- Parser-to-`CompileError` conversion paths in `sifr_driver`.
- Project/workspace discovery diagnostics.
- Build/materialization/rustc diagnostics.
- Codegen panic and error boundaries.
- Test-runner diagnostics.

The migration milestones (`diag_6`/`diag_7`/`diag_8`) explicitly cover only HIR-side and `sifr_type_system` emission. The remaining surfaces — parser (`SIFR-PARSE-*`), codegen (`SIFR-CODEGEN-*`), build (`SIFR-BUILD-*`), workspace (`SIFR-WORKSPACE-*` after renumbering), and test runner — have no named migration milestone. Their migration is *implied* by `diag_4a`/`diag_4b` (renderers consume `SifrDiagnostic`, then phase-to-code mapping is deleted), but the scope wording does not say so.

In practice the implementer of `diag_4` will be forced to migrate these emission paths to satisfy `diag_4a`'s "renderers consume `SifrDiagnostic`" contract. That is not a wrong outcome, but it makes `diag_4` larger than its scope reads. Two implementers may also disagree about whether parser/build/codegen migration belongs to `diag_4a`, `diag_4b`, or a tail of `diag_8`.

Recommendation (during `diag_3`): the inventory milestone produces the full surface list; the proposal could be tightened in a follow-up edit to either (a) add an explicit "non-HIR surfaces" migration sub-step inside `diag_4a`, or (b) extend `diag_8`'s scope wording to "remaining HIR semantic diagnostics *and any non-HIR emission surface still using a phase-derived public code*". Either is fine; pick one when the inventory output is concrete.

This does not block `diag_1`. By the time it matters (start of `diag_4a`), the inventory exists and the question can be answered from data rather than from the proposal alone.

### R2. 🟡 Sequencing tension between `diag_5` (grammar cleanup) and `diag_6` (decimal migration)

`diag_5` removes acceptance of bare `[Edddd]` expectation codes from the e2e fixture grammar. `diag_6` is the decimal migration that converts `[E25xx]` message-embedded pseudo-codes to top-level `SIFR-DECIMAL-25xx`. The Mermaid graph orders `diag_5 → diag_6`.

The per-milestone fixture-ownership rule says "No migration milestone is complete until its fixtures, verification baselines, and focused tests are green with the new codes." After `diag_5` lands and before `diag_6` begins, any fixture whose primary expectation is a bare `[E25xx]` code (rather than `SIFR-TYPE-0001` plus a secondary message-embedded `[E25xx]`) will fail.

The cleanest resolutions:

1. Audit during `diag_3` whether any current fixture uses `[Edddd]` as the *primary* expectation (vs. as a secondary message classifier). If none do, `diag_5` can land safely.
2. If any do, the affected fixtures must be temporarily converted to `SIFR-TYPE-0001` in the same `diag_5` PR (still legal at that point — `diag_6` migrates them onward).
3. Alternatively, swap `diag_5` and `diag_6`. The cost is small: `diag_6` then has to cope briefly with the harness still accepting both grammars.

This is a real implementation-time decision but it is a small, local one. It does not need to be resolved in the proposal text; the inventory at `diag_3` will surface it concretely.

### R3. 🟡 `SIFR-CODEGEN-0001` and `SIFR-BUILD-0001` retirement is left optional

The renumbering table reads:

> `SIFR-CODEGEN-0001` — Retired or narrowed to a specific `SIFR-CODEGEN-7001` code.
> `SIFR-BUILD-0001` — Retired or narrowed to a specific `SIFR-BUILD-8001` code.

"Retired or narrowed" leaves a degree of freedom. Two implementers will resolve it differently — one will retire the codes and add specific new ones, the other will renumber `0001` to `7001` and call it done. That divergence is small, but it would cost a re-review cycle if it shows up across multiple PRs.

Recommendation: pick one phrasing. Either "Retired; replaced by specific `SIFR-CODEGEN-7xxx` codes assigned during `diag_4a`/`diag_2b`" (preferred — consistent with the no-catch-all stance) or "Renumbered to `SIFR-CODEGEN-7001` as the catch-all panic-boundary code." The first is more aligned with the document's elsewhere-stated principle that broad codes are reserved for `SIFR-INTERNAL-*` only.

### R4. 🟢 STDLIB sub-range allocation policy is not seeded

"Each stdlib module should receive a reserved contiguous sub-range, preferably 50 codes at a time, tracked in the diagnostic registry." The active range is `5200..5599` (8 sub-ranges of 50). The proposal does not allocate the first one (e.g., `sifr.math` → `5200..5249`, `sifr.io` → `5250..5299`).

This is fine to leave for `diag_2b` once the inventory has identified which stdlib modules currently emit static API contract errors. Worth noting only because the registry skeleton in `diag_2a` may want to seed at least one example sub-range for shape verification.

### R5. 🟢 50-cap recovery interaction with `reveal_type`

"The 50 top-level recovery cap applies to all top-level diagnostics after severity ordering, while the existing user-error exit behavior remains based on whether any top-level diagnostic has `Severity::Error`."

A user with 60 `reveal_type(...)` calls (`Severity::Note`) and zero errors would see only the first ≤ 50 reveal-type notes after severity ordering pushes errors first. That is probably the intended behavior — cap is a hard anti-flooding limit. But the proposal could be explicit that the cap is intentional for non-error diagnostics too, or carve out an exemption for `Severity::Note` reveal-type output (which users explicitly ask for).

Either choice is defensible. Worth a one-line clarification in `diag_10` (Recovery Semantics) when implementation gets there. Not a current blocker.

### R6. 🟢 `diag_2a` docs generator runs against an empty registry

`diag_2a` adds the docs generator binary; `diag_2b` populates active codes. Between the two, the generator runs against a registry containing only reserved family bases — its output will be the index plus reserved-only stub pages.

This is harmless but may briefly check in `docs/errors/diagnostic-codes.md` showing only family reservations. The proposal could clarify that `diag_2a`'s generator output is intentionally a skeleton and `diag_2b` produces the first content-bearing pages. Trivial; mention only because reviewers of the `diag_2a` PR may flag the empty pages without that context.

### R7. 🟢 Family range gaps are unlabeled

The family table has unassigned numeric gaps:

- `1800..1999` (between `SIFR-IMPORT-*` and `SIFR-TYPE-*`)
- `6500..6999` (between `SIFR-WORKSPACE-*` and `SIFR-CODEGEN-*`)
- `7500..7999` (between `SIFR-CODEGEN-*` and `SIFR-BUILD-*`)
- `8500..8999` (between `SIFR-BUILD-*` and `SIFR-INTERNAL-*`)

These are presumably reserved for future families. The convention is consistent ("the family base is reserved"; gaps imply future families) but not stated explicitly. A one-line note ("Numeric gaps between family ranges are reserved for future family allocation; the registry treats them as unassigned, not as catch-all space") would prevent a future implementer from filling a gap with codes from an existing family.

---

## 3. Strengths preserved

These are correct and should not be touched in any subsequent edit:

- The semantic-vs-phase identity policy ("Design Principle").
- The no-fallback / no-historical-compatibility stance throughout Hard Rules and Non-Goals.
- The accumulator emission model with `LoweringOutcome { result, diagnostics }` and `LowerCtx::emit(...)`.
- The `SifrDiagnostic::Source(...)` / `SifrDiagnostic::Internal(...)` split with mandatory `SourceSpan` on the source variant.
- The single envelope `version` (no per-diagnostic schema version).
- `message`/`message_template`/`args` co-existing with the named-brace template grammar.
- The `(severity, code, message_template, primary file)` compact grouping rule.
- The `ChildSeverity { Note, Help }` enum and the hard rule against `Severity::Error` children.
- `sifr_diagnostics` as the sole owning crate for canonical diagnostic types, with the `diag_4b` deletion deadline for `sifr_driver` re-exports.
- The `crates/sifr_diagnostics/src/codes.rs`-as-source-of-truth registry with named generator binary and drift check.
- The "Type System Integration" subsection rejecting `impl From<TypeError> for SifrDiagnostic>` as a long-term design.
- The exact workspace renumbering table (split structure preserved).
- The Phase 27 amendment stance ("Do not reopen Phase 27") and the Phase 39 stability anchor.
- The per-milestone fixture/baseline ownership rule preventing late mass-fixture cascades.
- The decimal migration scope explicitly covering `sifr_type_system::check`.
- The `[E2507]` negative test in `diag_5`.

---

## 4. Bottom line

The amended proposal is implementable as-is. All 17 round-2 amendments landed correctly, the no-fallback stance is intact, sequencing is consistent (`diag_1 → diag_2a → diag_3 → diag_2b → diag_4a → diag_5 → diag_6 → diag_7 → diag_8 → diag_4b → diag_9 → diag_10 → diag_11`), the data model is precise, and the inventory milestone has the right inputs (registry skeleton + tooling) and the right outputs (a populated registry).

The seven residual items in §2 are tactical, not architectural. They are appropriate to address inside the implementation PRs that would surface them (`diag_3` for R1/R2, `diag_2a`/`diag_2b` for R4/R6/R7, `diag_4a` for R3, `diag_10` for R5) rather than as gating proposal amendments.

Recommended next step: begin `milestone_diag_1`. No round 4 review is needed.
