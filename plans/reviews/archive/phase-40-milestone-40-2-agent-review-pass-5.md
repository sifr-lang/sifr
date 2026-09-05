## Phase 40 · Milestone 40.2 — full re-review at `f3a5bed94` (read-only)

**State confirmed.** `HEAD = f3a5bed94a35299a00d4373891861819759508c6`; tracked tree clean (only untracked file is `plans/reviews/active/phase-40-milestone-40-2-agent-review-pass-5.md`). Reviewed the whole `e0cc278a5..HEAD` range (6 commits, 70 files). No files modified.

### Independently re-run at this head

| Gate | Result |
|---|---|
| `verification/areas/distribution_release/runner.py` | **variants=102, failures=0, blocking=0** |
| governance self-tests | 14/14; qualification 8/8; schema epoch, evidence custody ok |
| `cargo test -p sifr -- self_update` | all pass; `self_update_metadata` + `exact_pin_tests` = **16/16** |
| `cargo clippy --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `check_file_size_guardrails.py` | PASS (limit 900; `self_update_metadata.rs` = 882) |
| `check_hir_maintainability_guardrails.py` | PASS |
| `git diff --check e0cc278a5..HEAD` | **FAILS** (see finding 1) |
| `demos/stable_self_update_demo.sh` | **not runnable here** — host disk exhausted (`ld: write() failed, errno=28`, 158 MiB free). Environment limitation, not a code defect; unverified by me this pass. |

### Pass-4 findings — all five genuinely closed

1. **Ruleset governance fail-open** — closed. Both boundaries (`release-publication.yml:104-121`, `:524-541`) now add `.updated_at == $updated_at` against the pinned revision `2026-07-27T04:17:06.204Z` (`:56`), plus `(.bypass_actors // []) == []` and `(.current_user_can_bypass // "never") == "never"`. The reasoning holds: any ruleset mutation (including adding a bypass actor) goes through the ruleset update endpoint and moves `updated_at`, so a token that cannot see the privileged fields still fails closed on the revision pin. A 404/permission failure also fails closed — `site_ruleset="$(gh api …)"` is a simple command under `set -euo pipefail`, so a failing substitution aborts the step. `.conditions` omission likewise yields `null ≠ [$ref]`. Fragments are asserted in `cases/site_release_workflow_contract.sh:70-88`.
2. **Deadline semantics** — closed. `poll_deadline=$((SECONDS + 1200))` (`:598`), each `gh api` bounded by `timeout --foreground "${remaining_seconds}s"` (`:606`), exit 124 → break, non-zero → `poll_error`, cancellation attempted when `site_run_id` is known (`:659-663`), terminal `::error::` on both paths. `SECONDS` is per-step so the 20 min is measured inside the poll only, and `timeout-minutes: 60` (`:43`) leaves ~40 min of headroom for the preceding steps. Sleep is clamped to remaining time (`:650-657`).
3. **Rust/dispatcher asymmetry on stable pins under preview** — closed. `ChannelMetadata` now stores `ga_active` (`:160`, `:297`) and `resolve_exact` rejects any `PreviewChannel::Stable` version when `!ga_active` (`self_update_metadata.rs:313-329`), independent of whether a historical active stable record exists. `exact_pin_tests::preview_metadata_rejects_exact_stable_record` builds exactly that adversarial index. Matches `generate_dispatchers.sh:160-163`.
4. **Credential scope** — closed. `preview-release.yml:147-148` maps only `SIFR_WEBSITE_ACTIONS_TOKEN`; `secrets: inherit` is rejected by `preview_release_workflow_yaml_parses.sh:34-35`, and the reusable workflow declares it `required: true`.
5. **Pipefail/SIGPIPE guard shape** — closed. `snapshot_names` is captured first, then `grep -Fxq … <<<"${snapshot_names}"` (`:441-450`).

Pass-3 findings 1–4 also remain closed: the lifecycle README lists only commands that exist, `sifr-lang/sifr-website` is now the enforced constant in `release_plan.py:279`, `stable_release_plan.schema.json:125`, `schema_contracts.py:239`, `selftest.py:173` (zero residual `blog-website` occurrences outside historical prose), the dead `prerelease` arm and `installer_sha256` alias are gone with exactly one `resolve_exact` per plan, and `gh release create` carries `--target "${SOURCE_COMMIT}"` plus post-create tag verification (`:373-387`).

### Additionally verified sound this pass

- Write-once/replace-only ordering (snapshot upload → `channels.json` replace → dispatch) enforced structurally and asserted at `preview_release_workflow_yaml_parses.sh:72-76`; `--clobber` appears exactly once.
- Max-generation allocator scans `channels-generation-N.json`, cross-checks name vs. payload generation, and the duplicate-snapshot guard backstops a silently-empty process substitution (`gh release upload` has no `--clobber` there).
- `release_governance.py update-preview-index` now requires `--proposed-generation` and `release_index.py:151-157` rejects a non-advancing value; dropping the "preview index only" restriction correctly permits alpha/beta publication under `ga_status: active` while preserving `channels.stable`.
- Installer digest is verified before `make_executable` (`self_update_runner.rs:46, 189-198`); `resolve_update_plan` requires metadata unconditionally, so `--version` pins are now digest-bound too.
- `rc` is fully swept from every runtime/workflow surface; the only remaining `-rc.` strings are rejection assertions.
- The `create_new_version.sh:118` / `validate_self_update_metadata.sh:157` expectation that the *checked-in* `index` dispatcher defaults to `stable` while publication deploys `beta` under `ga_status: preview` is intentional and documented (`internal_docs/distribution_pipeline.md:52`, `:436-441`), matching the pass-3 adjudication. Not a finding.

---

## Findings

**1 — Low · whitespace · `verification/areas/distribution_release/cases/install_installer_checksum_mismatch_rejected.sh:14`, `verification/areas/distribution_release/cases/install_withdrawn_stable_rejected.sh:27`**
Both new case files end with a trailing blank line (`…run_dispatcher index\n\n`), so `git diff --check e0cc278a5..HEAD` exits 2 with `new blank line at EOF` on both. This contradicts the stated passing gate. No merge gate enforces it (`run_all_tests.sh` runs no whitespace/shellcheck pass) and there is no behavioral impact, but it is a real, introduced-by-this-diff defect and a two-byte fix.

**2 — Informational · polling robustness · `.github/workflows/release-publication.yml:620-623`**
A single non-timeout `gh api` failure (transient 5xx, secondary rate limit, a `--paginate` page erroring) sets `poll_error` and terminates the publication immediately, with no retry. Because this runs *after* `channels.json` replacement and dispatch, and `site_run_id` is still empty on an early failure, the correlated site run is neither cancelled nor awaited — it will proceed to deploy while the main workflow reports terminal failure. Fail-closed on the index (the site side re-validates generation+digest before deploy), so this is operational noise rather than a correctness hole; a bounded retry before giving up would match the care taken elsewhere in the step.

**3 — Informational · unverified locally**
`demos/stable_self_update_demo.sh` could not be executed at this head — the host filesystem is full (158 MiB free; the demo uses a separate 2.2 GB `CARGO_TARGET_DIR`). Its result is carried over from pass-4, not independently confirmed by me.

---

Only finding 1 is actionable, and it is cosmetic; findings 2 and 3 are not blockers. Per the stated bar, an actionable finding remains.

NOT APPROVED
