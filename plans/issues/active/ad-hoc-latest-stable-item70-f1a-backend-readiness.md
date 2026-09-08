# Item 70-F1A: durable backend capability and implementation readiness

Assessment complete, 2026-09-08. Owner: release/distribution,
[issue #3775](https://github.com/sifr-lang/sifr/issues/3775).
This docs-only unit implements the coordinator's Item 70-F1A registration.
It makes no resource, credential, permission, upload, qualification or
publication changes. No suitable existing durable backend or independent
retained copy is verified within the inspected scope. That finding completes
this assessment; it is an external prerequisite for later online work.

The [Item 70 disposition](ad-hoc-latest-stable-item70-evidence-recovery.md)
remains authoritative: 20 original payloads and four result files remain
INCOMPLETE/UNRECOVERED. There was no historical re-search or digest rebinding.
Any eventual qualification uses NEW source/run/attempt/evidence identities.

## Ownership and inspected identities

Base: `65b8ec51d2e23435cd71716d16f07bb6fbcd5977`.
Owned branch: `codex/latest-stable-item70-f1a`; independent clone:
`/private/tmp/sifr-item70-f1a.SSOhNS/codebase`. Sibling evidence root:
`/private/tmp/sifr-item70-f1a.SSOhNS`. Parent and predecessor stores, indexes,
branches and build targets remained read-only. Only this clone is writable.

Sanitized authenticated metadata, observed 2026-09-08 07:57:51–07:57:56 UTC,
is retained as `metadata.json`, SHA-256
`34090f6bd042dbfca1b7150104a4c7bca2929671b74f8b7fd060c5da43d7dc7d`.
It records exact commands, endpoints, timestamps, exit codes, names and
capabilities; no secret values or credential-file contents. Each collection
has a total count no greater than the returned page size, so no omitted page
is being interpreted as empty. All 14 GitHub metadata queries exited zero.

| Exact API identity under `api.github.com` | Observed result and limit |
| --- | --- |
| `GET /repos/sifr-lang/sifr` | Public repository ID 1157559254; owner organization `sifr-lang`; current caller has admin, maintain, pull, push and triage. These are repository permissions, not cloud or CI-token permissions. |
| `GET /repos/sifr-lang/sifr/actions/secrets` | One name: `SIFR_WEBSITE_ACTIONS_TOKEN`; created/updated 2026-07-28T23:10:52Z. No storage capability inferred from its name. |
| `GET /orgs/sifr-lang/actions/secrets` | Zero organization Actions secrets visible to this successful query. |
| `GET /repos/sifr-lang/sifr/actions/variables` and `GET /orgs/sifr-lang/actions/variables` | Zero variables in each collection. |
| `GET /repos/sifr-lang/sifr/environments` | Four environments: `preview-release` (19710599148), `stable-release` (18874432287), `stable-release-drill` (18893316077), `staging - docs` (16850939394). |
| `GET /repos/sifr-lang/sifr/environments/{environment}/secrets` for all four above | Stable has names `SIFR_WEBSITE_ACTIONS_TOKEN`, `VSCE_PAT`; other three have zero. URL encoding for staging is `staging%20-%20docs`. |
| `GET /repos/sifr-lang/sifr/environments/{environment}/variables` for all four above | All four have zero variables. |
| `GET /repos/sifr-lang/sifr/immutable-releases` | `enabled:false`, `enforced_by_owner:false`; separate `immutable-releases-metadata.json`, SHA-256 `562ff635467ac233f66638c7920381833b00010ecb09a411eb54b1a8184d99cd`. |

The last query was also on 2026-09-08 and exited zero. Secret listing is
metadata access only: neither token validity nor an upload was tested.
No denial occurred in the GitHub metadata queries. The bounded AWS identity
query `AWS_EC2_METADATA_DISABLED=true aws sts get-caller-identity --output json
--no-cli-pager --cli-connect-timeout 5 --cli-read-timeout 5` exited 253:
`Unable to locate credentials`. No AWS account/role identity was established;
no bucket listing followed. Available CLI: `/usr/local/bin/aws`. `gcloud`,
`az`, `wrangler`, `rclone`, and `b2` were absent from PATH. Environment-name
inspection found no `AWS_`, `AZURE_`, `GOOGLE_`, `CLOUDFLARE_`, `R2_`, `B2_`
or `RCLONE_` prefixes. No credential files, other CLI profiles, personal
resource inventories, browser accounts, removable media or unrelated accounts
were inspected. Those exclusions prevent a claim that no storage exists.

Inspected the 186 tracked files in `.github`, `scripts/distribution`,
`verification/areas/distribution_release` and
[distribution_pipeline.md](../../../internal_docs/distribution_pipeline.md),
plus relevant documentation/configuration filenames. No relevant bucket,
cloud role, endpoint, archival lifecycle or independent-copy destination was
found. `CLOUDFLARE_API_TOKEN` occurs only in forbidden/scrubbed credential
lists; it is not an observed deployment credential. Cloud SDK examples and
AWS-LC crypto dependencies do not establish an archive account.

Exact base-tree evidence identities:

| Path | Git blob |
| --- | --- |
| [.github/workflows/release-qualification.yml](../../../.github/workflows/release-qualification.yml) | `94837691f6ec3b74a4b79523b512a45026f78a4d` |
| [collect_qualification_artifacts.py](../../../scripts/distribution/collect_qualification_artifacts.py) | `bb78cf39fd419c4e34df01e03c4a15e16a2f6b45` |
| [fetch_qualification_artifacts.py](../../../scripts/distribution/fetch_qualification_artifacts.py) | `450f001810093df73454f90fd29e254a78feb522` |
| [artifact_index.py](../../../verification/areas/distribution_release/governance/artifact_index.py) | `7f5409040c6f694245fb3bcac0e0c467267dc099` |
| [release_qualification_workflow_contract.sh](../../../verification/areas/distribution_release/cases/release_qualification_workflow_contract.sh) | `8cdd987a28331a207748958fdaba09e1c4158bba` |

The workflow requests only `contents: read` and `actions: read`, uploads with
`overwrite: false` and 30-day retention. The collector/index contract enforces
that lifetime. Actions transport is not the required permanent archive.
GitHub repository access and mutable releases are also insufficient: immutable
releases protect assets after publication, while draft assets remain mutable.
Enabling that setting or publishing an evidence release is outside this item;
it would still need an independent retained copy. See
[GitHub immutable releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases).

## Concrete options for coordinator disposition

These are candidate services, not observed accounts, approved spending, or
permission to provision. Official capability/pricing pages below were checked
on 2026-09-08. Recommendation: dispatch the backend-independent offline item
now; have the coordinator resolve an actual primary destination and separately
administered copy through later online scope. R2 is the simpler indefinite
retention option if an eligible account is subsequently established; S3 is
the alternative if version-level retention is required. Implement only the
selected provider adapter; this does not request dual-provider fallback code.

| Option | Capability and constraints | Concrete access/cost actions for the later owner |
| --- | --- | --- |
| Cloudflare R2 Standard | Indefinite bucket/prefix locks prevent deletion and overwrite and take precedence over lifecycle expiry. Administrators can remove rules; keep bucket-configuration authority separate from the uploader. S3 bucket-versioning and Object Lock APIs are unsupported: use the Cloudflare lock API and content hashes, never invent S3 version IDs. | Establish actual authorized account ID and bucket; verify an enabled indefinite rule covers every archive key with no expiry. Select bucket-scoped Object Read & Write credentials for capture and separate Object Read only credentials for readback; admin configuration credentials remain outside producer jobs. Standard list price: $0.015/GB-month, $4.50/million Class A, $0.36/million Class B; no egress charge. Account-wide monthly free tier: 10 GB-month, 1 million A and 10 million B, subject to other use and billing rounding. |
| Amazon S3 Standard with Object Lock | Requires versioning. Compliance retention protects a version for a fixed period; indefinite legal hold has no expiry but can be removed by a principal with `s3:PutObjectLegalHold`. Neither blocks creation of a new version at the same key. Require conditional create, deny delete/delete-marker operations, record version IDs, and verify exact versions. | Establish account, region, bucket ARN, OIDC trust/role or other approved credential delivery and independent reader. Inspect lock, versioning, lifecycle and IAM policies; preserve indefinite legal hold with no lifecycle expiry and segregate hold-management authority. Reference US East (N. Virginia) example: $0.023/GB-month and $0.005/1,000 PUT/COPY/POST/LIST. Region, GET, transfer, replication and optional KMS charges require an actual quote. No free-tier eligibility is assumed. |

Sources: [R2 locks](https://developers.cloudflare.com/r2/buckets/bucket-locks/),
[R2 compatibility](https://developers.cloudflare.com/r2/api/s3/api/),
[R2 credential permissions](https://developers.cloudflare.com/r2/api/tokens/),
[R2 pricing](https://developers.cloudflare.com/r2/pricing/),
[S3 retention](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lock.html),
[S3 conditional-write enforcement](https://aws.amazon.com/about-aws/whats-new/2024/11/amazon-s3-enforcement-conditional-write-operations-general-purpose-buckets/),
[AWS regional cost example](https://docs.aws.amazon.com/solutions/latest/live-streaming-on-aws-with-amazon-s3/cost-example-1.html),
[S3 pricing and ancillary charges](https://aws.amazon.com/s3/pricing/).

For orientation only, 100 retained GB-month costs $1.50 on R2 before free tier
and $2.30 using that S3 example, plus applicable operations/transfer/tax. These
are calculations, not quotes or total qualification costs. The historical
533,998,429-byte payload total excludes original ZIPs, all results and raw logs;
it is not a capacity estimate for a new complete bundle. Measure every class
before choosing budget; never meet a budget by dropping evidence.

The independent copy must contain all bytes and the manifest, on separately
administered retained storage or an owner-designated offline medium outside
the producer host and its temporary directories. A second prefix/bucket under
the same producer administrator is not independent custody. No medium, backup
account, capacity, retention or usable path was verified here. Cost is either
the actual existing medium's capacity/handling or a separately priced second
storage copy; the primary prices above exclude it. Later owner must name the
custodian, destination identity, access separation, no-expiry policy and
readback procedure before online acceptance. No purchase or new account is
inferred. A same-host temporary copy remains only a test fixture.

## Registered later items; no implementation in 70-F1A

**70-F1B — offline archival contract**, owner release/distribution. Ready for a
future isolated child after this assessment merges; no compiler/policy/cloud
dependency. Scope is pure manifest construction, byte verification, and a
store interface exercised by temporary-directory/in-memory test doubles.
Proposed files (not present yet):
`verification/areas/distribution_release/governance/archive_manifest.py`,
`archive_store.py`, `archive_offline_selftest.py` in that same directory;
`verification/areas/distribution_release/schemas/release_evidence_archive.schema.json`;
and `scripts/distribution/archive_release_evidence.py` for offline CLI entry.
No production provider adapter, credential acquisition, workflow wiring,
existing report/index schema rewrite, Cargo, producer execution or transport
download belongs to F1B. Keep hand-maintained files below 900 lines by ownership.

The separate versioned archive manifest binds repository/source, workflow path,
run ID/attempt, index SHA-256, submodule/lock/toolchain/profile identities and
each logical artifact's role, producer identity, byte size, SHA-256 and
content-addressed key. Root is `sifr/<S>/<R>/<A>/<index-sha256>/`; the root
manifest is written once, after every byte object is verified. Manifest digest
is recorded externally as the trusted readback anchor. Separate append-only
copy/readback receipts bind that digest; no receipt rewrites its own manifest.
Object version IDs are optional provider receipts, never a substitute for
byte hashing. Duplicate logical records are rejected even if their hashes
agree; distinct logical roles may reference the same verified content object.
Use the existing governance `canonical_json_bytes` convention for the new
manifest, strict duplicate-key rejection on parse, deterministic record ordering
by logical identity, and SHA-256 of those exact canonical bytes. Reject absolute
paths, traversal, symlinks and conflicting normalized paths before accessing
payloads; fail on declared-versus-observed byte size or digest mismatch.

Inventory must include the complete four-native/two-structured-skip matrix,
all 20 prospective payload classes plus new index, original transport ZIPs,
run metadata, expanded release-profile report and every bound result, separate
documentation results, raw producer logs, source/toolchain/profile inventory,
support/candidate records and review evidence listed in Item 70. Later approval
and publication evidence enters a new additive manifest only when produced;
its absence before publication is not manufactured as a successful receipt.
Missing required producer classes fail closed. Do not rebind historical bytes
or relax the live 30-day freshness validator when reading an old archive.

The interface supports create-only object/manifest writes and exact-byte reads.
Reject a conflicting existing key and incomplete inventory before sealing;
an existing identical content object may be reused only after full verification.
Verify a fresh read from each configured custody location against an externally
pinned manifest digest, expected source/run/attempt and exact inventory.
Partial or interrupted copy cannot emit success. Offline doubles prove logic
only; they cannot prove cloud immutability, permissions or physical independence.

Exact planned F1B checks (register now, implement and run only in F1B):

```bash
python3 -m unittest verification.areas.distribution_release.governance.archive_offline_selftest
git diff --check
python3 scripts/check_file_size_guardrails.py
```

The new `unittest` module must discover these named cases:
`test_complete_all_class_roundtrip`, `test_missing_mutated_truncated_bytes`,
`test_duplicate_logical_records_and_wrong_source_run_attempt`,
`test_create_only_conflict_and_interrupted_manifest`,
`test_independent_copy_requires_complete_verified_readback`,
`test_archive_read_does_not_relax_live_freshness`, and
`test_unsafe_paths_symlinks_and_untrusted_manifest_digest`.
They use fresh synthetic identities and local bytes, with no subprocess build
or network. CLI contract to implement in that later item:

```text
python3 scripts/distribution/archive_release_evidence.py plan --inventory <inventory.json> --payload-root <owned-input> --out <new-manifest.json>
python3 scripts/distribution/archive_release_evidence.py verify-local --manifest <manifest.json> --manifest-sha256 <trusted-digest> --expected-source <S> --expected-run <R> --expected-attempt <A> --payload-root <separate-copy>
```

**70-F1C — actual backend/copy delivery and online proof**, owner
release/distribution with coordinator storage disposition. Blocked on actual
primary/copy identities and approved access, not on compiler delivery for an
isolated synthetic archive proof. Select one adapter, inspect actual policies,
and freeze the exact focused provider-test commands before dispatch. Possible
read-only commands once a repository-scoped bucket is identified:
`aws s3api get-bucket-versioning --bucket <bucket>`,
`aws s3api get-object-lock-configuration --bucket <bucket>`,
`aws s3api get-bucket-lifecycle-configuration --bucket <bucket>` for S3;
for R2, authenticated `GET /accounts/<account_id>/r2/buckets/<bucket>/lock`
on `api.cloudflare.com/client/v4`, plus its lifecycle metadata. These are
unexecuted templates, not available resource identities or current proof.
After separately authorized online scope, prove every write and independent
readback, reject mutation/overwrite/duplicate/source-run mismatch, verify
no automatic expiry, and capture sanitized policy and object receipts.
No test upload or resource/permission mutation is authorized by this assessment.

**70-F1D — producer/workflow integration and NEW qualification**, owner
release/distribution after delivered compiler/accepted solo policy and F1B/C.
Capture before cleanup/Actions expiry; update directly owned workflow/runner
and archive schema contracts only as required. Freeze targeted integration
commands before dispatch, one exact-SHA Opus review (max one remediation),
and one merge-profile gate on its final changed implementation SHA, reusing
an existing exact-SHA pass. Subsequent full release profile and qualification
require coordinated platform capacity and the complete Item 70 matrix.
Publication remains separate with actual GitHub-recorded human approval.
These registrations refine 70-F1; none is started by this assessment.

## Checks and terminal boundary

Named checks for F1A only: `git diff --check`,
`python3 scripts/check_file_size_guardrails.py`, local documentation links and
paths, metadata counts and source/API evidence identities above. All later
commands are specifications and were not run. No Cargo, native execution,
historical search, create-PR gate, merge gate, qualification or publication.
One exact-SHA Opus assessment review, at most one remediation; publish review
and validation outside the reviewed tree. Merge assessment, then update the
phase record without another review or Sifr gate, and stop.
