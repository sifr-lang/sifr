

Let me verify each of the four requested changes against the actual phase documents.

**A. `milestone_36_0` with explicit scope, decision matrix, spike deliverable, and definition of done**

Phase 36 milestones section (lines 269-286) has:
- [ok] Named milestone with detailed scope auditing `ty_server/src/session.rs`, `ty_server/src/server.rs`, `ty_server/src/document/`, `ty_ide/src/`, `ty_project/`, `ruff_server/`
- [ok] Classification categories: `reuse-direct`, `reference-only`, `reject`
- [ok] Creates `internal_docs/tooling_reuse_strategy.md` as first artifact with decision matrix, evidence, accepted/forbidden dependency graph
- [ok] Spike: wire mock/minimal Sifr `AnalysisHost` through selected LSP shell path, specifically testing separation from Python semantics
- [ok] Spike removal/convert clause before phase exit
- [ok] Definition of done: reviewed decision matrix, spike proves/disproves clean separation, doc records rationale, forbidden dep graph documented, guardrail covers Python dependencies

**B. `lsp-server`/`lsp-types` as first-class direct dependencies, shell reuse audit-gated**

Architecture Ownership section lines 57-69:
- [ok] "must use `lsp-server` and `lsp-types` as the direct LSP protocol foundation"
- [ok] "These crates are generic protocol/data-type dependencies, carry no Python semantics, and are already used by the Sifr Ruff fork's `ty_server`"
- [ok] "The open reuse question... is whether audited `ty_server` session, document, request-queue, cancellation... patterns can be adapted cleanly without importing Python semantic or project assumptions"
- [ok] Forbidden dependencies explicitly listed: `ty_python_semantic`, Python module-resolution semantics from `ty_project`, Python environment discovery, Python diagnostic rules, `ruff_server`/`ty_server` semantic paths

**C. `internal_docs/tooling_reuse_strategy.md` as required documentation/artifact**

- [ok] Required documentation section (lines 211): "`internal_docs/tooling_reuse_strategy.md` documents audited `ty_server`, `ty_ide`, `ty_project`, and `ruff_server` reuse decisions and forbidden dependency boundaries"
- [ok] `milestone_36_0` scope: "Create `internal_docs/tooling_reuse_strategy.md` as the first Phase 36 artifact. It must contain the decision matrix, evidence for each classification, accepted dependency graph, forbidden dependency graph, and follow-up implementation plan."
- [ok] Phase 36 exit criteria: file "records the audited reuse decision before LSP implementation"

**D. Phase 35 extends split-brain guardrail to guard Python semantic dependencies in tooling/LSP paths**

- [ok] Feeds Into section lines 22-29: "The split-brain guardrail mechanism created in this phase must be extendable by Phase 36 to reject Python semantic dependencies in tooling and LSP paths, including `ty_python_semantic`, `ty_project` Python project semantics, Python module-resolution semantics, and Python environment discovery"
- [ok] Frontend Migration Path line 92: guardrail "must be structured so Phase 36 can extend it to tooling dependency checks without rewriting the core mechanism"

All four changes are present and correctly implemented.

**Checking for new contradictions**

Milestone ordering: spike -> analysis -> parity -> LSP -> VS Code - correct.

No-fallback vs spike: Quality Contract requires "no fallback, migration, or legacy compatibility code" and the spike DoD explicitly requires spike removal/convert before phase exit - consistent. The spike is a time-bounded test, not a permanent fallback path.

Direct-dependency wording: settled and explicit throughout.

Forbidden dependency graph: documented in both Architecture Ownership and `milestone_36_0` DoD, consistently covering the full list.

User goal alignment: Option D rejected as Sifr's brain but aggressively mines `ty_server` for protocol/session architecture - still supports smart/powerful tooling without unnecessary work.

**Verdict: READY.**

The four requested changes are all satisfied. The phase docs now own the audit milestone with concrete scope, state `lsp-server`/`lsp-types` as settled direct dependencies, require `tooling_reuse_strategy.md` as an artifact, and extend Phase 35's guardrail to cover Python semantic dependencies in tooling/LSP paths.
