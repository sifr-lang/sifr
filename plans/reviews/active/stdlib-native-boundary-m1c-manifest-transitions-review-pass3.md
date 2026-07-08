Reviewing the diff against the pass-1 blocker checklist:

**Verified OK:**
- CI checkout: `fetch-depth: 0` added on all four jobs in `.github/workflows/local-first-validation.yml:20,44,58,74` ✓
- PR references: `_is_pr_reference` at `scripts/check_stdlib_manifest_schema.py:274-278` requires `PR #<digits>` or `/pull/<digits>` (numeric-only) ✓
- Git base-manifest error surfaced: `scripts/check_stdlib_manifest_schema.py:55-61` appends failure with `base_ref` and `base_error` (git stderr from line 269) ✓
- Self-test — all 5 allowed transitions: `scripts/check_stdlib_manifest_schema.py:381-391` ✓
- Self-test — new retained-by-design positive: `scripts/check_stdlib_manifest_schema.py:424-438` ✓
- Self-test — duplicate closure ids: `scripts/check_stdlib_manifest_schema.py:500-522` ✓
- Self-test — deletion checks (closing→closed with/without closure, non-closing deletion rejected): `scripts/check_stdlib_manifest_schema.py:440-471` ✓
- Self-test — empty-surface closure validation: `scripts/check_stdlib_manifest_schema.py:524-532` ✓

**Blocker:**
- `scripts/check_stdlib_manifest_schema.py:224-229` — the "closed_surface records must not reference active surface rows" check has no positive self-test. Nothing in `_self_test` constructs a manifest where a `current_surfaces` id also appears in `closure_records` under `_validate_transitions`, so a future regression that silently drops this check would still show `self-test: PASS`. The `bad_closure` case at line 473 only exercises `_validate()` (inline closure-record validation), not the transition-level active/closed intersection. The user's checklist explicitly requires this coverage.

BLOCKED
