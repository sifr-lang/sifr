## Review: PR #3030 — Phase 40 / milestone 40.2 (head `2c282f1c7`)

Verified head matches the expected SHA; reviewed the full 74-file diff against merge-base `56f8c41ee`, not just the tip commits.

**NOT APPROVED** — 6 actionable findings.

---

### 1. HIGH — The read-only preview planner cannot run against the canonical site repository, and its dispatcher preflight contradicts this PR's own GA-aware default binding

`scripts/distribution/create_new_version.sh:111-113` requires a `stable` dispatcher file in the site checkout, and `:118-119` requires `index` to have `DEFAULT_CHANNEL="stable"` — unconditionally.

But this PR itself establishes that during `ga_status: preview` the governed default is **beta**:
- `.github/workflows/release-publication.yml:305-309` → `preview) site_default_channel="beta"`
- `verification/areas/distribution_release/fixtures/site_release_contract.json:23-26` → `"default_channel_by_ga_status": {"active": "stable", "preview": "beta"}`
- `internal_docs/distribution_pipeline.md` → *"Before GA activation, the paired preview deployment explicitly retains beta as the live `index` default."*

And the exact pinned canonical site base (`.github/workflows/preview-release.yml:55` → `07d88cc3c2…`) contains only `alpha`, `beta`, `index`, with `index` at `DEFAULT_CHANNEL="beta"` (confirmed via the GitHub contents API). The site workflow regenerates dispatchers into the working tree but never commits them back, so this state is durable.

Reproduced against a checkout of the real pinned site commit:
```
create-new-version: site dispatcher missing: …/apps/sifr-site/public/install/stable
# after adding a stable file:
create-new-version: site dispatcher drift: index must default to stable
```
So the only supported local planning command fails closed for every preview release in the current (pre-GA) epoch. The tests miss this because `verification/areas/distribution_release/cases/common.sh:308-323` synthesizes an install root generated with the default `--default-channel stable`, so no case ever exercises a preview-era site checkout.

### 2. MEDIUM — `validate_self_update_metadata.sh` is GA-aware for metadata but hardcodes the stable index default

`verification/areas/distribution_release/tools/validate_self_update_metadata.sh:157` asserts `DEFAULT_CHANNEL="stable"` on `index` unconditionally, even though the same file now computes `metadata_ga_status` (`:129-145`) and branches on `preview` vs `active` everywhere else. Run against a preview index plus the paired deployed dispatchers, it fails with `index dispatcher must default to stable`. Same root cause as #1: a GA-status-dependent property asserted as a constant.

### 3. MEDIUM — New governed publication artifact sits outside the `schema_version: 2` governance epoch

`scripts/distribution/generate_site_publication_facts.py:68-86` emits `{"contract": "sifr-site-publication-binding-v1", …}` with **no `schema_version: 2`**, **no checked-in JSON Schema** under `verification/areas/distribution_release/schemas/`, and **no `release_governance.py validate --kind` entry**. Its digest is a gating dispatch input (`release-publication.yml:487-502, 565, 581`) and is re-verified by the site workflow, so it is governed release data.

The phase contract's "Single schema epoch and ownership" section requires every Phase 40 machine-readable contract to carry `schema_version: 2` with a checked-in schema and a rejecting validator, and explicitly bans a v1-named producer/consumer in the epoch. The pre-existing canonical `stable_site_release_facts.schema.json` + `generate_site_release_facts()` were not usable here (they require `ga_status == "active"`), which is a legitimate reason to add a second artifact — but not a reason to leave it unschematized and v1-named.

### 4. LOW — All-zero digest accepted in site publication facts, and the test pins that behavior

`generate_site_publication_facts.py:20-23` `require_sha256()` checks only `^[0-9a-f]{64}$`; the all-zero digest passes. `verification/areas/distribution_release/cases/site_publication_facts_generated.sh:26` and `:62` actively pass `--dispatcher-beta-sha256 0…0` (64 zeros) as an accepted input. Every other Phase 40 digest surface rejects it: `schemas/stable_site_release_facts.schema.json` (`"not": {"const": "000…"}` on each dispatcher digest), the dispatcher's zero-installer-digest guard (`generate_dispatchers.sh:268-272`), and `validate_hex()` in `crates/sifr/src/self_update_metadata.rs:575`. Fail-open inconsistency in a governed digest binding.

### 5. LOW — No authoritative create-PR validation evidence is recorded for milestone 40.2, and the last recorded review predates a substantive change

`plans/issues/active/phase-40-stable-channel-ga-execution.md:309-364` records six review passes but **no `scripts/run_all_tests.sh --profile create-pr` result**. The only "131/131 E2E / zero blocking failures" evidence in the tracker is at `:296-299`, and it belongs to **milestone 40.1 / PR #3028** at a different head — it is not evidence for this PR. AGENTS.md makes `--profile create-pr` the authoritative pre-PR gate, and the milestone checklist item is "Record review rounds, PR, validation, and merge."

Separately, the last recorded review (pass 6, `:361-364`) is at `f28b9d8fa`; commit `a0d7ba875` then renamed the dispatch tag `sifr-release-site-m40-2` → `sifr-release-site-stable-distribution` and switched ruleset `19790146` → `19791667` with no recorded re-review.

### 6. LOW — New plan file fails the whitespace check and is missing from the phase index

`plans/issues/active/adhoc_performance_budget_host_variance.md:53` has a trailing blank line at EOF; `git diff --check <base>..HEAD` exits 2. Review pass 5 flagged exactly this defect class in two case files and the tracker records it as closed, but this file still carries it. The file is also absent from `plans/phases/index.md`, which the file itself declares is "a maintained navigation index."

---

## What I verified as correct

- **Live identities.** Site ruleset `19791667` matches the pinned assertion byte-for-byte (`target: tag`, `enforcement: active`, `updated_at 2026-07-27T05:06:21.354Z`, include `refs/tags/sifr-release-site-stable-distribution`, empty exclude, no bypass actors, rules `[deletion, update]`). Tag resolves to `07d88cc3c2…`, and the site workflow bytes at that commit hash to `7a27abaf…`, matching `SITE_WORKFLOW_SHA256`. Ruleset + tag + workflow-bytes are re-verified both before mutation and immediately before dispatch (`release-publication.yml:100-165, 521-552`).
- **Write-once semantics.** Existing release and existing tag both rejected before any write (`:132-155`); `gh release create` with `--target "${SOURCE_COMMIT}"` and no `--clobber`; published tag re-resolved to `SOURCE_COMMIT`; every published asset byte-compared against local (`:396-410`); exact asset-name set enforced (`:196-218`). `--clobber` appears exactly once and only on `channels.json`, machine-checked at `cases/preview_release_workflow_yaml_parses.sh:73-74`.
- **Metadata governance.** Max-generation allocator across current index + all validated snapshots (`:240-270`); lease revalidated (generation + digest) immediately before the snapshot upload and index replacement (`:420-451`); snapshot name collision rejected; activated bytes re-downloaded and digest-verified (`:461-471`); ordering snapshot → replacement → dispatch enforced by test.
- **Dispatchers.** Canonical single-line schema-v2 shape enforced before any field read; stable resolution gated on `ga_status: active`; the installer-digest regex requires `"status":"active"`, so withdrawn and unlisted versions fail; zero installer digest rejected; installer SHA-256 verified before `chmod +x`/execution; metadata and release-base URLs are generator-owned constants with the former `SIFR_CHANNEL_METADATA_URL` / `SIFR_INSTALLER_RELEASE_BASE_URL` overrides removed; version pins pass an anchored regex before sed interpolation.
- **Self-update.** Metadata is now fetched for *all* request kinds (`self_update_cli.rs:89-95`); `resolve_exact()` runs before the force check, so an exact stable pin under preview metadata fails closed with the GA message rather than the force message; strict exact-key/hex/target validation on every release record; SHA-256 verified in `validate_installer()` before `make_executable()`/spawn (`self_update_runner.rs:43-45`); no archive extraction or binary replacement in Rust; no `unwrap`/`expect` on data-dependent paths.
- **`rc` removal** across installer `APP_CHANNEL` derivation, dispatcher parsing, artifact builder, trigger script, and workflow inputs, with a repository-search guard (`cases/release_surfaces_reject_rc.sh:33-47`).
- **Local read-only.** `--real-run`/`--mutation-mode`/`--artifact-dir`/`--binary`/`--sysroot-root`/`--work-dir` all rejected; plan prints only; fixture bytes and site HEAD proven unchanged.
- **Preview-after-GA.** Removing the `ga_status != preview` guard in `propose_preview_release()` is required and safe — `channel` is still enum-restricted to `{alpha, beta}` and `stable` is preserved through the spread.
- **Execution.** 49 self-update unit tests pass; all 49 distribution case scripts pass; `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --check` clean; HIR guardrails PASS; no non-generated first-party file over 900 lines; `demos/stable_self_update_demo.sh` runs end-to-end (forced beta→stable, stable→stable, no-op, preview-workflow stable-input assertion).
- **Scope.** No stable input reaches any publication workflow; no rollback/incident surface added; no Rust-interop implementation touched.
