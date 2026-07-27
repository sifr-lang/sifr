## State confirmed

- `HEAD = 310e0dec633cc667b1f2a21a49a09fe52e01c957`, tracked tree clean (only untracked file is `plans/reviews/active/phase-40-milestone-40-2-claude-opus-review-pass-4.md`). Reviewed the whole `e0cc278a5..HEAD` range (5 commits, 69 files). No files modified.

### Independently re-run at this head
| Gate | Result |
|---|---|
| `verification/areas/distribution_release/runner.py` | **variants=102, failures=0** |
| `cargo test -p sifr --bins self_update` | **48/48** |
| `cargo fmt --check` | clean |
| `check_hir_maintainability_guardrails.py` | PASS (`self_update_metadata.rs` = 875 lines) |
| `site_release_workflow_contract.sh`, `preview_release_workflow_yaml_parses.sh` | PASS |
| `demos/stable_self_update_demo.sh` | forced beta→stable, stable→stable, no-op, alpha\|beta-only workflow check — pass |
| External read-only: ruleset `19790146`, site tag, pinned workflow bytes | `enforcement=active`, `target=tag`, `include=[refs/tags/sifr-release-site-m40-2]`, `rules=[update,deletion]`, `bypass_actors=[]`, `current_user_can_bypass=never`; tag → `07d88cc3…` (commit); `release-site.yml` sha256 = `7a27abaf…` matching `SITE_WORKFLOW_SHA256` |

## Pass-3 findings: all four genuinely closed

1. **README** (`demos/preview_release_lifecycle/README.md`) — rewritten to six commands, all of which exist and are the same read-only planner cases the suite runs; no `--real-run`/`--mutation-mode`/`--binary`, no `plan_sha256`/`dry_run_side_effects`/artifact/stable-entrypoint output claims. Closed.
2. **Canonical site repository** — `release_plan.py:279-280`, `stable_release_plan.schema.json:125`, `schema_contracts.py:239`, `selftest.py:173` all now require `sifr-lang/sifr-website`. Remaining `blog-website` hits are historical prose about the redirect only. Closed.
3. **Parsing/lookup** — `self_update_metadata.rs:58-67` is now a single `let Some(prerelease) = prerelease else { return Ok(stable) }` with no unreachable arm; `installer_sha256` alias deleted; `resolve_update_plan:360-365` performs exactly one `resolve_exact` for both validation and digest. Closed.
4. **Release provenance** — `release-publication.yml:370` adds `--target "${SOURCE_COMMIT}"`, `:375-382` verifies the created tag resolves to `SOURCE_COMMIT`, and pre-mutation checks now reject both an existing release (paginated `/releases`, `:126-139`) and an existing exact tag (`git/matching-refs`, `:140-149`). Both are fail-closed under `set -euo pipefail` (a failed `gh` or `jq` aborts the step). Closed.

## Findings

**1 — Medium · governance fail-open · `.github/workflows/release-publication.yml:103-116` and `:516-529`**
The milestone requires "an active exact-name repository ruleset **with no bypass actors** … and revalidate that ruleset at both boundaries." Both jq guards check `id`, `target`, `enforcement`, `conditions.ref_name.include/exclude`, and `[.rules[].type]` — but **not** `bypass_actors == []` (nor `current_user_can_bypass == "never"`), even though the reviewed contract records `"bypass_actors": []` (`fixtures/site_release_contract.json:8`, `cases/site_release_workflow_contract.sh:31`) and the contract test asserts the other error strings but no bypass check. Failure scenario: an admin or integration is added as a bypass actor on ruleset 19790146; the `sifr-release-site-m40-2` tag becomes force-movable, and both the pre-mutation and pre-dispatch revalidations still pass, so the workflow reports the tag as immutably protected when it is not. Residual mitigation (why Medium, not High): polling binds `head_sha` to `SITE_BASE_COMMIT` (`:599,603`), so a run started from a moved tag fails to correlate and the step errors rather than accepting a wrong deployment. Fix: add `and .bypass_actors == []` to both jq filters and assert that fragment in `site_release_workflow_contract.sh`.

**2 — Low · deadline semantics · `.github/workflows/release-publication.yml:585,620` (job `timeout-minutes: 30` at `:43`)**
The "hard 20-minute deadline" is implemented as `for _ in $(seq 1 120)` + `sleep 10`, i.e. 1200 s of sleep **plus** one paginated `gh api` per iteration — real elapsed is ~21–23 min. That poll shares a 30-minute job budget with artifact download, installer generation, four-archive verification, snapshot download/validation, the 9-asset release create, and a full asset re-download/byte-compare. If those precede it by ≳8 min, GitHub kills the job at 30 min, so the `actions/runs/${site_run_id}/cancel` request and the terminal failure line (`:622-628`) never execute — contradicting "On expiry the main workflow requests cancellation, records a terminal failed attempt, exits." Use a wall-clock bound (`deadline=$((SECONDS+1200))`) and give the poll headroom (own step/job timeout or a larger `timeout-minutes`). Bounded by the site side re-checking index generation+digest before deploy, so an orphan run cannot deploy stale bytes.

**3 — Low · consumer asymmetry vs stated invariant · `crates/sifr/src/self_update_metadata.rs:165-300, 360-365`**
`ga_status` never appears in any Rust source. `ChannelMetadata::parse` enforces "preview ⇒ no stable *channel*" but keeps every `active` release record in `active_installers` regardless of class, and `release_index.py` does not forbid a stable release record under `ga_status: preview`. So for a governed-valid preview index that contains an active stable record, `sifr self update 0.1.0` resolves, whereas the shell dispatcher rejects the same pin (`generate_dispatchers.sh:160-163`, "stable channel installs require active GA metadata"). The phase invariant is "While `ga_status` is `preview` … stable channel or exact stable-version selection is rejected." Not producible by the governed pipeline today (stable records and `ga_status: active` land together in 40.5), and no test covers it — hence Low, but it is a real divergence between the two consumers of the same index.

**4 — Low · credential scope · `.github/workflows/preview-release.yml:147`**
`secrets: inherit` hands every repository secret to the mutation workflow, which needs only `SIFR_WEBSITE_ACTIONS_TOKEN`. The scope asks for the dispatch credential to be "scoped only to the required Actions/contents operations on `sifr-lang/sifr-website`"; passing the whole secret set widens the blast radius of any future step added to `release-publication.yml`. Prefer an explicit `secrets:` map.

**5 — Informational · fragile guard shape · `.github/workflows/release-publication.yml:436-442`**
`if gh release view … --jq '.assets[].name' | grep -Fxq "$(basename "${snapshot}")"` runs under `set -o pipefail`: if `grep -q` exits on the first match and `gh` takes SIGPIPE, the pipeline status is non-zero and the "generation snapshot already exists" branch is skipped (commands in an `if` condition are exempt from `set -e`). Not exploitable — the subsequent `gh release upload` has no `--clobber` and fails closed on a duplicate asset — and the asset list realistically fits the pipe buffer. Worth reshaping to `grep -Fxq … <<<"$(gh …)"` for the same reason the rest of the file avoids early-exit pipelines.

### Examined and judged sound
- Write-once/replace-only ordering: snapshot upload < `channels.json` replacement < site dispatch, asserted structurally at `preview_release_workflow_yaml_parses.sh:67-71`; `--clobber` appears exactly once.
- Installer digest is verified **before** `make_executable` (`self_update_runner.rs:46, 189-198`) with a negative test asserting the installer never ran; no extraction or binary replacement in the Rust runner.
- Lease reacquisition re-validates live generation **and** digest after entering the concurrency group (`:406-433`) and re-verifies the activated bytes after replacement (`:452-462`).
- Preview-index-with-`channels.stable` rejection lost from the deleted `channel_metadata_stable_rejected.sh` is preserved in `governance/selftest.py:406`; `preview_metadata_rejects_stable_channel` covers the Rust side.
- `rc` sweep (`release_surfaces_reject_rc.sh`) covers every non-JSON runtime/workflow surface including `release-publication.yml`; the only remaining `-rc.` string in the tree is a rejection assertion.
- Local planner is genuinely read-only: `--dry-run` mandatory, mutation/artifact flags hard-rejected, stdout-only plan, verified by `create_new_version_plan_is_read_only.sh` byte comparison.

Finding 1 is a concrete, explicitly-scoped requirement that is not implemented at either boundary and is not test-covered.

NOT APPROVED
