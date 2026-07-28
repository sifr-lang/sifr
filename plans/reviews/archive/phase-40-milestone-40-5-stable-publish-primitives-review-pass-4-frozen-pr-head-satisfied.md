## Identity — verified

| Location | SHA |
|---|---|
| local `HEAD` | `f355a2b0a40a4ab644f711d0e6fd6d2aa63bf19a` |
| `origin/codex/phase-40-milestone-40-5-stable-publish` | same |
| PR #3044 `headRefOid` (OPEN, MERGEABLE, base `main`) | same |

Worktree clean except untracked `plans/reviews/active/…pass-4-frozen-pr-head.md` (this review's own target, not in the PR). No files modified by me. Diff vs `origin/main` (merge-base `aa68199c4`): 26 files, +1435/−104 — workflow, 3 new scripts, `generation.py`, the new selftest, suite/profile/report/matrix registration, and docs. No `crates/`, demo, or Rust-interop content.

## What I verified independently

**Generation history.** `governance/generation.py:30-57` returns `max(live ∪ every retained snapshot) + 1`, validates each `channels-generation-<N>.json` canonically, requires name≡payload, and requires the live index to be byte-identical to its retained snapshot — so a burned reservation stays burned and a silently-empty `prepare/history` fails closed rather than yielding `live+1`. Production-satisfiable: `release-publication.yml:564-614` uploads `channels-generation-${PROPOSED_GENERATION}.json` copied from `publication/channels.json` *before* the `--clobber` replace, for every non-`bootstrap-alpha` mode, so live ≡ snapshot in steady state including post-bootstrap gen 1.

**Exact artifact custody / atomicity.** `fetch_qualification_artifacts.py` checks repository, `source_commit`, run `id`/`run_attempt`/`head_sha`/`conclusion`, then per upload `id`/`name`/`expired`/`expires_at`/`workflow_run.id`/positive size, streams `/zip` to an exact `size_in_bytes` boundary (kill + unlink on overrun, unlink on every failure path, stderr to a file so a full pipe can't deadlock), bounds the uncompressed total in `extract_artifact` before any write, re-hashes all 20 transported files, and publishes only via `staging.rename(output_root)` from a `TemporaryDirectory` on the same filesystem. No partial `stable-assets` is reachable.

**Byte-for-byte protected revalidation.** `revalidate_stable_publication.py` reads once → `sha256_bytes` → `load_json_bytes_strict` (no read/hash race, governed diagnostic on an unreadable file), re-runs the allocator to detect a generation burned after prepare, and requires `canonical_json_bytes(recomputed) == summary_bytes`.

**Workflow.** `permissions: actions: read / contents: read`, no `secrets:`, `persist-credentials: false` on all three checkouts, `GH_TOKEN: github.token`, no mutation (only `upload-artifact` with `overwrite: false`, `retention-days: 30`). The stable arm now requires `channel`/`version`/`source_commit`/`bootstrap_alpha_version` to be **empty** (`:278`) and derives `candidate_version` from the regex-validated `CANDIDATE_PATH` and `source_commit` from the digest-pinned plan (`:288-298`) before either reaches `ref:` or `GITHUB_OUTPUT` — so the removed `test "${CHANNEL}" = "stable"` is compensated, and `stable_prepare.py:286-287` still requires an exact stable version in the summary. The removed `proposed_generation` input is not passed by the sole caller (`release-publication.yml:62-71`), so the reusable contract stays valid; the relaxed `channel` is re-pinned to `alpha|beta` in the `preview)` arm (`:121`).

**Test load-bearingness — my own source mutations, each caught by the suite:** `revalidate:91` byte equality → `if False`; `fetch:183` `written != expected_bytes` → `if False`; `verify_transported_artifacts(...)` → `None`; `generation.py:57` `max(generations)+1` → `live+1`; `generation.py:53` live-snapshot equality → `if False`; `generation.py:43` name/payload disagreement → `if False`.

**Registration and gates.** `stable-publish-primitives` is coherent across manifest, `runner.py` (including the dedup flag), merge/nightly/release profiles, coverage matrix, `REQUIRED_SUITES`, `schema_contracts`, `selftest`, `release_evidence_selftest`, `qualification_fixture`, and `stable_gate_inventory.json`. Reproduced: `--suite full --suite stable-prepare --suite stable-publish-primitives` → **61 variants, 0 failures**; `--suite full` alone → also 61 (dedup holds); prepare workflow contract case exit 0; coverage_matrix 5/5; file-size guardrails PASS (2918 files, limit 900; largest new file 552). `git diff --check` clean.

**Documentation truthfulness.** `internal_docs/distribution_pipeline.md:607-619` and `plans/releases/README.md:20-27` both scope revalidation to a caller-supplied primitive and state the production workflow does not yet invoke it — confirmed: `revalidate_stable_publication.py` has zero references outside its own file and the selftest, downloads nothing, creates no checkouts, and the publish checklist items stay unchecked. The three archived pass 1–3 artifacts match the ledger's filenames and verdicts, and the ledger's remediation and coverage claims (including the corrected "defense-in-depth subsumed by byte equality" wording) match the code at this head.

## Findings

None actionable.

## Nonblocking suggestions

- **`size_in_bytes` ≡ `/zip` response length** remains the one gate in the wave that is unverifiable locally and hard-fails prepare if GitHub's artifact accounting ever diverges from the delivered ZIP length (`fetch_qualification_artifacts.py:183`). Third consecutive pass carrying this.
- **Residual unpinned fail-closed checks**: `generation.py:27` (non-directory root), `:34` (symlink/non-file entry), `:41` `require_canonical=True` for snapshots, `fetch:43` `require_unexpired=True`, and the streaming overrun branch at `fetch:164` (subsumed by the exact-size gate). A refactor dropping any would go unnoticed.
- **Empty `snapshot_root`** (no retained snapshots at all) is untested; it fails closed via the live-snapshot requirement, but a one-line case would pin it.
- **Peak disk** grew: all six ZIPs are retained in `archives/` until the loop ends, versus the old per-iteration `rm`.
- `fetch:71-72` (`len(uploads) != 6`) is unreachable behind `artifact_index.py`; retained deliberately per the ledger.
- The `.source.commit // .source_commit` output binding (`release-publication-prepare.yml:424`) legitimately spans the stable and preview summary shapes rather than being a dead fallback — a short comment would prevent a future reader reading it as one.
- CLI surface is still split three ways (`prepare-stable-publication` as a `release_governance.py` subcommand; allocation and revalidation as standalone scripts); worth unifying in the wiring wave.
- The nine legacy `scripts/distribution/*.py` still prepend `AREA_ROOT` and import top-level `governance`, so a process loading both families gets two `GovernanceError` identities. The three new scripts are consistently qualified (I confirmed the single identity by mutation).
- `plans/reviews/active/…publish-primitives-review-pass-2.claude.log`, `…pass-4-frozen-pr-head.md`, and `…pass-4-frozen-pr-head.claude.log` are 0 bytes in the working tree (untracked, not in the PR); the pass-4 file should carry this review before the archive update.

VERDICT: SATISFIED
