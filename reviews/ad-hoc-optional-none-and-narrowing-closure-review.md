## Review: `ad-hoc-optional-none-and-narrowing-closure.md`

### Verdict: **Mostly Ready**

The phase document has improved significantly over the prior `optional-none-category-breakdown`. The workstream decomposition, cross-stream dependencies, guardrails, and execution ledger are well-structured. It is one strong PR away from being fully implementation-ready.

---

### Findings (ordered by severity)

**1. Technical approaches are orientation-level, not implementation sketches**
Every workstream's "Technical approach" paragraph is a restatement of *what* needs to happen, not *how* the compiler mechanics would change. An engineer starting `workstream_1` (the foundational lane) needs at minimum: which HIR or type-system files own the CFG fact propagation, what the per-program-point fact structure looks like, and how narrowed facts are consumed at expression/type-checking time. Currently every approach reads as direction, not procedure.

**2. Wave acceptance targets are underspecified**
The "Acceptance target" per wave is a single sentence. For wave 1 this says "dominated use site still sees `T | None`" but does not define: what constitutes a "dominated use site" in Sifr's current CFG model, what the narrowed type at that site must actually be after the fix, or what the failure mode looks like at the HIR layer (e.g., is it an expression-check failure, a type-join failure, a codegen failure?). Without this, an engineer cannot write a unit test that asserts the correct behavior.

**3. No test inventory or testing approach at the workstream level**
The "Validation" section per workstream names categories (unit tests, e2e fixtures, LeetCode reruns) but provides no concrete test locations, existing test patterns to follow, or isolation strategy. Given that HIR has maintainability guardrails and e2e fixtures are discovered lexicographically, engineers need to know: which existing test files to extend, what the snapshot update workflow is, and how to prevent narrowing fixes from regressing non-optional paths.

**4. Wave status sections are empty templates**
The execution ledger has 5 wave sections, each with `status: pending` and placeholder fields for "validation to record." While this is expected for a planned phase, the phase document never defines what intermediate status reporting looks like, what "wave root cause landed" means in practice, or how reclassification decisions get made before starting the next wave. This creates a risk of the phase drifting without a crisp closeout trigger.

**5. Representative fixture list is incomplete for some lanes**
Wave 3 (`container_element_refinement`) lists only `0023_merge_k_sorted_lists` and `0115_distinct_subsequences` as representative, but the category breakdown analysis suggests 8–12 fixtures. The execution ledger should either enumerate the full set or explicitly track which fixtures are being used as canaries versus which are full targets.

**6. No effort or complexity sizing**
Impact is described as "roughly N fixtures" with no complexity classification. This makes it impossible to judge whether waves 3 and 4 can truly run in parallel, or whether the residual lane (wave 5) will fit within the phase scope. Even a rough 3-tier sizing (simple/medium/hard) per workstream would improve planning fidelity.

---

### Minimum Changes Required

1. **Per-wave acceptance target**: Expand each wave's "Acceptance target" into 2–3 concrete behavioral statements with example HIR-level diagnostics (before/after).
2. **Technical approach depth**: Add 1–2 sentences per workstream identifying the primary code location (specific files/modules in `sifr_hir` or `sifr_type_system`) and the mechanism (e.g., "carry `OptionFact` through branch entry/exit blocks in the CFG; consume at expression checking in `expr.rs`").
3. **Test isolation strategy**: Add a brief paragraph per workstream describing the relevant existing test files and the unit/e2e split. Reference `sifr_hir` unit tests for CFG fact propagation and `sifr/tests/e2e/` fixtures for regression coverage.
4. **Wave 3 fixture inventory**: Either expand the representative list to cover the full 8–12 fixture estimate or explicitly separate canary fixtures from full targets.
5. **Complexity sizing**: Add a 3-tier effort tag (small/medium/large) per workstream to enable parallel sequencing decisions.

---

### What Works Well

- Guardrails and "What this phase must not do" correctly protect Sifr's explicit safety model and ownership semantics.
- The workstream decomposition is architecturally sound and crisply separated from unrelated categories.
- The cross-stream dependency table correctly identifies `wave_1` as foundational and `wave_5` as strictly post-closure.
- The phase integration loop (land → rerun representative → rerun full corpus → reclassify → next wave) is the right process.
- The "Scope" section clearly excludes truthiness redesign, `nonlocal`, and fixture-first strategies.
- The execution ledger provides a clean template for tracking wave-by-wave progress.
