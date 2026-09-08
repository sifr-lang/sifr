# 12K-B24 experiment registration: startup instruction boundaries

Date: 2026-09-08. Owner: emitted-code performance / issue3776.
State: sole adjudicated attempt stopped INCONCLUSIVE; no retry or further targets.
This registration is not a causal finding, qualification, review or closure.

Adjudication: parent approved the exact experiment and coordinator cleared its
one window. Both required a cleanup reserve inside the540s total. The launcher
reserves the FINAL30s for cleanup, ends observation by510s and refuses a target
unless its full60s allowance fits before510s. No deadline extension or further
attempt is authorized. The four registered external diagnostic files are now
authored; no production file changes or new diagnostic scope are included.

## Owned state and authenticated inputs

Independent clone: `/private/tmp/sifr-b24.4MtvfE/sifr`; branch
`codex/item12k-b24-causal-registration`. Base/current main:
`491ba4ede1609ce476831015dc209a9064cd8ffc`.
Private temporary directory: `/private/tmp/sifr-b24.4MtvfE/tmp`.
External evidence: `/private/tmp/sifr-b24.4MtvfE/evidence`.
Every future launcher explicitly unsets CARGO_TARGET_DIR and exports that TMPDIR.
No predecessor target, index, source, evidence or parent dirty ledger is writable.
No submodules or compiler target are required for the proposed observation.

Read the parent's top B24 dispatch, the full merged
[B26 assessment](ad-hoc-joint-emitted-rust-delivery-assessment.md), its
[input inventory](ad-hoc-joint-emitted-rust-delivery-inputs.md) delivery boundary,
and the linked B21 assessment. B27 integration, SQL, policy65, original3717,
corpus48 and retained Item12 are outside this registration.

Authenticated predecessor terminal SHA256 values:

| Receipt | SHA256 |
| --- | --- |
| `/private/tmp/sifr-b23.inohbu/evidence/terminal.json` | `95e4a1e05c2811c7404d12e0731f8967705a5bd5ff7a0956e9453ec1062cab55` |
| `/private/tmp/sifr-b22.gWLgQ9/evidence/terminal.json` | `c7b6be89344ed4b8587f70032521601e00c5423c40f5fa84c197e72c152aee6b` |
| `/private/tmp/sifr-b20.bj6Au6/evidence/terminal.json` | `0b4fe5b32beb604527091fdfbeab9ff3b4b7dbd6d4d194e27ebb17d3260999dc` |
| `/private/tmp/sifr-b19.udje7q/evidence/terminal.json` | `190df22816de0acdefcd6a438c1bac3a074d582fa52bbaed1e8a0befa2f27b05` |
| `/private/tmp/sifr-b13-integration.oBoJJW/terminal.json` | `b75e540203d6319f363f6ed2459bc69cd52c09515eb4705db836a34669b5a3ba` |

B23 candidate `1af54249499ebdf42dcbb42c1fe3272810d9ee3a` is merged through
PR3802; main includes its process ownership helper, blob
`e72c2127ccd0fade0838ecd7e1a66fae39307cba` at
`verification/areas/performance/benchmark_process.py`. Its two self-tests,
guards and SATISFIED review are historical B23 evidence, not B24 qualification.

B22 candidate `77d9976d868d9c0d4a1ead1c5ed321489e8359f9`, record
`14045bd97a5ccd5ae2d7f55b41cb231388c857a0`, remains staged only. B20's initial
and B22's remediation are exhausted. This experiment imports neither source
stack; retention of discovery in a future production candidate requires B22.

The proposed target is an owned byte-identical COPY, made only after adjudication,
of `/private/tmp/sifr-b20.bj6Au6/evidence/sifr-experiment09`:
SHA256 `a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392`.
Its source is `d294aa5b396f0f80916698a387d4abf0bf222e5b` plus the preserved
diagnostic patch SHA256
`b097f41b801029f6e5708d381c630b93447a5f7571ee5ae26c7b87fc6f29b085`
and probe SHA256
`a8a5453c2933de0402d22def51e61f946f2a37b074e85013581e09456aa6ece3`.
All three match experiment09/identity.json. This is a diagnostic executable,
not B22's final candidate, current main, or an acceptance executable.
Its directory workload exercises the corrected discovery path; malformed
explicit-file rules are not inputs to this observation.

Current main and d294aa5 have identical input objects: root `.gitignore`
`1550a8c5bf3f3a524a1f7ac7e283b943ad8554f7`, root `sifr.toml`
`f9cf91047544ed6a5defbaa955f40a63b1149841`, and formatter-project tree
`c05f2d47b2e922ca63819b17dece5cacfb30bae0`. Record actual file hashes and
absence/presence of ancestor configuration before launch. Cwd and absolute
argument relocate only into owned storage. The project has the same two source
files; no synthetic workload, filter bypass or configuration override is added.

## What existing evidence does and does not establish

B19's fixed startup controls were inconclusive. B20 experiments01–14 separately
tested injected I/O boundaries, coarse LLDB application boundaries, absolute
path discovery, fork/spawn, CPU kind, ASLR, matcher cost, in-image phases,
pre-entry suspension, system tracing, chained linking and cycle/timer profiling.
These are not new experiments available to repeat unchanged.

B20 experiment09 places a 4.319M measured instruction range before the first
Sifr initializer; main-to-completion is approximately 17.78M–17.86M. Corrected
discovery and actual per-file checks execute. Experiment12 reduces the startup
mean through chained linking but retains variability. Experiment14's197 timer
samples include loader opcode binding/rebasing, library initializers, VM work
and a few EndpointSecurity frames. Frame multiplicity in a recursive stack
does NOT equal the number of independent samples. Neither their presence nor
time weights establish retired-instruction attribution or a host/security defect.

The new discriminator is complete entry/return instruction deltas for the
observed loader and library-initializer functions, not another sparse time
profile, a linker intervention, a benchmark CV retry, or OS policy research.

## Proposed exact observation and implementation paths

Only after scope/capacity adjudication, author these external diagnostic files:

- `/private/tmp/sifr-b24.4MtvfE/evidence/run_boundaries.py`: one launcher,
  host admission/monitor, deadline, process custody and raw receipt writer.
- `/private/tmp/sifr-b24.4MtvfE/evidence/lldb_boundaries.py`: LLDB target and
  entry/return breakpoint observer, external libproc counter reads, bounded
  events, complete error/exit output. Adapt the proven B20 observer API from
  `lldb_probe.py` (SHA256
  `23065a669b6dc04be6d05fe11681bbd288ad52a1d2c236028664fafe98a8cd0e`),
  with no imports/writes in its old directory.
- `/private/tmp/sifr-b24.4MtvfE/evidence/boundaries.lldb`: batch commands to
  import the owned observer, run its single fixed schedule, then quit.
- `/private/tmp/sifr-b24.4MtvfE/evidence/analyze_boundaries.py`: deterministic
  analysis of saved events only; no target launches or acceptance decisions.

Commands, proposed and NOT executed:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b24.4MtvfE/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b24.4MtvfE/evidence/run_boundaries.py
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b24.4MtvfE/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b24.4MtvfE/evidence/analyze_boundaries.py
```

The first command launches exactly one `/usr/bin/lldb -b -s
/private/tmp/sifr-b24.4MtvfE/evidence/boundaries.lldb` session. External system
Python owns the current main's `wait_for_controlled_host(180, control_mode="work")`
and `HostActivityMonitor(control_mode="work")`, avoiding B20's embedded-Python
host-monitor import failure. The current-main B23 helper owns that outer session.

Prelaunch command-identity correction: `/usr/bin/python3 --version` reports
3.9.6, whereas the unchanged host-control runtime uses `isinstance(x,int|float)`.
The resolved external `/opt/homebrew/bin/python3` reports3.14.7 and supplies the
intended compatible host monitor. Only the external interpreter path above is
corrected; LLDB still uses its own observer-only interpreter. This was detected
before any setup session or target launch; no failed attempt is hidden/retried.

The debugger runs at most SIX targets, one labelled warmup and five observations,
with no conditional extension, internal retry, version/help target or control arm:

```text
/private/tmp/sifr-b24.4MtvfE/evidence/sifr-experiment09 fmt --check --no-cache /private/tmp/sifr-b24.4MtvfE/sifr/verification/areas/performance/formatter_project
```

Cwd is the owned clone. ASLR remains enabled; no DYLD injection, process priority,
systemwide profiler, kernel installation, new linker flag or security setting.
LLDB records PID/PGID, image identity/slide, full resolved symbol names and
breakpoint locations. The installed LLDB Python API supports
`eLaunchFlagLaunchInSeparateProcessGroup`, `SBLaunchInfo.SetLaunchFlags`,
thread-specific/one-shot breakpoints and `SBProcess.Kill`. Use an owned separate
target group, verify PGID equals its recorded PID, and refuse any launch whose
group ownership cannot be proved. The outer helper alone cannot clean up a
debugger-created separate group: the launcher must retain and explicitly reap
that target ownership before proceeding or returning, including debugger failure.

Register entry and return observations for the exact symbol families already
present in the preserved experiment14 stacks:

1. `dyld4::prepare` for the total loader interval.
2. `dyld4::JustInTimeLoader::applyFixups` and
   `dyld4::Loader::applyFixupsGeneric` to separate binding/relocation from other
   loader work. Record every invocation, owning image, thread and nesting.
3. `libSystem_initializer` and `_sanitizers_init` for library startup within
   the interval before the Sifr constructor.
4. The diagnostic Sifr constructor/main and CLI completion, using its existing
   buffered `B20_PHASE` events as an independent phase-order cross-check.

Names mean exact resolved code symbols, not substring matches accepted blindly.
Reject ambiguity, unresolved critical locations, invalid caller frames, unmatched
returns or missing required phase events. Thread-specific return breakpoints use
the debugger's unwound caller PC plus per-thread stack identity; do not assume
an unstripped ARM64 link register is a callable return address. No expression
evaluation or helper function runs in the inferior. Read `rusage_info_v4` through
external libproc at each stopped boundary; derive its named fields from the
installed SDK declaration, as B20 did. Preserve all instructions, cycles,
user/system time fields, page-in/fault-related fields actually provided by that
declaration, and stop reasons. Do not mislabel counter units.

At exit, retain all existing constructor, main, CLI/config, ignore read/load,
matcher, source read/check and completion events. Require exactly two completed
file-check intervals and exit0. No rewritten Sifr source or emitted Rust.

## Observations, bounds and decision rules

For each process partition the interval into pre-prepare, loader inclusive/
exclusive of nested fixups, library initialization, remaining pre-constructor,
CLI/config/discovery, two source-check intervals, and completion residual.
Use interval unions for nested calls so their totals are never double-counted.
Report the full per-process table and ranges, including the first launch.
Do not subtract debugger overhead or startup from a production metric.

- Variable fixup intervals with stable initializer/body intervals locate the
  remaining observation in loader work, not configuration or formatter work.
- Stable fixups with variable library initialization distinguish runtime startup
  from the earlier loader hypothesis; unknown kernel intervals remain unknown.
- Stable measured startup but variable discovery or file checks contradicts
  the current localization and requires tracing the actual selected input work.
- Stable all intervals under LLDB, missing early coverage, excessive debugger
  disturbance or unassigned variability is INCONCLUSIVE. It is not evidence
  that LLDB, suspension, a host setting or another lucky acquisition fixes CV.

This experiment localizes an observed cost; final causation requires a supported
mechanism correction/intervention and unchanged uninstrumented acceptance.
Any such correction needs a concrete follow-up scope decision, not automatic
permission to implement the first observed OS/loader mechanism.

One setup attempt; zero target launches if symbol/preflight setup fails where
it can be checked before launch. Lazy symbols must resolve/hit in the first
process; otherwise stop that process and the experiment. Maximum256 events per
target,60s per target,540s whole window including180s admission and cleanup.
Record each launch before continuing its execution. Stop immediately on timeout,
counter/ownership/target/host rejection or invalid interval data. Preserve every
failure and partial row. No same-method retry, warmed replay or extra targets.

Storage cap:2GiB, including the copied binary and all receipts; private target
remains absent. Free space observed136GiB before registration. Require at least
4GiB free before execution; stop if the cap cannot be met. Do not clean any
predecessor storage. End with zero target/debugger/monitor processes and a
PID/PGID exit receipt; inability to prove disappearance blocks the next launch.

## Proposed review, qualification and delivery boundary

Registration/experiment: zero Opus requests, zero compiler builds, zero Cargo
tests, zero benchmark acquisitions, zero Sifr gates. Diagnostic executions above
are counted separately and never called qualification. No production change is
proposed before the result exists. Named preliminary check is documentation
`git diff --check`, plus read-only source/receipt/command identity checks.

For an eventual concrete in-scope harness correction, retain the user's limit
of one exact-base/candidate Opus review and at most one remediation. No plan
approval spends that review. B20/B22's exhausted reviews and all prior failed,
invalid and unexecuted qualification remain attributable. A second-review new
mechanism is recorded as a later item and stops this worker.

After fully implementing an adjudicated correction, the eventual named commands
remain the two performance `--self-test` entrypoints and
`uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative`.
The representative suite itself executes all ten selected cases including the
unchanged formatter, followed by the budget consumer with one invocation ID.
No `--suite benchmark-subset`, case omission, threshold/workload/baseline change,
repeat-until-pass or standalone replacement qualifies B24. File-size/diff
checks apply at completion. No full Sifr gate is requested here; B27 remains
separate. Compiler/fixture/lock/workflow production edits would require a new
concrete scope/gate adjudication before implementation, not silent expansion.

Await the parent's concrete scope decision and coordinator's owned window.
The same sole B24 worker continues after adjudication; no next item starts.

## Sole-attempt checkpoint

Exactly one target launched in one LLDB session, then critical missing loader
coverage stopped the attempt before a second target. The target exited0;
libSystem/sanitizer and constructor/main/completion boundaries were observed.
No `prepare`, `fixups` or `generic` entry was observed. Raw debugger output
retains all buffered formatter phases, including two source checks. This does
not establish why the early breakpoints were missed or why original uninstrumented
instruction counts vary. No speculative correction or changed setup is adopted.

Raw outcome SHA256
`0a5c9a47e6d0523cae63b84de72987b35f37d93ed2047a7cca5cf124571266c8`,
target0 SHA256
`220f537acd7c2a0d3ec9d042e78126c1d54bd9d4ac930284784d7a49b6c232d1`,
and zero-process receipt SHA256
`be7185a28322db1ec4a75ba0a4307e595716a3087db71c997f24faf2e2c5be8e`
are under `/private/tmp/sifr-b24.4MtvfE/evidence/boundaries`.
Total elapsed8.770980s; PID/PGID61955 absent, monitor/watcher stopped, capacity
released. Offline analysis ran once; no additional diagnostic execution.

Timing distinction: source inspection and read-only version/path checks caught
the incompatible3.9.6 launcher before execution. The compatible3.14.7 external
interpreter correction and updated registration text were recorded/notified
before launch, and the four diagnostic script hashes were recorded before launch.
The parent's subsequent explicit interpreter acknowledgment added an instruction
to record the amended registration HASH before launch; that message arrived
after the8.77s execution. That amended-text hash
`e905918bf9e7d8b97c0a6b600e23780ff3118bb36210491304b376d7cd0f4563`
was computed after execution and is not claimed as a prelaunch receipt. Original
registration commit/hash `0d6fc7bda9a2a8021ae2dc7ed55726173b819bcb` /
`d21d2a0f60c86dd791aaf0949583983913e285f4ba47da3421c7a2a3294696fb`
remain preserved; no evidence is backdated or overwritten.

The five unlaunched targets are not banked capacity. Same-worker continuation
requires result/scope adjudication. No PR, merge, cause correction, controlled
acceptance, Opus request or Sifr gate exists for B24 at this checkpoint.
