# Review: Phase 40 M40.5 schema-v2 preview epoch bootstrap (pass 10 — satisfied)

I read the full tracked `git diff` (21 files, 862/300), all 21 untracked files, the phase/issue ledger, and archived passes 1–9, then regenerated the widened mutant grammar from scratch off `schema_bootstrap.py`'s AST.

### Pass-9 remediation confirmed dead

The beta half of the release attestation now carries the complete seven-case alpha set (`schema_bootstrap_selftest.py:236-254`): unexpected key, wrong-channel version with matching asset shape (`release_evidence("0.1.0-alpha.7")`), invalid `source_commit`, invalid `release_record_sha256`, invalid asset digest, popped asset, and foreign same-channel asset set (`0.1.0-beta.99`).

Measured, not asserted — dropping the sole beta call at `schema_bootstrap.py:198`:

```
[30] drop-stmt L198: _validate_release_evidence
     rc=1 :: AssertionError: evidence mutation 32 unexpectedly passed
```

It is now killed, symmetric with its alpha twin at `:163` (killed by mutation 25). Both `_validate_approvers` calls (`:148`, `:189`) and `_require_exact_bootstrap_membership` (`:314`) are likewise killed by governed `AssertionError`s, not by crashes.

### Widened mutation sweep re-run

**88 mutants — one per `or`-clause of every `if …: fail(…)`, one per dropped `require_*`/`_validate_*`/`_require_*` statement, one per bypassed `x = require_*(y, …)` — 33 survive.** All 88 were verified to compile, so every "killed" result is a real behavioral kill rather than a syntax error. Every mutation ran in a `/tmp` copy; the working tree was never touched (diffstat identical before and after).

The 33 survivors are exactly the structurally masked set carried forward from passes 8–9: the 13 container/type-guard bypasses, the 4 `require_nonempty_string` bypasses (`:93`, `:216`, `:461`, `:486`), `:268`, `:310`, `:429`, `:462`, the five membership clauses (`:389`, `:390`, `:397`×2, `:406`), and the seven `:294` `is None` clauses. Pass 9's prose enumerated these as "32" while listing 33 — a bookkeeping slip in the report, not a change in the code. I re-probed the ones that could plausibly be coverage gaps rather than masking; the sharpest, `index["ga_status"] != "preview"` at `:397`, is genuinely pinned by its live sibling: `validate_release_index` (`release_index.py:33-39`) requires a `stable` channel key whenever `ga_status == "active"`, so any non-preview index fails the `index["channels"] != expected_channels` clause in the same guard. The seven `is None` clauses are masked by each other within one `or` under the single missing-inputs negative. No survivor is a fail-open path.

### Gates

`governance.schema_bootstrap_selftest` PASS · named `epoch-bootstrap` suite PASS (variants=1, failures=0) · full `distribution_release` area PASS (**variants=110, failures=0**, `schema-v2-preview-epoch-bootstrap` appearing exactly once — the pass-3 dedup holds) · `governance.selftest` PASS (tests=14) · `ruff check` **rc=0** on all 11 changed/new Python files (pyenv 3.10.12, ruff 0.1.4) · file-size guardrail PASS (2898 files, limit 900) · `git diff --check` PASS. All earlier gates and remediations remain green.

### Ledger

Pass 9 is archived at `plans/reviews/archive/…-pass-9.md` and ledgered at `plans/issues/active/phase-40-stable-channel-ga-execution.md:427-432`; the entry describes the beta mirror accurately (object shape, channel/version, source commit, release-record digest, exact asset membership, individual asset digests). Passes 1–9 are all present and consistent.

## Actionable findings

None.

## Not findings

- 33 surviving mutants, each re-probed and confirmed masked by a pinned sibling on its real entry path. Correctly ordered defense-in-depth, not fail-open.
- `ruff format --check` would still reformat several files in the package; wired into no repo gate, pre-existing baseline.
- `release_governance.py --kind schema-bootstrap-evidence` routes to the semantic validator and not the JSON Schema. Now that both channel halves of `_validate_release_evidence` are independently pinned, the durable path is fully guarded; the schema remains the stricter redundant contract exercised by `schema_contracts`.
- Post-`gh release create` failures remain unrecoverable by re-run; fail-loud stays correct.
- Whether `actions/download-artifact@v4` resolves an attempt-1 artifact during "Re-run failed jobs" is still not locally reproducible.

## Commit mechanics and downstream execution requirements (not implementation defects)

- `plans/reviews/active/phase-40-milestone-40-5-schema-bootstrap-review-pass-10.md` is a 0-byte untracked file. It must receive this report, be archived, and be ledgered. I did not modify it, per instruction.
- Downstream, not implementation prerequisites: `stable-release` environment with ≥1 `release/distribution` reviewer and "prevent self-review" enabled; no reviewers on the auto-created `preview-release` environment; live `channels.json` still exactly 105 bytes / `71b3243925…4bf9ef` at `bootstrap-index` time; `sifr.sh` serving new dispatcher bytes inside the 180-second budget; the `SIFR_WEBSITE_ACTIONS_TOKEN` secret.

## Whole-wave verdict

Every structural, workflow, profile, schema, producer-path, and coverage finding from passes 1–9 is closed and independently re-measured. The last-standing defect — an unpinned beta release-evidence validator, surfaced only when the mutant grammar widened to whole-call deletions — is dead, killed by a seven-case set that mirrors alpha exactly. Re-running that same widened grammar from scratch produced no new survivor outside the structurally masked set, and no fail-open path remains in the bootstrap validator, producer, or workflow contract.

VERDICT: SATISFIED
