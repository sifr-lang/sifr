# Review — Phase 40 / milestone 40.5 stable publish-primitives wave

Scope inspected: 17 modified files + 7 untracked, `HEAD = da7c38fb15dbebe11b1e9be943f4d080b8e7bafc`. No files modified.

## What I verified as correct

**Generation allocation** (`governance/generation.py`, `scripts/distribution/allocate_release_generation.py`). `max(live ∪ every retained snapshot) + 1` with burned gaps preserved. I probed every rejection branch directly and all fail closed with governed errors: name/payload disagreement, non-matching name, directory entry, symlink entry, symlinked root, missing root, missing live snapshot. The new live-index-equals-retained-snapshot requirement is a genuine strengthening over the bash allocator at `release-publication.yml:326-356`, and it is satisfiable in production: `release-publication.yml:602-614` does `cp publication/channels.json channels-generation-N.json` before the `--clobber` replace, so live and its snapshot are byte-identical in steady state, including after the #3040 bootstrap (gen 1). It also closes the silently-empty-process-substitution hole a prior review flagged: an empty `prepare/history` now fails closed rather than yielding `live+1`.

**Exact-ID refetch** (`scripts/distribution/fetch_qualification_artifacts.py`). Probed with a fake `gh` on PATH: wrong repository, wrong source commit, pre-existing output, missing output parent, `conclusion != success`, `run_attempt` mismatch, and same-size byte tampering are all rejected, with **no leftover temp directory and no partial `stable-assets`** — output publishes only via `staging.rename(output_root)` after `verify_transported_artifacts` re-hashes all 20 files. Extraction is bounded before any write (`extract_github_artifact.py:43-44` sums `file_size` and compares to the index total, then writes with `"xb"`). Grouping by `workflow_artifact_id` and using `entries[0]`'s name/expiry is safe because `artifact_index.py:167-197` enforces one-to-one id↔name and one canonical expiry per upload; `expires_at` string equality works because `collect_qualification_artifacts.py:142-143,271` records the API value verbatim.

**Revalidation** (`scripts/distribution/revalidate_stable_publication.py`). Reusing `summary["mutation"]["proposed_index"]["generation"]` is necessary for byte equality and is not a hole: `materialize_stable_mutation` re-binds `expected_generation`/`expected_sha256` against the freshly-read live index, so any live-index drift fails. I confirmed the byte-inequality gate works by tampering `artifacts.vsix.size_bytes` (rejected: "did not reproduce the prepare summary") and the protected-input gate by flipping `mode`.

**Workflow** (`release-publication-prepare.yml`). `actions: read`/`contents: read`, no `secrets:`, `persist-credentials: false` on all three checkouts, `GH_TOKEN: github.token` only. Line 277 requires `channel`/`version`/`source_commit`/`bootstrap_alpha_version` to be **empty** for the stable path and derives `candidate_version` from `CANDIDATE_PATH##*/` and `source_commit` from the digest-bound plan (`:292-293`) — requirement 1 met. `mkdir -p prepare` precedes `mkdir prepare/history`; `proposed_generation="$(…)"` fails the step under `set -e`; `__pycache__` written into `stable-source` is gitignored so `_require_checkout`'s `--untracked-files=all` stays clean. Preview/bootstrap branch untouched in behavior; `Bind prepare outputs` `// ""` keeps it safe.

Suite registration is coherent across manifest / runner (including the dedup flag) / three profiles / coverage matrix / `REQUIRED_SUITES` / `schema_contracts` / `selftest` / `release_evidence_selftest` / `qualification_fixture`. Suite passes 3/3; the prepare workflow contract case exits 0. No `crates/`, demo, or Rust-interop work. Largest new file 287 lines.

---

## Findings

### 1. MEDIUM — durable docs assert a protected-publish integration that does not exist in this wave

`internal_docs/distribution_pipeline.md:611-617` states the fetcher "is used by read-only prepare **and protected publish revalidation**" and that "Protected revalidation recomputes the complete stable-prepare summary from **fresh evidence/source checkouts, fresh artifact downloads**, and the live index … before a production adapter may mutate anything." `plans/releases/README.md:21-26` states "The read-only prepare **and protected publish jobs** refetch the six qualification uploads … Publish may proceed only when its recomputed canonical summary equals the exact 30-day review artifact."

None of that is true at this head. `grep` shows `revalidate_stable_publication.py` has **zero** references outside its own file and the selftest — no workflow calls it. It performs no artifact download: it consumes `--artifact-root` as an input and never invokes the fetcher. It does not create the checkouts; it only asserts the ones it is handed are clean and at the exact commits. There is no protected publish job.

The wave brief says this explicitly: "intentionally does not expose GA/normal production dispatch yet … the next PR will wire [these] into the existing single protected publish job." The issue-plan ledger (`plans/issues/active/phase-40-stable-channel-ga-execution.md:580-586`) gets this right ("*adds a protected revalidation command that recomputes and byte-compares*", publish checklist items left unchecked). The two durable docs are the outliers, and they are the artifacts an operator or future reviewer treats as the contract.

Fix: scope both paragraphs to what exists — prepare uses the fetcher and the allocator; a reusable revalidation command exists that recomputes and byte-compares a summary against caller-supplied clean checkouts, live index, and artifact root — and defer the publish-job claims to the wiring PR.

### 2. MEDIUM — the revalidation primitive's central gate, and most enumerated rejections, are unexercised

`stable_publication_primitives_selftest.py` has 3 tests with 3 negative cases total, against roughly fifteen rejections the scope enumerates as requirements.

Most importantly, `test_protected_revalidation:156-170` only exercises the `expected_summary_sha256` mismatch branch (`revalidate_stable_publication.py:43-44`). The **byte-equality gate at `:73-76` — the entire reason this primitive exists — has no coverage**, and neither does the protected-input mismatch at `:49-59`. I confirmed both work by hand (tampered `artifacts.vsix.size_bytes`; flipped `mode`), so this is adequacy, not correctness: nothing in CI would catch a future refactor that turned `canonical_json_bytes(recomputed) != summary_bytes` into a no-op.

Also uncovered: in `generation.py`, only the missing-snapshot branch is tested (`:72-80`) — name/payload disagreement (`:42-45`), invalid name (`:35-37`), unsupported entry (`:33-34`), non-directory root (`:27-28`), and same-generation-different-bytes (`:47-51`) are not. In the fetcher, only the artifact `expires_at` mismatch is tested (`:121-135`) — repository/source-commit identity (`:46-49`), run conclusion/attempt (`:60-66`), pre-existing output (`:50-51`), and transported-hash mismatch are not.

Given the whole wave is "reviewed primitives" whose value is fail-closed behavior, at minimum add the revalidation byte-inequality and protected-input cases plus generation name/payload and content-drift.

Nonblocking sub-item: the contract case adds `"source_commit:"` to the fragment list but never asserts the new `version:` or `proposed_generation:` job outputs. `"version:"` would be a vacuous assertion anyway (it is a substring of `bootstrap_alpha_version:`), so assert the full `      proposed_generation: ${{ jobs.prepare.outputs.proposed_generation }}` lines instead.

### 3. LOW — `preview` mode lost its only guard on `channel`

`channel`, `version`, and `source_commit` were relaxed from `required: true` to `required: false, default: ""` (`:6-17`) so the stable path can require them empty. The preview branch compensates for two of the three — `source_commit` is checked 40-hex and `== git rev-parse HEAD` (`:144-148`), `version` is effectively pinned by the artifact-set completeness check (`:155-160`) — and `bootstrap-alpha`/`bootstrap-index` still pin `CHANNEL` to `alpha`/`beta` (`:127,133`). But `case preview)` (`:120-125`) checks only that `BOOTSTRAP_ALPHA_VERSION` is empty, so `preview` now accepts an empty or arbitrary `channel` that flows unvalidated into `prepare/summary.json` (`:241`) — the reviewer-visible artifact. Unreachable through the sole caller (`release-publication.yml:16-19` keeps `channel` `required: true`), so this is a strictness regression rather than a live bypass. One line: `[[ "${CHANNEL}" =~ ^(alpha|beta)$ ]]` in the `preview)` arm.

### 4. LOW — `revalidate_stable_publication.py` crashes instead of emitting a governed diagnostic on a missing summary

`require_sha256` runs first, then `sha256_file(prepare_summary_path)` at `:43` — and `common.sha256_file` does not wrap `OSError`. Verified: a missing `--prepare-summary` produces an unhandled `FileNotFoundError` traceback and exit 1, versus the sibling `allocate_release_generation.py`, which returns the governed exit 2 for an unreadable `--live-index` because `load_json_strict` catches `OSError`. Both fail closed under `set -e`, so this is diagnostic quality — but it is an inconsistency inside one wave. Reading the bytes once and hashing them (`sha256_bytes(summary_bytes)`) fixes this and removes the hash-then-reread window at `:43-45` in the same edit.

---

## Nonblocking suggestions

- **Revalidation cannot detect a generation burned after prepare.** It takes no `--snapshot-root` and never re-runs the allocator, so a summary whose generation was reserved by a concurrent attempt that died before the `--clobber` (live bytes unchanged, snapshot N+1 now present) still revalidates. Backstopped by the duplicate-snapshot guard at `release-publication.yml:610-612`, which fires before the first mutation — so it is a wasted-run, not a corruption. Passing `--snapshot-root` and re-asserting equality would move the check earlier.
- **Unbounded in-memory artifact download.** `_gh_bytes` (`:135-147`) buffers a whole ZIP in RAM before `write_bytes`; sysroot uploads can be large. The exact expected uncompressed total is already known from the index before the fetch, so the download could be streamed and bounded.
- **`gh release view --json assets` completeness.** `:353-357` (and the pre-existing `release-publication.yml:352-355`) assume the inline asset list is complete. The channels release accumulates one snapshot per generation permanently. Truncation is backstopped in both directions (dropping the live snapshot → allocate fails; dropping a burned higher one → duplicate-snapshot guard fails), so the worst case is a hard stop mid-GA rather than reuse. `gh api --paginate /repos/{repo}/releases/{id}/assets` removes the assumption.
- **First `verification/ → scripts.distribution` import in the repo** (`stable_publication_primitives_selftest.py:13-19`). Because the scripts prepend `AREA_ROOT` and import top-level `governance` (the established convention for all 11 `scripts/distribution/*.py`), the selftest process now loads the governance package **twice**: I confirmed `verification.areas.distribution_release.governance.common` and `governance.common` coexist and their `GovernanceError` classes are distinct. Handled correctly here via the `ScriptGovernanceError` alias, but worth a comment — the wiring PR's caller must catch both, or the scripts should import via the qualified path.
- `fetch_qualification_artifacts.py:71-72` (`len(uploads) != 6`) is unreachable: `artifact_index.py:249-250` already enforces exactly six uploads before this runs. Harmless defense-in-depth; keep or drop deliberately.
- `generation.py:49` re-hashes the live index once per snapshot; hoist it out of the loop.
- `plans/reviews/active/phase-40-milestone-40-5-stable-publish-primitives-review-pass-1.md` is 0 bytes. Presumably the target for this pass, but it should not land empty.
- The CLI surface is split three ways: `prepare-stable-publication` is a `release_governance.py` subcommand while allocation and revalidation are standalone scripts. Consistency would help the wiring PR.

Ledger claims spot-checked: `da7c38fb1` resolves to `da7c38fb15dbebe11b1e9be943f4d080b8e7bafc` ✓; `stable_gate_inventory.json` accurately reflects the release profile ✓; the archived pass-4 review describes PR #3043's head, not this one, and is internally consistent ✓.

VERDICT: NOT SATISFIED
