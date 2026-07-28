# Phase 40 / Milestone 40.5 — stable-publication prepare wave review

**Important process note first:** the working tree changed **three times underneath me during this review** (`release-publication-prepare.yml` went 182 → 188 → 192 → 199 insertions; `extract_github_artifact.py` gained `--expected-uncompressed-bytes`; `artifact_index.py` was not modified at review start and is now modified). I re-read every volatile file and re-ran every test against the **final** state. This verdict applies to exactly these bytes:

```
7006d9655df2c2d9  .github/workflows/release-publication-prepare.yml
97330224cdc7247c  scripts/distribution/extract_github_artifact.py
b258fe353e3470ae  verification/areas/.../governance/artifact_index.py
15f08d91ca9219fe  verification/areas/.../governance/stable_prepare.py
```

## What I verified (not just "tests pass")

**Fail-closed shell/expression correctness.** Anchored regexes on all five new inputs, `set -euo pipefail`, `exit 2` inside `while … done < <(…)` correctly terminates the step (process substitution, not a pipe, so the loop body runs in the current shell). Unknown `governance_mode` still falls into the preview branch's `*) unsupported governance_mode; exit 2`. The un-`if`'d "Bind prepare outputs" step is safe because `test -f prepare/summary.json` fails closed if neither branch ran.

One thing I specifically checked because it is a classic inverted-guard bug: `expired` uses `jq -r`, not `jq -er` (`.github/workflows/release-publication-prepare.yml:317`). With `-e`, a `false` value exits 1 and would have short-circuited the whole `&&` chain into the error path on every healthy artifact. It is correct as written; every other field that is a nonzero int or non-empty string correctly uses `-er`.

**Injection / provenance.** `run_id`, `run_attempt`, `workflow_artifact_id`, `workflow_artifact_name` are all validated by `validate --kind qualification-artifact-index --require-canonical` **before** interpolation into API URLs and filesystem paths (`artifact_index.py:97-100,155-208`: positive ints; name must have no `/` and must equal `sifr-stable-candidate-<version>-<source_commit>-<suffix>`). Run-attempt binding checks `.id`, `.run_attempt`, `.head_sha == SOURCE_COMMIT`, `.conclusion == success`; artifact binding checks `.id`, `.name`, `.expired`, `.workflow_run.id`. The new one-to-one `workflow_id ↔ workflow_name` map plus `len(...) != 6` (`artifact_index.py:171-178,254-255`) closes the ambiguity `group_by(.id) | .[0].name` would otherwise have had.

**Archive extraction.** Rejects absolute paths, `..`, backslashes, symlink/device members, duplicates, overwrites, and non-empty or symlinked destinations; resolves each target and confirms containment. The `--expected-uncompressed-bytes` total is sound and *also* rejects undeclared content, because each of the six uploads contains exactly the declared artifacts: `path: qualification/*` globs to the zip root and `qualify_stable_target.py` writes only the 4 declared files (install/receipt/smoke all live in a `TemporaryDirectory`), giving 4×4 + 2 (editor) + 2 (assemble) = the 20 governed ids.

**Submodule binding — I initially flagged this as a gap and it is not one.** `_require_checkout` does not compare gitlinks, unlike the sibling `planner.validate_source_identity:232`. But the binding is anchored transitively: `validate_release_profile_report(..., source_root=source_root)` → `validate_source` → `collect_submodules(source_root) != submodules` (`release_report.py:127-128`), and `stable_prepare.py:139` then requires `report["source"]["submodules"] == plan["submodules"]`. Recursive submodule identity *is* re-derived from the real checkout.

**Mutation non-authority.** `materialize_stable_mutation` returns a proposal without writing; the only artifact produced is `prepare/summary.json` via `write_canonical_json(refuse_existing=True)` (proven by the CLI test's second-invocation assertion). Read-only permissions, no secrets, and the contract case forbids `contents: write`, `${{ secrets.`, `gh release upload`, `vsce publish`, `unzip`.

**Backward compatibility.** The reusable workflow's *outputs* are unchanged; only the internal step id moved `summary` → `outputs`, and both existing contract cases were updated. No workflow references `steps.summary`. All five new inputs default to `""`, and the sole caller (`release-publication.yml`) cannot reach the stable branch — its dispatch choices are `drill-*` and its call default is `preview`; if it ever did, empty inputs fail closed at the regex gate.

**Profile registration and de-duplication**, verified empirically rather than by reading: `--suite full --suite stable-prepare` → `variants=60, failures=0` with `stable-publication-prepare` executing exactly once.

**Re-run against final bytes:** stable_prepare 6/6 · governance 14/14 · runner `run_all` 11/11 · distribution `full`+`stable-prepare` 60/60 · coverage matrix 5/5 · all three workflow contract cases · `check_file_size_guardrails.py` PASS (largest new file 603 lines) · `git diff --check` clean.

## Non-blocking observations (no change required)

- The candidate-evidence file contract is now hard-coded in three independent places — `stable_prepare.py:48-58`, `evidence_custody.py:25-29`, and `evidence_custody.py:186-208` — with no test binding them. Drift fails closed in both directions, so this is maintainability, not correctness; a shared constant would remove the hazard when the next evidence file is added.
- `mode` (`initial`/`resume`) is recorded in the summary but has no behavioral effect in `materialize_stable_prepare`. Defensible for a read-only prepare (`validate_release_plan(active_index=…)` already rejects a plan whose index flip has happened), but the resume distinction only becomes load-bearing in the publish job.
- `plans/reviews/active/phase-40-milestone-40-5-stable-prepare-review-pass-1.md` is a 0-byte placeholder; it should not be committed empty.

I found no fail-open path, no expression/shell defect, no provenance or extraction weakness, no canonical/schema contract gap, no mutation authority, and no guardrail violation.

VERDICT: SATISFIED
