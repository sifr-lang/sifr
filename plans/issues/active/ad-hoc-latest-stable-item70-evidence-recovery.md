# Item 70: original-evidence recovery and prospective disposition

State: bounded recovery assessment complete and merged, 2026-09-08, via
[PR #3810](https://github.com/sifr-lang/sifr/pull/3810). This is the authorized
current disposition, not qualification.
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
may still exist outside the completed bounded search. Item 70 closes the
recovery assessment and disposition only; the original custody gaps remain.

## Ownership and evidence identities

Inspection base: `7339aed05db02b3de95b7930e5e1291edecd07fe`.
Resumed owned branch: `codex/latest-stable-item70-recovery-resume`; independent
clone: `/private/tmp/sifr-item70-resume.aEgoQY/codebase`.
New evidence root: `/private/tmp/sifr-item70-resume.aEgoQY`.
Prior candidate `59f726c2594a626c00f30d2933c54378c4efd065` and evidence root
`/private/tmp/sifr-item70.L0HK3e` remain read-only. Its light-search evidence is
reused; this continuation does not reset the Item 70 review allowances.
The parent cb34/Kafka worktree, dirty orchestration ledger, predecessor
worktrees, Git stores, targets and indexes are read-only. The prior clone was
copied without shared hardlinks, current main fetched into the new clone and
the same three-document proposal adopted there; no foreign store was written.

Historical version `0.1.0`, producer
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
  `store-inventory.json`; alternate-store references are recorded. The later
  completed object scan below uses this exact frozen 59-store inventory.
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
  reference. The selected Git blob scan also inspected LFS pointer signatures
  and found none. No LFS server request was therefore needed. LFS pointers
  outside the scanned stores and small-blob bound remain excluded.

Excluded: recursive build-target traversal, nested/renamed local archives,
arbitrary local files outside the named locations, deleted remote refs,
inaccessible/offline/cloud backups, unrelated or newly created repository
stores outside the frozen inventory, unknown-size result blobs over 1,000,000
bytes, and large blobs with nonmatching artifact sizes. No gc, prune,
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

## Completed bounded Git search

The coordinator relayed E2 B37's explicit proof-window release before this
scan started, then confirmed the measured small-blob read budget. Sequential
read-only `git --git-dir=<store> cat-file --batch-all-objects
--batch-check='%(objectname) %(objecttype) %(objectsize)'` completed across all
59 stores in 41.25 seconds. All metadata and reflog commands exited zero.
Global object-ID deduplication, including alternate stores, yielded 225,250
unique objects. This enumerates extant reachable, reflog-only and unreachable
objects independently of filenames; it does not resurrect pruned objects.
Per-store counts, reflog entry counts and reflog-list hashes are retained.

There were 130 exact-size candidates totaling 119,521 bytes, all small. These
were read and hashed first. The full selected set contained 99,567 distinct
blobs no larger than 1,000,000 bytes, totaling exactly 2,269,583,001 bytes.
The coordinator authorized that exact hard cap before the sequential reads.
All selected reads completed in 5.13 seconds; no blob over the bound was read.
Exact-size candidates and blobs with a leading JSON object/array signature
were SHA-256 hashed: 6,058 candidates, 110,738,030 bytes. All selected blobs
were inspected for LFS pointer signatures; zero were found.

The only authenticated match was the already-retained Rust result:
Git blob `489a78026e68dfa8f3c9595e42c1e4108ba94125`, 6,436 bytes,
SHA-256 `95176b5937b4ed0e1c9843ef6c3896969f6336431bc8a0d08350cc2db9b9555e`,
read from `/Users/yaseralnajjar/work/sifr/codebase/.git` and preserved unchanged
in the new evidence root's `recovered/` directory. No missing original was
newly recovered. All 20 qualification payloads and all four separately listed
result gaps remain INCOMPLETE/UNRECOVERED within this coverage.

The scan ended and host capacity was released immediately. No scan process
or host reservation remains. Raw evidence is outside the reviewed Git tree:

| Evidence in new root | SHA-256 |
| --- | --- |
| `metadata.json` | `d62f8cd13d61cc73cdba32b5ab352b65c5532d3a4706d2fcfba762288748d9b9` |
| `selected-objects.json` | `b2c011e5d65362acfcbdcbead476c5c801602a02014a635ba6a4de3f3aea7bf1` |
| `selected-hashes.json` | `88367b803bcb62235d94224258a315b18d17da4aba802577121af3119fd87011` |

This completes the authorized bounded search and supports the accepted
disposition. It does not establish exhaustive absence from every backup.

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

Only three Markdown files changed. Named checks are exact retained identities,
recovered-candidate SHA-256/size checks, scan coverage/count consistency,
`git diff --check`, `python3 scripts/check_file_size_guardrails.py`, and local
documentation paths/links. The checks authenticate five retained records,
all 20 original index rows and their 533,998,429-byte total, prior evidence
identities, the six-row matrix and all selected-read evidence identities.
No custody suite, Cargo, native execution, Sifr gate or package update is
authorized or needed for this documentation-only assessment.

Reviewed candidate: `d9d23149f8a59579f5e399bbb2239912198b524e`.
Merge: `df093dab01b27484b684060443d1129d9898df04`.
All named checks passed on that exact candidate. The first and only Opus
review returned SATISFIED with no blocking findings; no remediation review
or Sifr gate ran. The
[validation/review receipt](https://github.com/sifr-lang/sifr/pull/3810#issuecomment-5581186433)
and [full review](https://github.com/sifr-lang/sifr/pull/3810#issuecomment-5581186774)
are published outside the approved tree. Raw response SHA-256:
`922ca904484ed50badb0330ab4fb21dccb8048694753ba99af37f0fb6288e682`.

Item 70 assessment blocker: none. Items 59/64 remain unqualified; historical
custody is still INCOMPLETE/UNRECOVERED. Their prospective replacement depends
on separately owned compiler/policy delivery and later 70-F1 durable custody
and qualification work. These are later prerequisites, not satisfied by this
assessment. Review follow-ups are settled by this current-status update or
retained under issue #3775/70-F1, including the impermanence of host-local raw
scan evidence. Exact next action: finish this record-only update and return
the handoff, then stop. Do not start 70-F1 or another item. No new review or
gate is required for this record-only update.
