# 12K-B33: module instance identity at mapped handover

Date: 2026-09-08. Owner: performance / issue3776. Exactly B32-F1.
State: MERGED / STATIC-OFFLINE CONTRACT COMPLETE; blocker none.

## Scope and ownership

Sole session owns `/private/tmp/sifr-b33.W1oIqs/sifr`, independent fresh-main
clone, branch `codex/item12k-b33-module-instance`, base
`491ba4ede1609ce476831015dc209a9064cd8ffc`; private index/object store and TMPDIR
`/private/tmp/sifr-b33.W1oIqs/tmp`. External apparatus/evidence lives in
`/private/tmp/sifr-b33.W1oIqs/evidence` (E). No shared targets or alternates.
The parent dirty phase ledger's top B33/B34 execution registration and direct
B33 onboarding authorize this scope; lower historic headers do not supersede it.
All parent/predecessor files, indexes, refs and raw remain read-only.

B32 native worker closed. Terminal SHA256
`cbcf4bd5ed995dc3453e46294c12ddb620bf7fec1a188b9378081e6c6a25f74a`, scoped
document SHA256 `f4f2116b378ae9506596e2fe7c275fa438ef1d3ef17a3a3ef3d4d17a93161d00`,
remote record `9aad3dd68a77671193523ae503e4251215a13afc`, branch
`codex/item12k-b32-output-custody`, issue3776 comment5579599977. Its stop7
duplicate dyld rows have equal recorded metadata but unrecorded object identity.
Historical identity stays unknown and historical coverage stays INCONCLUSIVE.

Implement only the owned external module resolver/observer integration and
offline coverage. Retain the inherited 231 address/symbol/handover/output cases,
all existing guards, canonical exclusive output files and outer consumer.
B34 alone owns custody rejection evidence. No B34 implementation, live debugger
probe, inferior launch/attach/continue, build, Cargo/Clippy/Sifr gate, CV/counter
or acceptance acquisition. This merge is a static prerequisite only. B34's
contract and integrated negatives must also complete before a later combined
proof can be proposed; that proof remains unallocated. B24 causal/full budget,
SQL/B27 joint delivery/approved65, and the full phase remain open.

## Registered owned files and named commands (before execution)

Owned apparatus under E: `coverage_module.py` (resolver and API snapshot),
`coverage_module_fakes.py` (explicit synthetic API objects),
`coverage_module_tests.py` (same resolver entry points), `coverage_symbols.py`
(inherited suite and encompassing entry), `lldb_coverage.py` (integration),
`coverage_address.py`, `coverage_output.py`, `coverage_output_tests.py`,
`run_coverage.py`, `coverage.lldb`, `identity-contract.json`, and
`module-inventory.json`. The outer launch apparatus is retained for static
integration review only; no executable or live command is registered for B33.
`record_b33.py` authenticates/hashes/serializes evidence and runs only the named
documentation checks; it never invokes behavioral tests or debugger execution.
Source references: installed LLDB Python wrapper, pinned public `SBModule.cpp`,
`SBAddress.cpp`, `SBSection.cpp`, `SBSymbol.cpp` under E.

All commands run from the owned clone. Complete implementation precedes ONE
encompassing offline suite, including Python3.9 syntax parsing and the actual
B32 duplicate representation with unknown identity, labeled synthetic aliases,
distinct/unknown/conflicting module/header/section/symbol/roundtrip negatives:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b33.W1oIqs/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b33.W1oIqs/evidence/coverage_symbols.py --self-test
```

Named documentation checks, with that same environment:

```bash
git diff --check 491ba4ede1609ce476831015dc209a9064cd8ffc HEAD
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b33.W1oIqs/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 scripts/check_hir_maintainability_guardrails.py
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b33.W1oIqs/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 scripts/check_file_size_guardrails.py
```

External maintained source line counts must also stay below900. Only Markdown
changes enter Git. No create-pr or merge gate. One exact-base/candidate Opus
review plus at most one remediation, with all frozen external paths/digests
and existing evidence supplied. No reviewer writes, new requirements or broad
test repetition. A new mechanism defect on second review is later work and
stops this item. Successful review lives outside the reviewed tree keyed by
candidate SHA. Merge this scoped contract, update the phase record and stop.

## Supported API contract and implemented resolution

Installed LLDB is `lldb-2100.0.17.203`, Apple Swift6.3.3. Bounded inspection read
the installed Python wrapper, SHA256
`08d7c4c689459429ae660a2a3e9f8a9b39788fcc3754e9db3316efd3b3fab676`, including
the final effective SBModule equality definitions at10034 and10040, SBSection
equality and SBAddress section/module methods. These dispatch to native `_lldb`
operators. No LLDB target or process was created for capability inspection.

Pinned public implementation at Swift LLVM commit
`82cdc19fa54d566969527b56f587ea8ea30bef51` supplies these semantics:

- [SBModule.cpp](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/API/SBModule.cpp#L197):
  valid module equality compares the underlying Module pointer; copy wrappers
  retain that pointer. FindSymbols preserves the owning module in each context.
- [SBAddress.cpp](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/API/SBAddress.cpp#L203):
  GetModule returns the module owning the section-relative address.
- [SBSection.cpp](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/API/SBSection.cpp#L229)
  and [SBSymbol.cpp](https://github.com/swiftlang/llvm-project/blob/82cdc19fa54d566969527b56f587ea8ea30bef51/lldb/source/API/SBSymbol.cpp#L82):
  equality compares the section shared pointer and symbol pointer respectively.

The installed API and these pinned semantics support the static contract;
exact Apple binary/source equivalence and historical duplicate-object equality
are not asserted. Invalid, non-boolean, asymmetric or contradictory equality
returns UNKNOWN and rejects, with no metadata fallback.

The shared Resolver captures stopped process/stop ID/epoch before and after
each operation. It retains all matching path rows and a native equality matrix.
All mapped representations must prove one instance. Only then does its unique
agreed header load resolve to the authoritative module handle; no enumerated
first row is selected. Native equality ties every alias to that handle. It
checks module/UUID/triple/header metadata, section ownership/tree/identity,
header file/load round trips and contradictory unloaded aliases.

Each alias and the mapped handle must independently return one exact typed
symbol context owned by that module. Existing static identities remain checked.
Every start/end address proves its module, TEXT/text section, file/load mapping
and section identity through current load and alias-specific file round trips.
Symbol pointers and rows must agree across aliases. Current entry load resolves
back to the same symbol and executable memory before existing bind predicates
approve. Missing, distinct, contradictory or unknown identities reject.

The observer uses this Resolver for current-module selection, the full thirteen
static identities, initial-PC module checking, arming and repeated site checks.
Strong native references assign observation-local instance tokens, carried in
the existing binding comparison; same-metadata replacements therefore remain
detectable across stops. Tokens never substitute for fresh current mapping or
native equality, and are never attributed retrospectively to B32. Successful
and rejected resolution receipts retain the common stop, rows, matrix and error.
Output helpers/tests and address helper are byte-identical to B32; outer
custody code has only its owned evidence path relocated. No B34 change.

## Validation and frozen external candidate

Input authentication PASS47 paths, including every B32 terminal-listed source,
raw receipt, binary and scoped record, plus the installed API. First invocation
of the named suite found two in-scope omissions (297/299): alias symbol file
round trips used the canonical handle and missed conflicting alias lookups.
The one correction uses each alias's own ResolveFileAddress for both endpoints.
The exact same named suite then passed299/299: inherited231 plus68 new cases;
Python3.9 syntax compatibility included. Total offline invocations2, additional
suite families0. The user clarified that the single encompassing suite limits
scope, not in-scope correction iterations. No test, source or live-proof quota
was reset. This is offline evidence only.

The first failing receipt is preserved at E/`offline-initial-failed.json`, digest
`72a7941ba55827ea3c32bf7d866f964b625f38214f7e7cd2f65ebebc76d4be5f`, with its
original resolver at E/`failed-source/coverage_module.py`, digest
`a6d731daf6d23456f9a2a9fb82c2c8346d46e17c410e21e82df2eecdda1b4572`.
All other tested sources were unchanged. Passing receipt E/`offline-result.json`
has digest`81120dfd6713423cc4955033e1d0884a0fcebab054c4a87d81d5c35894f6ed1c`.
Frozen manifest E/`frozen-manifest.json`, digest
`81597f166178476699cb73a6ca7b13c980624125a5603c89c6b7cafc5e185a31`, binds every
owned helper, reference, immutable input and passing receipt. Exact SHA evidence
may reuse this pass while those inputs remain unchanged.

| External candidate under E | SHA256 |
| --- | --- |
| coverage_module.py | abac42555a71b17273358980da83ad4f517bc15a57f379fcca7287caa12540e7 |
| coverage_module_fakes.py | 6365e9a9ced06301e7497341f147df11cb02b6aa10546c6f10874637d4f77d48 |
| coverage_module_tests.py | 49226b46873698a634ba8f90ee9bb5c14958420b8eadc5f36316138c25ee8699 |
| coverage_symbols.py | f881848d13904118a913f4d7dc88eac4790f8ed9992d890a3491d184e9a0368b |
| lldb_coverage.py | ec53861431600b96b4e6be4a16bdaa4ee479152d53707b3a8d11980c13ddfe86 |

Review covers only this Markdown delta and frozen external static contract.
Final review/check/merge/terminal identities are recorded after approval outside
the reviewed tree. No create-pr/merge gate, compiler/fixture/lock/workflow edit,
live debugger session, inferior launch/attach/continue or new proof occurred.

## Exact-SHA review, merge and terminal handoff

[PR3804](https://github.com/sifr-lang/sifr/pull/3804) merged at
2026-09-08T05:36:46Z, mergeSHA `dae0d285bf8cc34924ef4d47a349c28591eb50f0`.
Exact candidate `3446f57dc2d21f584d30188f91ede95540af67f7`, base
`491ba4ede1609ce476831015dc209a9064cd8ffc`; both merge and candidate trees are
`5a409daae3262532cce5b5bd9f73176dd0a1aac0`. No intervening base change.

The ONE initial Opus review returned SATISFIED with no blocking findings.
It authenticated every frozen helper/reference digest and inspected the complete
integration, installed API, preserved failure and passing evidence. No tests or
live probes were rerun by the reviewer. Response
E/`review-3446f57dc2d21f584d30188f91ede95540af67f7.md`, SHA256
`432ab8c8586843136b21f7c9a77b74d8075faeeb96998ed996d5c9fdc9a0b0b8`, is
[published outside the reviewed tree](https://github.com/sifr-lang/sifr/pull/3804#issuecomment-5579842289).
No remediation/provider retry; no final review was committed into its candidate.

E/`checks-3446f57dc2d21f584d30188f91ede95540af67f7.json`, SHA256
`c90fc41833a398d501c963ec78b61daa27a4a6c4e79ad43f2b54f0c8163b7d05`, records
exact candidate diff/HIR/file-size PASS3762 repository files. All external
maintained helpers are below900 lines, largest588; the resolver is261 lines.
The passing suite evidence and frozen inputs cover the reviewed SHA unchanged.
This post-merge update changes records only and receives no additional external
review, behavioral test or gate.

Non-blocking review follow-ups, sole issue3776 owner, deferred without new code:

- The SBModule source link line anchor was corrected to197 in this record update.
- Cosmetic dead imports in the observer may be removed in later maintenance;
  the reviewed helper is retained byte-for-byte here.
- Persistence failure can mask a rejection while saving evidence, analogous to
  the inherited event-save pattern. A later evidence-persistence item may retain
  both errors; no expanded mechanism work or new live allowance is inferred.
- B33 intentionally has no prelaunch manifest/executable proof registration.
  A future combined-proof owner must supply its own authenticated integrated
  apparatus after both static contracts and negatives are ready. This is not
  a B33 omission or permission to implement B34 here.

The review shell left an orphaned `sleep2400` watchdog holding its command
channel open after the reviewer exited. PID34385/group33942 was authenticated
read-only through its exact B33 cwd and command, then terminated with TERM;
subsequent ps returned no row. The completed response was collected through
command completion, with no response-file polling, new request or review quota
reset. This was review-harness cleanup only; no inferior existed to clean up.
All first-failure evidence and parent/predecessor sources remain preserved.

Implementation counts: offline invocations2, named suite families1, initial Opus1,
remediation0, provider retries0, PR1, merge1, create-pr/merge gates0, compiler
builds/tests0, live debugger contexts/sessions/proofs0, inferior launches/
attaches/continues0, CV/counter/acceptance acquisitions0. Final immutable
E/`terminal.json` binds the phase-record commit, all external hashes, review,
named checks, remote identity and process absence. Blocker: none.

Main's PR-only repository rule rejected the direct post-merge record push.
The owned branch push succeeded; the same record update therefore uses a
separate Markdown-only PR, with no new review or gate. Its PR/merge/record
identities and aggregate PR counts are bound in the external terminal receipt.

Next action: STOP. B33 static/offline prerequisite is complete and this worker
retires; B34 and any combined proof remain separate future work. This record
does not close output custody proof, B24, SQL/B27 or the full emitted-Rust phase.
