# Phase 40 Milestone 40.4 Exact-Source Evidence

This record preserves the commands, identities, digests, and relevant results
used by the milestone execution ledger. It does not close the milestone or
replace the canonical candidate evidence-only PR.

## Source and qualification

- Source commit:
  `7242e4737b1ee89f9f02a3b4793d5cdb13d372ea`
- Reviewed PR head: `95d5e2bbb0d2cb133a5ab2cf3f88cc6697fa7f3d`
- Qualification workflow:
  <https://github.com/sifr-lang/sifr/actions/runs/30297288986>
- Workflow conclusion: `success`
- Workflow expiry: `2026-08-26T19:17:58Z`
- Downloaded artifact containers: 7, all unexpired
- Indexed files: 20
- Indexed bytes: 533,743,470
- Size/SHA-256 replay mismatches: 0

Commands:

```bash
gh run view 30297288986 --repo sifr-lang/sifr \
  --json databaseId,headSha,status,conclusion,jobs,url
gh run download 30297288986 --repo sifr-lang/sifr \
  --dir /tmp/sifr-phase40-evidence-30297288986/qualified
python3 scripts/distribution/release_governance.py validate \
  --kind qualification-index \
  --input /tmp/sifr-phase40-evidence-30297288986/qualified/sifr-stable-candidate-0.1.0-7242e4737b1ee89f9f02a3b4793d5cdb13d372ea-index/qualification-artifact-index.json \
  --require-canonical
```

The replay loaded every index row, resolved its artifact container and relative
path under the download directory, compared `size_bytes`, and recomputed
SHA-256. The canonical index digest is:

```text
26fd6f8c8d4beb16bc68f28bae681e5c2deedab27fdd7b9120151afb04772c41
```

Local documentation qualification was produced from the clean isolated source
checkout:

```bash
python3 scripts/distribution/qualify_stable_documentation.py \
  --source-root /private/tmp/sifr-phase40-release-source \
  --source-commit 7242e4737b1ee89f9f02a3b4793d5cdb13d372ea \
  --out /tmp/sifr-phase40-evidence-30297288986/local/qualification-documentation.json
```

The canonical qualification report id is
`docs-7242e4737b1e-038b0eabc1c1`. The producer-owned area result path is
`target/verification/areas/documentation-stable-qualification-results.json`;
its SHA-256, also recorded as the report's `result_sha256`, is
`038b0eabc1c129c64cd046217d00f84c0e6bf5a90f41d7f491f0744b6e0c39e4`.
The canonical report's SHA-256 is:

```text
aea9f6f9c8a2adbf26269534e9bb018780d0fc05715f3955920e27aa69ae18a1
```

## Performance host-variance evidence

All invocations used the checked-in full suite without a waiver or policy
change:

```bash
uv run --project verification --locked python -m sifr_verify areas run \
  --area performance --suite full
```

The first canonical report attempt passed all preceding lanes and failed three
check/diagnostic medians. Its log SHA-256 is:

```text
babdace21ecbecf5d07f3b997ee01cfe028f6a6e58b473136d76ad6cc2678aca
```

The same-host `c17f3c7d1ea1` control stopped at the first benchmark's governed
120,000 ms timeout. Its log SHA-256 is:

```text
08107ac3eecce6c16ce44daa821b48e2807c30be79ec8ca58700b7ecdc244b45
```

The unchanged `7242e4737b1e` standalone retry passed all eight variants. Its log
SHA-256 is:

```text
04e8182b9811f48f252bb1c60511a0fc17f7abdebe402b9be931bc06d9dd7a16
```

The standalone pass's benchmark evidence, which contains the recorded
1.27–1.31 second check/diagnostic cluster, has SHA-256:

```text
2bf8a8eb589cffcdd70741afd43811543af79dbc64ed8235331efe3622e8279f
```

The immediate retry failed five metrics across four benchmarks; its log
SHA-256 is:

```text
af2331d5b0e3bf29d1e3740b422c81422853c975ef000bc15735cecd565fb546
```

The later end-to-end attempt that reproduced mid-run spikes has log SHA-256:

```text
4b4b752cee1ff377bcefdcb4b6cab2d76c6fac54f77a533421ed212eaaed897a
```

Across the preserved benchmark evidence, the pressure-affected maximum sample
was 3.880 seconds. The final canonical invocation's representative
check/diagnostic medians were 1.276, 1.278, and 1.297 seconds and it had zero
budget overruns. Its benchmark-evidence SHA-256 is:

```text
6c9b3f10920630eca76c6c928ae7ba77598de980cae09939174327ed980ae8e2
```

## Canonical release-profile attempt

The report directory was created empty outside the checkout:

```bash
report_dir="$(mktemp -d /tmp/sifr-phase40-release-report-7242-pass.XXXXXX)"
scripts/run_all_tests.sh --profile release \
  --release-report-out "${report_dir}/release-profile-report.json"
```

The invocation passed every lane through performance (8 variants) and
distribution validation (56 variants). It failed only
`sysroot_release:host-installed-smoke` because the live schema-v1 payload lacks
the schema-v2 `generation`, `ga_status`, and `releases` fields.
`sysroot_release:host-installed-stdlib-heavy` passed in the same run.

Preserved result digests:

| Result | SHA-256 |
| --- | --- |
| complete canonical log | `e31e6113031e95365b4fd9620efeff655094736e79de04337e8dc3d690053b28` |
| release lane report | `5aefc8f02c9779d7e6a42d8966d0d8e949c80c24bf8657727cd987c2e6ba04d6` |
| sysroot result JSON | `3a024e885b78ca81d0e63f67211ce97a0a201d3f84c3e8f737ebffc81d4c599d` |

The public payload inspected during this run had digest
`71b3243925670f56dc510b8f45b6614a622f58097a0fea9492f61d20dc4bf9ef`
and contained only schema version 1 plus alpha/beta mappings. Milestone 40.5
owns both the protected truthful epoch bootstrap and the test-only endpoint
override that separates isolated release qualification from public smoke.
