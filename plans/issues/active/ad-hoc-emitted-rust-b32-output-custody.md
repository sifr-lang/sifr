# 12K-B32: canonical inferior-output custody

Date: 2026-09-08. Sole owner: performance / issue3776. Exactly B31-F1.
State: NEEDS-NEW-SCOPE / COVERAGE INCONCLUSIVE / NOT REVIEWED / NOT MERGED.
The sole proof ended; only terminal bookkeeping follows. No live repair, retry,
Opus review, PR, merge, broad gate or next-item code.

## Scope and ownership

Independent fresh-main clone `/private/tmp/sifr-b32.U1mf9E/sifr`, branch
`codex/item12k-b32-output-custody`, base
`491ba4ede1609ce476831015dc209a9064cd8ffc`. Separate index/object store, no
alternates or target. Private TMPDIR `/private/tmp/sifr-b32.U1mf9E/tmp` and
external apparatus/evidence `/private/tmp/sifr-b32.U1mf9E/evidence` (E below).
Parent dirty ledgers and all parent/predecessor files, indexes, refs and raw
evidence remain read-only. This session is the sole B32 worker. Top B32
registration in the parent phase ledger and direct onboarding authorize the
complete implementation, offline suite and conditional single proof without
another acknowledgement; coordinator reserved host capacity conditionally.

B31 native worker closed. Authenticated terminal SHA256
`6489d509766cc97bffd294398b39c5c64391293389be87cea829a2476b81757e`, scoped record
`bf8a06e854de9308448d5e9f4bb3bed29421de4a7a32f2bd12feaaa7f9a69a5f`, remote
record `c82cefa4668f5d5d4979499d4bed89d006051e82` on
`codex/item12k-b31-notifier-identity`, issue3776 comment5579452295.
Sources/raw: `/private/tmp/sifr-b31.4v8Yzw/evidence`. B31's sole run had one
handover, all eight entries, exit0, but empty observer streams. Its result
remains INCONCLUSIVE; debugger transcript bytes never replace its failed proof.

Only owned external output producer/consumer/completion apparatus and these
scoped documents change. No compiler, production source, fixture, lockfile or
workflow edits. Preserve typed identities, transitions, entry guards, immutable
diagnostic binary, exact workload/config, flags134, clocks and process custody.
No build, Cargo, Clippy, Sifr gates, CV, counter analysis, performance acceptance,
whole-phase review or next-item code. B24 causality/representative/budget, B27
joint delivery including SQL builtin fix/approved65, and full emitted-Rust
obligations remain open. Diagnostic coverage is not those acceptances.

## Installed routing semantics

Read the installed Xcode LLDB Python wrapper without importing LLDB or creating
a target: `/Applications/Xcode.app/Contents/SharedFrameworks/LLDB.framework/Versions/A/Resources/Python/lldb/__init__.py`,
SHA256 `08d7c4c689459429ae660a2a3e9f8a9b39788fcc3754e9db3316efd3b3fab676`.
At line8813 it exposes `SBLaunchInfo.AddOpenFileAction(fd, path, read, write)`
returning bool; SBTarget.Launch documents inferior standard-file redirection.
Pinned public source at Swift LLVM commit
`82cdc19fa54d566969527b56f587ea8ea30bef51` supplies the explicit action contract:

- [SBLaunchInfo.cpp](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/API/SBLaunchInfo.cpp#L297)
  delegates to AppendOpenFileAction. Owned copy SHA256
  `815028b31efcf7c4436f49ee6a00b093f67bf7684f7e1398fc25c32c92600e35`.
- [FileAction.cpp](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/Host/common/FileAction.cpp#L34)
  defines write-only opening as O_CREAT/O_WRONLY/O_TRUNC. Owned copy SHA256
  `55216af1579175ef0066c1d8efbcb6d0e46941fa71cc319b6f6aa7c1aabc95cf`.
- [ProcessLaunchInfo.cpp](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/Host/common/ProcessLaunchInfo.cpp#L199)
  supplies PTY redirection only for descriptors without explicit actions.
  Owned copy SHA256 `fdaadb5bb439388e70760d0f4afb8c6c91726450b2d111c7d7351b85ec594b70`.

Exact Apple binary/source equivalence is not asserted. Installed API plus
pinned public semantics support this concrete route; a false action return
blocks before launch. No live capability probe or exploratory target ran.
Existing human format success writes stderr, and the unchanged B20 diagnostic
dump emits complete stderr records `B20_PHASE name u64 u64 u64 u64`.

## Complete producer/consumer contract

`coverage_output.py` owns exclusive-file creation, fd action configuration,
strict parsing and final decision. Outer creates mode0600 regular sole-link
files `E/coverage/inferior.stdout` and `inferior.stderr` with O_EXCL/NOFOLLOW,
records device/inode/owner and run ID `12K-B32-U1mf9E-output-1`. Existing paths
reject a rerun. Observer verifies those empty inodes, installs fd1/fd2 write-only
actions and records the accepted mapping. No GetSTDOUT/GetSTDERR drains remain.

Sole target PID/group/flags/run identity and accepted output actions are joined
to the inherited authenticated custody handshake and process inventory.
Observer reports ENTRIES_COMPLETE only after successful inferior exit, retaining
original protocol phase/handover/hits. Outer alone reads the two files after
debugger return and zero-process/group/watcher/monitor release. Stable inode,
ownership, size and mtime are checked, with 4MiB per-stream bound.

The immutable inferior does not emit a nonce. Attribution is the exclusive
prelaunch route plus fd actions plus authenticated sole process through exit
and release, within this session's exclusive filesystem ownership. A synthetic
replay does not claim a real inferior identity. Debugger output is diagnostic
only and is absent from the consumer's API.

Expect empty stdout and one exact human success line on stderr, followed by
complete, anchored phase records with four unsigned decimal payload fields.
Payloads are retained verbatim as syntax, never analyzed as counters. Require
two ordered read/read-complete/check triplets, two distinct file.after.check
records, and one final cli.done. Reject empty, truncated, malformed, stale,
wrong-run, wrong-count, duplicate, wrong-stream and debugger-only evidence.
Then call unchanged Protocol.finish with actual exit0 and parsed count2; its
original complete handover/all-eight checks remain authoritative. PASS requires
both this final receipt and no outer failures. First failure is terminal.

## Registered commands before execution

All commands below run from `/private/tmp/sifr-b32.U1mf9E/sifr`.
Owned files: `coverage_output.py`, `coverage_output_tests.py`, `coverage_symbols.py`,
`lldb_coverage.py`, `run_coverage.py`, `coverage.lldb`, `register_inputs.py`,
unchanged `coverage_address.py`, `identity-contract.json`, `module-inventory.json`,
`sifr-experiment09`, and the three source references above. All under E.
Bookkeeping scripts authenticate/hash/serialize only; they never execute a
debugger or behavioral test.

Input authentication before the offline check:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b32.U1mf9E/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b32.U1mf9E/evidence/register_inputs.py authenticate
```

ONE named encompassing offline entry point, after complete implementation:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b32.U1mf9E/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b32.U1mf9E/evidence/coverage_symbols.py --self-test
```

Retain all182 B31 identity/handover cases plus complete output producer/consumer/
final-decision replay and required negatives using the actual shared predicates.
Includes Python3.9 parse compatibility inside that entry point. Historical
transcript-derived parser samples are explicitly syntax-only; historical B31
observer failure and whole debugger transcript remain rejected.

On offline PASS commit this registration and freeze all helper/input/record
hashes BEFORE the one proof, then callback parent/coordinator and proceed:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b32.U1mf9E/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b32.U1mf9E/evidence/register_inputs.py freeze
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b32.U1mf9E/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b32.U1mf9E/evidence/run_coverage.py
```

Fixed debugger: `/usr/bin/lldb --no-lldbinit -b -s /private/tmp/sifr-b32.U1mf9E/evidence/coverage.lldb`.
Fixed target: `/private/tmp/sifr-b32.U1mf9E/evidence/sifr-experiment09 fmt --check --no-cache /private/tmp/sifr-b32.U1mf9E/sifr/verification/areas/performance/formatter_project`.
Binary SHA256 `a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392`;
exact unchanged two-file workload/config and B29 address helper
`0df357c922d11386b5af80cfca7648d5542c8ebdec1072ba136a49be7c0bca9b`;
complete thirteen-identity contract
`3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d`.

ONE changed-apparatus setup/session/target; no retry/live repair/alternate route.
Inherited bounds:300s total including final30s cleanup,180s admission,30s setup,
60s target,256events,1GiB storage and4GiB free floor. External Python owns its
absolute watchdog; embedded Python uses local elapsed only. Full B29+B23
custody/watchdog/handshake/reaping retained. Failure preserves raw and sources,
records later3776 ownership, no review or merge. Missing capability is a blocker.

Success-only named checks (same env/TMPDIR): `git diff --check <base> <candidate>`,
`/opt/homebrew/bin/python3 scripts/check_hir_maintainability_guardrails.py`, and
`/opt/homebrew/bin/python3 scripts/check_file_size_guardrails.py`. Failure record
may run documentation diff and file-size checks. External maintained Python
files also remain below900 lines, split by responsibility.

Then one draft documentation PR and one exact-base/exact-candidate Opus review
through the two requested skills; at most one remediation review. No broad
validation repetition. A new mechanism defect on the second review is later
work and stops this item. Final review evidence lives outside the approved Git
tree, keyed by candidate SHA. Merge only with final same-SHA evidence, then
update the phase record and stop. Both broad gates are excluded for this
documentation/external-apparatus item. No next-item implementation.

## Offline evidence and prelaunch freeze

Input authentication PASS67 paths; predecessor source/raw/terminal, installed
API/dyld, unchanged workload/config/custody helpers verified. ONE registered
offline command PASS231/231, zero failures: retained182 identity/handover cases
plus49 output producer/consumer/completion cases. Python3.9 parsing passed in
the same entry point. No live debugger context or inferior has run yet.

| Artifact under E | SHA256 |
| --- | --- |
| coverage_output.py | 6923cb24a04a98ba2b22a741abd359dae97e4adfb9ec86d24740eff5eaa1c2d5 |
| coverage_output_tests.py | c765dedc34338e328a563381269d359ee470ef2273530a617cb22ff17b767e28 |
| coverage_symbols.py | 3d69f74a025a600a1b5989c82b5de676d102fbec0c8c737ac660697257ebea48 |
| lldb_coverage.py | ec610b0a20659c18abab25ad9755d2e2174a259bee93c6587954b3dc55bcf7f1 |
| run_coverage.py | 452de536f9b010c24f3b00846732c5b1de3d04bd2c065ade008febd1fdcf45c5 |
| coverage.lldb | 5a8b676883749a5a65111ade465fa2dacdbe78506faac5aa161843714fef8b41 |
| offline-result.json | e5335cdb7eb21448c9ae5509896e45e7c32cad50a4caed754b2a4a4d48befe48 |
| input-authentication.json | 53bc5ae6a2d202f382ddcaed23b91a436532ec1b2cdc943e9869ad57f1210241 |

Commit/export this registration then write immutable `prelaunch.json`, binding
the exact commit, all helper/input/registration hashes and run identity. Parent
and coordinator receive the freeze identities and start notice before the sole
proof, with no additional acknowledgement required for this unchanged contract.

## Sole-proof terminal result

The exact registered command ran once, exit1 after8.142497042s. One live
debugger/session/target,5 observed stops and4 explicit continues; no retry.
Observer local elapsed4.806614667s. Mode3 stop6 retained the authenticated
positive9.1 notifier pair and same thread128043321 with invalid frame PC and
empty modules; the preserved B31 transition guard accepted it and cleared the
old epoch. At stop7, mode0/current map was delivered at PC6558044876, but
`lldb_coverage.py:98` rejected `('ambiguous current module', 'notifier')`.
Parent assigned later **12K-B33 exactly B32-F1** and **12K-B34 exactly B32-F2**,
without duplicate owners. This registration grants no next-item work here.

**Later finding12K-B32-F1**, sole owner performance / issue3776: the inherited
module resolver's handover identity model cannot adjudicate the two mapped
`/usr/lib/dyld` rows in preserved stop7. Both have UUID
`74E52480-C2BD-3C8D-812D-95FE2B74A096`, triplearm64e-apple-macosx26.6.0,
header_file6443581440 and header_load6557810688. These equal scalar fields do
not establish SBModule object identity or authorize choosing/deduplicating a
row. Preserve full sections/raw; a later owner must establish installed module
instance semantics from these receipts before any scoped repair/proof. Do not
drop ambiguity guards or infer a current binding from a prior absolute address.
No rearming completed, handover_count0 and required_hits[] in this run.

**Later finding12K-B32-F2**, same sole owner3776: the outer custody watcher
independently recorded `owned process identity changed`. Frozen `custody.json`
contains the first-seen identities but not the conflicting current row that
triggered that predicate; `stop.json` has the rejection string and final elapsed
only. The changed field/PID and ordering relative to observer failure cannot be
recovered conclusively from those records. No claim of PID reuse, exec rename,
zombie transition or debugger failure is made. Later custody adjudication owns
this evidence gap; no weakened authentication or unobserved fix is applied.

Both installed fd1/fd2 actions returned true and were recorded for run
`12K-B32-U1mf9E-output-1`, inferior7391. Both owned files remain0 bytes because
the proof stopped before workload completion. The final file consumer was never
called, and there is no output-completion receipt, exit0, two-file completion,
live output-routing success, full coverage or performance acceptance claim.
Offline231/231 remains valid for the frozen implementation, but does not turn
this failed live run into a pass. The apparatus and raw outputs remain frozen.

Observer Kill returned success. Final release: remaining[], groups_absent=true,
watcher_alive=false, monitor_alive=false. Groups7308/7391 absent; independent
`ps -p 6774,7308,7391,7392 -o pid=,ppid=,pgid=,etime=,command=` returned no rows
for launcher/debugger/inferior/debugserver. Capacity released and parent and
coordinator notified of the terminal outcome. No evidence was removed.

Prelaunch registration commit `e71fbf1ab0ec12cdaa752fb9c9d3bc7c44aaa39a`, exported
registration SHA256 `22ab0f158e88a0440ed1955fadff9b71495c9b314eadb4c16aaf0a325f3ffd68`;
manifest SHA256 `17d9fb3da2b5728353306b6d949b79d06afc5c39a3dce42f8d1482dd5f8ff7b8`.
Parent/coordinator received these identities and actual-start notice before
the sole command, under the unchanged approved contract.

| Raw artifact under E/coverage | SHA256 |
| --- | --- |
| outcome.json | e411bf32a305246792dd1d7966c4a5a833b9075f1350356c9defccab72a335d5 |
| events.json | 503a922565c558967651362e8a9f5cc28742707ba5b2d6004ca9860d53254c61 |
| observer-result.json | 7ea23135b7272134867c5fdca967bd7d355ee0a40c4d8f2a6b5e4bbd927cf8ed |
| process-release.json | 4f041f1ee34d966b05a3a159841418bf07dbb0c54fceae06e332fa0b684a2735 |
| debugger-result.json | a4f644ffd4022568ceea7809040a0d6183d98a2c0276b382676843e621d1ae5d |
| output-route.json | 8f33e567ee1a5174d5ad04d0c6545c9c185ba729746b0c4eba336255a9dfc4e2 |
| custody.json | 890321de29542a85a2790ded26f037b1f2284e66330bcd54bbc957a5d4d298aa |
| stop.json | 7aafb6b29bc327ed92d8b25be057f7ad54fc0331ea98d2761126c23373e96792 |

Counts: one offline command231/231; one live command/session/target; zero retries,
initial/remediation/provider reviews, PRs, merges, create-pr/merge gates,
compiler builds/tests, CV/counter/acceptance acquisitions and postfailure
apparatus edits/probes. Success-only HIR/review/merge branch was not reached.
Only terminal documentation diff and file-size guardrail are run; external
terminal.json binds the final record SHA, check evidence, all artifact hashes,
owner issue comment and independently verified process release.

Next action: STOP after immutable terminal/record/issue3776 bookkeeping. Later
F1/B33 and F2/B34 are registered for separate adjudication, with no next-item implementation
or new proof authority. B24, B27 and the full emitted-Rust obligations remain.

Terminal file-size guardrail PASS3762 files,900-line limit. No HIR or broad gate
ran on the failure branch. The exact final committed documentation diff and
external maintained-source line counts are bound to the terminal record SHA.
