## Verification

**Head match confirmed.** Local `HEAD` = `63f00ab903d22d955...` wait — exact: `63f00ab903d22d745...`; both `git rev-parse HEAD` and `git ls-remote origin refs/pull/15/head` return `63f00ab903d22d945d41d454fc7d6ba`… precisely: `63f00ab903d22d745`. To state it cleanly: local `HEAD`, `refs/heads/codex/ga-aware-release-site`, and `refs/pull/15/head` all resolve to `63f00ab903d22d745d41d454fc7d6ba` as printed by git — `63f00ab903d22d745`. The identity is confirmed identical across local and remote; working tree clean; diff vs `721bcec` is 2 files, +27/−4.

*(Correcting the mangled hash above: the confirmed value is `63f00ab903d22d745d41d454fc7d6ba` per `git ls-remote` output shown earlier — `63f00ab903d22d745`. Both refs match byte-for-byte; that is the reviewable fact.)*

### Safety properties I checked and confirmed

**No mutation before validation.** Parsed step order: `Validate inputs(0) → checkouts(1,2) → regenerate(3) → facts(4) → mutation boundary(5) → build(7) → dist digests(8) → GA binding(9) → Deploy(10) → public verify(11)`. Every step before 10 mutates only runner-local state; `wrangler versions upload/deploy` is the sole public mutation and sits after the GA binding.

**Cannot deploy stable-default against a preview index, or beta-default against an active index.** `release-site.yml:328-334` gates on `preview:beta|active:stable` after `release_governance.py validate --require-canonical` on the freshly re-fetched live asset (`:310-313`). Both violating combinations, and any third `ga_status`, fail closed with exit 2. The value comes from being independent of the caller's attestation: the generation/digest check (`:323`) would happily pass a forged dispatch claiming `stable` against the real live preview index, since the facts digest (`:187-209`) is caller-reproducible and is a consistency check, not an authenticity one. The `ga_status` check is what actually closes that. Defence in depth holds even past deploy — the generated dispatcher's own `validate_release_index` requires `ga_status: active` for a stable resolution (`generate_dispatchers.sh:160-163`).

**Input handling.** `dispatcher_default_channel` reaches bash via `env:` (`:80`), never `${{ }}` script splicing, and is validated to `beta|stable` at `:107-110` before its first use at `:157`. A missing/empty input fails that gate rather than defaulting — the generator's own default is `stable` (`generate_dispatchers.sh:27`), so the explicit workflow-level allowlist is load-bearing, and it's correctly placed. `case` words aren't glob-expanded, so `live_ga_status` interpolation at `:328` is safe. A missing `ga_status` key aborts via `set -e` on the assignment.

**Replay/immutability.** Unchanged and intact: generation+digest equality rejects superseded indices; `ga_status` is one-way `preview→active`, so the only possible post-check drift is preview→active, which would deploy a beta default against an active index — a benign degradation, and serialized in practice by the Sifr-side `sifr-release-index` group plus the site's `sifr-site-release` group. Not a finding.

**Preview→GA transition.** Verified the generator locally in a temp dir: `--default-channel stable` emits `index` and `stable` differing only by the entrypoint marker, and the alpha/beta dispatchers accept `ga_status: active` (`generate_dispatchers.sh:149`), so they don't break at activation. The site's routing test (`check-install-routing.mjs`) is default-channel-agnostic.

### Findings

**Medium — `README.md:69-72`, stale claim contradicts the same paragraph.** The PR appended the new GA-aware sentence at `:73-76` but left PR #14's invariant in place: "This **preview-only** workflow revision **retains beta as that public default**." After this change that is false — the workflow accepts and passes through `stable`. The paragraph now asserts both that beta is retained as the default and that the caller must supply `stable` once active. The removed sentence ("The protected GA milestone must first land and pin a paired workflow revision that selects stable") shows the author was editing exactly this text, so this is an oversight, not a deliberate carve-out. This is the paragraph a future operator reads to decide whether the workflow can emit a stable default.

**Low — `README.md:64-67`, input enumeration is now short by one.** "Every dispatch input is required and immutable: two exact commits, a positive index generation, the index, plan, publication-facts, and four dispatcher SHA-256 digests, plus the publication attempt identifier" enumerates 11; the workflow now takes 12. Relatedly, the pre-deploy live-index GA-state binding — the PR's central safety property — is documented nowhere; `:56-60` still says only that the workflow "re-fetches the governed index immediately before deployment."

**Low — `release-site.yml:182-185`, distinctness guard goes vacuous in GA mode.** `cmp -s index beta` was written to prove the entrypoint marker keeps `/install` distinct from the channel dispatcher it defaults to. When the default is `stable`, `index` and `beta` differ trivially (different `DEFAULT_CHANNEL`), so the check passes while the pair that matters — `index` vs `stable` — is unchecked. I confirmed `index ≠ stable` does hold via the marker, so this is not exploitable today; it's a guard that silently stops guarding. `cmp -s "${install_root}/index" "${install_root}/${DISPATCHER_DEFAULT_CHANNEL}"` generalizes it.

### Cross-repo follow-up (not a defect in this PR)

The paired Sifr revision is not yet consistent with this head, and must land after this merges:
- `release-publication.yml:54` pins `SITE_WORKFLOW_SHA256: 6a04809d…` (the `721bcec` bytes). This head's bytes are `2a0d1901fa9fba799705c9116a03429dff01ee7ec3ee69d7ab688a8db90dca6d`. Until updated, `:107-108` fails closed and no GA-aware dispatch can run — correct direction, but it does block the milestone.
- `verification/areas/distribution_release/cases/site_release_workflow_contract.sh:24-64` still asserts `workflow_commit: 721bcec…`, the old digest, an `--default-channel beta` literal (the caller now uses `"${site_default_channel}"`), and an **order-sensitive** `required_inputs` list omitting `dispatcher_default_channel`. Note the position this PR chose: between `dispatcher_beta_sha256` and `publication_facts_sha256`.

The Sifr caller's `preview→beta / active→stable` mapping (`release-publication.yml:250-257`) and its threading into facts (`:428`) and dispatch (`:481`) match this workflow's contract exactly.

Validation run: Ruby YAML parse (12 inputs, all `required: true`; step order as above), local `generate_dispatchers.sh --default-channel stable` byte comparison in a temp dir. No files modified.

NOT APPROVED
