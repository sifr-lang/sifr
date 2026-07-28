## Exact-head PR closure review — sifr-website PR #16

**Head identity — confirmed identical, three ways**
- local `HEAD` = `03a407933ad054309cef0d8408043012970af710`
- `git ls-remote origin refs/heads/codex/stable-release-facts-page` = same
- `gh pr view 16 → headRefOid` = same (state OPEN, base `main`, `MERGEABLE`/`CLEAN`)
- Site checkout `git status --porcelain --untracked-files=all` empty before and after my checks (I built only in a throwaway copy, since removed). Paired Sifr worktree not modified.

Diff is exactly the satisfied pass-3 shape: 6 files, +398/−13 (`ci.yml`, `release-site.yml`, `README.md`, `apps/sifr-site/package.json`, `apps/sifr-site/scripts/render-stable-release-page.mjs`, `sitemap.xml.ts`).

### Independently re-verified closures

| Area | Result |
|---|---|
| Preview path | `preview:beta:none` → `exit 0` with `STABLE_PAGE_REQUIRED=false`; positive absence assertion on `dist` (`release-site.yml:381-384`). Built without the flag: `dist/releases` absent, sitemap count 0. |
| Post-GA alpha/beta | `release-publication.yml:676-692` derives real facts whenever `site_default_channel == stable`, from the already-activated `publication/channels.json` (activation step `:622` precedes facts step `:641`). No path can pass `none` on an active index. |
| Stable / rollback / roll-forward | `run_stable_publication.sh:327` and `run_incident_publication.sh:474` pass the staged-facts digest with `--default-channel stable`. Facts seeded from `summary["evidence"]["plan_sha256"]` (`stable_publish.py:109`) and `summary["successor"]["plan_sha256"]` (`incident_publish.py:89`), each equal to the `--release-plan-sha256` the same dispatch carries; `release_index_sha256` = the dispatched `--index-sha256`. Exact on all three. |
| Exact facts-digest binding | Site re-fetches the live index, `validate --kind release-index --require-canonical`, pins digest **and** generation, regenerates via the same `generate-site-facts`, then byte-compares to `STABLE_SITE_FACTS_SHA256` (`:284`). Producer itself re-validates against the governed index (`release_plan.py:421`). |
| Node pin | `setup-node` (`.node-version` = 24) now at `:195`, before the renderer at `:293` and after both checkouts. |
| Mutation boundary | Rendered the page in a copy: `--untracked-files=all` lists exactly `?? apps/sifr-site/public/releases/stable/index.html`; the three-`grep -vE` filter yields empty. |
| Deploy + public byte checks | `STABLE_PAGE_REQUIRED=true npm run build:site`: `dist/releases/stable/index.html` sha256 `6fbe6b7b…` equals the `public/` digest — Astro copies verbatim. Public check uses `/releases/stable/`, matching the canonical link and the fixture route; `wrangler.jsonc` `run_worker_first: ["/install"]` and `worker.js` touch only `/install`, so `/releases/*` is asset-served with no redirect. |
| Renderer validation/escaping | Self-test PASS. Fed **Python**-produced canonical bytes (`common.write_canonical_json`) straight into the JS canonical check — accepted, rendered correctly: cross-runtime parity holds on real output. `schema_version !== 2` fails closed; no v1 reader, migration, fallback, or raw rendering. `wx` write, non-canonical rejection, ordering assertion, `&lt;incident-one&gt;` escaping all exercised. |
| Shell parse gate | `preview_release_workflow_yaml_parses.sh` sweeps `scripts/distribution/*.sh` with `bash -n`; case exits 0. |
| Contract | `site_release_workflow_contract.sh` → PASS. Fixture's 13 `required_inputs` match the workflow's `workflow_dispatch` inputs exactly (same names, same order, all `required: true`); `stable_documentation` carries renderer/route/`preview_behavior`; the case asserts both the CLI flag and the jq payload key. `stable_publication_workflow_contract.sh` → exit 0. |
| README | Documents the facts binding, GA precondition, preview `none` rule, the byte-verified GA page, and the single additional mutation path. |
| Permissions | `contents: read` at workflow scope only, read-only `github.token`, no new secrets, no `id-token`, no release-metadata write. |

### Findings

No blockers. No medium findings.

**Low (all pre-existing or already accepted in pass 3; none blocking)**
1. `release_plan.py:408` orders withdrawals by `sorted(index["releases"].items())` — lexicographic. Reconfirmed live: `0.10.0 (inc-2), 0.9.0 (inc-1)`. Pre-existing generator behavior, newly public.
2. `release-site.yml:263` `active:stable:[0-9a-f]*` is a loose glob; harmless because `:110-112` already pinned 64-hex, but it does not itself constrain identity.
3. The GA branch (sitemap route + materialize→dist→digest chain) has no CI gate — site CI builds only with `STABLE_PAGE_REQUIRED` unset. I exercised both branches manually here.
4. README writes the route as `/releases/stable` while the fixture, canonical link, and public verification use `/releases/stable/`.
5. `dispatch_stable_site_publication.sh` still advertises `--default-channel alpha` in usage and its `^(alpha|beta|stable)$` regex, though `channel_facts_valid` makes it unreachable. Paired-Sifr surface.
6. `docs.sifr.sh/releases/stable` (committed MDX) and `sifr.sh/releases/stable/` (live-rendered) can disagree after a rollback until the docs site redeploys.

The `workflow_commit` / `workflow_sha256` / protected-tag / ruleset repin in `site_release_contract.json` is correctly still at the pre-change values — that is the paired-Sifr post-merge step, not a defect in this PR.

VERDICT: SATISFIED
