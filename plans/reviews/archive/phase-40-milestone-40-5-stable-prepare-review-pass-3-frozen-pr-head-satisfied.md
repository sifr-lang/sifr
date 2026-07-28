All three heads match the frozen SHA. I reviewed the complete `origin/main...HEAD` diff (2 commits, 32 files) and reproduced the validation set locally.

## Head verification

| Location | SHA |
|---|---|
| local `HEAD` | `55c6d960c4ea29b7b945df88d72573a6008c9651` |
| `refs/heads/codex/phase-40-milestone-40-5-stable-publication` (origin) | same |
| PR #3043 `headRefOid` (state OPEN, MERGEABLE, base `main`) | same |

Working tree clean except one untracked file (`plans/reviews/active/…pass-3-frozen-pr-head.md`), which is not part of the PR. No files modified.

## Post-pass-2 hardening — each item confirmed

- **One canonical expiry per governed upload** — `artifact_index.py:199-206` adds `workflow_id_to_expiry` and rejects any second entry of the same `workflow_artifact_id` with a differing `expires_at`; pinned by a new mutation case (`selftest.py:659-674`, artifacts[1] expiry drift, plus artifacts[4] id-collision). Combined with the pre-existing "must not expire before the workflow qualification boundary" check, `workflow.expires_at` is a lower bound on API-verified per-artifact expiries — so the seven-day window (`stable_prepare.py:566-577`) can no longer be forged forward.
- **Exact API expiry comparison before download** — `release-publication-prepare.yml:334` now compares `.expires_at` from the authoritative `/actions/artifacts/{id}` response against the index value, alongside `.id`, `.name`, `.expired == false`, `.workflow_run.id`, and only then fetches `/zip` (`:342-345`). Contract-pinned (`stable_publication_prepare_workflow_contract.sh:26-27`).
- **Schema fixture** — `schema_contracts.stable_publication_prepare()` is now derived from `qualification_index()` (real names/digests/6 upload ids, `vsix_sha256` = transported VSIX). I verified it passes both `validate_instance(..., stable_publication_prepare.schema.json)` and `validate_stable_prepare_summary` (checked directly). I also confirmed the relative `$ref` to `stable_index_mutation_evidence.schema.json` genuinely resolves (an empty `mutation` is rejected) and the 20-identity `propertyNames`+`minProperties: 20` pair is enforced.
- **ZIP byte-count mismatch rejects before writing** — `extract_github_artifact.py:29-44` plans every member, then sums `file_size` and compares to `--expected-uncompressed-bytes` *before* the write loop at `:45`. New self-test fixture (`stable_prepare_selftest.py:306-324`) asserts non-zero exit **and** `not any(destination.iterdir())`.

## Requirement recheck (spot-verified, not carried over)

Credential-free/mutation-free (`permissions: actions: read, contents: read`; no `secrets:`; `persist-credentials: false` on both checkouts; `materialize_stable_mutation` returns without writing — `stable_planner.py:58-116`); exact evidence + source commits with `submodules: recursive` and clean-checkout enforcement (`stable_prepare.py:451-457`); exact successful run/attempt (`/attempts/{n}` with `id`/`run_attempt`/`head_sha`/`conclusion == success`); 6 uploads × 20 artifacts (`artifact_index.py:263-264`, `EXPECTED_ARTIFACT_IDS` = 4 singletons + 4 kinds × 4 targets); write-once (`retention_days == 30`, `overwrite is not False`); ≥7 full days; deterministic schema-v2 summary via `write_canonical_json(refuse_existing=True)` (double-run test at `stable_prepare_selftest.py:241-249`); exact Marketplace binding through `validate_marketplace_publish_plan` (`editor_qualification.py:95-130`) plus `vsix_sha256 == artifacts["vsix"].sha256`; site pinned to `sifr-lang/sifr-website`; 30-day `overwrite: false` summary upload (`:428`). Scope is Phase 40 release/distribution only — no `crates/`, no demos, no Rust-interop implementation.

**No preview/bootstrap regression:** the sole caller (`release-publication.yml:62-71`) passes none of the five new inputs, and the preview path rejects them outright (`:107-110`). The `Bind prepare outputs` step is unconditional and uses `// ""`, which is safe on the preview summary (missing keys index to `null`, not an error) — I confirmed the preview summary shape at `:220-241`. `ga-activation`/`normal` were previously rejected by the `case` default and now fail-closed at the stable regex gate, with `publish` (`needs: prepare`) skipped either way.

## Validation reproduced at this head

stable-prepare selftest **6/6** · governance **14/14** · combined `full`+`stable-prepare` **60/60**, 0 failures · all three workflow-contract cases exit 0 · `sifr_verify --self-test` all lanes pass (schema lint count 16) · coverage matrix 5/5 (`profile assignment matrix ok: rows=17`) · file-size guardrails PASS (2908 files, limit 900; largest new file 621) · `git diff --check` clean. Python lint is not a repo lane (no first-party `ruff` config; `ruff` here is the parser submodule) — I checked the refactored modules for orphaned imports instead and found none.

## Findings

None actionable.

Nonblocking, no change required:
- `release-publication-prepare.yml:415-421` — a `jq` failure inside `echo "k=$(jq …)"` cannot fail the step under `set -e`, so a malformed summary would silently yield an empty output rather than aborting. Unreachable in practice: `validate_stable_prepare_summary` already ran inside the producer, and the gating identity checks at `:387-392` use assignment/`test` forms that do abort.
- The artifact API exposes no attempt number, so attempt-granular artifact provenance rests on the index binding plus commit-qualified write-once upload names — the strongest available given the API.
- Carried from pass 2 and still open: the candidate-evidence file set is duplicated across `stable_prepare.py:49-58` and `evidence_custody.py:180-200`, and `mode` (`initial`/`resume`) is recorded without behavioral effect until the publish job lands (the corresponding checklist items in `plans/issues/active/phase-40-stable-channel-ga-execution.md` remain unchecked, so the docs are truthful).

VERDICT: SATISFIED
