# Review: Ad-hoc Phase — Codegen Runtime Build Gap Closure (Pass 1)

Reviewer: agent
Date: 2026-04-05
Source: `issues/ad-hoc-codegen-runtime-build-gap-closure-phase-2026-04-05.md`
Context: `issues/codegen-runtime-build-gap-root-cause-breakdown-2026-04-05-v3.md`

## Verdict: READY

All four validation criteria pass. Three minor suggestions below — none block implementation.

---

## 1. Ready-to-implement validation

| Criterion | Status | Notes |
|---|---|---|
| Clear scope per workstream | PASS | Each workstream has explicit family mapping, fixture list or anchor cases, and lane ownership. |
| Sequencing defined | PASS | 5 workstreams + full-corpus rerun, ordered with priorities. |
| Definition of done per workstream | PASS | Each workstream has concrete exit criteria (compile+run, regression tests, deterministic diagnostics). |
| Per-wave validation protocol | PASS | Targeted fixture rerun + quick profile test suite. |
| Phase exit gate | PASS | Fresh corpus artifact, fresh taxonomy artifact, `codegen_runtime_build_gap = 0`, recategorization with evidence for any residuals. |
| Required deliverables | PASS | Execution log, updated results JSON, updated taxonomy JSON, closure report. |
| Ready-to-implement checklist | PASS | 8-item checklist covers all workstreams and phase exit. |

## 2. Count and lane consistency with v3 breakdown

### Root-cause family counts

| Family | v3 | Phase | Match |
|---|---|---|---|
| `recursive_field_surface_leaks_to_codegen_without_gate` | 21 | 21 (ws2) | YES |
| `type_contract_emission_gap` | 20 | 20 (ws1) | YES |
| `ownership_and_borrow_emission_gap` | 6 | 6 (ws3) | YES |
| `other_codegen_build_gap` | 4 | 4 (ws4) | YES |
| `binding_scope_and_capture_emission_gap` | 3 | 3 (ws3) | YES |
| `runtime_oracle_canonicalization_needed` | 2 | 2 (ws5) | YES |
| `codegen_production_panic_missing_structured_emission` | 1 | 1 (ws4) | YES |
| `truthiness_bool_lowering_gap` | 1 | 1 (ws4) | YES |
| **Total** | **58** | **58** | **YES** |

### Resolution lane counts

| Lane | v3 | Phase | Match |
|---|---|---|---|
| `compiler_fix` | 35 | 35 | YES |
| `both` | 21 | 21 | YES |
| `sifr_adaptation` | 2 | 2 | YES |
| **Total** | **58** | **58** | **YES** |

### Workstream scope totals

| Workstream | Claimed size | Families covered | Computed size | Match |
|---|---|---|---|---|
| ws1 (type contract) | 20 | type_contract_emission_gap | 20 | YES |
| ws2 (recursive field) | 21 | recursive_field_surface_leaks | 21 | YES |
| ws3 (ownership+binding) | 9 | ownership_and_borrow(6) + binding_scope(3) | 9 | YES |
| ws4 (resilience+bool) | 6 | other_codegen(4) + panic(1) + truthiness(1) | 6 | YES |
| ws5 (oracle) | 2 | runtime_oracle | 2 | YES |
| **Grand total** | **58** | — | **58** | **YES** |

Per-case mapping verified: all 58 entries in v3 per-case list map to exactly one workstream with no orphans and no double-counting.

## 3. Missing workstream check

No critical missing workstream found.

- All 8 root-cause families are assigned to exactly one workstream.
- All 58 fixtures appear in the v3 per-case mapping and are covered by exactly one workstream.
- The 7 `NO_RUST_CODE` cases are distributed correctly: 4 in ws4 (other_codegen), 1 in ws4 (panic), 2 in ws5 (oracle).
- The phase exit gate requires `codegen_runtime_build_gap = 0` with explicit recategorization evidence for any residual failures, which closes the completeness loop.
- No fixture is left unowned. No family is left without a workstream.

## 4. Architecture consistency — nonlocal mutable capture unsupported

| Check | Status |
|---|---|
| Locked decision stated in Architecture Decisions section | YES |
| ws3 explicit guardrail: "Keep nonlocal mutable capture unsupported decision intact" | YES |
| ws3 DoD permits deterministic diagnostic failures (not codegen-build-gap) for unsupported forms | YES — consistent with the decision |
| Phase exit gate permits recategorization of intentionally-unsupported cases | YES — "recategorized outside this phase with explicit evidence and owner" |
| No workstream proposes broadening capture semantics | CONFIRMED |

Architecture consistency is maintained. The phase correctly distinguishes between fixing invalid codegen (in scope) and broadening language semantics (out of scope).

---

## Minor Suggestions (non-blocking)

### S1. Justify ws4-before-ws3 sequencing

The sequencing orders `workstream_crbg_4` (P1) before `workstream_crbg_3` (P1). Both are P1. The phase doc should add a one-line rationale for the swap, e.g.:

> ws4 before ws3 because ws4 fixes are isolated (panic removal, NO_RUST_CODE residual, single bool-lowering case) and provide quick signal, while ws3 requires more care around the nonlocal-capture architecture boundary.

This prevents implementers from questioning or reordering without context.

### S2. Name the recategorization target for ws3 diagnostic-only outcomes

ws3 DoD says fixtures may "fail with intentional deterministic diagnostics that are not codegen/runtime-build-gap failures." The phase exit gate says these are "recategorized outside this phase." Consider naming the expected target category explicitly (e.g., `unsupported_semantic` or `intentional_unsupported_capture`) so the final taxonomy refresh has a clear destination label rather than an ad-hoc one.

### S3. Disambiguate NO_RUST_CODE sentinel across lanes

In the v3 breakdown, `NO_RUST_CODE` appears in three distinct contexts:
- **other_codegen_build_gap** (0394, 0513, 0838, 1609): compiler failed to produce a Rust artifact.
- **codegen_production_panic** (0662): compiler panicked before emitting code.
- **runtime_oracle** (1968, 2215): build succeeded and code ran — no Rust compile error codes present.

The v3 doc clarifies the first group but not the third. Adding a one-line note in the phase doc or v3 doc that `NO_RUST_CODE` in the `runtime_oracle` lane means "no Rust compile error (build succeeded)" would prevent confusion during implementation, since the same sentinel superficially looks like "no code was generated."

---

## Summary

The phase document is well-structured, internally consistent, and fully aligned with the v3 root-cause breakdown. All 58 cases are covered with no gaps. The architecture decision on nonlocal mutable capture is correctly preserved as a guardrail. The three minor suggestions above improve clarity but do not block implementation.

**READY**
