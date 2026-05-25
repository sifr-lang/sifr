

Based on my thorough review of Phase 35, Phase 36, the tooling reuse strategy, the VS Code extension execution checklist, and the dependency chain in the roadmap, here is my final readiness assessment:

---

## VERDICT: **NOT READY**

---

## Implementation-Ready If Done Sequentially?

**Yes** - with the amendments below. The sequential lock, milestone boundaries, exit gates, and Phase 35's export confirmation requirement in `milestone_36_1` are correctly structured. But 6 gaps require concrete text additions before Phase 35 begins.

---

## Hidden Long-Term Maintainability Gaps

### Gap 1 (Phase 35): Ruff Fork Rebase/Update Governance Missing

Phase 35 declares the Sifr Ruff fork as "the canonical parser/AST/trivia/source-span substrate" but specifies no update/rebase policy. The fork is currently based on Ruff 0.15.12. Every upstream Ruff update risks breaking `sifr_syntax`, `sifr_frontend`, tokenization fixtures, and every Phase 35/36 feature that depends on them.

**Concrete text addition** - add to Phase 35 "Depends On" section, after the existing fork bullet:

```
- The Sifr Ruff fork update/rebase policy is defined before Phase 35 exits: upstream Ruff version bumps must be reviewed, validated against `sifr_syntax` API surface, verified against all Phase 35 `sifr_syntax` fixtures and Phase 36 syntax asset drift checks, and documented with the new upstream version hash and migration rationale. Fork updates that change parser behavior, AST shape, trivia semantics, or token classification must trigger a full Phase 35/36 re-validation before the fork update merges. Automated checks must fail when the fork's Cargo.lock or rust-toolchain.toml advances without corresponding fixture revalidation.
```

### Gap 2 (Phase 35): `sifr_syntax` Generated Syntax Asset Source-of-Truth Not Declared

Phase 36 requires that grammars be "generated from or validated against `sifr_syntax`/the Sifr Ruff fork tokenization fixtures," but Phase 35 never declares it as an obligation. `sifr_syntax` tokenization output must be the checked-in fixture source for all generated syntax assets (TextMate, Tree-sitter, VS Code grammar). No editor integration may use a grammar that bypasses `sifr_syntax` fixtures.

**Concrete text addition** - add to Phase 35 "Editor Analysis Boundary For Phase 36" section, in the required exports list:

```
- `sifr_syntax` tokenization fixture fixtures (checked-in per-module token sequences with trivia) serve as the authoritative source-of-truth for all generated syntax assets (TextMate, Tree-sitter, VS Code grammar). Phase 35 must produce these fixtures for representative corpus entries. Phase 36 editor integrations must use grammars generated from or validated against these fixtures; manually authored grammars without a drift-validation test against `sifr_syntax` fixtures are forbidden.
```

### Gap 3 (Phase 36): LSP Protocol Versioning and Upgrade Policy Missing

Phase 36 locks on LSP 3.17. There is no policy for: (a) when to adopt a new LSP version, (b) backwards-compatibility handling for clients on older LSP versions, (c) what happens when the target LSP version adds new capabilities Sifr should adopt, (d) deprecation of old protocol behavior.

This gap will surface at first upstream `lsp-types` upgrade and again at every LSP spec change.

**Concrete text addition** - add to Phase 36 "LSP Server Contract" section, before "Explicitly unsupported protocol surfaces":

```
## LSP Protocol Versioning Policy

- The target LSP version is locked to LSP 3.17 at phase exit and must be recorded in `internal_docs/lsp_server.md` with the exact `lsp-types` crate version pinned in `Cargo.lock`.
- Upstream `lsp-types` version bumps require a reviewed PR that documents: which new capabilities are adopted, which are deferred, how backwards compatibility with older LSP clients is maintained, and which protocol matrix entries change.
- New LSP capabilities are adopted only when: Sifr's semantic model has a meaningful answer for the capability, the implementation does not require Python/ty semantics, and the capability has positive and negative protocol test coverage.
- A capability must not be advertised in server capabilities unless it passes protocol smoke tests locally.
- Deferring a new LSP capability requires a documented rationale, not silence.
```

### Gap 4 (Phase 36): Cross-Repository VS Code Extension Versioning Covenant Missing

Phase 36 recommends a separate `sifr-lang/sifr-vscode` repository. The `check_vscode_extension_contract.py` validates the contract, but there's no explicit versioning covenant: when `sifr` releases version `X.Y.Z`, what is the required extension version alignment, and what happens when the extension lags the main repo?

This gap will surface at first release and at every release thereafter.

**Concrete text addition** - add to Phase 36 "VS Code Extension Contract" section, after "Publication readiness":

```
## Extension Versioning Covenant

- The VS Code extension version must be explicitly coupled to the Sifr compiler version or have a documented version independence policy.
- When the main `sifr-lang/sifr` repository releases version `X.Y.Z`, the extension must either: (a) release a corresponding version with an explicit compatibility statement, or (b) document a supported version range in the extension's marketplace metadata.
- Extension releases must be gated on: the extension contract check passing, `sifr lsp --stdio` smoke tests passing with the new Sifr version, and the `check_vscode_extension.py` build/test/package sequence passing.
- Extension versioning policy is owned by Phase 39 release governance, but the contractual coupling to Sifr version numbers must be established in this phase and locked before `milestone_36_7` closes.
- The extension must not silently skip validation when the main Sifr version advances.
```

### Gap 5 (Phase 36): Diagnostic Rule Lifecycle Policy Missing

Phase 36 correctly distinguishes hard-correctness from policy-rule diagnostics, implements `# sifr: ignore[rule-id]` suppression, and classifies severity. But there is no rule lifecycle policy: when a rule is deprecated, renamed, experimental, or promoted from experimental to stable. No mention of which rules can be added in patch vs minor releases.

This gap will surface the first time a rule is deprecated or when an experimental rule needs promotion.

**Concrete text addition** - add to Phase 36 "Diagnostics, Rules, Suppressions, And Exclusions" section, after the existing diagnostic split description:

```
## Diagnostic Rule Lifecycle Policy

- Every Sifr diagnostic rule id is stable once it ships in a release. Deprecated rules must: retain their id for backward-compatible suppression comments, emit a deprecation notice in rule metadata, and document the replacement rule id.
- New rules added in patch releases must be `off` by default or have an explicit stability label. New rules added in minor releases may be `warn` by default if they do not conflict with existing code behavior.
- Experimental rules are allowed only with an explicit `experimental` status label and documentation URL. Experimental rules may be removed without deprecation warning if they prove unmaintainable.
- `sifr: ignore[deprecated-rule-id]` must continue to work for at least two minor releases after deprecation and must produce an actionable "use new-rule-id instead" diagnostic.
- Rule metadata (id, summary, docs URL, default level, status) belongs in `sifr_diagnostics` or a Sifr-owned rule registry. Rule metadata must not be sourced from `ty_python_semantic` or any Python semantic dependency.
```

### Gap 6 (Phase 36): `sifr_analysis` Snapshot and Cache Coherence Validation Missing

Phase 35 establishes the `sifr_frontend` query cache with explicit invalidation semantics. Phase 36's `sifr_analysis` introduces `AnalysisHost` snapshots. There is no explicit coherence validation between `sifr_frontend` query cache state and `sifr_analysis` snapshot state - specifically, that `sifr_analysis` snapshots never expose stale `sifr_frontend` results.

This matters because `sifr_analysis` wraps `sifr_frontend` for editor queries; if the snapshot discipline is violated, diagnostics and navigation could return stale results silently.

**Concrete text addition** - add to Phase 36 "AnalysisHost" scope in `milestone_36_3` definition of done:

```
- A snapshot coherence validation test exists that verifies: `AnalysisHost` snapshots reflect the latest `FrontendContext` revision, stale document versions are deterministically rejected at the snapshot boundary, invalidated `sifr_frontend` queries cannot produce results through `sifr_analysis` snapshots, and no `sifr_analysis` query method exposes a result whose source revision differs from the snapshot's captured revision.
```

Also add to Phase 36 verification infrastructure:
```
- `verification/tooling/check_analysis_snapshot_coherence.py` - verifies `sifr_analysis` snapshots cannot publish stale `sifr_frontend` query results, rejects stale revision publications, and validates the `InvalidationReport` boundary between `sifr_frontend` and `sifr_analysis`.
```

---

## Gaps in Phase Sequencing

### Monitoring Item (Not Blockers): Phase 37 Package Management Handoff Risk

Phase 37 "depends on" Phase 36 but its single milestone only covers "dependency declaration, lockfile semantics, resolution workflow." Phase 37 does not mention consuming the Phase 36 LSP surface for package-aware completion, auto-import from packages, workspace symbols across package boundaries, or diagnostics for package-level errors.

This is not a Phase 35/36 gap, but a Phase 37 planning gap that could cause Phase 37 to discover it needs Phase 36 export work that Phase 36 already closed. Recommend flagging this to Phase 37 planning - no change needed to Phase 35/36.

### Monitoring Item (Not Blockers): Phase 39 Stable Release Governance

Phase 39 does not explicitly govern the LSP server version, VS Code extension version, or generated-Rust-preview command stability as part of its promotion checklist. These are release artifacts that will need version management at GA. Recommend Phase 39 planning add explicit coverage - no change needed to Phase 35/36.

---

## Overpromising, Ambiguous, or Architecture-Forcing Requirements

**None identified.** The phases are appropriately scoped:

- Phase 36 explicitly limits to "current workspace/project model" and defers package-registry intelligence to Phase 37.
- "Not an MVP phase" language is correctly enforced with the "every capability listed is required for phase exit" clause.
- Non-goals are explicit and correctly exclude: Python semantics, custom protocols, notebook support, marketplace credentials.
- The 8-milestone sequential structure with no parallel implementation rescue work is correctly enforced.

---

## Extra Phases Needed?

**No.** Phase 35 and Phase 36 are correctly contained. The work items are: Phase 35 (4a + 4b + 1 + 2 + 3 = 5 tracked milestones), Phase 36 (8 milestones). The VS Code extension is properly a `milestone_36_7` requirement, not a separate phase. The LSP protocol matrix, formatter, linter, and multi-editor assets are all inside the Phase 36 contract.

---

## Non-Blocking Notes

These do not block readiness; they are implementation guidance items:

1. **Phase 35 milestone_35_4a audit language**: "If coupling to driver-local build/artifact state is tighter than planned, split this milestone" - this is correct but the audit step could easily become a scope-creep vector. Implementation should treat it as a one-pass mechanical extraction audit, not a design discussion.

2. **Phase 36 `ty_completion_eval` adaptation**: The completion quality framework references `ty_completion_eval` patterns. Phase 36 should decide early in `milestone_36_1` whether to build the ranking infrastructure from scratch or adapt the `ty_completion_eval` test harness structure. This is a known complexity that could absorb milestone time if not scoped early.

3. **Phase 36 semantic token legend lock**: The legend in Phase 35 is already documented. Implementation should not reopen token type naming decisions - the `SifrSemanticTokenType` enum is the locked contract.

4. **Baseline measurement host variance**: The measurement protocol specifies recording host OS, architecture, Rust toolchain, and compiler binary fingerprint. This is correct. Implementation should add explicit guidance that baseline captures must use a clean Rust toolchain (no cache-contaminated prior runs) and should document the recommended clean-worktree procedure.

---

## Summary

**READY** with the 6 concrete amendments above. The phases are sequenced correctly, the anti-split-brain foundation is correctly structured, the LSP shell adaptation strategy is explicit, the extension boundary is correct, and the verification infrastructure is comprehensive. The amendments add: Ruff fork update governance, syntax asset source-of-truth declaration, LSP versioning policy, cross-repo extension versioning covenant, diagnostic rule lifecycle policy, and snapshot coherence validation. No new phases are needed. The Phase 37/39 monitoring items are advisory for downstream planning.
