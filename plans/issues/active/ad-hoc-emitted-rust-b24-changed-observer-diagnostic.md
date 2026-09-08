# 12K-B24: changed-observer startup diagnostic

Date: 2026-09-08. Owner: performance / issue3776.
State: INCONCLUSIVE / NEEDS LATER SCOPE / TERMINAL STOP; full B24 OPEN.
No causal correction, acceptance, Opus review, PR or merge claim.

## Sole diagnostic outcome and terminal handoff

Prelaunch registration was committed/pushed as
`802a960836ebfdd72f2ac74f6d1bdbd3efee40a6` before parent/coordinator callbacks
and the sole command. One setup, one LLDB session, one warmup target70018;
no second target or retries. Total21.368428665969986 seconds. First live failure
was custody sequence62: `invalid acquisition: new unobserved acquisition
descendant`. Root69420, LLDB69936 and debugserver70020 were owned. All three
admission snapshots passed. No counter acquisition failure was reported.

The observer obeyed the outer stop at main entry and killed its target through
the retained SBProcess handle (`Kill` succeeded). Nineteen stops retain raw
counters, one mode3→mappedmode0 handover, seven entry hits and five matched
startup returns. Completion was never observed; canonical stdout/stderr are
both empty, no two-file formatter completion or target exit0 is established.
LLDB itself returned0 without timeout. That is not target success.

| Sole process / partition | Retained evidence |
| --- | --- |
| P0 warmup, PID/PGID70018 | INCONCLUSIVE;19 stops; all later targets unlaunched |
| Pre-prepare | first stop279924 instructions; prepare entry4278078 |
| Loader inclusive (`prepare`) | entry4278078; return34082915; delta29804837 |
| JIT fixups inclusive | entry6405715; return25905636; delta19499921 |
| Generic fixups inclusive | entry14212176; return25817460; delta11605284 |
| Fixups union |19499921; generic interval is nested, not an additional total |
| Loader exclusive of fixups |10304916; includes library/constructor work and is not a disjoint pre-constructor category |
| libSystem inclusive | entry28036990; return31702059; delta3665069 |
| Sanitizers inclusive | entry28470939; return28599694; delta128755, nested in libSystem |
| Remaining pre-constructor | unassigned; no qualified complete partition |
| Constructor / main stop |32325959 /34474683; main classification interrupted by outer stop |
| CLI/config/discovery, source check1/check2, completion residual | unavailable: stopped before buffered completion |

These are unqualified raw retained endpoints, not accepted attribution. The
named analyzer ran once after the attempt and preserved one partial warmup,
zero observed complete processes and no cross-process ranges. All counters and
partial events are in E/analysis.json; no debugger overhead was subtracted.
There is no stability, causation, production CV or full B24 pass.

The runner's final native collector was sticky-rejected and honestly recorded
`zero_processes=false`. The registered post-command check then proved the four
accepted root/debugger/target/tracer PIDs and groups absent. A supplemental
terminal record check extended that inventory to every82 observed subjects,
including unaccepted transient helpers and nested acquisition snapshots:
native-before/full-ps/native-after all absent, all four groups ESRCH, zero
signals. Watcher/host monitor are false. This supplemental read-only release
receipt does not rewrite or validate the failed custody chain, launch a target,
repair the apparatus or qualify a cleanup mechanism. All resources are released.

Evidence under E:

| File | SHA256 |
| --- | --- |
| boundaries/outcome.json | `8402a5d527e7dc088e189b26d012886106acdfb9417e3ff624401e1395bdfce2` |
| boundaries/process-release.json (failed proof retained) | `6602a49e205f0193f9b644dc73762965208537d29843b40def553d51254f222f` |
| fresh-process-release.json (four accepted PIDs) | `94a1840d73a35efebf4618d9591d9b4f42cc00c7adb67e57ea341ee9de174684` |
| complete-release-record.json (all82 observed subjects) | `2651f951e667cad43f20e02ec666d4059f7bf1320327902d6aff3c79c21791eb` |
| analysis.json | `839c6adc40ce246bb7b9785ca98db80d895fe8f602158ec16a0872ac6da4cc9b` |

Post-run relevant diff checks PASS, repository file-size PASS3770 files,
maximum external maintained source611 lines; all623 frozen inputs reauthenticated.
Only scoped Markdown is committed. No post-run tests, Opus requests, Sifr gates,
acceptance acquisitions, production changes, next-item code, PR or merge.

## Deferred findings for owner3776; not implemented

- **B24-CONT-F1 — acquisition/helper interaction:** sequence62 initially records
  root child70233 as `(Python)`, absent in native acquisition. Its fresh round
  contains root child70234 displayed `(ps)` and the current capture's `/bin/ps`
  child70235.70234 is outside the original subject set and is not that capture's
  exact probe PID, so the unchanged B38 new-descendant guard rejects. This is a
  preserved observation, not proof of a process's historical executable or an
  OS/security defect. Sequence63 then rejects `batch absence contradiction`;
  cleanup's `observed_rows` raises, and sequence64 has sticky native failure.
  A later owner must adjudicate the complete producer/monitor/acquisition and
  release interaction with these exact receipts; no blanket helper exemption,
  dropped subject, weakened identity or same-method live retry is authorized.
- **B24-CONT-F2 — interval partition assumption:** actual `prepare` entry is
  event6, constructor event16, and `prepare` return event17. The frozen analyzer
  requires whole startup intervals to end by constructor in `accounting`, so
  a future complete trace with this ordering would reject. The current partial
  trace stops first on missing completion; this later defect was identified by
  inspecting saved endpoints, not by another run. Later scoped accounting must
  handle intervals that span the constructor with unions/intersections and
  preserve unassigned work. No analyzer correction is made after the live failure.
- **B24-CONT-F3 — terminal subject inventory:** the original final-release list
  contains only accepted native subjects; rejected/unaccepted transients can
  be omitted. The supplemental all82-subject absence record resolves actual
  current resource retirement, while the original failed/four-PID receipts stay
  immutable. Any future launcher inventory correction belongs to later scope.

Next action: coordinator adjudication from this frozen terminal, then a new
separately scoped owner if warranted. This owner STOPS after durable issue3776
publication and terminal callbacks. Five unlaunched targets are not banked.
Full B24 causation/unchanged ten-case performance/budget, B27 joint delivery,
exact65, SQL and the original emitted-Rust phase remain OPEN.

## Authority and ownership

Current parent top B38 handoff and B24 changed-observer route authorize one
diagnostic continuation after B38's reviewed merge and native retirement.
The original B24 experiment registration remains attributable; its unlaunched
targets are not banked. This is one new allocation, with zero Opus requests,
zero Sifr gates and zero acceptance acquisitions. Production, B27, exact65,
SQL, and the full emitted-Rust phase remain open. The binding delivery contract
is the merged [B26 assessment](ad-hoc-joint-emitted-rust-delivery-assessment.md)
and [input inventory](ad-hoc-joint-emitted-rust-delivery-inputs.md).

Sole independent clone `/private/tmp/sifr-b24-cont.6WbF7L/sifr`, branch
`codex/item12k-b24-6WbF7L`, fresh-main base
`4b4cc339964baeeb6641e57dc669fef700a5fa24`. Own objectstore/index/refs,
no alternates, submodule setup, shared target or private target. Private sibling
TMPDIR `/private/tmp/sifr-b24-cont.6WbF7L/tmp`; E is
`/private/tmp/sifr-b24-cont.6WbF7L/evidence`. Parent and all predecessor
worktrees/indexes/files remain read-only. Exact initial registration is
`/private/tmp/sifr-b24-cont.6WbF7L/registration.md`.

B38 terminal authenticated:
`bae725e022af6e6c1a958abecec9c926d9143ec7ea734c14dc025c0ea4818443`;
367 terminal-bound paths,351 frozen inputs,380-path union authenticated.
The full predecessor evidence was independently copied to E/prior-b38.
B38 candidate f7e0f1fdb5a74862f5c3be1671263e66264db693, PR3820 merge
ab16c6687bb65bed22e1d3bead3a0682ce67ef2b; record PR3822 merge is this base.
Its493/493 offline evidence is reused for unchanged qualified guards.
Its live proof used no resampling; seq16/26 race coverage remains offline.
Its four nonblocking follow-ups are not B24 prerequisites.

## Authenticated executable and inputs

Owned E/sifr-experiment09 is byte-identical SHA256
`a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392`.
Source d294aa5b396f0f80916698a387d4abf0bf222e5b plus diagnostic patch
`b097f41b801029f6e5708d381c630b93447a5f7571ee5ae26c7b87fc6f29b085`
and probe `a8a5453c2933de0402d22def51e61f946f2a37b074e85013581e09456aa6ece3`.
Original B19/B20/B23/B22/B13 receipts from the original B24 registration
are authenticated and preserved in E/original-receipts. This executable is
diagnostic-only, not current main, B22's candidate or an acceptance binary.

The13-identity contract remains SHA256
`3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d`.
Root .gitignore blob1550a8c5bf3f3a524a1f7ac7e283b943ad8554f7,
sifr.toml blobf9cf91047544ed6a5defbaa955f40a63b1149841 and formatter-project
treec05f2d47b2e922ca63819b17dece5cacfb30bae0 match source and current main.
All ten inherited runtime/workload hashes match. Freeze additionally binds
root gitignore, ancestor config presence/hashes, Python/LLDB/SDK inputs and
the unchanged exact two-source-file inventory. No production mutation/build.

## Changed diagnostic apparatus and offline evidence

E/run_boundaries.py owns external Python host admission, one LLDB process,
local deadlines and continuous native acquisition. E/lldb_boundaries.py reuses
the B38 entry observer and adds every invocation's thread-specific unwound
caller/stack return pairing, raw stopped counters and the fixed six-target
schedule. E/boundaries.lldb contains the single import/run/quit sequence.
E/analyze_boundaries.py owns saved-event replay, buffered phase crosschecks,
interval unions, inclusive/exclusive and unassigned accounting.

Supporting boundaries_core.py owns pairing/SDK/local clock predicates;
boundaries_output.py owns per-target completion while the same root/LLDB
remain alive. It replays the unchanged B38 native/ack/custody contract,
requires all target/helper groups absent and consumes exclusive canonical
stdout/stderr inodes before allowing the next target. Final cleanup also
proves debugger/root/helpers absent. Qualified helper bytes are unchanged
apart from reversible owned path/run-ID relocation, explicitly hash-mapped.
No display-name lookup, metadata deduplication, raw LR, inferior evaluation,
signal-authority bypass or unproved absence is introduced.

SDK resource.h layout is16 UUID bytes plus35 uint64 fields (296 bytes).
The complete exact declaration and field/unit table is E/counter-layout.json.
Instructions/cycles/pageins/wakeups are counts; memory/I/O fields are bytes.
User/system/runnable and abstime fields retain raw Mach ticks; other time/energy
units not specified by the declaration remain raw SDK uint64 with no conversion.
All fields and syscall return/errno are preserved. No minor/major faults are
invented, no debugger overhead is subtracted, and Python clock origins are never
compared across processes. See E/counter-units.md for the producer references.

All code was completed before the two named offline commands:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b24-cont.6WbF7L/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b24-cont.6WbF7L/evidence/run_boundaries.py --self-test
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b24-cont.6WbF7L/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b24-cont.6WbF7L/evidence/analyze_boundaries.py --self-test
```

Final PASS121 observer checks and42 analyzer checks, plus reused unchanged493.
Nested/reentrant/same-return-PC and multithread pairing, all symbol families,
unresolved/ambiguous prelaunch and runtime symbols/returns, counter failure/
nonmonotonicity, clock/deadline/cleanup reserve, per-target native identity,
continuous watches across two synthetic targets, canonical output/identity/
release-before-next, partial warmup retention and all metric partitions are
covered without native process tests. Three invocations of each named suite:
initial103/106 and39/42 exposed my incorrect SDK36/304 assertion; corrected
106/106 and42/42; final prelaunch application-location checks121/121 and42/42.
Every failed/passing receipt and exact source snapshot is retained in E/offline.
No assertion was relaxed to permit a live failure or changed workload.

## Sole live allocation and terminal rule

Prelaunch freeze:623 bound paths,1,091,691,251 evidence bytes,125,078,925,312
free bytes. All frozen sources/inputs are read-only. SHA256 receipts:

| Owned file under E | SHA256 |
| --- | --- |
| frozen-manifest.json | `1b6cf4746f45fd64b25cb0b231fe2500561bbb3b60afcafcbc467b173e546d09` |
| prelaunch.json | `6d765c37c1e4d7c7b9e1a58946b885fdfc2b0253b9c03c9586afee3f6ad25946` |
| offline-summary.json | `4955fff8720c7ea3e8b0914aab0b48a81c7eee18e1e5bf4f98df4a05c86aa372` |
| run_boundaries.py | `a79f81157a590b74f71a2885106453a16dca7b5ac18e6b127878bc47080b4d18` |
| lldb_boundaries.py | `db46dfa60f70fa4aa7b0471659c604afb75b1621194f7269b44acaa0170b5e1e` |
| boundaries.lldb | `ab2acda19e56c5203af11ae22066a1ffdfbae41abd850ecf45873c3f39e4531d` |
| analyze_boundaries.py | `78da1d31aad52caeb1b1532c1b2f859af30763448d86d2e48dc3d788a9297c8f` |
| boundaries_core.py | `001d751e69d9e8601f2fd0b19d4bc9f03928477577bd0e0cc81ef178b10a5f52` |
| boundaries_output.py | `ceeb32f75c0d469d48c9f031a2958dd06fc7f931d19eeab956fd3335a993e514` |
| boundaries_tests.py | `29823fa1744415ef71a8de7ce5467d094aedab17f151e933d08a4da317032577` |
| counter-layout.json | `24eb87a67dd328ea7e334710a9554a95f490054f0e2b30e19f0c6e55c28b5f52` |

Commit/push this frozen prelaunch registration and source hashes, notify parent
and coordinator, then execute once under the already approved allocation.
No additional permission acknowledgment is required. An actual competing
host workload is coordinated; local admission remains authoritative.

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b24-cont.6WbF7L/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b24-cont.6WbF7L/evidence/run_boundaries.py
```

Cwd is owned clone. Exactly one `/usr/bin/lldb --no-lldbinit -b -s
/private/tmp/sifr-b24-cont.6WbF7L/evidence/boundaries.lldb` session. At most six
fixed targets: warmup0, observations1–5, run IDs `12K-B24-6WbF7L-target-N`.
Each runs the owned immutable executable `fmt --check --no-cache` against
owned `verification/areas/performance/formatter_project`. ASLR on, flags134,
B38 mode3→mappedmode0 handover. No DYLD/link/priority/security changes.

540s total including180 admission and FINAL30 cleanup; observations end by510,
refuse another target unless its full60s fits. 256events/target,2GiB evidence,
4GiB free floor. First live entry/return/counter/identity/host/budget failure
is terminal. Preserve partials, release all owned processes, no repair/retry,
extension, control arm or additional help/target launch.

Only named saved-event analysis, relevant diff/file-size/record checks, durable
issue3776/terminal and callbacks follow. This stage has no PR/merge requirement
and cannot close full B24. Stop after its terminal record. Further causation or
a supported production correction requires concrete coordinator adjudication.
Later unchanged full ten-case representative with the same-invocation budget
and both benchmark/budget self-tests remain required; B27/exact65/SQL remain open.
