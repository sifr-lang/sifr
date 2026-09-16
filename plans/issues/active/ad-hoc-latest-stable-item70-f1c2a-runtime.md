# Item 70-F1C2A: real SDK runtime and controlled configuration transport

State: implementation complete; pre-edit registration retained below, 2026-09-08.
Owner: release/distribution, issue #3775, child `/root/item70_f1c2a`.
Owned clone `/private/tmp/sifr-item70-f1c2a.rE2PJS/codebase`, branch
`codex/latest-stable-item70-f1c2a`, base
`07d47de0a8358dde0344b6b2cf01d3ee5057cd07`; all evidence, cache, venv and
temporary paths belong to `/private/tmp/sifr-item70-f1c2a.rE2PJS`.
Parent and predecessor paths remain read-only.

The coordinator's top F1C2A registration in the parent phase ledger splits
offline runtime work from later F1C2B access/custody proof. Preserve the
[F1C0 contract](ad-hoc-latest-stable-item70-f1c0-r2-plan.md) and F1B/F1C1
semantics. No cloud operation, credential discovery, workflow wiring,
compiler, fixture, historical research, qualification or later-item work.

## Exact implementation registration

New paths under `verification/areas/distribution_release/`:

- `governance/archive_r2_runtime.py`: maintained Boto3/Botocore factory,
  explicit supplied credentials, isolated SDK configuration, endpoint/TLS/
  request binding and one total attempt; lifecycle observation bridge.
- `governance/archive_r2_control.py`: explicit read-only Cloudflare HTTPS GET
  transport for bucket metadata/locks, no redirect/proxy/ambient auth, bounded
  complete fresh responses and sanitized observations.
- `governance/archive_r2_runtime_selftest.py`: actual SDK serialization,
  SDK stubs and local transport doubles only; deny all real sockets.
- `runtime/pyproject.toml` and `runtime/uv.lock`: distribution-owned exact SDK
  dependencies, Python 3.14.7, uv 0.12.5; lock is tracked and gate-bearing.

Only this item document and `ad-hoc-latest-stable-release-convergence.md`
may additionally change as scoped phase records. No existing adapter edit
is currently needed, so the existing nine-case `archive_r2_selftest` is not
registered for execution. Any necessary adapter integration must be registered
here before its edit and nine-case test invocation.

## Exact commands and focused test contract

Run from the owned clone, with these task-owned environment variables:

```sh
export UV_CACHE_DIR=/private/tmp/sifr-item70-f1c2a.rE2PJS/uv-cache
export UV_PROJECT_ENVIRONMENT=/private/tmp/sifr-item70-f1c2a.rE2PJS/venv
export UV_PYTHON_DOWNLOADS=never
export UV_PYTHON=/Users/yaseralnajjar/.local/share/uv/python/cpython-3.14.7-macos-aarch64-none/bin/python3
export TMPDIR=/private/tmp/sifr-item70-f1c2a.rE2PJS
```

Read-only package metadata retrieval (no provider requests):
`curl --fail --silent --show-error https://pypi.org/pypi/boto3/json --output /private/tmp/sifr-item70-f1c2a.rE2PJS/boto3.json`
and the same exact command for `botocore/json` / `botocore.json`.
Use `jq '{version:.info.version,requires_python:.info.requires_python,requires_dist:.info.requires_dist,files:[.urls[]|{filename,digests,url}]}'`
on each metadata file; `shasum -a 256` on both files preserves metadata identity.
Choose the latest compatible stable exact versions from those official records.

```sh
uv lock --project verification/areas/distribution_release/runtime --default-index https://pypi.org/simple
uv lock --check --project verification/areas/distribution_release/runtime --offline
uv sync --locked --project verification/areas/distribution_release/runtime --no-dev
uv run --locked --offline --project verification/areas/distribution_release/runtime python -m unittest verification.areas.distribution_release.governance.archive_r2_runtime_selftest
git diff --check
python3 scripts/check_file_size_guardrails.py
```

Named cases in the one new suite:

- `test_exact_runtime_versions_and_lock_hashes`: installed SDK pins, Python,
  lock dependency hashes against retained official metadata.
- `test_conditional_put_serialization_and_target_binding`: actual signed SDK
  method, canonical target/path, condition, region/signature and TLS settings.
- `test_one_total_attempt_and_redirect_refusal`: one transport attempt for
  retriable/ambiguous/redirect responses and errors, no endpoint diversion.
- `test_no_ambient_credentials_or_configuration`: hostile profiles/files,
  metadata, endpoint/proxy/CA settings cannot supply credentials or retarget.
- `test_required_only_checksums`: actual PUT wire headers omit optional
  algorithms/trailers while SigV4 remains SDK-owned.
- `test_fresh_complete_sdk_reads_and_cleanup`: fresh GET/body each time,
  length/truncation/error handling and stream close with real SDK parsing.
- `test_secret_safe_factory_and_transport_errors`: credentials, bearer values,
  arbitrary exception text and authorization headers never enter errors.
- `test_controlled_get_target_binding_and_observations`: exact metadata/lock
  method/host/path/jurisdiction, bound raw-response digests and fresh reads.
- `test_controlled_get_failures_and_cleanup`: redirect/status/size/truncated/
  malformed/duplicate-key responses fail closed and connection/body close.
- `test_lifecycle_sdk_observation_binding`: actual SDK lifecycle request/stub
  200 rules and explicit absent configuration; denial/mismatch fail closed.

The version/hash case uses the two retained metadata files via explicit
`SIFR_R2_SDK_METADATA_DIR=/private/tmp/sifr-item70-f1c2a.rE2PJS`; it compares
the locked SDK wheel/source artifact hashes to official metadata. This variable
is required only for that evidence assertion, not runtime operation.

## Review, gate dependency and terminal boundary

Freeze one candidate, open one draft PR, and request one exact-base/exact-SHA
Opus source-inspection-only review (Read/Grep/Glob tools, no commands or tests),
with at most one remediation review. Preserve final evidence outside Git.
The new `runtime/uv.lock` requires one merge-profile gate on the final candidate;
no create-PR gate when merging the same SHA here. Do not run a knowingly
blocked gate or reset predecessor counters. Coordinator reports compiler
PR #3717 and solo-policy PR #3785 remain unmerged with retained gate failures;
check actual main and coordinator before any heavy/native work. E2 B38 owns
priority for its conditional proof. Finish focused implementation/review first.
If those external prerequisites still block, preserve candidate and evidence,
record the precise blocker and stop without merging or starting F1C2B/F1D.

## Implemented boundary and dependency evidence

Official [Boto3 PyPI metadata](https://pypi.org/pypi/boto3/json) and
[Botocore PyPI metadata](https://pypi.org/pypi/botocore/json), retrieved
2026-09-08, both select latest stable `1.43.89`. Boto3 requires Botocore
`>=1.43.89,<1.44.0`; both require Python `>=3.10`. The owned lock pins both
plus jmespath 1.1.0, s3transfer 0.19.2, python-dateutil 2.9.0.post0, six 1.17.0,
and urllib3 2.7.0 with official artifact SHA-256 hashes. Lock generation,
offline lock check and locked sync passed. This lock and pyproject are runtime
inputs; the owned installation/cache and downloaded metadata are verification
evidence, not alternate dependency authorities. Metadata JSON SHA-256:

- `boto3.json`: `ebcbba503f36cd02050f2129d455a2549a2c1060fc9bfce828e716fbfa9e6583`.
- `botocore.json`: `0b33ee3748fb26faadf35778fb02176c2d86120f929fb93265b1652ebc8b6304`.

The version/hash test embeds the two official wheel/source digest pairs so
it remains reproducible without the original cache. With the registered
`SIFR_R2_SDK_METADATA_DIR` it additionally compares the retained PyPI records;
the final item evidence uses that explicit directory. No metadata fetch occurs
inside the tests. SDK configuration follows the official
[Config reference](https://docs.aws.amazon.com/botocore/latest/reference/config.html)
and [retry contract](https://docs.aws.amazon.com/boto3/latest/guide/retries.html).

`create_r2_client` creates a fresh session with explicit credentials and
role/reference binding. Its isolated session never loads shared profile or
credential files; an empty credential resolver and SDK-owned model search path
disable metadata/container/web-identity and custom ambient model discovery.
Session configuration does not read environment overrides. Explicit configuration
fixes endpoint, region `auto`, path addressing, SigV4, verified TLS, no proxy,
no endpoint discovery, required-only checksums and one total attempt. Caller
parameter checks run before SDK byte-to-stream conversion; the final signed
request is rechecked before sending. Errors stop before S3 redirect handling
can discover a region or issue a HEAD/retry. The existing adapter remains the
sanitized error/full-body transport boundary. No signing code is implemented.

Roles limit producer to conditional PUT/fresh GET, reader to GET, receipt writer
to receipt-key PUT, and configuration reader to lifecycle GET. These are local
operation limits, not a claim of actual provider permissions. Configuration
observations require a separate nonsecret reference. `observe_lifecycle` binds
the real SDK request/response and explicit 404/NoSuchLifecycleConfiguration;
its raw response is canonical SDK-shaped JSON, with dates converted to ISO
text and opaque transport headers/messages excluded. This is the existing
F1C1 representation, not a claim to retain raw HTTP XML bytes.

`observe_control` constructs only the fixed Cloudflare HTTPS metadata/locks
GET paths, binds jurisdiction and uses the SDK-owned CA bundle explicitly.
It does not follow redirects, use proxies or discover authentication. A new
connection/response is consumed and closed each time; responses are bounded
to 1 MiB, framing/encoding errors fail closed, and the existing strict parser
checks the exact raw JSON and observation identity. Credentials are hidden
from representations and error text; only the nonsecret authority reference
is recorded. Pure `admit_configuration` remains the policy authority.

The first focused run found SDK Body conversion preceding the original
generic hook; fixed locally before review by moving parameter checks earlier.
The second run passed nine cases and identified a test-only bytes/string
header expectation, corrected before the final candidate. No broad suite,
Sifr gate, existing nine-case suite or provider call was run. Final validation
and exact-SHA review evidence will be retained outside the reviewed Git tree.
