# Coverage Matrix And Taxonomy Ownership

The `compiler-verification` owner maintains this area's
[manifest](manifest.json), shipped guarantees, compiler surfaces, profile
assignments, and naming checks. The executable `readiness` suite is the source
of current readiness; dated reports under `reports/` retain their original
observations.

| Registry or check | Owned paths and meaning |
| --- | --- |
| [Shipped guarantees](shipped_guarantees.json) and [compiler surfaces](compiler_surface_matrix.json) | Guarantee and surface identities refer to owners in [the owner registry](../../owners.json) and executable area/suite tokens. |
| [Profile assignments](profile_assignment_matrix.json) | The [assignment checker](checks/profile_assignment_matrix.py) compares the four delivery profiles in [profiles](../../profiles/) with area manifests. Python delivery coverage is also derived directly from the [Python manifest](../python_interop/manifest.json). |
| [Cargo classification](data/cargo_metadata_classification.json) | Package names are checked against offline, locked `cargo metadata --no-deps`; classification does not run package tests. |
| [Taxonomy checker](checks/verification_taxonomy.py) | Scans the active source, documentation, verification, script, workflow, and editor roots declared in `ACTIVE_ROOTS`. It owns naming checks, not runtime suite selection or fixture execution. |
| [Python evidence topology](../python_interop/README.md) | `runtime/python-interop` owns package matrices, declaration ledgers, fixture-relative source paths, repository-relative generated reports, and the compiled HTTPX2 client. |

Python delivery coverage is enforced by
[`sifr_verify.profiles`](../../runner/sifr_verify/profiles.py): every non-live
suite in the Python area manifest must be selected by at least one of
`create-pr`, `merge`, `nightly`, or `release`. Effective `network_mode` comes
from the suite or its area default; live suites are outside that requirement.
The opt-in live-service profile does not count as delivery-profile coverage.
This readiness check establishes profile reachability, not that the selected
Python suites have executed in the current run.

Runtime-generated artifacts live below repository-relative `target/` paths.
Taxonomy excludes `target`, `.venv`, `__pycache__`, `node_modules`, `third_party`,
`vendor`, `.git`, and `skills` directories according to the checker. Historical
issue and review records under `plans/` are outside its active roots. These
boundaries preserve recorded environments and upstream ownership; active
documentation still uses current API identities and actual repository paths.

Run the named readiness suite with:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix --suite readiness
```

The area emits `target/verification/areas/coverage-matrix-results.json`.
This report is generated run evidence and does not replace any Python area's
compiled-example report or retained performance measurement.
