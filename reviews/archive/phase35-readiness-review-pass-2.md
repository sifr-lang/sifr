# Phase 35 Readiness Review: Pass 2

**Reviewer:** Claude Code review agent
**Date:** 2026-05-15
**Phase Document:** `internal_docs/phases/35_performance_benchmarking_and_budgets.md`
**Prior Review:** `reviews/phase35-readiness-review-pass-1.md`
**Status:** READY

---

## Verdict: READY

Phase 35 has been substantially rewritten. All three pass-1 blocking gaps are closed with concrete specifications. The remaining items are minor documentation consistency issues, not implementation blockers. The phase is ready for implementation.

---

## Pass-1 Blockers: Status

### BLOCKING-1 (Shared Query Architecture) — CLOSED

The phase doc now provides full Rust-like pseudo-signatures covering `FrontendContext`, `ModuleId`, `ModuleGraphView`, `ModuleGraphNode`, `ModuleGraphEdge`, `InvalidationReport`, `QueryKind`, `FrontendInput`, `ProjectRoot`, `SourceHash`, `GraphRevision`, and all query methods. The 8-step invalidation algorithm with explicit cache key components (lines 148–168) is a concrete specification, not prose intent. Phase 19 integration is explicit (lines 168, 293). The `sifr_frontend` target crate is unambiguously named (lines 38–55).

**One non-blocking gap remains:** `QueryResult` is used as a return type throughout the API section but is never defined. The implementer needs to decide whether `QueryResult<T>` is a `Result<T, E>` (with explicit error typing) or a `Result<T, QueryError>` wrapper, and what cache-hit vs cache-miss semantics it exposes. This is a Rust API design decision the implementer must make; the phase doc is not required to prescribe it.

### BLOCKING-2 (Benchmarking and Budget Enforcement) — CLOSED

Each of the three benchmarking milestones (35_1, 35_2, 35_3) is now a concrete implementation plan with explicit file paths, concrete threshold formulas, and explicit waiver governance:

- `manifest.json`, `baselines.json`, `budgets.json`, `waivers.json` with required fields
- `run_benchmarks.py` and `check_budgets.py` as the canonical runners
- Threshold formula: `max(baseline_median * 1.10, baseline_median + 25ms)` for median latency (line 230), `max(baseline_p95 * 1.15, baseline_p95 + 50ms)` for p95 (line 231), `max(baseline_peak_rss * 1.10, baseline_peak_rss + 32MiB)` for RSS (line 232)
- Waiver required fields: id, owner, issue, created, expires, benchmark_ids, budget_ids, override, rationale, removal_criteria
- `check_budgets.py` rejection criteria enumerated explicitly (lines 255–261)
- Enforcement integration into `scripts/run_all_tests.sh` with `quick` and `pr` profiles (lines 326–337)

The `phase27-non-regression` corpus group (lines 203, 358) ensures diagnostics/renderer/panic-free contracts are exercised by the performance infrastructure.

### BLOCKING-3 (Milestone Dependency Ordering) — CLOSED

The milestone sequencing diagram (lines 269–281) explicitly requires `m35.4a` before `m35.1`. The text confirms: "Benchmarks must measure the canonical frontend path, not ad hoc compiler paths" (line 283). The dependency graph is concrete.

---

## Remaining Non-Blocking Issues (Ranked)

### NB-1: `QueryResult` Type Not Defined (Minor)

**Location:** Lines 75–81, API section
**Issue:** `QueryResult<ParsedModuleView<'_>>` and related return types are used throughout but `QueryResult` itself has no definition in the phase doc.
**Impact:** Low — implementer decides. The type needs to distinguish cache hit from recompute and expose error vs. empty-success semantics. This is a straightforward Rust API design decision.
**Recommendation:** Note it in the phase doc's non-goals or add a one-line "QueryResult is a typed result wrapper whose error variant covers cache-internal errors only; query unavailability due to unloaded modules is expressed through specific `ModuleNot*` error variants" to prevent implementer ambiguity.

### NB-2: Stability Limit Value Not Specified (Minor)

**Location:** Line 226
**Issue:** "fail baseline capture if coefficient of variation exceeds the configured stability limit" — the configured stability limit is never given a concrete value.
**Impact:** Low — implementer can pick a sensible default (e.g., CoV > 0.10 or > 0.15). The architecture doc references `criterion` which has implicit defaults. But if the phase doc intends a specific policy, it should state the value.
**Recommendation:** Add "(recommended default: CoV ≤ 0.10)" or "stability limit is defined in `verification/performance/budgets.json` with a recommended default of 0.10" so the threshold is findable even if the value is adjustable per-case.

### NB-3: Guardrail Mechanism for m35.4b Split-Brain Detection is Hand-Wavy (Minor)

**Location:** Line 350 ("Static or scripted guardrails catch new parse/lower/type-check/semantic diagnostic paths added outside `sifr_frontend` or approved HIR internals")
**Issue:** "static or scripted guardrails" is vague. Is this a compile-time lint (`#[deny]`), a script that greps the crate list, a test that asserts all HIR query paths go through `sifr_frontend`? The phase doc does not specify the mechanism.
**Impact:** Low — m35.4b is the final milestone and the implementer will discover this during implementation. But if the guardrail is just a grep script, it can be bypassed. A Rust-level compile-time enforcement (e.g., a sealed trait on `sifr_hir` query entrypoints that only `sifr_frontend` can access) would be stronger.
**Recommendation:** Note that the guardrail mechanism must be reviewed as part of m35.4b design. At minimum, the phase doc should state whether it expects a code-level constraint or a script-level check.

### NB-4: Negative Seeds Location Slightly Ambiguous (Minor)

**Location:** Line 190 ("`verification/performance/negative_seeds/` — seed inputs or result fixtures proving budget and waiver gates fail when expected")
**Issue:** The line says `negative_seeds/` is a directory, but `run_benchmarks.py` and `check_budgets.py` likely produce or consume seeds from it. The interaction between seeds and the runner scripts needs clarity.
**Impact:** Negligible.
**Recommendation:** Add one sentence: "Negative seeds are JSON fixtures consumed by `check_budgets.py` that inject known-regression benchmark results to verify gate failure behavior."

### NB-5: "Warmup Count" vs "Discard Explicit Warmup Samples" Contradiction (Minor)

**Location:** Lines 222–224
**Issue:** Line 222 says "at least five warm measured samples." Line 224 says "discard explicit warmup samples." Both appear in the Measurement Protocol. The first says warm samples are measured; the second says explicit warmup samples are discarded. These could be interpreted as consistent ("discard explicit warmup, keep warm measured samples") or contradictory ("don't measure warmup, then measure warm samples — which?").
**Impact:** Negligible — the intent is clear: measure warm samples, don't run a separate warmup phase and discard it. The phrasing just needs a small cleanup.
**Recommendation:** Clarify: "Warm samples are measured directly without a separate warmup phase. Discard any explicit warmup runs that were used only for setup evidence (line 222)."

---

## Required Concrete Edits

**None.** All blocking gaps from pass-1 are closed. The phase doc is implementation-ready without required edits. The five items above are non-blocking improvements for the implementer to address during execution, not pre-conditions for starting.

---

## Non-Blocking Improvements

### NI-1: Phase 36 Anti-Split-Brain Cross-Reference

The phase doc states Phase 36 "must consume the canonical Phase 35 frontend/query API" (line 23) and the exit gate mentions "anti-split-brain foundation" (line 347). But there is no explicit cross-reference to Phase 36's milestone_36_1 requirement that "disallow semantics reimplementation in tool-specific paths." The cross-reference would tighten the contract and make the Phase 35/36 boundary explicit.
**Effort:** One sentence in the Feeds Into section.

### NI-2: Cross-Reference Phase 34 Corpus for Reuse Clarity

Lines 206 and 305 say the corpus "reuse representative Phase 34 generated-code-quality fixtures where possible." The Phase 34 manifest is at `verification/generated_code_quality/manifest.json`. A direct path reference would prevent implementers from hunting for it.
**Effort:** One parenthetical "(e.g., `verification/generated_code_quality/manifest.json`)".