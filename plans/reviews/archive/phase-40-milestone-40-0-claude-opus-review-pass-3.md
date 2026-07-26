# Re-review: `milestone_40_0` — Architecture and Gate Lock (pass 3)

Scope inspected: the complete working tree (39 modified, 23 untracked), the Phase 40
plan (`milestone_40_0` scope, DoD, Canonical Cutover Policy, Quality Contract), the
execution issue, and review passes 1 and 2. Re-run locally during this pass:
`governance.selftest` (14/14), `governance.schema_epoch`, `governance.evidence_custody`,
`documentation/check_structure.py` (incl. the GA mutation harness),
`demos/milestone_40_0_demo.sh`, `preview_release_workflow_yaml_parses.sh`,
`python -m sifr_verify --self-test` (11/11, including the documentation, release-report
precondition, and release-report production self-tests),
`cargo test -p sifr rejects_stable_metadata` (1 passed), and
`scripts/check_file_size_guardrails.py` (PASS, 2841 files, limit 900).

## Pass-2 findings: re-audit

| # | Pass-2 finding | Status |
|---|---|---|
| 1 | `release_plan.py` `incident_request_sha256` `KeyError` | **Fixed.** Conditionally required at `release_plan.py:56-59`; schema `allOf/if/then/else` in `stable_release_plan.schema.json`; omission and stray-field mutations at `selftest.py:494-504` |
| 2 | `incident.py` `rollback_target` `KeyError` | **Fixed.** `incident.py:50-57`; schema conditional present; omission mutation at `selftest.py:520` |
| 3 | Governance schemas never applied | **Fixed.** `schema_contracts.py:20-43` registers a fixture per schema, asserts registration parity against `schemas/*.schema.json`, and validates each through `verification/json_schema_202012.py`, whose `lint_schema` rejects unsupported keywords (real linting, fail-closed). Driven from `selftest.py:337`. I additionally differentially fuzzed schema-vs-validator over every single-key deletion: **zero cases where the schema rejects and the validator accepts**; the only two divergences (`qualification_artifact_index artifacts[].target`, `stable_release_plan.submodules.*`) are the validator being stricter, which is the safe direction |
| 4 | `create_new_version.sh` plan/index split | **Fixed.** `read_current_channel_versions()` (`:153-176`) now prefers `${RELEASE_INDEX}`, which `validate_inputs` (`:192`) makes mandatory for `--real-run`, so plan text, `PLAN_SHA`, checklist, and the generated index (`:369-390`) all derive from one authority |
| 5 | `rejects_stable_metadata` tested nothing | **Fixed.** `self_update_metadata.rs:719-731` now builds from `metadata_payload()` (`ga_status: "preview"`, valid alpha/beta records) and asserts the distinct `stable channel metadata is disabled` message from `:205-210` |
| 6 | No real CLI JSON producer↔consumer parity | **Fixed.** `cases/self_update_json_surface_parity.sh` builds a real `sifr`, stages a schema-v2 receipt + sysroot, and feeds actual `self version --format json` and `self update --dry-run --format json` bytes to `release_governance.py validate`. The `sysroot_sifr_version == receipt_version` note is a non-issue: `self_update_cli.rs:276,301` sources that field from the receipt, and `matches_receipt` compares the running binary, not the sysroot |
| 7 | Schema-epoch allowlist | **Fixed.** `schema_epoch.py:17-70` globs the governed surface set and hard-fails if `self_update_receipt.rs`, `generate_version_installer.sh`, or `tools/validate_self_update_metadata.sh` fall out of scan scope. No false positives observed: the `(?<![A-Za-z0-9_])` guard and quoted-key patterns correctly ignore `sysroot_schema_version` and `schema-version = 1` |
| 8 | Stale real-run example in docs | **Fixed.** `internal_docs/distribution_pipeline.md:328` |
| 9 | Planning drift | **Fixed.** Execution issue `:31-35` now describes owner/behavior/boundary/disposition (matching the inventory fields validated at `selftest.py:355-361`); `plans/roadmap.md:85` is `in progress`; the phase header is `in-progress`; pass-2 is no longer a placeholder |
| 10 | Docs self-test executed the real area | **Fixed.** `selftest.py:475-511` injects a stub `command_runner`, so no lane runs the `documentation` area outside its selection |
| 11 | YAML case checked fragments only | **Fixed.** `preview_release_workflow_yaml_parses.sh:10-30` parses the YAML and asserts (a) the `validate` job rejects stable input, (b) `live_metadata=` precedes `gh release upload channels` inside `publish-release` |
| 12 | Documentation registration partly self-enforcing | **Fixed.** `check_structure.py:79-84` requires an `active` check's suite to exist in the manifest, with a `missing-active-suite` mutation at `:105`; `check_ga_release_docs.py:43` now exercises the `unsupported-rust-claim` overclaim |
| 13 | grep-based dispatcher parser | **Adequately handed off.** `generate_dispatchers.sh:125-134` still text-matches, and the execution issue (`:150-153`) explicitly assigns the strict schema-v2 active/preview parser and stable behavior to `milestone_40_2`, which the phase plan already scopes. Requesting it here would be scope creep; 40.0 correctly keeps stable resolution unavailable |

Spot-verified beyond the pass-2 list: the preview workflow's compare-and-swap is real
(expected generation/digest captured at `preview-release.yml:229-230`, re-fetched and
re-compared at `:302-320`, all inside the workflow-level `sifr-release-index`
concurrency group at `:27-28`); `release_evidence.py:242-251` splits `developer_tooling`
`full` into non-editor and `editor-release` evidence so `editor-release:*` appears
exactly once, reinforced by the duplicate-suite check at `release_report.py:219-221` and
the prefix check at `:212-214`; `release_report.py:32-43` mandates
`rust_interop:stable-candidate` and `developer_tooling:editor-release`, so a real
`--release-report-out` run is fail-closed on the missing prerequisite rather than
silently green; version-asset `--clobber` at `preview-release.yml:283` is correctly
`milestone_40_2` scope.

## External integration blocker (unchanged; do not implement here)

`verification/areas/rust_interop/manifest.json` still exposes only `matrix`, `tiers`,
`compatibility-matrix`, `stale-drafts`, and `verification/areas/rust_interop/data/`
contains no `stable_support_claims.json`. `verification/profiles/release.json`
therefore cannot list `stable-candidate`, while `release_report.py:38` requires it.
Integration once the prerequisite merges: append `"stable-candidate"` to that one array
in `release.json`; it flows through `run_rust_interop_checks` → `run_selected_area` →
`validate_area_result` and satisfies both `validate_profile` and `validate_steps`. No
other local change is needed.

## Local findings

**1. MEDIUM — governed enum checks raise `TypeError` instead of a governed rejection, at
nine sites; this is the unfixed root cause of pass-2 findings 1 and 2.**

Pass 2 fixed two named sites; the underlying pattern — testing an unvalidated value with
`value not in {…}` — still crashes on any unhashable value. Reproduced end-to-end:

```
$ python3 scripts/distribution/release_governance.py validate \
    --kind release-index --input <index with "status": []>
  File ".../governance/release_index.py", line 83, in validate_release_record
    if status not in {"active", "withdrawn"}:
TypeError: unhashable type: 'list'
```

An exhaustive value-corruption sweep over every field of all eleven governed fixtures
found exactly this class at:

- `release_index.py:32` (`ga_status`), `:83` (`release[].status`)
- `release_plan.py:65` (`transition`), `:77` (`desired_release.status`), `:251`
  (`advertised_claim_ids` via `len(set(claims))`)
- `incident.py:165` (`attempt.mode`), `:168` (`attempt.status`) — reached from both
  `validate_release_signoff` and `validate_incident_signoff`
- `artifact_index.py:75` (`artifact.kind`)
- `surface_contracts.py:141` (`plan.action`)

`release_governance.py:146` and `evidence_custody.py:33` catch only `GovernanceError`,
so every one of these surfaces as a traceback with exit 1 rather than the governed
`release-governance: …` / `evidence-custody: …` diagnostic and exit 2. That contradicts
the phase Quality Contract's stable-diagnostic and stable-exit requirements and the DoD
language that validators *reject* these payloads. A central `require_enum(value,
allowed, location)` in `common.py` (string check first) fixes all sites at once.

**2. MEDIUM — `artifact_index.py:56` raises a bare `ValueError` for a malformed
`expires_at`, with no mutation coverage and no schema backstop.**

```
$ python3 scripts/distribution/release_governance.py validate \
    --kind qualification-artifact-index --input <expires_at: "not-a-timestamp">
ValueError: $.workflow.expires_at: must be an ISO-8601 timestamp
exit=1
```

`GovernanceError` subclasses `ValueError`, but not the reverse, so this escapes both
governed entrypoints exactly as in finding 1. The schema cannot catch it either:
`qualification_artifact_index.schema.json` declares `"format": "date-time"`, and
`verification/json_schema_202012.py:198-203` accepts `format` as a keyword without
enforcing it. `test_artifact_index_mutations` (`selftest.py:606-633`) has a single
mutation (`artifacts[0].target`) and no `expires_at` case — the thinnest coverage of any
governed artifact. Raise `GovernanceError` via `fail()` and add the mutation; since the
artifact index is the `milestone_40_1`/`40_5` transport-and-expiry gate, an expiry-value
mutation is worth adding alongside it.

**3. LOW-MEDIUM — `selftest.py:315` accepts a bare `ValueError`, so the harness
structurally cannot detect findings 1 and 2 or any recurrence.**

```python
except (GovernanceError, ValueError) as exc:
```

Production callers catch only `GovernanceError`; the self-test accepts either. That gap
is why the pass-2 remediation could fix two sites without the class being detected at
the nine others. Narrowing this to `GovernanceError` is the highest-leverage change in
this pass: it converts findings 1 and 2 from review artifacts into mechanically enforced
ones and permanently locks the fail-closed contract this milestone exists to establish.

**4. LOW — `schema_epoch.py:29` character class is `[^\\n]`, not `[^\n]`.**

Inside `r"…"`, `[^\\n]` excludes a literal backslash and a literal `n`, not a newline, so
the `schema_version … == 1` pattern silently cannot match across any span containing the
letter `n`. The self-test at `:83` passes only because its fixture has no intervening
`n`. Narrower than intended; a false negative, not a false positive.

**5. LOW — `docs_inventory.json` mutation-case names are nominal, not bound to executed
mutations.**

`check_structure.py:63-67` only requires ≥3 unique strings. `ga-release` declares four
names while `check_ga_release_docs.run_self_test()` runs five unnamed mutations; nothing
proves the declared `unsupported-rust-claim` corresponds to the executed
`all Rust crates are supported` case. It happens to be true today (verified), but the
registration is documentation, not a contract. Worth binding when `milestone_40_4`
activates the `ga-release` suite.

**6. LOW — `run_documentation_checks` reads `legacy_facade.documentation_suites`
(`profile_runner.py:256,513-524`) while the report validator reads
`selected_areas[documentation]`.** `_documentation_profile_self_test:455-463` asserts the
two agree for the `release` profile only; other lanes can diverge silently. Non-blocking
for 40.0, since only `release` produces evidence.

## Verdict

**CHANGES REQUESTED — 6 local findings.**

All thirteen pass-2 findings are genuinely resolved in code, with mutation coverage and
schema/validator parity that I verified by differential fuzzing rather than by reading
the remediation notes. Documentation registration, workflow semantic ordering, real CLI
JSON parity, release-index authority for `create_new_version`, schema-epoch repository
coverage, release-report production purity, planning status, and file-size/style
constraints are all satisfied, and the 40.2 dispatcher-parser handoff is adequately
tracked.

What blocks approval is that pass-2's two MEDIUM findings were fixed *at their two
reported sites* rather than at their root cause: the same "unvalidated value reaches an
enum or parse check, producing an uncaught exception instead of a governed rejection"
defect remains reachable at nine further sites (findings 1 and 2), and the self-test
harness is written so that it cannot detect it (finding 3). For a milestone whose entire
purpose is locking one fail-closed governance contract before mutation-capable work
begins, that class should be closed centrally and enforced by the harness. Findings 1–3
are small, mechanical, and mutually reinforcing: a `require_enum` helper in `common.py`,
`fail()` in `artifact_index.py:56`, and narrowing `expect_rejected` to
`GovernanceError`.

`rust_interop:stable-candidate` and `stable_support_claims.json` remain correctly scoped
as an external integration blocker owned by another worktree; it is not the reason for
this verdict.
