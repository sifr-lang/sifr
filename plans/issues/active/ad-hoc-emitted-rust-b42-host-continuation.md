# 12K-B42-HOST — changed-host representative: terminal HOLD

Date: 2026-09-08. Owner: [3776](https://github.com/sifr-lang/sifr/issues/3776).

The ONE authorized changed-host canonical representative invocation **FAILED**.
Host admission proceeded to the first lexical case, whose first warmup exceeded
its unchanged 120000ms command timeout. No measured samples were taken. The
producer stopped; its dependent budget consumer was blocked and did not execute.
The remaining nine cases were not reached. All six rules variants passed.

This item is terminal/HOLD, not qualified, delivered or merged. No retry,
source repair, new review, Sifr gate, PR, merge or next-item work was performed.
The original B42 admission failure remains immutable and separately attributed.

## Ownership, source and reused evidence

- Sole owner: Socrates, task `01a08156-ea9d-73e0-80a4-156c70ed5b0f`.
- Independent remote clone: `/private/tmp/sifr-b42-host.FVuvOb/sifr`.
- Private sibling TMPDIR: `/private/tmp/sifr-b42-host.FVuvOb/tmp`, exported for
  every command after creation; `CARGO_TARGET_DIR` unset, private default target.
- Evidence directory E: `/private/tmp/sifr-b42-host.FVuvOb/evidence`.
- Exact source: `4a867cbde6d83e56ec7b66ca32887388dfa42803`.
- Exact base: `4b4cc339964baeeb6641e57dc669fef700a5fa24`.
- Own source ref: `codex/item12k-b42-host-FVuvOb`, pushed at the unchanged source.
- Own docs-only record ref: `codex/item12k-b42-host-record-FVuvOb`, above that source.
- Original source ref: `codex/item12k-b42-TzkTjs`; predecessor record
  `2d7f22f7510b78e627bed704a49de0eb5337fc0c` remains untouched.

The parent inherited worktree and predecessor were read-only. This clone owns its
index, refs and object store; there are no alternates or shared target execution.
Both named skills and project instructions/architecture were read. The explicit
host-continuation rules superseded generic implementation/review/PR/merge steps.
Registration, capacity, prelaunch and result callbacks went to parent and Police.

Predecessor terminal was fully read and SHA256-authenticated:
`/private/tmp/sifr-b42.TzkTjs/evidence/terminal.json`,
`17dac010d8415dc64c13725c1f684be570c20bee52dc64aef84a37c14b2043b0`.
Its inventory hash is
`62a9d0f8af2da5ce201b268b00e5ec757c9f026c8fcc21414e2dcbed2234139b`.
All 56 inventory files and 129 bound evidence/source/workload/binary paths passed
authentication. E/input-authentication.json binds that crosswalk, eight named
check receipts/logs and the exact-base/candidate review. None was gratuitously rerun.

Reused checks: formatter unit filter 6 PASS; formatter CLI filter 6 PASS; benchmark
self-test PASS; budget self-test PASS; package fmt PASS; HIR guard PASS; file-size
guard PASS; exact base/candidate diff check PASS. Original commands are preserved
in the authenticated predecessor registration/freeze and own authentication report.
The canonical area independently executes its required six rules variants.

Reused initial Opus verdict: SATISFIED, no blocking findings, zero remediation or
provider failures. [Original full review](https://github.com/sifr-lang/sifr/issues/3776#issuecomment-5586026207)
has SHA256 `ebf72c35898bc84068ef20a374c0809836592d1b6b1b419d9955303e7bae0351`.
This item made zero new review calls. B20/B22 exhausted review history is not reset.

## Necessary canonical cold preparation

The frozen canonical runner initializes both readiness flags false and calls Cargo
on first use even if binaries exist (`run_benchmarks.py:108-109,516-560`). A byte-only
copy does not prepare dependency/fingerprint state in a fresh independent target.
Using the standing necessary-preparation authorization, this owner initialized
only its pinned Ruff and WASI-Virt submodules, then ran serialized commands:

| Cold preparation, not performance evidence | Result | Seconds |
| --- | --- | ---: |
| `cargo build --locked -p sifr` | PASS, release PASS | 94.894 |
| `cargo build --locked -p sifr_frontend --bin frontend_query_bench` | PASS, release PASS | 45.719 |

Both commands used the authenticated supervisor and a 180-second execution limit.
No cleanup or shared-target copy was required. Capacity before first Cargo:
96GiB free, private target absent, no competing heavy process. Before qualification:
89GiB free, target 7.3GiB; local pmset reported AC Power, AC attached,83%,not charging.
That observation supplemented, and did not replace, canonical host admission.

Old binaries were authenticated as inputs but not used as the qualification
binaries. Actual freshly prepared private binaries were frozen before launch:

- `target/debug/sifr`: SHA256
  `ba05221d463c08e8c4ca2f1a5001e6d0088ef705d0eb1a09896ee0a23028c293`.
- `target/debug/frontend_query_bench`: SHA256
  `6c5e9793e16c50ad73115b181c4aaababfbc1845a9127e0a5091d722c4a3e4fc`.

Actual LSP selection is private `target/debug/sifr lsp --stdio`, as defined by
`developer_tooling/lsp_protocol.py`; no alternate binary or environment override.
Final collation authenticated both binaries, all frozen source/protected inputs,
workloads and ancestor configuration unchanged. The index remained clean at source.

## Single canonical qualification and failure evidence

```sh
export TMPDIR=/private/tmp/sifr-b42-host.FVuvOb/tmp
unset CARGO_TARGET_DIR
uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative
```

Actual generated invocation: `performance-representative-1788876840586332000`.
It was not forced or preassigned. The prelaunch binds the canonical generation rule
and all original 10 cases plus six rules, workloads/counts/exits/baselines/policies
and maximum three controlled attempts. No subset, special prewarming, cache-policy
change, threshold waiver, extra attempt or banked sample was used.

Authenticated predecessor `run_receipt.py`, `prelaunch.py` and `finalize.py` were
copied using apply_patch without textual changes; their relative root resolution
places all new artifacts here. Supervisor SHA256:
`4d6b17d72d7de79fd9b9a4d02ff57aea95a786a6cb591829a8dfd27a1841fd43`.
The effective limit remained1800seconds execution plus at most120seconds cleanup,
at most1920total inside original7200. Lower case/admission limits remained intact.

Command exit1, elapsed128.438716seconds, no enclosing timeout. Actual producer
diagnostic: `benchmark build-project-001-additional-modules timed out after 120000ms`.
Run root: `sifr/target/performance/bench-1788876840-33516` under this private root.
One attempt, status `error`; one first-warmup receipt, sample_index0,warmup=true;
duration120011.657ms, exit_code=null,timed_out=true. Zero measured samples,
zero accepted cases, no successful work metrics or certification.

The raw available warmup stdout and stderr were both zero bytes. Their recorded
SHA256 is `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
The stderr sidecar is labeled `decoded-text-utf8`; no unavailable bytes or work
counters are invented. Exceptional attempt evidence retained25 host snapshots
and the sample reference before the original exception propagated. All raw sample,
attempt, control-failure and area-result artifacts are bound by the summary.
Available empty output does not establish a deeper cause for the timeout.

Six rules PASS: benchmark-manifest, benchmark-runner-self-test, budget-policy,
budget-policy-self-test, trend-policy, trend-policy-self-test. The producer failed.
Budget-subset has argv=[],actual_exit_code=null and blocked-by-benchmark-subset;
it did not run or read stale output. No budget report was produced. Thus there is
no successful producer/budget same-invocation qualification to claim.

Nine cases not reached: build-single-file-001-break-continue;
check-project-004-project-graph; check-single-file-001-arithmetic;
diagnostic-non-regression-002-json-diagnostic-schema;
formatter-corpus-001-project-check; formatter-large-file-001-check;
incremental-local-loop-001-unchanged-file-update;
interactive-tooling-foundation-002-warm-diagnostics-query; lsp-query-003-diagnostics.

The earlier `performance-representative-1788874689585756000` remains a separate
failed initial AC admission, zero samples/attempts. Its outputs were never changed.

## Evidence, complete release and remaining scope

- E/prelaunch.4a867cbde6d83e56ec7b66ca32887388dfa42803.json:
  SHA256 `56261a304dafb56fc573bff8f956abf052f6e1322c4462b3e4a54bddad7106ea`.
- E/launch-binding.json:
  SHA256 `ec949cbe596ba36361442f874cb46fdb8a29825bc5c4693f218b3d09690e3f4e`.
- E/representative-summary.json:
  SHA256 `0b0795db213a66868c7a8d5343a49795209d5b6fd6fdd14a3f335d53017d739c`.
- E/representative-01.started.json, .json and .log preserve launch, command,
  root/descendant PID/start identities, separately created groups and full output.
- E/process-release.json verifies all three supervised command trees/groups gone.
- E/terminal.json and E/evidence-inventory.json bind final record SHA, remote refs,
  owner publication, documentation checks, all receipts and final release.

Complete release PASS: representative groups33395,33429,33436,33589,33597 retired;
remaining=[]; no external supervisor cleanup signals were needed. Canonical B23
timeout cleanup handled the timed-out work. Final read-only verification also
checked cold-preparation trees/groups; it did not signal foreign processes.
Original review group72937 remains absent. Root exit alone was not used as proof.

Blocker: first build-project warmup timeout under unchanged canonical policy.
Owner3776/compiler-performance and coordinator Police own any later assessment or
new acquisition scope. No new source mechanism defect is established by this
timeout alone; no repair or next-item implementation occurred here.

Formatter selection restoration remains proven by reused six unit/six CLI checks.
Historical zero-file CV cause remains UNKNOWN, and this run did not reach either
formatter case. Original B24 causal adjudication, B26 assessment and B27 joint
delivery remain held: full semantic stack264 companions,90native+411check,Python30,
exact65SQL,full area/toolchain gates and retained12D/12E/12F/full12/final12A closure.
No isolated/main-first delivery claim or waiver is implied. Parent/coordinator
owns the next scope; this owner stops after publishing this terminal record.
