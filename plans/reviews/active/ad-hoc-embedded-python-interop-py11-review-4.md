I reviewed the milestone_py_11 diff against the six blocker axes you listed.

## Blocker checks

**Package certification matrix semantics** — `runner/certification_matrix.py:32-54` enforces all stated invariants: Tier 1a anchor set fixed at 21 (`CORE_TIER1A_PACKAGES` matches phase contract anchor list exactly), `native=true` requires the `native` group, `host_dependent` requires `skip_reason`, Tier 4 must be host-dependent, Tier 2/3 must not be, every entry must expose ≥1 import root. Negative self-tests in `run.py:280-326` exercise each. ✓

**Report status/schema** — `report_status` (`run.py:260-272`) returns `matrix-passed` for tier/gate/group/package selectors without env probe, `passed` only with env evidence, `scaffold` otherwise — confirmed live: tier1/2/3/4 all read `matrix-passed`. `package_certification` block exposes `selected_packages`, `certified_packages`, `host_dependent_skips`, plus `tier_counts`/`gate_counts`/`group_counts` and a sorted `packages` array; `summary.skipped` now tracks host-dependent skips. ✓

**Host-dependent skip policy** — Validator forbids Tier 2/3 host-dependence, requires it for Tier 4, and demands `skip_reason` whenever host-dependent. All 30 Tier 4 entries in `tier4.toml` declare skip-reason; `tensorflow` (`data.toml:16-22`) is the lone Tier 1 host-dependent and ships its skip-reason. ✓

**Fixture coverage** — 15 new contract JSON fixtures land alongside existing ones; all 22 are wired into `REQUIRED_FIXTURE_FILES` (`run.py:43-77`) and json-parsed at every run. Every new fixture declares `schema_version`, `groups`, `packages`, `default_gate`, `host_policy`, and `cases`; package references all resolve in the matrix. ✓

**Package/import-root metadata** — `PackageEntry` gains `import_roots` and `skip_reason` (`import_matrix.py:8-17`); `parse_entry` backfills via `default_import_root` (dashes→underscores). All explicit `import-roots` entries map correctly to actual dotted module paths (`google.cloud.pubsub`, `pydantic_core`, `Cython`, `PIL`, etc.). ✓

**README / phase-contract drift** — README updated to document `matrix-passed`, the certification-record fields, and the tier4 filter. Plan adds `google-cloud-pubsub` to the Tier 1b cloud/AI list, adds the explicit "Tier 1 host-dependent → matrix-only skip + external live evidence" DoD line, and marks `milestone_py_11` complete. ✓

## Validation

`run.sh --self-test` ok. Tier reports match the cited numbers exactly: tier1 149/148/1, tier2 46/46/0, tier3 39/39/0, tier4 30/0/30.

reviewer satisfied: no blockers
