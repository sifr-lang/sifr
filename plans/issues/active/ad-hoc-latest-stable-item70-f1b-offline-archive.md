# Item 70-F1B: offline release evidence archive contract

Current supersession, 2026-09-11: [Item70-F1C2B](ad-hoc-latest-stable-item70-f1c2b-minimal-retention.md)
narrows retained bytes to shipped deliverables and essential structured evidence.
Transport ZIPs, separate staging sysroots, duplicate source/toolchain inventories
and every raw success log are no longer mandatory. Full digest/source/run
verification still applies to retained records. The optional synthetic copy
test proves byte verification only; independent custody is not a release
prerequisite. Original review and validation below remain historical evidence.

COMPLETE via [PR #3814](https://github.com/sifr-lang/sifr/pull/3814), merged
`7b2c25707e012c66893de5eb631fef7270384dfa` on 2026-09-08T08:42:37Z.
Final tested/reviewed candidate `d85d2008572090f62e7d6d8d900a8bb42e92e5b9`.

Implements the independently ready offline item registered by the merged
[F1A assessment](ad-hoc-latest-stable-item70-f1a-backend-readiness.md).
Owner: release/distribution, [issue #3775](https://github.com/sifr-lang/sifr/issues/3775).
The [Item 70 disposition](ad-hoc-latest-stable-item70-evidence-recovery.md)
remains INCOMPLETE/UNRECOVERED. Synthetic tests are new test identities and
local bytes; they are neither historical recovery nor actual qualification.

## Format and ownership

The new [archive schema](../../../verification/areas/distribution_release/schemas/release_evidence_archive.schema.json)
is version 1, separate from existing qualification/index/report schemas.
[archive_manifest.py](../../../verification/areas/distribution_release/governance/archive_manifest.py)
owns canonical construction, strict parsing, portable path checks and byte
verification. [archive_bindings.py](../../../verification/areas/distribution_release/governance/archive_bindings.py)
owns the required inventory and cross-links.
[archive_store.py](../../../verification/areas/distribution_release/governance/archive_store.py)
defines the provider interface and sealing/readback/receipt operations.
No production store or filesystem fallback is supplied. The only store
implementation is an in-memory double inside the named offline selftest.

Input inventory has exactly `schema_version`, `identity`, `matrix`, and
`artifacts`. Its identity binds repository, exact source SHA, qualification
workflow path, run ID and attempt, canonical index digest, recursive submodule
SHAs, lockfile digest, the four reported toolchain versions, and canonical
release-profile manifest digest. The schema spells out every field.

Each artifact has `role`, `path`, positive `size_bytes`, `sha256`, and
`producer`. Producer fields repeat the enclosing repository/source/workflow/
run/attempt association and name `job`, `execution` (`local` or `workflow`),
plus the real upload `artifact_id` for transported payloads and ZIPs.
For local capture, the run fields associate custody with the qualification
bundle; `execution: local` does not claim that the workflow ran the local
release profile or review. Source provenance and observed producer identities
must be supplied by the later online integration; this offline format does
not establish GitHub identity or authenticate a Git source tree on its own.

The manifest adds `archive_root`, `inventory_sha256`, and each artifact's
`key`. It sorts artifacts by unique logical role and matrix rows by target;
duplicate roles are rejected even when their hashes agree. The inventory
digest covers the exact normalized inventory without these derived fields.
The manifest uses the existing `canonical_json_bytes` convention and SHA-256
of those exact bytes. Root is `sifr/<S>/<R>/<A>/<index-sha256>/`, with
`objects/sha256/<digest>` and `manifest.json` below it. Distinct logical roles
may share a verified content object while retaining distinct portable input
paths. Path aliases, duplicate/case-folded/prefix conflicts, traversal,
absolute paths and symlinks are rejected before payload reads. Local reads
walk directory descriptors without following any symlink, including root
ancestors; pass a resolved owned root on systems where `/tmp` is a symlink.

## Required inventory

The role registry in `archive_bindings.py` requires all 20 indexed payload
roles, the canonical index, seven original transport ZIPs and upload metadata,
run metadata, workflow logs for all seven jobs, expanded release report and
every bound result, standalone documentation result and summary, source and
toolchain inventories, original lockfile and profile bytes, platform policy,
support claims, candidate plan, release notes, and review evidence/raw log.

The six-row matrix contains exactly four native rows and two Windows
structured-skip rows. Skip reasons must match the archived platform policy;
each skip has its own source-bound result. Every native report must bind its
source, target, runner, archive/sidecar/sysroot bytes and native smoke result.
Transport ZIPs bind API upload IDs and digests and contain exactly their
indexed files (or the index itself), whose sizes and hashes are rechecked.
There is no extraction or download.

Dynamic roles are `result:<report-relative-path>`, `step-log:<step-name>`, and
`case-log:<area>:<suite>:<case-id>`. The bound report defines their exact set;
all selected suites must be represented, including areas beyond the older
four-area minimum. The archived profile's selected areas, including the
existing developer-tooling full-to-editor-release expansion, must exactly
match the report. Missing classes, bytes, results or logs fail closed.
Candidate/report/index/source/toolchain/documentation/review links are checked
against actual retained bytes. These are custody checks, not a replacement
for live qualification or release-plan validation.

Only pre-publication roles are registered in this version. Future protected
approval/sign-off/publication bytes require separately owned additive archive
work after those actions actually occur. No successful approval receipt is
invented to fill their current absence.

## Offline entry points and store semantics

The [CLI](../../../scripts/distribution/archive_release_evidence.py) implements
the two registered commands:

```text
python3 scripts/distribution/archive_release_evidence.py plan --inventory <inventory.json> --payload-root <owned-input> --out <new-manifest.json>
python3 scripts/distribution/archive_release_evidence.py verify-local --manifest <manifest.json> --manifest-sha256 <trusted-digest> --expected-source <S> --expected-run <R> --expected-attempt <A> --payload-root <separate-copy>
```

`plan` verifies complete local inventory and atomically creates a new manifest
file without replacing an existing output. It does not upload/seal a backend.
`verify-local` checks the externally supplied digest, source/run/attempt,
inventory and every local byte. The trusted digest must be pinned outside
the copy being verified; deriving it from an untrusted copy provides no trust.

`ArchiveStore.create` must atomically refuse an existing key and never change
its bytes; `read` must retrieve fresh exact bytes. `seal` verifies input and
each created/reused content object, rechecks all stored bytes and links, then
creates the root manifest exactly once. Existing identical content can be
reused only after full readback; conflicting bytes and resealing fail.
Interrupted object transfer leaves no sealed manifest. Provider-level atomic
create and retention guarantees are prerequisites for a real adapter.

`attest_readback` writes a separate append-only receipt only after every
configured location has independently returned and verified its manifest and
all objects against the same external digest. Copy receipts require at least
two distinct location identities/handles; this does not prove physical or
administrative independence. Receipts explicitly state
`complete-byte-readback-only`, bind exact manifest and inventory digests, and
never modify their own manifest. An interrupted/missing/mutated copy cannot
produce success. If receipt replication itself interrupts, any already
written receipt truthfully attests completed byte reads; the operation still
raises and cannot report complete receipt replication. Event IDs are
caller-owned and create-only, so retry with a reused receipt ID is rejected.

Archive readback intentionally accepts expired custody bytes through the
existing non-live index validation path. Calling the unchanged live validator
with `require_unexpired=True` still rejects the same expired bytes and still
requires exactly 30-day Actions retention. Durable custody is not renewed
authorization, qualification, or publication approval.

## Validation and closure boundary

Only the registered checks run:

```text
python3 -m unittest verification.areas.distribution_release.governance.archive_offline_selftest
git diff --check
python3 scripts/check_file_size_guardrails.py
```

The seven required named tests cover all-class roundtrip and both CLI
operations, schema enrollment, missing/mutated/truncated bytes, removed
classes/results/logs, rehashed cross-link defects and ZIP entries, duplicate
logical roles and producer identities, interrupted/immutable storage, full
fresh copy readback, unchanged live freshness, unsafe paths/symlinks and
untrusted digests. Tests use only inline synthetic data and temporary files.
The new schema has one entry in the existing exact-schema registry and one
explicit independently versioned declaration in the two v2-epoch guards.
Only the two archive modules owning v1 literals are exempted from the v2
source sweep; existing schemas and source retain v2 enforcement. The runner's
schema-count registration includes the new twentieth schema. The named
selftest checks this enrollment and the preserved v2 rejection boundaries.
No broad test command, Cargo, native
execution, provider, workflow, lockfile, tracked fixture file, or historical
evidence changes are part of this item; no Sifr gate is required.

Owned base `579770185d7c14c62e1b60bee7f740b077cac84e`, branch
`codex/latest-stable-item70-f1b`, independent clone
`/private/tmp/sifr-item70-f1b.qRSMYj/codebase`; external evidence root
`/private/tmp/sifr-item70-f1b.qRSMYj`. Parent/predecessor stores remain read-only.
One exact-SHA Opus review, at most one remediation. Record validation and
review outside the reviewed tree, merge, update the phase record without a
second review/gate, and stop. F1C provider/copy proof and F1D integration are
later owners; neither is started by this item.

Final [validation](https://github.com/sifr-lang/sifr/pull/3814#issuecomment-5581936758)
passed seven named tests (6.408 seconds), both diff checks and the file-size
guard (3,767 files). One initial review found only required schema-registry
enrollment regressions; one batch fixed them while retaining strict v2
enforcement for existing owners. The single
[remediation review](https://github.com/sifr-lang/sifr/pull/3814#issuecomment-5581974957)
returned SATISFIED/no blockers, no new mechanism defect. Final raw review
SHA-256 `56a3255816233e49bdcd2a88fdf8a5dbdc6499e12ed4e6e18e8c2d1025aa79ab`.
Zero Sifr gates; no further review for this record-only closure. The initial
reviewer's two unauthorized extra read-only guard executions and outcomes,
plus deferred nonblocking 70-F1B-M1/M2 suggestions, are recorded in the
[phase record](ad-hoc-latest-stable-release-convergence.md). The remediation
review obeyed source-inspection-only limits. Blocker: none; after record
merge, stop.
