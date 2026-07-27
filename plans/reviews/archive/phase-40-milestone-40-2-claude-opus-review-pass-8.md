## Review: PR #3030 — Phase 40 / milestone 40.2 (head `28fe8527f`)

**Identity verified.** Local `HEAD` = `28fe8527fcf9b7586c69992684d0b26f4e38a10b`; `gh pr view 3030` reports `headRefOid` identical, base `main`, state OPEN. Reviewed the complete 85-file diff from merge-base `56f8c41ee`, not just the tip commits. Working tree is clean except my own untracked review placeholder (`plans/reviews/active/…pass-8.md`, 0 bytes) — not part of the PR.

---

## Pass-7 findings — independently verified closed

**1 (HIGH, preview/active site planner) — CLOSED, reproduced end-to-end.**
`scripts/distribution/create_new_version.sh:124-154` now derives `site_default_channel` from the index `ga_status` and only requires a `stable` dispatcher when `ga_status == active`; the `stable` file is validated *if present* (`:148-154`). I cloned the real pinned site commit `07d88cc3c2…` (`apps/sifr-site/public/install/` contains exactly `alpha`, `beta`, `index`, with `index` at `DEFAULT_CHANNEL="beta"`) and ran the planner against it: exit 0, `ga_status=preview`, `site_default_channel=beta`, site checkout left clean. The test blindness is also fixed — `cases/common.sh:311-326` now generates the fixture site repo with `--default-channel beta` and deletes `stable`, and `cases/create_new_version_active_site_dispatchers.sh` covers the active branch.

**2 (MEDIUM, validator hardcoded stable index default) — CLOSED.**
`verification/areas/distribution_release/tools/validate_self_update_metadata.sh:157-163` now selects `expected_index_channel` from `metadata_ga_status`. Both branches are exercised: `cases/channel_metadata_installer_agreement.sh:17` (preview, index→beta) and `cases/channel_metadata_stable_active.sh:16-21` (active, index→stable). The unconditional `stable`-file requirement at `:147` is correct here — this validator runs against the *deployed* tree, and the site workflow always emits all four entrypoints (`/tmp/release-site.yml:164`).

**3 (MEDIUM, unschematized v1 publication artifact) — CLOSED.**
`scripts/distribution/generate_site_publication_facts.py:80-81` emits `schema_version: 2` / `sifr-site-publication-binding-v2`; checked-in schema at `verification/areas/distribution_release/schemas/site_publication_facts.schema.json`; rejecting validator at `governance/site_publication.py:24-79`; registered as `--kind site-publication-facts` (`release_governance.py:66,187`) and used that way in `release-publication.yml:500-503`; schema-contract fixture added (`schema_contracts.py:64-79`); runner schema-count assertion bumped 11→12 (`verification/runner/sifr_verify/selftest.py:89`); listed in the phase contract's schema-v2 inventory (`plans/phases/40_…md:190,199-203`). Schema/validator parity confirmed field-by-field, including `additionalProperties:false` ↔ `require_exact_keys`.

**4 (LOW, all-zero digest accepted) — CLOSED.**
Rejected in the producer (`generate_site_publication_facts.py:32-33`), in shared governance (`governance/common.py` `require_sha256`/`require_commit`), and in the schema (`not:{const:"000…"}` on every digest). `cases/site_publication_facts_generated.sh:70-77,116-127` asserts rejection at both layers; the previously-pinned zero input is gone.

**5 (LOW, missing authoritative evidence) — CLOSED.**
`plans/issues/active/phase-40-stable-channel-ga-execution.md:377-384` records `scripts/run_all_tests.sh --profile create-pr` passing at implementation head `e29722dfe`, 131/131 E2E, `report_signature=7c39b8c1dd4fec7c`, with the warm-wall-time advisory routed to the indexed `PERF-HOST` follow-up. `28fe8527f` is documentation-only (1 file, +8 lines), so the gate head is the last functional head. The tag/ruleset rename is now covered by the pass-7 entry and this remediation record.

**6 (LOW, EOF/phase-index hygiene) — CLOSED.**
`git diff --check 56f8c41ee..HEAD` exits 0. `plans/phases/index.md:51` indexes `PERF-HOST`.

---

## Independent verification of the rest

**Live cross-repository identities (re-checked against GitHub, not just the fixture).** Tag `sifr-release-site-stable-distribution` → `07d88cc3c24707e386c5ad73fb0875c06ffd598f`. Ruleset `19791667`: `target=tag`, `enforcement=active`, `updated_at` 2026-07-27T05:06:21.354Z, `bypass_actors=[]`, rules `{update, deletion}`, include exactly `refs/tags/sifr-release-site-stable-distribution`, exclude `[]`. Site workflow bytes at that commit hash to `7a27abaf9d7e…958`, matching `SITE_WORKFLOW_SHA256` (`release-publication.yml:57`) and `fixtures/site_release_contract.json:17`. Ruleset + tag + bytes are all re-verified before mutation (`:100-165`) and again immediately before dispatch (`:525-556`).

**Mutation order and write-once.** Existing version release and existing tag both rejected pre-write (`:132-155`); `gh release create --target "${SOURCE_COMMIT}"`, no `--clobber` on version assets; published tag re-resolved to `SOURCE_COMMIT` (`:381-388`); every published asset byte-compared against local (`:399-410`); exact asset-name set enforced (`:196-218`). Lease revalidated (generation + digest) immediately before snapshot upload (`:430-439`); snapshot-name collision rejected (`:448-451`); activated bytes re-downloaded and digest-checked (`:461-471`). Ordering snapshot → replacement → dispatch is machine-asserted (`cases/preview_release_workflow_yaml_parses.sh:74-78`), and `--clobber` count == 1 (`:68-69`).

**Generation allocation.** Max over the current index and every validated `channels-generation-<N>.json` snapshot, with snapshot-name/payload agreement enforced (`:240-275`). `propose_preview_release` now takes an explicit `proposed_generation` that must exceed the current generation (`governance/release_index.py:151-157`), with positive and negative self-tests (`selftest.py:455-480`).

**Cross-repo binding is symmetric.** The site workflow independently re-runs `generate_site_publication_facts.py` from `.release/sifr` at the dispatched `SIFR_SOURCE_COMMIT` and compares the digest (`/tmp/release-site.yml:189-211`), regenerates all four dispatchers and verifies each digest (`:155-181`), and enforces a mutation boundary limited to the four dispatcher files (`:214-232`). Producer and verifier come from the same source commit, so the v1→v2 contract rename is coherent.

**Stable install/self-update integrity.** Dispatcher enforces the canonical single-line schema-v2 shape before reading any field, gates stable resolution on `ga_status: active`, requires `"status":"active"` in the installer-digest regex, rejects the zero installer digest, and verifies SHA-256 before `chmod +x` (`generate_dispatchers.sh:144-291`). In Rust, `resolve_exact()` runs before the force check so an exact stable pin under preview metadata fails with the GA message (`self_update_metadata.rs:313-330, 372`); user-supplied version text must exactly match a governed release key before it reaches `installer_url()`, closing URL injection; `validate_installer()` hashes before `make_executable()` (`self_update_runner.rs:46, 189-199`), with a test asserting the installer never ran on mismatch. No archive extraction or binary replacement in Rust; no data-dependent `unwrap`/`expect`.

**Scope boundaries.** No demo filename contains a phase/milestone number or phase name (`demos/stable_self_update_demo.sh`, `demos/preview_release_lifecycle/README.md` only). No Rust-interop implementation: the only `crates/` changes are the five self-update files plus a `sha2` dependency edge. No stable input reaches any publication workflow (`preview-release.yml:11-13`, `release-publication.yml:76-79`, asserted at `cases/preview_release_workflow_yaml_parses.sh:39-41,72-73`). `rc` removed from all remaining runtime/workflow surfaces with a repository-search guard (`cases/release_surfaces_reject_rc.sh:33-47`).

**Execution (all run here).** 49/49 self-update unit tests pass. 52/52 distribution_release variants pass, 0 failures, including all 14 governance self-tests and the schema-epoch check. `cargo clippy --workspace -- -D warnings` clean. `cargo fmt --check` clean. HIR maintainability guardrails PASS. No changed non-Markdown file exceeds 900 lines. `demos/stable_self_update_demo.sh` completes all four stages (forced beta→stable, stable→stable, no-op, preview-workflow stable-input assertion).

---

## Non-blocking notes (not findings, no action required for this merge)

- **Post-GA planner precondition.** When `ga_status` flips to `active`, `create_new_version.sh:132-143` will require the *checked-in* site repo to contain `stable` and an `index` defaulting to `stable`. The site workflow generates dispatchers into the working tree and never commits them, so GA activation must land that site-source change. This branch is unreachable today (`ga_status` is `preview`; stable publication arrives in `milestone_40_5`), and the required end state is exactly what the shared fixture already encodes — so this reads as intentional forward encoding, not a defect.
- **`cargo clippy --workspace --all-targets`** fails in `sifr_lowering` (lib test, `semicolon_if_nothing_returned`). This PR touches no lowering code, and `--all-targets` is not part of the AGENTS.md gate or `run_all_tests.sh`; the canonical `cargo clippy --workspace -- -D warnings` is clean.

---

## Verdict

**APPROVED** for merging milestone 40.2.

All six pass-7 findings are independently verified closed, two of them by direct reproduction against live external state (the real pinned site checkout and the live GitHub tag/ruleset/workflow bytes). I found no actionable correctness, governance, security, ordering, custody, parity, scope, or documentation defect in the complete diff, and every check I ran locally passes.
