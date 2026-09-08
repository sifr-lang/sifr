# 12K-B31: authenticated notifier dispatch and protocol identity

Date: 2026-09-08. Owner: performance / issue3776; exactly B30-F1.
State: NEEDS-NEW-SCOPE / COVERAGE INCONCLUSIVE / NOT REVIEWED / NOT MERGED.
The sole live proof ended. Only terminal bookkeeping follows; no retry,
apparatus repair, review, gate, PR, merge or next-item code is authorized.

## Ownership and authorization

Independent fresh-main clone `/private/tmp/sifr-b31.4v8Yzw/sifr`, branch
`codex/item12k-b31-notifier-identity`, base
`491ba4ede1609ce476831015dc209a9064cd8ffc`. Private TMPDIR
`/private/tmp/sifr-b31.4v8Yzw/tmp`, external evidence
`/private/tmp/sifr-b31.4v8Yzw/evidence`. Separate Git index and object storage.
Parent dirty ledgers, index, all other parent files and predecessor roots remain
read-only. Sole worker; coordinator capacity reserved for one proof. The top
parent B31 registration and direct onboarding authorize these steps without
another acknowledgement. No production/compiler/fixture/lock/workflow changes.

B30 terminal SHA256 `383bacfeabdf12630a426cfa70a93020923793b720f582cac366008d72b462f2`
and full514-line scoped document SHA256
`f360c07625be277cf52a6f9ff56d23e1651dcb075f9edb0a8de9ecde903875f0`
authenticated and read completely. Terminal record
`dd6aa52db1ae775a7ec7976271574dee038d1ed0`, predecessor owner closed.
All source/raw inputs remain at `/private/tmp/sifr-b30.PiHI2k/evidence`.
The old one-proof allowance is exhausted; B31 has its own approved changed
apparatus allowance. B24 cause/representative/budget, B27 joint delivery and
the complete emitted-Rust phase remain open regardless of this diagnostic.

## Registered paths and commands before behavioral execution

Owned copies of `coverage_symbols.py`, `coverage_address.py`, `lldb_coverage.py`,
`run_coverage.py`, `coverage.lldb`, `identity-contract.json`,
`module-inventory.json` and `sifr-experiment09` are under the external evidence
root. Only the symbols helper, observer and path/manifest plumbing change.
The B29 address helper and B20 experiment09 binary remain byte-identical.
Pinned API reference copies `SBThread.cpp` and `StopInfo.cpp` and the installed
Python wrapper are source evidence, read without creating a debugger context.
Bookkeeping scripts may authenticate inputs and serialize receipts only.

Exact bookkeeping commands (not behavioral tests or debugger operations), with
the same cwd and environment as the named checks below:
`env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b31.4v8Yzw/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b31.4v8Yzw/evidence/register_inputs.py authenticate`
before the offline check; the same command with `freeze` after offline PASS and
registration commit. These exclusively create authentication/manifest receipts.

After completing all apparatus changes, run exactly once from the owned clone:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b31.4v8Yzw/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b31.4v8Yzw/evidence/coverage_symbols.py --self-test
```

All existing99 checks remain, plus authenticated B30 stop4/stop6 replay;
missing/wrong/internal-only/malformed pairs, wrong thread/epoch/mode/count/phase,
competing owned entry, contradictory valid PC, repeated handover and shared
dispatcher/protocol consistency negatives. These use the observer predicates.

On PASS freeze helper/input/registration hashes in external `prelaunch.json`,
preserve the registration bytes, notify parent and coordinator, then execute
exactly once without waiting for another acknowledgement:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b31.4v8Yzw/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b31.4v8Yzw/evidence/run_coverage.py
```

Fixed debugger argv: `/usr/bin/lldb --no-lldbinit -b -s
/private/tmp/sifr-b31.4v8Yzw/evidence/coverage.lldb`.
Sole target argv: `/private/tmp/sifr-b31.4v8Yzw/evidence/sifr-experiment09
fmt --check --no-cache
/private/tmp/sifr-b31.4v8Yzw/sifr/verification/areas/performance/formatter_project`.
Unchanged two-file workload/config; flags134. One setup/session/target only.
300s total including final30s cleanup, up to180s admission/30s setup/60s target,
256events, 1GiB evidence cap/4GiB free floor. Separate external/embedded clock
domains, full inherited B29/B23 authenticated custody, handshake, watcher,
reaping and zero-process release. No retry, alternate mechanism, counters,
CV, compiler build/test/Cargo/Clippy or Sifr gate.

First live failure is terminal INCONCLUSIVE: preserve all sources/raw evidence,
record exact later ownership under3776, no repair/retry/Opus/merge. Success only:
`git diff --check <base> <candidate>`,
`env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b31.4v8Yzw/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 scripts/check_hir_maintainability_guardrails.py`,
and same environment `scripts/check_file_size_guardrails.py`. Failure bookkeeping
may use exact documentation diff and file-size guardrail as in B30.
One exact-SHA Opus review plus at most one remediation via the named skills;
new mechanism defect on second review becomes later work and stops this item.
Docs-only PR/main merge then phase record and STOP. No next-item implementation.

## Acceptance contract

Both notifier dispatch and Protocol use one pure identity predicate. Its
authority is the prior authenticated owned notifier site in the active epoch,
exact positive breakpoint/location pair, one stopped thread with the prior
notifier thread identity, and well-formed raw pairs with no competing owned
entry. Internal-1.1 alone is never authority. The special empty-map/invalid-frame-
PC representation requires first mode3/count1 before any required hits. Valid
frame PC must equal the saved notifier address; mapped events retain exact
current binding checks. No address is inferred from registers or old processes.
Invalidate all old bindings on mode3; require the original mapped mode0/current
UUID/changed header/rearming before any entry. Normal entry checks stay intact.
Capture actual PC-register validity/value/error only as diagnostics.

## Installed API/source support and implemented mechanism

Installed Xcode LLDB Python wrapper at
`/Applications/Xcode.app/Contents/SharedFrameworks/LLDB.framework/Versions/A/Resources/Python/lldb/__init__.py:14642`
documents uint64 alternating breakpoint/location IDs. Its SHA256 is
`08d7c4c689459429ae660a2a3e9f8a9b39788fcc3754e9db3316efd3b3fab676`.
Pinned Swift LLVM commit `82cdc19fa54d566969527b56f587ea8ea30bef51`
[SBThread.cpp](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/API/SBThread.cpp#L153)
delegates these reads to stopped-thread StopInfo.
[StopInfo.cpp](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/Target/StopInfo.cpp#L253)
returns twice the constituent count and alternates each constituent's
breakpoint ID and location ID. These references explain the public contract;
exact Apple binary/source equivalence is not asserted. Preserved B30 raw
stop4 and stop6 independently show the same positive owned9.1 plus uint64
internal-1.1 encoding. No new debugger context or inferior was needed.

External pinned source SHA256s: SBThread.cpp
`2e3dfec994348ab8b31003061b357c2c5dea361002f29b30e22c01e3119bdfb1`;
StopInfo.cpp `7ee9a24523f99fd04b8af49c80badc3dcced0d82b8e8f330003764ca4b4ccf3e`.
Authenticated B30 raw events SHA256
`a0c2ed89affe3c9d0e10498a4fa8d58f446f3d606dce1c74ebddf3c032fa8d3f`.
Remote predecessor ref independently verified at the terminal record SHA.

`stop_evidence` parses live and replayed raw stops identically. Strict
`breakpoint_pairs` rejects malformed/odd/duplicate/invalid words.
`Protocol.notifier_identity` is the common predicate called by `is_notifier`
dispatch and `notifier` state mutation. A prior mapped owned notifier stop
binds thread, epoch, earlier stop ID and the complete verified site. Mode3
must match that receipt, exact pair and transition constraints. A valid frame
PC must still match the saved binding; unavailable frame PC is admitted only
for the explicit mode3 with a genuinely empty module list. Mapped dispatch
revalidates actual sites before the protocol changes state. Register-PC
diagnostics never enter `stop_evidence` or acceptance predicates.

Mode3 clears protocol bindings and the prior receipt, increments epoch and
deletes old owned dyld breakpoints in the observer. The inherited next-stop
mapped mode0/current exact UUID/new header/rearming contract and eight-family
entry ordering remain. Launcher code changes only its owned root/item label;
custody, limits, clock domains and admission are inherited unchanged.

## Offline PASS and frozen prelaunch registration

Input authentication PASS47 paths; source/manifest/B30 receipts, installed API,
dyld, unchanged two-file workload/config and inherited custody helpers match.
The sole named offline command PASSed182/182, zero failures: retained99 cases
plus83 raw-replay/shared-dispatch/protocol/identity checks. No other behavioral
test or debugger context ran. Python3.9 parsing of all four apparatus modules
passed inside that same command. Synthetic maps remain labeled separately from
B29 coordinates and B30 historical raw replay; offline PASS is not live proof.

Final tested sources, unchanged from authentication through launch:

| Artifact under external evidence | SHA256 |
| --- | --- |
| coverage_symbols.py | 5cba1276c3b2a1fd62b4b2b4886a168a028fbaf50f6c86684fa26bfdcaaa46e2 |
| coverage_address.py | 0df357c922d11386b5af80cfca7648d5542c8ebdec1072ba136a49be7c0bca9b |
| lldb_coverage.py | 5188595d5590f19dff8de86fcab21831d42da230fdc9663c74e1be1bb1da7033 |
| run_coverage.py | 7f9e071d6850d366eeade1d880081af957d752e3dedb236e6c0380d6ea0b4020 |
| coverage.lldb | 845877cb5525b853c7a0d6e257c92075411b8b1d98bb201c679fa86731914a2c |
| offline-result.json | 0dfd9c7bd6b642d6e8d58ad256ade9df21cf87bf44afbcdacecfbb4a6ad19204 |
| input-authentication.json | 25bcff2ad2e49bc130b12ca27d1884edcac47fcdb540e4f3d592a858970ad8c5 |
| identity-contract.json | 3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d |
| sifr-experiment09 | a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392 |

This registration is committed and exported outside Git before launch.
`prelaunch.json` binds its exact commit/bytes, these helpers, offline result,
all authenticated inputs and ancestor-config presence. Parent/coordinator
receive manifest/registration/offline identities and actual start notice before
the sole command. No further acknowledgement is required. All original bounds,
first-error terminal behavior and success-only review/merge branch apply.

## Sole-proof terminal result

The exact registered command ran once, exit1 after8.188232208s. One live
debugger session, one target,15 stops and15 continues; no retry. Observer
elapsed5.00455325s in its separate embedded clock domain. The repaired
dispatch/protocol accepted mode3 stop6 with empty modules, invalid frame PC,
owned9.1 plus internal-1.1, thread127985349 and prior authenticated notifier
binding. The original mapped mode0 transition at stop7 completed once with
the registered dyld UUID `74E52480-C2BD-3C8D-812D-95FE2B74A096`, changed header
and newly enabled/resolved notifier13.1. No old absolute address was reused.

Historical values for this process only: initial dyld header4512825344 and
notifier4513059532; new header6557810688 and notifier6558044876. Stop6 raw
PC register was valid, value4513059532, value error success and read success;
Frame.GetPC was18446744073709551615. Register evidence was diagnostic only,
never used in classification. It does not create an alternate acceptance route.

All eight required families hit exactly once in order: prepare stop11, JIT
fixups13, generic15, libSystem19, sanitizers21, constructor23, main25 and
completion27. The target exited0. This proves observed handover and entry
delivery in the one run, but does not meet the full registered proof predicate.

Final `lldb_coverage.py:360` called `Protocol.finish`, which rejected at
`coverage_symbols.py:248`: `missing successful two-file completion`.
Observer stdout/stderr were both empty, so its marker count was0. Preserved
outer `debugger-result.json` stdout contains `format check passed` and exactly
two `B20_PHASE file.after.check` records. This is evidence of a diagnostic
output ownership/reader-boundary defect; absent workload completion is not
established. The outer transcript is not substituted post hoc into the failed
observer criterion. Result remains INCONCLUSIVE, no qualification or merge.
Incidental phase payloads in the raw transcript are retained as bytes only;
no counter analysis, CV sampling or performance acceptance was performed.

**Later finding12K-B31-F1**, sole owner performance / issue3776: establish a
single authenticated inferior-output custody path for the registered two-file
completion criterion. The current observer's SBProcess stream reads were empty
while the debugger's captured output contained both markers. A later item must
first ground its complete output ownership contract in these preserved raw
receipts and relevant installed API/source. No fix, alternate read route,
weakened completion criterion, probe or new proof is implemented here. This
finding is a later owner registration, not authority to start its code.
Parent assigned later **12K-B32** to exactly B31-F1, with no duplicate owner.
Parent directed only immutable terminal/record/issue3776 bookkeeping and STOP;
there is no need to await the coordinator's read-only later-contract work.

Process release: remaining[], groups_absent=true, watcher_alive=false and
monitor_alive=false. Inferior84722 exited0; debugger84681, debugserver84723
and launcher84114 are absent in independent
`ps -p 84114,84681,84722,84723 -o pid=,ppid=,pgid=,etime=,command=` output.
Capacity released. No owned or predecessor evidence was removed or rewritten.
All apparatus source hashes still match the tested prelaunch manifest.

Prelaunch registration commit `efe99d3daaf0ad94b2bd090c2192db9ef5036b8e`;
exported registration SHA256
`2e79aafeb6359da4bb2b936b7121e72e0ebf5f35bc1d036cff73129d114180da`;
manifest SHA256 `5a29e12f165e345ad6224cf611abc2885d5bcb2b5c71dfa931327d196bb79b4a`.
Both parent and coordinator received exact hashes and actual-start callback
before execution, then received terminal outcome/release notification.

| Raw artifact under evidence/coverage | SHA256 |
| --- | --- |
| outcome.json | 9be1b4cd726d1d6005c44db9e89492640406f3ea517f9627af0471eb4561c308 |
| events.json | 3293ca9c76e5877482cd5a5fbb9cd29284197753c0244c5768c389ad4811320b |
| observer-result.json | 1db95cc2feaf3478173c2f4546321c92e152f4df06972221abc3d4843a6854b2 |
| debugger-result.json | dd7663a0005c1942cefc1d4358dd9e03e5c90b9a71c9dc3752939fc45aa53b2f |
| process-release.json | 66b71c7778d70444f92e1a89d68e5ca1aec9f65e043c697164e989a49649eb5d |
| site-arming.json | 011d5eb6b097ab2979e0acc2b46d01546c337b6fd24a5434bc7167f85373944d |
| identity-resolutions.json | 08ed3a7f668a2f68998919d775c9ee8f7d3d37b19202888af37dc6af35c82d6b |
| inventory.json | 3615402b0e6ad2af7fc5a40e2cb3fc7172c0d7a8b2964345cf82e285b988d7d5 |

Counts: one offline command182/182; one live command/session/target; zero
retries, Opus/provider requests, remediation reviews, PRs, merges, create-pr
or merge gates, compiler builds/tests, counter/CV/acceptance acquisitions and
postfailure apparatus edits/probes. Success-only HIR/review/merge branch was
not reached. Exact terminal record diff/file-size checks and remote record SHA,
owner issue comment, final hashes and independent release are authenticated
in external `evidence/terminal.json`. Next action: STOP after that terminal
callback. B24, B27 and all full-phase obligations remain open.

Terminal file-size guardrail PASS3762 files with900-line limit. Owned maintained
sources: symbols577, address140, observer378, launcher280, registration101 lines;
each is below the limit. No HIR or broad gate ran on this failure branch.
The exact final committed documentation diff is checked once in terminal
bookkeeping, with its command/result bound to the terminal record SHA.
