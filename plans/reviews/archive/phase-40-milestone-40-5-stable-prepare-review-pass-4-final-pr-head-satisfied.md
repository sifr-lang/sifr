## Head identity

| Location | SHA |
|---|---|
| local `HEAD` | `8d81040e597cb3d64f90788fb2ce2e822eb236f1` |
| `origin/codex/phase-40-milestone-40-5-stable-publication` | same |
| PR #3043 `headRefOid` (OPEN, MERGEABLE, base `main`) | same |

Worktree clean except one untracked file (`plans/reviews/active/…pass-4-final-pr-head.md`), which is not tracked and not in the PR. No files modified. The branch diff contains only `plans/reviews/archive/**` review artifacts — zero `plans/reviews/active/**` entries.

## Delta since pass 3 — documentation only

`55c6d960c…HEAD` is exactly one commit, `8d81040e5 docs(release): archive stable prepare review`, 2 files / +45 / −0:

- **New** `plans/reviews/archive/…stable-prepare-review-pass-3-frozen-pr-head-satisfied.md` — records head `55c6d960c4ea29b7b945df88d72573a6008c9651` in all three rows, verdict `SATISFIED`, and the validation it actually ran. I spot-verified its factual claims rather than trusting them: `artifact_index.py` expiry/one-to-one binding and the `!= 6` upload check, `release-publication-prepare.yml:335` `.expires_at` compared to the index value before the `/zip` fetch at `:342`, `extract_github_artifact.py` summing `file_size` and comparing to `--expected-uncompressed-bytes` before the write loop, `stable_prepare_selftest.py:306-324` asserting non-zero exit **and** empty destination, `_require_publication_window` at `:566-577`, and "2 commits, 32 files" (33 today minus the one file this commit added) and "largest new file 621" (`stable_prepare.py`). All accurate.
- **Ledger** `plans/issues/active/phase-40-stable-channel-ga-execution.md` — six lines attributing the review to `55c6d960c…`. It does not claim this documentation-inclusive head was revalidated, and no pass-4 entry is pre-written. The publish/resume/rollback checklist items remain unchecked, matching the fact that the stable path is not yet reachable from `release-publication.yml`.

## Recheck of the complete PR

- **Fail-closed provenance** — `case` default still rejects unknown `governance_mode`; `ga-activation`/`normal` skip the preview steps and hit the stable gate, which requires `channel == stable`, a 40-hex evidence commit, canonical candidate path, 64-hex plan digest, `initial|resume`, and positive generation, else `exit 2`. The prepare workflow is `workflow_call`-only; the sole caller passes none of the five new inputs, so `ga-activation` fails prepare and `publish` (`needs: prepare`) is skipped — fail-closed, not silently permissive.
- **Credentials/permissions** — `permissions: actions: read, contents: read`; `persist-credentials: false` on both checkouts; no `secrets:`; contract case forbids `contents: write`, `packages: write`, `id-token: write`, `${{ secrets.`, `vsce publish`, `gh release upload`, `unzip`. `materialize_stable_prepare` writes only via `write_canonical_json(refuse_existing=True)`.
- **Safe extraction** — rejects absolute/`..`/backslash names, non-regular members, duplicates, post-resolve escapes, and pre-existing targets; writes with `"xb"`; byte-count check precedes any write. Combined with `verify_transported_artifacts` re-hashing every file, extra or substituted ZIP members cannot survive.
- **Expiry binding** — one canonical expiry per governed upload in the index, matched exactly against the authoritative `/actions/artifacts/{id}` response, with `workflow.expires_at` as a lower bound and the ≥7-day window enforced in the validator. No forward-forgeable window remains.
- **Schema/validator coherence** — `propertyNames` enum + `minProperties: 20` pins exactly the 20 identities; the relative `$ref` to `stable_index_mutation_evidence.schema.json` resolves; the fixture is derived from `qualification_index()` and passes both the JSON Schema and `validate_stable_prepare_summary`; lint count updated 15 → 16.
- **`Bind prepare outputs`** — `// ""` keeps the preview summary safe; every value is a validator-constrained sha256, so no `GITHUB_OUTPUT` injection path. The pass-3 nonblocking note about `jq` inside command substitution remains unreachable and non-actionable.
- **Compatibility/scope** — the two existing contract cases were updated for the `steps.summary` → `steps.outputs` rename; suite registered in merge/nightly/release profiles, coverage matrix, and `REQUIRED_SUITES`. No `crates/`, no demos, no Rust-interop implementation, no performance work. `selftest.py` refactor leaves no orphaned imports (`tempfile`, `json` still used).

## Validation reproduced at this head

stable-prepare 6/6 · `full`+`stable-prepare` **60 variants, 0 failures** · `representative` 54 variants, 0 failures · all three workflow-contract cases exit 0 · `sifr_verify --self-test` all 11 lanes pass · coverage matrix 5/5 (`profile assignment matrix ok: rows=17`) · file-size guardrails **PASS** (2908 files, limit 900) · `git diff --check` clean.

## Findings

None actionable.

VERDICT: SATISFIED
