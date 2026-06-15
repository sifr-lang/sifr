Review complete. Findings below — no blocking issues.

## Findings

### Blocking
None.

### Non-blocking / follow-up

**N1 — Placeholder `bless_reference` (known follow-up).** All three new entries in `verification/areas/diagnostics/data/baseline_metadata.json` use `"bless_reference": "wave-4-package-metadata-layout-baselines-pr"` (lines 2421, 2436, 2451 in the post-diff file). Existing wave-4 slices in this same file land an `https://github.com/sifr-lang/sifr/pull/<n>` once the PR is opened (see e.g. `bless_reference` for slice-15 fixtures `package_projection_*` at the URL `…/pull/2598`). The `validate_baseline_metadata` contract in `verification/areas/diagnostics/checks/code_baseline_coverage.py:309-310` only checks `bless_reference` is non-empty, so this passes gates as-is, but flag it for the same post-PR edit the prior two slices used.

**N2 — `(os error 2)` is encoded in the missing-manifest stderr baseline (portability risk).** `verification/areas/diagnostics/fixtures/diagnostics/package_missing_sifr_manifest/src/baselines/package-check-compact.stderr.txt:2` ends with `… No such file or directory (os error 2)`. The compiler emits this verbatim because `crates/sifr_package/src/manifest/sifr/load.rs:21-27` forwards `error.to_string()` from `DiskSourceProvider::read_file` into `PackageDiagnostic::missing_sifr_manifest`. The Linux/macOS CI matrix is stable on this text (both share ENOENT=2 and the C/en_US locale), and no normalizer in `verification/runner/sifr_verify/area_adapter.py:17-23, 590-602` scrubs `(os error N)` or localized OS strings. This baseline will need either (a) a new normalizer or (b) a compiler-side reword if Windows or non-English locales ever join the verification matrix. Not a blocker today — it's the same behavior the public CLI emits — but worth recording so future locale/Windows expansion does not silently flake.

### Nits

**Nit 1 — `SIFR-PACKAGE-0001` deferral reason still generic.** `verification/areas/diagnostics/data/code_baseline_coverage.json` keeps the boilerplate deferral text on the remaining `SIFR-PACKAGE-0001` row. The tracker line 985 explains the actual reason (malformed Cargo metadata JSON parsing is not a natural `cargo metadata` public CLI emission path) but the JSON keeps the shared "rendered baseline fixture expansion is staged inside Wave 4" string. The contract in `code_baseline_coverage.py:200-203` only checks `owner/reason/issue/expires_in_wave` non-empty, so this passes. A future improvement would be to surface the actual deferral rationale in the JSON so the registry alone explains why 0001 is excluded; optional, does not block this slice.

**Nit 2 — `Cargo.lock` absent (matches sibling fixtures).** None of the three new fixtures ships a `Cargo.lock`. This matches every other `package-check` / `package-check-default` package fixture except `package_duplicate_public_api_symbol` (which needs locked deps because its diagnostic only fires after compilation). The three new diagnostics all fire before any rustc invocation (manifest load / metadata parse / pure-marker AST scan), so omitting the lockfile is correct.

## Per-focus answers

1. **Public CLI path and package-root cwd.** `verification/runner/sifr_verify/area_adapter.py:464-491` routes `package-check` through `find_package_root(entry)` (which requires `Cargo.toml` + `sifr.toml` at the package root — both present in all three fixtures), then invokes `cargo run --manifest-path <repo>/Cargo.toml --locked -q -p sifr -- [--diagnostic-format compact] check <entry-relative>`. The relative path is computed from `entry.relative_to(cwd)` where `cwd` is the package root, so the runner reproduces a user running `sifr check src/main.sifr` from the fixture's package root. Confirmed identical to the manual reproductions called out in the slice scope.

2. **Fixture construction is deterministic and minimal.**
   - `package_missing_sifr_manifest/`: `Cargo.toml` declares `[package.metadata.sifr] manifest = "missing.sifr.toml"`, a path that does not exist. The on-disk `sifr.toml` at the package root is required only to satisfy `find_package_root` (the compiler ignores it because Cargo metadata takes precedence). Stub `src/main.sifr`, empty `src/__init__.sifr`, and a comment-only `src/lib.rs` are enough to reach the manifest-load step and stop there. Single diagnostic, exit 1.
   - `package_misplaced_cargo_sifr_metadata/`: `Cargo.toml` keeps the valid `manifest = "sifr.toml"` pointer and adds an unsupported `[package.metadata.sifr.package]` table with `name = "package_misplaced_cargo_sifr_metadata"`. The `[package]` key is what triggers SIFR-PACKAGE-0003. Same minimal `src/` skeleton. Exit 2 (validation-layer failure, matching the existing exit-code convention for misplaced-metadata families).
   - `package_non_trivial_pure_marker/`: `Cargo.toml` is clean (`manifest = "sifr.toml"`), but `src/lib.rs` contains `pub fn hidden() {}` instead of the marker comment, which is exactly the non-trivial-marker shape SIFR-PACKAGE-0501 flags. Exit 1.
   - All three include a trailing `[workspace]` table in `Cargo.toml` to detach the fixture from any ambient workspace — same isolation pattern as `package_projection_*`.
   - Every entry source file shares the trivial `def main():\n    pass\n` body, so `source_hash` collapses to one value (`cde0429b…`) for the three baseline_metadata entries.

3. **Coverage rows ↔ rendered baselines align.**
   - `verification/areas/diagnostics/data/code_baseline_coverage.json` flips `SIFR-PACKAGE-0002`, `SIFR-PACKAGE-0003`, and `SIFR-PACKAGE-0501` from `deferral` to `baseline_fixture_id` with `renderer_formats: ["compact"]`. I grepped each stderr baseline and confirmed the code substring appears verbatim, so `validate_coverage_baseline_evidence` (`verification/areas/diagnostics/checks/code_baseline_coverage.py:227-243`) accepts each.
   - Coverage census after the change: 170 codes total, 135 with rendered baselines, 35 deferred (`BUILD 5 / INTERNAL 1 / PACKAGE 23 / STDLIB 2 / WORKSPACE 4`). This matches the tracker text exactly.
   - `SIFR-PACKAGE-0001` remains deferred in the coverage file, consistent with the scope note.

4. **Baseline metadata hygiene.**
   - File locations: `<fixture>/src/baselines/package-check-compact.{stdout,stderr,exit-code}.txt` lines up with `baseline_artifact_paths` (`area_adapter.py:561-567`, computes `entry.parent / "baselines"`) since the manifest entry is `…/src/main.sifr`. Same placement as sibling `package_duplicate_public_api_symbol` and `package_explicit_file_outside_source_root`.
   - Trio complete: stdout empty, stderr rendered with `<WORKSPACE>` token, exit-code matches manifest `expect_exit_code` (`1`, `2`, `1`).
   - Source hashes recomputed: all three new entries' `source_hash` (`sha256:cde0429b…`) matches `sha256(src/main.sifr)` for every fixture — verified by hand against `validate_baseline_metadata` at `code_baseline_coverage.py:305`.
   - Normalizers `{workspace-path, tmp-path, crlf, artifact-cache-lines}` ⊆ the `NORMALIZERS` allowlist in `code_baseline_coverage.py`. Owner/renderer/suite fields mirror prior slice-16 entries.

5. **Tracker accuracy.** `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:981-987` correctly describes scope (three new package-root `package-check` baselines, codes, exit codes, intentional 0001 deferral), coverage delta (135 covered / 35 deferred, family breakdown), and the focused validations. The status line at line 3 cleanly extends the prior status phrasing.

6. **Validation gaps before create-pr / merge.** The reported validations cover the diagnostics area (`baselines --bless`, `baselines`, `contracts`), the Python guardrails (`py_compile`, `check_file_size_guardrails.py`, `git diff --check`), and direct CLI reproductions of each diagnostic. Still pending per AGENTS.md:
   - `scripts/run_all_tests.sh --profile create-pr` — required before opening the PR; the broader gates exercise clippy/fmt/e2e snapshots and the cross-area baseline suites.
   - `scripts/run_all_tests.sh` — required as the merge gate; tracker already lists this as pending.
   - Post-PR: replace the three `bless_reference: "wave-4-package-metadata-layout-baselines-pr"` placeholders with the actual PR URL, matching the pattern slice-15 and slice-16 used.

## Conclusion

No blocking findings. Fixture construction is minimal and production-realistic for each diagnostic, the entry/baseline placement and exit codes match the runner's `package-check` contract, coverage/metadata/manifest counts are internally consistent and align with the tracker, and `SIFR-PACKAGE-0001` is correctly left deferred with a recorded scope rationale. Another review round is not required after fixes — proceed to `scripts/run_all_tests.sh --profile create-pr`, open the PR, run the full merge gate, then complete the post-PR `bless_reference` URL replacement and tracker merge.
