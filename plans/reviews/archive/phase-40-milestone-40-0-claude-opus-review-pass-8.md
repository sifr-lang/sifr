I re-audited the full diff, the uncommitted plan/issue changes, and re-ran the adversarial sweeps rather than trusting passes 1–7.

## What I verified independently

**Fail-closed governance core — no regression after the rebase.** Fresh sweep over all eleven governed fixtures from `schema_contracts.schema_fixtures()`, every JSON path × 39 corrupt values including deletion and unknown-key injection — **17,682 cases: 0 non-`GovernanceError` escapes, 0 schema-engine escapes, 0 unsafe schema-vs-validator divergences.** Every defect class from passes 3–7 stays closed: enum strictness (`common.require_enum`), `expires_at` parse + timezone (`artifact_index.py:54-59`), plan identity (`require_plan_id`), `sysroot_schema_version == 1` on both surfaces, sign-off stable-version class (`release_plan.py:334`), artifact-id shape (`ARTIFACT_ID_RE`).

**Suites/self-tests on this exact state:** governance self-tests 14/14, `sifr_verify` self-tests 11/11 (including documentation-step, report precondition, report production), `governance.schema_epoch` ok, file-size guardrail PASS (largest touched: `profile_runner.py` 874, `governance/selftest.py` 870), demo exit 0 with no network or repo mutation.

**All four Rust structural suites visibly selected in all four authoritative profiles** (`create-pr`/`merge`/`nightly`/`release` each list `matrix, tiers, compatibility-matrix, stale-drafts`), confirmed rather than re-added — `profiles.py`/`release.json` diffs are purely additive (`documentation` area + `evidence-custody`).

**Stable publication still gated:** `preview-release.yml:64-79` rejects stable three independent ways; `propose_preview_release` refuses to mutate an `active` index; the Rust reader rejects the stable channel key and stable pins. CAS is real — expected generation/digest captured at `:229-230`, live index re-fetched, re-validated and re-compared at `:306-317`, mutation at `:338`, all under the `sifr-release-index` concurrency group; `source_sha` resolved once in `validate` (`:84`) and consumed at `:105`/`:146`.

## The certification_0 / stable-candidate deferral is internally consistent

It is enforced by code in two independent places, not by prose:

1. `profiles.required_rust_interop_suites()` (`verification/runner/sifr_verify/profiles.py:203-215`) derives the required set **from the rust_interop manifest**. The manifest today has exactly the four suites, so the current selection is complete; the moment `certification_0` registers `stable-candidate`, every authoritative profile that omits it fails to load (`ProfileError`, `:184-190`). 40.1's registration cannot be silently skipped.
2. `release_report.REQUIRED_SUITES` (`verification/areas/distribution_release/governance/release_report.py:32-43`) still mandates `rust_interop:stable-candidate` in both `validate_profile` and `validate_steps`. No release-profile report can be produced or validated without it, and 40.1's planner requires a passing report for the same commit (plan `:541-547`) whose plan must reference the stable-candidate report (40.1 DoD). Stable activation without it is unreachable.

40.1 owns it explicitly: issue checklist item added, plan scope bullet at `:537-540`, and a Validation Contract clause starting at 40.1. The 40.0 DoD line was correspondingly narrowed. This does not weaken final qualification.

**Demo convention:** `demos/stable_release_governance_demo.sh` is capability-based; temp prefix `sifr-stable-release-governance`, label "Canonical, non-mutating stable release governance plan" — no phase or milestone token. The old `milestone_40_0_demo.sh` is deleted with no dangling references.

## Blocking findings (2, both one-line)

**1. `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:149-151` and `:174-176` still assign the stable-candidate registration to `milestone_40_0`.**
> `:149` "Phase 40 `milestone_40_0` is downstream of both `hardening_1` and this item … the stable-candidate claim check **that milestone registers**"
> `:174` "Confirm Phase 40 `milestone_40_0` **registers the stable-candidate suite in all four authoritative profiles**, `milestone_40_1` consumes its result…"

Both statements were correct before this PR and are false after it. This is an *active, unmerged* prerequisite issue: the second bullet is a `certification_0` exit-gate confirmation that can no longer be satisfied, and its implementer will expect registration to already exist. The PR that moved the ownership must carry the counterpart correction. **Required:** repoint both bullets to `milestone_40_1` as registrant-and-consumer. (The archived `rust-interop-verification-matrix-hardening.md:13` names 40.0 as the eventual registrant too, but it is archived and its literal claim — 40.0 may not register — remains true; leave it.)

**2. `verification/areas/distribution_release/cases/self_update_json_surface_parity.sh:20` — phase-named runtime path.**
```sh
build_root="${REPO_ROOT}/target/phase40-self-json-parity"
```
This is the only phase/milestone token in any non-plan file in the diff, and the file is new in this milestone. It violates the convention you just asked me to verify. **Required:** rename to a capability-based path, e.g. `target/self-update-json-parity`.

## Non-blocking

- The renamed demo appears nowhere in the plan or issue, and the issue's "Passing local evidence" list omits both the demo and the rust_interop four-suite command. The "Record positive/negative evidence, commands, review rounds, PR, and merge" checkbox is still unchecked, so this closes at PR time — but name the demo path there.
- `profiles._optional_arg` (`profiles.py:598-608`): a trailing valueless `--release-report-out` returns `None`, so the flag is silently ignored on a direct `python -m sifr_verify profiles run` invocation. `run_all_tests.sh:52-56` rejects it, so the documented entrypoint is safe.
- `plans/phases/index.md:50` still shows Phase 40 status `unspecified` against `in-progress`. Pre-existing on `origin/main`, unrelated to this scope change.
- The demo still sources its input from `governance.selftest.valid_plan()` (pass-1 note, never actioned). A checked-in fixture would be cleaner; not a defect.

## Later-milestone scope (correctly excluded, do not implement here)

`generate_dispatchers.sh:125-134` grep-based index parsing and stable dispatcher behavior → 40.2; residual `-rc.N` operator strings in `generate_version_installer.sh:71`, `build_preview_artifacts.sh:83`, `trigger_preview_release.sh:159`, `create_new_version.sh:187`, `generate_dispatchers.sh:88`, `docs/self_update.md:51` → 40.2/40.4 (each gated downstream; `ga-release` is `reserved`); version-asset `--clobber` at `preview-release.yml:284` → 40.2; `stable_support_claims.json` + stable-candidate suite → `certification_0` then 40.1.

## Verdict

**CHANGES REQUESTED** — two actionable `milestone_40_0` defects, both single-line: the `certification_0` issue still names `milestone_40_0` as the stable-candidate registrant (`:149-151`, `:174-176`), and `self_update_json_surface_parity.sh:20` carries a `phase40` build path against the capability-based naming convention.

Everything else in the milestone is sound. The schema-v2 cutover, fail-closed schemas/validators/generators, canonical release report, evidence custody, documentation area/profile integration, stable gate inventory, stable gating, and the four-suite selection are all satisfied and independently re-verified, and the certification_0 deferral is mechanically enforced rather than merely documented. With those two corrections this is approvable without a further deep pass.

Note: `plans/reviews/active/phase-40-milestone-40-0-claude-opus-review-pass-8.md` exists as a 0-byte placeholder; I did not write to it, per your instruction.
