## Findings

### Round 3 follow-ups verified

1. **Distribution per-script timing.** `scripts/run_distribution_validation.sh:8-34` now wraps each `verification/distribution/*.sh` in start/stop monotonic measurement and emits `[sifr-case-timing] bucket=distribution case=<name> status=pass|fail` before the loop-terminating `exit`. The round-1 #15 / round-3 follow-up about the bare `exit "${status}"` is properly mitigated — the failure record is logged before the loop bails.

2. **Hardening profile aliases.** `scripts/run_verification_hardening/core.py:40, 88-93` widens `--profile` choices to `create-pr|merge|quick|pr|nightly|release|full|stress` and canonicalizes `pr|full → merge`, `quick → create-pr`, `stress → release`, matching `verification/validation_lanes/manifest.json:3-8`. `should_run_suite` (line 144-156) returns `False` for `create-pr` and runs the `diagnostics|project|fixedbugs|crashes|oss-curated` set for `merge`. `schema_version: 1` in the hardening JSON is unchanged; round-3's pre-existing minor about the canonicalized name leaking into the artifact is still cosmetic.

3. **Validation-contract row timing.** `crates/sifr/tests/validation_contract_support/runner.rs:62-84` wraps each row body in a closure, removes the temp dir unconditionally at line 73, then emits `[sifr-case-timing] bucket=validation_contract` before `result?`. The round-3 cosmetic about losing the timing line on `temp_root(...)?` failure (line 65) is still present but the failure is genuinely catastrophic (cannot create `${TMPDIR}/sifr-validation-contracts-…`), so no real evidence is lost.

### New surface — `diagnostic_rendering_harness` binary

4. **Binary builds and exists.** `crates/sifr_driver/src/bin/diagnostic_rendering_harness.rs` (375 lines, under 900 cap) is invoked via `verification/tooling/check_diagnostic_source_canonicalization_rules.py:186-218` (`cargo build -q -p sifr_driver --bin diagnostic_rendering_harness` → run). `target/debug/diagnostic_rendering_harness` exists at ~30 MB, confirming it compiles. All imports (`check_single_file`, `check_project`, `check_package_project`, `PackageEntrypoint`, `render_package_diagnostic`) are publicly re-exported from `crates/sifr_driver/src/lib.rs:19-27`.

5. **Harness decomposition is clean.** One function per fixture family (`check_parser_runtime_contract:109`, `check_project_runtime_contract:122`, `check_cycle_runtime_contract:141`, `check_package_runtime_contract:160`). `process::exit(1)` lives only in `main` (lines 83-89); all internal paths return `Result<(), String>`. `assert_contract` (line 259) takes 7 parameters which is at the clippy pedantic threshold, but the user's `cargo clippy --workspace -- -D warnings` validation passed.

6. **Package path still uses one `cargo metadata`** (`crates/sifr_driver/src/bin/diagnostic_rendering_harness.rs:186-204`). That's necessary to derive the package graph and is one external invocation per package fixture (~3 fixtures) rather than the ~42 `cargo run` invocations the Python harness was making. The 80.48s → 3.18s measurement in `issues/…:164` is consistent.

### Other touched files

7. **`crates/sifr/tests/e2e_support/fixture_compilation.rs:570-599`.** `max_group_fixtures` resolves `SIFR_E2E_MAX_GROUP_FIXTURES`, parses to `usize`, filters `> 0`, defaults to `usize::MAX`. Cases are sorted before chunking and each chunk goes through `build_group_sources` with proper planning-failure attribution per fixture. Group-skew acceptance for milestone_gate_speed_5 is met (largest group `19→8` quick lane, `43→12` merge lane).

8. **`verification/generated_code_quality/generated_code_quality.py:320-405`.** `shared_artifact_root()` resolves `SIFR_GCQ_SHARED_ROOT` (set in `scripts/run_all_tests.sh:308-310` and wiped each run). `entry_cache_key` hashes id + source_path + source bytes. `materialize_entry` checks for `Cargo.toml` and emits `[sifr-artifact-cache] cache_hit=true|false` lines parsed by `scripts/validation_lane_report.py:30-32`. `CARGO_TARGET_DIR=<shared>/cargo-target` is set only for `cargo check` / `cargo clippy` (line 343-350), correctly excluding `cargo run -p sifr -- build` (which uses the workspace target) and `cargo fmt` (no compilation). Per-mode reuse within one lane run is what milestone_gate_speed_4 asks for.

9. **`verification/tooling/lsp_protocol.py:43-48, 111-175`.** `events` ring buffer kept to last 20; `_diagnostic_context` returns `lifecycle{pid,returncode,args}` + last-10 events + `stderr_tail[:-4000]`. Wired into `close()` (line 96, 99), `_read_message()` (line 127), satisfying milestone_gate_speed_3's "emit stderr, process lifecycle, and last protocol event evidence on failure".

10. **`verification/performance/run_benchmarks.py:253-272`.** Per-case `[sifr-case-timing] bucket=performance case=<id>` emission added via `try/except/finally`. `status=fail` only set in the `except Exception` arm before re-raise; status defaults to `pass` if no exception fires.

11. **`scripts/run_verification_hardening/main_flow.py:22-42, 167-170`.** `timing_token` sanitizes case/variant names to match `validation_lane_report.py:40-43`'s `[A-Za-z0-9_.:/+-]+` pattern. `emit_case_timings` runs after `execute_suite_once` for each selected suite.

### Issue / docs

12. **`issues/ad-hoc-pr-gate-speed-and-validation-lane-rebalancing.md:3,103-140,159-208`.** Status flipped to `implemented`, all 6 milestone checkboxes `[x]`, Implementation Measurements table covers both warm and cold create-PR runs (74.82s / 206.74s), merge lane (595.66s under 15-min target), and the per-bucket breakdowns for both lanes. The merge-lane group-skew advisory is documented at lines 205-208 with `cache_hits=0/22`, explaining the cold-cache caveat.

13. **`internal_docs/validation_lane_policy.md`.** Aligned with manifest: `create-pr` excludes "broad project-mode matrices, full generated clippy/corpus, full performance budgets, and the slower `sifr` integration/e2e-support crate tests" (line 25), matching `verification/validation_lanes/manifest.json` and `scripts/run_all_tests.sh:375-388`'s `crate_tests=smoke` split.

### Phase-closure obligation (not a code defect)

14. **`crates/sifr_driver/src/bin/` is untracked in git.** `git status` lists `crates/sifr_driver/src/bin/` as untracked, so the new harness file (and its containing directory) is **not yet committed**. The Python validation harness at `verification/tooling/check_diagnostic_source_canonicalization_rules.py:188` issues `cargo build -p sifr_driver --bin diagnostic_rendering_harness`, which would fail on any fresh clone or CI environment until this directory is staged into the closure PR. Round-3 SATISFIED with this same state, so the prior reviewer evidently expected the user to `git add` before opening the closure PR — flagging it here so the closure commit doesn't accidentally omit it.

## Verdict

**SATISFIED.** The implementation is sound and matches every milestone's stated acceptance criterion: harness collapses the ~42-invocation Python check to a single binary (3.18s warm, well under the 10s target), generated-code smoke at 18.11s (under 30s target), create-PR lane at 74.82s warm (under 120s), merge lane at 595.66s (under 15m), group skew capped via `max_group_fixtures`, LSP error reporting includes lifecycle + events + stderr tail, distribution and hardening buckets emit per-case timing, lane manifest / resolver / shell / hardening alias map / policy doc are all consistent. Round-3 follow-ups are addressed. The only obligation outside the code itself is staging `crates/sifr_driver/src/bin/` in the closure PR — without that, the source-canonicalization contract breaks on a fresh tree.
