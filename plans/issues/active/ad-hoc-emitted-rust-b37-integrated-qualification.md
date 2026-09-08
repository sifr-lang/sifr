# 12K-B37: integrated native-custody qualification

Date: 2026-09-08. Owner: performance / issue3776.
State: TERMINAL INCONCLUSIVE after changed-host continuation; unmerged, STOP.

## Continuation terminal outcome

Registered/pushed candidate `d98f4d48b3495c005fc090eeadb2fb3b1dbabf1e` ran
exactly once. All three admission snapshots passed on AC with nominal thermal
and no rejections. One LLDB session/target; elapsed9.842366958037019s. The observer
stopped during launch before acknowledgement, events, handover or entry hits,
then reported Kill success. No successful target exit or exact2-file output
completion was proved. The original battery admission below remains failed.

Later owners B37-F2/F3 are recorded in
[issue3776](https://github.com/sifr-lang/sifr/issues/3776#issuecomment-5582457963).
No repair/retry, guard weakening, review, PR or merge follows this live failure.

B37-F2: native sequence16 rejected first target36592 because ps PPID36554
(LLDB) differs from native before/after PPID36593 (debugserver). PGID36592
agrees. Native target start1788858610.861809/path remain equal across its
registration and before/after samples. Native tracer36593→LLDB36554→root35993
is recorded. `first:null` means no prior accepted target identity. No atomic
transition, historical cause, replacement or conflict acceptance is inferred;
cause UNKNOWN. Full first rows and native records are retained. ps bracket
510165.395965791→510165.447953541, native capture bracket
510165.395963041→510165.449392083; source call order is ps followed by native
sampling/registration/re-sampling, without invented individual sample timings.

B37-F3: cleanup sequence26 exhausted the unchanged two-corroboration limit.
Complete fresh chains for36554 and36592 retain initial/nativeBefore/nativeAfter
None with fresh ps absence, brackets510167.559749708→510167.608136166 and
510167.609517375→510167.657458791. Full capture bracket
510167.158748625→510167.658408541 also contains pmset36633, PPID/PGID35993.
This does not establish that pmset caused exhaustion. Full partial records and
later sticky failures remain for coordinator adjudication; no mechanism change.

Runner `groups_absent:false`, `zero_processes:false` remains unchanged. Fresh
post-terminal native-before/ps/native-after independently proves launcher35993,
LLDB36554, target36592 and debugserver36593 absent; all four groups return ESRCH,
ps has no matching PID/group/binary, watcher/monitor false. Fresh check delivered
no signals. Capacity-release callbacks went to parent, coordinator and dependency
parent after verification. Release does not relabel the failed proof.

Continuation evidence root `/private/tmp/sifr-b37-cont.bJIsfC/evidence`:

- `coverage/outcome.json`: `7f3503bf86b4cab69a7f10128c4951b519025bb96801dd546e0732e0872b772e`
- `coverage/custody.json`: `ae6ba64cf9027bb18042d0e47ab88e4c6ec03f3b0d2e6ce08614caae1ff6d904`
- `coverage/admission.json`: `cc12fe2ddb0812b26bc1b0942eb57ce4194013e5d99409c61e56ee8dedf8adbf`
- `coverage/process-release.json`: `1dd59a00bd588535631e692cf8c315772d62eed726ea45a63bed2f980538dd82`
- `coverage/inventory.json`: `826bef376306e40023c54fefd32fa037fd751bbcac6b6a5028ef060cc63af405`
- `failure-evidence.json`: `223c1e01b8aa0952e2b3dcd31138ccd7b3a93a0f40cb5030620b76e71c3e8d98`
- `fresh-process-release.json`: `c202c2a4520b11c5c4be2d9aa7d35892d7a5621f6c0e69c4d8dad16975e9e0ca`

Failure evidence binds the complete first rejection, full sequence26/corroboration
chain, all raw coverage hashes and fresh release. Original144/current200 frozen
inputs and original terminal authenticated unchanged after execution.443/443
offline evidence reused; zero reexecution. Across B37: two preserved original
suite invocations, original failed admission plus one changed-host attempt,
one actual LLDB/target total, zero events/handover/hits/reviews/PRs/merges/gates.
Success-only diff/HIR/file-size checks were not entered. Frozen external terminal
`terminal.json` follows this unmerged record commit. Parent/predecessor files and
refs remain untouched. Coordinator owns later adjudication; STOP, no next code.

## Changed-host continuation registration (2026-09-08)

The original battery admission below remains failed. Parent/coordinator approved
one continuation after actual AC restoration and Item70 bulk/F1B release.
PR3810/3811 and3814/3815 merges independently authenticated. Fresh local power
receipt reports AC Power33%, charging. The reserved host-sensitive slot is owned
by this worker; actual local admission must pass without retry or guard changes.

Independent fresh-main clone `/private/tmp/sifr-b37-cont.bJIsfC/sifr`, base
`5614f06c8ff49411dd8d0b8107e6479e4274ec96`, branch
`codex/item12k-b37-continuation-bJIsfC`, private sibling `tmp` and `evidence` (C).
No alternates, target, shared index or object store. Parent/predecessor roots,
records, indexes and refs remain read-only. Exact commands and bounds are in
`/private/tmp/sifr-b37-cont.bJIsfC/registration.md`.

The complete original terminal,144 frozen inputs and tested source identities
were authenticated.149 prior evidence files were independently copied under
C/`prior-b37`, including both offline attempts and the failed battery admission.
All19 active apparatus files are mapped to copied runtime paths. Seven files
change only the registered old-root string and/or run identity; each substitution
is byte-reversible to the original tested source. The remaining12 are
byte-identical. No mechanism, assertion, bound, flag or guard changed. Original
443/443 evidence is reused, with zero offline reexecution or native process tests.
Relocated source hashes are separately recorded, not claimed to be the original
tested hashes. Binary,13-identity contract,10 workload/config/runtime files and
two-file inventory authenticate unchanged; no ancestor config exists.

C/`authentication.json` SHA256
`176a0a5a355e933a22a91533fb937849cb0fef6fd07ca5e23f654e1a45ff3228`
records source/destination identities, reversible substitutions, copied receipts,
unchanged runtime and actual AC observation. C/`frozen-manifest.json` SHA256
`8e12ae8efe670c254cb45965902c18f7fade3a26569b4b2ad78ccdf0d53fc3f4`
binds200 files. C/`prelaunch.json` SHA256
`bfa3f299c7019c7908f8bbd82a7bdeaf133c99cae06e38c2bedb54a83f9f7c5c`
binds the exact new runtime command and identity `12K-B37-cont-bJIsfC-output-1`.
Frozen evidence occupies616777998 bytes, below1GiB, with121GiB free observed.

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b37-cont.bJIsfC/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b37-cont.bJIsfC/evidence/run_coverage.py
```

Run only after this concrete registration is committed/pushed and callbacks sent.
All original bounds and success criteria below apply. This is the same B37 item,
with one prior failed admission preserved and at most one actual coverage target.
First continuation live failure is terminal: retain raw, record owner3776 and
stop without repair/retry. Success only: named diff/HIR/file-size checks, initial
exact-SHA Opus and at most one remediation, scoped docs evidence merge, phase
record and terminal. No Cargo/build/compiler/fixture/lock/workflow changes,
Sifr gates, counters/CV, whole-phase review or next-item code. B24 causal/full
budget, B27 joint delivery/builtin fix/exact65, SQL and original phase stay OPEN.

The sections below preserve the original integration, test and failed-attempt
record verbatim; their original ownership and terminal actions are historical.

## Ownership and approved scope

Sole implementer owns independent fresh-main clone
`/private/tmp/sifr-b37.qPJOjN/sifr`, branch
`codex/item12k-b37-integrated-qualification`, base
`7339aed05db02b3de95b7930e5e1291edecd07fe`. Independent index/object store,
no alternates, shared target or Cargo target. Private sibling `tmp` and external
`evidence` (E); exact commands registered in owned root `registration.md`.
Parent dirty phase/dependency records and all predecessor data remain read-only.

Coordinator approved complete B36-F1/F2/F3 adoption before ONE combined proof.
B36-F4 wording remains separate. B36 predecessor Sagan is native CLOSED.
Terminal SHA256 `b8ad20e9207530e784e98e4d3723de5ff5c5293fda7156f0f92b5ae26bbfb34f`;
all48 frozen sources authenticated against manifest
`4eba23d064c0dc568fbb71debdd121a31ab2d2a92f72c81acd5fcafda8160dc5` and
independently copied. Remote PR3808 candidate
`e2d42f05dec4cce446ac1a33beccd68080eadf20` merged as
`aa11ed8a8dfe808b8727863ffe6c83e7ca4a29fe`; record PR3809 candidate
`0a62fd364ba7424be8b2fd2da1052cf90bf07048` merged as the current base.
The B35 prior proof remains INCONCLUSIVE and historical native cause UNKNOWN.

## Complete integration

The copied B36 module resolver, sampled native identity/watch custody,
acknowledgement replay, bounded cleanup and canonical output contract are retained.

- B36-F1: `coverage_acquisition.py` records initial ps/native disappearance and
  a fresh native-before/ps/native-after corroboration. At most two corroborating
  rounds per capture, six seconds total acquisition budget, 64 subjects, all
  within the sole outer proof window. Raw initial rows and full corroborating
  clocks/rows/samples remain in the custody history. Contradictory presence,
  stale/order/budget failure and PID reappearance reject persistently.
  Unregistered transient absence is diagnostic only, never a signal identity.
  Consumers validate corroboration before reconciling vanished rows. Registered
  identities retain their original registration and cumulative watch history.
- B36-F2: a missing tracer is accepted only for a native zombie inferior with a
  previously authenticated attach transition, unchanged subject identity/group,
  freshly absent authenticated tracer, and unchanged live original LLDB/root.
  Native status5 is the terminal observation; ps display or absence of a kqueue
  event does not establish termination. Missing-tracer histories never authorize
  live acknowledgement or signals. The eventual output consumer still requires
  actual successful inferior exit, complete protocol and fresh native absence.
- B36-F3: the ps producer owns its spawned process handle, records its actual
  PID, and reaps it including timeout cleanup. Native collection and final release
  exempt only that capture's PID with `/bin/ps` display. Another ps descendant
  receives normal native acquisition and remains visible at final release.

Replacement, native contradictions, watch loss, unknown chains and foreign groups
remain rejecting. Accepted public sampled-native API limitations are unchanged:
no atomic table, no atomic check/signal, no pre-watch history or no-event liveness.
No generation-API research, compiler/lock/fixture/workflow edit or next-item code.

## Named offline evidence and frozen candidate

After complete integration, only this suite family was executed:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b37.qPJOjN/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b37.qPJOjN/evidence/coverage_symbols.py --self-test
```

PASS443/443: all414 inherited names/assertions retained, plus29 acquisition-race,
terminal-tracer and actual-probe-PID producer/ack/cleanup/finalization cases.
No live process tests. Python3.9 parse coverage includes both new modules.
Two invocations of the same suite: first441/443, then443/443. Original failing
receipt and exact source copies retained in E/`offline-result-attempt-1.json`
and `attempt-{1,2}-sources`. Corrections removed a spurious cleanup conflict on
already-delivered exit and corrected the synthetic native-return injection timing.
No assertion was dropped, deferred or made permissive.

Final E/`offline-result.json` SHA256
`120f26aa7e13a524acf887c46a5d6845bd059bbc9f9e5eab17d6bd84c600f408`.
Full E/`frozen-manifest.json` SHA256
`98359e20861fdf6a823a93c520c673cdc9028543bbc2b49e194836cb3c025f67`
binds144 files: tested apparatus, predecessor copies, raw inputs, receipts,
registration, unchanged runtime helpers/workload/config and immutable binary.
E/`prelaunch.json` SHA256
`09da1b2c0d45587305650b38c44b23c0d3e5e359c9c5e6aa35d4a9320f379afa`
binds those files and complete exact command/admission limits. Frozen owned files
are read-only. Later review must inspect the actual external executable sources,
not only this Markdown record; review evidence stays outside the approved tree.

Immutable binary SHA256
`a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392`;
13-identity contract
`3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d`.
Two `.sifr` inputs and both selected configs authenticated unchanged against the
B35 prelaunch retained/authenticated by B36. No ancestor config exists.

## Sole live proof registration

This concrete prelaunch must be committed/pushed before callback and actual run.
Host capacity is conditionally reserved; notify coordinator before acquisition,
with no further permission checkpoint when standing contract/admission match.

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b37.qPJOjN/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b37.qPJOjN/evidence/run_coverage.py
```

Owned cwd above. Debugger command:
`/usr/bin/lldb --no-lldbinit -b -s /private/tmp/sifr-b37.qPJOjN/evidence/coverage.lldb`.
Target: owned `sifr-experiment09 fmt --check --no-cache` against owned
`verification/areas/performance/formatter_project`.
ONE setup/session/target, flags134; 300s total including30cleanup,
180admission/30setup/60target; 256events; 1GiB evidence/4GiB free floor.
Separate outer and embedded clock domains; exclusive canonical stdout/stderr.
Require one mode3→mappedmode0 handover, all8 ordered entries, actual exit0,
exact2-file completion, complete raw identity/watch/ack/output receipts and
fresh zero-process release.

First LIVE failure is terminal INCONCLUSIVE: preserve raw, record later owner3776,
no repair/retry/fallback/guard bypass. Success only: named diff/HIR/file-size checks,
one exact-SHA initial Opus plus at most one remediation, scoped docs evidence merge,
phase record and retirement. No compiler/Cargo/build/CV/counters/Sifr gates or
whole-phase review. Release capacity to dependency Item70 only after verifying
owned proof processes absent; notify its parent, coordinator and parent orchestrator.

B24 actual causal/full representative budget and B27 complete joint source
delivery with builtin fix/exact65 remain OPEN. This diagnostic proof does not
unblock SQL or close the original emitted-Rust excellence phase.

## Terminal outcome: B37-F1 host admission

The registered sole attempt ran after concrete prelaunch commit/push
`b250dd59bae5946b5eec49ce669bb9c2dfb78dd4`. It stopped on the first admission
snapshot with `not-on-ac-power`: host power source `battery`, required AC true.
The raw admission reports no competing processes and nominal thermal status.
Elapsed0.7924854999873787s; INCONCLUSIVE. No LLDB setup/session, target launch,
attach, continue, event, handover or entry hit occurred. No output completion was
claimed. This is a failed registered attempt, not unused permission to retry.

E/`coverage/outcome.json` SHA256
`7dc4dd2a8e7d3fd5da728a97521f16ef85e496d1c78c38862320a679df23eb89`;
E/`coverage/admission.json`
`7f1c426e43167d6873b779b19ab8a48e81b425aa94475c1f836de5cf554e694a`;
E/`coverage/process-release.json`
`6a7ce93dafe80e36b5ea54a9349f6a30ed981a8595ecf7ae5be83ef2fa226b80`;
E/`coverage/custody.json`
`002a6cbc6b8ea920252e6bbebe05a4783207385299b36c0dcc41a96db9e4bb18`.
Complete raw inventory and original sources/failed/final offline receipts retained.
The frozen144-file manifest remains authenticated; live execution made no source
changes and did not mutate parent/predecessor records.

Release: owned groups[], remaining[], groups_absent=true, watcher_alive=false,
monitor_alive=false. Launcher88318 independently verified absent after command
completion by `/bin/ps -p 88318 -o pid=,ppid=,pgid=,stat=,comm=` (exit1, empty).
Explicit capacity-release callbacks sent to coordinator, dependency Item70 parent
and parent orchestrator only after this verification. Item70 can proceed under
its standing authorization. No B37 heavy work remains.

Later owner **B37-F1**, infrastructure: AC-power admission blocked qualification.
Owned by performance / issue3776. A later separately scoped qualification must
retain this failure and the frozen tested apparatus; this session does not
repair/retry the proof, change host guards, or start any successor item.

Counts: one suite family/two invocations, final443/443; native process tests0;
registered live attempts1, debugger sessions0, targets0; initial/remediation Opus0,
provider retries0, PRs0, merges0, compiler/build/Cargo/CV/counter/Sifr gates0.
Success-only diff/HIR/file-size validation and review/merge were not entered.
Only terminal documentation bookkeeping follows. The initial implementation is
not reviewed or merged; no main-delivery or live-readiness claim follows.

Blocker: B37-F1 `not-on-ac-power`. Next action: freeze terminal metadata and
send terminal callback, then STOP. B24/B27/SQL/original phase remain OPEN.
