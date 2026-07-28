# Re-review: Phase 40 site stable-release-facts rendering (pass 2)

Both worktrees inspected read-only; no files modified. I verified each pass-1 finding and independently re-derived the GA/preview, provenance, and digest-binding paths.

## Pass-1 findings: status

| # | Finding | Status |
|---|---|---|
| 1 | Unconditional page materialization breaks preview | **Resolved** — explicit `preview:beta:none` accept-and-skip, `active:stable:<sha>` required, `STABLE_PAGE_REQUIRED` gates both `${STABLE_PAGE_SHA256}` references, plus a positive absence assertion on `dist`. |
| 2 | Unpinned Node | **Resolved** — `setup-node` (`.node-version` = 24) now precedes the renderer at `release-site.yml:195`. `.release/sifr` is checked out earlier, so ordering is sound. |
| 3 | Cross-repo contract does not govern the change | **Partially resolved** — see M1. |
| 4 | README | **Resolved** — mutation/provenance/preview/active contract documented. |
| 5 | Page bytes not tied to Sifr canonical facts | **Resolved and verified exact.** Both sides use `write_canonical_json`; stable passes `expected_plan_sha256`, which `revalidate_stable_publication.py:66` proves equals `summary["evidence"]["plan_sha256"]` used by `stable_publish.py:109`; incident passes `.successor.plan_sha256`, matching `incident_publish.py:89`. Site regenerates and byte-compares. |
| 6 | JS/Python validator divergence | **Resolved** — `PUBLIC_VERSION` now admits alpha/beta, `incident_id` is nonempty-string, and the `version\0incident_id` sort is order-equivalent to Python's tuple sort (`\0` < every printable byte). |
| 7 | Self-test gaps | **Resolved** — empty/multiple withdrawals, escaping, `--facts/--out`, `wx` refusal, noncanonical bytes, bad ordering all covered. |
| 8, 9, 10 | Low | 8 and 9 unchanged; 10 partially (renderer moved). |

---

## Blockers

**B1 — `scripts/distribution/dispatch_stable_site_publication.sh:88` does not parse. Every site dispatch aborts.**
The validation expression has one unbalanced `)`:

```
      "${stable_site_facts_sha256}" == "none")) &&
```

Verified against GNU bash 5.3.9:
```
line 67: syntax error in conditional expression: unexpected token `)'
line 88: syntax error near `"none"))'
```
`bash -n` fails on this file (and only this file — every other `scripts/distribution/*.sh` parses). The reported "Sifr: bash -n" validation did not cover it. Consequence: `run_stable_publication.sh:333`, `run_incident_publication.sh:479`, and `release-publication.yml:693` all invoke this helper, so GA activation, rollback, roll-forward, and every preview publication fail at the site-dispatch step with a shell syntax error.

**B2 — Once B1's paren is removed, the mixed `&&`/`||` at lines 84–89 opens two validation bypasses.**
`&&` binds tighter than `||` inside `[[ ]]`, so the expression parses as
`(all-input-checks && stable-branch) || (beta-branch && !-e && !-L)`.

I reproduced both consequences with the exact shape of lines 84–89:

- `--default-channel beta --stable-site-facts-sha256 none` → the *entire* first conjunct group is short-circuited away. Garbage `--repository`, `--source-commit`, `--workflow-sha256`, and all four dispatcher digests are accepted (`BYPASS: bad input accepted`). This is the path `release-publication.yml:709` uses today.
- `--default-channel stable` with a valid digest → the first disjunct is already true, so `! -e "${result_out}" && ! -L "${result_out}"` is never evaluated. The write-once/symlink guard on the result file is inert on the GA path (`BYPASS: existing result_out accepted`).

Fix: parenthesize the whole channel/facts pairing as a single conjunct, e.g. `... && ( (stable && sha64) || (beta && none) ) && ! -e ... && ! -L ...`.

**B3 — Post-GA preview publications are not wired and will hard-fail.**
`release-publication.yml:431-434` derives `site_default_channel` from the proposed index's `ga_status`: `active → stable`. Alpha/beta publications continue after GA activation (that mapping exists for exactly this case, and `create_new_version_active_site_dispatchers.sh:30` asserts it). But `release-publication.yml:709` unconditionally passes `--stable-site-facts-sha256 none`.

Result once GA is active and an alpha/beta release is published:
1. The dispatch helper rejects it (`stable` requires 64-hex, `none` only pairs with `beta`) → exit 2, publication fails.
2. Even if it were let through, `release-site.yml:255-267` would evaluate `active:stable:none`, fall to `*`, and fail with "stable page facts do not match the governed GA state".
3. And if *that* were relaxed to a skip, it would be worse: the page is only ever produced by that step, so `dist` would lack `/releases/stable` and the deploy would silently delete the live GA release page.

The preview path must generate the canonical facts (`generate-site-facts` against the proposed index) and pass the real digest whenever `ga_status == active`. This is a code change required before the site PR, independent of the site commit hash.

---

## Medium

**M1 — The cross-repo contract still under-specifies the new dispatch and the site-side renderer.**
`verification/areas/distribution_release/fixtures/site_release_contract.json` lists 12 `required_inputs`; the site workflow now requires 13 and the dispatcher sends `stable_site_facts_sha256`. `site_release_workflow_contract.sh`'s `validate()` enforces `set(candidate) != set(fixture["required_inputs"])`, but only against its own hand-written `payload` — nothing ties it to the real jq payload in the helper, so the drift is invisible. Add the input to `required_inputs` and to the test's `payload`, and assert `stable_site_facts_sha256: $stable_facts` appears in the dispatch helper.

Separately, the new `stable_documentation` block names only the Sifr-side `render_stable_release_docs.py` / `docs/releases/stable.mdx`. Nothing in the Sifr repo asserts that the site renders `/releases/stable` from schema-v2 facts with no v1 reader, no fallback, and no raw rendering — that guarantee still lives only in the site repo's own CI. The site-side renderer path, route, and the `preview → no page` rule are all knowable now and belong in the fixture. (The `workflow_commit`/`workflow_sha256`/tag/ruleset repin is correctly deferred to post-site-PR.)

**M2 — No shell parse or lint gate exists anywhere in the repo.**
`grep -rn "bash -n\|shellcheck" scripts/ verification/ .github/` returns nothing. B1 is a total-failure bug in a release-critical script that passed every reported validation, including the site/incident/stable/preview contract cases — because those cases only substring-match file text. A `bash -n` sweep over `scripts/distribution/*.sh` in the distribution_release area would have caught it in under a second.

---

## Low

- **L1** — `render-stable-release-page.mjs:75-76` accepts any nonempty `incident_id`, while the governed index constrains it to `^[a-z0-9][a-z0-9-]{2,63}$`. Mitigated by the exact digest binding (facts are byte-compared before rendering) and by `escapeHtml`, so this is defense-in-depth only. The self-test's `<incident-one>` input is deliberately outside the governed alphabet — fine for testing escaping, but worth a comment saying so.
- **L2** — Withdrawals remain lexicographically ordered (`release_plan.py:407`), so `0.10.0` will render before `0.9.0` on the public page. Pass-1 finding 8, now additionally cemented by the JS ordering assertion. Pre-existing; newly public.
- **L3** — The page's `rel=canonical` is `https://sifr.sh/releases/stable`, but with Cloudflare's default `html_handling: auto-trailing-slash` and an asset at `/releases/stable/index.html`, the served canonical URL is `/releases/stable/`. `verify_public_dispatcher` uses `--location` so verification still passes, though the cache-busting query may be dropped across the redirect (the `no-cache` headers still apply). Consider verifying `/releases/stable/` as well.
- **L4** — `test:stable-release-page` lives in the root `package.json` while the sibling `test:install-routing` is `-w apps/sifr-site`; and `/releases/stable` is absent from `sitemap.xml.ts`. Pass-1 finding 10's placement half is resolved, these two halves are not.
- **L5** — `docs.sifr.sh/releases/stable` (committed MDX) and `sifr.sh/releases/stable` (live-rendered) can disagree after a rollback until the docs site redeploys. Unchanged from pass-1 finding 9.

---

## Verified sound

Preview/active branching is explicit and fail-closed in both directions; `STABLE_PAGE_SHA256` is never referenced unbound under `set -u`; duplicate `GITHUB_ENV` keys resolve last-wins correctly. Index provenance is re-fetched, `--require-canonical`-validated, and digest- and generation-pinned before use. Facts regeneration is byte-exact against the Sifr-staged bytes on both the stable and incident paths (traced through `revalidate_stable_publication.py`, `stable_publish.py`, `incident_publish.py`, and the shared `write_canonical_json`). Mutation boundary uses `--untracked-files=all`, so the new file is listed individually rather than as a collapsed directory, and the allowance is the one exact path. Permissions unchanged (`contents: read`, read-only `github.token`, no new secrets). `wx` write refusal, canonical-bytes rejection, and deterministic re-runs all hold. `RUNNER_TEMP` freshness makes `refuse_existing=True` safe. The Cloudflare worker does not intercept `/releases/*` (`run_worker_first` is `/install` only). JS↔Python withdrawal-ordering semantics are equivalent.

VERDICT: CHANGES_REQUIRED
