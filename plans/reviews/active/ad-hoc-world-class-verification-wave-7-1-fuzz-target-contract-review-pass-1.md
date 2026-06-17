I've inspected the changed files, the runner code, the helper scripts, the fuzz_property area runner, the diagnostic_rendering_harness binary it points at, the existing area manifest, and the Wave 7 / 7.1 sections of the tracker. Findings below.

## Verdict

**No blocking findings.** Wave 7.1 lands as advertised: a target-contract and minimization-command slice. The runner validation is real, the manifests are coherent with the validator, all referenced paths resolve, the policy/tracker text does not overclaim cargo-fuzz or sustained execution, and the focused validation in the task summary (variants=45 failures=0) demonstrates the happy path. Another Opus review round is **not required** before PR/create-pr.

## Findings (ordered by severity)

### Notable, non-blocking

1. **Per-target `corpus_dir`, `seed_files`, and per-target commands in `fuzz_smoke_manifest.json` are documentary, not driven by the runner.** `run_fuzz_smoke_suite` (`verification/runner/sifr_verify/hardening/property_and_fuzz.py:319,406`) reads only the top-level `seed_files` and the top-level `command: "check"`. So the actual fuzz iterations today exercise the `parse_check_entrypoint` + `hir_type_ownership_entrypoint` lanes (both reachable through `sifr check`); `codegen_entrypoint`, `diagnostic_renderer_entrypoint`, and `package_project_manifest_entrypoint` get only contract validation, not mutation iterations. This is consistent with the wave being scoped to "target contract and minimization commands," but `verification/policy/fuzz_property.md:6-12` would read more honestly if it stated explicitly that the deterministic smoke gate currently mutates only Sifr source; per-target structures are scaffolding for later waves.

2. **`validate_property_target_contract` (`property_and_fuzz.py:488-502`) does not cross-validate `program_class` against the referenced target's `program_class`.** For example, a property entry could declare `valid-only` but link `target_ids: ["diagnostic_renderer_entrypoint"]` (target's class is `structured-diagnostics`) without being flagged. The four current property entries are coherent, but the contract is one step short on consistency enforcement.

3. **No unit tests cover the new validators.** `validate_fuzz_target_contract`, `validate_property_target_contract`, `validate_command_paths`, and `load_known_target_ids` are exercised only through the happy-path area run. A small negative-path test (e.g., a perturbed in-memory manifest asserting the expected `mismatches`) would harden the slice and make future regressions visible. Not blocking for this PR because the runner already fails-closed on malformed input via `SystemExit` and metadata mismatch.

4. **`reproduction_command` for `codegen_entrypoint` points at the property suite (`--suite property`), not the fuzz-smoke suite.** This is correct (PROP-INT-0001/0002 live there) but means a contributor who runs only `--suite fuzz-smoke` while debugging a codegen-class finding will not actually hit codegen. A one-line note in `verification/areas/fuzz_property/sustained_lane.md` or in the codegen target's note would prevent that confusion.

5. **`diagnostic_renderer_entrypoint` and `package_project_manifest_entrypoint` share the same `reproduction_command` (`cargo run … --bin diagnostic_rendering_harness`).** Defensible — `crates/sifr_driver/src/bin/diagnostic_rendering_harness.rs:129-220` covers both `check_project`/`check_package_project` fixtures and renderer assertions — but worth a one-line cross-reference in the project-tree target so a reader doesn't think this is a copy-paste mistake.

6. **`required_target_ids` is compared via `set(required_target_ids) != REQUIRED_TARGET_IDS` (`property_and_fuzz.py:510`).** Duplicates in the list are not flagged. Cosmetic — the list is canonical and not user-extended.

7. **`minimize_project_tree.py:30` blows away `output` if it exists** (`shutil.rmtree(output)`). Only triggers when the caller passes `--output` at a populated directory, which is what they asked for, but a future caller could accidentally lose state. Switching to `shutil.copytree(..., dirs_exist_ok=True)` (Python ≥3.8) would be safer.

8. **`minimization_command` entries contain literal `<failing-source>`/`<failing-rendered-diagnostic-json>`/`<failing-project-dir>` placeholders.** The runner correctly accepts them (`validate_command_paths` only checks `.py` parts). The policy file at `verification/policy/fuzz_property.md:67-70` shows correct usage, but a one-line `README.md` under `verification/areas/fuzz_property/checks/` would help future contributors.

### Honesty check (no findings)

- **No cargo-fuzz overclaim.** Tracker (`plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:1296-1301`) scopes Wave 7.1 to "target-contract and minimization-command slice." The Wave 7 task list (`:1306-1330`) explicitly leaves cargo-fuzz, sustained lanes, and sanitizer lanes as future work. Status line (`:3`) labels it "in progress on branch …" without claiming Wave 7 completion.
- **No sustained-lane overclaim.** `sustained_lane.md` is framed as "Non-blocking" status contract with documentation-only operational notes; no runner code claims to execute the 10/30-minute budgets.
- **Parser-fork separation is preserved.** `verification/policy/fuzz_property.md:28` explicitly excludes `third_party/ruff/fuzz` from Sifr-original target claims.
- **Diagnostic-renderer grammar is documented separately** (`policy/fuzz_property.md:45-56`) so it cannot be confused with parser fuzzing.
- **Path resolution is correct.** `Path(__file__).resolve().parents[4]` in each minimization helper lands at the repo root from `verification/areas/fuzz_property/checks/`.
- **Validator coverage matches the slice's enforcement claims:** missing/duplicate target ids, missing/non-file seeds, malformed `reproduction_command`/`minimization_command`, missing `.py` script files, unknown property `target_ids`, missing required program_class, and `corpus_dir` not a directory are all enforced in `validate_fuzz_target_contract` and `validate_property_target_contract`.

### Suggested Wave 7.2 (not blocking 7.1)

- Per-target fuzz dispatch (so `codegen_entrypoint`, `diagnostic_renderer_entrypoint`, and `package_project_manifest_entrypoint` get their own mutation lanes, not just metadata).
- Cross-field validator for property/target `program_class` compatibility.
- Negative-path unit tests on the contract validators.

## Recommendation

Proceed to PR and `scripts/run_all_tests.sh --profile create-pr`. The slice is honest, the runner validation is enforceable, and the validations already run cover the surface this slice actually changes.
