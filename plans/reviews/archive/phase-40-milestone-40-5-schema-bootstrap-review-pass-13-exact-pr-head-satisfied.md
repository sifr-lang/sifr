## Archived pass 13 — exact-PR-head review, PR #3040

### Identity (verified independently)
| Source | SHA |
|---|---|
| local `HEAD` | `e51491338e396e6b8f2d19345c9df68242e2b029` |
| `origin/codex/phase-40-milestone-40-5-preview-bootstrap` | `e51491338e396e6b8f2d19345c9df68242e2b029` |
| PR #3040 `headRefOid` | `e51491338e396e6b8f2d19345c9df68242e2b029` |

Base `d8dd28a80` is the merge-base; PR is `MERGEABLE`, 47 files, 2 commits (`e7fc93efd` implementation, `e51491338` docs link) — matches the local diff exactly. Working tree is clean except the untracked, zero-byte pass-13 report slot (not modified).

### Prior findings — all closed
- **Pass 11 / finding 1 (ledger)** — `plans/issues/active/phase-40-stable-channel-ga-execution.md:346-471` now ledgers passes 1–12, the create-PR transient LSP timeout and its replay, both runner-registration remediations, the pass-11 prepare-workflow gap and its fix, the host-variance run, and the PR link.
- **Pass 11 / finding 2 (prepare outside the no-v1-residue sweep)** — `cases/schema_epoch_bootstrap_workflow_contract.sh:145-151` now runs the `bootstrap_channel_metadata.py` / `migrate` / `fallback` sweep over `(prepare, publication, bootstrap)`.
- **Pass-10 runner-registration repairs** — `sifr_verify/selftest.py:89` (`13` schemas) and `:684` (`epoch-bootstrap` in the release-report production fixture) both present and load-bearing against `release_report.REQUIRED_SUITES`.

### What I re-verified in the implementation
- **Approval/security**: `release-publication.yml:67-71` binds the publish job to `stable-release` for both bootstrap modes; `:108-156` revalidates the prepare-summary digest against the prepare job's output, cross-checks every field against the protected job's own inputs, then resolves approvers from the run's immutable approval history. `resolve_distinct_approvers` fails closed on an empty history and case-folds self-approval — exercised live: `["Bob"]`/`["alice"]` on valid histories, exit 2 with `requires a stable-release approval by someone other than alice` on `[]`. Preview mode is unchanged (`approvers_json=[]`).
- **Rerun-safe prepare**: artifact name `publication-prepare-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}` with `overwrite: false`, `retention-days: 30`, read-only permissions, no environment, no secret. Publish's `download-artifact` now carries `pattern: sifr-${version}-*` so the summary artifact can't leak into `release-assets`. `verify_release_publication_assets.sh:81-99` re-digests the publish-side set and requires `.assets` equality with prepare; `generate_version_installer.sh` is deterministic (its `date`/`mktemp`/`uname` are escaped into the emitted script, not evaluated at generation).
- **Opaque legacy identity**: the `71b3243…f9ef` / 105-byte pair is pinned identically in prepare (`:236-241`), publish index fetch (`:273-278`), lease revalidation (`:568-574`), `schema_bootstrap.py:28-29`, and the JSON Schema (`:33-34`) — cross-asserted by the contract case. No field of the pre-epoch asset is parsed anywhere.
- **Staged alpha → generation 1**: `fetch_schema_bootstrap_alpha.sh` re-validates staged evidence canonically, pins tag→source, re-downloads all 9 assets, checks the exact name set and per-asset digests, and proves the release record is byte-reproducible. `_require_exact_bootstrap_membership` forces generation 1 / `ga_status: preview` / exactly the two evidenced channels and records; `"stable"` is asserted absent.
- **Write-once ordering**: snapshot reservation (with a pre-existence guard) → single `--clobber` on `channels.json` → activated-digest recheck → site identity revalidation → dispatch → poll → smoke → final evidence upload with post-upload `cmp`. `bootstrap-alpha` correctly short-circuits before any index mutation.
- **Site reconciliation**: the extracted `verify_site_workflow_identity.sh` is called twice (pre-publication and pre-dispatch) and is *stronger* than the previous inline code — the dispatch-time check now also verifies the pinned workflow bytes, which it did not before.
- **Public smoke**: `run_schema_bootstrap_public_smoke.sh` refuses to run with `SIFR_TEST_CHANNEL_METADATA_PATH` set, byte-matches the live index and both dispatchers, requires the stable dispatcher to fail with the governed GA-metadata message, installs from the real endpoint, and pins `${version}:beta:false`. All four `<id>.txt` filenames match the `SMOKE_IDS` the materializer reads.

### Gates re-run at this exact head
| Gate | Result |
|---|---|
| `distribution_release --suite epoch-bootstrap` | PASS (1/1) |
| `distribution_release --suite representative` | PASS (52/52), new `schema_epoch_bootstrap_workflow_contract` discovered and passing |
| `distribution_release` full+qualification+evidence-custody+incident-governance+epoch-bootstrap | PASS (58/58: full 54, +4) |
| Combined with representative | 52 + 58 = **110**, reconciling the prior 110/110 and merge-selection 55/55 figures exactly |
| `sifr_verify --self-test` | PASS (all 11 sections) |
| `coverage_matrix` | PASS (5/5) |
| `documentation --suite structure` | PASS |
| file-size guardrail + self-test | PASS (2898 files); `release-publication.yml` 795/900, `schema_bootstrap.py` 498, selftest 574, prepare 219 |
| `git diff --check`, `bash -n` on all changed shell, `compileall` | clean |
| `resolve-publication-approvers` behavioral probe | fail-closed as specified |

### Notes (not findings)
- `- [x] Add the single protected publication workflow and production site adapter.` is checked while `ga-activation`/`normal`/`rollback`/`incident-roll-forward` and the `stable-release-drill` job remain unimplemented. The surrounding ledger prose and `distribution_pipeline.md` both state explicitly that those mutations are gated to later slices, and the next two checklist items are unchecked, so no reader is misled — bookkeeping granularity, not a false claim.
- `expect_failure` in `schema_bootstrap_selftest.py:568` catches `OSError` alongside `GovernanceError`; for the missing-smoke case the rejection is in fact an `OSError` from `sha256_file`. Still fail-closed, and the producer path is the one under test.
- Downstream live configuration (`stable-release` reviewers with prevent-self-review, `preview-release` environment creation, `SIFR_WEBSITE_ACTIONS_TOKEN`, confirming the live 105-byte asset at dispatch time) is post-merge operational, per scope.

No actionable finding at this head.

VERDICT: SATISFIED
