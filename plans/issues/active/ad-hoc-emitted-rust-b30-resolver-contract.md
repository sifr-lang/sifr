# 12K-B30: complete observer identity and arming contract

Date: 2026-09-08. Owner: performance / issue3776; exactly B29-F1.
State: APPROVED COMPLETE APPARATUS IMPLEMENTED; OFFLINE99 PASS; SOLE PROOF NEXT.
The first-stage proposal and its chronology below are preserved. The parent
approved proposal05b2a2055dcac56dcdc32affb20fea5775163f19, document SHA256
644e46b9b7921073464c2ad38c6656050a3ebc9c7598e092c85f3814d0bae9eb, and the
coordinator cleared/reserved capacity. No further acknowledgement is required
after the prelaunch hash/start callback absent a material scope deviation.

## Ownership and inputs

Independent fresh-main clone `/private/tmp/sifr-b30.PiHI2k/sifr`, branch
`codex/item12k-b30-resolver-contract`, base
`491ba4ede1609ce476831015dc209a9064cd8ffc`. Private TMPDIR
`/private/tmp/sifr-b30.PiHI2k/tmp`; owned external inventory/evidence
`/private/tmp/sifr-b30.PiHI2k/evidence`. No alternate object store or shared index.
Parent dirty ledgers and all predecessor trees/refs/targets/evidence read-only.
No target directory or build; CARGO_TARGET_DIR unset in inventory commands.

Read the parent TOP B30 dispatch and following-heading B30 registration, full B29
scoped record and B28 assessment. B29 terminal authenticated SHA256
`8b33348c38800ad076517f3b51bb30587c462def46e85df7824ce64a46719d7a`;
remote record `34333c1023c50756fa6d8cfed4ffde8af2e23cb3`. Original scripts/binary
remain at `/private/tmp/sifr-b29.XrqnzO/evidence`; no import of its live observer.
B28 primary-source/disassembly artifacts are reused at
`/private/tmp/sifr-b28.prJZsY/evidence`; exact Apple source equivalence is unproved.

## Static inventory command registration (before execution)

The following exact commands run from this clone. They are source/API/receipt
identity and non-running module inventory, not behavioral tests or live proof.
Only the two inventory scripts below plus generated artifacts are authored now.

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b30.PiHI2k/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b30.PiHI2k/evidence/inventory_inputs.py
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b30.PiHI2k/tmp PYTHONDONTWRITEBYTECODE=1 /usr/bin/lldb --no-lldbinit -b -o 'command script import /private/tmp/sifr-b30.PiHI2k/evidence/inventory_modules.py' -o 'script inventory_modules.run(lldb.debugger)' -o quit
```

`inventory_inputs.py` authenticates the predecessor terminal, listed raw/script
identities, installed Python API and dyld; records command/runtime versions;
reads installed API signatures; retrieves only pinned `SBModule.cpp` and
`Module.cpp` from Swift6.3.3 source commit
`82cdc19fa54d566969527b56f587ea8ea30bef51` to explain concrete lookup semantics.
It writes exclusive external inventory receipts and source copies.

`inventory_modules.py` creates ONE non-running SBTarget module-inspection
context for the immutable predecessor binary and installed dependencies. It
never calls Launch, Attach, Continue, expression evaluation or target functions,
never creates breakpoints, and asserts the SBProcess is invalid before/after.
It enumerates all symbol entries for dyld, libSystem, sanitizers and the app,
retaining relevant full names/mangled/display names, symbol types, UUID,
header/file/section-relative addresses and validity. It records full-name typed
FindSymbols and full-name FindFunctions results for every required entry plus
initial entry, image notifier and handover helpers. No unchecked Any fallback.
Invalid/unloaded static addresses remain explicit; no static result claims an
installed trap or live image transition. Missing module/API capability is saved
as a missing mapping, not followed by an inferior launch.

The static context may use AddModule only to inspect one of the three explicitly
named installed OS modules when absent from the loaded dependency inventory.
It does not set section loads or manufacture an active process mapping.
Exact scripts and this registration are hashed before inventory execution.

## Delivery boundary

First return the complete identity mapping, deterministic resolver design and
wrong type/name/UUID/ambiguity/unloaded-map negatives, exact proposed owned
implementation/test paths and commands, and complete initial/handover arming
sequence. If the mapping cannot be supported, return precise missing data.
No live target, attach/continue, build/Cargo/Clippy, counter/CV acquisition,
Opus review, Sifr gate, PR or merge at this stage. No next-item code.
Future one-target proposal retains 300s total/final30s cleanup, 256 events,
1GiB storage/4GiB floor, separate clock domains and authenticated custody.
Only meaningful mapping/proposal/terminal callbacks to parent
`01a06e86-414a-7e11-9256-1f45bdb5a6c7` and coordinator
`01a07d84-7c62-77f0-b7c7-ecf310829a11`; no new user permission loop.

## Additional disk-image inventory registration (before execution)

The first static context completed without a process. LLDB selected the shared-
cache representation of dyld (header file6443581440), matching B29's module
coordinates. All three required C++ names are enumerated code symbols; exact
display-name queries return0 in BOTH APIs, while exact mangled queries return1.
The remaining bounded inventory step reconciles this representation against the
actual bootstrap disk image, using one byte-identical owned inventory copy.
This is a second static LLDB session/context, still zero inferior executions.
Exact commands from this clone, registered before execution:

```bash
cp /usr/lib/dyld /private/tmp/sifr-b30.PiHI2k/evidence/dyld-on-disk
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b30.PiHI2k/tmp PYTHONDONTWRITEBYTECODE=1 /usr/bin/lldb --no-lldbinit -b -o 'command script import /private/tmp/sifr-b30.PiHI2k/evidence/inventory_disk.py' -o 'script inventory_disk.run(lldb.debugger)' -o quit
```

`inventory_disk.py` imports only the owned inventory serialization helpers,
authenticates the owned dyld copy against installed dyld's registered hash and
creates an arm64e module context with dependency loading disabled. It records
all dyld symbol names/types relevant to the prior inventory, section/UUID/file
coordinates and exact typed/full-function query results for the mapped names.
No section loads, breakpoints, process, expression, launch/attach/continue.
The original module inventory remains immutable. The new receipt will state
whether disk/cache code-relative identities actually agree; no assumed match.

## Derived complete mapping inventory registration (before execution)

One final pure data inventory script, `inventory_contract.py`, reads the two
immutable inventory JSON files and B29's SYMBOLS literal, selects exact code
entries and emits `identity-contract.json` with all13 identities and their
provenance/coordinates/API observations. It calls no LLDB API and is not an
implementation resolver or offline behavioral test. Exact command:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b30.PiHI2k/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b30.PiHI2k/evidence/inventory_contract.py
```

## Installed mapping result

All37 authenticated input/source identities matched. Both module inventories
completed: two static debugger sessions, two non-running SBTarget contexts,
zero inferior launches/attaches/continues, zero breakpoints. Each context had
an invalid SBProcess and was deleted. The first context loaded installed
dependencies without AddModule; the second inspected an authenticated owned
disk-dyld copy with dependencies disabled. No processes or load addresses were
manufactured. External inventory scripts are not a live observer or resolver.

Full exact names (requested, mangled and display), types, module paths/UUIDs,
start/end section offsets, API query contexts, disk/cache agreement and source
receipt hashes are in `evidence/identity-contract.json`. Its13 entries select
the exact8 required entry families and5 bootstrap/handover identities. Each
exact lookup returns one code-symbol context through typed FindSymbols and one
through FindFunctions Full. The OS contexts contain SBSymbols, not SBFunctions;
an implementation must not assume GetFunction is valid after FindFunctions.

The concrete B29 failure is reproduced statically: full display-name queries
for prepare/JIT/generic each return0 through BOTH typed FindSymbols and
FindFunctions Full, even though enumeration returns those exact display names
and their code symbols. The corresponding exact mangled-name queries each
return1 through both APIs. Replacing only FindSymbols with FindFunctions would
therefore preserve this setup failure on the inspected installation. This
does not identify why Apple's internal name index behaves this way.

Pinned reference `SBModule.cpp:332` uses the unified symbol table's name/type
lookup; `SBModule.cpp:390` enables symbols and inlines for FindFunctions, and
`Module.cpp:836` searches debug functions and function symbols. Installed API
signatures and observed query results are authoritative here; the Swift6.3.3
reference and upstream API pages do not prove Apple's exact binary internals.

All entries below are eSymbolTypeCode=2 in `__TEXT.__text`. Offsets are relative
identities, NEVER future absolute load addresses. Exact lookup strings live in
the manifest; C++/Rust entries use the observed mangled name, the C/assembly
entries use the exact observed name and require a null mangled name.

| Entry | Module identity | Header-relative offset | __text-relative offset |
| --- | --- | --- | --- |
| prepare | dyld | 0x20b4c | 0x1fb4c |
| JIT applyFixups | dyld | 0x412c0 | 0x402c0 |
| generic applyFixups | dyld | 0x48374 | 0x47374 |
| libSystem_initializer | libSystem | 0x125c | 0xbc |
| _sanitizers_init | sanitizers | 0x37ec | 0x2f4c |
| sifr::b20_probe::constructor | app | 0x100378 | 0xffa68 |
| sifr::cli_model_and_entrypoint::main | app | 0x960e8 | 0x957d8 |
| sifr::b20_probe::dump | app | 0x1003a4 | 0xffa94 |
| _dyld_start | dyld | 0x49c0 | 0x39c0 |
| lldb_image_notifier | dyld | 0x392cc | 0x382cc |
| prepareInCacheDyldAllImageInfos | dyld | 0x3e7e0 | 0x3d7e0 |
| completeAllImageInfoTransition | dyld | 0x3e9bc | 0x3d9bc |
| restartWithDyldInCache | dyld | 0x49d4 | 0x39d4 |

Module identities:

- dyld `/usr/lib/dyld`, arm64e UUID `74E52480-C2BD-3C8D-812D-95FE2B74A096`.
  Disk header file0/TEXT737280bytes; cached header file6443581440/TEXT734464bytes.
  Both __text starts are header+4096, length647224. All8 listed dyld identities
  agree in UUID, code-section offset/length and header-relative offset. Data
  segments differ substantially; do not transplant their header-relative layout.
- libSystem `/usr/lib/libSystem.B.dylib`, arm64e UUID
  `4FED5EE2-5D3E-35B1-A170-9859C4B683BB`; static header6714400768,
  __text6714405280/1564bytes.
- sanitizers `/usr/lib/system/libsystem_sanitizers.dylib`, arm64e UUID
  `D88EC709-9AA8-3083-B22B-D3CB39678D71`; static header10837291008,
  __text10837293216/25080bytes.
- app, arm64 UUID `E527F68E-54F9-3463-ABC5-F8C8DD88A725`, predecessor binary
  SHA256 `a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392`;
  static header4294967296, __text4294969616/58471352bytes. A future owned copy
  must match this hash; only its registered path may replace the inventory path.

Negative real-world discriminators are present in the inventory: libSystem has
a `_sanitizers_init` trampoline (type5) in `__auth_stubs`; dyld has similarly
named prebuilt-loader, block-invoke and cold symbols. Full exact names, code
type, owning module UUID and section identity exclude them without Any fallback.

B29's observed initial module used the cached file-address domain while its
actual header load was4509745152. PC minus header load18880 matches both disk
and cache entry identities. This supports a domain-independent relative
resolver; it does not establish that cache startup had already occurred.
The old live header/PC values are never resolver constants for a future process.
All new static load addresses are LLDB_INVALID_ADDRESS as expected.

## Proposed complete resolver and arming contract (not implemented)

Immutable inventory evidence under `/private/tmp/sifr-b30.PiHI2k/evidence`:

| Artifact | SHA256 |
| --- | --- |
| input-inventory.json | 34727abf7de2df3587f45206d9cb2b661274cc63995069e9d7eba502c2e04ef9 |
| module-inventory.json | a14745757edf8a595f2106643e214ace53748e0da5409c1b84c0c85cbf0d2eb1 |
| disk-inventory.json | 3536715fa3c8262b71d5a7d5f055c8ab03a43121590951361d10c939e4ad711a |
| identity-contract.json | 3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d |
| SBModule.cpp | f0384391905aa2ac6e755183b8abccc5a08bffffb8dcac4ab2486e632924c58f |
| Module.cpp | 257a1820a339667fb80fd520f171eb300aee20e73f23a165e0bc10472cf3dc33 |

One deterministic authority would use `FindSymbols(exact_lookup,
eSymbolTypeCode)` in the uniquely identified current module. Require exactly
one valid symbol and exact observed mangled/name/display identity, code type,
UUID, __TEXT.__text path, bounded start/end offsets, and the manifest's relative
entry. No alternative query, normalization, fuzzy matching or Any fallback.
The function inventory corroborates identities but is not a second runtime path.

Separate static identity from live binding: derive load address only through
the current symbol's SBAddress and current section load. Require a non-invalid
load, same module/UUID/section+offset on ResolveLoadAddress round-trip, correct
current header/section relationship, and a successful memory-region query with
executable bounds containing the actual entry. B29's PC-region interval proves
only its initial PC, not other entries; each future entry needs its own region
check. Epoch identity includes process/stop, module, UUID and current section
loads. A module transition invalidates every old binding. Never use static
file VA as a load address or reuse another process's absolute address.

The complete sequence would be:

1. Authenticate binary/workload/helpers/inventory and final source/registration
   hashes. Configure ASLR enabled, skip-prologue false, shared-library events
   true. Register pending exact module/name breakpoints for the five later
   entries; a pending OS entry is not an installed site. Launch flags134 only.
2. Obtain the initial stop/custody handshake. Preserve B29's unchanged
   initial_entry_matches predicate. Validate current dyld and initial PC, then
   bind and arm prepare/JIT/generic AND the exact notifier entry by address.
   Record valid/enabled/resolved location, one exact load address, module/UUID,
   region, breakpoint+location IDs and stop before any continue. The notifier
   is an observer transition site; it is not a ninth required workload family.
   Validate all already-loaded later sites; retain genuinely unloaded pending
   sites until the corresponding image-added stop. App identity must already
   match the owned binary. Do not fail because an initializer is not yet loaded,
   and do not claim its pending location as coverage.
3. At each stop capture raw reason/data, PC, x0/x1, thread/stop ID, complete module
   and site snapshots BEFORE classification or current-module lookup. A normal
   required entry must match its verified site and exact current image; multiple
   breakpoint IDs at one notifier address identify the same transition family.
4. Mode3 handover is recognized only at the notifier PC verified in the preceding
   epoch, with a breakpoint stop, x0=3 and x1=1, before any required entry hit.
   The internal LLDB callback may already have emptied the module list and
   removed section mappings. Retain this explicitly expected empty-map state;
   do not depend on GetFunctionName or a description substring after removal.
   The prior verified notifier binding authenticates this single transition
   event, not a reusable mapping. Record unavailable current symbol/caller
   fields honestly. Remove owned old dyld sites, invalidate every load binding,
   and allow exactly one continuation to the registered mode0 handover stop.
5. At that next stop require x0=0, breakpoint reason, freshly available dyld
   module/UUID/current mapping, and PC equal to its newly resolved exact notifier.
   The caller may still be in disk startup; it is evidence, not the active-image
   selector. Require the header mapping to change for the mode3 route. Re-arm
   all three early sites and notifier from the fresh module and validate all
   now-loaded initializer/app locations before continuing. Missing mapping,
   wrong next stop/mode, another transition or prior hit makes the proof fail.
   No manual image loads, instruction stepping or alternate notifier route.
6. Other image-added/removed events require the exact current notifier binding,
   mode0/1/2, and renewed current-image/site checks; unknown empty-map states
   fail. Newly loaded OS entries must obtain verified sites at their image stop
   before returning into initialization. Required image removal/UUID change
   fails. The explicit successful mode3-to-mode0 route must be observed once;
   skipping it is not a handover coverage pass.
7. Require prepare hit before either fixup family; all three early families
   before libSystem/sanitizers/constructor/main/completion. For every required
   hit validate actual breakpoint/location PC and current module/type/entry
   identity, not just an ID-to-label map. Require all8 families, both file-check
   phases, exit0, and authenticated zero-process release. Coverage only; no
   counters, return intervals, causality or performance acceptance claim.

This covers the bootstrap-before-prepare and transient empty-map state already
documented in B28. Pinned DynamicLoaderMacOS.cpp:366 clears images/section loads
at mode3 and sets its internal one-shot notifier; mode0 at334 refetches images.
Installed B28 disassembly corroborates mode3 and mode0 before cache restart and
prepare. Actual delivery/trap installation under Apple's build remains exactly
the capability to be proved by the proposed one target, not an established pass.

## Concrete proposed paths, checks and one-target proof

After adjudication only, all new files under this owner's `evidence`:

- `coverage_symbols.py`: pure deterministic identity/binding/transition predicates
  and their single `--self-test`; no LLDB import in the offline command.
- `coverage_address.py`: byte-identical B29 helper, retained and imported unchanged.
- `lldb_coverage.py`: B29-derived observer using those predicates for all8 families,
  current-site binding and the complete notifier/handover sequence above.
- `run_coverage.py`: B29-derived outer custody/deadline/manifest launcher, owned
  paths relocated and all new helpers/contract identities included in custody.
- `coverage.lldb`: fixed import/run/quit batch; `sifr-experiment09`: byte-identical
  immutable diagnostic copy. No other executable or production change proposed.

Complete implementation precedes the single offline check command, cwd the clone:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b30.PiHI2k/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b30.PiHI2k/evidence/coverage_symbols.py --self-test
```

Offline plan: table-driven positives for every manifest entry and both dyld
representations, relocated synthetic live mappings and the full initial/mode3/
empty-map/mode0/early/later sequence. Replay B29's preserved initial coordinate
receipt through its unchanged helper. Require explicit rejection of wrong
name/mangled/display/type/UUID/path/architecture, block/cold/trampoline entry,
zero/two contexts (including duplicate exact identities), invalid/zero-size/out-
of-bounds section, wrong section or header offset, unloaded/invalid/overflowed
load, wrong ResolveLoadAddress round-trip or nonexecutable/out-of-region entry,
stale epoch/address after handover, unverified notifier PC, unexpected empty-map,
wrong/repeated/missing mode3-to-mode0, missing/reused site or mismatched location,
and later boundary before any required early family. Synthetic memory/process
rows are labeled as such; offline checks cannot prove LLDB trap installation.
All negatives call the same pure predicates the observer uses. No compiler test.

Only on PASS, adjudicated final script/input hashes and coordinator capacity,
exactly one changed-apparatus proof command:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b30.PiHI2k/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b30.PiHI2k/evidence/run_coverage.py
```

It runs `/usr/bin/lldb --no-lldbinit -b -s
/private/tmp/sifr-b30.PiHI2k/evidence/coverage.lldb`; sole target argv is
`/private/tmp/sifr-b30.PiHI2k/evidence/sifr-experiment09 fmt --check --no-cache
/private/tmp/sifr-b30.PiHI2k/sifr/verification/areas/performance/formatter_project`.
One setup/session/target, zero retries; 300s total from admission start, up to180s
admission/30s setup/60s target with final30s reserved inside total for cleanup;
256events, 1GiB owned storage including inventory/copies and4GiB free floor.
External Python3.14.7 owns its absolute deadline; embedded Python3.9 owns only
its local elapsed durations. No transfer/comparison of absolute clock origins.
Retain B29's merged-B23 debugger-group helper, separately authenticated inferior
group, launch-pending custody, handshake before continue, bounded output,
watchdog/reaping and zero-process release. No host window requested yet.

Any first setup/guard/custody/coverage failure is terminal INCONCLUSIVE with raw
evidence and precise later owner under3776; no repair/retry/alternate mechanism
or automatic review. Proposed success-only final named checks: exact
`git diff --check <base> <candidate>`,
`python3 scripts/check_hir_maintainability_guardrails.py`,
`python3 scripts/check_file_size_guardrails.py`; one exact-SHA Opus assessment
and at most one remediation, then docs-only merge/phase record and stop. No
create-pr/merge Sifr gate for a Markdown-only Git delta. This success branch
still awaits adjudication; no review allowance has been spent at this checkpoint.

No static identity remains missing for the registered13 entries. Future process
load maps, installed sites and stop delivery are necessarily unobserved. B24
cause/full unchanged representative/budget acceptance, B27 joint delivery and
all retained generated-Rust phase obligations remain unmet. No later item starts.

Next action: send this complete mapping/proposal and external receipt identities
to the parent/coordinator for automatic scoped adjudication. Hold execution;
do not request redundant user authorization or treat this checkpoint as terminal
closure. Pre-execution registration/script hashes and command outcomes are
retained in the external execution chronology; the exact command sections above
remain in this proposal. No final review or terminal closure is claimed.

## Execution authorization and completed apparatus

Parent explicitly approved the complete proposal and same-owner implementation,
single offline check and, only on PASS, self-registration/freeze and one proof.
Coordinator's conditional capacity clearance is effective with that scope
approval. Both callbacks are required before launch; another acknowledgement
is not. All limits/failure/success-only review-and-merge rules above apply.

All proposed source files are now implemented before the offline command:
`coverage_symbols.py` supplies the pure identity, binding, site and Protocol
predicates plus the one self-test; `lldb_coverage.py` uses these for all8 required
families and verifies all8 dyld identities at each arming epoch. Exact typed
mangled/C-name lookup is the sole runtime path. It records initial-entry inputs,
pending sites, complete live identity/binding/site records and every stop/continue.
The observer arms its own verified notifier site alongside the three early
sites; mode3 classification survives the expected empty module list by matching
the previously verified notifier PC. It invalidates old bindings and re-arms
from the freshly mapped mode0 image before continuing. Every required hit checks
the actual location ID/PC and renewed image mapping. Protocol enforces exactly
one handover and all8 entry families plus both file phases/exit0.

`coverage_address.py` and `sifr-experiment09` are byte-identical B29 copies.
`run_coverage.py` retains B29's custody/clock/watchdog/admission/release mechanism;
only the owned root and B30 labels change. The new manifest will include every
new helper/contract plus the unchanged B23 helper/workload/ancestor config.
The single offline command also parses all four Python files against3.9 grammar
without importing the live observer. No command or live attempt has run yet.

## Frozen offline result and prelaunch registration

The single named offline command executed once and PASSed99/99 with no failures.
This includes all13 static/live synthetic identities, disk/cache positives,
actual wrong-module trampoline/block/cold negatives, unknown/ambiguous contexts,
wrong name/type/UUID/mapping/site cases, complete and rejected handover/entry
sequences, and B29's authenticated live coordinate receipt/unchanged predicate.
All four Python source files parse as Python3.9. This is not a live coverage pass.

The complete external implementation is frozen at these SHA256 identities:

| Owned artifact | SHA256 |
| --- | --- |
| coverage_symbols.py | 57590d84f0f7b3ce6ebc6003647db7df95a0de24c4cf4de95281072e05d3202d |
| coverage_address.py | 0df357c922d11386b5af80cfca7648d5542c8ebdec1072ba136a49be7c0bca9b |
| lldb_coverage.py | d2033d1b3bfde0493c54eda381777a01218d956626e6794822c8e0934dc95c26 |
| run_coverage.py | 884dfd3887264461b7c6b4070bae97760e13e385660fd6624f879cdd588390d7 |
| coverage.lldb | 9f93bd03b0a6387b0e76ca6d752c048db8fbf6b6a3021da090e34de51e183882 |
| sifr-experiment09 | a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392 |
| offline-result.json | 77598cac829f297bec1a31f377d6bee0b830b59635cf79c5f972ad59ce248ef3 |
| identity-contract.json | 3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d |

The final external prelaunch.json will bind these exact bytes, installed API and
dyld, all new static receipts, this committed registration and the main phase
record, unchanged owned workload/config, B23 custody helpers and predecessor
receipt identities. Ancestor config presence is recorded before launch. The
launcher authenticates this manifest before admission. Prelaunch registration
is copied outside Git before any run result changes this document.

The sole command, target argv, flags134, first-error/no-retry behavior and all
limits are exactly the approved proposal. Initial free disk139338388KiB;
owned evidence181612KiB including both immutable binary inventory/proof copies.
The launcher will recheck floor/cap at admission and during custody. No target
directory exists and no cleanup is needed or performed. The external and
embedded clocks stay separate. Parent/coordinator receive final manifest and
registration identities plus actual start notice BEFORE execution; they have
already authorized execution without another acknowledgement.
