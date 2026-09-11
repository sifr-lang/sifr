# Item 70-F1C0: selected R2 contract and offline implementation plan

Current supersession, 2026-09-11: [Item70-F1C2B](ad-hoc-latest-stable-item70-f1c2b-minimal-retention.md)
retires this R2 selection and independent-copy prerequisite under the user's
minimal retention direction. This reviewed plan remains historical evidence;
its unused adapter is removed, not retained as a live legacy/fallback lane.
Existing GitHub Release publication/readback is the supported-deliverable path.

COMPLETE via [PR #3816](https://github.com/sifr-lang/sifr/pull/3816), merged
2026-09-08T09:01:45Z. Exact tested/reviewed candidate
`cae41b5aaf98f132d6eebe39f761d4a514293e38`, merge
`a69457362791b1807475cded4a98ea9a04ee0ceb`.
Owner: release/distribution,
[issue #3775](https://github.com/sifr-lang/sifr/issues/3775).
This implements the coordinator's registered documentation-only F1C0 scope.
R2 Standard is the selected primary design, with indefinite bucket locks,
configuration credentials segregated from producer credentials, and a
separately controlled full-byte copy. Provider selection is settled. No actual
account, bucket, credential, copy medium or custodian is established here.

The [F1A assessment](ad-hoc-latest-stable-item70-f1a-backend-readiness.md) and
[merged F1B contract](ad-hoc-latest-stable-item70-f1b-offline-archive.md) are
prerequisites. The [Item 70 disposition](ad-hoc-latest-stable-item70-evidence-recovery.md)
and complete four-native/two-structured-skip matrix remain authoritative.
Historical custody remains INCOMPLETE/UNRECOVERED; this plan creates no
qualification, approval, publication or retrospective evidence.

## Canonical boundary and capability evidence

Use the existing [ArchiveStore](../../../verification/areas/distribution_release/governance/archive_store.py)
protocol unchanged: `location`, atomic `create(key, content) -> bool`, and
fresh `read(key) -> bytes`. Existing `seal`, `readback`, and `attest_readback`
continue to own inventory validation, hashing, root sealing and append-only
receipts. Provider code transports exact bytes; it does not duplicate the
manifest validator or declare administrative independence.

Official primary sources checked 2026-09-08:

| Source identity | Supported fact and planning consequence |
| --- | --- |
| [R2 S3 API compatibility](https://developers.cloudflare.com/r2/api/s3/api/), object-level `PutObject`, `GetObject`, and unsupported bucket operations | `PutObject` supports `If-None-Match` and `STANDARD`; direct `GetObject` is supported. S3 versioning, bucket policy, replication and Object Lock configuration are unsupported. Do not invent version IDs or use AWS policy/lock APIs. |
| [Boto3 PutObject](https://docs.aws.amazon.com/boto3/latest/reference/services/s3/client/put_object.html), `IfNoneMatch` parameter | The standard SDK spelling is `IfNoneMatch="*"`; the documented S3 condition refuses an existing key with 412. R2's supported condition is the basis for the design, not an executed R2 race proof. |
| [R2 consistency](https://developers.cloudflare.com/r2/reference/consistency/), operations and caching | Direct S3 reads are strongly consistent and bypass the public cache. IAM changes can take up to a minute to propagate. A cached custom-domain read is unsuitable evidence. |
| [R2 bucket locks](https://developers.cloudflare.com/r2/buckets/bucket-locks/), rule semantics | Enabled locks prevent overwrite/delete, cover new and existing objects, and outrank lifecycle expiry. Prefixless rules cover all objects. Indefinite rules have no end date, but administrators can remove them. This is administrator-controlled retention, not irrevocable compliance lock. |
| [Get bucket lock rules](https://developers.cloudflare.com/api/resources/r2/subresources/buckets/subresources/locks/methods/get/), `GET /accounts/{account_id}/r2/buckets/{bucket_name}/lock` | Control-plane response has rules with `id`, `enabled`, optional `prefix`, and `condition.type` including `Indefinite`. This separate API, not S3 Object Lock, is the protection authority. |
| [R2 lifecycle](https://developers.cloudflare.com/r2/buckets/object-lifecycles/), behavior and get configuration | Lifecycle expiration/transition is configurable; the default seven-day rule aborts incomplete multipart uploads. Retention admission must distinguish that from deleting completed evidence. |
| [R2 authentication](https://developers.cloudflare.com/r2/api/tokens/), endpoint and permissions | Bucket-scoped Object Read & Write and Object Read only use S3; bucket configuration uses account-level administration. Account/jurisdiction select the endpoint. These roles exist in the provider model, not as verified project credentials. |
| [Get bucket](https://developers.cloudflare.com/api/resources/r2/subresources/buckets/methods/get/), `GET /accounts/{account_id}/r2/buckets/{bucket_name}` | Metadata includes bucket name, jurisdiction, creation date and default storage class. Response identity must match the selected target. |
| [R2 limits](https://developers.cloudflare.com/r2/platform/limits/), limits table | Single-part upload is limited to 5 GiB; keys to 1,024 bytes. The adapter must reject unsupported size before writing, and never omit a required artifact to fit. |
| [R2 errors](https://developers.cloudflare.com/r2/api/error-codes/), authentication and bucket errors | Authentication, permission, entitlement and missing-bucket errors are failures, not evidence that an object exists or can be created. |

The atomic-create interpretation follows the supported conditional operation;
it is not inferred from a preceding `HEAD` or from strong consistency alone.
The published R2 sources do not specify error precedence when a conditional
duplicate write also hits a bucket lock. F1C2 must prove that the selected
locked bucket returns the supported duplicate outcome. A 403 or other outcome
must fail closed, be recorded with its exact sanitized provider code, and be
routed to a later bounded compatibility assessment. Do not reinterpret general
permission denial as successful deduplication, remove locks, or add fallback.
This uncertainty does not prevent isolated offline implementation of the
documented conditional contract; it does prevent claiming live readiness.

## Adapter operations and failure contract

`R2ArchiveStore` takes an immutable validated target and an explicitly supplied
S3 client. The client boundary uses standard SDK operations, never handwritten
SigV4, a Worker proxy, shell-assembled signed requests, or alternate providers.
The later runtime factory must use the maintained Boto3/Botocore SDK with TLS
verification, explicit credentials, endpoint and region `auto`; ambient AWS
profiles, EC2 metadata, automatic endpoint discovery, and redirects to a
different endpoint are outside the supported boundary. No credential values,
authorization headers, presigned URLs or raw SDK exceptions enter receipts.

The target binds account ID (32 lowercase hex), jurisdiction
(`default`, `eu`, `us`, or `fedramp`), bucket (R2's lowercase 3–63 character
name contract), exact archive root, storage class `STANDARD`, and nonsecret
credential-role reference. Derive the HTTPS endpoint from account/jurisdiction:
`https://<account>.r2.cloudflarestorage.com` for default and
`https://<account>.<jurisdiction>.r2.cloudflarestorage.com` otherwise. No arbitrary
host, userinfo, query, port override, path prefix or custom domain is accepted.
The actual client's endpoint, region and path addressing must match that
target; supplied target labels alone do not prove a client's destination.

`location` is `r2://<account>/<jurisdiction>/<bucket>`, without credentials,
display-name aliases or archive prefixes. Thus two handles or prefixes in
one bucket cannot masquerade as distinct custody locations. Even distinct
locations remain only identities, not proof of independent control.

The permitted keys are exactly the bound manifest's
`sifr/<S>/<R>/<A>/<index-sha256>/` root followed by
`objects/sha256/<64-lowercase-hex>`, `manifest.json`, or
`receipts/<64-lowercase-hex>.json`. Validate source/run/attempt/index fields
through the existing canonical manifest authority before creating the target.
Reject mismatched roots, absolute/traversal/backslash/percent-encoded aliases,
unknown suffixes and overlong keys before any client operation. Pass the
canonical key to the SDK once; do not concatenate or re-encode a request URL.

| Operation | Exact behavior |
| --- | --- |
| `create` | One `put_object(Bucket=bucket, Key=key, Body=content, ContentLength=len(content), IfNoneMatch="*", StorageClass="STANDARD")`. No preceding existence check, multipart transfer helper or overwrite call. SDK-owned signing and required integrity handling remain enabled. Disable optional unsupported checksum negotiation through documented SDK configuration rather than custom signing. |
| Successful create | Accept only the expected complete success response and return `True`. Do not claim byte integrity from ETag or server checksum; `seal` immediately reads and verifies bytes. |
| Existing key | Return `False` only for HTTP 412 with the SDK's `PreconditionFailed` code from this conditional PUT. Existing content objects are then fully read/hashed by `seal`; a root manifest is never resealed, even when bytes agree. |
| Other create outcome | Raise sanitized `GovernanceError` for 401/403/404, 409, throttling, 5xx, signature/entitlement errors, unexpected status/code, malformed response and transport failure. Disable automatic write retries (one total SDK attempt). A timeout may have committed bytes; do not report success or undo them. A separately initiated retry still uses the same condition; existing F1B rules decide reuse or reject a sealed root. |
| `read` | A new `get_object(Bucket=bucket, Key=key)` every time, without Range, conditional headers, version IDs, local memoization, public URLs or a HEAD-only shortcut. Require full 200 response, consume the body to EOF, compare actual length to `ContentLength`, and close the body on success or failure. Reject partial, missing, unavailable, interrupted, malformed or over-limit reads. Return untransformed bytes for F1B's size/SHA-256/link checks. |
| Size limit | Single-part objects at most 5 GiB; `ArchiveStore` currently materializes bytes in memory. Before online dispatch measure the largest full object and host memory budget. Oversize or unsafe memory demand is a concrete later streaming/multipart mechanism prerequisite, not permission to split manifests, drop classes, or use an unconditional upload. |

SDK request IDs and sanitized status/code can be captured as transport evidence,
but they never replace independent full hashing. Missing Standard metadata,
unexpected encoding, or an expiration indicator during online readback must
be investigated against the retained control-plane observation before admission;
an ETag is not a SHA-256 digest. An `Expires` HTTP cache header is not a bucket
retention rule.

## Configuration admission and custody evidence

Keep configuration observation outside the producer `ArchiveStore`. A pure
validator consumes sanitized, explicitly bound control-plane observations;
it does not acquire credentials, set rules, or certify a live account from
test JSON. The online owner separately retrieves authenticated observations.
Evidence binds account, jurisdiction, bucket, exact endpoint, request method
and path, observation UTC timestamp, response status/request ID where returned,
raw response digest, selected archive root and source/run/attempt, and the
nonsecret authority/credential reference that obtained it. Authentication
headers and values are excluded. Strictly reject duplicate JSON keys and
missing/unknown policy shapes used for admission; a caller-authored success
boolean is not provider evidence.

Use the two exact control-plane GET paths above on
`https://api.cloudflare.com/client/v4`; pass `cf-r2-jurisdiction` for non-default
jurisdictions. Use S3 `GetBucketLifecycleConfiguration` for the same target
with separately authorized configuration-reader access. An authenticated
`NoSuchLifecycleConfiguration` is an explicit empty configuration observation;
a denied, absent or malformed response is not. No control-plane writes belong
in the adapter or its CLI.

Admission requires matching metadata and Standard selection, and an enabled
`Indefinite` lock rule whose absent/empty prefix or prefix ancestry covers
the entire archive root, including objects, manifest and all future receipts.
An objects-only prefix is insufficient. Age/Date, disabled rules, missing
coverage, and unknown condition variants cannot satisfy admission. Inspect
all rules, and reject any enabled completed-object expiration or class
transition whose prefix overlaps the archive root, including both ancestor
and descendant prefixes. Unsupported filter expressions fail closed. Disabled
rules and unrelated prefixes cannot be mistaken for active expiry. An
abort-incomplete-multipart rule alone is not completed-object expiry; this
single-part adapter does not depend on that rule. No-expiry admission remains
strict even though the provider gives locks precedence over lifecycle rules.

Capture fresh configuration observations before the online batch and after
complete readback; compare exact target/protection identities and preserve
both. These observations do not make admin changes atomic with writes or
guarantee future retention. The named configuration custodian must maintain
indefinite protection; a later protection change invalidates readiness until
assessed. Do not treat a stale/synthetic observation as current protection.

Configuration administration stays outside producer jobs. The producer has
bucket-scoped Object Read & Write only; the verifier has separately delivered
Object Read only credentials. Neither receives configuration, token-creation
or independent-copy administration authority. Object-write permission is not
a provider-enforced create-only IAM role: the conditional operation plus
bucket locks enforce the write boundary. Actual role/resource metadata and
observed producer denial of configuration management are online evidence,
not claims derived from credential names. Existing solo-maintainer authority
does not create a new human-approval checkpoint for this design.

F1B `attest_readback` creates its receipts, so a read-only verifier cannot
itself complete that API on R2. Preserve that separation explicitly: perform
all reads through the dedicated read-only client; route only the final
receipt creates through a separately supplied receipt writer bound to the
same target. The later runtime composes this `ArchiveStore` handle and checks
the bindings; it must not grant the verifier general writer credentials or
silently retry reads through the producer. Receipt-key enforcement is a
local boundary, not a claim that R2 supplies a receipt-prefix-only IAM role.
The copy custodian owns its own receipt writer and never exports its credentials
to the primary producer. A missing writer causes receipt replication failure,
not success; completed byte-read evidence remains accurately labeled.

The independent copy includes every content object, exact manifest and
append-only receipts on separately controlled retained storage or an
owner-designated offline medium outside the producer host. Required evidence
names the custodian, destination/medium identity, control and failure-domain
separation, capacity, no-expiry arrangement, and independent retrieval path.
A second prefix, two SDK handles, provider-internal replication, or producer
temporary directory does not establish it. Same-producer administrative
control over both destinations fails independence even with two bucket names.
No second provider adapter or filesystem production store is authorized here.

Pin the manifest digest outside each location being read, with independently
authenticated source/run/attempt expectations. Run the unchanged F1B readback
over every object from each actual copy; retain copy/custodian evidence
separately from the receipt's `complete-byte-readback-only` assurance. A mock
receipt can demonstrate algorithm behavior only. Durable archived old bytes
still do not satisfy live freshness or publication approval.

## Existing facts and minimum missing actions

Reuse F1A's scoped metadata from 2026-09-08T07:57:51Z–07:57:56Z, including
`metadata.json` SHA-256
`34090f6bd042dbfca1b7150104a4c7bca2929671b74f8b7fd060c5da43d7dc7d`
and [published metadata](https://github.com/sifr-lang/sifr/pull/3812#issuecomment-5581503738).
It established no project storage secret/variable, bucket/endpoint, AWS
identity, or independent copy within its coverage. No relevant changed
configuration was supplied, so no account inventory or metadata sweep was
repeated. This is not proof that no Cloudflare account or backup exists.

Read-only dependency inspection at base
`0c0f4f046b769deeb8cf91a83d62fbb7ae983ec1` found Boto3 and Botocore 1.43.80
in the Python interop `uv.lock`, with Boto3 declared in its `pyproject.toml`.
The active Python 3.14.7 has neither distribution installed. That other
owner's lock is availability evidence, not a distribution-runtime dependency
or permission to modify/sync its environment. Cloudflare's
[Boto3 example](https://developers.cloudflare.com/r2/examples/aws/boto3/)
supports using the standard SDK. F1C1 stays standard-library-only with injected
SDK-shaped doubles; F1C2 must freeze a distribution-owned SDK environment
before real factory/wire checks. No dependency was installed or updated here.

| Missing fact/action | Existing owner and consequence |
| --- | --- |
| Actual authorized Cloudflare account ID, R2 entitlement, selected bucket and jurisdiction | Coordinator/storage owner supplies existing project-scoped identities or resolves the concrete access/provisioning action. Provider selection is not repeated and no purchase or resource is inferred. |
| Authenticated lock/lifecycle/metadata and configuration custodian | Storage owner establishes indefinite complete-root protection and Standard/no-expiry admission under separately authorized online scope. No secret values are requested in the plan. |
| Scoped producer, read-only verifier and receipt-writer credential delivery; admin separation | Storage/release owner supplies nonsecret credential references and resource/role evidence, then proves actual access/denial. GitHub repository admin capability is not R2 access. |
| Independently controlled complete-copy destination, custodian and retrieval procedure | Coordinator/copy custodian names the actual medium or destination, retention/capacity and who can alter it. This is the concrete unresolved custody action; names and mocks cannot supply it. |
| Reproducible distribution SDK runtime and actual byte/memory budget | Release/distribution freezes installation inputs and SDK/wire checks in F1C2, then measures synthetic proof size. No compiler prerequisite for isolated proof. |

These are F1C2 prerequisites, not blockers to closing this bounded plan or
starting the later offline F1C1 item in a fresh session.

## Registered later items; not implemented here

**70-F1C1 — offline R2 adapter and admission contract**, release/distribution.
COMPLETE via [PR #3818](https://github.com/sifr-lang/sifr/pull/3818), merged
2026-09-08T09:22:26Z. Exact tested/reviewed candidate
`29a1f965ebfdfb7c40261d903a993e74507fdc13`, merge
`c1d91d5df93ac289270941473ce3c5c89c2666c6`. No live account, compiler/policy, credential or SDK
installation prerequisite. Exact new implementation paths:

- `verification/areas/distribution_release/governance/archive_r2_store.py`:
  SDK-shaped client protocol, validated target binding, canonical location,
  conditional create/fresh read, sanitized failures and explicit split
  read/receipt-write composition using separately supplied clients.
- `verification/areas/distribution_release/governance/archive_r2_config.py`:
  pure target/control-plane observation parsing and lock/lifecycle admission.
- `verification/areas/distribution_release/governance/archive_r2_selftest.py`:
  inline synthetic policy/client/body doubles and contract tests.
- This plan and the active convergence ledger for item records only.

Keep F1B source/schema/CLI unchanged; reuse its existing synthetic complete
inventory builder by import for integration proof. No new schema enrollment,
real SDK factory, CLI transport, fixture file, workflow, lockfile, online
probe or provider resource belongs to F1C1. If the planned boundary cannot be
implemented without changing that scope, record the exact issue for later
registration rather than starting another item. Files remain under 900 lines
by responsibility, not arbitrary slicing.

Exact F1C1 commands, registered now and not executed in F1C0. Run at the
owned repository root with the project's maintained Python 3.14.7 selected
as `python3`. Imports use the standard library and in-repository validators,
including `archive_offline_selftest.synthetic_bundle`; no ambient Boto3,
virtual environment, external JSON-schema package or dependency sync is needed:

```text
python3 -m unittest verification.areas.distribution_release.governance.archive_r2_selftest
git diff --check
python3 scripts/check_file_size_guardrails.py
```

Required discovered cases, using only fresh inline bytes and test identities:

| Test name | Required assertions |
| --- | --- |
| `test_target_endpoint_key_and_location_binding` | Accept canonical account/jurisdiction/bucket/root; reject malformed/aliased targets, client mismatch, wrong source/run/attempt/root and unsafe/unknown keys before calls; same bucket always has the same location. |
| `test_atomic_conditional_create_and_existing_content` | Exact conditional PUT parameters, no HEAD/unconditional/multipart call, one attempt; deterministic competing creates preserve first bytes, only 412/PreconditionFailed is False; unchanged F1B accepts identical object reuse and rejects conflicting bytes/reseal. |
| `test_create_errors_and_ambiguous_completion_fail_closed` | 401/403/404/409/429/5xx, wrong 412 code, malformed success, timeout before/after simulated commit all raise; no success receipt, automatic retry, delete or permission fallback. |
| `test_fresh_complete_reads_and_body_cleanup` | Repeated calls issue fresh GETs; mutation between reads is detected by F1B; verify full length/EOF, close on success/error; reject partial/ranged, missing, truncated, unavailable and oversized data. ETag/checksum alone never passes. |
| `test_indefinite_complete_root_lock_admission` | Accept active all-bucket/root-ancestor Indefinite coverage; reject missing/disabled/Age/Date/objects-only/unknown rules, mismatched authenticated-observation bindings and duplicate keys. |
| `test_no_expiry_standard_lifecycle_admission` | Active ancestor/descendant expiration and class transition reject; malformed/missing/denied and unknown filters reject; unrelated/disabled rules and abort-incomplete-only are distinguished; metadata/Standard mismatch rejects. |
| `test_reader_receipt_writer_and_custody_separation` | All object/manifest reads use the explicit read-only double; only receipt creates use the separate bound writer; wrong target and absent writer reject, no producer-read fallback; duplicate location aliases reject and different buckets never claim administrative independence. |
| `test_all_class_seal_readback_and_interrupted_copy` | Reuse F1B's complete four-native/two-skip inventory through the R2 double and two simulated locations; pin external digest and identities; missing/mutated copy or interrupted receipt replication fails with original manifest intact and truthful byte-only assurance. |
| `test_single_part_size_and_no_live_side_effects` | Reject over-limit declarations/content without allocating 5 GiB or performing calls; test module needs no SDK/network/credentials/subprocess build; no factory, workflow or provision side effect. |

Mocks establish adapter logic, parameterization and refusal behavior only.
They cannot prove real concurrent conditional serialization, actual R2
retention/error precedence, credential rights or custody independence.
Use one exact-SHA Opus review plus at most one remediation; only source
inspection in review. Apply actual changed-path gate rules; the registered
source/docs-only F1C1 paths require no Sifr gate. Merge, record, and stop.

F1C1 implementation: `R2Target` is immutable and validates the canonical
manifest against an externally supplied digest and source/run/attempt before
deriving the root. Clients expose standard SDK `meta.endpoint_url` and
`meta.config` fields; admission requires region `auto`, path addressing,
one total request attempt and required-only checksum negotiation. The same
binding is rechecked before every operation. Successful PUT requires 200 and
a nonempty ETag, used only as response completeness metadata. GET requires
200, exact ContentLength and explicit STANDARD, reads through EOF and closes
the body, rejecting range/encoding/expiration indicators. The ETag never
establishes byte integrity. Missing Standard metadata is a fail-closed online
compatibility question, not an inferred Standard default.

`PolicyObservation` holds immutable UTF-8 JSON envelope bytes. Its exact fields
are `account`, `jurisdiction`, `bucket`, `archive_root`, `source_commit`,
`run_id`, `run_attempt`, `endpoint`, `method`, `path`, `operation`, `headers`,
`observed_at`, `status`, `request_id`, `authority_ref`, `raw_response` and
`response_sha256`. `raw_response` preserves the provider JSON text whose
UTF-8 bytes the digest identifies. Operations are `metadata`, `locks` and
`lifecycle`; paths and jurisdiction headers follow the contract above.
Lifecycle observations serialize the SDK response shape with ResponseMetadata,
Rules or the explicit 404/NoSuchLifecycleConfiguration error. All JSON layers
reject duplicate keys; policy shapes admit only supported fields and filters.
The admission result retains all three observations and says
`supplied-policy-observations-only`. This format is an offline consumption
boundary, not a network transport, authentication claim or live freshness check.
The later owner must capture and preserve before/after observations.

`R2ArchiveReader` exposes only reads. `R2ReadbackStore` composes it with a
separate `R2ArchiveStore` receipt writer with matching location/root/digest
and distinct client/reference; only receipt keys may be written. Every read,
including receipt verification, stays on the dedicated reader. Local labels
do not prove the actual credential rights or administrative independence.
Only the three registered Python modules and this plan/phase record change;
the F1B manifest, schema, store, CLI and synthetic builder remain unchanged.
Owned clone `/private/tmp/sifr-item70-f1c1.fDMx42/codebase`, branch
`codex/latest-stable-item70-f1c1`, base
`5614f06c8ff49411dd8d0b8107e6479e4274ec96`; sibling private evidence root.
No SDK installation, factory, real service call, fixture, workflow, lock,
Cargo, native run, historical recovery, F1C2/D work or qualification occurs.

F1C1 final [validation](https://github.com/sifr-lang/sifr/pull/3818#issuecomment-5582542157)
passed all nine named tests under Python 3.14.7 in 1.277 seconds, both diff
checks and file-size guard (3,770 files). The one
[Opus review](https://github.com/sifr-lang/sifr/pull/3818#issuecomment-5582574518)
returned SATISFIED/no blockers, Read/Grep/Glob only and no command/test/network
execution. Raw review SHA-256
`11a289c658c2e209d16c5ae35140e37bcee81c7faaeeb7e1f419b88d591364a4`.
Zero remediation reviews, zero Sifr gates. Four nonblocking maintainability/
diagnostic suggestions are deferred as 70-F1C1-M1 in the phase ledger, not
implemented and not new F1C2 prerequisites. Blocker: none. This record-only
closure reuses implementation evidence without another review or gate; after
record merge, stop. No later item is started.

**70-F1C2 — real SDK/configuration/access and independent-copy proof**,
release/distribution with the existing storage/copy owners. Not ready for
online dispatch until the concrete facts above exist and its exact paths,
commands and permitted mutations are separately registered. Own the standard
SDK factory, controlled configuration GET transport, sanctioned credential
delivery and a bounded synthetic cloud proof. Choose a distribution-owned
locked Python environment; do not silently consume another owner's mutable
venv or write a signing implementation. Prove serialized SDK headers, target
binding/TLS, absence of automatic write retries/ambient credentials, primary
conditional create including duplicates under locks and competing writers,
fresh all-byte reads through the independent reader, deny overwrite/delete
and producer configuration changes on designated synthetic evidence, and
complete custodian-controlled copy/readback with externally pinned digest.
The online scope must name permitted probe keys and their indefinite-retention
consequence; no cleanup requiring lock removal is implied. Capture sanitized
before/after configuration and role observations and actual request outcomes.
No real upload, token creation, provisioning, mutation or acquisition is
authorized by F1C0. A provider mismatch stops that later proof for bounded
rescoping. Apply a merge gate only if actual changed paths trigger it.

**70-F1D — producer integration and NEW qualification** remains separate,
after delivered compiler/accepted solo policy and demonstrated F1C2 primary
and independent-copy custody. Its existing F1A/Item 70 registration owns
complete capture, workflow changes, exact-SHA validation and new source/run/
attempt/inventory identities. No F1D code or qualification is started here.

## F1C0 ownership, checks and terminal boundary

Owned isolated clone `/private/tmp/sifr-item70-f1c0.XM898D/codebase`, branch
`codex/latest-stable-item70-f1c0`, base
`0c0f4f046b769deeb8cf91a83d62fbb7ae983ec1`; sibling private evidence root
`/private/tmp/sifr-item70-f1c0.XM898D`. Parent/predecessor trees, branches,
indexes, targets and temporary stores are read-only to this child.

Relevant exact base-tree blobs:

| Path under repository root | Blob |
| --- | --- |
| `verification/areas/distribution_release/governance/archive_store.py` | `eae85298b50de1a89da7465c1f8b924271b31d65` |
| `verification/areas/distribution_release/governance/archive_manifest.py` | `c2c1c91be0aeea84ee5db21d6f5fb8fe91eedc10` |
| `verification/areas/distribution_release/governance/archive_bindings.py` | `eb1f17f3cb1c5ac4d08b6a58656629b58c7f021a` |
| `verification/areas/python_interop/pyproject.toml` | `bab5656e95d48eb418bbdca0b983e09bbd262b76` |
| `verification/areas/python_interop/uv.lock` | `6b88403eed24792d1b1f2b7e040ea03fa529142d` |

Primary-source HTML snapshots fetched without authentication from the exact
Cloudflare URLs in the capability table, 2026-09-08T08:51:31Z–08:51:32Z,
are retained outside Git under the owned `sources/` directory. These hashes
identify the reviewed documentation bytes, not live configuration evidence:

| Snapshot | SHA-256 |
| --- | --- |
| `r2-s3-api.html` | `c032b69ad9392f4d52c5e4d61de536453f23c0d8a0dced5167ba4c83871f12e6` |
| `r2-locks.html` | `e5c53687acb645b5099da80fd9a3f17293376892378d1801651c2e226bc15232` |
| `r2-consistency.html` | `ac9a30de94b18a7fcfea7ae5d8da48181de01a987a6fec23ff5a498b73b8ca66` |
| `r2-auth.html` | `42e6f36c07e0664a06e9c1cbd0c5a6a16a37f902a4611217cad7b77db0a67880` |
| `r2-lifecycle.html` | `d3369fa6d766e605769990b3654163c33aef862cc21ea56b0f169f6e827b6bbe` |

F1C0 checks only: `git diff --check`,
`python3 scripts/check_file_size_guardrails.py`, local documentation paths/
links and exact source/API evidence identities. Future paths above are
explicitly proposed and must be absent before implementation. Only two
Markdown files change; no Cargo, native execution, tests for another item,
create-PR/merge gate, historical search, cloud mutation or qualification.
Publish exact-SHA validation and Opus review outside the reviewed tree,
merge the plan, make a record-only update without another review/gate, then
stop. Plan blocker: none. Real account/access/custodian proof remains later
F1C2 work, explicitly unfulfilled.

Final [validation](https://github.com/sifr-lang/sifr/pull/3816#issuecomment-5582210368)
passed exact-base/working-tree diff checks, the file-size guard (3,767 files),
four local plan links and the ledger link, three absent proposed source paths,
five exact base-tree blobs, five source snapshots and API identity markers.
The one [Opus review](https://github.com/sifr-lang/sifr/pull/3816#issuecomment-5582250991)
returned SATISFIED/no blockers on that exact candidate, using only
Read/Grep/Glob; no command/test execution, remediation review or Sifr gate.
Raw review SHA-256:
`ec97097e32daf80946563e3765e07f138e9e8f16d1714881a4f93d5de184aaaa`.

Nonblocking review follow-ups stay with F1C2, not this completed item:
carry F1A's current pricing recheck into actual account/capacity planning;
retain remaining official SDK/control-plane/limits/error source snapshots
alongside the five explicitly named R2 HTML snapshots above; consider citing
the provider's longest-retention precedence for overlapping lock rules in
the eventual admission rationale. These are documentation suggestions, not
new mechanism defects, approval requirements or ready code work in F1C0.
This record-only update reuses implementation evidence without another review
or gate. After its merge, stop; no F1C1/C2/D implementation or test is started.
