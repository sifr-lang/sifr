# 12K-B42 — formatter restoration and warmup validation: terminal HOLD

Date: 2026-09-08. Owner: [3776](https://github.com/sifr-lang/sifr/issues/3776).

## Disposition

Implementation and named checks PASS; one exact-SHA Opus initial review
SATISFIED, no blocking findings. The one authorized canonical representative
invocation failed before any case sample because the unchanged controlled-host
admission reported `not-on-ac-power` after its existing 180-second deadline.
The dependent budget consumer was blocked and did not execute.

HOLD the reviewed candidate for original B27. This is not delivery, a main fix,
a representative pass, historical variance causal closure, or authorization for
another acquisition. No PR, create-PR gate, merge gate, or merge was performed.
No next-item implementation was started. Parent and all predecessor checkouts,
indexes, refs and sources remained read-only; the parent owns its phase ledger.

## Exact ownership and source

- Independent remote clone: `/private/tmp/sifr-b42.TzkTjs/sifr`.
- Remote: `https://github.com/sifr-lang/sifr.git` (fresh `--no-local` clone).
- Fresh main base: `4b4cc339964baeeb6641e57dc669fef700a5fa24`.
- Frozen source candidate: `4a867cbde6d83e56ec7b66ca32887388dfa42803`.
- Published source branch: `codex/item12k-b42-TzkTjs` (left at that SHA).
- Record-only branch: `codex/item12k-b42-record-TzkTjs`, based on that candidate.
- Own sibling TMPDIR: `/private/tmp/sifr-b42.TzkTjs/tmp`, explicitly exported
  for every command; `CARGO_TARGET_DIR` unset, own default `sifr/target`.
- Evidence root: `/private/tmp/sifr-b42.TzkTjs/evidence` (abbreviated E below).
- Independent index/object store; no alternates, shared Cargo target, or
  predecessor artifact cleanup. Disk before acquisition: 97 GiB free, private
  target 9.0 GiB; no cleanup required and no competing Cargo/build work observed.
- Source/lock finished before named tests; protected inputs and actual binaries
  were frozen before review/acquisition and authenticated unchanged afterward.

Both requested skills were read and used: `phase-closure-loop` for registration,
bounded evidence/review/record sequencing, and `talk-to-claude-opus` for the
read-only reviewer invocation. The explicit B42 HOLD override supersedes their
generic gate/PR/merge steps. AGENTS, architecture, current parent TOP, B41 full
assessment and eleven companion crosswalks, B26 assessment/input inventory, and
original B24 registration were read. `E/input-authentication.json` preserves the
full authenticated predecessor evidence crosswalk. Root, scope amendments,
prelaunch, review and qualification callbacks went to parent and Police.

## Implemented scope — ten registered paths only

1. `crates/sifr/src/check_and_package_commands.rs`: complete approved B22
   selection integration, with explicit-file bypass before malformed-rule load,
   lazy-once directory/forced-file loading, mixed-target error order and empty
   directory diagnostics preserved.
2. `crates/sifr/src/formatter_discovery.rs`: approved complete root-relative
   glob/component/negation semantics and B20 aff93 cached mandatory literal.
3. `crates/sifr/src/main.rs`: module declaration only.
4. `crates/sifr/Cargo.toml`: workspace `ignore` edge.
5. `Cargo.lock`: only the `sifr` dependency edge to already-resolved `ignore`;
   no old whole-lock replacement or other graph movement. Full locked offline
   Cargo metadata succeeded.
6. `crates/sifr/tests/formatter_discovery.rs`: all six retained CLI contracts.
7. `verification/areas/performance/run_benchmarks.py`: record every returned
   result, validate timeout and expected exit, then exclude valid warmups from
   metrics. Minimal command/query receipt hooks and named self-test integration.
8. `verification/areas/performance/benchmark_case_selftest.py`: responsibility-
   scoped deterministic warmup/result/query contract tests; runner stays below
   the 900-line guardrail.
9. `verification/areas/performance/sample_evidence.py`: explicitly coordinator-
   adopted B19-based receipt restoration, full available stderr sidecars with
   byte counts/hashes/representation labels, command stdout hashes and complete
   query JSON stdout sidecars; unchanged process return object and call policy.
10. `verification/areas/performance/controlled_sampling.py`: preserve accepted,
    rejected and exceptional attempt identity/results/sample references before
    propagation; acyclic copied results. No extra attempt or retry.

B20 `aff93db2ff199488d191b0e32584f9a4c4a0cbbd` and B22 final
`77d9976d868d9c0d4a1ead1c5ed321489e8359f9` semantics are reused/attributed, not
represented as newly discovered fixes. B22 PR3801 was staging-only, not main.
No diagnostic d294/probe/old compiler stack or B40 apparatus was used.
B23 `benchmark_process.py` remains blob
`e72c2127ccd0fade0838ecd7e1a66fae39307cba`; all real process-cleanup tests remain.
No fixture, manifest, budget, threshold, host policy, query policy, retry
parameter, or metric/acceptance protocol changed. The coordinator explicitly
approved the additional file-based evidence paths/hooks before their edits.

## Named validation on the frozen candidate

All eight receipts below have exit 0, no timeout, and process release PASS.
Each has full argv, timestamps, process identity and log/hash in the freeze.

| Command | Result | Seconds | Receipt prefix in E |
| --- | --- | ---: | --- |
| `cargo test --locked -p sifr --bin sifr formatter_discovery` | 6 passed | 104.054 | formatter-unit-01 |
| `cargo test --locked -p sifr --test formatter_discovery` | 6 passed | 8.647 | formatter-cli-01 |
| `python3 verification/areas/performance/run_benchmarks.py --self-test` | PASS | 4.343 | benchmark-selftest-01 |
| `python3 verification/areas/performance/check_budgets.py --self-test` | PASS | 0.806 | budget-selftest-01 |
| `cargo fmt -p sifr --check` | PASS | 0.761 | fmt-01 |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | 0.792 | hir-01 |
| `python3 scripts/check_file_size_guardrails.py` | PASS | 1.903 | file-size-01 |
| `git diff --check 4b4cc339964baeeb6641e57dc669fef700a5fa24 4a867cbde6d83e56ec7b66ca32887388dfa42803` | PASS | 0.181 | diff-01 |

Unit names (prefix `formatter_discovery_`):

- `root_ignores_do_not_match_host_ancestors`
- `uses_globs_components_and_negation`
- `disabled_ignores_select_all_files`
- `malformed_rules_keep_source_line_diagnostics`
- `disabled_ignores_do_not_parse_malformed_rules`
- `literal_filter_agrees_with_complete_engine`

CLI names (same prefix):

- `absolute_and_relative_paths_detect_the_same_drift`
- `preserves_explicit_file_and_exclusion_controls`
- `explicit_files_bypass_malformed_ignore_rules`
- `directory_and_forced_files_report_malformed_ignore_rules`
- `no_respect_gitignore_bypasses_malformed_rules`
- `mixed_targets_load_rules_at_directory_selection`

Benchmark self-test retained B23 ordinary exits 0/7, unreaped-group failure,
cooperative/resistant/leader-exit/closed-pipes tree cleanup and no overlap. Added
warmup timeout/nonzero/later-warmup failure, measured timeout/nonzero, allowed
exit sets `[1]` and `[0,1]`, exact manifest/smoke counts and warmup exclusion,
query aggregate/work/internal-warmup/timeout/invalid-JSON receipts and unchanged
return identity, bytes/text full stderr/hash binding, and accepted/rejected/
exhausted/exceptional attempt persistence with acyclic JSON. No following
measurement or retry occurs after a warmup failure.

Cold preparation was separate, not performance evidence: locked `sifr` build
24.333 seconds and `frontend_query_bench` build 49.511 seconds, both PASS.
No broad codegen, workspace Clippy or other speculative gate was run.

## One exact-SHA review

Initial review: **SATISFIED**, no blocking findings. One successful provider
call, zero failed provider attempts, zero remediation reviews.
Exact base/candidate as above; all ten source blobs and frozen named evidence
were checked. No reviewer-run tests or writes. Settings: `claude-opus-5`, medium,
project settings only, plan permission mode, no session persistence.

[Full review published outside the candidate tree](https://github.com/sifr-lang/sifr/issues/3776#issuecomment-5586026207).
Local keyed response: `E/opus-review.4a867cbde6d83e56ec7b66ca32887388dfa42803.md`,
SHA256 `ebf72c35898bc84068ef20a374c0809836592d1b6b1b419d9955303e7bae0351`.
B20 initial plus B22 remediation remain historically exhausted; this review
uses only the expressly granted changed-input B42 integration allowance.

Nonblocking observations are retained without implementation: lexical receipt
listing order for indices above 9; future raising runner contracts may lack a
returned-result receipt; full-scale receipt disk growth not yet measured;
inherited per-file matcher construction and physical-root/symlink assumptions;
workspace Clippy belongs to B27. Full review, including qualification caveats,
is preserved rather than replaced by this summary.

## Exactly one canonical representative acquisition

```sh
export TMPDIR=/private/tmp/sifr-b42.TzkTjs/tmp
unset CARGO_TARGET_DIR
uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative
```

Actual invocation: `performance-representative-1788874689585756000`.
Root PID/PGID 86524; producer PID 86619. Started Unix 1788874684.9899561,
ended 1788874870.769411; elapsed 185.775355 seconds; exit 1.
The unchanged initial admission hit its 180-second timeout. The external
supervisor did not time out: existing 1800-second area execution limit plus
at most 120-second final cleanup stayed inside the enclosing 7200 allocation.
Lower per-command/case/admission limits remained intact. Prelaunch recorded
the theoretical summed maxima, accepted by the coordinator as not guaranteeing
completion within the unchanged enclosing deadline. No silent extension.

All six rules variants passed: benchmark-manifest, benchmark-runner-self-test,
budget-policy, budget-policy-self-test, trend-policy, trend-policy-self-test.
Producer diagnostic (verbatim):

```text
performance benchmark error: controlled host admission timed out after 180s: not-on-ac-power
```

The producer exit was 1. Budget consumer: `argv=[]`, `actual_exit_code=null`,
`mismatches=["blocked-by-benchmark-subset"]`; it did not execute or read stale
results. Area totals: 8 variants, 2 blocking failures (producer and blocked
consumer), 0 nonblocking failures. The passing budget-policy rules variant
is not a successful budget check of this invocation's benchmark results.

All ten registered cases were NOT REACHED:

- build-project-001-additional-modules
- build-single-file-001-break-continue
- check-project-004-project-graph
- check-single-file-001-arithmetic
- diagnostic-non-regression-002-json-diagnostic-schema
- formatter-corpus-001-project-check
- formatter-large-file-001-check
- incremental-local-loop-001-unchanged-file-update
- interactive-tooling-foundation-002-warm-diagnostics-query
- lsp-query-003-diagnostics

Zero case attempts, zero warmup/measured samples, no work metrics or sample
exits/timeouts, no certification, no benchmark report or run root. The original
maximum-three controlled attempts were preserved but never reached. There was
no subset acquisition, special prewarming, external retry, policy bypass, or
additional host-sensitive experiment. Initial admission's internal snapshots
are not persisted by the unchanged host helper; the retained failure is its
actual diagnostic and complete available enclosing log/process/area receipts,
not an invented snapshot history or proof about every individual observation.

Actual prepared binaries (unchanged after acquisition):

- `sifr/target/debug/sifr`: SHA256
  `be1537096b151a075497dac1e97a67d9df08aee2d112fb48f68f68c3a3d06a8e`.
- `sifr/target/debug/frontend_query_bench`: SHA256
  `f71f2d634c186e9f3c44904f099da359ac1f5cd08435806ac5f65619e8f0ee70`.

Process release PASS: registered root, observed descendants and separately
created B23 test groups disappeared. Supervisor cleanup signals were unnecessary;
remaining list empty. Review/watchdog group 72937 was absent before launch.
Terminal inventory binds final release verification as well.

## Frozen evidence

- `E/registration.md`: exact paths/commands plus coordinator-adjudicated
  receipt/query amendments recorded before relevant code edits.
- `E/input-authentication.json`: complete B41 eleven-companion/source crosswalk.
- `E/freeze.4a867cbde6d83e56ec7b66ca32887388dfa42803.json`: source, protected
  input and eight named check receipt/log hashes; SHA256
  `49121d3745707633c036768709e01a24a385991ff02c75de209ba1c0d6f5f741`.
- `E/prelaunch.4a867cbde6d83e56ec7b66ca32887388dfa42803.json`: full case/command/
  timeout/count/workload/binary resolution; SHA256
  `71932b6d78cc555c5ae87f7f5c840a1762a48c1c19e49cdcf51ef96dad260f82`.
- `E/launch-binding.json`: exact supervisor source/hash/argv, source/test/review
  bindings, owned distinct-group cleanup contract and capacity; SHA256
  `ae9dfd4181feeb379b90aaad0527fd528841ad47c4ebf902518d89b3d50a5ee3`.
- `E/representative-01.started.json`, `.json`, `.log`: actual launch and all
  available failure/process-release evidence.
- `sifr/target/verification/areas/performance-results.json`: complete canonical
  variant argv/statuses, blocked budget and actual invocation identity.
- `E/representative-summary.json`: authenticated post-run source/binary identity,
  every case/variant status, absence of samples and budget; SHA256
  `3aa1ebb680328daa8d7de59a2509aac0299fe8a6597bf717b95a05c7801be254`.
- `E/terminal.json` and `E/evidence-inventory.json`: final record SHA, published
  owner references, all evidence hashes and final owned process release.

## Causal disposition and original B27 handoff

Formatter selection restoration is proven by the retained unit/CLI negative
contracts on this candidate, including absolute/relative drift equivalence.
B41's same-old-binary absolute 0 / relative 1 result establishes the original
zero-file workload omission; it does not establish the cause of historical CV.
The warmup validation defect is independently fixed and tested; it was not the
original formatter cause because all 18 historical sample exits were zero.
Original B24 historical variance causal closure remains UNKNOWN/unresolved.

B27 readiness recommendation: source input is review-ready to retain, but
performance qualification is NOT READY. Owning blocker is controlled-host AC
power admission, under compiler/performance owner3776 and coordinator Police.
Fact-based next step for the coordinator: provide the unchanged required AC
power condition and explicitly adjudicate any later acquisition within original
B27; B42 does not retry and does not relax policy or claim source causality.
The coordinator separately authenticated this summary and reported actual
battery 93%, discharging. Any changed-host continuation belongs to a fresh owner
only after this terminal/native close and actual AC release; this session does
not start it. This coordinator report is distinct from the producer diagnostic.

Original B27 fresh-main joint full semantic H/I/M1/12B plus B25, exact 65 cases,
source264 companions, 90 native plus 411 check, Python30, all matrix/SQL,
retained12 and full-phase obligations remain mandatory. Full same-invocation
ten-case representative/budget and original B24 causal adjudication are not
waived. Compiler/lock changes still require original B27 full merge qualification
before delivery. No B27 implementation, gate, PR or merge starts in this item.
