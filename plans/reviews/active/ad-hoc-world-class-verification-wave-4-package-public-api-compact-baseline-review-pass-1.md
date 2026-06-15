## Findings — Wave 4 package duplicate public API compact baseline slice

**Blockers:** none.
**Major:** none.

### Minor

1. **Fixture's internal package name was not renamed after copy-paste.** `verification/areas/diagnostics/fixtures/diagnostics/package_duplicate_public_api_symbol/sifr.toml:2` is `name = "package_fatal_source_map_no_import_ambiguity"` and the sibling `Cargo.toml:2` is `name = "package-fatal-source-map-no-import-ambiguity"` — both inherited verbatim from the twin fixture at `verification/areas/package_management/fixtures/package/package_fatal_source_map_no_import_ambiguity/`. The diagnostic (`SIFR-PACKAGE-0713`) is emitted from `src/__init__.sifr` re-exporting `value` from both `.parse` and `.other`, so the package name doesn't influence the baseline — this is cosmetic, but a future reader grep-ing for "package_duplicate_public_api_symbol" inside the fixture will be confused. Recommend renaming the internal `name` fields to match the directory.

2. **`bless_reference` is a placeholder, not a PR URL.** `baseline_metadata.json` for the new entry uses `"bless_reference": "wave-4-package-public-api-compact-baseline-pr"`. This matches the precedent set by the eleventh slice (`SIFR-BUILD-0901`) where the reviewer marked the same swap-in as an optional follow-up after the PR is opened. Keep the same follow-up here.

### Info (process, not code)

3. **Broad-gate evidence is intentionally missing.** Focused validation listed in the plan note covers direct CLI emission (1× `SIFR-PACKAGE-0713`, exit 1), a bless+verify of the `baselines` and `contracts` suites (123 cases / 151 renderer variants), `py_compile`, file-size guardrail, and `git diff --check`. This is enough to clear *code review*, but `scripts/run_all_tests.sh --profile create-pr` and the full merge gate still need to run before PR/merge — the slice text already lists them as pending, matching the eleventh-slice workflow. No action for review.

### Verified correct

- **`package-check` command path** (`verification/runner/sifr_verify/area_adapter.py:461-483, 596-602`):
  - `BASELINE_COMMANDS` updated; all generic baseline plumbing (`baseline_case_metadata`, `validate_unique_baseline_artifact_paths`, `compare_or_bless`, timing) treats `package-check` identically to other commands — no special-cased gaps.
  - `find_package_root` walks `entry.parent` upward, requires both `Cargo.toml` *and* `sifr.toml`, and bails at `REPO_ROOT.parent` with a clear error. For the new fixture it resolves on the second iteration (`.../package_duplicate_public_api_symbol/`).
  - `cargo run --manifest-path REPO_ROOT/Cargo.toml --locked -q -p sifr -- … check src/main.sifr` correctly invokes the workspace `sifr` binary while running from the package root, so the user-facing `sifr check src/main.sifr` invocation is faithfully reproduced and the workspace `target/` cache is reused. The fixture's own `Cargo.toml`/`Cargo.lock` are inert under `--manifest-path` and aren't workspace members anyway.

- **Nested `src/baselines/` change** (`verification/areas/diagnostics/checks/code_baseline_coverage.py:101-121`):
  - `glob("**/baselines/*.txt")` is scoped to `fixture_root` so it cannot leak into other areas.
  - `fixture_id = path.relative_to(fixture_root).parts[0]` is a strict generalization of the prior `path.parent.parent.name` — for the existing flat `<fixture>/baselines/*.txt` layout it returns the same value, so no existing baseline ownership flips. For the new nested `<fixture>/src/baselines/*.txt` layout it still attributes to the top-level fixture id.
  - `expected_baseline_files` derives the baseline dir from `entry.parent / "baselines"`, which for the new case is `<fixture>/src/baselines/` — matches the on-disk layout, so the trio existence check and `actual_files - allowed_files` orphan check both pass.
  - `validate_baseline_metadata` uses `case["entry"]` for `source_path`, so the SHA is taken against `<fixture>/src/main.sifr` — matches the metadata.

- **Scope discipline:** baseline stderr emits exactly one diagnostic (`E SIFR-PACKAGE-0713 … public API symbol 'value' is exported more than once …`), so `validate_coverage_baseline_evidence` (which substring-matches the code) passes; `code_baseline_coverage.json` flips only the `SIFR-PACKAGE-0713` row and the other 33 PACKAGE codes remain deferred — no over-claim.

- **Counts and hash:**
  - Total active codes = 170, covered = 125, deferred = 45 (BUILD 5, INTERNAL 1, PACKAGE 33, STDLIB 2, WORKSPACE 4) — matches the plan note exactly.
  - `sha256(src/main.sifr) = 16a15c2908801c728acef50fb9c07adae953f1e3dccaa9fe7b95a3ce3152c929` — matches `baseline_metadata.json`.
  - Catalog supports `compact`; coverage entry uses `compact`; manifest case lists `compact` and `expect_exit_code: 1` matching the baseline `1\n`.

### Verdict

No blockers and no must-fix issues. The two minor items follow precedent from the eleventh slice and were already left as optional follow-ups there. **Another review round is not required** — the only remaining gates are mechanical (run `--profile create-pr`, run the merge gate, optionally swap `bless_reference` for the real PR URL).
