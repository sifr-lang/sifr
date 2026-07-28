## Review — Phase 40 / milestone_40_5 incident publication production wiring

Scope reviewed: branch diff vs `origin/main` plus all untracked files (workflows, `scripts/distribution/*`, `verification/areas/distribution_release/**`, schemas, runner registration). No files modified.

The overall shape is right: one `publish` job with `contents: write` (asserted by `incident_publication_workflow_contract.sh:41`), read-only `prepare` with no protected environment, summary-digest binding, approver-distinct-from-initiator resolution, live-lease re-read immediately before the single `--clobber` of `channels.json` (`run_incident_publication.sh:398-412`, `--clobber` count asserted == 1), write-once governance assets, burned-generation skipping via `allocate_next_generation`, and correct `initial`/`resume`/`pending`/`activated` semantics. The findings below are what blocks it.

---

### Blockers

**1. `governance` cleanliness check makes every real rollback / incident-roll-forward publish fail before mutation**
`verification/areas/distribution_release/governance/incident_prepare.py:80` — `_require_clean_checkout(governance_root, "governance")`, reached via `scripts/distribution/run_incident_publication.sh:265` which passes `--governance-root "${repo_root}"`, and `repo_root` (`run_incident_publication.sh:124`) resolves to `$GITHUB_WORKSPACE`.

In the `publish` job the workspace root is itself a checkout (`.github/workflows/release-publication.yml:147`), and later steps create siblings at that root: `incident-evidence/` (line 167), `protected-prepare/` (line 180), plus `stable-source/`(152) and `stable-evidence/`(160) for roll-forward. `git status --porcelain --untracked-files=all` in the root repo reports embedded repos and plain dirs as untracked (verified empirically), so `materialize_incident_prepare` raises “governance checkout must be clean” on the first `revalidate` call (`run_incident_publication.sh:294`) for every rollback and incident-roll-forward run.

Fail-closed, so not a security hole — but the production path can never succeed. The `prepare` job is unaffected because it uses a nested `governance-source` checkout (`release-publication-prepare.yml:282`), which is why the discrepancy is easy to miss. No test covers it: `incident_publication_selftest.py:63-78` builds a pristine temp repo as `governance_root`.

Fix: give `publish` a nested `path: governance-source` checkout of `github.sha` and pass that, or scope the cleanliness check to the tracked paths it actually reads (`plans/releases/candidates/**`).

**2. The primary incident path — `incident-roll-forward` — has no end-to-end test**
`verification/areas/distribution_release/governance/incident_publication_selftest.py:35-38` registers only `test_protected_rollback_prepare_publish_and_resume`, `test_retained_release_adapter`, and a static workflow-text test. Consequently these are entirely unexercised:
- `incident_publish.py:145-158` (roll-forward branch, release-sign-off cross-binding)
- `incident_prepare.py:455-474` (`_mutation_evidence_from_stable`)
- `incident_prepare.py:349-361` (`release_prepare` ↔ incident summary cross-binding)
- the `release_prepare: {$ref: stable_publication_prepare.schema.json}` schema branch
- `materialize_incident_publication.py --release-signoff`

Per milestone_40_5 the first-GA playbook “uses `incident-roll-forward` and remains roll-forward-only until a later normal stable release establishes an eligible rollback target” — i.e. this is the *only* usable incident operation at GA, and it is the untested one. Missing negatives too: rollback supplying `--release-signoff` (`incident_publish.py:160-161`), site-run mismatch (`_site_run`, :255), smoke-set drift (`_smoke_evidence`, :276), wrong `site_plan` digest (`:72`), wrong dispatcher digests (`:250`).

**3. Drill-harness isolation assertions deleted without replacement**
`incident_recovery_selftest.py` drops `test_no_production_adapter_surface`, which asserted `incident_fixture.py` imports no `socket`/`urllib`/`requests`/`subprocess` and that `scripts/distribution/run_incident_fixture.py` contains no `gh release`, `vsce publish`, or `repository_dispatch`. The replacement `incident_publication_selftest.py:227-251` keeps only the workflow/drill-YAML assertions and drops both harness assertions. A repo-wide grep confirms nothing else asserts them. milestone_40_5 requires “no drill calls `gh release`, real `vsce publish`, or repository dispatch” and “blocks external network access”; `test_concurrency_and_credential_boundaries` covers only the runtime `GH_TOKEN` refusal, not the static surface.

---

### Major

**4. `Bind prepare outputs` silently produces empty outputs for `rollback`**
`.github/workflows/release-publication-prepare.yml:573,574,578` index `.release_prepare` — which is the string `"none"` for rollback (`incident_prepare.py:164`, schema `release_prepare: {const: "none"}`). jq 1.7.1 raises `Cannot index string with string "release_report"` and exits 5 (verified). Because the failure is inside `$( )` in an `echo`, `set -euo pipefail` does not trip, so `release_report_sha256`, `qualification_sha256`, and `source_commit` are set to `""` and the reusable-workflow outputs of those names are empty for rollback, with jq errors buried in the log. Rollback doesn’t consume them today, so it is latent — but it is a silent-empty in exactly the reviewer-facing digests. Use `(.release_prepare | objects).release_report.sha256 // ""`.

**5. File-size cap satisfied by cosmetic squeezing instead of decomposition**
`verification/areas/distribution_release/governance/stable_publish_selftest.py` is exactly 900 lines, and this change removes three blank lines from the *embedded generated fake-`gh` script* solely to fit. `.github/workflows/release-publication.yml` is at 899 lines, with the single `publish` job repeating the identical four-way mode-exclusion `if:` expression at lines 142, 175, 197, 246, 335, 344, 440, 557, 640, 691, 710, 745. AGENTS.md: “If a touched file exceeds the cap, refactor it by responsibility rather than adding more code to an oversized module.” Extract the preview/bootstrap steps into a reusable workflow (preserving exactly one `contents: write` job) rather than shaving whitespace.

---

### Medium

**6. milestone_40_5 demo requirement unmet.** The milestone requires the capability-named governance demo to record “a real GA dry run, protected approval evidence, the public stable install/update flow, VS Code Marketplace installation, the non-production rollback drill, and `workflow_dispatch operation=incident-roll-forward`.” `demos/stable_release_governance_demo.sh` (37 lines) still only renders and validates a plan fixture; `demos/stable_incident_recovery_demo.sh` covers only the M40.3 fixture harness. Neither references the protected workflow, approval evidence, or `incident-roll-forward` dispatch. (Filenames are correctly capability-based — no phase/milestone identifiers.)

**7. Stray empty artifact.** `plans/reviews/active/phase-40-milestone-40-5-incident-publication-wiring-review-pass-1.md` is 0 bytes. Populate or remove before the PR.

**8. Rollback deploys the *withdrawn* release’s site base commit, undocumented and unverified.** `incident_prepare.py:160-163` takes `site.base_commit` from the affected plan, and `run_incident_publication.sh:173` uses the affected plan as `site_plan`. This is defensible — it is the newest plan, so its `site.dispatcher_sha256` still matches the current generator (`incident_publish.py:250`), which an older rollback-target plan would not — but the rationale is stated nowhere, and nothing verifies that the site content at that commit cannot advertise the version being withdrawn. `run_stable_public_smoke.sh` checks only `/install` and `/install/stable` bytes, not docs, while the exit gate requires “public docs name the current active stable version.” Add a comment plus a facts-driven check that the deployed generation renders from `stable-site-release-facts.json` (whose `stable_version` correctly comes from the index — `release_plan.py:414` — and whose `withdrawals` lists the affected version).

---

### Minor / non-blocking

**9.** `run_incident_publication.sh:371-378` hardcodes `sifr` / `sifr-vscode` in the gallery URL and the `--publisher/--extension` flags, while the roll-forward branch (lines 355-356) derives them from the plan. Derive from `affected_plan.vscode` so a publisher/extension change can’t be bypassed. (The `--compiler-version "${successor_version}"` check itself is correct and well-targeted — it is what proves “Marketplace metadata truthfully covers … any non-`none` governed rollback target”.)

**10.** `scripts/distribution/revalidate_incident_publication.py:106-111` constructs a throwaway `ArgumentParser` and reports governance failures via `parser.error`, so a digest/binding failure prints an empty usage block instead of the diagnostic. It also diverges from `revalidate_stable_publication.py`, which reuses its real parser. Prefer stderr + `return 2`, matching the sibling entrypoints.

**11.** `incident_publication_prepare.schema.json` inlines the mutation-evidence object rather than `$ref`-ing a shared file the way `stable_publication_prepare.schema.json` does for `stable_index_mutation_evidence.schema.json`, so it can drift from `_validate_mutation_evidence` (`incident_prepare.py:477`).

---

### Verified clean (no findings)

- Exactly one stable-mutating job; `contents: write` count asserted == 1; `sifr-release-index` concurrency retained; drills on a separate group/environment.
- Prepare rejects raw JSON / workstation paths (`INCIDENT_PATH_RE`, `release-publication-prepare.yml:313`), symlinks and out-of-root resolution (`incident_prepare.py:84-92`), and requires evidence commits merged into protected main (prepare :327 and :353; publish `run_incident_publication.sh:137-160`).
- `revalidate_incident_publication.py:72` requires byte-exact reproduction of the reviewer-visible summary; approver resolution is fail-closed (`schema_bootstrap.py:115-119`); `jq -er '.[0]'` on an empty list would also abort.
- Snapshot-before-replace ordering, post-snapshot live `cmp`, single `--clobber`, post-mutation activated-digest check, and `upload_or_verify_governance` write-once semantics (existing asset requires explicit `resume`) are all correct; ordering is locked by `incident_publication_workflow_contract.sh:62-96`.
- Credential boundaries: `SITE_TOKEN`/`VSCE_PAT` captured then `unset` (`run_incident_publication.sh:116`); all smoke/recovery/installer invocations scrub `GH_TOKEN`/`SITE_TOKEN`/`VSCE_PAT`; prepare workflow asserted free of `secrets.`, `contents: write`, `gh release upload`, `vsce publish`.
- `jq -cS`/`canonical_json_bytes` are byte-compatible (`ensure_ascii=False`) — the `.release_prepare` extraction at `run_incident_publication.sh:335` is sound.
- Cross-file `$ref` resolution is supported by `verification/json_schema_202012.py`; schema↔runtime looseness (`proposed_index: {"type":"object"}`) matches existing precedent, and the new schema-count bump to 17 is correct.
- Retained-release verification binds tag commit, target commitish, draft/prerelease flags, plan/qualification digests, and exact asset inventory (`verify_retained_stable_release.py:46-74`), with a drift negative in `incident_publication_selftest.py:209-224`.
- No Rust interop work; all changes stay within Phase 40.

VERDICT: CHANGES_REQUIRED
