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
