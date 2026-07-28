# Review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 9)

I read the full `git diff`, all 20 untracked files, the phase/issue ledger, and archived passes 1–8, then regenerated pass-8's full-validator mutant sweep from scratch off `schema_bootstrap.py`'s AST (one mutant per `or`-clause of every `if …: fail(…)`, one per dropped `require_*`/`_validate_*`/`_require_*` statement, one per bypassed `x = require_*(y, …)`). **88 mutants; 33 survive.** Every mutation ran in a `/tmp` copy — the working tree was never touched (diffstat identical before and after: 21 files, 856/300).

### Pass-8 remediation confirmed

`_validate_approvers`' container guard (`schema_bootstrap.py:227`) is now **killed**. With `values = payload` substituted for `require_array(payload, location)`, the merge-gated selftest fails at `evidence mutation 7 unexpectedly passed` — the distinct-character scalar at `schema_bootstrap_selftest.py:197` (`"approvers": "abc"`) walks past the `normalized in seen` dedup that masked `"release-reviewer"` and lands on the container requirement. Pass-8 finding 2 is also closed: the ledger entry at `plans/issues/active/phase-40-stable-channel-ga-execution.md:423-429` now states the container/value isolation accurately and records pass 7's claim as retroactively made true.

### Survivor audit

32 of the 33 survivors are exactly pass-8's re-confirmed masked set — `:268`, `:310`, the `:388`/`:397` clauses, `:429`, `:462`, the seven `:294` `is None` clauses, the type-guard bypasses, and the four `require_nonempty_string` bypasses at `:93`, `:216`, `:461`, `:486`. I re-probed each at its real entry path; all still fail closed via a pinned sibling. Not findings.

The 33rd is new — my generator emitted `_validate_*` drop-statements, a class pass 8's `require_*`-only generator did not produce.

### Gates re-run

`schema_bootstrap_selftest` PASS · named `epoch-bootstrap` suite PASS (variants=1) · full `distribution_release` area PASS (**variants=110, failures=0**, covering `validate_schema_contracts` and all contract cases) · `governance.selftest` PASS (tests=14) · `ruff check` **rc=0** on all 11 changed/new Python files (pyenv 3.10.12) · file-size gate PASS (2898 files) · `git diff --check` PASS. All earlier gates and remediations remain green.

## Actionable findings

### 1. `$.beta` release evidence is validated by exactly one unpinned call — Medium

`verification/areas/distribution_release/governance/schema_bootstrap.py:198`. Replacing `_validate_release_evidence(evidence["beta"], "beta", "$.beta")` with `pass` leaves `governance.schema_bootstrap_selftest` at **rc=0**. Its alpha counterpart at `:163` (same mutant class, index 26) is killed — the asymmetry is the whole defect.

The selftest's semantic mutation table (`schema_bootstrap_selftest.py:189-249`) carries seven mutations against `$.alpha` (`unexpected` key, wrong-channel version, bad `source_commit`, bad `release_record_sha256`, bad and popped `published_assets` entries, foreign asset set) and **zero** against `$.beta`. Measured with `:198` dropped:

```
beta.source_commit = "not-a-commit"             → ACCEPTED
beta.release_record_sha256 = "not-a-digest"     → ACCEPTED
beta = release_evidence("0.1.0-alpha.7")        → ACCEPTED
beta.published_assets: one entry popped         → ACCEPTED
beta.published_assets: one digest not-a-digest  → ACCEPTED
beta.unexpected = True                          → ACCEPTED
beta = "nope"                                   → ACCEPTED
```

Baseline rejects all seven, so the guard is correct today and genuinely load-bearing. It is also the sole gate on the durable path: `scripts/distribution/release_governance.py:221` maps `--kind schema-bootstrap-evidence` straight to `validate_bootstrap_evidence` and never consults `schema_epoch_bootstrap_evidence.schema.json`, whose `$defs.beta` (`:169`) would otherwise catch the shape. The merge-gated schema contracts only exercise `beta`'s presence/absence (`schema_contracts.py:71,88`), never its content. With `:198` gone, generation-1 evidence could durably attest a beta release whose version, source commit, record digest, or published asset set is unrelated to what was actually published — the same attestation the alpha half is pinned seven ways against.

Add `$.beta` counterparts to the alpha mutations at `schema_bootstrap_selftest.py:215-234`. One wrong-channel case (`{"beta": release_evidence("0.1.0-alpha.7")}`) isolates the call on its own; mirroring the full alpha set restores symmetry.

## Not findings

- 32 surviving mutants re-confirmed masked by a pinned sibling. Correctly ordered defense-in-depth.
- Full-area selection is 110 variants with `schema-v2-preview-epoch-bootstrap` appearing exactly once — the `full`+`epoch-bootstrap` dedup from pass 3 holds.
- `ruff format --check` would still reformat several files in the package; wired into no repo gate, pre-existing baseline.
- The four cross-field semantics 2020-12 cannot express remain validator-stricter and semantically covered.
- Post-`gh release create` failures remain unrecoverable by re-run; fail-loud stays correct.

## Commit mechanics and downstream execution requirements (not implementation defects)

- `plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-9.md` is a 0-byte untracked file. It must receive this report, be archived, and be ledgered (passes 1–8 are at `:361-429`). I did not modify it, per instruction.
- Downstream, not implementation prerequisites: `stable-release` environment with ≥1 `release/distribution` reviewer and "prevent self-review" enabled; no reviewers on the auto-created `preview-release` environment; live `channels.json` still exactly 105 bytes / `71b3243925…4bf9ef` at `bootstrap-index` time; `sifr.sh` serving new dispatcher bytes inside the 180-second budget.
- Still not locally reproducible: whether `actions/download-artifact@v4` resolves an attempt-1 artifact during "Re-run failed jobs".

## Whole-wave verdict

Every structural, workflow, profile, schema, and producer-path finding from passes 1–8 is closed and re-measured green, and the approver-container guard that survived four rounds is now dead. The remaining defect is one more instance of the same masking pattern, surfaced only because this pass widened the mutant grammar to whole-validator-call deletions: the beta half of the release attestation carries none of the seven negatives its alpha twin does. It is a test-coverage gap, not a behavior defect — but it sits on the evidence field that binds generation 1 to a real published beta release, and it is a mechanical fix.

VERDICT: NOT SATISFIED
