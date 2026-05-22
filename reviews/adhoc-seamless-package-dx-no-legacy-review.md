Reviewing the simplification against the coherence, maintainability, and gap criteria:

**What changed:**

1. Removed the compatibility matrix (Legacy / Migrating / New rows)
2. Canonical model = `src/` + `__init__.sifr` + `sifr.toml [dependencies]` only
3. Phase 37 `sifr/` layouts and `[exports].modules` are **internal fixtures only** — no external backward-compatibility contract
4. `0701` redefined from "conflicting models" to "using `[exports].modules` in production"
5. `0708` redefined from "conflicting alias sources" to "Cargo alias metadata conflicts with canonical projection"
6. Renamed section to "Internal Fixture Migration Plan" and removed "keep `source.roots = ["sifr"]` supported" line
7. Updated guardrails language to allow Phase 37 fixtures in explicitly named internal regression tests only

**Coherence:** The changes are internally consistent. The phrase "there is no external backward-compatibility contract to preserve" is explicit and unambiguous. `0701` now has a single clear meaning. The Phase 37 model is cleanly framed as implementation substrate, not user-facing surface.

**Maintainability:** The old compatibility matrix required maintaining 3 simultaneous behavioral states with conflict-detection rules between them. Removing it eliminates `0701` conflict logic and the legacy/migrating/new conditional branches. The remaining `0701` is a simple reject rule: if you use `[exports].modules`, you get a diagnostic pointing you to `__init__.sifr`. This is a significant simplification.

**Gap analysis:**

| Area | Status |
|------|--------|
| Migration (in-tree fixtures) | Snapshot validation still derives from old model before deriving from new — intact |
| Demos | Explicitly deferred until milestone 7, stays on current internal layout until compiler integration supports new layout end-to-end |
| Guardrails | Exception exists for parser/backfill regression fixtures, which is correct |
| Implementation order | Unchanged: layout first → projection → session → compiler → run/test → hardening → publish/migration |
| Externally-created Phase 37 packages | **Assumption-based gap**: The simplification assumes no external Phase 37 packages exist. For a language still being built, this is reasonable. If it later proves wrong, the migration command's `--from sifr-rooted` path provides the recovery path. The assumption should be noted in the issue for future reference. |

**No blockers found.** The simplification is sound and the design is ready for implementation.

**READY**
