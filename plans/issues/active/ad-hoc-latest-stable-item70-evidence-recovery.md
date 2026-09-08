# Item 70: original-evidence recovery and prospective disposition

State: BLOCKED / bounded recovery incomplete, 2026-09-08. This is an
unreviewed assessment and authorized current disposition, not qualification.
Owner: release/distribution, [issue #3775](https://github.com/sifr-lang/sifr/issues/3775).
Source of scope: Item 70 in the coordinator's read-only convergence ledger,
relaying the user's authorization to take recommended decisions on their behalf.

The historical qualification evidence is **INCOMPLETE/UNRECOVERED**. Adopt a
prospective replacement qualification under a NEW source/run/evidence identity
once its separately owned prerequisites are delivered. Original report, index,
sign-off and waiver bytes remain unchanged. This disposition does not turn
the recorded historical pass into authenticated complete custody, close Item
59, approve a publication run, renew a waiver, or authorize qualification now.
It replaces the need to ask again for the same Git pointer. Genuine originals
may still be recovered; the expanded object-store search remains unperformed.

## Ownership and evidence identities

Inspection base: `7339aed05db02b3de95b7930e5e1291edecd07fe`.
Owned branch: `codex/latest-stable-item70`; independent clone:
`/private/tmp/sifr-item70.L0HK3e/codebase`.
Evidence root: `/private/tmp/sifr-item70.L0HK3e`.
The parent cb34/Kafka worktree, dirty orchestration ledger, predecessor
worktrees, Git stores, targets and indexes are read-only. Only main was cloned;
no remote ref was fetched into another owner’s store.

Historical version `0.1.0), producer
`c9d611fb7c7c5d05421d784d53a2b78c1a7dcae9`, workflow run
[30416219284](https://github.com/sifr-lang/sifr/actions/runs/30416219284),
attempt 1. Reuse the authenticated Item 64 run metadata and Item 59 HTTP 410
evidence; no expired download was retried.

| Retained bytes | Bytes | SHA-256 |
| --- | --- | --- |
| Original release report, commit `74c5dd02f1ca692c0fb1f9c8b50004827028cdfb`, blob `93b4ea4ddf46282427b71e9ac7372d54adf0dfb3` | 14177 | `e5200229dfdacb2503190d4c3784cdfb3085088f7ad687e49659f54f3a11de98` |
| Current step-renamed release report | 14173 | `9414915dcc474b9be2b27ab484bd8e76f75faf36b25a8c1461d72c8bf62cdc4d` |
| Original qualification index | 9543 | `503f4fcc0dcf4843e0476fbbd1aaa02994c431fac3e7aebb89fb5565bba04703` |
| Retained Rust release result | 6436 | `95176b5937b4ed0e1c9843ef6c3896969f6336431bc8a0d08350cc2db9b9555e` |
| Standalone documentation summary | 392 | `a7a13122d6e8cc34a4403f70dcd97a10eb4e40d9543e109378c3521e331bfa71` |

Each was read, SHA-256/size checked and copied without transformation into
the owned `preserved/` directory. `light-recovery.json` records individual
source paths and sizes. These are retained records, not newly recovered
missing payloads. Item 64's external inventory is
`/private/tmp/sifr-item64.cXwNXG/git-recovery-evidence.md`, SHA-256
`674f7ed8f47b46140fe6c821a55821b7aee1498155bc5b6df133ca8607a4a10e`,
also retained in [its issue comment](https://github.com/sifr-lang/sifr/issues/3775#issuecomment-5576764253).

## Completed recovery coverage and exclusions

- Enumerated relevant repository roots at depth three under `/private/tmp`,
  `/Users/yaseralnajjar/work/sifr`, and
  `/Users/yaseralnajjar/.codex/worktrees`. Selected Sifr task clones/worktrees,
  the main compiler repository and editor repositories; unrelated ecosystem
  source repositories were excluded. Resolved common Git directories and
  alternates: 91 roots, 59 distinct common directories, 6,060,100,266 pack bytes.
  Multiple worktrees sharing one store are deduplicated in
  `store-inventory.json`; alternate-store references are recorded. This is
  store metadata, not a completed blob search.
- Checked 273 explicit directories: each root's
  `target/verification/areas`, `target/validation_lane_reports`, and
  `plans/releases/candidates/0.1.0`; 148 exist. Inspected direct files only.
  Also checked direct Sifr temporary-root release/qualification JSON and
  ZIP/tar.gz candidates and matching Downloads names. Examined 960 files,
  hashed 820 candidates totaling 12,713,367 bytes. Selection was exact known
  qualification sizes, or JSON at most 1,000,000 bytes. Every match was a
  copy of the already-retained Rust result. No missing original was recovered.
- GitHub release API returned 19 releases and 161 assets. No large asset had
  a required exact payload size. All 72 size-matched small assets were
  65-byte checksum sidecars; every one was downloaded and hashed, none matched.
  These were existing release assets, not the expired Actions uploads.
  Raw metadata and per-asset identities/results are `release-assets.json`
  and `light-recovery.json`. No release was created or changed.
- Original and current root `.gitattributes`, `.gitmodules`, release
  candidate records and the qualification workflow expose no relevant LFS
  reference. No LFS server or complete historical LFS-pointer search ran.
  The queued blob search must inspect matching-size LFS pointer contents and
  follow a relevant pointer only into the child's owned evidence root.

Excluded: recursive build-target traversal, nested/renamed local archives,
arbitrary local files outside the named locations, deleted remote refs,
inaccessible/offline/cloud backups, unrelated repository stores, and the
queued reachable/reflog-only/unreachable Git blob search. No gc, prune,
reflog expiry, cleanup, build, reconstruction, synthetic receipt or digest
rebind was performed. The search does not establish that every backup is absent.

## Exact unrecovered bytes

The original [qualification index](../../releases/candidates/0.1.0/qualification-artifact-index.json)
binds the following 20 missing payloads, totaling 533,998,429 bytes. All remain
missing. Four target uploads plus editor and assemble own these payloads; the
seventh upload owns the separately retained index. The expired IDs are
`8710312634`, `8710327423`, `8710332418`, `8710519427`,
`8710534546`, `8710536142`, and `8710544640`.

| File | Bytes | Required SHA-256 |
| --- | --- | --- |
| `sifr-0.1.0-aarch64-apple-darwin.tar.gz` | 71791441 | `1de0e0724764feb150d0669f0cf39fe3cf3c39a75cfef22a4c3275fc18a720e1` |
| `sifr-0.1.0-aarch64-unknown-linux-gnu.tar.gz` | 72628536 | `cf37c395bb051243189cf5374bb93ce6bb11693b99b63a3fce09684ade495de6` |
| `sifr-0.1.0-x86_64-apple-darwin.tar.gz` | 72476442 | `94460bd10715bc9a4e13aa415217d1c42f5deb1f4d702f91d723b5a4a38182b6` |
| `sifr-0.1.0-x86_64-unknown-linux-gnu.tar.gz` | 73123139 | `3e5d7c18cf82cc4566488d3d04398cbd41420249d40b9f56ad121b9249a875aa` |
| `sifr-0.1.0-aarch64-apple-darwin.tar.gz.sha256` | 65 | `2344e27c6b72e38abd58c43cbcd7f606c55b552080f12322ca388f2c59b65e02` |
| `sifr-0.1.0-aarch64-unknown-linux-gnu.tar.gz.sha256` | 65 | `54a755ab8408bbe312e8c5117a709e7c29ac38ca54e8d21ac7503e6a242d1f38` |
| `sifr-0.1.0-x86_64-apple-darwin.tar.gz.sha256` | 65 | `4613c87eacac9a69680cac002a58c3d0338220db52afc98b05c23404aa12c968` |
| `sifr-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256` | 65 | `a93cbb2399a6c6ddd92c0d8634a3cc4e5ea9b2b1714ff9b7562dc81ccf150852` |
| `checksums.txt` | 1344 | `2a074661b76967e398a88c9a838ce853f99b33dd99f61ca5d1e9412dac777e9e` |
| `qualification-editor.json` | 1105 | `73a69c3ce7f8f4fa36664f8d35c0a8383641295b986cf49c5593ea3a58d6fe56` |
| `sifr-installer-0.1.0` | 19571 | `907e7a4fc6ded693bc4eab568def878197f19f3775327e00cfdea6cb71bb6296` |
| `qualification-aarch64-apple-darwin.json` | 880 | `f720e18a7b26876f1240bf8c7fa8dc5cbd3e4918d241f5c66672c55c78f553d3` |
| `qualification-aarch64-unknown-linux-gnu.json` | 898 | `d535d8e2a53430d4d3ea96f7e858655c5a39747b0dd5ab6a30b6f5a78e9eb079` |
| `qualification-x86_64-apple-darwin.json` | 884 | `deb5cbb40321ccc25b0fb5a90ff6f4e5a4a767ca59bfb0f8871f19128ae07ea3` |
| `qualification-x86_64-unknown-linux-gnu.json` | 892 | `d5f72bbebef8096ca89829e43c4f26b8c148fa04407c424d926a98f5b2107c91` |
| `sifr-0.1.0-aarch64-apple-darwin-sysroot.tar.gz` | 60831696 | `f0b20a4eb7f95f63b47fd4fa476bae89be772e6b7c37c43ac2344ccf3e823b37` |
| `sifr-0.1.0-aarch64-unknown-linux-gnu-sysroot.tar.gz` | 60832459 | `ca194ab09144f610cbbbf135977a3f49ce1c8068aa1d818da1292e6b2e62e403` |
| `sifr-0.1.0-x86_64-apple-darwin-sysroot.tar.gz` | 60832069 | `f1a3dfd26338e41ea0e9800d61438dfa3218afc3f22eea709d904953577bbdfb` |
| `sifr-0.1.0-x86_64-unknown-linux-gnu-sysroot.tar.gz` | 60834944 | `d9963d229e6f8eab95841b636fe6e3b2048449bda211c4c9460561d4f0948fd2` |
| `sifr-vscode-0.2.0.vsix` | 621869 | `5a6d706b3faca91143f4a6ac3f29dacfee5a4a4d635a726db5a9068ac148eb21` |

The local release-profile result files are a separate missing evidence set;
their byte sizes were not recorded and must not be invented:

| Result file | Required SHA-256 |
| --- | --- |
| `developer-tooling-release-results.json` | `5dd4ffdc4f29aa90e875252e6d2771b2b8fa81acc5b9c194bfb726cd8218da1f` |
| `distribution-release-release-results.json` | `0438b88b0b8302680f49d26a249869a69530a30ac8dc989329c6558af3df2ad4` |
| `documentation-release-results.json` | `a7cd2e048e41ca8c665144992aed5d882385de46027e8d0f6ad0fc3eab4a50a4` |
| Separate `documentation-stable-qualification-results.json` | `998179ecd3eeb8879d77c7de0d04d6d1a1918942c1b05be7f0e123d4ee5aaa2e` |

The original producer directory
`/private/tmp/sifr-phase40-candidate-rebuild.s49VOp/release-output-pass8`
remains absent under the reused Item 59 evidence. A result digest, report
summary, successful run metadata, matching filename or source commit cannot
replace any of these bytes. A recovered subset remains partial recovery.

## Queued bounded Git search

The shared-host coordinator has not cleared bulk scanning; outgoing thread
coordination is unavailable. The parent explicitly requires a hold. No process
is waiting or holding a host reservation. Estimated work is 1–5 minutes of
sequential metadata enumeration across approximately 6.06 GB of packs,
followed by separately measured selected payload reads.

Upon explicit host clearance, use read-only `git cat-file --batch-all-objects`
with type/size metadata in the 59 inventoried stores, deduplicate object IDs
globally and alternate-store ownership, then hash exact known-size candidates.
This includes extant unreachable and reflog-only objects without relying on
paths or branch reachability. Inventory reflog roots read-only for provenance.
For unknown-size results, first total small-blob candidates (up to 1 MB),
inspect JSON/result/source signatures, and hash qualifying candidates; report
candidate counts/byte estimate before expanding that bounded read budget.
Check small LFS-pointer signatures against the required OIDs/sizes. Do not
use a full filesystem hash walk or inflate every large blob. Preserve any
matching original bytes with store/object provenance and exact hash/size.
If coordination remains unavailable, preserve this blocked handoff; do not
review an incomplete recovery scope as complete.

## Prospective replacement qualification plan

This is an adopted direction and concrete later-work proposal, not a claim
that the new inputs, durable backend or qualification already exist.

1. The compiler/E2 owner must finish its attributable B24 qualification and
   separately authorized B27 joint delivery, including approved Item 65 policy
   source `4c7068b36904e216b02778f5664d2b8fd1159a6a` / [PR #3785](https://github.com/sifr-lang/sifr/pull/3785).
   See the [joint-delivery assessment](ad-hoc-joint-emitted-rust-delivery-assessment.md).
   Its existing one failed gate and one satisfied review remain consumed;
   Item 70 creates no retry, counter reset, or Item-65-first dependency.
2. Later release-evidence implementation must deliver durable custody before
   new qualification is dispatched. Register this as **70-F1**, owner
   release/distribution, a separate later item. Scope: producer capture,
   archive writer/readback, custody manifest and negative tests, plus directly
   owned workflow/schema contracts. Do not implement it in Item 70.
   Acceptance: archive every payload and result below; reject a missing,
   mutated, duplicate, mismatched-source/run or overwritten object; retrieve
   and verify the archive independently of Actions artifact availability.
   Preserve historical records and current 30-day freshness semantics.
3. Freeze a clean NEW source SHA `S` containing the delivered compiler,
   policy and custody changes, exact submodule SHAs, Cargo.lock digest,
   toolchain and profile manifest digests. It must differ from the old producer.
   Bind one exact-SHA Opus review (at most one remediation) and one
   merge-profile gate to that changed implementation candidate, under its
   separately authorized later-item allowance. Reuse an already-passing gate
   on that exact SHA/unchanged inputs; do not run create-pr if merging that
   SHA in the same session. Scope changes require affected evidence, never
   reuse the old producer's release pass for new inputs.
4. After delivery and coordinated host/platform capacity, run the NEW source's
   complete `scripts/run_all_tests.sh --profile release --release-report-out <owned-new-root>/release-profile-report.json`.
   Archive the expanded selected-suite matrix from
   [release.json](../../../verification/profiles/release.json), all produced
   `target/verification/areas/*-results.json`, raw step/case logs and release
   report before temporary-directory cleanup. No filtering or substitution
   of the historical selected-suite matrix is allowed.
5. Execute the maintained
   [release-qualification workflow](../../../.github/workflows/release-qualification.yml)
   at NEW workflow/source identity `S`, version `0.1.0` if still the
   unpublished first stable candidate, rollback `none` only if still first
   GA. Record the actual NEW run ID `R`, attempt `A`, metadata and upload
   IDs; these placeholders are not receipts. Produce a NEW artifact index
   digest and a separate evidence commit `E`. Preserve source/evidence
   separation across both commits and the compared changeset.
6. Revalidate source/submodules, complete matrix, report/result hashes, target
   reports and index against archived bytes. Qualification requires all four
   native target executions and editor evidence below. Structured skips stay
   explicit where declared; no unsupported host becomes supported by this
   document. Refresh the current candidate/support/docs plan under NEW
   identity without rebinding the historical `0.1.0` report or index.
7. Publication is a later action, not authorized here. Its exact run/attempt
   and evidence require mandatory GitHub-recorded approval by
   `yaseralnajjar` under the accepted solo-maintainer policy: self-review
   allowed, admin bypass disabled. Archive that actual approval/sign-off.
   Historical waiver authorization and disposition prose are not approval.
   Freshness checks still apply to live preparation/publication even when an
   archival copy is durable.

The full currently declared host/target matrix is
[supported_platforms.json](../../../verification/areas/runtime_platform/supported_platforms.json).
The release workflow's four native rows are:

| Target | Qualification runner | Required output |
| --- | --- | --- |
| `aarch64-apple-darwin` | `macos-15` | binary, sysroot, sidecar, native target report |
| `x86_64-apple-darwin` | `macos-15-intel` | binary, sysroot, sidecar, native target report |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | binary, sysroot, sidecar, native target report |
| `x86_64-unknown-linux-gnu` | `ubuntu-24.04` | binary, sysroot, sidecar, native target report |
| `x86_64-pc-windows-msvc` | host-limited; no release archive row | declared structured-skip reasons |
| `x86_64-pc-windows-gnu` | host-limited; no release archive row | declared structured-skip reasons |

Additionally capture editor qualification JSON and the built VSIX, assemble
installer and aggregate checksums. This is all 20 payload classes in the
original index, with fresh sizes/hashes, plus the new canonical index itself.
Retain run metadata, each original Actions ZIP and digest, source/submodule/
toolchain/lock/profile inventory, release report, every result JSON it binds,
standalone documentation result/summary, support claims, candidate plan, raw
validation/review logs and, only if later executed, actual protected approval,
sign-off and publication records. Verify cross-links and exact byte counts;
reports/index in Git alone are insufficient custody.

Recommended durable design for 70-F1: a dedicated release-evidence object
store with overwrite/delete protection and no automatic expiry for accepted
qualification bundles, plus an independent retained offline copy. The owner
must provision and verify actual access before execution; no backend is
claimed available here. Keys are content-addressed by SHA-256; a write-once
manifest rooted at `sifr/<S>/<R>/<A>/<index-sha256>/` binds object keys, sizes,
producer identities and verification receipts. Upload before the Actions
30-day expiry and read back every object before accepting the bundle.
The current-status/disposition record is versioned in Git and names the
immutable archival manifest; later freshness or withdrawal changes are
additive status events, not edits to historical bytes. Retention has no
automatic expiry, while authorization to use an old qualification still does.
The existing collector and index validator require exactly 30-day Actions
retention; merely increasing `retention-days` or fabricating an unexpired
index is outside this design.

## Handoff

Only documentation changed. Named checks are exact retained identities and
candidate SHA-256/size checks, `git diff --check`,
`python3 scripts/check_file_size_guardrails.py`, and local documentation
paths/links. No custody suite, Cargo, native execution, Sifr gate, package
update, Opus review, PR or merge was performed for this incomplete recovery.
The review allowance remains unused.

Completed named checks PASS: five retained exact identities and preserved
byte sizes; all 20 index rows and their 533,998,429-byte total; prior Item 64
inventory identity; seven new local documentation links; diff whitespace;
first-party file-size guard (3,762 files, 900-line limit). These checks
authenticate the bounded assessment; they do not cover the queued scan.

Blocker: explicit shared-host clearance is required for the queued expanded
Git recovery scan. External compiler/policy and durable-custody delivery
separately block future qualification, not these documentation checks.
Exact next action: coordinator clears the bounded scan, then resume this same
Item 70 from its preserved clone/evidence; update the assessment with actual
coverage and recovery, run its named checks, and use its single exact-SHA
review before any mergeable completed assessment. Do not start 70-F1 or
another item. The final candidate SHA and check receipts are recorded outside
the reviewed tree in `terminal.md` under the evidence root.
