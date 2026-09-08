# 12K-B36: sampled native custody lifecycle

Date: 2026-09-08. Owner: performance / issue3776. Scope: B35-F1 only.
State: offline implementation complete; exact-SHA review and merge pending.

## Ownership and execution boundary

Sagan owns independent fresh-main clone `/private/tmp/sifr-b36.WgYCUo/sifr`,
branch `codex/item12k-b36-custody-lifecycle`, base
`b92faf990877d2f553c70b2d29994d82ac60d4f0`. Independent index/object store,
no alternates/shared target. Sibling private `tmp` and external `evidence` (E).
Exact paths/commands registered before use in root `registration.md`.
Parent intentional planning edits and all predecessor files/indexes/refs are
read-only. Sole implementer; no parallel mutation or live process tests.

The parent TOP B36 registration and direct coordinator adjudication authorize
this complete static/offline correction through review and docs merge. The
coordinator accepted the explicitly sampled public-API contract below and
required no additional generation API or permission checkpoint. No live proof,
host reservation, compiler/build/Cargo/CV/counter/Sifr gate or next item code.

B35 native owner is closed. Authenticated terminal
`/private/tmp/sifr-b35.a3TVXg/evidence/terminal.json`, SHA256
`2a2cef162d400f9a06999681137ba32c80badfb0df6e6c6ed423ad026165b69d`, remote
record `ef8d70f4d417c7cad4803ef23b16f8ceec3c7b4b`, prelaunch
`379ab2ea8884ebb0b856b312f738e5b3ee210d0f`. All64 combined terminal/manifest
paths authenticated before independent copies. E/`predecessor` retains the
entire evidence directory unchanged; E/`input-authentication.json` binds sources.

B35's sole proof remains INCONCLUSIVE9.87207841698546s with zero events,
handovers or hits. Inferior1959 changed parent1885→1960 and stateT→TX;
debugserver1960 subsequently displayed `(debugserver)`. The full later raw row
also shows inferior `(sifr-experiment0)`. Those displays do not establish exec,
PID reuse or zombie status. Historical native identity/cause remains UNKNOWN.
All B35 processes were released. No failed proof was retried or relabeled.

## Supported source and API contract

Bounded source inspection is frozen under E/`api`, together with installed
SDK `libproc.h`, `proc_info.h`, and `event.h`:

- [XNU ptrace implementation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/mach_process.c):
  PT_ATTACHEXC selects the attach path, preserves original parent, and reparents
  the target to the tracer. Detach restores the original parent when present.
- [Apple ps formatting](https://github.com/apple-oss-distributions/adv_cmds/blob/main/ps/print.c):
  parenthesized kernel command names are a display error path for argument
  retrieval; zombie handling is separate. Parentheses never grant identity.
- [LLDB debugserver](https://github.com/llvm/llvm-project/blob/main/lldb/tools/debugserver/source/MacOSX/MachProcess.mm):
  launch/attach paths use suspended launch and PT_ATTACHEXC. This supports the
  allowed observed topology, without reconstructing B35's missing native data.
- [XNU process-event implementation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_event.c):
  EVFILT_PROC attaches a knote to the process; exec/exit flags accumulate after
  registration. Exit delivery to the outer observer may be delayed while a
  debugger is the tracing parent. NOTE_TRACK is unsupported and is not used.
- Installed public libproc APIs provide PROC_PIDTBSDINFO PID, group, start
  seconds/microseconds, parent, status and kernel command name, plus proc_pidpath
  executable path. No private PROC_PIDUNIQIDENTIFIERINFO is assumed or called.
  The public EV_RECEIPT value comes from the installed header; CPython's select
  module does not export a KQ_EV_RECEIPT name.

The contract is sampled identity evidence, not an immutable kernel generation
handle. It does not provide atomic process-table snapshots, atomic check-and-
signal, pre-registration event history, or proof of liveness from no event.
Native acquisition/registration/short-read/poll failures are explicit and sticky;
there is no ps fallback granting signal authority. No new identity research or
API question remains for this accepted static contract. Live qualification is
still unallocated, including actual permissions and timing on the installed OS.

## Executable correction and complete boundary

`coverage_native.py` owns the public libproc adapter, bounded per-run kqueue
watch, registration bracket, native evidence and consumer validation. Initial
native samples must agree around watch registration. Subsequent samples retain
that registration and cumulative event history, and compare PID/group/start
and executable path. Root and all relevant chain identities use the same check.
Observed exec/watch loss is sticky; an exited member cannot regain authority.
Known native disappearance requires ESRCH and absence from the fresh ps table;
reappearance is rejected even when recorded scalar fields match.

`coverage_custody.py` preserves first ps/native observations and separates them
from current parent/display/state. It admits only identified inferior, LLDB and
debugserver roles into ownership chains. Arbitrary helper descendants cannot
grant signal authority, and foreign group members/unknown parents reject custody.
Only the inferior's transition from its authenticated original LLDB to that
LLDB's authenticated debugserver child is accepted, with unchanged subject
identity/group and authenticated root/old/new parents. Return to the same original
parent is recorded. Accepted transitions retain observation indexes. Parent
escape, other-debugger children, missing/changed parents, identity/path changes
and replacement remain rejected. Native-backed kernel command display can vary
without rewriting the original observation, including parenthesized inferior and
debugserver names; the display alone never establishes executable identity.

Acknowledgement carries the full custody history. The embedded observer calls
`validate_ack`, which replays the actual producer against all native/ps evidence
and compares the reconstructed acknowledgement. Final completion replays the
full terminal custody document and requires the acknowledged prefix unchanged.
There is no independent permissive consumer predicate or historical upgrade.

`coverage_cleanup.py` retains B35's bounded stable-set scheduling and checks
current native identity plus all authenticated ancestors before signals.
Expected exit events disallow live acknowledgement/signals but do not by
themselves reject a completed run once fresh native/ps absence is established.
Final absence includes all known native subjects and newly observed descendants,
not only the target/group set from an earlier scheduling snapshot.
`coverage_output.py` additionally requires fresh native absence before accepting
the existing actual-exit, canonical-output, full-handover/all8-entry and two-file
completion contract. `run_coverage.py` uses the same native capture for discovery,
acknowledgement, signal authorization and final absence, then closes the watcher.
No runner or native adapter was launched during B36.

## Named offline evidence

Only one encompassing suite family was run, after complete implementation:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b36.WgYCUo/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b36.WgYCUo/evidence/coverage_symbols.py --self-test
```

Final PASS414/414: all365 inherited assertions retained, plus49 lifecycle cases.
Three same-suite invocations: initial410/414, then413/414, then414/414. All
failed receipts and exact source copies remain at E/`offline-result-attempt-1.json`,
`offline-result-attempt-2.json`, and `attempt-{1,2,3}-sources`. Corrections addressed
test expectations for unaffected groups/new authenticated roles and the complete
B35 inferior-display replay, including the synthetic kernel-name width. No
inherited acceptance assertion was dropped or converted into a later item.

Cases include actual B35 rows with native identity explicitly UNKNOWN, and
separately labeled synthetic native augmentation of that exact observed shape;
supported attach/display/return; root/subject/parent replacement and escape;
foreign groups/unknown chains; missing/short native data; watch registration and
exec races; watch loss/replay; delivered/delayed exit; PID reappearance; consumer
tampering; cleanup and real canonical-output finalization. All process/native
events and signal callbacks are injected. Python3.9 parsing is included for the
embedded apparatus. No live process, debugger, target, proof or compiler test.

E/`offline-result.json` SHA256
`c3ee76d1a16f8d847f84df2cf5e8976e13f14bff83630d94ea63b5e8d6711f89` binds
every tested source hash. E/`frozen-manifest.json` SHA256
`4eba23d064c0dc568fbb71debdd121a31ab2d2a92f72c81acd5fcafda8160dc5` binds
the complete external apparatus, API sources, raw replay inputs, failed/final
receipts, registration, binary and unchanged13-identity contract. Frozen sources
are read-only. Review must inspect those executable external files, not only
this Markdown delivery. The immutable binary remains
`a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392` and the
identity contract `3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d`.

Named remaining checks: candidate `git diff --check BASE HEAD`, HIR and file-size
guardrails, external maintained files under900. One exact-SHA initial Opus plus
at most one remediation; no broad validation or live execution by the reviewer.
Only Markdown changes enter Git, so no create-pr or merge-profile gate applies.
Publish approval outside the reviewed Git tree keyed to the candidate SHA.
After merge, a record-only PR updates the phase without another review or gate.

Remaining obligations: B35 combined live coverage unqualified; B24 causal/full
unchanged representative budget, SQL/B27/builtin-fix/approved65 delivery and the
original emitted-Rust phase remain OPEN. B34-F2/F3/F4 are not absorbed.
After this item's merged phase record and frozen terminal callback: STOP.
