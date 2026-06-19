# Follow-up review: `python-interop-container-policy` (verification_py_area_2)

## Blockers
None.

## Non-blocking suggestions

- **One negative path is uncovered.** `_validate_offline_profile_payload` checks two surfaces — `resource_policy.classes` (line 221) and per-selection `resource_classes` (line 226-228). The new self-tests only poison the first (`live_policy.py:82-91`). A second case that appends `container-runtime` to the `python_interop` selection's `resource_classes` would fully exercise the new guard. Optional, since the schema/profile/runner already constrain it, but the negative test should match the surface it's protecting.

- **Substring matching in `_expect_policy_failure` is loose.** `live_policy.py:255` accepts any `SystemExit` whose message *contains* the expected substring. If a future edit causes the validator to fail earlier with a different message that happens to share a substring, the negative test still passes silently. Consider tagging each failure with a short stable code (e.g., `"E_LIVE_POLICY_OFFLINE_CONTAINER"`) and matching on that. Minor.

- **Wider applicability of the suite-level-overrides-area-level doc note.** The new sentence in `internal_docs/architecture.md` is python-interop-specific phrasing; the area schema change at `verification/schemas/area.schema.json:165-188` actually makes suite-level `network_mode`/`resource_classes`/`timeout_seconds` available to every area. A one-line generalization next to the schema or in the area authoring section would prevent future authors from re-introducing duplicate area-level live modes.

## Observations (no action)

- Manifest top-level `network_mode: "offline"` restored at `verification/areas/python_interop/manifest.json:165`; live-policy suite carries its own `network_mode: "live"` (line 152). Schema now models both surfaces.
- `run_live_policy_self_tests` is correctly wired into `python interop --self-test` (`run.py:124-125`) and covers the four failure modes called out in review-1: missing key, drifted status set, profile-network drift, offline-profile container-runtime poisoning, plus a manifest network_mode poisoning.
- `_validate_offline_profile_payload` now closes the resource-class gap identified in review-1 suggestion 2.
- `deepcopy` is used for nested-mutation cases (profile, manifest) and shallow `dict()` for the policy-key cases — appropriate.

## Verdict
**Satisfied.** No further review required before PR2 merges.
