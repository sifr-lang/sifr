Review complete. No blocking findings. The slice flips `SIFR-PACKAGE-0605` and `SIFR-PACKAGE-0606` from Wave 4 deferral to rendered compact baseline coverage, with two minimal fixture packages and two narrowly-scoped adapter aliases. Internal consistency (catalog ↔ coverage ↔ manifest ↔ metadata ↔ baseline trio) holds.

## Findings

### Blocking

None.

### Non-blocking / follow-up

**N1 — Per-fixture adapter command names (design pattern).** `verification/runner/sifr_verify/area_adapter.py:34-37,470-501` adds two fixture-specific aliases (`package-run-bin-bad-name` → `run --bin bad!name`, `package-run-target-admin` → `run admin`). This mirrors the per-fixture pattern already used by `package-run-script` (`run --script dev`) and `package-check-default` (`check`), so it does not break a prior convention. It does mean `BASELINE_COMMANDS` and the elif chain in `run_sifr_variant` grow one entry per Wave 4 run-target fixture instead of using a parameterized `package-run-target <name>` route. The hardcoded argv keeps the baseline evidence stable (no chance of selector drift) and there is no shell interpolation (subprocess.run with list args, `bad!name` is treated literally), so this is safe; flagging only as a pattern to keep in mind as more `SIFR-PACKAGE-06xx` slices land — at some point a parameterized adapter route may read cleaner than continuing to grow the elif chain.

**N2 — `bless_reference` placeholder (matches prior slices).** `verification/areas/diagnostics/data/baseline_metadata.json:2466-2495` lands `"bless_reference": "wave-4-package-run-target-baselines-pr"` on both new entries. `validate_baseline_metadata` (`verification/areas/diagnostics/checks/code_baseline_coverage.py:309-310`) only requires non-empty, so this passes gates. Prior slices (e.g. slice-15 `package_projection_*` lands `https://github.com/sifr-lang/sifr/pull/2598`) replaced the placeholder with the PR URL post-PR — apply the same edit here once the PR is open. Not a blocker.

### Nits

**Nit 1 — `!` in fixture file path (portability/UX).** `verification/areas/diagnostics/fixtures/diagnostics/package_run_invalid_app_target/src/bin/bad!name.sifr` is the first diagnostics fixture with a shell-meta character in its name. The harness invokes via `subprocess.run(argv, ...)` so this is safe in CI, but a developer copying the command into an interactive `bash`/`zsh` would need to single-quote it (`!` triggers history expansion). The `!` is essential for the SIFR-PACKAGE-0606 emission per `crates/sifr_package/src/diag/package.rs:296-308` (alphanumeric/`_`/`-`/`/` are the only valid characters), so the fixture is correct — just noting the new ergonomic footnote. No action needed.

**Nit 2 — Tracker phrasing.** The status line at `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:3` now reads "package run-target coverage implemented pending review/gates", which is accurate but slightly different shape from prior slices that read "and …-coverage merged" once merged. Once the PR merges, fold this phrase back into the "merged" list as prior slices have done. Cosmetic.

## Per-focus answers

1. **Fixture construction (stable and minimal).**
   - `package_run_target_ambiguous/`: `sifr.toml` declares `[scripts] admin = { command = "check", args = [] }` and `src/bin/admin.sifr` ships a `def main(): pass`. Together they create two run-target candidates named `admin` (one app target via `src/bin/`, one script entry). `sifr run admin` from the package root is forced through `crates/sifr_package/src/diag/package.rs:258-274`, emitting `SIFR-PACKAGE-0605` with `candidates: bin:admin, script:admin`. The candidate list is deterministic — sorted alphabetically (`bin:admin` < `script:admin`), so the rendered stderr is stable.
   - `package_run_invalid_app_target/`: `sifr.toml` has no `[scripts]` table and `src/bin/bad!name.sifr` is the sole app-target candidate. `sifr run --bin bad!name` exercises the validator at `crates/sifr_package/src/diag/package.rs:296-309`, which rejects any character outside `[A-Za-z0-9_\-/]` and emits `SIFR-PACKAGE-0606`. Single diagnostic, exit 1.
   - Both fixtures add the `[workspace]` table in `Cargo.toml` to detach from any ambient workspace — same isolation pattern as `package_projection_*` / `package_script_recursion`.
   - `src/lib.rs` is the comment-only "Pure Sifr package marker" used by sibling fixtures; both `sifr.toml` files declare `edition = "2026"` and `sifr-version = ">=0.3,<0.4"`, matching the rest of the diagnostics package corpus.

2. **Adapter commands are scoped and safe.**
   - `verification/runner/sifr_verify/area_adapter.py:466-505` routes both new command names through the same `cargo run --manifest-path <repo>/Cargo.toml --locked -q -p sifr -- …` invocation as the existing `package-*` commands, with `cwd` set by `find_package_root(entry)` (`area_adapter.py:618-624`). Both new fixtures have `Cargo.toml` + `sifr.toml` at the package root, so `find_package_root` resolves the CWD deterministically. The argv lists are hardcoded — no string interpolation of fixture data — so `bad!name` cannot expand. Each command is gated by exact match on `command_name`, so they only fire for the two new manifest cases.
   - `BASELINE_COMMANDS` is updated in lockstep at `area_adapter.py:34-37`, so both names participate in baseline routing (rather than being treated as `area-check`).

3. **Manifest / coverage / metadata / source-hash consistency.**
   - `verification/areas/diagnostics/manifest.json:490-507` adds both cases under the `baselines` suite with `expect_exit_code: 1` and `diagnostic_formats: ["compact"]`. Catalog renderer_support for both codes already includes `human/json/compact` (`verification/areas/diagnostics/data/code_catalog.json:1876-1909`), so the `renderer_formats` subset check in `validate_coverage` (`code_baseline_coverage.py:215-216`) accepts.
   - `verification/areas/diagnostics/data/code_baseline_coverage.json:1262-1280` flips both rows from `deferral` to `baseline_fixture_id` with `renderer_formats: ["compact"]`. I grepped each `…-compact.stderr.txt` and confirmed the code substring is verbatim present, so `validate_coverage_baseline_evidence` (`code_baseline_coverage.py:227-243`) passes.
   - `verification/areas/diagnostics/data/baseline_metadata.json:2465-2495` adds two metadata rows. Their `source_hash` values match `sha256(sifr.toml)` for each fixture — verified independently via `openssl dgst -sha256`: `e2e5edbf…ef` for `package_run_target_ambiguous/sifr.toml` and `5c017ef7…67` for `package_run_invalid_app_target/sifr.toml`. The `case["entry"]` (per manifest) is the `sifr.toml` path, which is what `validate_baseline_metadata` (`code_baseline_coverage.py:297-306`) hashes for these rows.
   - Normalizers `{workspace-path, tmp-path, crlf, artifact-cache-lines}` ⊆ the `NORMALIZERS` allowlist; owner (`compiler/frontend`) matches the catalog entries; renderer (`compact`), suite (`baselines`), and `bless_reason` are non-empty.
   - Baseline trio is complete at `<fixture>/baselines/`: `.stdout.txt` is empty, `.stderr.txt` is exactly the rendered single-diagnostic compact frame, `.exit-code.txt` is `1\n`. Placement matches sibling `package_script_recursion/baselines/` (entry is `sifr.toml` → `entry.parent / "baselines"`).
   - Coverage census: 170 codes, 137 covered, 33 deferred (`BUILD 5 / INTERNAL 1 / PACKAGE 21 / STDLIB 2 / WORKSPACE 4`) — confirmed by tallying `code_baseline_coverage.json` directly. Matches the tracker text at line 995.

4. **Tracker accuracy and validation gaps.**
   - `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:991-996` correctly enumerates the two new public CLI forms, the two codes, the coverage delta (135 → 137 covered, 23 → 21 PACKAGE deferred), and the focused validations actually run. The numerical delta is consistent with slice-17's "135 covered, 35 deferred" baseline.
   - Reported validations cover the diagnostics area (`baselines --bless`, `baselines`, `contracts`), Python guardrails (`py_compile`, `check_file_size_guardrails.py`, `git diff --check`), and direct compact CLI reproductions from each fixture package root.
   - Pending per AGENTS.md and prior wave-4 cadence:
     - `scripts/run_all_tests.sh --profile create-pr` — required before opening the PR; exercises clippy/fmt/e2e snapshots and the cross-area baseline suites.
     - `scripts/run_all_tests.sh` — required as the merge gate.
     - Post-PR: replace both `bless_reference: "wave-4-package-run-target-baselines-pr"` placeholders with the actual PR URL, matching the slice-15/16/17 pattern (per N2).
   - The two fixture directories are present on disk but currently untracked (`git status` shows them under `??`). They must be `git add`-ed before the create-pr profile runs against the staged tree; otherwise the diagnostics gate will pass locally on the working tree but the PR will not contain the fixtures.

## Conclusion

No blocking findings. Fixture construction is minimal and stable for both diagnostics (deterministic candidate ordering for 0605, single invalid-name token for 0606), the adapter aliases are scoped and shell-safe, every manifest/coverage/metadata/baseline pairing is internally consistent, source hashes are correct, and the tracker counts match the JSON. Another review round is not required after fixes — proceed to staging the two new fixture directories, then `scripts/run_all_tests.sh --profile create-pr`, open the PR, run the full merge gate, and complete the post-PR `bless_reference` URL replacement plus tracker "merged" phrasing.
