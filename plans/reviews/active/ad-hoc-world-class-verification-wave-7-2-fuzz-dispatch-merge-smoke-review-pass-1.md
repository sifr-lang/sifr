I have enough to write a thorough review. Let me synthesize the findings.

## Wave 7.2 review — per-target fuzz dispatch and merge smoke

### Blockers

**B1. `diagnostic_renderer_entrypoint` and `package_project_manifest_entrypoint` execute identical commands; the harness has no per-target dispatch.**

- `verification/areas/fuzz_property/fuzz_smoke_manifest.json:167-175` and `:198-206` both set `reproduction_command` to `["cargo", "run", "-q", "-p", "sifr_driver", "--bin", "diagnostic_contract_harness"]` with no positional args.
- `verification/runner/sifr_verify/hardening/property_and_fuzz.py:484` calls `argv = list(target["reproduction_command"])` and never appends seed files or a target id.
- `crates/sifr_driver/src/bin/diagnostic_contract_harness.rs:118-133` ignores `env::args` and runs a hard-coded sweep of `PARSER_FIXTURES`, `PROJECT_FIXTURES`, `CYCLE_FIXTURES`, and `PACKAGE_FIXTURES` on every invocation.
- Effect: the two targets are not differentiated at execution time. The same set of fixtures is exercised twice, and `seed_files` in the manifest are not consumed by the harness — they exist only as decoration the validator file-existence-checks. This is exactly the "metadata-only claim" the user asked to guard against (focus item 3) and undercuts the per-target dispatch the slice claims (focus items 1 and 2). Fix options: (a) make the harness accept a `--target` flag and split its hard-coded fixture lists by target id, or (b) route the package/project target through `sifr check` of the declared seed and reserve the harness for the renderer target only.

**B2. Nightly and release will now double-execute `fuzz-smoke` and `property`.**

- `verification/runner/sifr_verify/profile_runner.py:144` adds `run_fuzz_property_suites` as an unconditional step driven by `selected_areas`.
- `verification/runner/sifr_verify/profile_runner.py:453-466` (`run_hardening_suites`) still routes `"property"` and `"fuzz-smoke"` from `legacy_facade.hardening_suites` to the same `uv_area_command("--area", "fuzz_property", ...)`.
- `verification/profiles/nightly.json:174-189` and `:261-271` keep both pathways. `verification/profiles/release.json:174-189` and `:261-271` do the same.
- Result: nightly and release will run each fuzz/property suite twice. Merge is fine because `merge.json:279` does not list these in `hardening_suites`. Either remove the new step's entries from the legacy facade in nightly/release, or have the new step run only when those suites are absent from `hardening_suites`.

**B3. Wave 7.2 has no warm/cold merge wall-time evidence.**

- The phase plan itself (line 1595) lists Wave 7 as a gate-expanding wave that "must record warm/cold merge wall time before and after the change."
- The Wave 7.2 implementation notes (lines 1311-1314) only show `scripts/run_all_tests.sh --profile merge --emit-plan` — a plan emission, not an execution. The codegen target uses `command: "run"` (`fuzz_smoke_manifest.json:122`) with `timeout_seconds: 60`, which on a cold cache pays full sifr-build + generated-rust build twice. The diagnostic-contract harness adds another build. None of this has been measured against the 15 min warm / 25 min cold merge budget.

### Major

**M1. `run_reproduction_command_target` invokes Cargo without `--locked`.**

- `verification/runner/sifr_verify/hardening/property_and_fuzz.py:484-498` shells out the manifest's verbatim `reproduction_command` via `subprocess.run`. The manifest's command (`fuzz_smoke_manifest.json:168-175`, `:199-206`) is `["cargo", "run", "-q", "-p", "sifr_driver", "--bin", "diagnostic_contract_harness"]` — no `--locked`. CLAUDE.md/AGENTS.md mandate `--locked` for Cargo commands in create-pr/merge. Add `--locked` either to the manifest entry or unconditionally in the runner before exec.

**M2. Validation accepts targets whose `seed_files` are not actually consumed.**

- `validate_fuzz_target_contract` checks the seed files exist as files (`property_and_fuzz.py:648-654`), but for `coverage_mode == "diagnostic-contract-smoke"` the runtime path (`run_reproduction_command_target`) never opens them. This is the same defect as B1 from the validator side: the contract should either drop `seed_files` from diagnostic-contract-smoke targets or assert (e.g., by reading the harness binary's expectations) that the listed seeds are exercised. Otherwise the manifest can claim coverage of any seed by listing it.

**M3. No determinism check on fuzz-smoke targets.**

- The `property` suite repeats runs and compares (`property_and_fuzz.py:123-188`). The `fuzz-smoke` source-mutation targets do not. `verification/policy/fuzz_property.md:15-18` promises "reproducible local results (same inputs → same outcome)" yet the runner only relies on a deterministic input stream. Add at least one determinism re-run per source target, or weaken the policy text.

### Minor

**m1. Tmp files for failing snippets are not cleaned on subsequent runs.**

- `property_and_fuzz.py:411-445` writes `target/verification/tmp/<id>_<i>_<hash>.sifr`; on success it unlinks, on failure it keeps the file but never sweeps stale failures from prior runs. Inside `target/`, but worth a one-line directory rotation before each run.

**m2. `expect_exit_code` of 0 plus a panic-signal scan is fine, but the `124` timeout exit code from `run_reproduction_command_target:499-503` is reported as `unexpected-exit` without a distinct `timeout` mismatch label.** A future failure reads identically to a real non-zero exit.

**m3. Direct area-runner invocations (without `profile_runner.py`) won't get `CARGO_NET_OFFLINE`.**

- `profile_runner.py:128-130` sets the env globally only when going through `ProfileRunner.__init__`. Running `uv run … areas run --area fuzz_property --suite fuzz-smoke` directly leaves cargo online by default. Acceptable, but worth either documenting or moving the env policy into the area runner itself.

**m4. Policy/profile divergence.** `verification/policy/fuzz_property.md:12-14` lists the canonical runner as `--suite property --suite fuzz-smoke`, while merge.json selects only `fuzz-smoke`. If `property` is intentionally non-merge (sustained), say so explicitly in the policy doc next to the canonical-runner line.

**m5. Codegen smoke uses `command: "run"`.**

- `fuzz_smoke_manifest.json:122` runs each of 2 seeds through full rustc compile+execute (timeout 60s each). A `check`-only smoke would still exercise codegen lowering for free and move the cargo-build cost to a sustained lane. Worth a justification in the policy doc or a downshift.

**m6. Result accounting nit.** `run_fuzz_smoke_suite:340-342` increments `failed_cases` per target with any variant failure. `failed_cases` is summed across cases and exposed at the suite level — fine — but `target_metadata_failure` paths set `total_failures: 1` even when `total_variants: 1`, which makes "metadata invalid" indistinguishable from "ran, one variant failed" in summaries. Consider a distinct `metadata_failures` bucket.

### Tests/validation gaps

- No unit test that asserts each of the 5 target ids actually produces at least one `pass` variant when the manifest is exercised. The integration is implied by the user-reported `variants=23`, but a focused self-test in `verification/runner/sifr_verify/self_tests*` would prevent silent regressions (e.g., a future refactor that drops a coverage_mode branch).
- No assertion that the diagnostic-contract harness covers the seed fixtures named per target — pairs the harness hard-coded list to the manifest's declared seeds (closes B1's root cause as an invariant).
- The phase-mandated warm/cold merge wall-time measurement (see B3) is itself a validation gap.

### Verdict

Another Opus review round is required after fixes. Two of the blockers (B1 metadata-only diagnostic dispatch, B2 nightly/release double-execution) change the substance of what runs and must be re-reviewed once the harness/profiles are realigned.

A full `scripts/run_all_tests.sh --profile merge` run (not `--emit-plan`) is required for this slice before PR submission, both because Wave 7 is a gate-expanding wave with a phase rule that mandates warm/cold merge wall-time measurement, and because the codegen target's `sifr run` calls and the harness reproduction commands have never been executed end-to-end inside the merge lane. `create-pr` validation alone is not sufficient.
