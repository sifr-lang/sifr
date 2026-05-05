# Phase-Closure Review: Ad-Hoc Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics

**Branch:** `codex/diag-11-phase-closure`
**Review date:** 2026-05-03
**Phase:** 31.7
**Review round:** Pass 1

---

## Phase-Contract Verification

The phase-closure goals stated in the scope are verified as follows:

### Goal: No user-facing semantic diagnostic may use SIFR-TYPE-0001 as a catch-all

**Status:** PASS

- `rg "SIFR-TYPE-0001" crates/ --glob '*.rs'` returns **no matches** in tracked Rust source.
- The `SIFR-TYPE-0001` catch-all was retired before the 1.0 no-compatibility decision (documented in the issue file at `milestone_diag_4a` slice 2b.33, PR #1705).
- The `diagnostic_emission_inventory.md` retains `SIFR-TYPE-0001` in its historical provenance table as a note about the original 95-instance inventory, but this is clearly labeled as a historical snapshot (April 29, 2026) and does not represent current code state.

### Goal: No HIR user diagnostic should be emitted through raw ctx.error(String)

**Status:** PASS

- `rg "ctx\.error\(" crates/sifr_hir/src --glob '*.rs'` returns **no matches**.
- The final migration slice (comprehension/generator/walrus, PR #1784) explicitly verified this and removed the raw `LowerCtx::error` transport.
- The `check_diagnostic_transport_cleanup.py` guardrail runs clean with no output.

### Goal: LoweringError should not exist in tracked Rust source as active transport

**Status:** PASS

- `rg "LoweringError" crates/ --glob '*.rs'` returns **no matches** in tracked Rust source.
- PR #1754 renamed the transport to `HirDiagnostic` and the guardrail script `check_diagnostic_transport_cleanup.py` rejects re-introduction of retired transport symbols.
- All driver adapter and test surfaces were updated to matching `hir_diagnostic_*` terminology.

### Goal: Diagnostic schema/docs/code coverage/baseline hygiene/cancel usage/transport cleanup guardrails enforce the new architecture

**Status:** PASS

All six guardrails run clean (no output / exit 0):
- `check_diagnostic_schema_sync.py` — PASS
- `check_diagnostic_docs_sync.py` — PASS
- `check_diagnostic_code_coverage.py` — PASS
- `check_diagnostic_baseline_hygiene.py` — PASS
- `check_diagnostic_cancel_usage.py` — PASS
- `check_diagnostic_transport_cleanup.py` — PASS

### Goal: Docs accurately state completed current state, not historical raw-HIR inventory as current

**Status:** PASS with one non-blocking note

The `diagnostic_emission_inventory.md` contains two sections that could cause confusion:
- **Closure snapshot** (lines 12-16): Accurately states current state — "no matches" for raw HIR emissions, raw `LowerCtx::error` deleted.
- **Historical coverage snapshot** (lines 5-10): Accurately labeled as historical (April 29, 2026).
- **Target Code and Fixture Plan** table (lines 302, 305-306): Contains `fixture pending in milestone_diag_7` notes. These are stale provenance notes from the original inventory creation — `milestone_diag_7` has long since been completed (PRs #1714-#1718). However, since these are explicitly labeled with `milestone_diag_7` (a completed milestone), they read as historical plan notes rather than current pending work, and do not represent open TODOs.

The issue doc is the authoritative phase-closure record and is fully current, with every milestone item checked off and PR references.

### Goal: Roadmap marks phase 31.7 completed

**Status:** PASS

`internal_docs/roadmap.md` line 57:
```
| 31.7 | Ad Hoc Semantic Diagnostic Code Taxonomy and Structured HIR Diagnostics | completed | ... | Corrective Phase 27 addendum completed on 2026-05-03: ...
```

---

## Evidence Summary

| Check | Evidence | Result |
|---|---|---|
| `SIFR-TYPE-0001` catch-all | `rg --glob '*.rs'` zero matches | PASS |
| Raw `ctx.error(` in HIR | `rg crates/sifr_hir/src -g '*.rs'` zero matches | PASS |
| `LoweringError` symbol | `rg --glob '*.rs'` zero matches | PASS |
| Transport cleanup guardrail | `check_diagnostic_transport_cleanup.py` exit 0 | PASS |
| Schema sync guardrail | `check_diagnostic_schema_sync.py` exit 0 | PASS |
| Docs sync guardrail | `check_diagnostic_docs_sync.py` exit 0 | PASS |
| Code coverage guardrail | `check_diagnostic_code_coverage.py` exit 0 | PASS |
| Baseline hygiene guardrail | `check_diagnostic_baseline_hygiene.py` exit 0 | PASS |
| Cancel usage guardrail | `check_diagnostic_cancel_usage.py` exit 0 | PASS |
| Roadmap phase 31.7 | `internal_docs/roadmap.md` line 57 shows `completed` | PASS |
| Issue doc closure | All milestones checked, PRs cited, validation evidence recorded | PASS |
| Quick validation | `scripts/run_all_tests.sh --profile quick` passed (51.53s wall, 31 e2e pass) | PASS |

---

## Non-Blocking Observations

1. **Stale `fixture pending in milestone_diag_7` notes in inventory table:** These are clearly historical plan labels, not current open work. `milestone_diag_7` is long-completed. They appear in a provenance table and do not represent pending items.

2. **Group-skew advisory in validation output:** The quick validation lane reports `advisories=group skew is high; investigate batching balance or fixture clustering`. This is a pre-existing condition unrelated to this phase and noted in prior validation runs.

3. **Warm-cache hit-rate advisory:** Similarly pre-existing and noted in prior passes; not phase-related.

---

## Conclusion

**No blocking issues remain.**

The phase-closure contract is satisfied:
- No catch-all `SIFR-TYPE-0001` in active source.
- No raw `ctx.error(` in HIR.
- No `LoweringError` transport symbol.
- All six diagnostic guardrails pass.
- Roadmap accurately reflects completion.
- Issue doc is a complete, current record of all merged PRs and validated evidence.
- Quick validation passes.

This phase is ready for final closure PR.
