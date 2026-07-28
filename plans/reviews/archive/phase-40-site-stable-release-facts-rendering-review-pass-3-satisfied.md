Both worktrees inspected read-only. I created and then removed one temporary file in the site worktree while exercising the build; `git status --porcelain --untracked-files=all` there is byte-identical to the state I found it in.

## Pass-2 findings: verified status

| # | Finding | Status |
|---|---|---|
| B1 | `dispatch_stable_site_publication.sh` does not parse | **Resolved, executed.** GNU bash 5.3.9: every `scripts/distribution/*.sh` parses, and every git-tracked `*.sh` in the repo parses. |
| B2 | Mixed `&&`/`||` opens two bypasses | **Resolved, reproduced closed.** `channel_facts_valid` is computed in a separate `if`/`elif` (`:67-74`) and enters the guard as one conjunct (`:93`), with `! -e`/`! -L` unconditional. 10 executed cases: `beta:none` fresh → accept; `beta:none` + garbage `--repository` → reject; `stable:<64hex>` + pre-existing `result_out` → reject (the pass-2 inert guard now fires); `stable:none`, `beta:<64hex>`, `alpha:none`, uppercase hex, 65-hex → reject; `stable:<64hex>` fresh → accept; `beta:none` + pre-existing `result_out` → reject. |
| B3 | Post-GA alpha/beta publications not wired | **Resolved, provenance traced exact.** `release-publication.yml:676-693` derives facts via `generate-site-facts` from `publication/channels.json` whenever `site_default_channel == stable`, with the same `--source-plan-sha256`/`--release-index-sha256`/four dispatchers the dispatch carries. The site regenerates from a live index pinned to `RELEASE_INDEX_SHA256` and generation, so its input bytes are identical, and byte-compares the result (`release-site.yml:284`). The activation step (`:623-640`) precedes the facts step, and the site's own digest pin fails closed if it didn't. Post-GA previews therefore always render, so no deploy can silently delete a live GA page. |
| M1 | Contract under-specifies the dispatch and the renderer | **Resolved.** Fixture carries the 13th `required_input` plus `stable_documentation` with `renderer`, `route: "/releases/stable/"`, `canonical_producer`, `preview_behavior: "absent"`; the test asserts `--stable-site-facts-sha256 "${STABLE_SITE_FACTS_SHA256}"` and `stable_site_facts_sha256: $stable_facts` against the real workflow and jq payload, and adds a GA-status↔facts-identity rule with a `preview stable facts` negative. |
| M2 | No shell parse gate | **Resolved.** `preview_release_workflow_yaml_parses.sh:7-9` sweeps `scripts/distribution/*.sh` and is picked up by the `distribution-case-directory` manifest entry (executable, discovered). |
| L1 | `incident_id` looser than governed alphabet | **Resolved as intended** — comment at `render-stable-release-page.mjs:178-179` documents the deliberate out-of-alphabet escaping input; the digest binding remains the real constraint. |
| L3 | canonical vs served URL | **Resolved.** `:133` canonical and `release-site.yml:501` verification both use `/releases/stable/`; Cloudflare `auto-trailing-slash` serves it directly, no redirect, cache-buster preserved. |
| L4 | Script placement + sitemap | **Resolved.** `test:stable-release-page` is in `apps/sifr-site/package.json`, invoked `-w apps/sifr-site`; sitemap adds the route only under `STABLE_PAGE_REQUIRED=true`. |
| L2 | Lexicographic withdrawal ordering | **Still open, pre-existing.** Confirmed live: a real `generate-site-facts` run over an active index withdrawing `0.9.0` and `0.10.0` renders `0.10.0 (inc-2026-002), 0.9.0 (inc-2026-001)`. |
| L5 | docs.sifr.sh vs sifr.sh skew after rollback | Unchanged. |

## Executed verification

- Renderer self-test: PASS. `bash -n` over all `scripts/distribution/*.sh` and all tracked `*.sh`: clean.
- End-to-end provenance: built a canonical active index with two withdrawn stables, ran `release_governance.py generate-site-facts`, fed the Python bytes straight into the site renderer — the JS canonical-bytes check accepted them and rendered correctly. Python `write_canonical_json` and JS `canonicalFactsBytes` agree on real output.
- `STABLE_PAGE_REQUIRED=true npm run build:site`: `dist/releases/stable/index.html` digest equals the `public/` digest (Astro copies verbatim, no transform), and `dist/sitemap.xml` contains exactly `https://sifr.sh/releases/stable/`. No src-page route collision; `public/releases` is not gitignored; `wrangler.jsonc` has `run_worker_first: ["/install"]` and `not_found_handling: "404-page"`, so `/releases/*` is asset-served and preview 404s rather than falling back.
- Mutation boundary: with the page present, `--untracked-files=all` lists it as the single exact allowed path.
- Full `distribution_release` representative suite: **56 variants, 0 failures**, including `site_release_workflow_contract` and `preview_release_workflow_yaml_parses`.
- Production paths spot-checked: `run_stable_publication.sh:327-353` and `run_incident_publication.sh:474-497` both pass the exact staged facts digest with `--default-channel stable`, after index activation; `stable_publish.py:107-112` and `incident_publish.py:87-93` seed those staged facts from `summary["evidence"]["plan_sha256"]` / `summary["successor"]["plan_sha256"]` — matching the `--release-plan-sha256` each dispatch carries.
- Permissions: `contents: read` unchanged, read-only `github.token`, no new secrets, no new write scope.

## Low (non-blocking)

- **L-A — `--default-channel alpha` is now unreachable but still advertised.** `channel_facts_valid` can only become true for `stable`/`beta`, so `alpha` always falls through to the generic `usage()` with no specific diagnostic, while the usage text and the `^(alpha|beta|stable)$` regex (`:87`) still offer it — and `site_release_workflow_contract.sh` asserts that misleading string. `release-site.yml:115-118` accepts only `beta|stable`. Dead surface, worth dropping or diagnosing.
- **L-B — the GA branch has no registered gate.** Site CI builds only with `STABLE_PAGE_REQUIRED` unset, so the sitemap GA branch and the materialize→dist→digest chain are exercised only manually (I exercised both above). A CI step that renders fixture facts and builds with the flag set would pin it.
- **L-C — `active:stable:[0-9a-f]*` (`:263`) is a loose glob** (one hex char plus anything). Harmless because `:110-112` already pinned 64-hex, but it does not itself constrain the identity.
- **L-D — README writes the route as `/releases/stable`** while the fixture, canonical link, and public verification all use `/releases/stable/`.
- **L-E — ordering assertion doesn't cover the new step.** `preview_release_workflow_yaml_parses.sh` pins `snapshot < replacement < dispatch` but not that the facts step sits after the replacement. Correctness doesn't depend on it (the site's own digest pin fails closed), so this is a regression-guard gap only.
- **L-F — the `bash -n` sweep is scoped to `scripts/distribution/`.** Verification cases and `scripts/*.sh` remain unswept; all of them parse today, but a B1-class bug outside that directory would still slip through.

Every knowable implementation and contract issue from passes 1 and 2 is closed. The only remaining items are the deferred post-merge repin (`workflow_commit`/`workflow_sha256`/tag/ruleset), the pre-existing lexicographic withdrawal ordering, and the low-severity polish above.

VERDICT: SATISFIED
