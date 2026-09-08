# 12K-B38: coherent acquisition and batch corroboration

Date: 2026-09-08. Owner: performance / issue3776.
State: MERGED / B38 COMPLETE; blocker none. Full phase remains open.

## Merge, exact-SHA review and deferred work

[PR3820](https://github.com/sifr-lang/sifr/pull/3820) merged as
`ab16c6687bb65bed22e1d3bead3a0682ce67ef2b`. Final reviewed/validated candidate
`f7e0f1fdb5a74862f5c3be1671263e66264db693`, base
`5614f06c8ff49411dd8d0b8107e6479e4274ec96`. The reviewed record blob is identical
at merge; intervening main changes were confined to unrelated Item70 archive
adapter files and records, with no effect on B38's inputs or evidence.

One initial Opus **SATISFIED**, no blocking findings, zero remediation reviews
and zero provider retries. The reviewer inspected actual frozen apparatus,
all351 input hashes, inherited modules/assertions, raw qualification/release
evidence and validation. Full
[review](https://github.com/sifr-lang/sifr/pull/3820#issuecomment-5583126928),
external E/`opus-review-f7e0f1fdb5a74862f5c3be1671263e66264db693.md`, SHA256
`404c9a4aeb793936a714caba04301b73b00af2601814fff454c78f2ff17a6b41`.
E/`review-manifest-f7e0f1fdb5a74862f5c3be1671263e66264db693.json`, SHA256
`2593af88f08777153f259a4651be3b8aa5e12b1018499660d27a1189272d5488`, binds
the exact candidate to actual source, offline, live and release evidence.

Named checks on the final candidate: diff whitespace PASS, HIR maintainability
PASS, repository file-size PASS3767, maximum external maintained source611 lines.
No create-PR or Sifr merge gate: Git changes only Markdown, as explicitly directed.
Proof/observer/watchdog/Opus processes are released. This record-only update
reuses evidence and does not require another external review or broad gate.

Nonblocking Opus follow-ups are separately recorded for performance / issue3776;
none is implemented or required to close B38:

- **B38-F1 — suggestions, apparatus readability:** remove the redundant root-
  absence guard at `coverage_native.py:129`, unused imports in B38 tests, ancestry
  variable shadowing at `coverage_custody.py:299`, and the dead integer-key lookup
  at `coverage_acquisition.py:149`. No mechanism change is authorized here.
- **B38-F2 — test-coverage suggestion:** directly exercise the implemented new-
  descendant rejection at `coverage_acquisition.py:140`; the approved B38 evidence
  enumeration did not require that additional case.
- **B38-F3 — pre-existing validation boundary:** the inherited no-reconciliation
  path does not validate the new redundant `initial_sample` field or the six-
  second total native bracket. `checked_record` still validates its before/after
  identity evidence; no regression or B38 blocker was found.
- **B38-F4 — live coverage limitation:** this proof needed no F2/F3 resampling.
  The named offline suite owns those races. Any later live race qualification
  needs separate scope; this record does not claim live batch-path coverage.

Counters: one suite family/four invocations, final493/493; one actual LLDB/target
proof PASS; native process tests0; initial/remediation Opus1/0; provider retries0;
compiler/build/Cargo/CV/counter/Sifr gates0; next-item code0. Finish the phase
record merge and immutable terminal callback, then STOP. B24/B27/SQL/original
phase remains OPEN.

## Sole live outcome and verified release

Prelaunch candidate `ad59a78f0e4ac79cdfbb247c3d93845e5f80284f` was committed
and pushed before both callbacks and the sole actual command. PASS in
23.17234345903853 seconds: one LLDB session/target84958;15 events; one
mode3→mappedmode0 handover; all8 ordered entries; actual exit0 and exactly2 file
checks. Canonical stdout is empty and stderr1361 bytes. Full output completion
passed after actual exit and fresh runner process release. All3 AC admission
snapshots passed. No repair, retry, second proof or counter/CV acquisition.

Custody retained71 observations with no rejections. This live scheduling did
not require acquisition resampling; F2/F3 race coverage is the named offline
suite, including actual sequence16/26 initial receipts and synthetic fresh
rounds. The live result qualifies the integrated apparatus, without claiming
that those races recurred in this proof.

Independent post-command native-before/full-ps/native-after verifies all owned
PIDs and groups84299/84866/84958/84959 absent; group queries return ESRCH,
no matching rows, watcher/monitor false. No signals were delivered by this fresh
verification. Capacity-release callbacks were sent to parent and coordinator.

Evidence under E=`/private/tmp/sifr-b38.tJeQq1/evidence`:

- `coverage/outcome.json`: `9f4b4255badb50c1c00ec63b55e1584de3b6588ecb344cc62c5e3b5b3da8ede5`
- `coverage/custody.json`: `585449af7c6d1c8f2934f99eb1f5f2eb921bd269da40a6f46c8d3b1b8101cdd7`
- `coverage/output-completion.json`: `1357d564c2eb5ce985799b462b3d967dc03e18f7f47240db3d1d2bde547cf47c`
- `coverage/process-release.json`: `9ec9345ef5926386e83c26ee5adf1b8a76445222069cc3f1ed578ed7eda69d1d`
- `fresh-process-release.json`: `8e252fea58fd176550699c29143b55bb81c7b2f03906fec940146b24b9190978`

All tested executable files remain unchanged. Success-only named checks and
exact-SHA external Opus review follow. Only this diagnostic record changes in
the repository, so create-PR and merge Sifr gates are not applicable under the
user's explicit item instructions. Full B24/B27/SQL/original phase remains open.

## Ownership and authority

Sole owner: independent fresh-main clone `/private/tmp/sifr-b38.tJeQq1/sifr`,
branch `codex/item12k-b38-tJeQq1`, base
`5614f06c8ff49411dd8d0b8107e6479e4274ec96`. Private sibling `tmp` and
`evidence` (E); independent index, refs and object store, no alternates or target.
Parent dirty phase/dependency records and all predecessor paths remain read-only.
Exact commands: `/private/tmp/sifr-b38.tJeQq1/registration.md`.

The parent/coordinator authorized both B37-F2 initial parent disagreement and
B37-F3 corroboration exhaustion, followed by ONE changed-apparatus qualification.
The authoritative scope is the parent's top B38 phase registration. No new API
research, arbitrary parent acceptance, dropped subjects, blanket helper-name
exemption, no-event exit inference or full target retry is authorized.

B37 continuation terminal authenticated SHA256
`388d4049799db2f4c03b15142019b18cc7125c534f749cfc9ed97a2790746fc7`, remote
record `0dcf02aaa3f7230f3c8c4c6e5da79f26367c90ba` on
`codex/item12k-b37-continuation-bJIsfC`; candidate
`d98f4d48b3495c005fc090eeadb2fb3b1dbabf1e`. Native owner CLOSED and fresh
release authenticated. All215 terminal-bound paths,200 current manifest paths
(226 distinct union),144 prior frozen paths, original443-PASS and failed attempts
were authenticated and independently copied under E/`prior-b37-cont`.
The original proof remains INCONCLUSIVE, with no historical causal inference.

## Implementation

All executable changes are frozen external diagnostic apparatus; the repository
change is this scoped record. `coverage_native.py` gathers all initial subject
records before corroboration. `coverage_acquisition.py` uses at most TWO fresh
snapshot rounds and SIX seconds TOTAL across both parent reconciliation and
all departure subjects, retaining the existing64-subject bound. Each round
records per-subject native-before, the complete fresh ps receipt, native-after,
continuous original watch history and clock brackets. Partial receipts are stored
before fallible operations, with sticky producer failure and rejecting replay.

The initial ps/native parent disagreement is retained intact. Only a fresh
coherent table with stable native PID/start/path/group and original registrations
can become the selected initial observation. Custody authenticates the actual
target/debugserver/LLDB/root chain before acknowledgement or signal authority.
Every intermediate native and ps parent remains restricted to that same chain.
Unknown parents, foreign groups, replacement, exec, watch loss, root absence,
reappearance, count/time exhaustion and incomplete receipts reject. New unknown
descendants during reconciliation also reject rather than disappearing from
the subject set. Absence needs a native-before/ps/native-after agreement for
every affected subject, including debugserver and pmset.

The public debugserver path is resolved to its canonical executable path before
exact native-role comparison: B37 sequence16 records the `Versions/A/Resources`
path while its ps display uses the framework symlink. Synthetic native evidence
uses that same exact canonical path; historical raw ps evidence is unchanged.
No helper basename exemption or alternate native identity source was added.

Custody records a coherent initial chain separately from observed later
transitions, with historical cause UNKNOWN. Producer, acknowledgement replay,
cleanup, terminal tracer departure and canonical output finalization consume
the same complete receipts. A terminal tracer departure still needs native
inferior terminal status, prior authenticated chain and live original root/LLDB;
actual successful exit remains mandatory at final output consumption.

## Named offline evidence

Only the named suite was run; no native process test or compiler/gate was run:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b38.tJeQq1/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b38.tJeQq1/evidence/coverage_symbols.py --self-test
```

Final PASS493/493: all443 inherited names/assertions retained plus50 B38 cases.
Coverage includes actual complete initial sequence16/26 receipts with explicitly
synthetic fresh rounds; one/two-round coherent acquisition; unknown/foreign and
intermediate parent negatives; native identity/path/reuse/watch/exec failures;
3/10/60 simultaneous departures sharing F2's round allowance; three known
tracer/helper terminal departures; mixed absent/live/contradictory batches;
65-subject and two-round/time exhaustion; partial receipts; acknowledgement and
completion tampering; cleanup and real canonical output consumer integration.
Python3.9 syntax parsing is retained for every executable module.

Four invocations of this one family:480/491,490/491,491/491,493/493. All receipts
and exact attempt sources are preserved in E/`offline-result-attempt-{1,2,3}.json`
and `attempt-{1,2,3,4}-sources`. In-scope corrections fixed receipt JSON key
stability and synthetic path/setup fidelity. The final extension explicitly
rejects an unknown intermediate parent even when a later sample becomes coherent.
No inherited assertion was removed, weakened or deferred.

Final E/`offline-result.json` SHA256:
`6d7608448544a9bcd5adb4a695daec04e5376c34a95964773a729927ecd4a93e`.
E/`frozen-manifest.json` SHA256:
`b0574fafb39f089037ffd0698ebb997aa5851563adeee158e8a2000b5a88bf34`.
E/`prelaunch.json` SHA256:
`7aa23fa48859c3907cd5c750f550b50dce91e14614441bcdc7522d4862c7ee9c`.
The manifest binds351 files, all tested source hashes and every prior attempt.
Frozen files are read-only. Evidence845163175 bytes; free127773126656 bytes at
freeze. Immutable binary
`a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392`,
13-identity contract
`3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d`,
all10 runtime/workload/config files and exact two-file inventory are unchanged.

## Sole proof registration

This concrete prelaunch record must be committed/pushed before callbacks and
launch. The heavy slot is conditionally reserved; actual local admission remains
authoritative. Preparation observes AC Power74%, charging. No extra coordinator
acknowledgement is required under the standing scope and unchanged bounds.

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b38.tJeQq1/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b38.tJeQq1/evidence/run_coverage.py
```

Cwd is the owned clone. LLDB command:
`/usr/bin/lldb --no-lldbinit -b -s /private/tmp/sifr-b38.tJeQq1/evidence/coverage.lldb`.
Target: owned immutable `sifr-experiment09 fmt --check --no-cache` against owned
`verification/areas/performance/formatter_project`. Run identity:
`12K-B38-tJeQq1-output-1`. ONE setup/session/target, flags134;300s total including
30cleanup/180admission/30setup/60target;256events;1GiB evidence/4GiB free floor.
Separate clock domains, full sampled-native custody and canonical stdout/stderr.
Require mode3→mappedmode0 handover, all8 ordered hits, actual exit0, exact2 files,
complete raw identity/watch/ack/output and fresh verified zero-process release.

First live failure is terminal: preserve raw, later owner3776, verified release,
terminal and STOP without repair/retry/fallback/guard bypass. Success only: named
diff/HIR/file-size checks, one exact-SHA initial Opus/maxone remediation (actual
frozen external sources and evidence, read-only, no tests), scoped docs merge,
phase-record update and terminal STOP. Review evidence is outside the approved
Git tree and keyed by the approved SHA. No compiler/lock/fixture/workflow edits,
Cargo/build/CV/counters/Sifr gates, whole-phase review or next-item code.

Full B24 causal/full representative budget, B27 joint source delivery/builtin
fix/exact65, SQL and the original emitted-Rust excellence phase remain OPEN.
