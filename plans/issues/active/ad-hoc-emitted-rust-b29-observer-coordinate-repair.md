# 12K-B29: verified-image initial-entry coordinate repair

Date: 2026-09-08. Owner: performance / issue3776. B29 is exactly B28-F1.
State: IMPLEMENTED / REGISTERED BEFORE OFFLINE TEST AND SOLE LIVE PROOF.

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
