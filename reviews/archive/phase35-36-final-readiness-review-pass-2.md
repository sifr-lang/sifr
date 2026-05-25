

## VERDICT: **READY**

All six amendments are present, executable, and properly wired. Here is the verification:

---

### Amendment 1 - Phase 35 Ruff Fork Update/Rebase Governance [OK]
- **Depends On (line 21):** policy declared - fork updates require review, `sifr_syntax` API validation, fixture revalidation, and documentation with new hash/rationale. Automated checks must fail on unpinned advances.
- **Verification infra (line 459):** `verification/performance/check_ruff_fork_update_contract.py` explicitly listed.
- **m35.4a scope (line 575):** fork update/rebase contract check added as scope item.
- **m35.4a DoD (line 581):** "fork update contract check fails on a seeded fork pin update without fixture revalidation evidence" - this is the executable gate.

### Amendment 2 - Phase 35 Syntax Token/Trivia Fixture Source-of-Truth [OK]
- **Verification infra (line 460):** `verification/performance/sifr_syntax_token_fixtures/` declared as checked-in fixtures consumed by Phase 36 syntax-asset drift checks.
- **m35.4a DoD (line 581):** "Token/trivia fixtures exist for representative syntax corpus entries" - executable fixture requirement.
- **Editor Analysis Boundary (line 246):** "checked-in `sifr_syntax` tokenization fixtures for representative corpus entries" declared as Phase 35 export. "Phase 36 editor integrations must use grammars generated from or validated against these fixtures; manually authored grammar rules without drift validation against `sifr_syntax` fixtures are forbidden" - explicit enforcement.

### Amendment 3 - Phase 36 LSP Protocol Versioning Policy [OK]
- **LSP Server Contract (lines 319-325):** Full policy: LSP 3.17 locked, `lsp-types` version pinned in `Cargo.lock` and documented, upgrade requires reviewed PR with compatibility rationale, capability adoption criteria, advertisement guardrail, deferral rationale requirement. Matches the amendment text exactly.

### Amendment 4 - Phase 36 Diagnostic Rule Lifecycle Policy [OK]
- **Diagnostics section (lines 353-359):** Full policy: stable rule ids, deprecation retention with replacement naming, patch/minor release default levels, experimental rule requirements, two-minor-release backward compatibility, Sifr-owned metadata. Matches the amendment text exactly.

### Amendment 5 - Phase 36 VS Code Extension Versioning Covenant [OK]
- **VS Code Extension Contract (lines 452-458):** Full covenant: explicit coupling or documented independence, corresponding release or version range in metadata, release gated on contract check + smoke tests + build/test/package, Phase 39 owns publication, silent skip forbidden. Matches the amendment text exactly.

### Amendment 6 - Phase 36 Analysis Snapshot Coherence Verification [OK]
- **Verification infra (line 482):** `verification/tooling/check_analysis_snapshot_coherence.py` declared - verifies no stale `sifr_frontend` results, rejects stale revision publications, preserves `InvalidationReport` boundary.
- **m36.3 DoD (line 630):** "Snapshot coherence validation proves `AnalysisHost` snapshots reflect the latest `FrontendContext` revision, stale document versions are rejected at the snapshot boundary, invalidated `sifr_frontend` queries cannot produce results through `sifr_analysis` snapshots, and no `sifr_analysis` query method exposes a result whose source revision differs from the snapshot's captured revision" - exact executable gate from amendment.
- **m36.8 DoD (line 693):** coherence check in scope for closeout.
- **Exit criteria (lines 771, 772):** `check_analysis_snapshot_coherence.py` listed as required check with pass/fail-on-seeded-violations behavior.

---

### Remaining blockers before sequential implementation?

**None.** The six amendments are implemented in executable terms. The dependency chain is clean: Phase 35 exits before Phase 36 begins, `milestone_36_1` includes a Phase 35 export confirmation gate, and the split-brain guardrails are extendable from Phase 35 into Phase 36 tooling paths.

---

### Non-blocking notes (implementation guidance, no blockers)

1. **Phase 35 m35.4a audit scope:** The mechanical extraction audit before API skeleton work is a one-pass check - treat it as a discovery step, not a design discussion. The split is already defined: `sifr_syntax` then `sifr_frontend`.

2. **`ty_completion_eval` decision timing:** Phase 36 should resolve the completion ranking infrastructure question in `milestone_36_1` before `milestone_36_3` starts implementation. The framework is referenced in Phase 35/36 contracts but the Sifr-specific ranking approach is not yet designed.

3. **Baseline capture clean-worktree procedure:** The measurement protocol is correct. Implementation should add a short procedural note to `internal_docs/performance_budgets.md` (Phase 35 implementation doc) that baseline captures require a clean toolchain with no cache-contaminated prior runs - recoverable from the host variance guidance but easy to overlook.

4. **Phase 37 handoff risk:** Phase 37 does not yet declare it needs LSP-based package-aware intelligence (completion across packages, auto-import from registries, workspace symbols across boundaries). This is not a Phase 35/36 blocker, but Phase 37 planning should address it early.
