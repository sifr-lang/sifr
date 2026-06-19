I've reviewed the milestone_py_11 implementation against the phase contract. Below are findings ordered by severity.

## Findings

### Blockers
None.

### Non-blocking — worth flagging

1. **`tensorflow` host-dependent skip lives in Tier 1b**
   `verification/python_interop/packages/data.toml:16-23` marks `tensorflow` as Tier 1 + `host-dependent = true`, which is allowed by the validator (`runner/certification_matrix.py:51-52` only forbids Tier 2/3 host-dependence). It produces the single Tier 1 skip in the observed report. The phase contract DoD reads "Tier 1 package certification is green in the full Python interop gate", and the contract Tier 1b "Data/binary/parser" list includes tensorflow without a host-dependent caveat. Status today is technically "passed with explicit skip", which is defensible (wheel/AVX availability varies) but the contract should be updated to allow Tier 1 host-dependents explicitly, otherwise milestone_py_12 will reopen the policy question.

2. **`status = "passed"` is identical for "matrix evidence assembled" vs. "packages actually loaded"**
   `runner/run.py:144-150` and `report_status` return `"passed"` whenever a tier/gate/group/package selector is present, even though no import or smoke test runs. `summary.total_failures` and `package_certification.status` disambiguate per record, but the top-level field will read the same once live package gates exist. Consider a `"matrix-only"` / `"deterministic"` status now, or note explicitly in the report payload that this is matrix evidence, so milestone_py_12's live gate has a discriminator to flip.

3. **`google-cloud-pubsub` is in cloud.toml but not in the phase contract Tier 1b list**
   `verification/python_interop/packages/cloud.toml:64-69`. The fixture (`fixtures/pubsub/pubsub_contract.json`) and the phase's Pub/Sub-callback pattern motivate adding it, but it is a contract drift. Either add it to the phase contract's Tier 1b "Cloud/AI clients" enumeration or note the deliberate addition in the README/phase tracker.

4. **`avro` and `avro-python3` collide on the `avro` import root**
   `data.toml:79-90`. Deterministic matrix tracks both; the real environment can only install one. This is a phase-contract policy artefact (both names are in the phase list), not a runner bug, but the live gate will eventually need to choose one. Worth surfacing as a known eventual conflict.

### Nits

- `runner/certification_matrix.py:53-54` checks `entry.import_roots` is non-empty, but `runner/import_matrix.py:46-47` always backfills via `default_import_root`, so the assertion is dead under the parser. Either drop the check or comment it as defense-in-depth.
- `verification/python_interop/packages/async.toml:2-6` contains `urllib3` (sync HTTP) — pure file-organization choice; doesn't affect behavior, but worth moving to a non-async file if you ever sort matrix files by category.
- Phase contract notes "packages already covered by Tier 1a may reappear in Tier 1b when they anchor a category". The implementation enforces one canonical gate per package via `(name, tier)` dedup and validates the Tier 1a set exactly — fine in practice but does mean the contract's "may reappear" wording is now structurally impossible; consider tightening the contract.

### Verified

- Tier 1a set in `CORE_TIER1A_PACKAGES` (`certification_matrix.py:8-30`) matches the phase contract's 21-package list exactly.
- Tier 1 variant count (149), Tier 2 (46), Tier 3 (39), Tier 4 (30) all align with the phase contract enumeration after cross-checking every package across the 9 matrix files.
- Every host-dependent entry has a `skip_reason` and every Tier 4 entry is host-dependent (policy validator at `certification_matrix.py:44-54` and matrix data agree).
- No implicit uv sync: `runner/run.py:138` runs `run_env_probe` only when `--group env` is selected; tier/gate/package selectors emit purely from disk data. Fixture JSON validation (`run.py:225-229`) is `json.loads`-only.
- All 22 fixture JSON contracts are valid, reference packages present in the matrix, and declare `host_policy` (`fixtures/*/contract.json` reads match the `REQUIRED_FIXTURE_FILES` list and the matrix).
- Negative self-tests (`run.py:280-327`) cover unknown filters, missing tier1 gate, gate on non-tier1, missing tier4 skip, and certified/skip-count drift.

reviewer satisfied: no blockers
