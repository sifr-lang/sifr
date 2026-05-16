# Phase 35 + Phase 36 Tooling Readiness Review — Pass 2

## Review Context

- Reviewer: Claude (automated tooling readiness audit, round 2)
- Date: 2026-05-16
- Branch: `codex/phase-35-readiness-review`
- Target phases: `35_performance_benchmarking_and_budgets.md`, `36_developer_tooling_and_ecosystem_hooks.md`
- Prior review: `reviews/phase35-36-tooling-readiness-review-pass-1.md`
- Scope: Evaluate whether pass-1 blockers are closed and whether the two phases are architecturally elegant enough for implementation toward production-grade LSP/editor tooling.

---

## 1. Verdict

**READY.** Phase 35 and Phase 36 in their current revised form are architecturally sound and ready to proceed to implementation. All five pass-1 blockers are structurally addressed. One residual concern (medium severity) exists around migration mechanical risk, but it is a known, well-scoped technical challenge that does not constitute a contract gap.

The two phases now constitute a coherent, self-consistent, production-grade tooling architecture. The ownership hierarchy (`sifr_syntax` → `sifr_frontend` → `sifr_analysis` → `sifr_lsp`), the LSP protocol contract (3.17 over stdio, full-document sync, required vs optional capabilities), the VS Code extension architecture (semantics-delegation model), the parity corpus (diagnostics + editor queries, 8 explicit cases), the verification infrastructure (5 required files + `lsp-query` performance cases), and the split-brain guardrails (anti-split-brain rules at each layer) are all defined with sufficient precision for implementation without inventing behavior.

---

## 2. Remaining Blocking Gaps

### [CONCERN-1] `sifr_frontend` migration mechanical risk underestimated

**Severity: Medium (not a blocker, but real)**

**Location:** Phase 35 `m35.4a` + `m35.4b` scope.

**Observation:** Phase 35 now provides the 6-step migration path and correctly splits `m35.4a` (API skeleton + cache contract) from `m35.4b` (CLI adoption + regression lock). This is the right structure. However, the scope of `m35.4a` still aggregates several mechanically distinct operations:

1. Creating `crates/sifr_syntax/` as a stable wrapper around the Ruff fork parser/AST/trivia/span surface
2. Creating `crates/sifr_frontend/` with the full API surface
3. Extracting and repackaging existing functions from `sifr_driver/src/frontend/api.rs`, `module_lowering.rs`, and project frontend helpers
4. Maintaining a temporary facade while `sifr_driver` is updated
5. Adding the split-brain guardrail
6. Writing unit tests

Items 1-3 are routine refactoring when the source is well-structured. Items 3-4 carry the highest mechanical risk: extracting a query/cache API from a working driver risks introducing behavioral divergence during the transition unless the extraction is done as a pure mechanical rename with no semantic changes. The risk is compounded if `sifr_driver/src/frontend/` has implicit dependencies on `sifr_driver`-local state (build orchestration, rustc invocation, artifact tracking) that need to be decoupled before extraction.

**Assessment:** This is a medium-severity implementation concern, not a contract gap. The phase contract is correct; the migration plan is explicit; the split-brain guardrail will catch behavioral drift. The concern is that `m35.4a` could take longer than estimated if the source code has tighter coupling than the plan assumes.

**What would eliminate this concern:** A pre-implementation audit of `sifr_driver/src/frontend/` to confirm the extraction surface is clean (no `sifr_driver`-local state in the frontend functions), plus a commitment to the "pure mechanical rename" discipline during extraction. This audit should be done as a pre-milestone task, not as part of `m35.4a` scope.

**Recommended action:** Add a pre-milestone note in `m35.4a` scope: *"Before starting this milestone, audit `sifr_driver/src/frontend/` to confirm extraction surface is mechanically clean. If coupling is tighter than planned, split `m35.4a` into `m35.4a-1` (syntax wrapper) and `m35.4a-2` (frontend API) and adjust timeline accordingly."*

**Severity rating: 2/5.** Not a blocker because the phase contract is correct and the concern is scoped to implementation risk, not contract correctness.

---

## 3. Required Concrete Edits

No blocking edits required. The two phases are self-consistent and ready.

One non-blocking recommended edit:

**Edit for Phase 35 `m35.4a` scope:** Add the pre-milestone audit note described in CONCERN-1 above. This is a process-level risk mitigation, not a contract change.

---

## 4. Non-Blocking Improvements

### NBI-1: `sifr_frontend` to `sifr_analysis` handoff documentation

Phase 35's "Editor Analysis Boundary For Phase 36" section (lines 197-211) defines the required Phase 35 exports for Phase 36. Phase 36's `AnalysisHost` API references `sifr_frontend` types. However, there is no explicit documentation of what happens when `sifr_analysis` queries `sifr_frontend` for data that is not yet available (e.g., `DefId` not exposed, symbol table not fully built, type inference results not stable enough). The "documented gap" language exists ("or a documented gap for Phase 36 to close in `sifr_analysis`") but there is no actual list of gaps.

**Recommendation:** During `m36.1`, produce `internal_docs/tooling_analysis.md` as the first artifact. It should include: the `AnalysisHost` API implementation notes, the data required from `sifr_frontend` vs data that must be derived, the HIR view contracts, and the explicit gap list for Phase 36 to close.

### NBI-2: `SourceUri` in `SourceFileView` is optional but likely required for LSP

Phase 35 defines `SourceFileView.uri: Option<SourceUri>`. For LSP, every open file needs a `file://` URI. Making this optional means tooling consumers must handle `None`, which is correct behavior, but Phase 35 does not document when `None` is valid vs when it is a gap. LSP tooling will always need `Some(uri)`.

**Recommendation:** Document the `SourceUri` guarantee: for any file that has been opened in a tooling session, `uri` must be `Some`. The `None` state is valid only for files that have been loaded as project dependencies but not yet opened in an editor. This is a pre-condition for `sifr_lsp` correctness.

### NBI-3: LSP budget enforcement interaction with Phase 35 waiver policy

Phase 36 defines default interactive budgets (e.g., completion median <= 200ms) but notes they are "phase-start defaults" to be recorded in `verification/performance/budgets.json`. Phase 35's waiver policy governs `budgets.json` entries. The interaction is: Phase 36 adds `lsp-query` budget ids to `budgets.json`, and Phase 35's `check_budgets.py` enforces them. This is correct, but the waiver policy in Phase 35 (lines 326-351) does not document that LSP budgets are in scope.

**Recommendation:** Add to Phase 35 Waiver Policy: *"LSP-query budget waivers follow the same policy as CLI benchmark waivers. LSP budget ids are added by Phase 36 and governed by the same `check_budgets.py` enforcement. Phase 35's waiver policy applies to all budget ids in `budgets.json` regardless of phase origin."*

### NBI-4: `sifr_analysis` crate name decision in `m36.1` should be guided by existing naming conventions

The phase correctly leaves the final crate name to `milestone_36_1`. The name choice is constrained by workspace conventions (all Sifr crates use `sifr_*` prefix). `sifr_analysis` is the preferred name per the phase doc. `sifr_ide` is also valid. The decision criteria are not documented, which is fine for the phase contract, but the implementation team should pick within the first week of `m36.1` to avoid inconsistency.

**Recommendation:** Add a `milestone_36_1` scope note: *"The final crate name must be chosen within the first three working days of this milestone to avoid inconsistency in crate structure and imports. The decision must be documented in the milestone tracking issue with rationale."*

### NBI-5: LSP cancel and concurrency model is underspecified for multi-file projects

Phase 36 states "Request handling may be single-threaded in the MVP" and "document updates and query requests must be serialized." This is sufficient for correctness but not for performance at multi-file project scale. A single-threaded LSP server will block on type-checking a large module while serving hover on a small module.

**Recommendation:** Document this as a known MVP limitation with a follow-up criteria: *"MVP uses single-threaded request serialization. Multi-threaded request handling with per-file task isolation is a future improvement that requires explicit memory-ownership planning for the query cache. This improvement is out of scope for Phase 36."*

---

## 5. Explicit Satisfaction Statement

**All five pass-1 blockers are closed:**

- **BLOCKER-1** (sifr_frontend missing): Resolved. Phase 35 now has explicit `sifr_syntax` + `sifr_frontend` architecture ownership, migration path (6-step), milestone split (`m35.4a`/`m35.4b`), and `FrontendContext` API contract with all required types. One medium-severity concern remains about migration mechanical risk (CONCERN-1 above).

- **BLOCKER-2** (LSP query interface missing): Resolved. Phase 36 defines `sifr_analysis` ownership boundary, `AnalysisHost` with full editor query API (8 queries: diagnostics, completion, hover, definition, references, document symbols, semantic tokens, inlay hints), the anti-split-brain rules, and the `sifr_frontend` → `sifr_analysis` → `sifr_lsp` layer hierarchy.

- **BLOCKER-3** (VS Code extension absent): Resolved. Phase 36 adds the full VS Code Extension Contract section with explicit ownership model (extension owns grammar/filetype/launcher, delegates semantics to `sifr lsp --stdio`), binary discovery, syntax highlighting strategy, and `m36.4` milestone.

- **BLOCKER-4** (LSP workloads absent from benchmark corpus): Resolved. Phase 35 reserves `lsp-query` budget IDs and defines the `interactive-tooling-foundation` corpus group. Phase 36 adds the complete `lsp-query` benchmark cases (7 cases) with default budgets and the Phase 35 extension mechanism.

- **BLOCKER-5** (Parity corpus underspecified): Resolved. Phase 36 `milestone_36_2` explicitly enumerates 8 editor-query parity cases (completion locals, completion member/stdlib, hover, go-to-definition local, go-to-definition imported, document symbols, semantic tokens, stale version) plus 7 diagnostics cases and all coverage dimensions (codes, URLs, spans, help, suggestions, renderer outputs, symbol kinds, definition spans, semantic-token ordering, LSP severity mapping).

**Architecture quality:**

The two phases now define a coherent, self-consistent production-grade tooling architecture:

- `sifr_syntax` wraps the Ruff fork with an explicit ownership boundary
- `sifr_frontend` owns parse/lower/type-check/diagnostics with a deterministic query/cache/invalidation model
- `sifr_analysis` owns editor-oriented queries derived from `sifr_frontend` and approved HIR views
- `sifr_lsp` is a thin LSP 3.17 protocol adapter with full-document sync and explicit forbidden semantic behaviors
- VS Code extension delegates all semantics to `sifr lsp --stdio` with no type checker, parser, or HIR traversal
- Verification infrastructure covers parity, protocol smoke, split-brain guardrails, and LSP performance budgets
- Cross-phase contracts are explicit: Phase 35 provides `sifr_frontend`, Phase 36 consumes it and builds `sifr_analysis` + `sifr_lsp`
- The exit gate for Phase 35 is: canonical frontend/query/cache established and CLI-adopted. The exit gate for Phase 36 is: editor tooling capability proven and split-brain-resistant.

**Non-blocking concerns:**

All four non-blocking improvements (NBI-1 through NBI-5) are documentation and process-level enhancements that do not affect the architecture's correctness or completeness. They can be addressed during implementation.

**Conclusion:** Phase 35 and Phase 36 are ready for implementation. Proceed.

---

## Summary Scorecard (Pass 1 → Pass 2)

| Dimension | Pass 1 | Pass 2 | Status |
|---|---|---|---|
| `sifr_syntax` wrapper layer | Missing | Defined with ownership, must-not-own list, migration source | CLOSED |
| `sifr_frontend` crate + API | Missing | Defined with migration path, API contract, milestone split | CLOSED (migration risk flagged) |
| `sifr_analysis`/`sifr_ide` boundary | Missing | Defined with `AnalysisHost` API, ownership list, anti-split-brain rules | CLOSED |
| `sifr_lsp` thin adapter | Partial | Defined with protocol contract, required/optional capabilities, document sync model | CLOSED |
| VS Code extension | Missing | Defined with ownership model, binary discovery, grammar strategy, `m36.4` milestone | CLOSED |
| LSP benchmark workloads | Missing | `lsp-query` cases with default budgets, Phase 35 extension mechanism | CLOSED |
| Parity corpus | Diagnostics only | 8 editor-query cases + 7 diagnostics cases + full coverage dimensions | CLOSED |
| Source/session/document-sync model | Missing | `FrontendContext` source map, document version, `InvalidationReport` consumption, concurrency model | CLOSED |
| Split-brain guardrails | Defined but enforcement unclear | Anti-split-brain rules at every layer, `check_split_brain_guardrail.py`, `check_lsp_split_brain.py` | CLOSED |
| LSP performance budget integration | Missing | Default budgets, Phase 35 extension mechanism, waiver policy alignment | CLOSED |
| Migration mechanical risk | N/A | Medium — extraction surface audit recommended before `m35.4a` | CONCERN-1 |
| `sifr_frontend` to `sifr_analysis` handoff docs | N/A | Gap list needed during `m36.1` | NBI-1 |
| `SourceUri` optional guarantee | N/A | `Some(uri)` must hold for opened files | NBI-2 |
| LSP budget waiver governance | N/A | Phase 35 waiver policy should explicitly cover `lsp-query` ids | NBI-3 |
| `sifr_analysis` crate name decision deadline | N/A | First-3-days constraint recommended | NBI-4 |
| Multi-file LSP concurrency model | N/A | Single-threaded MVP limitation documented | NBI-5 |

**Pass 2 verdict: READY.** Proceed to implementation.