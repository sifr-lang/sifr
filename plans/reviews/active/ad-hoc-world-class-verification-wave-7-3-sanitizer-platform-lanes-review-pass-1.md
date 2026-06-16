# Wave 7.3 Sanitizer And Runtime Platform Lanes Review Pass 1

Reviewer: Claude Opus (code-review)
Date: 2026-06-16
Branch: `codex/wave-7-3-sanitizer-platform-lanes`
Scope reviewed:

- `verification/areas/runtime_platform/sanitizer_manifest.json`
- `verification/areas/runtime_platform/manifest.json`
- `verification/areas/runtime_platform/runner.py`
- `verification/runner/sifr_verify/profile_runner.py`
- `verification/profiles/merge.json`, `nightly.json`, `release.json`
- `verification/areas/coverage_matrix/compiler_surface_matrix.json`
- `verification/areas/runtime_platform/platform_contract.md`
- `verification/areas/runtime_platform/supported_host_matrix.md`
- `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md`

## Verdict

No blocking issues. Wave 7.3 satisfies its declared exit criteria: sanitizer/Miri/deterministic-concurrency lanes are declared with supported host triples, required tools/toolchains/components, exact commands and environments, structured skip reasons, and finding-promotion policy; the runtime-platform runner validates the manifest schema and emits machine-readable per-case skip evidence; profile and matrix integration line up with the declared blocking suite. Another Opus review round is not required before opening the PR. The non-blocking items below are follow-ups that can be addressed in this PR if convenient, or rolled into Wave 10 closeout.

## Blocking Findings

None.

## Non-Blocking Suggestions

1. **Hardening summary line is silent about skips.** `verification/areas/runtime_platform/runner.py:81-86` prints `verification ok: variants=N, failures=0, blocking_failures=0, non_blocking_failures=0`, which is indistinguishable between "all cases passed" and "all cases skipped because the host lacks nightly + llvm-symbolizer" — both of the Wave 7.3 local validation runs are the latter (variants=2/7, all skipped). Adding a `skipped=K` counter to the summary line (and propagating it through `summary` in the result JSON) would surface "the merge gate is decoratively passing today" without forcing a reader to open the per-case JSON. Pattern parallels the fuzz/property summary already used elsewhere.

2. **`sanitizer_manifest.json` shape is only enforced at run-time, not at `areas check`.** `verification/runner/sifr_verify/areas.py:65-66` only validates each area's top-level `manifest.json` against `area.schema.json`. The new `sanitizer_manifest.json` is only parsed when the `sanitizer-smoke`/`sanitizer-full` suite runs. This mirrors the existing `golden/manifest.json` and `platform_contract.json` pattern, so it is not a regression — but adding a sanitizer-manifest-only check that `areas check` can call (or a self-test in `verification/runner/sifr_verify/selftest.py`) would catch a malformed manifest at the cheap gate instead of when a profile runs the suite.

3. **`validate_sanitizer_case` does not reject unknown fields or strict-type the `always_skip` flag.** `verification/areas/runtime_platform/runner.py:273-309` accepts arbitrary extra keys and treats `always_skip` through `bool(case.get("always_skip", False))`, so `"yes"` or `1` would silently coerce to truthy. Either reject unknown keys or restrict `always_skip` to a strict `bool` instance check. Low impact today because the manifest is hand-maintained, but the validator is otherwise quite strict, so this is the one soft spot.

4. **`deterministic-concurrency-model-full` carries a misleading reproduction command.** `verification/areas/runtime_platform/sanitizer_manifest.json:181-200` declares `always_skip: true` with skip reason "no Loom/Shuttle-style deterministic concurrency model is currently vendored," but the `command` is a plain `cargo test --locked -p sifr_stdlib ipc_request_tracker`, i.e., not a deterministic-concurrency-model command at all. Because `always_skip` is set, that command is never executed, but a reader looking at the manifest will plausibly assume that the command demonstrates Loom/Shuttle coverage. Suggested fixes: either (a) drop `command` to an empty list and have `validate_sanitizer_case` allow that when `always_skip: true`, or (b) point the command at a TODO doc / a Wave 9.5 placeholder, with a comment clarifying that the command is the future reproduction target, not today's execution.

5. **Sanitizer merge gate is currently decorative on the declared reference host.** The merge profile lists `darwin-or-linux` / `aarch64-or-x86_64` as the reference host and `sanitizer-smoke` is "smoke where host-supported" per the verification target matrix. The implementation cleanly enforces structured skips when nightly or `llvm-symbolizer` is missing — and on the implementer's host every sanitizer case skipped, so no actual sanitizer command was exercised end-to-end during this wave's local validation. This is consistent with the matrix's "smoke where host-supported" classification, but Wave 10 closeout says "Sanitizer lanes are documented but not executable" is a non-acceptable state (tracker line 1679). A small follow-up before Wave 10: identify at least one supported host (e.g., a CI runner) where the full nightly+llvm-symbolizer stack is provisioned, and check in evidence (report hash or recorded run) that at least one sanitizer case actually passed. Not a Wave 7.3 blocker; calling it out so it does not slip past closeout.

6. **Sanitizer test selection is single-test, not target-scoped.** Each sanitizer case names exactly one `mod::test_function`, e.g., `int::tests::parse_enforces_digit_limit_without_panicking`. That keeps wall time tight, but if the named test is renamed/removed in a refactor, the sanitizer case silently degenerates to a no-op cargo invocation (which may exit 0 because cargo prints "no tests run" without failing). Consider either (a) adding a manifest validator that greps the workspace for the named test at `areas check` time, or (b) widening the test filter to a module path so a single rename does not silently empty the lane. Low priority — most cargo configurations do error on "0 matching tests" only with `--exact`.

7. **Reference-host advisory: `platform-specific` resource class is referenced by merge.json but not declared in `resource_policy.classes`.** This is pre-existing for nightly/release and is unchanged by Wave 7.3; merge.json line 9-14 still declares only `default-local`, while line 111-113 selects `platform-specific`. `selected_resource_classes` (`verification/runner/sifr_verify/profiles.py:57-70`) silently unions the two sources, so nothing fails today. Surfacing this in `validate_selected_area_suites` would catch typos in resource-class names. Out of Wave 7.3 scope, but worth tracking.

8. **Tracker phrasing.** Line 1329 reads "structured skips for missing `llvm-symbolizer` and nightly toolchain"; on this host the skip reasons aggregated **both** plus the always-skip placeholder for the deterministic concurrency case. The summary is accurate enough, but if you want the tracker line to be machine-faithful, add "and always-skip placeholder for the deterministic concurrency model case."

## Correctness Spot Checks

- All five tests referenced in `sanitizer_manifest.json` exist in their declared crates (verified via grep): `int::tests::parse_enforces_digit_limit_without_panicking` (`crates/sifr_runtime/src/int.rs:545`), `json::tests::json_digit_limit_checks_nested_array_numbers` (`crates/sifr_runtime/src/json.rs:392`), `http::tests::http1_malformed_response_maps_to_typed_error` (`crates/sifr_runtime/src/http.rs:604`), `ipc_connection::tests::shutdown_drains_and_rejects_new_runs` (`crates/sifr_stdlib/src/ipc_connection.rs:663`), `timeouts::tests::rejects_non_finite_non_positive_and_overflow_sized_timeouts` (`crates/sifr_runtime/src/timeouts.rs:20`).
- `run_sanitizer_case` has well-defined `failures`/`status` in both the try-success path and the `TimeoutExpired` path; `print_case_timing` and the result dict at line 376-389 see them in scope on both paths.
- The supported host triple check correctly treats `["*"]` as a wildcard and falls through to other reason checks; `deterministic-concurrency-model-full` therefore returns only the `always_skip` reason, not "host not in supported_host_triples."
- `sanitizer-smoke` selects exactly cases tagged `"sanitizer-smoke"` in their `suites` list; `sanitizer-full` is a superset that includes both smoke cases and the five extra full-only cases. The empty-variants guard at line 249-250 protects against typos that would otherwise silently emit a zero-case suite.
- `run_runtime_platform_suites` correctly replaces the previous hard-coded `run_platform_golden_suite` step with a profile-data-driven step that no-ops when an area is unselected. create-pr still selects `platform-golden` only, so its behavior is unchanged.
- Coverage-matrix row diff is clean: `sanitizer_hardening` flips from `expected-missing` (with `issue`/`closes_in_wave`/`expiry`) to `blocking` with the canonical reproduction command; advisory temporary-row count drops 17 → 16 as the tracker claims.

## Wave 7 Sanitizer Requirement Mapping

| Wave 7.3 task | Status |
| --- | --- |
| Sanitizer lanes for generated binaries | covered by `generated-binary-asan-smoke` (sanitizer-smoke + sanitizer-full) |
| Sanitizer lanes for `sifr_runtime` | covered by `runtime-asan-smoke`, `runtime-lsan-full`, `runtime-miri-full` |
| Sanitizer lanes for async/concurrency runtime | covered by `runtime-tsan-full`, `deterministic-concurrency-model-full` (structured skip) |
| Sanitizer lanes for filesystem/process/network runtime | covered by `runtime-http-asan-full` |
| Miri lane | covered by `runtime-miri-full` |
| Loom/Shuttle deterministic concurrency lane | structured skip with reason and reproduction boundary — consistent with "If Miri or Loom/Shuttle-style coverage is skipped, record the determination with reason and reproduction command in the platform or sanitizer manifest" (tracker line 1356) |
| Structured skip on unsupported hosts | implemented in `sanitizer_skip_reasons` |
| Merge selects smoke / nightly+release select full | implemented in all three profile JSONs |
| Coverage matrix row promoted from `expected-missing` to `blocking` | done |

No overclaiming detected: the implementation reports skip (not pass) when prerequisites are missing, never silently swallows an unsupported sanitizer, and the matrix row exactly tracks what the suite enforces.

## Recommended Follow-Up Before Wave 10 Closeout

- Record a "sanitizer-passing" host run somewhere (a CI runner with nightly + llvm-symbolizer, or a maintainer machine with both installed) so that the merge gate has empirical evidence of at least one non-skipped sanitizer case before the Wave 10 closeout policy gates "Sanitizer lanes are documented but not executable."
- Decide where the Loom/Shuttle-style coverage placeholder graduates to real work — likely Wave 9.5 "Runtime/platform executable evidence" — and update the deterministic-concurrency-model case's `command` accordingly when that lane lands.

## Verification Of The Author's Stated Validation

The provided focused validation matches the diff:

- `python3 -m py_compile`: I re-read both touched Python files and confirm there are no syntax issues.
- `jq empty` JSON validations: all six JSON files parse and conform to the relevant schemas inspected here.
- `--hardening-summary` runs reporting `variants=2 failures=0 blocking_failures=0` and `variants=7 failures=0 blocking_failures=0` are consistent with the runner's skip-counts-as-pass aggregation logic and the structured-skip reasons set for this host.
- Merge/nightly profile plans showing `runtime_platform:sanitizer-smoke` / `runtime_platform:sanitizer-full`: matches the `selected_areas` updates and the new `run_runtime_platform_suites` step.

## Final Recommendation

Ready to open the PR for Wave 7.3. Address the non-blocking suggestions in this PR if convenient — items 1, 3, 4, and 6 are small, local edits; items 5 and the Wave-10 closeout follow-up are tracker-level concerns that can be queued. Another Opus review round is not required before PR.
