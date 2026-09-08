# 12K-B34: truthful process-custody rejection evidence

Date: 2026-09-08. Exactly B32-F2, performance / issue3776.
State: STATIC/OFFLINE IMPLEMENTATION COMPLETE; awaiting exact-SHA review/merge.

## Scope, ownership and registration

Sole worker owns independent fresh-main clone `/private/tmp/sifr-b34.nVIi3L/sifr`,
branch `codex/item12k-b34-custody-evidence`, base
`d1cfc49a12938affb921afcb806f9f9e9ecf67c7`, private index/object store, sibling
`tmp` and `evidence` (E). Parent dirty ledgers are intentional and read-only;
their top B33/B34 registration and direct onboarding authorize only this item.
No alternates, shared target or predecessor mutation. B33 terminal authenticated
SHA256 `218d0b5cb0b7f4c012117a6b37c5007b6c66529642506f220b2c618055e5ae5b`;
PR3804 and record3805 are merged, James is natively closed.

Preserve B33's complete module resolver/observer and B32's exclusive output
route in owned copies. Implement complete process observation, immutable initial
authentication, truthful rejection producer/consumer and cleanup predicates.
Capture first and conflicting rows, comparison fields, timestamps, group and
ancestry context. No historical reconstruction: B32's missing conflicting row,
cause and cross-process failure ordering remain UNKNOWN, proof INCONCLUSIVE,
output acceptance UNREACHED. No lifecycle classification authorizes replacing
an initial signal identity or bypassing rejection.

Owned apparatus paths under E, registered before creation/execution:
`coverage_custody.py`, `coverage_custody_tests.py`, `coverage_symbols.py`,
`run_coverage.py`, `lldb_coverage.py`, `coverage_output.py`,
`coverage_output_tests.py`, `coverage_address.py`, `coverage_module.py`,
`coverage_module_fakes.py`, `coverage_module_tests.py`, `coverage.lldb`,
`identity-contract.json`, `module-inventory.json`, `record_b34.py`.
`inputs/` holds authenticated raw B29/B30/B31/B32 replays and installed ps(1),
execve(2), killpg(2) references. `record_b34.py` copies/hashes/relocates inputs,
freezes evidence and records named checks only; it cannot launch the apparatus.
No binary, live invocation or prelaunch permission is registered for this item.

Complete implementation and tests precede the sole encompassing suite family:

```bash
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b34.nVIi3L/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 /private/tmp/sifr-b34.nVIi3L/evidence/coverage_symbols.py --self-test
```

Retain all299 B33 tests and add row capture, lifecycle/unknown/PID-replacement,
ancestry/group rejection, observer acknowledgement and integrated module/output
negatives through actual shared predicates. Include Python3.9 syntax parsing.
Only this suite may be rerun for an in-scope fix, retaining failed receipts and
source hashes; no new suite or review/gate quota reset.

Named documentation checks from the owned clone, in the same environment:

```bash
git diff --check d1cfc49a12938affb921afcb806f9f9e9ecf67c7 HEAD
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b34.nVIi3L/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 scripts/check_hir_maintainability_guardrails.py
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b34.nVIi3L/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 scripts/check_file_size_guardrails.py
```

External maintained sources also stay below900 lines. One exact-SHA Opus review
and at most one remediation, readonly including frozen external inputs; no
reviewer tests/new requirements. A second-review new mechanism defect is later
work and stops the item. Only Markdown enters Git, so zero create-pr/merge gates.
After approval, merge this contract, update phase records (record-only PR if
required), and STOP. No compiler/fixture/lock/workflow edit, Cargo/Clippy/build,
live process/proof/launch/attach/continue, CV/counters, acceptance acquisition,
whole-phase review or next-item implementation. A later one-proof proposal is
documentation only, pending adjudication and allocation after both contracts.

## Installed API contract and uncertainty

Bounded read-only inspection used installed `/usr/share/man/man1/ps.1` and
the current Xcode MacOSX SDK's `usr/share/man/man2/{execve,killpg}.2`; copies are
under E/inputs. No process table was queried to establish this contract.

- ps(1) lines290-295 specify lstart as strftime `%c`, not an opaque process
  identity. Lines317-341 define I/R/S/T/U/Z; Z is an observed dead/zombie state.
  Lines400-420 discuss command display, exiting/defunct representations and
  its limitations. The contract does not equate a comm string with an
  executable object or infer that a particular historical change was a zombie.
- execve(2) lines150-156 say exec retains PID, parent PID and process group.
  Therefore equal IDs with a changed command cannot establish PID reuse or
  exec. This apparatus has no native process-generation identity capability.
- killpg(2) signals the current members of a process group. A prior group ID
  alone cannot authorize cleanup; every current member and its observed
  ancestor chain must still match the original authenticated tuples.

Reference SHA256s: ps.1
`fbddf023ce4960c7e6eadebcfb7f0badf0ab6c3f59b2b97c2807d3f67a85142d`, execve.2
`551d9ae27d03112b9db14479fe283e16f9bcd722868ce43e8861adcc6fe2d770`, killpg.2
`fec81b2fbda8ffab3f17575a90623c8f98b52f52caf6e8a2fab9748b769791da`.

The existing ps identity tuple is retained, with parent PID additionally
checked. Start/command changes reject; no classification repairs the identity.
Normal run-state changes only update observations. Unknown states reject, and
a zombie cannot receive a live inferior acknowledgement. A changed start field
is labeled replacement-suspected, with cause UNKNOWN. Missing historical rows
remain UNKNOWN_MISSING_ROW. No stable native identity, subsecond uniqueness,
atomic process-table snapshot or atomic check-and-signal guarantee is asserted.
This is a supported evidence contract with conservative rejection, not a new
process identity primitive or retrospective B32 causal proof.

## Implemented producer, custody and consumers

`coverage_custody.py` owns the shared contract. Its only process-table producer
uses `/bin/ps -ww -axo pid=,ppid=,pgid=,lstart=,stat=,comm=` with C locale, UTC
and the inherited2s timeout. Each capture brackets outer monotonic and wall
times, preserves exact stdout/stderr/status/error, and parses without dropping
malformed rows. Duplicate PIDs, malformed or mismatched raw/parsed rows,
unavailable capture, non-finite/reversed/stale timestamps reject. Tests inject
the process reader and clocks; they never invoke ps or create a process.

Initial authentication is a deep-copied row plus its original full snapshot,
observed ancestry and index. PID/group/start/command/parent fields are compared
to that immutable initial row on later observations, including known PIDs after
they leave the original ancestry or group. Current owned-group members are also
checked for foreign membership. Each rejection preserves the first row and
conflicting row, all compared values including state, both capture intervals,
observed ancestry, group/target sets, lifecycle classification and custody state.
The first conflict per reason/PID is retained; later polls cannot replace it.
For an unknown/foreign PID or unavailable row, missing initial evidence is null,
never synthesized. Complete observations retain non-rejecting lifecycle changes
and disappearance without pretending disappearance proves exit.

Custody rejection is sticky and tainted PIDs cannot be readopted, even when a
later row returns to its initial scalar values. New descendants/groups are not
admitted after rejection. Cleanup consumes a fresh validated observation and
requires every member AND every ancestor up to the owned launcher to match an
untainted initial tuple; parent/current groups, foreign members, changed
identities, unknown lifecycle and missing ancestry cannot grant signal authority.
Initial signal identities are never overwritten to make cleanup succeed.
If cleanup cannot authenticate or absence cannot be observed, release remains
unproved and the later attempt must stop INCONCLUSIVE under the existing bounds.

`run_coverage.py` observes through this authority, writes the complete custody
document before generic stop notification, issues the shared acknowledgement,
and uses the same fresh-capture cleanup predicate. `lldb_coverage.py` validates
schema/run/initial/current/raw/clock/ancestry evidence before proceeding; its B33
Resolver, typed symbols, handover and entry guards remain intact. The exclusive
output consumer calls `validate_completion` before accepting output: no rejected
or tainted state, wrong owner/run, lost initial identity or missing sole target
may finalize. Existing output, protocol, clock and release checks still apply.
Only owned path relocation and the new custody context adapt inherited tests;
all299 inherited cases remain in the encompassing entry. Every invoked replay
input is owned and frozen, including B31's failed observer result.

## Offline evidence and frozen external candidate

Initial suite PASS344/344 is retained in E/`offline-initial-pass.json`. A
pre-review inspection then tightened cleanup to compare ancestors as well as
members and added five direct changed-ancestor cleanup negatives. Initial helper
sources are retained under E/`initial-pass-source`. The same registered suite
then passed349/349: all299 inherited plus50 custody/integration cases. No failed
test invocation, additional suite family, live operation or quota reset.
Python3.9 syntax parsing is included in the suite.

Final receipt E/`offline-result.json` SHA256
`55588b1e4d9a338bacb6c6c35d8211d4580f1637fd3df23c5036939733cd4e9a`.
Frozen manifest E/`frozen-manifest.json` SHA256
`f54cda93e7518f6d234fd12afd30b3c74098e4020692a3ebb808e3ec7688127e`
binds all owned helpers, raw inputs, API references, initial/passing receipts
and bookkeeping. The two custody source digests are:

- coverage_custody.py:
  `80613956db9ed848107b3182fb1c61422881d3b921ca61e53022df64e040e8fd` is the
  final349-case helper (including ancestor cleanup checks).
- coverage_custody_tests.py:
  `4857e33537c4fa7f538e22f14dd629bded6a5a6e3786715082489354fd228637`.

Review covers the exact Git candidate AND that external manifest, not Markdown
alone. Final approval is published outside the reviewed tree, keyed by SHA.
Named check receipts and the terminal record bind exact candidate/merge/record
identities after review. No further behavioral validation unless inputs change.

## Combined proof proposal for later adjudication; not execution authority

Both static contracts are prerequisite apparatus, not live qualification. A
later owner may propose ONE integrated attempt using this complete external
module/custody/output observer and outer runner, copied into its own independent
fresh-main clone and evidence root. It must authenticate and freeze those exact
copies, the thirteen-identity contract
`3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d`, immutable
diagnostic binary
`a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392`, and unchanged
two-file formatter workload/config before proposing its concrete invocation.
No binary or prelaunch file has been staged for execution by B34.

The combined offline cases exercise actual B33 native-identity predicates,
synthetic native aliases, real custody producer/observer consumer, exclusive
file actions and final output/protocol consumer together. They reject unknown
module identity, custody conflict after acknowledgement, empty output, missing
handover and missing entry. Retained suites cover the full alias/address/symbol,
handover and output negative matrices; custody adds replacement/start/command,
lost/cyclic ancestry, group escape/foreign membership, unknown state, stale/raw
capture, acknowledgement corruption and changed-ancestor cleanup negatives.
Synthetic completion reports and identity tokens are explicitly offline inputs,
not actual LLDB executions or historical reconstructions.

The proposal retains one debugger session/target, flags134, original300s total
including30s cleanup,180s admission,30s setup,60s target,256events,1GiB evidence
ceiling and4GiB free floor, and separate outer/embedded clock ownership. It
requires one completed mode3-to-mapped-mode0 handover, all8 entries, exit0,
exclusive-file two-file completion and zero-process release. First live failure
is terminal with full raw evidence; no repair/retry, alternate reader, guard
bypass or per-prerequisite proof allowance. No host window is reserved and the
combined proof remains unallocated pending orchestrator adjudication.

Remaining uncertainty: historical B32 module object identity, conflicting process
row/cause and cross-process ordering cannot be recovered; actual combined live
behavior, bounded cleanup and output acceptance are unqualified. B24 causal/full
representative budget, SQL/B27 joint delivery including builtin fix/approved65,
and the full emitted-Rust phase remain open. This item stops after its own
static-contract merge and phase record; no successor code is authorized here.
