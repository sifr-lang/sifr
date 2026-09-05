## Verdict: BLOCKED

**Blocking findings**

1. **`check_fixture_matrix.py` will fail** — `REQUIRED_FIXTURES` (verification/areas/rust_interop/checks/check_fixture_matrix.py:21) does not include `opaque_resource_core`. The new fixture directory + fixture-matrix row will trigger both `unexpected fixture directory: opaque_resource_core` and `unexpected fixture matrix entry: opaque_resource_core`. Local validation (`scripts/run_all_tests.sh`) will fail.

2. **Fixture is missing `fixture.json` and evidence files** — check_fixture_matrix.py:172 requires `fixture.json` for any listed fixture, and `_validate_evidence` requires evidence paths on disk. The new fixture ships only a README. Compare to `opaque_resource_matrix/` which has `fixture.json`, `positive/`, `negative/`, `examples/`. Either add the full fixture scaffolding or narrow the row so this check doesn't apply.

3. **Evidence overclaim** — the compat/fixture matrix rows declare `positive_evidence.status = "passing"` for ids `stdlib_handle_close_poison_lifecycle` / `stdlib_handle_double_close_poisoned_access`, but those ids appear nowhere in the Rust test suite (only in the matrix JSON). The gate script now trusts these `status: passing` claims to unlock supported-row treatment — the ids should be real test names or files, not a README pointer to "`cargo test -p sifr_runtime interop`" (which is a whole module, not identifiable evidence).

**Non-blocking nits**

- Gate script `_is_supported_stdlib_core` (scripts/check_sysroot_stdlib_resource_certification_gate.py) accepts *any* row_id ending in `_core` with passing evidence and no `future_owner`. That's a broad key — consider anchoring to an explicit allowlist (e.g., `{"opaque_resource_core"}`) so future misnamed rows can't slip past the gate. At minimum the self_test doesn't cover the "ends in `_core` but not stdlib-owned" case.
- Doc bump "14 → 15 supported rows" is consistent with the new row, but the sentence about `opaque_resource_matrix` being "future-owned" is now duplicated between the paragraph you edited and the surrounding context; fine, but a nit.
- README's "Scope note" is good; consider adding it to the compatibility matrix `notes` field too (already partially there).

**Another pass needed:** yes — after (1) adding `opaque_resource_core` to `REQUIRED_FIXTURES`, (2) landing a real `fixture.json` + evidence files (or explicitly justifying a runtime-only exemption in the check), and (3) grounding the evidence ids in named tests. Then re-review.
