

Review complete. Written to `reviews/phase37_workspace_mono_review.md`.

**Verdict: not-ready**

Six critical gaps were identified that block workspace/monorepo implementation:

1. **Virtual workspaces** — undefined; will fail on the most common Cargo monorepo pattern
2. **`default-members` / `exclude`** — inconsistent `--workspace` behavior vs Cargo
3. **`[workspace.dependencies]`** — unaddressed; breaks common shared-dependency monorepos
4. **Path dependency semantics** — compilation order, cycle detection, namespace isolation undefined
5. **Subdirectory lock behavior** — undefined; common user ergonomics failure
6. **Root `sifr.toml` vs per-package `sifr.toml`** — architectural contradiction between `sifr_workspace_design.md` and Phase 37's delegation model

Plus four moderate gaps (diagnostics for ambiguous selection, mixed Sifr/Rust workspaces, LSP behavior, selector flag completeness) and four nits (demo repos all flat single-package, `sifr outdated` underspecified, test coverage gaps, milestone DoD vagueness). The section-level edit table gives exact line-level fixes for each.
