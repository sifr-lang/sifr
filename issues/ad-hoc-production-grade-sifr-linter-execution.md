# Ad Hoc Phase Execution: Production-Grade Sifr Linter

Status: planning

Phase contract: `issues/ad-hoc-production-grade-sifr-linter.md`

## Checklist

- [ ] Phase plan reviewed and approved for implementation
- [ ] Ruff linter reuse manifest created
- [ ] Forbidden Ruff/Python lint dependency guardrail completed
- [ ] Lint config and file discovery completed
- [ ] Parser-aware suppression engine completed
- [ ] Phase-gated lint runner completed
- [ ] Sifr policy rule families completed
- [ ] Fix engine and LSP code actions completed
- [ ] LSP/editor docs and contracts updated
- [ ] Full local validation recorded
- [ ] Final production-readiness review approved

## Planning Lock Addendum

This phase locks the lint/Ruff reuse decisions before implementation starts. Changing a reuse classification, config surface, suppression rule, rule namespace, or LSP action policy requires a reviewed planning update.

### Required Implementation Work

| ID | Work item | Required closeout |
| --- | --- | --- |
| W-1 | `sifr_lint` uses line-only suppression attachment. | Milestone 3 implements parser-aware statement/range suppression before syntax, HIR, or workspace rules ship. |
| W-2 | `sifr_lint` file discovery uses simple string/path matching. | Milestone 2 adapts Ruff-style glob/gitignore file discovery with Sifr defaults. |
| W-3 | Lint configuration is not loaded from `sifr.toml`. | Milestone 2 implements Sifr-owned lint config and override precedence. |
| W-4 | The lint runner is not phase-gated. | Milestone 4 implements Ruff-inspired phase gating behind Sifr APIs. |
| W-5 | LSP code-action gating can drift if it relies on diagnostic-code strings. | Milestone 6 adds typed hard-vs-policy diagnostic class and policy-only action gates. |
| W-6 | Fix-capable policy rules lack a production fix engine. | Milestone 6 adds applicability, edit isolation, conflict resolution, source-map tracking, and idempotence checks. |
| W-7 | The Ruff/Python lint reuse contract is not machine-enforced yet. | Milestone 1 adds `verification/tooling/check_linter_reuse_contract.py` with positive and negative self-tests. |
| W-8 | The parser-aware suppression gate is advisory until made mechanical. | Milestone 1 creates the gate manifest; Milestone 3 enables it; Milestone 5 cannot add non-physical-line rules unless the gate is closed. |

### Locked Reuse Decisions

| Area | Locked decision |
| --- | --- |
| Ruff Python rules | Reject as production dependencies |
| Ruff rule registry contents | Reject; Sifr owns rule IDs/categories |
| Ruff config architecture | Adapt pattern, not Python options |
| Ruff file discovery | Reuse/adapt `ignore`, `globset`, path normalization, explicit-target behavior |
| Ruff linter orchestration | Adapt phase-gated structure |
| Ruff AST checker | Reference only; Sifr syntax/HIR checker is native |
| Ruff suppression engine | Adapt mapping/directive lookup concepts with Sifr syntax and rule IDs |
| Ruff fix engine | Adapt applicability/isolation/apply-fixes concepts |
| Ruff LSP code actions | Adapt deferred resolution, workspace edit tracking, settings patterns |
| Ruff Server diagnostics | Reference only; Sifr diagnostics remain canonical |

## Review Log

- `2026-05-26`: Claude review pass 1 found the high-level reuse boundary sound but identified parser-aware suppression as a blocker before adding non-line rules.
- `2026-05-26`: Claude review pass 2 confirmed the revised strategy is sound if parser-aware suppression is a documented gate before syntax/HIR/workspace rules.
- `2026-05-26`: Claude review pass 3 cross-checked current code and confirmed the reuse boundary is clean, with parser-aware suppression as the known prerequisite gate.
- `2026-05-26`: Claude subsystem reviews covered Ruff config, registry/rules, lint engine, suppression/fixes, file discovery/cache/path utilities, and LSP/editor integration. Findings are incorporated in the phase reuse matrix and milestones.
- `2026-05-26`: Claude phase review pass 1 found two planning blockers: the forbidden Ruff/Python lint dependency check needed a named enforceable guardrail, and the parser-aware suppression prerequisite needed a mechanical gate before syntax/HIR/workspace rules. The phase was updated to require `check_linter_reuse_contract.py`, a suppression-gate manifest, and rule-family enforcement.
- `2026-05-26`: Claude phase review pass 2 found the suppression-gate manifest and M3-to-M5 enforcement path were still underspecified. The phase was updated to define `verification/tooling/linter_manifests/suppression_gate.json`, its schema, the `physical_line_only` to `parser_aware` transition, and a single compile-time parser-aware suppression API dependency for non-physical-line rules.
- `2026-05-26`: Claude phase review pass 3 confirmed all pass-2 blockers are resolved and the phase is implementation-ready with no remaining blockers.

## Validation Log

- Validation evidence will be recorded per implementation milestone.
- Planning PR validation starts with `git diff --check` and docs/review artifact checks.

## PR Log

Implementation PR links will be recorded here as each milestone closes.
