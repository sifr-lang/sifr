## Review — Phase 40 / milestone 40.5 stable publish-primitives, pass 3

Scope: 17 modified + 8 untracked files at `HEAD = da7c38fb15dbebe11b1e9be943f4d080b8e7bafc`. No files modified.

### Pass-1 / pass-2 closures verified

**Docs and ledger state the true unwired state.** `internal_docs/distribution_pipeline.md:614-619` and `plans/releases/README.md:20-27` both scope revalidation to a caller-supplied primitive and say the production workflow does not yet invoke it. `revalidate_stable_publication.py` still has no caller outside the selftest, downloads nothing, and creates no checkouts — consistent. The publish checklist items (`plans/issues/active/phase-40-stable-channel-ga-execution.md:613-617`) remain unchecked. Ledger line 596-597 now correctly describes the protected-input check as defense-in-depth subsumed by byte equality (pass-2 finding 1 closed), and 606-612 accurately describes the pass-2 remediation.

**Generation allocation.** `generation.py:30-57` preserves burned gaps (`max(live ∪ retained) + 1`), validates every snapshot canonically, and requires live ≡ its retained snapshot with the digest hoisted out of the loop. Mutation-verified load-bearing: live-snapshot equality, name/payload disagreement, invalid snapshot name, and `max(generations)+1` → `live+1` all fail the suite.

**Exact-ID refetch.** `fetch_qualification_artifacts.py` verifies repository/source identity, run id/attempt/head_sha/conclusion, per-upload id/name/`expired`/`expires_at`/`workflow_run.id`/positive size, the exact compressed length against the authoritative `size_in_bytes`, the exact uncompressed total via `extract_artifact`, and every transported SHA-256 — then publishes only via `staging.rename(output_root)`. Mutation-verified load-bearing: exact-size gate (pass-2 finding 2 closed, both the truncated `size+1` and overlong `size-1` fixtures reject with `not rejected_*.exists()`), `expired`, expiry identity, artifact-run custody, repository identity, conclusion, attempt, pre-existing output, output-parent symlink, `verify_transported_artifacts`, uncompressed bound. `source_commit` vs `head_sha` are individually redundant but jointly load-bearing (confirmed by neutralizing both).

**Deadlock and cleanup.** `_gh_to_file:152-187` now redirects stderr to a `tempfile.TemporaryFile` and reads it after the transfer, so a full stderr pipe cannot block `gh`'s stdout writes (pass-2 finding 3 closed). Every exit path — overrun kill, `BaseException`, nonzero exit, size mismatch — kills the child, waits, and unlinks the partial archive; the archive/staging tree lives in a `TemporaryDirectory` on the same filesystem as `--out`, so no partial `stable-assets` can survive.

**Revalidation.** Single read → `sha256_bytes` → `load_json_bytes_strict` on the same bytes (`:46-58`), governed diagnostic on an unreadable summary, burned-generation detection via `allocate_next_generation` (`:71-78`), and byte reproduction of the reviewer-visible summary (`:91-94`). All four mutation-verified load-bearing.

**Preview and least privilege.** `release-publication-prepare.yml:121-125` pins `preview` to `alpha|beta`; `bootstrap-alpha`/`bootstrap-index` unchanged; the only reachable caller chain (`preview-release.yml:78,194` → `release-publication.yml:67`) restricts channel to alpha/beta and never passes stable inputs or the removed `proposed_generation` input, so the reusable-workflow input contract stays valid. `permissions: actions: read / contents: read`, zero `secrets` references, `persist-credentials: false` on all three checkouts. `candidate_version` and `source_commit` derive from the digest-pinned plan and are regex-validated before reaching `ref:`/`GITHUB_OUTPUT`.

**Registration and outputs.** `stable-publish-primitives` is coherent across manifest, runner (including dedup), merge/nightly/release profiles, coverage matrix, `REQUIRED_SUITES`, `schema_contracts`, `selftest`, `release_evidence_selftest`, `qualification_fixture`, and the gate inventory; a repo-wide `stable-prepare` sweep found no list left unupdated. I confirmed against a real fixture summary that all three new job outputs are non-empty (`version=0.1.0`, `source.commit`, `mutation.proposed_index.generation=8`) and that both jq fallbacks yield `""` without error on the preview summary shape.

**Validation reproduced.** stable-publish-primitives 3/3; `full` + `stable-publish-primitives` = 61 variants, 0 failures (dedup holds); coverage_matrix 5/5; prepare workflow contract exit 0; ruff `E,F,I,UP,RUF` clean on the five Python files; `git diff --check` clean; no trailing whitespace in untracked files; file-size guardrails PASS (2913 files, limit 900). No `crates/`, demo, or Rust-interop changes.

### Findings

None actionable.

### Nonblocking

- **Residual unpinned hardening gates.** Mutations that survive the suite: `generation.py:34` (symlink/non-file snapshot entry), `:27` (non-directory snapshot root), `:41` `require_canonical=True` for snapshots, and `fetch_qualification_artifacts.py:43` `require_unexpired=True`. All are fail-closed path/format checks unreachable in a fresh CI download directory, and canonical-JSON rejection for release indices is covered elsewhere in the governance suite — but a refactor that dropped them would go unnoticed.
- **Overrun branch is subsumed, not independently pinned.** Neutralizing `fetch_qualification_artifacts.py:164` alone keeps the suite green because the exact-size gate at `:183` catches the same fixture. The ledger's claim ("rejects both truncated and overlong downloads") is accurate; only the streaming disk bound itself is untested black-box.
- **Peak disk grew.** The old bash path did `rm "${archive}"` after each extraction; the fetcher keeps all six ZIPs in `archives/` until the loop ends, so peak usage is roughly all six archives plus their extracted contents. Comfortably within a runner today; deleting each archive after extraction restores the old profile.
- `plans/reviews/active/phase-40-milestone-40-5-stable-publish-primitives-review-pass-3.md` is 0 bytes — same class pass-1 and pass-2 flagged for their own active files. It should carry this review before landing.
- The ledger records no `scripts/run_all_tests.sh --profile create-pr` result for this wave yet (the stable-prepare wave recorded one at `:549-553`); the required-workflow gate still has to run before the PR.
- `fetch_qualification_artifacts.py:71-72` (`len(uploads) != 6`) remains unreachable behind `artifact_index.py:263`; retained deliberately per the ledger.
- The publish job's own history enumeration (`release-publication.yml:351-354`) still uses non-paginated `gh release view --json assets` while prepare now paginates. Pre-existing, out of diff, and fail-closed via the write-once snapshot guard.
- CLI surface is still split three ways (`prepare-stable-publication` as a `release_governance.py` subcommand; allocation and revalidation as standalone scripts).
- Unverifiable locally, as in pass 2: that a real GitHub artifact's `size_in_bytes` exactly equals the `/zip` response length. It is the one gate that hard-fails prepare if GitHub's accounting ever diverges.

Ledger cross-checks: all three archived review filenames match the files on disk; `da7c38fb1` resolves to the stated full SHA; `stable_gate_inventory.json:262-268` matches the release profile.

VERDICT: SATISFIED
