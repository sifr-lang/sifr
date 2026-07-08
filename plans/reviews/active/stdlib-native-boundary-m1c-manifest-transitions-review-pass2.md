All six requirements are met and create-pr reports `advisories=none`.

Verification against each M1c requirement:

1. **Compare manifest to main** — `_base_manifest()` at scripts/check_stdlib_manifest_schema.py:255-268 loads via `git show origin/main:...` (configurable via `SIFR_STDLIB_MANIFEST_BASE_REF`); a missing base is a hard failure (line 55-59).
2. **Only documented state transitions** — `ALLOWED_STATE_TRANSITIONS` at lines 41-47 is a closed set; scripts/check_stdlib_manifest_schema.py:203-206 rejects anything not listed. No transitions out of `retained-by-design` or `closing` exist, so those are effectively terminal via omission.
3. **New non-retained-by-design rows rejected** — scripts/check_stdlib_manifest_schema.py:194-200 forces `current_state == "retained-by-design"` for any surface_id absent from base.
4. **Deletions require prior closing + PR-linked closed_surface** — scripts/check_stdlib_manifest_schema.py:208-220 requires `base_state == "closing"` and matching `closed_surface` id; `_validate_closure_records` at :173-180 enforces `previous_state == "closing"`, non-empty `evidence_links`, and `removed_in_pr` matching `_is_pr_reference` (:271-273) which requires `pr #` or `/pull/`.
5. **Active rows cannot have closed_surface records** — scripts/check_stdlib_manifest_schema.py:222-227 intersects current surfaces with closure ids and fails on any overlap.
6. **create-pr clean** — `scripts/run_all_tests.sh --profile create-pr` completed with `advisories=none`; self-test and live manifest both PASS.

Self-tests cover all five transition rules (:375-463) including the happy path and each rejection case.

READY
