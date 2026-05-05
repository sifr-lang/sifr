# milestone_diag_5 slice 2 review (pass 2)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-5-baseline-fixture-guardrails` against `origin/main`. Slice intent (per [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75) and the slice DoD subset at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1010](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1010), [:1029](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1029)): add verification-harness duplicate-baseline artifact path detection that fails loudly before any case in a baseline suite executes or is blessed, so two cases/variants cannot silently share one checked baseline file.

Files in scope (vs. `origin/main`):

- [scripts/run_verification_hardening.py](scripts/run_verification_hardening.py) — added `BASELINE_COMMANDS`, `--self-test` flag, `validate_unique_diagnostic_formats`, `baseline_variant_label`, `baseline_artifact_paths`, `baseline_artifact_key`, `format_repo_relative_path`, `baseline_case_metadata`, `validate_unique_baseline_artifact_paths`, `assert_self_test_failure`, `run_self_tests`; refactored `baseline_case_result` and `run_fixedbugs_suite` to share the new helpers; wired the validator into `run_baseline_suite`; added a `--self-test` short-circuit in `main`.
- [scripts/run_all_tests.sh](scripts/run_all_tests.sh) — added `python3 "${SCRIPT_DIR}/run_verification_hardening.py" --self-test` between the diagnostic-docs sync check and the `sifr_diagnostics` Cargo test, inside the post-capture inner block so it runs in every lane (quick/pr/nightly/release).
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) — unchanged from pass 1: the slice 2 status line at [:75](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75) still reads "in progress: add verification harness duplicate-baseline artifact path detection before command execution or blessing so two variants cannot share the same checked baseline output."

The pass 1 review companion file [reviews/semantic-diagnostic-code-taxonomy-diag-5-baseline-guardrails-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-5-baseline-guardrails-review-pass-1.md) is on disk untracked; that is review evidence, not implementation.

## Verdict

**Satisfied — no must-fix blockers.** All actionable pass 1 findings (A, B, C, D, E, G) are addressed in the working tree, the self-test exits 0 locally, and the validator still runs ahead of both execution and `--bless` in `run_baseline_suite`. The remaining items below are informative or out-of-slice follow-ups; none gate the slice.

## Pass 1 follow-up coverage

| Pass 1 finding | Pass 1 disposition | Pass 2 status | Evidence |
| --- | --- | --- | --- |
| A — no automated test exercises the validator | should-fix | **Resolved.** `--self-test` flag added at [scripts/run_verification_hardening.py:83-87](scripts/run_verification_hardening.py:83); `run_self_tests` exercises positive + 3 failure modes at [:355-430](scripts/run_verification_hardening.py:355); short-circuited from `main` at [:1742-1743](scripts/run_verification_hardening.py:1742); wired into `scripts/run_all_tests.sh` at [scripts/run_all_tests.sh:105-106](scripts/run_all_tests.sh:105). Local invocation prints `verification hardening self-tests ok`. ✓ |
| B — `case_entry` not normalized | should-fix | **Resolved.** `baseline_case_metadata` calls `(repo_root / case_entry).resolve()` at [:304](scripts/run_verification_hardening.py:304) and `baseline_artifact_key` re-normalizes via `path.resolve()` at [:243-244](scripts/run_verification_hardening.py:243). The "normalized duplicate" self-test case at [:375-396](scripts/run_verification_hardening.py:375) directly exercises `fixtures/a/main.sifr` vs. `./fixtures/a/main.sifr` and asserts the collision message contains the resolved relative path. ✓ |
| C — `relative_to(repo_root)` can raise on absolute entry | nit | **Resolved.** `baseline_case_metadata` rejects absolute `case_entry` at [:289-290](scripts/run_verification_hardening.py:289) before any `relative_to` call, and wraps the resolved-path containment check in `try/except ValueError` at [:305-310](scripts/run_verification_hardening.py:305). The error path uses `format_repo_relative_path` at [:247-251](scripts/run_verification_hardening.py:247) so the collision message degrades gracefully to an absolute string instead of raising a second `ValueError`. The "absolute baseline entry" self-test at [:413-428](scripts/run_verification_hardening.py:413) covers the absolute-rejection branch. ✓ |
| D — duplicate-format-within-one-case reports `X:label and X:label` | nit | **Resolved (option 2 from pass 1).** `validate_unique_diagnostic_formats` at [:213-227](scripts/run_verification_hardening.py:213) raises a dedicated message ("lists diagnostic_format 'json' more than once") before the cross-case validator can fire, and is invoked from `baseline_case_metadata` at [:298-302](scripts/run_verification_hardening.py:298). Self-test asserts the new message at [:397-411](scripts/run_verification_hardening.py:397). ✓ |
| E — repeated per-case shape checks in detector and consumer | nit | **Resolved.** `baseline_case_metadata` at [:273-311](scripts/run_verification_hardening.py:273) is the single source of truth for `(case_id, entry_path, command_name, formats)` validation; `baseline_case_result` calls it at [:460-464](scripts/run_verification_hardening.py:460) and now only owns the consumer-specific `expect_exit_code` and `entry_path.is_file()` checks at [:467-470](scripts/run_verification_hardening.py:467). Future tweaks to the four-command set or the formats shape flow through `BASELINE_COMMANDS` ([:28](scripts/run_verification_hardening.py:28)) and `baseline_case_metadata` exactly once. ✓ |
| F — `actual` namespace duplicate-`case_id` not policed | informative, OOS | Still not policed; correctly out of slice 2 scope. ✓ |
| G — `run_fixedbugs_suite` re-derives label inline | informative, OOS | **Adopted as cleanup.** [scripts/run_verification_hardening.py:689](scripts/run_verification_hardening.py:689) now calls `baseline_variant_label(str(command_name), diagnostic_format)`, and the `command_name not in {"check", "run", "build", "test"}` literal at the same site became `command_name not in BASELINE_COMMANDS` at [:666](scripts/run_verification_hardening.py:666). One-line consistency win without behaviour change. ✓ |
| H — issue status text accurately scoped | informative | Still accurate; status line unchanged. ✓ |
| I — no stale references to old inline label expression | informative | `grep -n 'f"{command_name}-{diagnostic_format}"' scripts/run_verification_hardening.py` returns only the helper body at [:231](scripts/run_verification_hardening.py:231). The fixedbugs duplicate from pass 1's report is now also routed through the helper. ✓ |

## Contract verification (pass 2 re-check)

The slice's contract: "baseline suites must fail before executing or blessing if two cases/variants would use the same checked baseline artifact path (stdout/stderr/exit-code), so two fixtures cannot silently share one baseline file."

1. **Validator runs before execution AND before bless.** [scripts/run_verification_hardening.py:571-575](scripts/run_verification_hardening.py:571) calls `validate_unique_baseline_artifact_paths` at the head of `run_baseline_suite`, before the `print(f"  suite=…")` line and before any `baseline_case_result` (which is the only path that runs `cargo` or writes baselines). `args.bless` is not consulted at the validator gate, so blessing is also blocked on collision. ✓
2. **Path identity (not text identity) is what the validator enforces.** Both `baseline_case_metadata` ([:304](scripts/run_verification_hardening.py:304)) and `baseline_artifact_key` ([:243-244](scripts/run_verification_hardening.py:243)) resolve their paths through `Path.resolve()`, so `foo/main.sifr`, `./foo/main.sifr`, `foo/./main.sifr`, and `bar/../foo/main.sifr` collapse to one key. The self-test guards the prefixed-vs-canonical case directly. ✓
3. **Stdout, stderr, and exit-code artifacts are all keyed.** `validate_unique_baseline_artifact_paths` iterates every entry of `baseline_artifact_paths` ([:327-339](scripts/run_verification_hardening.py:327)), and that helper returns the same triple the writer/comparator at [:502](scripts/run_verification_hardening.py:502) consumes. Any artifact-path drift would have to change both sites in lockstep. ✓
4. **Label derivation cannot drift between detector and consumer.** Validator and `baseline_case_result` both go through `baseline_variant_label` ([:230-231](scripts/run_verification_hardening.py:230), used at [:328](scripts/run_verification_hardening.py:328) and [:482](scripts/run_verification_hardening.py:482)). `run_fixedbugs_suite` now also routes through the helper at [:689](scripts/run_verification_hardening.py:689), so the label format has exactly one definition site in the script. ✓
5. **Failure modes are clear `SystemExit` strings.**
   - cross-case collision: "suite '<name>' baseline artifact path collision for <repo-relative-path>: <previous-owner> and <current-owner>" ([:335-338](scripts/run_verification_hardening.py:335))
   - duplicate format within one case: "suite '<name>' case '<id>' lists diagnostic_format '<x>' more than once" ([:223-225](scripts/run_verification_hardening.py:223))
   - absolute entry: "suite '<name>' case '<id>' entry must be repo-relative" ([:290](scripts/run_verification_hardening.py:290))
   - escapes-via-`..`: "suite '<name>' case '<id>' entry must stay under repo root" ([:308-310](scripts/run_verification_hardening.py:308))
   
   Each is greppable and identifies the offending case. ✓

## Coverage of "any other runner path in this slice"

Re-audit of every runner against the new central helpers:

| Runner | Writes checked-in baseline artifacts? | Goes through `baseline_artifact_paths`? | Notes |
| --- | --- | --- | --- |
| `baseline` ([:560](scripts/run_verification_hardening.py:560)) | Yes | Yes — and gated by `validate_unique_baseline_artifact_paths` at [:571](scripts/run_verification_hardening.py:571) before the per-case loop | Contract honored. |
| `fixedbugs` ([:606](scripts/run_verification_hardening.py:606)) | No (only `target/verification/actual/...` on failure at [:717-720](scripts/run_verification_hardening.py:717)) | No — uses raw `actual_root` paths | Now reuses `baseline_variant_label` for label naming consistency. |
| `crashes` ([:758](scripts/run_verification_hardening.py:758)) | No (metadata-only) | No | Unchanged. |
| `property` ([:872](scripts/run_verification_hardening.py:872)) | No (in-memory determinism) | No | Unchanged. |
| `fuzz-smoke` ([:1115](scripts/run_verification_hardening.py:1115)) | No (writes to `target/verification/tmp/`) | No | Unchanged. |
| `oss-curated` / `ecosystem-broader` ([:1305](scripts/run_verification_hardening.py:1305)) | No (exit-code/panic checks) | No | Unchanged. |
| `determinism-scale` ([:1567](scripts/run_verification_hardening.py:1567)) | No (external-command exit checks) | No | Unchanged. |

Conclusion: the only runner that produces a checked-baseline namespace is still `baseline`, and the validator is wired exactly there. No missed runner path. ✓

## `run_all_tests.sh` self-test wiring assessment

Self-test placement at [scripts/run_all_tests.sh:105-106](scripts/run_all_tests.sh:105):

- **Lane coverage.** The block lives below the `SIFR_LANE_REPORT_CAPTURED` re-exec at lines 52-79 and above any conditional `if [[ ]]` lane gates, so it runs unconditionally on every lane (quick/pr/nightly/release and the legacy aliases). That matches Finding A's intent: a regression in any local validation lane fails fast. ✓
- **Cost.** The self-test is pure-Python with no `cargo`/`subprocess` calls; cost is sub-100ms locally. Adding it to `quick` is appropriate. ✓
- **Position relative to other guardrails.** It sits next to other harness-shape checks (`check_diagnostic_schema_sync.py`, `check_diagnostic_docs_sync.py`) at lines 99-103 and ahead of every Cargo invocation. A regression in the validator therefore short-circuits before any expensive build, which is the desired blast-radius. ✓
- **Failure surface.** `set -euo pipefail` is set at line 3, so a non-zero exit from `--self-test` aborts the lane. The capture wrapper at lines 52-79 propagates the inner status. ✓
- **No CI drift risk.** `AGENTS.md` declares `scripts/run_all_tests.sh` as the authoritative gate that CI mirrors, so wiring the self-test here means CI inherits it without an additional config file. ✓

The wiring matches Finding A's option 2 and is appropriate.

## Self-test coverage assessment

`run_self_tests` ([:355-430](scripts/run_verification_hardening.py:355)) covers:

| Scenario | Self-test case | Asserted message fragment |
| --- | --- | --- |
| Two distinct entries, same command/formats — should pass | [:357-374](scripts/run_verification_hardening.py:357) | (no exit; positive case) |
| Same fixture authored two ways (`./` prefix) | [:375-396](scripts/run_verification_hardening.py:375) | `fixtures/a/baselines/check-json.stdout.txt` |
| Duplicate format inside one case (`["json", "json"]`) | [:397-411](scripts/run_verification_hardening.py:397) | `lists diagnostic_format 'json' more than once` |
| Absolute `entry` (`/tmp/main.sifr`) | [:413-428](scripts/run_verification_hardening.py:413) | `entry must be repo-relative` |

The `assert_self_test_failure` harness at [:342-352](scripts/run_verification_hardening.py:342) demands both `SystemExit` AND the expected substring, so a future refactor that drops the message specificity (e.g., "validation failed") would fail the assertion. That is the regression-locking property the slice needs.

What is **not** covered today (informative — see Finding J):

- A repo-relative entry that resolves outside the repo via `..` (e.g., `../escape/main.sifr`). The "entry must stay under repo root" branch at [:308-310](scripts/run_verification_hardening.py:308) has no test case. Manually exercised, the branch fires correctly because `(repo_root / "../escape/main.sifr").resolve()` strips `repo_root`'s last component and `relative_to` raises `ValueError`, which `baseline_case_metadata` converts to `SystemExit`.
- A genuine cross-case collision where two distinct `id`s point to two distinct `entry` paths whose computed baseline directories alias each other (the only way this can happen given the layout is via filesystem symlinks; not a realistic manifest hazard).
- The "no `diagnostic_formats` key" branch where `parse_formats(None)` returns `[None]` and the default-format path is taken. The positive case in the self-test only exercises explicit format lists; the `None`-format path is exercised in production by `project` suite cases like `multi_module_run` ([verification/suites/manifest.json:25-30](verification/suites/manifest.json:25)) but not in the self-test.

These omissions are not blockers; the validator's behavior on those branches is exercised by the production manifest every lane run.

## Findings

### Finding J (nit) — self-test does not exercise the `..`-escape branch

[scripts/run_verification_hardening.py:305-310](scripts/run_verification_hardening.py:305) raises `SystemExit("... entry must stay under repo root")` for resolved paths that escape `repo_root`. Adding one self-test case with `entry: "../escape/main.sifr"` and asserting the message fragment "must stay under repo root" would lock that branch in alongside the absolute-rejection case. The failure shape is asymmetric today — absolute entries are rejected by string check, escaping relatives are rejected by post-resolve containment check — and only one of the two branches is regression-locked. Cost is one extra `assert_self_test_failure` block; benefit is symmetric coverage of the "entry must be inside repo" invariant. Not gating; the absolute case already exercises the broader contract.

### Finding K (informative — out of slice 2 scope) — cross-suite collision is still not policed

`validate_unique_baseline_artifact_paths` is called once per suite at [scripts/run_verification_hardening.py:571-575](scripts/run_verification_hardening.py:571), so its `seen` dict is suite-local. If two baseline suites in the same manifest reference the same fixture entry with the same command and format, their baseline directories alias because `baseline_artifact_paths` keys on `entry_path.parent / "baselines"` ([:234-240](scripts/run_verification_hardening.py:234)) and that parent is determined by the fixture path, not the suite name.

In today's manifest this is structurally avoided: the two baseline suites (`diagnostics` and `project` per [verification/suites/manifest.json:5-8](verification/suites/manifest.json:5), [:23-26](verification/suites/manifest.json:23)) live under disjoint subdirectories (`crates/sifr/tests/verification/diagnostics/...` vs. `crates/sifr/tests/verification/project/...`), so cross-suite alias is impossible by convention. But the slice's framing in the issue at [:1010](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1010) ("Two fixtures must not generate the same baseline artifact path") is broader than per-suite. Promoting the validator to a manifest-level pass — accumulating `seen` across all baseline suites before any executes — would make the guardrail enforce the broader invariant directly rather than relying on the directory convention.

This was already noted as informative/OOS in pass 1 (Finding F territory but for the checked-baseline namespace, not the actual-tree namespace). Not gating for slice 2; flagging so a future hardening slice can promote the scope when convenient.

### Finding L (informative) — `parse_formats` still coerces non-string list items via `str(item)`

[scripts/run_verification_hardening.py:205-210](scripts/run_verification_hardening.py:205) returns `[str(item) for item in raw]`, so a manifest entry with `"diagnostic_formats": [null]` becomes `['None']` — a string format that downstream variant labels would then propagate as `check-None`. This predates the slice and is not introduced by it. The new `validate_unique_diagnostic_formats` does not catch this because it sees the coerced strings as distinct from `None` (the no-formats case) and from each other. Out of scope; flagging only because the slice now centralizes format handling and a future cleanup could harden `parse_formats` shape-validation here. No action required for slice 2.

### Finding M (informative) — issue status line is in the right shape but stays "in progress" until merge

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75) reads:

> `milestone_diag_5` slice 2 in progress: add verification harness duplicate-baseline artifact path detection before command execution or blessing so two variants cannot share the same checked baseline output.

The "in progress" wording matches the workflow used by every other slice in the file (compare [:75](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75) to the merged-slice format at [:74](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:74)). After merge, the line will be flipped to "implementation complete and reviewer-satisfied" with the PR link, and a Claude implementation review entry will be appended around lines 113-115 alongside slice 1's entry. That is normal and not a stale-doc issue today. ✓

The status line still scopes the slice to duplicate-baseline detection only, not to the broader DoD bullets at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009) and [:1011](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1011), which remain explicitly deferred. No over-promise. ✓

## Net regressions vs. main

None. The diff against `origin/main` is purely additive and structurally a refactor:

- New helpers (`BASELINE_COMMANDS`, label/path/key helpers, `baseline_case_metadata`, `validate_unique_diagnostic_formats`, `validate_unique_baseline_artifact_paths`, `format_repo_relative_path`, self-test plumbing) introduce no behavioural change to existing call sites that they replace; the consumer body at [scripts/run_verification_hardening.py:481-555](scripts/run_verification_hardening.py:481) computes the same labels and paths it computed before pass 1.
- The added validator gate at [:571-575](scripts/run_verification_hardening.py:571) runs strictly before the existing per-case loop, so a manifest that was passing before continues to pass; only newly-detected collisions become hard errors.
- The fixedbugs runner refactor at [:666](scripts/run_verification_hardening.py:666), [:689](scripts/run_verification_hardening.py:689) replaces a literal command-set and inline label expression with shared helpers — same output strings, same failure conditions.
- The `--self-test` flag at [scripts/run_all_tests.sh:105-106](scripts/run_all_tests.sh:105) adds one ~tens-of-ms Python invocation per lane and short-circuits on regression; no existing step is removed or reordered.

## Recommended action plan for pass 3 / follow-up

No must-fix or should-fix blockers remain. Optional polish that would harden the slice without expanding its scope:

1. **Nit (Finding J)** — add a self-test case for `entry: "../escape/main.sifr"` asserting `"entry must stay under repo root"`. Symmetry with the absolute-rejection case; one extra `assert_self_test_failure` block.
2. **Out-of-scope follow-up (Finding K)** — promote `validate_unique_baseline_artifact_paths` to a manifest-level pre-pass that accumulates `seen` across all baseline suites, so cross-suite aliasing fails at the same gate. Belongs in the same hardening slice that picks up Finding F (actual-tree `case_id` collision detection from pass 1).
3. **Out-of-scope follow-up (Finding L)** — reject non-string entries in `parse_formats` rather than coercing via `str(item)`. Pairs naturally with the centralized normalization DoD bullet at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1009).

None of the above blocks merging slice 2. The contract the slice claims to deliver — duplicate-baseline artifact path detection before execution or blessing for the only runner that has baselines, with regression-locking via `--self-test` wired into the authoritative local gate — is delivered.
