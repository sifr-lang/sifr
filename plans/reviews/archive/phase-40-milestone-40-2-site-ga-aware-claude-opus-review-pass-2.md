## Verification

**Head match confirmed.** Local `HEAD`, `refs/heads/codex/ga-aware-release-site`, and `refs/pull/15/head` all resolve to `b33c8e731084f1f7638ba4227ee963a8309b70e2`. Working tree clean; no files modified by this review. Diff vs `721bcec` is 2 files, +40/−14.

Exact-diff YAML parse: 12 `workflow_dispatch` inputs, all `required: true`, `type: string`; step order `validate(0) → checkouts(1,2) → regenerate(3) → facts(4) → mutation boundary(5) → setup-node(6) → build(7) → dist digests(8) → GA binding(9) → deploy(10) → public verify(11) → summary(12)`. The sole public mutation (`wrangler versions upload/deploy`) still sits after the GA binding.

## Pass-1 findings — all closed

**Medium (README stale preview-only claim) — closed.** The contradicting sentences ("This preview-only workflow revision retains beta as that public default" and the "protected GA milestone must first land and pin a paired workflow revision that selects stable") are deleted, not merely appended past. `README.md:71-73` now states the live rule: caller supplies `beta` while preview, `stable` once active. No residual text asserts the workflow is preview-only.

**Low (input enumeration / undocumented GA binding) — closed.** `README.md:66-68` now enumerates 12 (2 commits + generation + index/plan/publication-facts digests + 4 dispatcher digests + GA-aware default + attempt), matching the parsed input set exactly. `:61-62` documents the central safety property — "requires the requested default channel to match the live index's GA state."

**Low (vacuous distinctness guard) — closed.** `release-site.yml:182-188` now compares `index` against `${install_root}/${DISPATCHER_DEFAULT_CHANNEL}`. Verified by running the generator in a temp dir for both modes: in stable mode `index` vs `stable` differ only by the two entrypoint-marker lines (guard is live, not vacuous); in beta mode `index` vs `beta` likewise. The interpolated path is safe — the value is allowlisted to `beta|stable` at `:107-110`, before its first use at `:157`.

## Re-assessed properties

**Digest binding of the default choice is real.** Generating both modes shows `stable`/`alpha`/`beta` are byte-identical across modes; only `index` changes (`93a40ff1…` beta-default vs `e824f984…` stable-default). So `dispatcher_index_sha256` alone fully binds the default channel, and the README's "the dispatcher digests bind the default choice" is accurate. A forged dispatch cannot flip the default without also supplying a matching index digest — and even then `:331-337` gates on the live `ga_status`, which is not caller-attested.

**Input handling.** `dispatcher_default_channel` reaches bash only via `env:` (`:80`), never `${{ }}` splicing. Empty/missing fails the `case` rather than silently inheriting the generator's own `stable` default (`generate_dispatchers.sh:27`), so the workflow-level allowlist is load-bearing and correctly placed. `live_ga_status` is the `case` subject, not a pattern — no glob expansion; a missing `ga_status` key aborts via `set -e` on the command substitution.

**Replay / race.** `ga_status` is monotonic — `release_index.py:121-122` rejects `active → preview`. Combined with the unchanged generation+digest equality check, the only possible drift between `:331` and deploy is `preview → active`, i.e. a beta default deployed against an active index. That is a benign degradation, serialized in practice by the `sifr-site-release` concurrency group, and the generated dispatcher's own `validate_release_index` (`generate_dispatchers.sh:160-163`) independently rejects stable resolution against non-active metadata at install time.

**Preview→GA handling.** Caller mapping `preview→beta / active→stable` (`release-publication.yml:250-257`) threads identically into facts (`:428`) and dispatch (`:481`), and `generate_site_publication_facts.py:36-39` enforces the same `("beta","stable")` choice set — the contract matches on both sides. The committed `public/install/index` remains `DEFAULT_CHANNEL="beta"`, which is the correct safe default for non-release builds; the release run regenerates it in place and the mutation boundary (`:225`) permits exactly that.

## Remaining items

No blocking or actionable findings in this PR.

One unchanged cross-repo sequencing item, not a defect here (this PR must merge first): the Sifr side still pins the pre-PR bytes — `release-publication.yml:54` has `SITE_WORKFLOW_SHA256: 6a04809d…` while this head's `release-site.yml` hashes to `7a27abaf9d7e67298ea3033abf19f1c504c68bf50bdcd4e3cc5577330456a958`, and `site_release_workflow_contract.sh` still asserts `workflow_commit: 721bcec…`, the old digest, and an order-sensitive `required_inputs` list omitting `dispatcher_default_channel` (this PR places it between `dispatcher_beta_sha256` and `publication_facts_sha256`). Until those are updated the guard at `:107-108` fails closed — correct direction, but it gates the milestone.

Checks run: `git ls-remote` head comparison, Ruby YAML parse of the exact head, `git diff --check`, workflow byte digest, and `generate_dispatchers.sh` executed for both `--default-channel beta` and `--default-channel stable` in a temp directory with byte/digest comparison. No files modified.

APPROVED
