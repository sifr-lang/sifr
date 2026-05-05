# milestone_diag_5 slice 2 review (pass 1)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-5-baseline-fixture-guardrails` against `main`. Slice intent (per [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75) and the slice DoD subset at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1010](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1010), [:1029](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1029)): add verification-harness duplicate-baseline artifact path detection that fails loudly before any case in a baseline suite executes or is blessed, so two cases/variants cannot silently share one checked baseline file.

Files in scope:

- [scripts/run_verification_hardening.py](scripts/run_verification_hardening.py) (added `baseline_variant_label`, `baseline_artifact_paths`, `validate_unique_baseline_artifact_paths`; refactored `baseline_case_result` to use the helpers; wired the validator into `run_baseline_suite`).
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) (added the in-progress slice 2 status line at line 75).

Out-of-scope DoD bullets explicitly carried forward to later slices: centralized baseline normalization, contradictory-expectation detection within a single fixture, and the JSON/compact/human renderer fixture-level test (per [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009-1031](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009)). Slice 1 review pass 3 already documented these as follow-up items at [reviews/semantic-diagnostic-code-taxonomy-diag-5-harness-contract-review-pass-3.md:70](reviews/semantic-diagnostic-code-taxonomy-diag-5-harness-contract-review-pass-3.md:70). Slice 2's status line correctly limits its claim to duplicate-baseline detection only, so DoD framing stays honest.

## Verdict

No must-fix blockers. The check correctly implements the contract for the only runner that produces checked baseline artifacts, runs before both command execution and `--bless`, and uses the same path-derivation helper as the consumer code so the detector and the writer cannot drift. Several should-fix and nit items below would harden the slice; none are gating.

## Contract verification

Intended contract: "baseline suites must fail before executing or blessing if two cases/variants would use the same checked baseline artifact path (stdout/stderr/exit-code), so two fixtures cannot silently share one baseline file."

1. **Detection trigger covers stdout/stderr/exit-code.** [scripts/run_verification_hardening.py:211-217](scripts/run_verification_hardening.py:211) returns the same triple `(stdout_file, stderr_file, exit_file)` that the writer/comparator at [scripts/run_verification_hardening.py:362-369](scripts/run_verification_hardening.py:362) consumes. The validator at [scripts/run_verification_hardening.py:268](scripts/run_verification_hardening.py:268) iterates all three so a collision on any one of the three suffixes (e.g., two cases that share an `entry.parent + label` prefix but where only the `.exit-code.txt` happens to overlap — impossible in this layout, but the loop is exhaustive anyway) is caught. ✓
2. **Detection runs before execution and before blessing.** [scripts/run_verification_hardening.py:431-435](scripts/run_verification_hardening.py:431) calls `validate_unique_baseline_artifact_paths` before the per-case loop and before the `print(f"  suite=…")` line; `args.bless` is not consulted, so blessing follows the same gate. ✓
3. **Label derivation is shared between detector and consumer.** Both the validator and `baseline_case_result` derive labels through `baseline_variant_label` ([scripts/run_verification_hardening.py:207-208](scripts/run_verification_hardening.py:207), used at [:267](scripts/run_verification_hardening.py:267) and [:342](scripts/run_verification_hardening.py:342)). A future change to the label format flows through both paths atomically — this is the structurally important property and the refactor gets it right. ✓
4. **Failure mode is `SystemExit` with the relative artifact path and both owners.** [scripts/run_verification_hardening.py:271-276](scripts/run_verification_hardening.py:271) reports the colliding artifact's repo-relative path and both `case_id:label` owners. That message is enough for a human to grep the manifest and resolve the collision. ✓ (See Finding C for one edge case where this message can itself raise.)

## Coverage of "any other runner path in this slice"

The slice description asks whether the check should cover other runners in this slice. Audit of every runner in this script:

| Runner | Writes checked-in baseline artifacts? | Evidence |
| --- | --- | --- |
| `baseline` | Yes — `<entry.parent>/baselines/<label>.{stdout,stderr,exit-code}.txt` | [scripts/run_verification_hardening.py:362-369](scripts/run_verification_hardening.py:362) |
| `fixedbugs` | No — writes only to `target/verification/actual/...` on failure | [scripts/run_verification_hardening.py:577-580](scripts/run_verification_hardening.py:577) |
| `crashes` | No — metadata-only validation | [scripts/run_verification_hardening.py:618-724](scripts/run_verification_hardening.py:618) |
| `property` | No — in-memory determinism comparison only | [scripts/run_verification_hardening.py:732-884](scripts/run_verification_hardening.py:732) |
| `fuzz-smoke` | No — writes mutated sources to `target/verification/tmp/`, no checked-in baselines | [scripts/run_verification_hardening.py:1092-1093](scripts/run_verification_hardening.py:1092) |
| `oss-curated` / `ecosystem-broader` | No — exit-code/panic checks only | [scripts/run_verification_hardening.py:1165-1393](scripts/run_verification_hardening.py:1165) |
| `determinism-scale` | No — runs external commands and asserts exit codes | [scripts/run_verification_hardening.py:1427-1533](scripts/run_verification_hardening.py:1427) |

Conclusion: scoping the validator to `run_baseline_suite` is correct for slice 2's contract; no other runner has a checked baseline-artifact namespace to police. ✓

(There is a *separate* collision surface — `target/verification/actual/<suite>/<case_id>/<label>.<stream>.txt` — that every runner shares, governed by `(suite_name, case_id, label)`. That namespace is not the slice's concern. See Finding F.)

## Findings

### Finding A (should-fix) — no automated test exercises the new validator

`grep -rn 'validate_unique_baseline_artifact_paths\|baseline_artifact_paths\|baseline_variant_label' .` shows the symbols only inside `scripts/run_verification_hardening.py`. The repo has no Python test infrastructure (no `pytest.ini`/`pyproject.toml` outside `third_party/ruff`), so a true `pytest` test is not idiomatic here. But the validator is a pure Python function with no I/O side effects on the success path and one clear `SystemExit` on failure — exactly the shape that benefits from a unit test, because:

- A future refactor that drops the helper, mishandles `parse_formats(None) → [None]`, or moves the call out of `run_baseline_suite` would silently regress the guardrail. The slice's whole point is to prevent silent baseline collisions; losing the detector to silent regression is the same failure mode at a different level.
- The symptom of a regression is "two fixtures quietly bless to the same file again" — slow to surface, hard to attribute.

Cheap options that fit the existing toolchain:

1. Add a Rust integration test (in the same crate as `run_e2e_pass.sh` style harnesses) that invokes the script with a synthetic in-memory manifest containing a deliberate collision and asserts non-zero exit + the specific message. This matches how other Python scripts (`scripts/check_diagnostic_schema_sync.py` etc.) are wired into `scripts/run_all_tests.sh`.
2. Or add a small `--self-test` flag to the script that constructs a synthetic suite-with-collision in-process and asserts SystemExit, then wire it into `scripts/run_all_tests.sh` so a regression breaks the local gate.

Either path closes the slice's regression-locking loop. The Rust-integration option is more consistent with how the rest of `verification/` is gated. This is a should-fix because the slice is a guardrail and an unguarded guardrail does not actually guard.

### Finding B (should-fix) — `case_entry` not normalized, so semantically-equivalent textual variants would not collide

`validate_unique_baseline_artifact_paths` keys on `Path` values produced by `repo_root / case_entry` ([scripts/run_verification_hardening.py:265](scripts/run_verification_hardening.py:265)) without `.resolve()`. Two cases that author the same fixture two different ways — e.g., `crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/main.sifr` vs `./crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/main.sifr`, or with a `..` normalization — produce different `Path` keys but resolve to the same on-disk baseline directory and would silently bless on top of each other.

Today the manifest at [verification/suites/manifest.json](verification/suites/manifest.json) keeps entries in canonical `crates/...` form, so this is not actively exploited. But the slice's framing is "two fixtures cannot silently share one baseline file" — and `Path` string equality is a strictly weaker invariant than filesystem identity. Recommended fix:

```python
artifact_path = (baseline_dir / f"{label}.{suffix}.txt").resolve()
```

or, more conservatively, `os.path.normpath(str(...))` before keying. The error message at [scripts/run_verification_hardening.py:271-276](scripts/run_verification_hardening.py:271) should keep using `relative_to(repo_root)` for human readability after normalization — keep the resolved key separate from the displayed path.

### Finding C (nit) — `relative_to(repo_root)` on the error path can itself raise on absolute `case_entry`

[scripts/run_verification_hardening.py:272](scripts/run_verification_hardening.py:272) calls `artifact_path.relative_to(repo_root)`. If a future manifest entry has an absolute `case_entry` (e.g., `/abs/path/main.sifr`), `repo_root / case_entry` returns the absolute path unchanged (Python `Path.__truediv__` semantics), `entry_path.parent / "baselines"` is fine, but the relative-to call raises `ValueError: '/abs/.../baselines/check.stdout.txt' is not in the subpath of '<repo>'`. The user then sees a Python traceback instead of the intended SystemExit message.

Today's manifest uses relative paths only and there is a separate `entry_path.is_file()` check in `baseline_case_result` ([scripts/run_verification_hardening.py:325](scripts/run_verification_hardening.py:325)), so this is not actively exploitable. But the validator is the *first* thing that runs and is supposed to fail loudly with a clear message; trapping `ValueError` (or asserting `case_entry` is relative up front) would make the failure mode robust. Pairs naturally with the Finding B fix.

### Finding D (nit) — duplicate-format-within-one-case collisions report `X:label and X:label`

If a manifest case lists `"diagnostic_formats": ["human", "human"]` (a per-case authoring mistake, not a cross-case collision), `parse_formats` returns `["human", "human"]` ([scripts/run_verification_hardening.py:199-204](scripts/run_verification_hardening.py:199); it does not dedupe), the validator's loop hits `seen[artifact_path]` on the second iteration with `previous == owner == f"{case_id}:check-human"`, and the error reads:

```
suite '<name>' baseline artifact path collision for <path>: A:check-human and A:check-human
```

The error correctly identifies *that* there is a problem and is recoverable by the author, but the duplicated owner string reads like a logic bug rather than a user-fixable manifest mistake. Two cheap improvements:

1. Detect this specific case: `if previous == owner: raise SystemExit(f"... case '{case_id}' lists '{diagnostic_format}' more than once in 'diagnostic_formats'")` before the generic collision branch.
2. Or have `parse_formats` reject duplicate entries up front (`SystemExit("... duplicate diagnostic_format '<x>' in 'diagnostic_formats'")`), keeping the validator's collision message reserved for genuine cross-case collisions.

Option 2 is structurally cleaner because it pushes the per-case shape check next to the other per-case shape checks at [scripts/run_verification_hardening.py:328-330](scripts/run_verification_hardening.py:328) and keeps `validate_unique_baseline_artifact_paths` focused on cross-case collisions. Either is a nit; the slice's contract is met without it.

### Finding E (nit) — `baseline_case_result` and `validate_unique_baseline_artifact_paths` repeat the same per-case shape checks

Both functions independently validate `case_id` is `str`, `case_entry` is `str`, `command_name` is in the four-command set, and `formats` is non-empty. See [scripts/run_verification_hardening.py:247-263](scripts/run_verification_hardening.py:247) vs [:313-330](scripts/run_verification_hardening.py:313). The two error messages are textually identical, so a future tweak (e.g., adding `"check-tests"` to the command set) must be made in two places or the validator and consumer disagree about which manifests are accepted.

A small `assert_baseline_case_metadata(case, suite_name) -> tuple[str, str, str, list[str | None]]` helper that returns `(case_id, case_entry, command_name, formats)` after raising on any shape failure would let both sites share one source of truth. This is a DRY nit, not a correctness issue — `baseline_case_result` additionally validates `expect_exit_code` and `entry_path.is_file()` which the new validator deliberately does not, so the helper would only cover the four shared checks.

### Finding F (informative — out of slice 2 scope) — `target/verification/actual/<suite>/<case_id>/...` collisions are not policed

Every runner that fails a case writes diagnostic actuals to `actual_root / suite_name / case_id / <label>.<stream>.txt` (e.g., [scripts/run_verification_hardening.py:396-399](scripts/run_verification_hardening.py:396), [:577-580](scripts/run_verification_hardening.py:577)). Uniqueness in this namespace requires `(suite_name, case_id, label)` to be unique within a suite. Today nothing validates that two cases in the same suite have distinct `case_id`, so two cases with id=`X` and different entries would silently overwrite each other's failure-side actuals. This is *not* the slice's contract (the actual subtree is transient debug output, not checked-in baselines), so it is correct to leave it for a follow-up. Flagging only so it is not lost: a sibling validator `validate_unique_case_ids` belongs in the same neighborhood whenever the harness next gets a maintenance pass. No action required for slice 2.

### Finding G (informative — out of slice 2 scope) — `run_fixedbugs_suite` re-derives the label inline

[scripts/run_verification_hardening.py:549](scripts/run_verification_hardening.py:549) still has `label = f"{command_name}-{diagnostic_format}" if diagnostic_format else str(command_name)`, which is structurally the same expression `baseline_variant_label` now centralizes. The fixedbugs runner's label is only used for `actual` path naming (no checked-in baselines), so a divergence between the two label formats does not break the new guardrail. But the slice already extracted the helper; reusing it in `run_fixedbugs_suite` is a one-line consistency improvement that costs nothing. The helper signature requires `command_name: str | None`, and `command_name` is already constrained to a string at the fixedbugs validation step at [scripts/run_verification_hardening.py:526](scripts/run_verification_hardening.py:526), so the conversion is safe. Defer to a separate cleanup if desired.

### Finding H (informative) — issue status text accurately scopes the slice

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75) reads:

> `milestone_diag_5` slice 2 in progress: add verification harness duplicate-baseline artifact path detection before command execution or blessing so two variants cannot share the same checked baseline output.

This precisely matches the implemented behavior — pre-execution AND pre-bless, baseline runner only, artifact path scope (not actual-tree, not contradictory-expectation). It does not over-promise the broader DoD bullets at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009-1011](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009) (centralized normalization, contradictory-expectation detection), which is appropriate. No stale-doc finding here. ✓

### Finding I (informative) — no stale references to the old inline label expression

`grep -n 'f"{command_name}-{diagnostic_format}"' scripts/run_verification_hardening.py` returns two hits: the helper body at line 208 and the unrelated `run_fixedbugs_suite` call at line 549 (Finding G). The original baseline-runner inline copy at the previous line 339 is gone. No stale duplicate path remains. ✓

## Verification of slice scope coverage vs. broader DoD

The slice 2 issue status line covers exactly one of the [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009-1011](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009) bullets:

| DoD bullet | Slice 2 scope? | Status |
| --- | --- | --- |
| Centralize diagnostic baseline normalization in the harness ([:1009](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009)) | Out of scope | Deferred to a later slice |
| Detect duplicate baseline names at harness startup ([:1010](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1010)) | In scope | Implemented for the `baseline` runner, runs before execution and bless |
| Detect fixture-grammar contradictions at harness load time ([:1011](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1011)) | Out of scope | Deferred — this is e2e expectation grammar, not verification baseline files |

The slice's contract is "two fixtures cannot silently share one baseline file." That contract is satisfied for the only runner that has checked baseline files, and the validator runs at the only two moments — execution and bless — when sharing would matter. ✓

## Net regressions vs. main

None observed. The refactor that introduces `baseline_variant_label` and `baseline_artifact_paths` is a pure extraction; the consumer body at [scripts/run_verification_hardening.py:341-369](scripts/run_verification_hardening.py:341) computes the same labels and paths it computed before. The validator is additive and only raises before the existing per-case loop runs.

## Recommended action plan for pass 2 / follow-up

No must-fix or should-fix blockers prevent shipping the slice as-is, but the following would harden the guardrail materially:

1. **Should-fix — add a regression test for the validator (Finding A).** Either an in-process `--self-test` flag wired into `scripts/run_all_tests.sh` or a small Rust integration test that exercises the script with a synthetic colliding manifest. Without this, the slice ships a guardrail that future refactors can silently lose.
2. **Should-fix — normalize `Path` keys before collision lookup (Finding B).** `.resolve()` (or `os.path.normpath`) on the artifact key. Optional: pair with a relative-only check on `case_entry` to also cover Finding C.
3. **Nit — disambiguate the duplicate-format-within-one-case error (Finding D).** Most cleanly handled by deduping in `parse_formats`.
4. **Nit — extract a shared `assert_baseline_case_metadata` helper (Finding E).** Removes a textual-duplication trap between the validator and `baseline_case_result`.
5. **Nit — reuse `baseline_variant_label` in `run_fixedbugs_suite` (Finding G).** One-line consistency win; not gating.
6. **Out-of-scope follow-up — duplicate `case_id` detection in the `actual` namespace (Finding F).** Belongs in a future hardening slice, not slice 2.

None of the above blocks merging slice 2 in its current form. The contract the slice claims to deliver — duplicate-baseline artifact path detection before execution or blessing for the only runner that has baselines — is delivered.
