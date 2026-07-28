## Review — Phase 40 / M40.5 stable publication production wiring

I read every changed and untracked file, plus the surrounding pre-existing workflow steps, `generation.py`, `release-qualification.yml`, and the `editor_integrations` submodule, to check the claims. The governance logic is, on the whole, well-constructed: ordering (release → Marketplace → generation snapshot → stale-lease check → `channels.json` clobber → site dispatch → poll → smoke → sign-off) matches the required sequence and is pinned by the new contract test; `--clobber` appears exactly once and only on `channels.json`; `publish_stable_release.py` has no `--clobber` at all, paginates assets properly, and downloads by immutable asset ID with `text=False` for byte comparison; the `pending`/`activated` split correctly makes post-activation resume verify rather than re-mutate; the top-level `contents: read` demotion is safe because the `publish` job re-grants `contents: write` and the `drill` job was already `contents: read`; profile/manifest/runner/coverage-matrix registration is consistent and deduplicated; preview/bootstrap/drill steps are gated off by explicit `governance_mode` guards rather than restructured.

Findings below are ordered by severity.

### 1. Blocker — the Marketplace publish path cannot execute in GitHub Actions

`scripts/distribution/publish_marketplace_extension.sh:84` runs `npx --no-install vsce publish --packagePath ...`. In `release-publication.yml` there is no `actions/setup-node` step and no `npm` install of any kind. `vsce` (`@vscode/vsce`) exists only as a **devDependency of `editor_integrations/vscode`**, and `editor_integrations` is a **git submodule**: the root checkout (`actions/checkout` with no `submodules:`) leaves that directory empty, and the `stable-source` checkout (`submodules: recursive`) provides sources but no `node_modules`. `npx --no-install` resolves only from local `node_modules/.bin` or an already-installed global binary; neither exists on `ubuntu-24.04`. `run_stable_publication.sh` also never `cd`s into the extension directory.

The comparison point is in-repo: `release-qualification.yml:161,182` explicitly does `actions/setup-node@v4` + `npm ci --prefix editor_integrations/vscode` before touching `vsce`. The publication workflow does neither.

Consequence: the very first `ga-activation` / `initial` run — where the Marketplace version is absent — aborts at the Marketplace step with `npx: command not found: vsce`. This is fail-closed (it happens before any index mutation, so stable cannot activate on a failed Marketplace step — the security ordering requirement holds), but the wave's primary path cannot complete. Fix: add `actions/setup-node` + `npm ci --prefix <vscode dir>` in the stable branch of the publish job and invoke the resolved `vsce` binary, or vendor a pinned `vsce` install step.

### 2. High — mutation-path code is unpinned, and prepare/publish revalidate with two different code copies

`release-publication-prepare.yml` deliberately runs governance code from the approved source: `(cd stable-source && python3 scripts/distribution/release_governance.py prepare-stable-publication ...)`. The new publish step does the opposite — `run_stable_publication.sh` is invoked from the workspace root (the *dispatch ref* checkout), and internally resolves `repo_root` to that root and calls `scripts/distribution/fetch_qualification_artifacts.py`, `revalidate_stable_publication.py`, `generate_dispatchers.sh`, `publish_stable_release.py`, and the Marketplace/smoke adapters all from the dispatch ref.

Two problems follow:

- **Approval scope.** Approving a candidate approves its evidence and its source commit, but not the code that performs every production mutation (release creation, Marketplace publish, `channels.json` clobber, governance uploads). That code is whatever is on the branch the operator dispatched from, gated only by the `stable-release` environment's deployment-branch policy — which is not asserted anywhere in the workflow or the contract test.
- **Reproduction across versions.** `revalidate_stable_publication.py` must reproduce, byte-for-byte, a summary that was produced by the `stable-source` copy of `stable_prepare.py`. Any change to governance materialization on the protected branch after a candidate is qualified will make that comparison fail permanently for that candidate — a self-inflicted brick, not a drift signal. Note the preview path does not have this asymmetry: it checks out `inputs.source_commit` at the workspace root and runs pinned code.

Fix: run the publish adapters from `stable-source` as prepare does, or assert the two copies of the governance tree are byte-identical before proceeding.

### 3. Medium — sign-off is not reproducible, so resume cannot converge once it exists

`materialize_stable_signoff` embeds `run_id` (the current `GITHUB_RUN_ID`) in `attempts[0].run_id`, and `run_stable_publication.sh` then calls `upload_or_verify_governance "${work}/stable-release-signoff-${version}.json"`. That helper verifies byte-equality when the asset already exists. Because the sign-off bytes change with every attempt, any resume run reaching that step after the sign-off asset exists dies with `governance asset bytes drifted`.

Every other retained governance asset is content-stable across attempts by construction — the generation snapshot is the proposed index, and `stable-site-release-facts-generation-N.json` is derived only from the proposal — so this is specific to sign-off. The realistic trigger is a transient failure between asset creation and the client seeing success, or an operator re-running `resume` to confirm final state; the run then fails permanently with no path forward that doesn't violate write-once. Either exclude run-scoped fields from the retained bytes, or make the existing-asset case verify semantic equivalence (same version/plan/assets/generation/marketplace/site binding) rather than raw bytes.

### 4. Medium — test adequacy gaps around the two controls that matter most

- `test_marketplace_adapter` stubs `npx` onto `PATH`, which is precisely what conceals finding 1. `stable_publication_workflow_contract.sh:99` then pins the literal string `"npx --no-install vsce publish"`, so the contract test actively locks in the non-executable invocation.
- There is **no negative test for Marketplace raw-byte drift**: the case where the Gallery returns HTTP 200 for the exact publisher/extension/version but with bytes that do not match `marketplace.vsix_sha256`. That second `verify_marketplace_vsix.py` call is the sole control preventing reuse of a foreign Marketplace version, and it is the only reuse path exercised (the test copies the identical VSIX to the fake server). The GitHub-release adapter, by contrast, does have both a duplicate-`initial` rejection and a remote-byte-drift rejection — the Marketplace adapter should have parity.
- `run_stable_public_smoke.sh` and `run_stable_publication.sh` have no execution coverage at all; the only assurance is literal-substring matching in the contract test.

### 5. Low — documentation asserts a capability the wiring does not yet have

`internal_docs/distribution_pipeline.md` now states that the absent version "is published once with `vsce publish --packagePath` and re-downloaded", and `plans/issues/active/phase-40-stable-channel-ga-execution.md` records the wave as validated and production-wired. Given finding 1, the Marketplace-publish branch is documented as operational but cannot run; neither document mentions the Node/`vsce` provisioning the step depends on. Once finding 1 is fixed, the docs should name that provisioning explicitly, since it is the only external toolchain dependency in the mutation path.

### Non-blocking notes

- `upload_or_verify_governance` uses `gh release view channels --json assets --jq '.assets[].name'` while `fetch_governance` correctly uses `gh api --paginate --slurp`. If the `channels` release ever exceeds the asset page the former returns, an existing asset would be treated as absent and the subsequent `gh release upload` would 422 — fail-closed, but a spurious failure. This mirrors the pre-existing preview step, so it is parity rather than a regression; worth unifying on the paginated form.
- Marketplace re-signing: Microsoft may repackage/sign uploaded VSIXs, in which case the raw `Microsoft.VisualStudio.Services.VSIXPackage` bytes will not equal the uploaded bytes and the post-publish verification loop will never converge. The exact-bytes rule is what the wave specifies, so this is not a deviation — but it is a real first-publish risk worth validating against the live Gallery before GA.
- `mktemp -d` scratch dirs in `run_stable_publication.sh` (`live_after_snapshot`, `activated`, and the per-asset verify dirs) are never cleaned; harmless on an ephemeral runner.
- `plans/reviews/active/phase-40-milestone-40-5-stable-publication-wiring-review-pass-1.md` is an empty (0-line) file. I did not modify it, per your constraint.

Findings 1–5 are actionable for this wave; finding 1 blocks the path the wave exists to enable, and finding 2 undermines the pinning guarantee the rest of the design is built around. To be explicit about the safety question you raised: I found **no** failure mode that activates stable before exact release and Marketplace state — ordering, the post-snapshot stale-lease `cmp`, the post-activation digest check, and `set -euo pipefail` throughout all hold.

NOT SATISFIED
