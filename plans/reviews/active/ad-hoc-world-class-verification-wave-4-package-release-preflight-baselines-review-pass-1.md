## Wave 4 Package Release-Preflight Baselines — Review (pass 1)

Inspected all changed paths against runtime emitters, manifest/metadata schema validators, gitignore behavior, public CLI surface, and tracker counts. **No blockers**.

### Verified

- **Adapter wiring** (`verification/runner/sifr_verify/area_adapter.py:33`, `:473`, `:502`): `package-list-default` added to both `BASELINE_COMMANDS` and the package-root cwd switch, and emits `["package", "--list", "--no-verify", "--allow-dirty"]`. `--list/--no-verify/--allow-dirty` are public clap flags on `sifr package` (`crates/sifr/src/cli_model_and_entrypoint.rs:185-194`), and `find_package_root` (`area_adapter.py:634`) accepts these four fixtures because each has both `Cargo.toml` and `sifr.toml` at root.
- **Manifest** (`verification/areas/diagnostics/manifest.json:535-570`): all four entries in the diagnostics `baselines` suite, `expect_exit_code: 1`, `compact` only — matches sibling package slice shape.
- **Code coverage** (`code_baseline_coverage.json`): 0301/0305/0401/0403 each cleared the deferral block, set `baseline_fixture_id`, and set `renderer_formats: ["compact"]`. 0402 correctly left deferred (outside this slice's scope).
- **Baseline metadata** (`baseline_metadata.json`): four entries with valid normalizer subset, owner `compiler/frontend`, non-empty `bless_reference`/`bless_reason`. All four `source_hash` values verified against `shasum -a 256` of the fixture `sifr.toml` files — exact matches.
- **Fixture realism**: each fixture is minimal and targets exactly one code:
  - `package_untrusted_backend`: Cargo dep on local `rust-helper`, no `[trust] native` → 0301 (message matches `diag/mod.rs:308`).
  - `package_stale_trust_entry`: `[trust] native = ["missing-backend"]` with no Cargo dep of that name → 0305 (message matches `diag/mod.rs:328`).
  - `package_archive_missing_sifr_source`: `src/` contains only `lib.rs`, no `.sifr` → 0401 (`diag/package.rs:346` family).
  - `package_archive_omits_required_source`: `exclude = ["src/main.sifr"]` while the file is present → 0403 (`diag/package.rs:401`).
- **Coverage counts**: scripted count confirms 170 codes total → 146 covered / 24 deferred, deferred families BUILD 5, INTERNAL 1, PACKAGE 12, STDLIB 2, WORKSPACE 4 — exactly the tracker claim.
- **Baseline trios**: stdout (empty), stderr, exit-code all present for each fixture under `baselines/package-list-default-compact.*` (validator at `code_baseline_coverage.py:283-286` requires the full trio).
- **No accidental generated artifacts in git**: `target/` directories exist on disk in the two archive fixtures from local validation runs, but `**/target/` and `**/Cargo.lock` are in root `.gitignore`; `git ls-files -o --exclude-standard` confirms only the 6 source/baseline files per fixture are tracked. Sibling fixtures (e.g., `package_selector_invalid/Cargo.lock`) follow the same pattern.

### Non-blocking observations

- `bless_reference` is a placeholder slug (`"wave-4-package-release-preflight-baselines-pr"`) rather than a PR URL. The validator only checks non-empty, and the same pattern is already in tree for two other Wave 4 slices (`wave-4-diagnostic-baseline-catalog-pr`, `wave-4-hir-recovery-baseline-pr`). Swap to the actual PR URL after filing — consistent with how prior slices (e.g., `pull/2604`) were back-filled.
- `plans/reviews/active/ad-hoc-world-class-verification-wave-4-package-release-preflight-baselines-review-pass-1.md` is a 0-byte placeholder. Fill it in with the review notes before commit; no schema check fails on it being empty.
- The two `target/` cache dirs aren't a commit risk but will quietly regrow on every local run. Optional: `cargo clean` in those fixture roots before opening the PR for a tidier diff in `git status`.

### Recommendation

**No additional review round required.** Proceed to `scripts/run_all_tests.sh --profile create-pr`, open the PR, then run the merge gate.
