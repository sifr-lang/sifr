I read the archived pass-1 review, the full diff of all 27 modified files, all 12 untracked files, the surrounding pre-existing preview/bootstrap steps, `release-qualification.yml`, `evidence_custody.py`, and the `editor_integrations/vscode` submodule (`package-lock.json`, `@vscode/vsce@3.9.2`, `@vscode/vsce-sign`, `keytar`). I re-ran the focused suites rather than trusting the reported numbers.

## Pass-1 findings: all five genuinely closed

**1. Marketplace publisher executable — closed.** `release-publication.yml:174-186` adds SHA-pinned `actions/setup-node@49933ea5` (`node-version: 22`, `cache-dependency-path: stable-source/editor_integrations/vscode/package-lock.json`) then `npm ci --ignore-scripts --prefix stable-source/editor_integrations/vscode` with `GH_TOKEN: ""`. That is the *candidate's* submodule lockfile (`stable-source` is `submodules: recursive` at `needs.prepare.outputs.source_commit`), and it runs before `VSCE_PAT`/`SITE_TOKEN` appear — both are scoped only to the `Run exact protected stable publication` step. `VSCE_BIN` is the explicit pinned `.../node_modules/.bin/vsce`, and both `run_stable_publication.sh:96` and `publish_marketplace_extension.sh:46` refuse to proceed unless it is executable; `npx` is gone. I verified `--ignore-scripts` is safe here rather than assuming it: the two `hasInstallScript` packages are `@vscode/vsce-sign` (whose `src/main.js` resolves `bin/vsce-sign` lazily at `execFile` time, and which `vsce` touches only in `verifySignature`) and `keytar` (an *optional* dep, dynamically `await import`ed in `store.js` only via `openDefaultStore`, which `publish --pat`/`VSCE_PAT` bypasses).

**2. Unpinned mutation code / two governance copies — closed.** Prepare now runs `governance-source` checked out at `${{ github.sha }}` (`release-publication-prepare.yml:267-274`) instead of `stable-source`, so prepare and publish materialize from the same tree. Publish checks out the workspace root at the workflow commit and `run_stable_publication.sh:108-117` asserts `rev-parse HEAD == workflow_commit`, fetches `origin/main`, and asserts `origin/main == workflow_commit`, with `--workflow-ref` regex-pinned to `refs/heads/main` at line 77. The prepare/revalidate byte-reproduction asymmetry is gone.

**3. Sign-off write-once per attempt — closed.** The asset is now `stable-release-signoff-${version}-attempt-${run_id}-${run_attempt}.json` (`run_stable_publication.sh:385,392`), so each attempt writes a distinct immutable asset. `materialize_stable_signoff` has no clock or nondeterministic input, so the residual same-name `cmp` path is a true no-op rather than a brick. The generation-scoped site-facts asset stays byte-stable across attempts (derived only from `proposed_index`), including in the `activated` branch where `proposed_generation == live generation`.

**4. Test adequacy — closed.** `stable_publish_selftest.py` runs 8 tests, verified locally, all pass. Specifically: raw Gallery HTTP-200 byte drift is now negative-tested (lines 199-210: server bytes replaced with `b"foreign Marketplace bytes"`, non-zero exit, `--verified-out` not created); `run_stable_public_smoke.sh` executes end-to-end against a fake `curl` for `/releases/download/channels/channels.json`, `https://sifr.sh/install`, `/install/stable` and version assets, plus a real dispatcher install and `sifr self update --dry-run` (lines 316-402); the orchestrator preflight is executed (lines 405-465) with `--workflow-ref refs/heads/unprotected` as the sole invalid argument, asserting non-zero exit and that no work directory was created. The contract test no longer pins `npx --no-install`; it pins `"${VSCE_BIN}" publish --packagePath` and the verify-before-publish ordering.

**5. Docs — closed.** `internal_docs/distribution_pipeline.md` names the Node 22 + `npm ci --ignore-scripts` + pinned local `vsce` provisioning and the per-attempt sign-off asset; `plans/releases/README.md` states both.

**Pagination remediation — closed for the new code.** `fetch_governance` and `upload_or_verify_governance` both use `gh api --paginate --slurp .../assets?per_page=100`, and `publish_stable_release.py:159-185` pages the version release assets explicitly.

Registration is consistent (`manifest.json`, `runner.py` including the dedup `include_stable_publication` flag, all three profiles, coverage matrix, `REQUIRED_SUITES`, gate inventory). Verified locally: stable-publication 8/8, stable-prepare 7/7, stable-publish-primitives 4/4, both workflow contract cases pass, both workflows parse as YAML, the three new shell scripts pass `bash -n`, and `file-size guardrails: PASS (2923 files, limit 900)` with `release-publication.yml` at 899.

## Actionable findings

### 1. Medium-high — the stable path drops the protected-main reachability check the preview path enforces

`release-publication.yml:282-287` (`Validate protected publication inputs`) does:

```
git fetch --no-tags origin main:refs/remotes/origin/main
git merge-base --is-ancestor "${SOURCE_COMMIT}" refs/remotes/origin/main || exit 2
```

This step is now excluded for `ga-activation`/`normal` (`:237`), and nothing re-establishes it on the stable path. `run_stable_publication.sh:108-117` fetches `origin/main` but only compares it to the *workflow* commit; it never relates `origin/main` to the candidate's `source_commit` or to `evidence_commit`. `materialize_stable_prepare` only does `_require_checkout` (HEAD equals the expected SHA), `fetch_qualification_artifacts.py:63` only binds `run.head_sha == expected_source_commit`, and `release-qualification.yml` has no ancestry or ref constraint at all (grep for `is-ancestor|merge-base|origin/main` across `scripts/distribution`, `verification/areas/distribution_release`, and both publication workflows returns only `release-publication.yml:283`).

Failure scenario: an operator dispatches `ga-activation` from `refs/heads/main` (so the workflow-commit/main-HEAD assertions pass) with `evidence_commit` pointing at an unmerged branch or PR-head commit whose `plans/releases/candidates/X.Y.Z` holds a self-consistent plan plus a qualification index from a `release-qualification` run dispatched against that same unmerged commit. Every machine control in the wave passes — plan digest, artifact digests, `stable-source` checkout identity, revalidation byte-equality — and stable GA activates from source that was never merged to protected `main`. The only thing standing in the way is a human approver reading the source commit out of the prepare summary. The lower-stakes alpha/beta path machine-enforces exactly this; the production stable path does not.

Fix: assert `git merge-base --is-ancestor` for both the plan's `source_commit` and `evidence_commit` against the already-fetched `refs/remotes/origin/main` in `run_stable_publication.sh` (right after the existing `origin/main` fetch at line 112), and add an execution test alongside `test_orchestrator_rejects_unprotected_ref`.

### 2. Low — documented line count is wrong

`plans/issues/active/phase-40-stable-channel-ga-execution.md` records "File-size guardrails pass with the single publication workflow at 898 lines." `release-publication.yml` is 899 lines. Small, but it is a validation record in the tracking doc for this wave.

## Non-blocking notes

- The **preview/bootstrap** governance-asset inventory is still unpaginated (`release-publication.yml:419,671` use `gh release view channels --json assets`, which caps at 100 assets). Pass-1 called this parity; it is now slightly worse in practice because this wave adds two new asset families (`stable-site-release-facts-generation-N.json`, `stable-release-signoff-*-attempt-*.json`) to the same `channels` release, and the per-attempt sign-off naming means the count grows per attempt, not per release. Fail-closed (spurious 422), but worth unifying on the paginated form.
- `revalidate_stable_publication.py:70` assigns `proposed_generation` and never uses it after the switch to `summary["next_generation"]`. Dead local; no Python lint gate exists, so it does not fail anything.
- `test_orchestrator_rejects_unprotected_ref` invokes the real orchestrator with `cwd=REPO_ROOT`. It is safe today only because argument validation rejects before `mkdir`; if that ordering ever changes, the test would create `stable-publication-work` in the live checkout and start issuing `gh` calls.
- Marketplace server-side re-signing (pass-1 note) remains an open first-publish risk by design, not a deviation: if Microsoft repackages the uploaded VSIX, the raw `Microsoft.VisualStudio.Services.VSIXPackage` bytes will never equal `marketplace.vsix_sha256` and the convergence loop at `publish_marketplace_extension.sh:89-104` will exhaust. Worth one live-Gallery validation before GA.
- `plans/reviews/active/phase-40-milestone-40-5-stable-publication-wiring-review-pass-2.md` is a 0-byte file. I did not write to it, per your no-modification constraint.

On the safety question: I again found no path that activates stable before exact release and Marketplace state. Ordering is pinned by the contract test, `--clobber` appears exactly once and only on `channels.json`, the post-snapshot `cmp` stale-lease check and post-activation digest check both hold, `publication_state == "activated"` correctly verifies instead of re-mutating, and `set -euo pipefail` is in place throughout.

NOT SATISFIED
