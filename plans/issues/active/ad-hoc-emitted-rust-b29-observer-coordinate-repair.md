# 12K-B29: verified-image initial-entry coordinate repair

Date: 2026-09-08. Owner: performance / issue3776. B29 is exactly B28-F1.
State: NEEDS-NEW-SCOPE / COVERAGE INCONCLUSIVE / NOT REVIEWED / NOT MERGED.
The prelaunch registration below is preserved chronologically; the terminal
result follows it. No repair or later-item implementation follows this failure.

## Scope and ownership

The parent phase's top B29 dispatch and following B29 registration authorize
this concrete apparatus correction, deterministic offline tests, and one live
coverage proof. The user reiterated that exact scope in this session. No new
feasibility study or permission checkpoint applies. Coordinator capacity is
free/reserved conditional on concrete root, commands and prelaunch hashes.

Owned root: `/private/tmp/sifr-b29.XrqnzO`. Fresh independent main clone:
`/private/tmp/sifr-b29.XrqnzO/sifr`; private TMPDIR:
`/private/tmp/sifr-b29.XrqnzO/tmp`; external evidence:
`/private/tmp/sifr-b29.XrqnzO/evidence`. Branch:
`codex/item12k-b29-observer-coordinate-repair`. Exact base/main:
`491ba4ede1609ce476831015dc209a9064cd8ffc`. No alternate Git object store,
shared index or target. The parent dirty ledgers and every predecessor are
read-only; their history is not imported into this main-based delivery.

Read the complete B28 assessment, actual registration and terminal in
`/private/tmp/sifr-b28.prJZsY/sifr/plans/issues/active/ad-hoc-emitted-rust-b28-observability-feasibility.md`.
Its remote terminal record is `e56c9882bc555d9717eb86580d8a663950c57a8b` on
`codex/item12k-b28-observability-feasibility`, based on the same main SHA.
Authenticated original external sources under `/private/tmp/sifr-b28.prJZsY/evidence`:

| Input | SHA256 |
| --- | --- |
| `terminal.json` | `3722091a0bdfdea8e7070f13dcb3196028301b8a72c0cbecee9adba27daab614` |
| `run_coverage.py` | `0c1858bf4fd10d419b772a9d4868a24b307edcf63aca2330af9550bcded2a785` |
| `lldb_coverage.py` | `29d81013eb32291798639b97f593aa3a04e69713d6384a90cfda3f1570bde344` |
| `coverage.lldb` | `9e4e47c91918202df759139246354fcb18b1b79f8dd4315283b504570d796b7a` |
| `sifr-experiment09` | `a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392` |

The three original scripts and immutable binary were copied into this owner's
evidence root. Only address validation, root relocation and B29 labels change.
The fourth authorized file is `coverage_address.py`: a pure predicate imported
by the observer plus its deterministic `--self-test`. No production source,
lockfile, fixture, workflow, build or benchmark policy changes.

## Complete implementation and named commands

The same predicate requires the expected dyld path/UUID for the active module
and resolved PC, valid header and TEXT file/load addresses, both header-relative
offsets equal to `0x49c0`, consistent section-relative mapping, section bounds,
and an executable region containing the PC. The observer saves all live inputs
and the predicate result before arming traps. Existing stop, UUID and executable
checks remain. The failed direct LLDB file-VA/disk-offset comparison is replaced.

Offline cases authenticate and replay B28's raw initial stop and terminal:
PC4550560192, header/TEXT load4550541312, header/TEXT file6443581440,
PC file0x1801189c0, TEXT size734464. The positive check uses these coordinates;
because B28 saved no live memory-region bounds, the offline executable interval
is explicitly synthetic. Additional positives cover a zero-based file domain
and relocated load domain. Negatives cover wrong offset, UUID, image, header,
section/file/load mapping, executable bounds, invalid addresses and overflow.
All 26 cases call the exact observer predicate. No LLDB import or live target
occurs in the offline command. `offline-result.json` is exclusively created.

All four scripts are complete before testing. Exact first command, cwd the clone:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b29.XrqnzO/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b29.XrqnzO/evidence/coverage_address.py --self-test
```

Only if that command PASSes, the sole corrected coverage command, same cwd:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b29.XrqnzO/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b29.XrqnzO/evidence/run_coverage.py
```

Debugger argv: `/usr/bin/lldb --no-lldbinit -b -s
/private/tmp/sifr-b29.XrqnzO/evidence/coverage.lldb`. Single target argv:
`/private/tmp/sifr-b29.XrqnzO/evidence/sifr-experiment09 fmt --check --no-cache
/private/tmp/sifr-b29.XrqnzO/sifr/verification/areas/performance/formatter_project`.
The unchanged workload contains `main.sifr` and `util.sifr`. Launch flags134:
debug2, stop-at-entry4, separate-group128. ASLR remains enabled. Identity and
corrected scripts, imported helpers, workload, original receipts, registration,
offline evidence and ancestor configuration are hashed before launch in
`evidence/prelaunch.json`; the launcher authenticates them before admission.
Root/commands/hashes are sent to parent and coordinator before execution.

## Unchanged proof protocol and stop rules

Exactly one setup, debugger session and target; zero retries. Total300s includes
up to180s admission,30s setup,60s target and final30s cleanup. At most256 events,
1GiB evidence including binary,4GiB free floor. Admission rejection consumes the
attempt. Three non-calibrating admission snapshots and the unchanged monitor
provide host custody only; no counters, CV or performance qualification.

The outer process owns its monotonic absolute deadline/watchdog; embedded LLDB
uses its own elapsed durations. PID/PGID/creation identity and target handshake
must agree before any continue. The unchanged B23 helper owns the debugger
group; the launcher owns and authenticates the separately grouped inferior and
checks cleanup/reaping. A release receipt must prove no owned processes remain.

At the earliest stop validate dyld entry, mapping and executable bounds; arm
unique enabled/resolved address sites for prepare, JIT applyFixups and generic
applyFixups in the active dyld. Preserve every module transition. At mode3 remove
stale sites, allow the single handover continuation, require mode0 and re-arm
the current UUID/header/section mapping before continuing. Missing/ambiguous
maps, sites, handover or unexpected stops end the proof immediately.

Require actual prepare entry before JIT/generic fixups and all three before any
later boundary, with stop/thread/site identity. Require libSystem, sanitizers,
constructor, main and completion entry hits, both file-check phases, exit0 and
clean release. Preserve stdout/stderr, stops, sites and every continue. Coverage
success does not prove return intervals, B24 cause, representative acceptance,
B27 integration or whole-phase completion. No OS/linker/ASLR/priority changes,
production edits, Cargo/builds, counters/CV or inherited unused target allowance.

On any new guard/custody/mechanism defect, preserve truthful terminal evidence,
record the later item under3776 and stop: no repair, retry, automatic review or
merge. No next-item code. On proof success only, named final checks are
`git diff --check <base> <candidate>`,
`python3 scripts/check_hir_maintainability_guardrails.py`, and
`python3 scripts/check_file_size_guardrails.py`. File-size and exact record diff
also apply to terminal documentation. No create-pr or merge gate applies to this
docs-only Git delta. One exact-SHA Opus assessment plus at most one remediation;
no further target allowance, third review or whole-phase review. Final review
evidence stays outside the approved Git tree, keyed by candidate SHA. After
success, merge the docs-only PR, update the phase record and stop.

Callbacks: parent `01a06e86-414a-7e11-9256-1f45bdb5a6c7`, coordinator
`01a07d84-7c62-77f0-b7c7-ecf310829a11`: actual start, outcome and terminal only.

## Sole-proof terminal result and later owner

The registered offline command PASSed all26 cases. Its exact guard SHA256 is
`0df357c922d11386b5af80cfca7648d5542c8ebdec1072ba136a49be7c0bca9b`.
The sole live invocation ended INCONCLUSIVE in7.694957792s, launcher exit1:
one debugger session, one target, one initial event, zero continues, zero early
sites and zero entry hits. The corrected coordinate predicate returned true at
the actual stop; full coverage did not pass. No retry or postfailure code change.

Exact raw initial identity, from `coverage/events.json`: stopID1, SIGSTOP17,
thread127815425, function `_dyld_start`, active and PC module `/usr/lib/dyld`,
UUID `74E52480-C2BD-3C8D-812D-95FE2B74A096`. Main diagnostic image UUID is
`E527F68E-54F9-3463-ABC5-F8C8DD88A725`, at the owned binary path. Dyld values:

| Coordinate | Live value |
| --- | --- |
| PC load / file | 4509764032 / 6443600320 (`0x1801189c0` file) |
| Header load / file | 4509745152 / 6443581440 |
| TEXT load / file / size | 4509745152 / 6443581440 / 734464 |
| Executable region, exclusive end | [4509745152, 4509974528) |
| Header-relative offset in both domains | 18880 = `0x49c0` |

At stage `initial-trap-arming`, the unchanged observer's active-module API call
`module.FindSymbols(SYMBOLS[label][1], lldb.eSymbolTypeCode)` returned a context
list of size0 for label `prepare`. Exact configured symbol string, preserved in
the hashed observer: `dyld4::prepare(dyld4::APIs&, mach_o::UnsafeHeader const*)`.
The raw exception is `AssertionError: ('exact active symbol not unique',
'prepare', 0)` at `lldb_coverage.py:89`, called from `run` at line246. No exact
prepare start address or site was obtained. JIT/generic availability, transition
coverage, all eight entry families, file completion and exit0 remain unproved.
This records the observed lookup failure, not its cause, general symbol absence,
an LLDB/OS capability limit, runtime defect or B24 performance causality.

Later item **12K-B29-F1**, performance / issue3776: the initial active-dyld
module's exact prepare lookup yielded0 contexts despite the validated initial
image/mapping. Record only. Parent/coordinator own the later registration;
no new lookup design, alternate target, retry, repair or research is implemented.

Inferior PID/PGID20326 was killed successfully at the initial stop. Debugger
PID/PGID20285, debugserver PID/PGID20327 and launcher19712 are absent in an
independent final process check. `process-release.json` records both owned
groups absent, remaining[], monitor_alive=false, watcher_alive=false. The
169MiB evidence root retains the immutable binary and all four final scripts;
no predecessor or owned evidence was cleaned. Capacity is released. Three
non-calibrating admission snapshots passed; no timeout branch was exercised.

Prelaunch registration commit `f2fcc1641a17c39819cf9c6cfc27cb87dd52f5af`,
registration SHA256 `2f43b6b8b2b2a7749193534492680aceb5cee934876b1025e5895b12b73ce64a`.
The original text is retained in Git and `evidence/prelaunch-registration.from-git.md`.
Root/commands/script hashes were notified before the offline command; the
completed manifest and offline hashes were notified before the live command.
No prelaunch identity is backdated. Final sources equal those tested/launched.

| Evidence under owned `evidence` | SHA256 |
| --- | --- |
| `prelaunch.json` | `29eaea9e76156f4886795a5c29598a93f3743d25d41e4ac96c98e10385fbec6f` |
| `offline-result.json` | `3674d5ffee7679cc77dd001b4f940efa12e7cb99214bd5761145b3fa34d1ac47` |
| `coverage/outcome.json` | `0e7ca9ce63d3064349f0398d048d3ded18bbb109b700e35285894d79a4efb328` |
| `coverage/events.json` | `af2f8db1ff5cea92f5a1af1e2fec4b20857466478e801f6a1db483081a22a7fa` |
| `coverage/observer-result.json` | `41748ff56f5f0209b1012b8e948005ecb40332360765071d49ebdf78f46e841f` |
| `coverage/process-release.json` | `19bdece1641fa05dad1fac66a17de656c30f17d312986e536db35ceca2244630` |
| `coverage/debugger-result.json` | `2ef5ea41042cf30653e9cf82537586cb81f208dd06a3c02c274596b06d60720a` |
| `coverage/inventory.json` | `cf5469b6dcaee969b74737349ce996ab0bae3f4563bce636dbb926b2b8e3aab4` |

Counts: one offline command/26cases, one sole coverage invocation, zero retries,
zero initial/remediation Opus requests, zero PRs, zero merges, zero Sifr gates,
zero production edits/builds/compiler tests/counters/CV/acceptance acquisitions.
The proof-success HIR/review/merge branch was not reached. Only terminal record
diff and file-size checks follow; their exact results and terminal record SHA
are external `evidence/terminal.json`. B24 cause and unchanged acceptance, B27,
and every full-phase obligation remain open. Parent acknowledged the failure
and directed this faithful terminal record and stop. Next action: STOP.
