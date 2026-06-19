Now I have a complete picture. Producing review.

# PR Review: `python-interop-container-policy` (verification_py_area_2)

## Verdict
**Satisfied — no blockers. Open PR2.** Findings below are non-blocking suggestions and observations.

## Answers to specific questions

1. **`container-runtime` as explicit resource class** — Yes. Added to all four enum sites: area top-level `resource_classes`, suite-level `resource_classes`, profile `resource_policy.classes`, and per-selection `selected_areas[].resource_classes`. See `verification/schemas/area.schema.json:44, 180` and `verification/schemas/profile.schema.json:65, 246`.

2. **`python-interop-live` avoids the legacy sequence** — Yes. `execution_mode: "selected-areas-only"` short-circuits at `verification/runner/sifr_verify/profile_runner.py:134-135`, dispatching only to `run_selected_areas_only` → `python_interop:live-policy`. Confirmed by the user's `--emit-plan | rg ... && exit 1 || exit 0` check returning clean for `create-pr`.

3. **`live-policy` validates the contract without containers** — Yes. `verification/areas/python_interop/runner/live_policy.py` reads `live_policy.json`, the live manifest suite, the live profile, and offline profiles; it does pure JSON shape and cross-file consistency checks. No subprocess, no Docker, no testcontainers import.

4. **Offline profiles stay clean** — Confirmed. None of `create-pr`/`merge`/`nightly`/`release` select `live-policy`, declare `container-runtime` in `resource_policy.classes`, or set `network_policy.mode = "live"`. `live_policy.py:152-170` enforces the inverse at runtime when the live profile is exercised.

5. **Result/status semantics for PR3** — Adequate. `live_policy.json` enumerates the four future statuses (`policy-passed`, `live-passed`, `structured-skip`, `live-failed`) and the validator pins to that exact set (`live_policy.py:28, 75-76`), giving PR3 a stable contract to extend. Docs in `internal_docs/python_interop_architecture.md` and `docs/python-interop.mdx` describe each.

6. **Production-readiness blockers** — None. See suggestions below.

## Non-blocking suggestions

- **No negative self-test for the live-policy validator.** `runner/run.py` `run_self_tests` (line 288) doesn't exercise `build_live_policy_report` with malformed inputs. The runner has strong negative coverage for matrix/filter/cert paths but the new validator silently passes if regressed. Recommend adding a couple of negative cases (missing key, drifted status set, offline profile poisoned with `live-policy`) before PR3 lands real container code on top.

- **`validate_offline_profiles` doesn't gate `resource_policy.classes`.** `live_policy.py:152-170` checks `network_policy.mode`, `live_network_allowed`, and forbidden suites — but an offline profile could silently add `container-runtime` to `resource_policy.classes` without selecting `live-policy`. Adding a `resource_policy.classes ∩ {container-runtime} == ∅` assertion would close that drift.

- **`python_interop/manifest.json` is now the only area without a top-level `network_mode`.** The diff removes `"network_mode": "offline"`; suite-level `network_mode` is set only on `live-policy`. The schema permits omission, and the readiness gate at `coverage_matrix.py:360` only fires for surface-referenced areas (python_interop is not one), so this is safe today. Keeping a top-level `"network_mode": "offline"` for documentation continuity and letting the suite override it would match every other area manifest in the tree (`grep "network_mode"` on `verification/areas/*/manifest.json`).

- **Schema-required `legacy_facade` is dead weight on selected-areas-only profiles.** `python-interop-live.json` carries a fully populated but unused `legacy_facade` block (lines 75-95). Either tighten the profile schema with a conditional (`if execution_mode == selected-areas-only, legacy_facade not required`) or drop the values to explicit nulls. Not urgent — runner correctly skips it via the `execution_mode` branch.

- **Suite-level uniqueness is hard-pinned to one case.** `live_policy.py:144-149` requires the `live-policy` suite to contain exactly one case with command `python-interop-live-policy`. That's correct for the policy gate, but ensure PR3's testcontainers suites are added as *separate* suites (e.g., `live-redis`, `live-postgres`) rather than additional cases under `live-policy`. Worth a comment in `live_policy.py` or `python_interop_architecture.md` to forestall confusion.

- **Doc consistency.** `verification/areas/python_interop/README.md:27` reads "Live dependency examples are intentionally opt-in. The `python-interop-live` profile uses selected-areas-only execution and currently runs the `live-policy` suite" — clear. The `internal_docs/architecture.md` line about `network_mode=offline` was updated, but consider a single forward-pointing sentence about per-suite `network_mode` taking precedence so future area authors don't re-introduce a top-level `live` mode.

## Observations (no action needed)

- `profile_assignment_matrix.py` PROFILE_NAMES tuple is correctly scoped to the four readiness profiles; `python-interop-live` is appropriately excluded.
- `selftest.py` `_profile_schema_self_test` now asserts both presence and execution mode of the new profile (lines 82-86) — good.
- `_resource_class_self_test` exercises the union logic with the new class (line 278-284) — good.
- The lambda capture in `run_selected_areas_only` (`profile_runner.py:253`) uses default-arg binding correctly.

Final: ready to ship as PR2; address the negative-self-test and `resource_policy.classes` gap in PR3 prep or follow-up.
