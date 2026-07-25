# Review: Phase 40 — Stable Channel GA Promotion and Release Governance (Round 11)

**VERDICT: NOT SATISFIED**

Reviewed `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (1054 lines) against the ten artifacts in `plans/reviews/iterations/` and live evidence: `.github/workflows/` (`local-first-validation.yml`, `preview-release.yml`; `release-publication.yml` still absent, as `40:521-523` assumes), `.github/workflows/preview-release.yml:5-71` (dispatch `channel` choice is `alpha|beta`; validate rejects `^X.Y.Z$` and permits `-rc.N`), `preview-release.yml:74-122` (four-target `build-artifacts` matrix + `upload-artifact@v4`), `preview-release.yml:123-140` (`download-artifact` → `publish-release`), `scripts/distribution/` (12 entries; `build_preview_artifacts.sh`, `create_new_version.sh`), `scripts/run_all_tests.sh:31-52` (unknown flags already pass through to `profiles run … -- ARGS`), `verification/runner/sifr_verify/profile_runner.py:697-736` (overwriteable `target/validation_lane_reports/<profile>.latest.{log,time,json}`), `verification/owners.json` (`schema_version` + `owners`, no `documentation`), `verification/areas/` (20 dirs, no `documentation`), `verification/areas/rust_interop/manifest.json:16,29,42,55` (four suites, no `stable-candidate`), `verification/areas/rust_interop/data/` (no `stable_support_claims.json`), `plans/releases/` (absent). Entry state matches what `40:52-61`, `40:356-378`, and `40:521-523` assume.

## Round-10 resolution audit

| R10 item | Status | Evidence |
|---|---|---|
| **M1.** Release-profile report artifact did not exist, was unowned, and was destructible/unrevalidatable | **Resolved** | Canonical contract at `40:188-193` (report id, clean source commit + recursive submodules, profile-manifest digest, command/toolchain, overall verdict, every required step/suite result, result-artifact digests; externally SHA-256-bound, explicitly **no self-referential digest**). Owned by M40.0 at `40:348-355`: schema plus `--release-report-out <path>` requiring a clean source tree, a fresh work directory, failing if the output exists, with `<profile>.latest.*` declared never valid release evidence; `run_all_tests.sh` passthrough at `40:354-355`. Validator negatives `40:407-408`. Consumed `40:461-463`, DoD `40:472-475`, produced `40:768-770`, retained `40:657`. Revalidation is now *defined* (`40:864-868`) as schema + canonical-byte digest + source/submodule/profile agreement + `pass` + mandatory step/suite presence, explicitly not a CI rerun — which also keeps it satisfiable without transporting per-suite result artifacts. |
| **M2.** Custody/transport of the approved plan and incident request was undefined; approval object unspecified | **Resolved** | Evidence-only PRs at `plans/releases/candidates/<version>/` (`40:206-213`) and `plans/releases/incidents/<incident-id>/stable-incident-request.json` (`40:195-197`); dispatch accepts **only** evidence commit SHA + repo-relative path + expected digest, never raw JSON or a workstation path (`40:197-198`, `40:835-836`). Read-only `prepare` with no protected environment emits the reviewer-visible summary and passes digests as outputs (`40:833-840`); `publish` depends on it, approval follows inspection, and the exact evidence commit and transported artifacts are re-fetched and re-hashed before mutation (`40:842-846`). Negatives at `40:940-943`; DoD `40:906-907`. Circularity is closed explicitly: “That evidence commit is not the release source commit” (`40:210`), and M40.0 checks forbid mixing compiler source with release evidence (`40:356-359`, `40:626-627`). |
| R10 polish 2 (drill environment) | **Resolved** | `40:847-848` names a distinct credential-free `stable-release-drill` environment. |
| R10 polish 3 (job topology) | **Resolved** | `40:830-832` routes every stable-changing operation through one `publish` job; `40:859-860` scopes write permissions to that job. |
| R10 polish 4 (`40:597` conflict) | **Resolved** | `40:638-639` now models dispatch/deadline/recheck/cancel/resume “through the non-deploying site fixture”. |
| R10 polish 5 (`fail-closed` inputs) | **Resolved** | `40:522-523` now reads “no rollback or stable-changing input exists until `milestone_40_5`”; the contradicting phrasing is gone. |
| R10 polish 6 (`documentation` area registration) | **Resolved** | `40:362-363` now says create `verification/areas/documentation/manifest.json` and register its manifest owner in `owners.json`. |
| R10 polish 1 (cross-doc drift) | **Unaddressed** (carried since R3) | Polish 1 below. |

All 36 prior material findings (rounds 1-9) plus both round-10 findings are closed against live evidence. The prompt's specific probes check out: no circular commit (`40:210`), no approval-before-bytes inversion (`40:842-846`), no raw-byte dispatch (`40:197-198`), non-overwriting custody throughout (`40:260-264`, `40:349-351`, `40:655-666`, `40:897-900`), and no workstation credential path (`40:540-541`, `40:835-836`, `40:1012-1015`). One finding is new, and it is exactly the "untransported artifacts" class.

---

## Material findings

### 1. No milestone owns a workflow that produces the four-target stable candidate artifacts whose run/artifact ids, digests, and expiry the plan binds and `prepare`/`publish` download

The transport chain now terminates in a producer that does not exist and is assigned to nobody:

- The plan must bind “qualification workflow run/artifact identifiers, digests, and expiry for every candidate binary, installer, sysroot, VSIX, and other transported artifact” (`40:226-228`).
- M40.4 must “Record the exact qualification workflow run/artifact ids, digests, and expiry in the plan. Artifacts are uploaded with immutable names and overwrite disabled; expiry before publication invalidates the candidate and requires requalification” (`40:775-778`).
- `prepare` “downloads the recorded unexpired qualification artifacts, verifies every digest/source identity” (`40:838-839`); `publish` “re-fetches the exact evidence commit and transported artifacts and recomputes all digests” (`40:843-844`) and in `initial` mode “publish[es] the write-once version release and assets first” (`40:874`), i.e. it uploads bytes it did not build.
- DoD `40:908` (“The published assets are byte-identical to the qualified assets”) and negative `40:942` (“expired/missing qualification artifacts”) both depend on that upload existing.

Repository and plan evidence show the only existing multi-target builder is closed to stable, and the phase keeps it closed:

- `.github/workflows/preview-release.yml:79-122` is the sole four-target build + `upload-artifact` path, but its `validate` job hard-rejects stable versions — `preview-release.yml:56-59`: `if [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] … "stable-looking versions are disabled"` — and its dispatch `channel` input offers only `alpha|beta` (`preview-release.yml:10-13`).
- `40:509-511` preserves exactly that: “The preview workflow accepts only alpha and beta; stable is accepted only by the protected stable path introduced in `milestone_40_5`.”
- `release-publication.yml` is defined as the mutation workflow only (`40:520-523`, `40:290-297`), and M40.5 requires it to consume recorded artifacts without rebuilding (`40:785`, `40:878-880`).
- M40.1's builder scope is `40:443-445` (“Extend artifact generation to stable SemVer without adding an alternate builder … Build on each supported target … Execute the produced `sifr` and installed sysroot on the matching host”), but it names no workflow, no artifact-upload contract, and no run/artifact-id emission — while its demo is a single local host build (`40:489-492`). Its positive validation “All four target artifacts install and run on matching hosts” (`40:481`) is unreachable from one workstation.

Consequences an implementer hits at M40.1/M40.4:

- **Unowned producer.** Every other workflow in the phase is named by path and assigned a creating milestone (`40:520-523` for `release-publication.yml`, `40:531-537` for the site workflow, after R2-3 and R4-1 forced exactly that). The stable qualification build is the last workflow-shaped surface with no file, no milestone, and no permission scope.
- **Untransported artifacts.** `40:838-839` and `40:843-844` cannot download artifacts no run produced; `40:874`'s `initial`-mode asset publication has no byte source. This is the residual instance of the transport gap round 10 closed for the plan and report.
- **Immutable-name/overwrite-disabled and expiry rules have no host.** `40:776-778` states properties of an upload step that no milestone adds, so the M40.4 DoD “only unexpired exact-digest candidate artifacts” (`40:800-801`) and the negative `40:942` have no configuration to verify.

**Required:** assign to M40.1 (with M40.4 recording the ids) an explicitly named stable-candidate qualification workflow — either a stable-candidate build mode added to `preview-release.yml`'s `build-artifacts` matrix that is build/upload-only and cannot publish or mutate metadata, or a new `.github/workflows/release-qualification.yml` with read-only repository permissions — stating its four-target matrix, immutable artifact names, overwrite-disabled upload, retention/expiry window relative to the candidate, and the run/artifact identifiers it emits for the plan. Add the matching M40.1 DoD (“the plan's recorded artifact ids resolve to unexpired artifacts whose digests equal the plan's”) and negative case (“a qualification run at a different source commit, or an expired artifact, fails planning”).

---

## Optional polish (non-blocking)

1. **Cross-doc drift — open continuously since round 3, both issue files modified in the working tree.** `plans/issues/active/rust-interop-verification-matrix-hardening.md:12` and `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:137,158-159` still name `milestone_40_1` / “the `milestone_40_4` activation gate” as the stable-candidate consumer, while `40:56-61` requires those artifacts *before* `milestone_40_0` and `40:360-361` registers the suite there. `plans/phases/index.md:50` still lists Phase 40 status `unspecified` against `40:3` `implementation-ready`.
2. **M40.0 DoD has no falsifiable item for its new evidence-directory checks.** The checks are scoped at `40:356-359`, but `40:390-417` never asserts their behavior; the only falsifiable statements are downstream at `40:626-627` and `40:800-801`. Add “an evidence commit that mixes compiler source with release evidence, names two candidates, or drifts from its recorded digest fails”.
3. **`40:844-845` (“before receiving write permissions”) is not expressible at step granularity on GitHub Actions** — job `permissions` are fixed for the job's lifetime, and the environment gate already precedes job start. Reword to “before any mutation step”; `40:859-860` already captures the job-level grant correctly.
4. **Release work directory location vs the clean-tree requirement.** `40:206-208` and `40:349-351` require a clean source tree and a fresh release work directory without saying the directory sits outside the repository working tree (or is ignored). State it, or an implementer's first `--release-report-out ./release-work/...` run fails its own precondition.
5. **`40:195-198` interrupts the “Schema ownership” bullet list with a prose paragraph, orphaning the `40:199-200` installer bullet.** Move the custody paragraph below the list.
6. **Registration home for the evidence-directory checks is unnamed** (`40:356-359`), unlike the `documentation` area (`40:362-363`) — name the area/suite so `40:1011` (“No CI-only semantic validation”) is verifiable for them.
7. **`40:408`'s “unknown profile digest” rejection has no stated source of truth** for the set of known profile-manifest digests.
8. **Cosmetic, carried since round 1.** `distribution_release --suite full` (`40:957`) implies a scope distinction that does not exist — the manifest binds `representative` and `full` to the same case.

## What remains strong

- `40:188-193` + `40:348-355` + `40:407-408` + `40:461-463` + `40:472-475` + `40:657` + `40:864-868`: the release-profile report is now a checked-in, non-self-referential, non-overwriting, retained artifact with a defined revalidation meaning that needs no untransported result artifacts and does not displace the authoritative local gate.
- `40:195-198` + `40:206-213` + `40:833-846` + `40:906-907` + `40:940-943`: custody is a git evidence commit, dispatch carries identifiers only, the approver sees a read-only summary of the exact digests, and post-approval re-fetch/re-hash is fail-closed — closing approval-before-bytes and evidence/source mixing simultaneously.
- `40:614-624` + `40:626-627` + `40:655-666` + `40:869-873` + `40:894-900`: incident authorization rests on an immutable, evidence-committed, write-once-published request, with atomic withdraw-plus-activate, no pruning, and paired sign-off.
- `40:628-639` + `40:706-709` + `40:847-856` + `40:938-939`: the non-production boundary is stated twice with the same four adapters, a named credential-free `stable-release-drill` environment, and falsifiable repository checks.
- `40:67-80` / `40:1009-1010`: across 1054 lines there is no compatibility shim, migration path, legacy reader, dual metadata, or fallback URL/installer/path; the pre-GA no-users constraint holds throughout.
