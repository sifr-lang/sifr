## Findings

### Verification of the three follow-up edits

1. **`primary-body-only` normalizer entry** — `verification/areas/core_language/data/lowering_layer_inventory.json:14, 35`. Added to both parsed-source rows. Correctly addresses pass-1 residual risk #1 (disclose body-only descent semantics).

2. **`status == "mapped"` requires non-empty replacement** — `verification/areas/core_language/checks/lowering_layer_inventory.py:87-90`. Verified by direct tests:
   - mapped + `null` replacement → fails with "must declare replacement"
   - mapped + `""` replacement → fails with "must declare replacement"
   - mapped + non-empty replacement → passes
   
   Correctly addresses pass-1 residual risk #4.

3. **`status == "active"` requires replacement null** — `lowering_layer_inventory.py:91-92`. Verified: active + non-null `replacement` fails with "active snapshot replacement must be null". Sensible tightening; the current inventory rows already comply.

4. **`snapshot_id` derives from fixture stem** — `lowering_layer_inventory.py:176`. `expected_snapshot_id = f"{fixture_path.stem}.{collection}.{fixture_id}"`. Verified: a row with a wrong-stem `snapshot_id` is now rejected. Correctly addresses pass-1 residual risk #3 — Wave 5.2+ fixture files (e.g. `hir_lowering_matrix.json`) will produce `hir_lowering_matrix.<collection>.<id>` without code changes.

### Non-blocking observations

- `status == "deferred"` still has no constraint on `replacement` (any value, including null or string, accepted). That is consistent with the meaning of "deferred" — replacement may not yet exist. No action needed.
- `fixture_path.stem` collisions across different directories (two `*.json` files with identical stems in different subdirs) are theoretically possible but irrelevant to current scope. Worth keeping in mind if the inventory grows broad enough to span multiple data dirs.
- Pass-1 residual risk #2 (the inventory check is purely static and does not transitively invoke the Rust shape test) is unchanged. That was flagged as a Wave 5.2+ planning item, not a Wave 5.1 blocker.

### Verdict

**No blocking findings.** The three follow-up edits are correct, well-scoped to the residual risks raised in pass 1, and introduce no new blockers. The data file still validates (`lowering layer inventory ok`, exit 0) and matches the user-reported `sifr_verify` results.

**Another review round is NOT required.** Pass 2 can be closed as approved.
