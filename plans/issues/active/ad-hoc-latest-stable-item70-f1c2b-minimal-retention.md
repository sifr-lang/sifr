# Item 70-F1C2B: minimal supported-release retention

COMPLETE via [PR #3825](https://github.com/sifr-lang/sifr/pull/3825), merged
`4faa48b76044ef742a95bc22eeaac85ad0e404bf` on2026-09-11T21:38:30Z.
Final candidate `a5a5e042239573c42ee7c76945287a383a06a1ab`, tree
`70ecf14f8df00ef221f108b6ac0a50b70fe8e8af`, exact merge base
`81f73f4170da26df1606449d413ff6e7fc082328`.
Eight named offline tests PASS in5.468s on Python3.14.7, diff PASS and
file-size PASS (3767files). [Validation/review identities](https://github.com/sifr-lang/sifr/pull/3825#issuecomment-5640931669)
and [final raw Opus response](https://github.com/sifr-lang/sifr/pull/3825#issuecomment-5640931829)
are outside the reviewed tree. Final review SATISFIED/no blockers, raw SHA256
`cf9588d254f391e76320c4f30bf758bf1bdbda101b2fd8d6b8af5aa7e4533181`.

Counters: initial1, remediation1, Sifr gates0. Initial review of `fcde021980`
also passed, but the worker independently found the actual stable publisher
retains separate sysroots while the nine-asset verifier covers alpha/beta.
The correction restored those required stable assets and received the sole
final review; no historical counter reset. Source/test-source/docs only,
unchanged schema shape/publication primitives and no gate-bearing inputs.

Nonblocking observations remain observations, not new phase requirements:
retired-ZIP metadata digests are recorded claims without retained ZIP-byte
authentication; retained asset digests and their index/producer links remain
fully checked. Existing manual-suite enrollment and minor mapping/old-summary
wording suggestions do not add work. No third review or further source edits.
No qualification, live upload, credential/configuration operation, historical
artifact deletion or heavy capacity reservation occurred. Stop this item;
prospective qualification remains with its existing owners and prerequisites.

The following registration and consumer policy describe the completed scope.

Registration, 2026-09-11. Owner: `/root/minimal_release_retention`, private
clone `/private/tmp/sifr-item70-f1c2b.mWcinY/codebase`, branch
`codex/item70-f1c2b-minimal-retention`, base
`81f73f4170da26df1606449d413ff6e7fc082328`. Evidence is in the sibling directory.
The parent ledger and all other owners' roots/indexes are read-only.

The user's explicit minimal retention instruction (especially historical
outputs) supersedes the agent-added blanket archival, R2 and independent
administrator requirements. This is the same evidence item, not a new archive
system. Implementation scope: `archive_bindings.py`, `archive_offline_selftest.py`
under distribution governance; remove unconsumed `archive_r2_config.py`,
`archive_r2_store.py`, `archive_r2_selftest.py`; this record, the convergence
ledger and supersession notices in existing Item70/F1A/F1B/F1C0 records.
No publication primitive, workflow, compiler fixture, lock or schema shape edit
is planned. The generated schema example follows the existing synthetic builder.

Frozen checks: `python3 -m unittest verification.areas.distribution_release.governance.archive_offline_selftest`,
`git diff --check`, `python3 scripts/check_file_size_guardrails.py`.
Python is maintained 3.14.7. The focused tests cover minimal retained records,
optional known logs, digest/source/run/attempt links and duplicate rejection.
No broad tests, native work or live qualification. Publication primitives are
unchanged, so their conditional suite is not invoked. Actual source/test-source
and documentation categories trigger neither Sifr gate under the current
continuation rule; a schema shape change alone is also not a compiler fixture.
One exact candidate Opus review, at most one remediation; counters start at zero.

## Retention and consumers

Retain supported shipped compiler archives, checksum sidecars and version
installer, plus the shipped Marketplace VSIX. `verify_release_publication_assets.sh`
checks nine alpha/beta distribution files; it is not the stable inventory.
Archives include compiler, sysroot, runtime libraries and vendor inputs consumed
by `generate_version_installer.sh`. The actual stable publication contract
(`stable_prepare.py` and `stable_publish.py`) publishes all20 governed indexed
artifacts, including separate sysroots and qualification reports, plus its
release plan. Retain this existing supported asset set without changing it.
`publish_stable_release.py` authenticates GitHub asset readback against local
bytes and records digests. `stable_prepare.py` pins the VSIX digest and
`stable_publish.py` checks Marketplace readback. Reuse these existing paths;
this item neither publishes nor changes their asset set or approval checks.

Retain compact exact source/submodule/toolchain/lock/profile/run/attempt/checksum
provenance, qualification index and reports, essential structured results,
review/failure/disposition and publication records. Git preserves source and
lockfiles; the retained manifest binds their identities without duplicate
source/toolchain inventory files. Although separate sysroot tarballs originate
in qualification staging and duplicate installation content, the actual stable
publisher includes them: they remain required retained bytes. Additional local
copies beyond that release/evidence set are not mandatory archives.
Actions transport ZIPs, workflow/step/case success logs,
unpacked copies, caches and reproducible intermediates are not blanket retention
requirements. Any voluntarily retained known log still receives full byte and
identity verification. ZIP/source-inventory/toolchain-inventory/lockfile archive
roles are deliberately removed from this contract, not a compatibility lane;
source and lockfile bytes remain in Git. Unique necessary failure evidence
remains protected.

GitHub Release assets are the prospective supported-deliverable destination;
compact records remain in the existing versioned evidence/PR/issue records.
Authenticated complete readback is required for retained release assets, with
no claim of configured immutability or independently administered custody.
No R2 account/bucket/SDK or second copy is a prerequisite. The unused R2 modules
have no production importer or workflow consumer and are removed, not retained
as a fallback. PR #3821 remains historical reviewed/withheld work; this item
does not merge or close it.

## Honest remaining dispositions

Item64's exhausted original-byte recovery and Item59's demand to reconstruct
complete historical custody are retired as closure obligations. The original
20 missing payloads and four results remain **INCOMPLETE/UNRECOVERED**. Existing
original records, failed checks and review/gate counters are preserved; no
retrospective pass, reconstructed original, waiver renewal or history rewrite.
Item59's current source/digest integrity requirements still apply to retained
records; its existing implementation is not weakened by this policy change.

70-F1D retains actual prospective software qualification on delivered compiler
and accepted policy inputs, four native/two structured-skip coverage, new
source/run/attempt identities and essential result/provenance capture. R2 access,
duplicate all-byte uploads, independent-custodian proof and historical recovery
are retired prerequisites. Current compiler/policy blockers remain external.
Human stable-publication approval remains required. Items62/35 must report the
dated convergence cut and these gaps honestly; the phase is not complete.

No historical payloads or task artifacts are deleted here. Physical cleanup
belongs to its assigned owner after exact ownership/reference checks. No
concrete foreign paths have been adjudicated dispensable by this source change.
