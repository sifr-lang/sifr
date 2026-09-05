# Re-review: `milestone_40_0` — Architecture and Gate Lock (pass 5)

Scope inspected: the complete working tree (39 modified, 23 untracked), the Phase 40 plan
(`milestone_40_0` scope and DoD, lines 356–472), the execution issue, and review passes 1–4.

Re-run locally during this pass: `governance.selftest` (14/14 pass), `governance.schema_epoch`
(pass), `documentation/check_structure.py` including the GA mutation harness (pass),
`preview_release_workflow_yaml_parses.sh` (pass), `python -m sifr_verify --self-test`
(11/11 pass), `demos/milestone_40_0_demo.sh` (exit 0, canonical plan rendered without network
or repository mutation), `cargo fmt --check` (clean), and
`scripts/check_file_size_guardrails.py` (PASS, 2841 files, limit 900).

Two independent adversarial sweeps were re-run rather than re-read:

- **Exhaustive value-corruption sweep** over every field of all eleven governed fixtures
  (18 corrupt values + deletion per path, 8626 cases) against all eleven validators:
  **zero** non-`GovernanceError` escapes. The pass-3 raw-exception class remains closed.
- **Differential schema-vs-validator sweep** with the same grid: exactly **one** divergence
  in the unsafe direction (schema rejects, validator accepts) — finding 1 below.
- Supplementary static sweep: every `pattern` in all eleven checked-in schemas is anchored
  `^…$`, so `json_schema_202012.py:205`'s use of `re.search` is not exploitable.

## Pass-4 findings: re-audit

| # | Pass-4 finding | Status |
|---|---|---|
| 1 | `plan_id` enforced by schema but not by the validator; not bound to `plan.version` | **Fixed.** `common.py:27-29` adds `PLAN_ID_RE` with a named `version` group matching the schema shape at `stable_release_plan.schema.json:31`; `common.py:116-124` `require_plan_id` rejects non-strings, shape violations, and any `plan_id` whose embedded version differs from the plan's; `release_plan.py:66` calls it with `plan["version"]`. Mutation coverage at `selftest.py:484-487` (`stable-0.1.1-…` against a `0.1.0` plan). The differential sweep reports zero `plan_id` divergences, and `evidence_custody.validate_candidate_directory:164` already binds `plan["version"]` to the directory, so candidate identity is now closed end to end |
| 2 | Malformed **and** timezone-less `expires_at` must be rejected by validator and schema | **Fixed.** `artifact_index.py:54-59` routes the parse failure through `fail()` and rejects `tzinfo is None`; `json_schema_202012.py:207-213` enforces both halves for `format: date-time`, and `:90-91` rejects any other `format` value so the keyword cannot be added decoratively. Both branches are now executable: validator mutations at `selftest.py:660-663` (`"not-a-timestamp"`) and `:664-672` (`"2026-08-01T00:00:00"`), schema mutations over the same two values at `schema_contracts.py:28-43`. Deleting either branch now fails a suite |
| 3 | `structure` mutation IDs bound exactly to executed callbacks | **Fixed.** `check_structure.py:115-126` is now a `{case_id: callback}` dict, `:127-128` asserts `tuple(mutations) == STRUCTURE_MUTATION_CASES`, and `:129` iterates by key — the same shape as `check_ga_release_docs.py:47-58`. Combined with the inventory equality check at `:80-81`, a declared name with no executed mutation is now impossible for both documentation checks |
| 4 | `preview-release.yml:69` still admitted `-rc.N` | **Partially fixed.** The input regex at `:70` is now `^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta)\.[0-9]+$`; the `rc` branch is gone and non-alpha/beta publication is still rejected three ways (`:60-62` channel case, `:64-67` stable-looking pin, `:76-77` channel/version disagreement), all re-asserted by `preview_release_workflow_yaml_parses.sh:16-21`. The operator-facing message at `:71` was not updated — see finding 2 below |

Spot-verified beyond the pass-4 list, all still correct: the publish-release compare-and-swap
re-fetches and re-compares generation + digest (`preview-release.yml:302-320`) inside the
workflow-level `sifr-release-index` concurrency group (`:27-28`); `source_sha` pins build and
publish to one resolved commit (`:84`, `:105`, `:150`); `propose_preview_release`
(`release_index.py:141-149`) still refuses backward channel moves and
`validate_release_index_transition:119-122` still enforces generation monotonicity and the
no-active-to-preview rule; `prepare_release_report_output` (`release_evidence.py:43-62`)
refuses in-repo output, pre-existing output, a non-fresh parent, and a dirty tree;
`release_evidence.py:243-251` splits `developer_tooling:full` so `editor-release:*` appears
exactly once; `evidence_custody.require_comparison_base:111-116` fails closed with no base;
`documentation` suite authority is single-sourced through
`selected_suites_for_area` (`profiles.py:140-147`, `profile_runner.py:262-263`) and
`selftest.py:462-463` fails if a second facade key ever reappears; the schema-epoch guard
still hard-fails if a required governed surface falls out of scan scope
(`schema_epoch.py:56-70`).

## Local findings

**1. MEDIUM — `sysroot_schema_version` is pinned to `1` by the receipt schema and by the Rust
reader, but the Python validator accepts any positive integer; this is the pass-4 finding-1
defect class at a second field, and it is not mutation-covered.**

`verification/areas/distribution_release/schemas/self_update_install_receipt.schema.json:37`
declares `"sysroot_schema_version": { "const": 1 }`, and the Rust reader rejects anything else
at `crates/sifr/src/self_update_receipt.rs:189-193`
(`standalone install receipt sysroot_schema_version {} is unsupported`). The executable
governance gate is weaker than both:
`verification/areas/distribution_release/governance/surface_contracts.py:85` applies only
`require_positive_int(receipt["sysroot_schema_version"], …)`, and `:115` does the same for the
`self version --format json` response.

The differential sweep flagged exactly this and nothing else, and it reproduces through the
real entrypoint rather than only in-process:

```
$ python3 scripts/distribution/release_governance.py validate \
    --kind install-receipt --input <receipt with "sysroot_schema_version": 7> --require-canonical
release-governance validation ok: kind=install-receipt input=/tmp/receipt7.json
exit=0
```

The same payload is rejected by the checked-in schema and would be rejected by the compiler
that has to read it. Two consequences:

- A receipt declaring an unsupported sysroot layout passes
  `release_governance.py validate --kind install-receipt` — the gate `milestone_40_2` will use
  when stable installs and `sifr self update` become real, and the gate that
  `cases/self_update_json_surface_parity.sh` uses to certify real CLI bytes.
- `schemas/self_version.schema.json:31` declares the *same* field as
  `{"type": "integer", "minimum": 1}`, so the two published schemas disagree with each other
  about a value that `self_update_cli.rs:293,300` copies verbatim from the receipt into the
  version JSON. `self_version` is therefore weaker than both the receipt schema and the Rust
  reader.

There is also no mutation for this field anywhere in `test_surface_contract_mutations`
(`selftest.py:686-760`): the receipt fixture sets `sysroot_schema_version: 1` at `:689` and the
self-version fixture at `:709`, and neither is ever mutated, so the constraint could be dropped
entirely without failing a suite.

This is squarely in `milestone_40_0` scope — the plan requires this milestone to replace "the
existing install-receipt, `sifr self version --format json`, and self-update-plan schemas,
producers, consumers, fixtures, and tests" (phase plan `:375-380`) — and it is the same shape
as the class the milestone exists to lock: a governed field whose published contract and native
reader are both stricter than the code that actually gates artifacts. The fix is mechanical:
enforce `== 1` in `surface_contracts.py:85` and `:115` (via `fail()`), tighten
`self_version.schema.json:31` to `{"const": 1}` so the two schemas agree, and add one mutation
per surface to `test_surface_contract_mutations`.

**2. LOW — `preview-release.yml:71` still tells operators that `-rc.N` is an accepted pin.**

The regex at `:70` correctly dropped `|rc`, but the failure message one line below is unchanged:

```
echo "::error::version must be a semver prerelease using -alpha.N, -beta.N, or -rc.N"
```

An operator who supplies `0.1.0-rc.1` is now rejected by that exact branch and told that
`-rc.N` is a supported form. The canonical epoch decision is that `rc` does not exist
(`common.py:135` "rc is not supported"; `selftest.py:336` asserts no schema mentions it;
`check_ga_release_docs.py:31` lists `"-rc."` as a forbidden GA claim; the Rust surface rejects
it at `self_update_metadata.rs:281-292`). This is a governed file in
`schema_epoch.GOVERNED_FILES` (`schema_epoch.py:13`), and the message is the only remaining
`rc` surface in it. Fail-closed behavior is unaffected — this is a diagnostic-accuracy defect,
not a hole. Correct it to name only `-alpha.N` and `-beta.N`.

The identical stale string exists at `generate_version_installer.sh:71`,
`build_preview_artifacts.sh:83`, `trigger_preview_release.sh:159`,
`create_new_version.sh:187`, and `generate_dispatchers.sh:88`, where the surrounding regexes
*do* still admit `rc`. Those are pre-existing and each is gated downstream by an alpha/beta
channel check (`create_new_version.sh:181,188`; dispatcher `normalize_channel` at
`generate_dispatchers.sh:93-107`), and the strict schema-v2 dispatcher parser is explicitly
`milestone_40_2` scope. I am **not** requesting them here; only `preview-release.yml:71`,
which this milestone rewrote and left inconsistent with its own regex.

## Informational (no action required in this milestone)

- `docs/self_update.md:51` contains the literal `-rc.N`, which
  `check_ga_release_docs.FORBIDDEN_CLAIMS` (`:31`) rejects. The `ga-release` check is
  `reserved` (`docs_inventory.json`), so nothing fails today, but `milestone_40_4` will trip
  on this sentence the moment it points the GA check at `docs/self_update.md`. Worth
  anticipating there, not fixing here.
- Two entries of `FORBIDDEN_CLAIMS` — `"Windows installer"` and `"-rc."` — have no
  corresponding mutation in `check_ga_release_docs.MUTATION_CASES` (`:33-39`). Same reserved
  suite, same `milestone_40_4` owner.
- `profiles.py:387` adds a `DOCUMENTATION_SUITES` shell export that nothing consumes. This
  matches the pre-existing treatment of `TOOLING_SUITES`/`VALIDATION_SUITES`, which are also
  emitted and unread, so it is consistency with an existing surface rather than new dead code.

## External integration blocker (unchanged; do not implement here)

`verification/areas/rust_interop/manifest.json` still exposes only `matrix`, `tiers`,
`compatibility-matrix`, `stale-drafts`; `verification/areas/rust_interop/data/` contains
`rust_interop_fixture_matrix.json`, `rust_interop_compatibility_matrix.json`, and
`rust_interop_tiers.toml` but no `stable_support_claims.json`; and
`verification/profiles/release.json` `selected_areas[rust_interop].suites` therefore cannot
list `stable-candidate` while `release_report.py:32-43` requires it. The milestone is correctly
fail-closed on this: a real `--release-report-out` run rejects at `validate_profile`
(`release_report.py:148-154`) and `validate_steps` (`:225-228`) rather than emitting
silently-incomplete evidence. Integration once the prerequisite merges remains a one-line
change — append `"stable-candidate"` to that suites array; it flows through
`run_rust_interop_checks` → `run_selected_area` → `validate_area_result` and satisfies both
required-suite checks. No other local change is needed.

The strict schema-v2 dispatcher parser (`generate_dispatchers.sh:125-134` still text-matches)
and stable dispatcher resolution remain correctly assigned to `milestone_40_2` by the execution
issue (`:162-165`) and the phase plan; requesting them here would be scope creep.

## Verdict

**CHANGES REQUESTED — 2 local findings (1 MEDIUM, 1 LOW).**

Pass-4 findings 1, 2, and 3 are fully and correctly resolved, each with executable mutation
coverage rather than a code-only change, and I confirmed the plan-identity fix by re-running the
same differential sweep that produced it — zero `plan_id` divergences across 8626 corruption
cases. Pass-4 finding 4 is functionally resolved (the `rc` input branch is gone and non-alpha/
beta publication is still rejected on three independent checks) but left one stale operator
message. Fail-closed diagnostics, evidence custody, release-index transition safety, workflow
publication safety, release-report production purity, documentation authority and mutation
binding, profile selection/execution agreement, scope discipline, planning status, formatting,
and the file-size guardrail are all satisfied.

What blocks approval is finding 1: a second field, `sysroot_schema_version`, where the
published schema (`const: 1`) and the Rust reader (`self_update_receipt.rs:189`) are both
stricter than the Python validator that actually gates artifacts, verified reachable through
`release_governance.py validate --kind install-receipt`, with no mutation coverage and with the
two published schemas disagreeing with each other about the same value. It is the exact defect
class this milestone exists to lock, on the receipt surface `milestone_40_2` consumes, and the
remediation is a few lines plus two mutations. With that and the one-line message correction, a
sixth pass should be short.

`rust_interop:stable-candidate` and `stable_support_claims.json` remain correctly scoped as an
external integration blocker owned by another worktree; they are not the reason for this
verdict.
