# milestone_diag_5 slice 2 review (pass 3)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-5-baseline-fixture-guardrails` against `origin/main`. Slice intent (per [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75) and the slice DoD subset at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1010](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1010), [:1029](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1029)): add verification-harness duplicate-baseline artifact path detection that fails loudly before any case in a baseline suite executes or is blessed, so two cases/variants cannot silently share one checked baseline file.

Pass 3 delta vs. pass 2 (per the user prompt and confirmed by `git diff`):

- One new self-test block at [scripts/run_verification_hardening.py:429-444](scripts/run_verification_hardening.py:429) exercising `entry: "../escape/main.sifr"` and asserting the error message contains `"entry must stay under repo root"`. This implements the optional Finding J nit from [reviews/semantic-diagnostic-code-taxonomy-diag-5-baseline-guardrails-review-pass-2.md:98](reviews/semantic-diagnostic-code-taxonomy-diag-5-baseline-guardrails-review-pass-2.md:98).

No other changes vs. pass 2. `git diff origin/main` shows the same three files in scope as pass 2: [scripts/run_verification_hardening.py](scripts/run_verification_hardening.py), [scripts/run_all_tests.sh](scripts/run_all_tests.sh), and [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md), with the issue file's `+` hunk and the `run_all_tests.sh` self-test wiring identical to pass 2.

## Verdict

**Satisfied — no must-fix blockers.** Pass 2's verdict already had no blockers; the pass 3 addition is a purely additive coverage improvement that resolves the lone optional nit (Finding J) without changing any production code path. All earlier pass 1 findings (A-E, G) remain resolved exactly as documented in pass 2; pass 2's contract verification (validator runs before execution AND before bless, path-identity keying, label-derivation single source of truth) carries forward unchanged. The slice is ready to merge.

## Pass 2 follow-up coverage

| Pass 2 finding | Pass 2 disposition | Pass 3 status | Evidence |
| --- | --- | --- | --- |
| J — self-test does not exercise the `..`-escape branch | optional nit | **Resolved.** New `assert_self_test_failure` block at [scripts/run_verification_hardening.py:429-444](scripts/run_verification_hardening.py:429) calls `validate_unique_baseline_artifact_paths` with `entry: "../escape/main.sifr"` and asserts the failure message contains `"entry must stay under repo root"`. Local `python3 scripts/run_verification_hardening.py --self-test` exits 0 and prints `verification hardening self-tests ok`. ✓ |
| K — cross-suite collision is still not policed | informative, OOS | Unchanged. Still correctly out of slice 2 scope. ✓ |
| L — `parse_formats` coerces non-string list items via `str(item)` | informative, OOS | Unchanged. Still correctly out of slice 2 scope. ✓ |
| M — issue status line stays "in progress" until merge | informative | Unchanged. The line at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75) still accurately scopes the slice. ✓ |

## Pass 3 addition: trace through the new self-test case

The added self-test at [scripts/run_verification_hardening.py:429-444](scripts/run_verification_hardening.py:429) is structured identically to its three siblings (positive case at [:357-374](scripts/run_verification_hardening.py:357), normalized-duplicate at [:375-396](scripts/run_verification_hardening.py:375), duplicate-format at [:397-411](scripts/run_verification_hardening.py:397), absolute-entry at [:413-428](scripts/run_verification_hardening.py:413)), so it inherits the same regression-locking property: `assert_self_test_failure` ([:342-352](scripts/run_verification_hardening.py:342)) demands BOTH `SystemExit` AND the expected substring, so a future refactor that drops the message specificity (e.g., a generic "validation failed") fails the assertion.

Manual trace through `baseline_case_metadata` ([:273-311](scripts/run_verification_hardening.py:273)) for `entry: "../escape/main.sifr"` against `repo_root = Path("/tmp/sifr-verification-hardening-self-test").resolve()`:

1. Type-shape checks at [:282-288](scripts/run_verification_hardening.py:282) all pass — `id`/`entry`/`command` are valid strings, `command="check"` is in `BASELINE_COMMANDS`.
2. Absolute-check at [:289-290](scripts/run_verification_hardening.py:289): `Path("../escape/main.sifr").is_absolute()` returns `False`, so this branch is **not** taken (this is the property that distinguishes Finding J's branch from the absolute-rejection branch already covered).
3. `parse_formats(["json"])` returns `["json"]`; `validate_unique_diagnostic_formats` passes.
4. `entry_path = (repo_root / "../escape/main.sifr").resolve()` resolves to `Path("/tmp/escape/main.sifr")` — outside `repo_root`'s subtree.
5. `entry_path.relative_to(repo_root)` at [:306](scripts/run_verification_hardening.py:306) raises `ValueError`.
6. The `except ValueError` at [:307-310](scripts/run_verification_hardening.py:307) catches it and re-raises `SystemExit("suite 'self-test' case 'escape' entry must stay under repo root") from error`.
7. `assert_self_test_failure` ([:342-352](scripts/run_verification_hardening.py:342)) catches the `SystemExit` and confirms the substring `"entry must stay under repo root"` is present.

The path through the code is the one Finding J flagged as untested — `is_absolute() == False` AND `relative_to` raises — and is now regression-locked with the same shape as the existing three failure-mode self-tests.

## Self-test coverage assessment (pass 3 update)

`run_self_tests` ([:355-446](scripts/run_verification_hardening.py:355)) now covers:

| Scenario | Self-test case | Asserted message fragment |
| --- | --- | --- |
| Two distinct entries, same command/formats — should pass | [:357-374](scripts/run_verification_hardening.py:357) | (no exit; positive case) |
| Same fixture authored two ways (`./` prefix) | [:375-396](scripts/run_verification_hardening.py:375) | `fixtures/a/baselines/check-json.stdout.txt` |
| Duplicate format inside one case (`["json", "json"]`) | [:397-411](scripts/run_verification_hardening.py:397) | `lists diagnostic_format 'json' more than once` |
| Absolute `entry` (`/tmp/main.sifr`) | [:413-428](scripts/run_verification_hardening.py:413) | `entry must be repo-relative` |
| **Repo-relative `entry` that escapes via `..` (`../escape/main.sifr`) — pass 3 addition** | [:429-444](scripts/run_verification_hardening.py:429) | `entry must stay under repo root` |

The asymmetric coverage flagged in pass 2 (absolute-entry rejection covered, escape-via-`..` not covered) is now symmetric: both branches of the "entry must be inside repo" invariant are regression-locked. Each branch has its own message and its own self-test asserting the message-specific substring, so a refactor that merges the two branches into one generic check would fail both assertions.

`assert_self_test_failure` continues to demand both `SystemExit` and the expected substring, so the addition does not weaken the regression-locking property of the harness.

## Contract verification (pass 3 re-check)

Pass 2's contract-verification table is unchanged because no production code path was modified between pass 2 and pass 3. Re-stating the conclusions for record:

1. **Validator runs before execution AND before bless.** [scripts/run_verification_hardening.py:584-588](scripts/run_verification_hardening.py:584) calls `validate_unique_baseline_artifact_paths` at the head of `run_baseline_suite`, before the `print(f"  suite=…")` line and before any `baseline_case_result`. `args.bless` is not consulted at the validator gate. ✓
2. **Path identity (not text identity) is what the validator enforces.** Both `baseline_case_metadata` and `baseline_artifact_key` resolve their paths through `Path.resolve()`. ✓
3. **Stdout, stderr, and exit-code artifacts are all keyed.** `validate_unique_baseline_artifact_paths` iterates every entry of `baseline_artifact_paths`. ✓
4. **Label derivation cannot drift between detector and consumer.** Validator and `baseline_case_result` both go through `baseline_variant_label`; `run_fixedbugs_suite` also routes through the helper. ✓
5. **Failure modes are clear `SystemExit` strings.**
   - cross-case collision: "suite '<name>' baseline artifact path collision for <repo-relative-path>: <previous-owner> and <current-owner>"
   - duplicate format within one case: "suite '<name>' case '<id>' lists diagnostic_format '<x>' more than once"
   - absolute entry: "suite '<name>' case '<id>' entry must be repo-relative"
   - escapes-via-`..`: "suite '<name>' case '<id>' entry must stay under repo root"

   All four are now regression-locked by the self-test suite. ✓

## `run_all_tests.sh` self-test wiring (pass 3)

Wiring at [scripts/run_all_tests.sh:105-106](scripts/run_all_tests.sh:105) is unchanged from pass 2. The new self-test case adds one in-process `validate_unique_baseline_artifact_paths` call (no I/O, no `cargo`, no `subprocess`) — order-of-microseconds added to the lane. No change to lane coverage, position, or failure-surface analysis from pass 2.

## Net regressions vs. main (pass 3)

None. The pass 3 diff vs. pass 2 is a pure addition inside `run_self_tests`:

- The new `assert_self_test_failure` invocation does not call any new helper, does not exercise any new production code path, and does not touch the manifest or the filesystem (`Path.resolve()` on a non-existent path under `/tmp` is a string-only operation in Python).
- All other pass 2 contract checks (validator gate position, label-derivation single source of truth, suite-local `seen` dict, four `SystemExit` failure-mode strings, `run_fixedbugs_suite` label reuse, `BASELINE_COMMANDS` set) carry forward unchanged.

## Findings

### Finding N (informative — symmetry achieved)

Pass 2's Finding J is resolved. The `is_absolute() == False` AND `relative_to` raises branch is now regression-locked with the same `assert_self_test_failure` shape as the absolute-rejection branch. No new findings emerge from the pass 3 addition.

### Finding O (informative — out of slice 2 scope, carried from pass 2)

Pass 2's Findings K (cross-suite collision policing) and L (`parse_formats` coercing non-string entries) remain valid follow-ups for a future hardening slice. Neither is gating; both are correctly outside slice 2's scope per the issue's status line at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75).

## Recommended action plan

None required. The slice's contract — duplicate-baseline artifact path detection before execution or blessing for the only runner that has baselines, regression-locked via `--self-test` wired into the authoritative local gate — is delivered, and the four `SystemExit` failure modes the validator can produce are all covered by self-test cases. Pass 3 closes the optional polish item from pass 2 and merits a satisfied verdict without further iteration.
