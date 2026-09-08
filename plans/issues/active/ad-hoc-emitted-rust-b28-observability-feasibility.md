# 12K-B28: early-loader observability feasibility

Date: 2026-09-08. Owner: performance / issue3776. This is B24-F1's sole
registered owner, not a second owner. State: ASSESSMENT COMPLETE / PROPOSAL
AWAITING ORCHESTRATOR ADJUDICATION / NOT EXECUTED / NOT REVIEWED / NOT MERGED.

## Finding and limits

A supported, materially different interception proposal exists: request LLDB's
initial launch stop, expose dynamic-loader image-change stops, and verify the
current mapped dyld and its actual breakpoint sites before allowing execution
through `prepare`. Installed dyld disassembly corroborates a notification before
the transition to cached dyld and before `prepare`. Merely adding stop-at-entry
without observing the mapping transition is insufficient evidence of coverage.

This is a proposal, not proof that Apple's installed LLDB delivers every required
stop. B24's missing hits do not identify a loader/runtime/formatter defect. There
is no evidence-supported general OS limitation. One target is proposed solely
to establish interception capability; no instruction-counter acquisition, CV,
causality, performance correction or acceptance follows from it.

## Ownership and exact inputs

Owned root: `/private/tmp/sifr-b28.prJZsY`; independent clone `sifr`, branch
`codex/item12k-b28-observability-feasibility`, private `tmp`, external `evidence`.
Fresh main/base `491ba4ede1609ce476831015dc209a9064cd8ffc`. No shared Git index,
target, alternate object store or predecessor mutation. CARGO_TARGET_DIR remains
unset in proposed commands. No target directory or binary copy was created.

Read the parent dirty phase's top B28 dispatch and next-heading registration,
the full B24 experiment registration and B24's scoped phase record at preserved
record `513f6ff34220a43c308c6c3b1255470ef344fea1`. Do not import the parent's
uncommitted historical ledger or any retained compiler stack into this clone.

Authenticated B24 evidence, under `/private/tmp/sifr-b24.4MtvfE/evidence`:

| Input | SHA256 |
| --- | --- |
| `terminal.json` | `cc11c78958c8abcf0d290595048977af80346ec066f2fdcc8acc20905e4afb21` |
| `boundaries/target-0.json` | `220f537acd7c2a0d3ec9d042e78126c1d54bd9d4ac930284784d7a49b6c232d1` |
| `boundaries/symbols-prelaunch.json` | `63dfaec72efe4161d9cff961c430998176d50e11f2722c742daf6f4d6b137c50` |

All eight prelaunch symbols had one location. Target0 flags were130 (debug2 plus
separate-group128, without stop-at-entry4). Its first recorded event was
`libSystem_initializer` at `0x19704b25c`. Its later return PC `0x186e52a28`
resolved inside dyld. `target-0.json` has no modules, UUID/slide inventory,
earliest-entry record, or per-location loaded/site state. These cannot be
reconstructed as historical observations. B24's single target exited0 in8.77s;
the other five were never launched and confer no new allowance.

B20 raw `/private/tmp/sifr-b20.bj6Au6/evidence/experiment14/profile.xml`, binary
element92, records dyld arm64e UUID `74E52480-C2BD-3C8D-812D-95FE2B74A096`,
load address `0x186e04000`, path `/usr/lib/dyld`. This is a historical timer
profile's mapped address, not the address of a future process or an entry PC.

Read-only command identities on this host:

- `/usr/bin/lldb --version`: `lldb-2100.0.17.203`, Apple Swift6.3.3;
  `/usr/bin/xcrun --find lldb`: `/Applications/Xcode.app/Contents/Developer/usr/bin/lldb`.
- External `/opt/homebrew/bin/python3`:3.14.7; `/usr/bin/python3`:3.9.6.
- `sw_vers`: macOS26.6.2 /25G83. `what /usr/lib/dyld`: dyld-1387.
- `dwarfdump --uuid /usr/lib/dyld`: arm64e UUID matches B20's element92.
  Fat-file SHA256 `373fce2b3689c2113d5341a6bba735e9f970401ba64896c0a6379680192f1cdf`.
- Installed Python API source:
  `/Applications/Xcode.app/Contents/SharedFrameworks/LLDB.framework/Versions/A/Resources/Python/lldb/__init__.py`,
  SHA256 `08d7c4c689459429ae660a2a3e9f8a9b39788fcc3754e9db3316efd3b3fab676`.
  It exposes StopAtEntry, launch flags, load-address/module/UUID APIs,
  `SBBreakpointLocation.IsResolved`, `IsEnabled`, and `GetLoadAddress`.

## Primary-source chain and installed corroboration

LLDB reference is Swift6.3.3 release commit
`82cdc19fa54d566969527b56f587ea8ea30bef51`. This matches the reported Swift
release and available API surface; it is NOT an authenticated source build of
Apple's exact LLDB2100 binary. Dyld reference is Apple's latest published tag
dyld-1378, commit `fd8d0c4d52320ebf64db34f3cb280310d905c5ae`, NOT dyld-1387.
Relevant installed machine code was inspected to check the proposed call order.
No claim of exact source/binary equivalence or guaranteed live behavior is made.

1. [Launch flags](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/include/lldb/lldb-enumerations.h)
   define debug2, stop-at-entry4 and separate-group128. [Debugserver spawn](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/tools/debugserver/source/MacOSX/MachProcess.mm#L3604)
   uses `POSIX_SPAWN_START_SUSPENDED`. [Process launch](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/Target/Process.cpp#L2999)
   catches the first stop before calling dynamic-loader `DidLaunch`.
   [Target launch](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/Target/Target.cpp#L3589)
   returns/broadcasts that stop with StopAtEntry; without it, the stopped target
   resumes. Thus B24 flags130 did not request delivery of that first stop.

2. The earliest supported user-visible stop is the suspended launch/exec stop,
   before dyld begins user-space startup, not Sifr main. The exact live PC must
   be verified. Installed arm64e dyld exports `__dyld_start` at file VA0x49c0;
   its instructions set up registers and branch to `start` at0x1e994. For the
   corresponding mapped image, expected entry is its verified section load
   address plus section-relative offset (not a reused historical ASLR address).
   An unexpected initial PC/stop reason fails the proposed proof.

3. [Darwin DidLaunch](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/Plugins/DynamicLoader/MacOSX-DYLD/DynamicLoaderDarwin.cpp#L76)
   fetches loaded images and installs the loader notifier. [MacOS image fetch](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/Plugins/DynamicLoader/MacOSX-DYLD/DynamicLoaderMacOS.cpp#L196)
   replaces preloaded image data. Its notifier's mode3 clears images/section
   loads, sets a handover address breakpoint, then mode0 refetches modules and
   reinstalls notification. Its callback returns `GetStopWhenImagesChange()`;
   the [target property](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/Target/TargetProperties.td#L320)
   `target.process.stop-on-sharedlibrary-events` defaults false.

4. [Dyld's transition](https://github.com/apple-oss-distributions/dyld/blob/fd8d0c4d52320ebf64db34f3cb280310d905c5ae/dyld/ExternallyViewableState.cpp#L1260)
   notifies mode3 then, on its successful all-image-info transition, calls the
   new cached notifier with mode0 before returning to the cache restart.
   [Startup](https://github.com/apple-oss-distributions/dyld/blob/fd8d0c4d52320ebf64db34f3cb280310d905c5ae/dyld/dyldMain.cpp#L1244)
   performs that transition before `restartWithDyldInCache`, and cached startup
   [later calls prepare](https://github.com/apple-oss-distributions/dyld/blob/fd8d0c4d52320ebf64db34f3cb280310d905c5ae/dyld/dyldMain.cpp#L1450).
   The source's noted transient image-list gap is not evidence that B24 took a
   particular branch or that this installed LLDB has a diagnosed defect.

5. Installed dyld-1387 arm64e disassembly confirms the relevant sequence:
   `start` calls `prepareInCacheDyldAllImageInfos` at file VA0x20144 and
   `restartWithDyldInCache` at0x20184. The former's successful branch issues
   notification mode3 at0x3e96c then mode0 at0x3e988; the failure branch issues
   only mode3 at0x3e948. The later start-lambda calls `prepare` at0x20b2c;
   `prepare` itself is0x20b4c. These are static file VAs, not live trap addresses.
   Full selected disassembly is external `evidence/installed-dyld-static-inspection.txt`,
   SHA256 `c8722712d1551149f7768aecb01ff90047386c0997a152f06baf6276bece02f2`.

6. [Breakpoint site resolution](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/Breakpoint/BreakpointLocation.cpp#L485)
   distinguishes a symbol location from a process breakpoint site. For ordinary
   non-scripted address breakpoints, `IsResolved()` reports a site, while creating
   that site requires a process. Record enabled/resolved/address/UUID before
   resume and actual breakpoint hits afterward. `ReadMemory` can hide software
   breakpoint bytes; a readback of original instruction bytes alone does not
   prove that a trap was never installed. A future hit provides functional proof.

The source files above are preserved outside Git in `evidence`, with a separate
SHA256 inventory. Static `nm`, `llvm-objdump`, `what`, `dwarfdump`, XML extraction
and source reads neither create an LLDB target nor start a debugger session.
One objdump option spelling was rejected, then corrected per its help; one
unneeded source URL returned404. Neither was a launch or a behavioral test.

## Concrete one-target coverage-proof proposal — not authorized by this file

Only the orchestrator's next concrete registration may authorize implementation
and the coordinator's window may authorize launch. Proposed implementation is
external diagnostics only, precisely these three new paths:

- `/private/tmp/sifr-b28.prJZsY/evidence/run_coverage.py`: launcher, independent
  own-clock watchdog, target-custody handshake, bounded raw output and cleanup.
- `/private/tmp/sifr-b28.prJZsY/evidence/lldb_coverage.py`: first-stop/image-change
  observer, current-image/site records and entry hits; no counters or expressions.
- `/private/tmp/sifr-b28.prJZsY/evidence/coverage.lldb`: fixed batch import/run/quit.

No scripts are authored yet. After adjudication only, copy the immutable B24
diagnostic binary into `evidence/sifr-experiment09`, require SHA256
`a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392`, and use the
unchanged two-file formatter workload in this fresh-main clone. Record all
ancestor config and input hashes, exact commands and final script/registration
hashes BEFORE launch. Do not backdate B24's postrun amended-registration hash.

Proposed sole outer command:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b28.prJZsY/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b28.prJZsY/evidence/run_coverage.py
```

It would run exactly `/usr/bin/lldb --no-lldbinit -b -s
/private/tmp/sifr-b28.prJZsY/evidence/coverage.lldb`; cwd is the owned clone.
Exactly one target: owned `sifr-experiment09 fmt --check --no-cache
/private/tmp/sifr-b28.prJZsY/sifr/verification/areas/performance/formatter_project`.
ASLR stays enabled; no shell, environment injection, linker/security/priority
change, extra control, warmup, counter sampling, replay or second target.

Fixed observation protocol:

1. Set `target.disable-aslr false`, `target.skip-prologue false`, and
   `target.process.stop-on-sharedlibrary-events true`; check command errors.
   Launch with exactly debug2 | StopAtEntry4 | separate-group128 =134. Preserve
   full command identity, stream errors, stop IDs/reasons and every continue.
2. At the first stopped event, record PID/PGID and handshake with the outer
   custodian BEFORE any continue. Validate exec/suspended stop at dyld entry,
   current PC mapping, module UUID/path, header/section file+load addresses and
   executable memory-region bounds. No inferior expression/function calls.
3. Resolve `prepare`, JIT `applyFixups` and `applyFixupsGeneric` in that active
   dyld only; record exact symbol names/start addresses, create ordinary address
   breakpoints and require valid, enabled, resolved sites at the mapped entry.
   Missing active image/sites at this point fails closed, without ad hoc mapping.
4. Continue only to observed entry or shared-library stops. At mode3/image removal,
   retain that transition even if the module list is transiently empty; discard
   stale owned sites. Permit the single registered handover continuation to the
   loader's mode0 re-fetch stop. Before further execution, require the new dyld
   UUID/header/load mappings and all three newly resolved sites. No stale disk
   VA or historical cache address may be transplanted. If another stop cannot
   prove this pre-prepare transition, fail immediately; no custom loader patch,
   manual module load, instruction-stepping workaround or additional attempt.
5. Require an actual `prepare` entry hit in the active mapping before any fixup
   hit, then actual JIT and generic entry hits in the same process, preserving
   function/thread/stop/site identity. Register the five B24 initializer/app
   entry boundaries and preserve all hits, stdout/stderr and the original
   buffered phases. Require both file checks and normal exit0. This proves entry
   coverage only: no return-interval or instruction attribution is claimed.

Every module transition is recorded and sites revalidated before resume, including
whether the PC is temporarily in a cached notifier while its caller is still in
launch dyld. Do not call the initial bootstrap image the eventual active dyld
without observing the transition. Numeric future load addresses remain unknown
until this one process exists; the current record cannot supply them honestly.

Caps: one setup attempt, one LLDB session, one target, zero retries. Whole window
300s from admission start, up to180s admission, at most30s debugger setup and60s
target lifetime, with final30s reserved for cleanup inside that total. Refuse a
launch unless60s fits before the cleanup boundary. At most256 observer events,
1GiB total owned evidence including binary, and at least4GiB free at admission.
No inherited five-target allowance. Admission rejection consumes this attempt.

Clock/custody: outer Python uses only its own monotonic clock for the absolute
whole-window deadline and watchdog. Embedded LLDB measures elapsed time from
its OWN start; it receives durations and a stop request, never another process's
absolute monotonic timestamp. Target budget begins before Launch in the observer;
outer watchdog also remains live if Launch/API calls stall. Stop/reap before
deadline, not after adding a new cleanup budget. Record interpreter identities
and actual elapsed durations without claiming unexercised timeout branches pass.

Use merged B23's owned-process helper for the debugger group, with bounded
remaining time; it does not by itself own a separately grouped inferior. The
outer custodian also records the unique diagnostic-binary child and its PID,
PPID/PGID and creation identity while launch is pending, retains observer PID
handshake, and refuses continue until they agree. On any failure, the observer
kills its process and the custodian verifies/kills only authenticated owned
groups, reaps its children and checks target/debugserver/debugger disappearance.
Unknown custody or surviving descendants is a terminal blocker, not an excuse
to launch again. Preserve raw partial output and a zero-process release receipt.

Success is ONLY earliest-stop and active-dyld trap/entry coverage plus clean exit
and release. Any missing/ambiguous site, missed transition, early later-phase hit,
unobserved required entry, unexpected stop, host rejection, timeout, storage
overflow or unproved cleanup is INCONCLUSIVE with precise stage/receipt evidence.
No alternate mechanism is implemented during failure handling.

## Adjudication and delivery boundary

First-stage named checks are source/API/receipt/command identity and documentation
diff only. No debugger session, target, compiler/build/test, acquisition, Opus,
Sifr gate or PR was run/opened. Final review allowance has NOT been spent or
invented. The next adjudication must explicitly set this proposal's external
implementation/check allowance and whether any later documentation review/merge
is authorized. A proof failure cannot automatically trigger another design/run.

B24 cause and full unchanged formatter/representative/budget acceptance remain
unmet. B20 initial+B22 remediation remain exhausted. B23's historical pass stays
attributable to B23. B27 integration, original3717/corpus48, retained full-phase
goals and every next item's code remain outside this assessment.

Next action: send exact registration commit/path/hash and primary-source findings
to parent01a06e86-414a-7e11-9256-1f45bdb5a6c7 and coordinator
01a07d84-7c62-77f0-b7c7-ecf310829a11. Hold with zero owned execution processes
for concrete orchestrator adjudication; no user permission loop and no automatic
review, merge, target, benchmark or subsequent mechanism.

## Approved execution registration (before launch)

Parent authenticated original proposal hash `be2c46b17398e2b51737678fde5e49681cc9c1ec79de8a90706c9a2d60b83e23`
and handoff `8651499d6703122419aaa49784e37c38ab759fcd648ca3dd913be3505511f92f`,
then authorized the same worker to implement exactly the three external scripts
and owned immutable diagnostic copy and execute the sole coverage proof above.
Coordinator cleared/reserved the exact window; parent subsequently confirmed
both decisions complete. No further permission checkpoint is pending.

The three complete external files are now authored; the immutable copy has
the registered `a4386bae` full binary digest. Python3.9 grammar parsing is a
static source check, not an interpreter import, debugger session or target.
Installed API inspection removed a nonexistent shared-library stop enum before
execution; image events use actual breakpoint stops and notifier/kind identity.
There was no failed setup or target run. No diagnostic module has been imported.

Final registration, source/helper/workload and script SHA256 values are recorded
in external `evidence/prelaunch.json` before the command is invoked. The launcher
verifies all manifest hashes and ancestor-configuration identity before admission.
The actual outer command, debugger command, target arguments and bounds remain
exactly those registered above. The launcher records an exclusive attempt
directory and will refuse another invocation. No additional script is introduced.

Admission records three quiet-host snapshots with the existing host evaluator;
`include_calibration=False` and `require_work_counter=False` enforce the explicit
no-counters/no-CV scope. Any rejected snapshot ends this attempt. This is coverage
custody only, not performance admission/qualification or changed benchmark policy.
The monitor likewise uses the existing non-calibrating snapshot path.

If the proof PASSes only its stated coverage criteria, parent authorizes scoped
documentation completion, exact diff/HIR/file-size checks, one initial exact-SHA
Opus assessment review plus at most one remediation, and docs-only main merge
with phase record and terminal stop. Review covers evidence and claim boundaries,
not compiler/whole-phase/causal acceptance. No second proof is permitted by a
review. If the proof FAILs, preserve raw receipts and return NEEDS-NEW-SCOPE;
no automatic review, redesign, second target or next-item implementation.
