

Now I have everything I need. Let me deliver the verdict.

---

## VERDICT: READY

I am satisfied. Here is my full audit against all five review questions:

---

### Q1: Does `tooling_reuse_strategy.md` perform the audit sufficiently for planning, or is it still hand-wavy?

**Sufficient.** The document has been substantially upgraded since the early rethink memo. Key evidence:

- The Sources Reviewed section (lines 11-41) documents both external docs (ty features pages for diagnostics, language server, rules, suppression, exclusions) and concrete local crate paths (`ty_server/src/`, `ty_ide/src/`, `ty_project/`, `ruff_db/`, `sifr_diagnostics/`).
- The Decision Matrix (lines 84-108) provides granular `reuse-direct` / `adapt` / `adapt-with-review` / `reference-only` / `reject` classifications for every relevant crate, with explicit rationale.
- The Diagnostic Strategy (lines 110-135) covers canonical ownership, what to adopt from ty/Ruff, and what not to adopt - including explicit rejection of `ruff_db::Diagnostic` as Sifr's core type.
- The Rule, Suppression, and Exclusion Strategy (lines 136-192) addresses diagnostics, rules, suppression, and exclusions as a unified product concept, distinguishing hard correctness from policy rules, specifying suppression syntax (`# sifr: ignore[rule-id]`), and defining exclusion behavior.
- The Accepted Dependency Graph (lines 243-262) and Implementation Guidance (lines 264-286) give concrete Phase 35/36 direction.
- The Verification Requirements (lines 287-296) define what must be reflected in Phase 35/36.

The document no longer hedges or hand-waves. It makes definite calls on every non-trivial architectural decision.

---

### Q2: Are the reuse decisions right?

**Yes. Confirmed by local code inspection.**

| Decision | Strategy says | Code confirms |
|---|---|---|
| `lsp-server` | `reuse-direct` | Generic JSON-RPC/LSP transport, no Python semantics. |
| `lsp-types` | `reuse-direct` | Pure LSP data types, no semantics. |
| `ty_server` shell patterns (init, capability negotiation, document model, diagnostics lifecycle, scheduler) | `adapt` / `adapt-with-review` | Examined `session.rs`: Session owns a `ProjectDatabase` (salsa) typed to Python semantics. Document index is tightly coupled to project state through `LSPSystem`. Clean extraction requires replacing the `db` field. The `adapt` classification correctly identifies this as a pattern to adapt, not direct code to copy. |
| `ty_ide` query surface (completion, hover, goto) | `reference-only` | Examined `completion.rs` and `hover.rs`: functions call `SemanticModel::new(db, ...)`, `ty_python_semantic` types, and Python-specific APIs like `ty_module_resolver`. Direct code would import Python semantics. Correctly classified `reference-only`. |
| `ty_project` as project database | `reject` | Deeply Python-specific: `ProjectDatabase` ties to Python environment discovery, module resolution, source types, and config files. |
| `ty_python_semantic` as semantic/rule engine | `reject` | All editor queries depend on `ty_python_semantic::SemanticModel`. Not reusable for Sifr. |
| Sifr diagnostics canonical | `keep` | Examined `sifr_diagnostics/src/lib.rs`: stable schema, canonical codes, renderer views, child notes, suggestions, exit-code contracts. Correctly kept as canonical. |

The architectural pipeline is sound:
```
sifr_syntax -> sifr_frontend -> sifr_analysis -> sifr_lsp
  (direct)              (direct)        (direct)      (adapts ty_server shell,
                                                     reuses lsp-server/lsp-types directly)
```

---

### Q3: Are ty diagnostics/rules/suppression/exclusion concepts incorporated correctly without weakening Sifr's hard correctness guarantees?

**Correctly incorporated.**

The strategy makes an explicit, hard split between two diagnostic categories:

1. **Hard correctness diagnostics** (lines 141-149): parse errors, soundness-critical type errors, ownership/move/borrow errors, `Result`/`Option` safety errors, runtime-panic-prevention errors, workspace/import errors. These are **not suppressible** and cannot be downgraded to warning. This directly preserves Sifr's "if it compiles, it works" guarantee.

2. **Policy rules** (lines 150-152): unused code/imports, unreachable-code warnings, migration advisories, style-adjacent static analysis. These **may be** configurable (`ignore`/`warn`/`error`) only if doing so does not violate Sifr's core guarantee.

The suppression shape (`# sifr: ignore[rule-id]`) is Sifr-specific, not copied from Python `type: ignore` or Ruff `noqa`. The strategy explicitly says:
- Unknown rule ids produce a diagnostic
- Unused suppression comments produce a diagnostic (like ty's `unused-ignore-comment`)
- Python `type: ignore` must not suppress Sifr diagnostics by default
- Hard correctness errors are forbidden from suppression

This is the correct treatment. No path exists to suppress a hard correctness error through the configured suppression syntax.

---

### Q4: Are Phase 35 and 36 now consistent with the strategy, especially removing the future open-ended audit milestone and treating the strategy as a planning source of truth?

**Consistent and confirmed.**

**Phase 35 - confirmed:**

- Line 14: `internal_docs/tooling_reuse_strategy.md` is the reviewed planning input for Phase 36. Phase 35 must leave syntax/frontend/diagnostics boundaries compatible.
- Line 28: "The split-brain guardrail mechanism created in this phase must be extendable by Phase 36 to reject Python semantic dependencies in tooling and LSP paths, including `ty_python_semantic`, `ty_project` Python project semantics, Python module-resolution semantics, and Python environment discovery."
- The guardrail is explicitly structured so Phase 36 can extend it to tooling dependency checks without rewriting the core mechanism (line 94).

**Phase 36 - confirmed:**

- Line 12: The phase file says `internal_docs/tooling_reuse_strategy.md` is the reviewed reuse audit and decision matrix. Implementation PRs may add supporting docs but must not conflict with this phase file or the strategy.
- Lines 72-78: The Architecture Ownership section explicitly maps the strategy's reuse decisions into the phase: `lsp-server`/`lsp-types` as `reuse-direct`, `ty_server` patterns as `adapt`, `ty_project` as project database and `ty_python_semantic` as semantic engine as `reject`.
- Line 290: "Implementation must execute milestones in order unless a later reviewed PR updates this file with rationale. The ty/Ruff reuse audit has already been performed in `internal_docs/tooling_reuse_strategy.md`; implementation milestones must follow that strategy rather than reopen the audit as exploratory work."

The future open-ended audit is gone. The strategy is a planning source of truth, not a future deliverable. The phase documents own the decisions, not defer them.

---

### Q5: What exact edits, if any, are required before this is production-grade planning?

**No edits required.** I found no gaps, contradictions, or missing cross-references between the three files. All five review questions are satisfied.

The document chain is complete:
- `tooling_reuse_strategy.md` - the audit and decision matrix
- `35_performance_benchmarking_and_budgets.md` - references the strategy as reviewed planning input; extends guardrail for Python semantic dependencies
- `36_developer_tooling_and_ecosystem_hooks.md` - owns the strategy as source of truth, embeds reuse decisions in Architecture Ownership, removes open-ended audit

The review chain is also complete: the strategy has been reviewed twice (pass 1 and pass 2), both reviews verified structural completeness and identified the needed phase doc changes, and the phase docs now reflect those changes.

---

**I am satisfied. The planning is production-grade.**
