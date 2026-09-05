I've inspected the changed files, the harness binary, the manifest, runner, profile runner, profile JSON, policy doc, sustained-lane doc, and the existing harness consumers in `developer_tooling`. Findings below.

## Wave 7.2 review pass 2 — per-target fuzz dispatch and merge smoke

### Pass 1 blocker disposition

**B1 (per-target dispatch is metadata-only) — SATISFIED.**
- `crates/sifr_driver/src/bin/diagnostic_rendering_harness.rs:119-187` now parses `--target <id> --seed <path>`; the per-target arm dispatches to `check_parser_seed_runtime_contract` (for `diagnostic_renderer_entrypoint`) or `check_project_seed_runtime_contract` (for `package_project_manifest_entrypoint`).
- `verification/areas/fuzz_property/fuzz_smoke_manifest.json:167-181` and `:204-218` pass the differentiating `--target`/`--seed` argv.
- `verification/runner/sifr_verify/hardening/property_and_fuzz.py:766-774` enforces at validation time that for `diagnostic-contract-smoke` targets the `reproduction_command` actually contains `--target`, the target id, and every declared `seed_files` path. This converts the prior decorative `seed_files` into an executed contract.

**B2 (nightly/release double execution) — SATISFIED.**
- `verification/runner/sifr_verify/profile_runner.py:321-337` builds `legacy_fuzz_suites` from `self.hardening_suites` and filters those suites out of the new step.
- For nightly/release both `property` and `fuzz-smoke` remain in `legacy_facade.hardening_suites` (nightly.json:261-271, release.json:261-271), so the new step skips them and only `run_hardening_suites` runs them.
- For merge, `merge.json:279-285` does not list these suites, so the new step runs `fuzz-smoke` exactly once; `run_hardening_suites`' `fuzz_property_args` remains empty.

**B3 (warm/cold merge wall-time) — still pending** by design; the user explicitly carries the full `scripts/run_all_tests.sh --profile merge` run as the remaining required validation.

### Pass 1 secondary disposition

- M1 `--locked` — fixed (`fuzz_smoke_manifest.json:170, :207`).
- M2 seed files actually consumed — fixed; the validator above closes the gap.
- M3 determinism — fixed for source-mutation targets by `property_and_fuzz.py:445-523` (`determinism-rerun` variant). See observation O3 below for the gap that remains for `diagnostic-contract-smoke` and for failure-path determinism.
- m1 stale tmp sweep — fixed at `property_and_fuzz.py:407-409`.
- m2 timeout label — fixed at `:583-584`. See O2 below for the dual-label nuance.
- m3 `CARGO_NET_OFFLINE` for direct area runs — fixed at `verification/areas/fuzz_property/runner.py:42`.
- m4 policy/profile divergence — fixed at `verification/policy/fuzz_property.md:13-14`.
- m5 codegen `run` — acceptable; explicit in policy.
- m6 metadata accounting — not addressed; remains a minor nit.

### New findings (pass 2)

All low severity; none are blockers.

**O1 — Coverage narrows for the diagnostic-contract harness path (Minor).** Per-target dispatch invokes the harness with exactly one seed per target, so the merge fuzz-smoke route exercises only one parser fixture (`parser_bad_indent`) and one project fixture (`workspace_missing_import_canonical`). The harness `PACKAGE_FIXTURES` and `PACKAGE_FATAL_FIXTURES` (`diagnostic_rendering_harness.rs:77-117`) are no longer reached through the merge-gate path. Coverage is preserved at the workspace level because `verification/areas/developer_tooling/check_diagnostic_source_canonicalization_rules.py:213-218` still invokes the harness binary with no args (which hits `HarnessArgs::All`) and that suite is in merge. Worth a one-line note in `sustained_lane.md` saying merge fuzz-smoke covers one seed per target and that the broader fixture sweep is owned by the developer-tooling contract suite — so a future cleanup doesn't break the implicit dependency.

**O2 — Timeout exits get both `timeout` and `unexpected-exit` labels (Minor).** `property_and_fuzz.py:580-584` appends `unexpected-exit` whenever `exit_code != expect_exit_code` and then also appends `timeout` when `exit_code == 124`. Functionally fine — labels are additive, not exclusive — but two mismatches will be reported for every timeout. Either guard the `unexpected-exit` append (`elif exit_code == 124: …` style) or document that the two co-occur intentionally.

**O3 — Determinism rerun coverage is partial (Minor).**
- `run_reproduction_command_target` (`:544-604`) runs each diagnostic-contract-smoke target once; there is no rerun and no exit/output drift check, despite policy line 19 ("same inputs → same outcome"). For two targets that drive a fully deterministic Rust binary this is low risk, but it leaves a written policy claim without a runtime check.
- For source-mutation targets, the `determinism-rerun` variant is keyed on the first passing snippet (`:445-448`). If the first snippet fails (`run_mismatches` is non-empty), no determinism comparison is performed. Acceptable, but two natural reads of the policy disagree on whether failure-path determinism should also be asserted.

**O4 — `run_reproduction_command_target` inherits env implicitly (Minor).** It calls `subprocess.run(argv, cwd=repo_root, …)` with no `env=`. In merge it inherits `CARGO_NET_OFFLINE=true` from `ProfileRunner.__init__` (`profile_runner.py:128-130`); when invoked directly via `areas run`, it inherits from `runner.py:42`. That chain is correct today, but the runner depends on a caller-managed env var rather than an explicit `env={**os.environ, "CARGO_NET_OFFLINE": "true"}`. A passing comment or one-line env merge inside the function would prevent regressions when future profiles forget to set the env.

**O5 — Cold-build timeout headroom for the harness (Minor).** Both `diagnostic-contract-smoke` targets set `timeout_seconds: 120` and the merge-gate command is `cargo run --locked … --bin diagnostic_rendering_harness`. On a fully cold cache the first such invocation pays a `sifr_driver` build. In merge, `sifr_driver` is already built by the crate-tests step (`merge.json:59`) and the dev-tooling contract suite, so the harness will land warm. Worth confirming during the B3 wall-time run; if cold timing on the reference host approaches 120 s, lift the timeout or pre-build.

### Verdict

- **B1: satisfied.** Per-target dispatch is now wired end-to-end (harness, manifest, validator).
- **B2: satisfied.** Nightly/release no longer double-execute `property` / `fuzz-smoke`.
- **Another agent review round after fixes: not required.** B1 and B2 are closed and no new findings reach blocker severity. O1–O5 are minor and can ride either in this PR or in a follow-up note.
- **Is the remaining full merge run sufficient before PR?** Yes, conditional on it (a) passing without regressing the 132/132 e2e/fuzz-property variants, (b) recording warm/cold wall time inside the 15 min warm / 25 min cold budget per Wave 7 phase rule, and (c) confirming the merge plan actually executes `fuzz_property:fuzz-smoke` (not just emits it). No further code changes are needed beforehand; the optional follow-ups above can be folded in as policy/comment-only deltas if you choose, but they are not gating.
