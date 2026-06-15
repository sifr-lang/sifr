## Review: Wave 4 package selection/classification baselines

**No blockers.** The slice is correctness-clean and matches the patterns established by prior Wave 4 package slices.

### What I verified

- **Diagnostic codes are real active registry entries.** `SIFR-PACKAGE-0102/0106/0601` are all `active_entry!` in `crates/sifr_diagnostics/src/codes/registry/registry_entries/package.rs:62,88,248`.
- **Commands are real public CLI paths.** The three commands wired into `area_adapter.py:500-503,506-507` (`package -p rust-helper --list --no-verify --allow-dirty`, `package -p missing --list ...`, `package --workspace --list ...`) match the user-facing `sifr package` selection/classification surface — not lower-level graph helpers.
- **Adapter wiring is consistent.** Both new commands added to `BASELINE_COMMANDS` (lines 33-34) and to the package-root `cwd` set (lines 472-473), matching the existing `package-check`/`package-workspace-list` shape.
- **Cross-file references line up.** All three fixture IDs appear in `manifest.json`, `baseline_metadata.json`, and `code_baseline_coverage.json` with no duplicates; no orphan fixture IDs; no orphan manifest cases.
- **Coverage counts match the tracker.** I computed 170 active codes / 142 with baseline / 28 deferred, with family breakdown `BUILD 5, INTERNAL 1, PACKAGE 16, STDLIB 2, WORKSPACE 4` — exactly what the twentieth-slice entry claims.
- **`source_hash` is fresh.** Computed sha256 of each fixture's `sifr.toml` and got identical values to what `baseline_metadata.json` stores (`769f43…`, `9e739f…`, `799e84…`). Coverage-check's staleness assertion (`code_baseline_coverage.py:305`) will pass.
- **Baseline payloads are minimal and exit-1.** stderr trio is `1 error, 0 warnings, 0 notes` + a single `E SIFR-PACKAGE-0XXX` line; stdout empty; exit-code `1`. Format matches the nineteenth-slice `package_workspace_duplicate_*` baselines.
- **Tracker closure is correct.** The nineteenth slice is now marked `merged via PR #2604 and tracker closeout merged via PR #2605` (was previously "pending closeout"), and the twentieth slice is added with accurate validation evidence.

### Minor (non-blocking) observations

1. `package_selector_invalid/sifr.toml` declares `name = "package_selector_ambiguous"` — the inner package-name slug doesn't match the fixture-id slug `package_selector_invalid`. Cosmetic mismatch.
2. `package_selector_invalid` ships two duplicate-named `app` Sifr workspace members (`app-one`, `app-two`, both with `[package] name = "app"`). The 0601 path under `-p missing` short-circuits before the duplicate-name check fires, so neither package is strictly required. A single member would still produce the same diagnostic — the current fixture is correct but slightly over-specified for "minimal valid".
3. Prior workspace fixtures (e.g., `package_workspace_duplicate_import_root/Cargo.lock`) include a tiny generated `Cargo.lock`. The new fixtures omit it; `--no-verify --allow-dirty` permits this and the suites pass, but it's a small pattern divergence.
4. All three `baseline_metadata` entries share the identical `bless_reason`; each could optionally name its own code (`0102` / `0106` / `0601`) for grep-ability. Convention-consistent as-is.
5. `bless_reference` uses the slug `wave-4-package-selection-classification-baselines-pr`; this is the pre-PR convention and will be promoted to a PR URL at closeout (same as nineteenth slice did).

### Recommendation

Implementation is ship-ready. Another full review round is **not required** before create-pr/merge-gate validation. If you want to address the cosmetics (fixture/package-name alignment, drop the redundant second `app-*` member), it's a small one-shot edit before opening the PR — but neither is a correctness blocker.
