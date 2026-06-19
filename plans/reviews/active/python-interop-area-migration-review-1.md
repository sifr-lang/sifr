## Review: `verification_py_area_1`

Local working tree on `python-interop-area-migration` vs `main` — read-only review, no edits made.

### BLOCKERS
None. The milestone is internally consistent, locally validated, and answers the structural concern it set out to address.

### Verdict on the specific questions

1. **First-class area?** Yes. `verification/areas/python_interop/manifest.json:1` declares `schema_version: 2`, owner `runtime/python-interop` (added to `verification/owners.json:74`), `parallel_safe: false`, `resource_classes: ["default-local","platform-specific"]`, `network_mode: "offline"`, and 10 adapter suites. Each case's `entry` is the repo-relative path `verification/areas/python_interop/runner/run.py`, which `verification/runner/sifr_verify/schemas.py:143` accepts. `verification/runner/sifr_verify/areas.py:83` loads `runner.py` as a module and calls `main()`; that file (`verification/areas/python_interop/runner.py:96`) iterates `manifest["suites"]`, dispatches to `runner/run.py` via subprocess for each case, and accepts `--suite`, `--bless` (rejected), `--result-json`, `--hardening-summary` — matching the framework contract used by `verification/areas/runtime_platform/runner.py:23`.

2. **Profiles wire the right non-container suites and execute them?** Yes. `verification/runner/sifr_verify/profile_runner.py:139` registers the new `python_interop` step, and `profile_runner.py:313` queries `selected_areas` and invokes `uv … sifr_verify areas run --area python_interop --suite …`. Suite selections:
   - `verification/profiles/create-pr.json:83-97` — 7 suites: self-test, scaffold, env, tier1, callbacks, dataframes, cloud-boto3.
   - `verification/profiles/merge.json:95-112`, `nightly.json:108-125`, `release.json:108-125` — all 10 suites (adds tier2, tier3, tier4).
   None of the wired suites pull in testcontainers or live network — matches the milestone scope. Validation already showed create-pr `variants=7 failures=0` and standalone area run `variants=10 failures=0`.

3. **Reports under `target/`?** Yes for the profile-driven path. `runner.py:16` defaults `--result-json` to `target/verification/areas/python-interop-results.json`, and every `COMMAND_ARGS` entry (`runner.py:18-76`) passes `--report ../../../target/verification/areas/python_interop/<suite>.latest.json` to `runner/run.py`, which resolves it under `paths.area_root` (`runner/run.py:175` + `runner/env.py:16`) — landing under `target/`. Self-test writes no report.

4. **Repo-relative paths and docs updated?** Yes outside historical artifacts. `docs/python-interop.mdx:318`, `internal_docs/architecture.md:54`, `internal_docs/python_interop_architecture.md:3,63,88`, and `plans/issues/active/ad-hoc-embedded-python-interop.md` are all renamed to `verification/areas/python_interop/…`. `verification/areas/python_interop/README.md:11-16` and `reports/python_interop_exit_evidence.md:43-79` also use the new path. The only remaining legacy-path mentions are under `plans/reviews/active/` and historical body text of `plans/issues/active/ad-hoc-embedded-python-interop.md`, which the prompt explicitly permits.

5. **Concern that Python interop should not live outside `areas/`?** Yes — directly answered. The old `verification/python_interop/` tree is deleted (87 files removed in `git status`), the new tree lives at `verification/areas/python_interop/` alongside the other 16 areas, and discovery in `verification/runner/sifr_verify/areas.py:34` picks it up automatically.

6. **Clear that this is scaffold/matrix/env probes, not full dependency containers?** Yes. `plans/issues/active/python-interop-verification-production.md:11-24` explicitly scopes `verification_py_area_1` to area migration + non-container suites, and gates testcontainers behind `verification_py_area_2` (container profile/policy) and `verification_py_area_3` (live dependency examples). The non-negotiables at `:31-34` forbid using matrix-only evidence as a substitute for live dependencies. The `runner/run.py:266` `report_status` returns `"matrix-passed"` rather than `"passed"` for tier/gate/package selectors, and only `"passed"` when the env probe ran — preserving the existing distinction. The `env_probe` deliberately never invokes `uv sync` (`runner/env_probe.py:44` `uv_sync_invoked: False`).

### Non-blocking suggestions

- `verification/areas/python_interop/runner.py:74` writes the `cloud-boto3` case to `package.latest.json`. If `verification_py_area_2/3` add more `--package` suites (e.g. `cloud-google-cloud-storage`), they'll all clobber the same path. Consider `cloud-<package>.latest.json` once a second package suite is introduced.
- `verification/areas/python_interop/runner.py:110-122` summary lacks a `skipped` field, even though `runner/run.py` tracks `host_dependent_skips` and the peer `verification/areas/runtime_platform/runner.py:68` does surface it. Tier4 in particular is all host-dependent skips and will currently report as `variants=1 failures=0` with no visible skip count. Worth propagating before tier4 evidence is consumed by downstream tooling.
- `verification/areas/python_interop/reports/python_interop_exit_evidence.md:47-53` documents `--report reports/tier1.latest.json` style commands that write into the source tree when run by hand. The orchestrator correctly writes under `target/`, but copy-pasted commands from the exit evidence would not. Either retarget those examples to `target/…` paths or add a clarifying note.
- `plans/reviews/active/python-interop-area-migration-review-1.md` is checked in as an empty file. If it's meant to hold this review, populate it; otherwise drop it.
- `plans/roadmap.md` and `plans/phases/index.md` still describe the embedded interop track as complete and don't link the new `python-interop-verification-production.md` follow-up plan. Add a roadmap pointer when this PR merges so the next milestones are discoverable.
- `verification/areas/python_interop/runner.py:195` uses `capture_output=True` rather than streaming, so per-suite stdout/stderr is buffered until the subprocess exits. Other adapters (e.g. `runtime_platform`'s in-process pattern, `profile_runner.run_command`) stream live. For long tier runs this means quiet console output until completion. Minor UX nit, not a correctness issue.

### Final verdict
**Satisfied.** Ship it. The milestone is scoped correctly to area migration + non-container profile wiring, the contract with the framework is honored, paths and docs are consistent, and the follow-up plan keeps testcontainers explicitly out of scope. The non-blockers above are good candidates for `verification_py_area_2`.
