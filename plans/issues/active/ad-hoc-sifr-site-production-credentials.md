# Ad hoc: restore Sifr site production deployment credentials

## Status

Active external configuration prerequisite discovered during the live schema-v2
bootstrap on 2026-07-29.

## Evidence

The protected bootstrap-index run
[`30443929353`](https://github.com/sifr-lang/sifr/actions/runs/30443929353)
successfully published `0.1.0-beta.15`, reserved
`channels-generation-1.json`, and activated canonical schema-v2 generation 1
with SHA-256
`04edacb8ef64706e2285ec241fc23f7d5f2b80199bb1c2bac5889c48e8485964`.
Its exact correlated website run
[`30445065348`](https://github.com/sifr-lang/sifr-website/actions/runs/30445065348)
then failed only at `Deploy sifr.sh`: both `CLOUDFLARE_API_TOKEN` and
`CLOUDFLARE_ACCOUNT_ID` were empty.

Read-only inspection found neither secret at the `sifr-lang/sifr-website`
repository, organization, nor `sifr.sh-production` environment scope. The
local Wrangler OAuth credential is expired and cannot refresh
non-interactively. No secret value is present in the Phase 40 worktree or
process environment.

## Exact recovery inputs and deadlines

The reviewed recovery must use these immutable inputs:

- original publication run/attempt: `30443929353-1`
- failed correlated site run: `30445065348`
- alpha/beta versions: `0.1.0-alpha.2` / `0.1.0-beta.15`
- Sifr source commit:
  `94a5fec67b7bef51cae0034c84386c57d9ff1785`
- release-plan SHA-256:
  `979d469cb21675e4df6943220deb0f6453d4d1f8c3fb2056c108b8b7ec98f43f`
- generation/index: `1` /
  `04edacb8ef64706e2285ec241fc23f7d5f2b80199bb1c2bac5889c48e8485964`
- site base commit:
  `ff472f2af59255c8031b1a6f9b9b294c4b820496`
- dispatcher SHA-256 values: index
  `93a40ff1224a038402ed4952d968404ee503368d368b43166809db86ec562cc4`,
  stable
  `4dc2fde3dcc5deb8aa390900c3e8ef606e9ef46f6c1c3b2471a1caa3c29a73ae`,
  alpha
  `afbe013b87273e8b7aa0f676ff658ad82159434cfe5339369b1ae9ad63a69bac`,
  and beta
  `5885601276c1aa157146b5262ea505ba57c3081513dbe4338b09df2477d35481`
- dispatcher default channel: `beta`
- publication-facts SHA-256:
  `f3f03dd9366d61269d83f06d43c7d29b89edbe756207a40af0895ddb9ccf8dc1`
- stable-site-facts SHA-256: `none`
- original prepare-summary SHA-256:
  `f45c012c17d2908bc2ef227f202e1037343c63d1f1881ca7913f22628f62a086`

The original prepare artifact expires at `2026-08-28T10:46:13Z`, but it is no
longer a recovery dependency: its exact canonical bytes are retained at
`plans/releases/schema-bootstrap-recovery/prepare-summary-30443929353-1.json`
and the workflow verifies the digest above. The temporary single-maintainer
approval waiver expires earlier, at `2026-08-27T00:00:00Z`; the protected
recovery must complete before then unless a distinct reviewer is configured.
Recovery must finish before `ga-activation` advances the live index beyond
generation 1. Separately, the qualified `0.1.0` GA prepare must begin before
`2026-08-21T02:17:30Z` to preserve the required seven full days before its
`2026-08-28T02:17:30Z` qualification expiry; the waiver deadline does not
extend that candidate window.

## Required action

- Create or obtain a least-privilege Cloudflare API token authorized to deploy
  the existing `sifr.sh` Worker, and identify its Cloudflare account ID.
- Store the values as `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`
  environment secrets on `sifr-lang/sifr-website` /
  `sifr.sh-production`.
- Verify the secret names exist without exposing their values.
- Dispatch the reviewed `schema-bootstrap-recovery` workflow with the exact
  failed mutation/site identities and approve its `stable-release`
  environment.
- Confirm the recovered site run, public schema-bootstrap smoke, and
  `schema-v2-bootstrap-generation-1.json` evidence all succeed.

## Non-goals

- Do not move credentials into repository variables, source, logs, artifacts,
  or the Sifr publication repository.
- Do not replace, clobber, or allocate another release-index generation.
- Do not republish the alpha/beta releases or overwrite their assets.
- Do not relax the protected site workflow, its pinned identity, or its
  deployment verification.
