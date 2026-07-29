## VERDICT: NOT SATISFIED

Three actionable defects, all in the supporting-evidence layer. The qualification transport itself (run, index, 20 payloads, archives, installer, checksums, targets, editor/VSIX) is fully clean and independently reproduced.

---

## 1. Workflow / source / submodule identity — SATISFIED

`gh api repos/sifr-lang/sifr/actions/runs/30406842210`:
- `conclusion: success`, `status: completed`, `run_attempt: 1`, `path: .github/workflows/release-qualification.yml`, `repository: sifr-lang/sifr`, `head_sha: 53cc9c4bf36762d39a0b372402d202589f920c2e`.
- All 8 jobs `success`: validate, 4 target jobs, assemble, editor, collect.
- `gh run list --workflow=release-qualification.yml` — this is the **only** run at `53cc9c4`.
- Workflow permissions are `contents: read` / `actions: read` only; all four `upload-artifact` sites declare `retention-days: 30`, `overwrite: false`.
- All 10 index submodule pins match `git submodule status` in the clean checkout, including nested `editor_integrations/vscode = 273fd5d3eb…`.

## 2. Canonical index, 6 uploads, expiry, 20 payloads, structure — SATISFIED

- `validate_qualification_artifact_index(…, require_unexpired=True)` → OK; JSON Schema validation → OK; bytes are exactly `canonical_json_bytes` (9543 B, sha256 `864b4070868a1998ec3ce87dc85ffd5514361969adaaef1a0c1023d6257a6a45`).
- All 6 governed `workflow_artifact_id`s resolve live and `expired: false`; each upload's API `expires_at` matches the index **exactly**, and every transported entry shares its upload's single expiry (aarch64-darwin `23:12:52`, aarch64-linux `23:09:34`, x86_64-darwin `23:23:00`, x86_64-linux `23:11:27`, assemble `23:24:07`, editor `23:24:06`). `workflow.expires_at` = the minimum, `23:09:34Z` — ~29.9 days of lifetime, far above the 7-day floor.
- **20/20 payloads verified by exact size and SHA-256, 534,002,666 bytes total, 0 mismatches.**
- `verify_release_archive.py` at the source commit passes for all four archives; `validate_target_report` passes for all four (canonical bytes, `smoke_status: pass`, exact builder matrix `macos-15` / `macos-15-intel` / `ubuntu-24.04` / `ubuntu-24.04-arm`).
- **Installer regenerated from the source commit is byte-identical** to the transported `sifr-installer-0.1.0` (`7a658f2e09d9…`); `checksums.txt` replayed byte-identical, and `validate_aggregate_checksums` binds exactly the 12 target-kind artifacts.
- `validate_editor_report` passes (canonical); VSIX is `sifr-vscode` 0.2.0, publisher `sifr`, `sifrCompilerCompatibility: ">=0.1.0,<0.2.0"`, `rollback_version: none`, `target_report_sha256` = the x86_64-linux report digest, thin-client contents only, `dist/` has 0 committed files.

## 3. Supporting evidence — TWO BLOCKERS

**Finding A (blocking, unsatisfiable as designed): `rust-validation-report.json` cannot pass custody.**

`evidence_custody.py:255-263` and `stable_prepare.py:799-813` both load the candidate `rust-validation-report.json` with `require_canonical=True`, while `planner.py:688-700` requires its digest to equal the release report's `result_artifacts` entry named `rust-interop-release-results.json`. That digest is computed by `release_evidence.py:232` as `sha256_file(path)` over the runner's **pretty-printed 2-space** output. Proven against the real artifact in the checkout:

```
release-report-bound digest (runner bytes):  621e3e7157f89b2bbbc1482a96a07fee706ad0bbec2896976f9981cdad0898d1
digest of canonicalized same content:        eb5d458ad3a0cd08753a115b6e12043cc5e8c25947685c2e29a26dcaf620275d
```

Verbatim bytes satisfy the digest tie but fail the canonical gate; canonicalized bytes pass the gate but break the digest tie. `evidence_custody_selftest.py:35-36,59` masks this by building `rust_bytes = canonical_json_bytes(...)` and then *setting* the release report's `result_artifacts` digest to that canonical digest — a fixture that cannot occur in a real release run.

**Finding B (blocking): staged `stable-support-claims.json` is non-canonical.** It is a byte-verbatim copy of `verification/areas/rust_interop/data/stable_support_claims.json` (good provenance), but that source file is pretty-printed, so the same `require_canonical=True` gate rejects it. Canonicalizing the copy would sever the plan digest from the in-repo artifact the `stable-candidate` validator actually checked, so this needs an explicit decision, not a silent reserialization.

Direct reproduction:
```
rust-validation-report.json:  custody canonical gate REJECTS -> must use canonical JSON bytes
stable-support-claims.json:   custody canonical gate REJECTS -> must use canonical JSON bytes
documentation-report.json:    custody canonical gate PASSES
```

**Finding C (blocking provenance): the staged `rust-validation-report.json` is not the release-profile run's artifact.** `argv[0]` shows it came from a standalone `areas run` in the clean checkout at 02:40 (`be1f537f…`, 9703 B), whereas the release-profile run's `target/verification/areas/rust-interop-release-results.json` is `621e3e71…` (9701 B, 03:01). The two are structurally identical (same 5 suites, `summary.total_failures: 0`, 10 variants, exact `stable-candidate` case ids) and differ only in non-deterministic `duration_ms` fields — so this file can never be reproduced or bound. Candidate evidence must carry the release run's own artifact bytes; this violates the exact-byte/no-rebuild requirement.

**Clean parts of item 3:**
- `documentation-report.json` — canonical, `validate_documentation_report` OK, `source_commit = 53cc9c4…`, `status: pass`, suites `structure` then `ga-release`, and `result_sha256 = 88e3bd72f52d…` matches `target/verification/areas/documentation-stable-qualification-results.json` exactly, produced by the governed `scripts/distribution/qualify_stable_documentation.py`. `report_id = docs-53cc9c4bf367-88e3bd72f52d` is consistent.
- `stable-support-claims.json` content — byte-identical to the source-commit artifact; 29 claims, 2 declared runtime deferrals; all intrinsic Rust checks pass. Its `schema_version: 1` is upstream Rust-interop-owned state, not one of the Phase 40 schema-v2 contracts (phase lines 183-189), so not a defect.
- `rust-validation-report.json` content — all 5 required suites blocking with `failed_cases: 0` / `total_failures: 0`, `validate_passing_cases` OK for every suite, exact `stable-candidate` case ids `{rust-interop-stable-candidate, rust-interop-stable-candidate-self-test}`, `bless: false`.
- `release-notes.md` — consistent with the phase: four exact targets, extension range `>=0.1.0,<0.2.0`, roll-forward-only first-GA recovery, no rollback predecessor, no signing/notarization claim, Windows/package-manager exclusion, packaged generated-Rust limitation disclosed. Non-actionable note: it does not restate the macOS 15.0 / glibc 2.39 floors, but no contract requires that of the release asset and `check_ga_release_docs.py` enforces them in `docs/` (verified present in `installation.mdx`, `releases/0.1.0.mdx`, `releases/compatibility.mdx`, `troubleshooting.mdx`).

## 4. Staleness / cross-source / exact-byte

Findings A–C above. Everything else is fit: the index, all 20 payloads, and all four target reports plus the editor report bind `53cc9c4…` and the correct submodule set; the installer and checksums are byte-reproducible from the source commit; nothing was rebuilt.

## 5. Rust-interop implementation — respected

No Rust-interop implementation was reviewed or modified. Its suites were treated purely as consumed evidence (structure, verdicts, digests).

## 6. Demo naming — SATISFIED

Recursive scan of 1,473 paths under `demos/` for `phase`, `milestone`, or a standalone `40`/`m40` token: **0 violations**. Release demos are capability-named (`stable_candidate_qualification_demo.sh`, `stable_release_governance_demo.sh`, `stable_incident_recovery_demo.sh`, `stable_self_update_demo.sh`).

## 7. Commands and checks I ran

```bash
git -C /private/tmp/sifr-phase40-release-source-53cc9c4bf367 rev-parse HEAD; git status --short; git submodule status
git -C .../editor_integrations submodule status
gh api repos/sifr-lang/sifr/actions/runs/30406842210
gh api repos/sifr-lang/sifr/actions/runs/30406842210/jobs --paginate
gh api repos/sifr-lang/sifr/actions/runs/30406842210/artifacts --paginate
gh run list --workflow=release-qualification.yml --limit 20 --json databaseId,headSha,status,conclusion,createdAt
# canonical index: semantic validator + JSON Schema + canonical bytes + all 20 payload size/SHA-256
uv run --project verification --locked --with jsonschema python /tmp/pass1_full.py
# archive structure, all four targets
scripts/distribution/verify_release_archive.py <archive> --version 0.1.0 --target <target>
# installer + checksums byte replay from the source commit
scripts/distribution/generate_version_installer.sh --version 0.1.0 --artifact-dir <replay> --out <replay>/sifr-installer-0.1.0
cmp <regenerated installer> <transported installer>; cmp <replayed checksums.txt> <transported checksums.txt>
# governed validators against real evidence
uv run … python /tmp/pass1_final.py     # validate_aggregate_checksums + validate_target_report x4
uv run … python /tmp/pass1_editor.py    # validate_editor_report
uv run … python /tmp/pass1_evidence.py  # validate_documentation_report, validate_passing_cases x5
uv run … python /tmp/pass1_proof.py     # custody require_canonical gate + digest unsatisfiability proof
cmp <evidence>/stable-support-claims.json verification/areas/rust_interop/data/stable_support_claims.json
unzip -l / unzip -p sifr-vscode-0.2.0.vsix   # VSIX structure, package.json, compiler range
git -C editor_integrations/vscode ls-files dist
python3 -c "<recursive demos/ phase-milestone-40 name scan>"
```

No repository file was modified. All scratch scripts and the replay directory live under `/tmp`. The absent canonical `release-profile-report.json` was excluded from scope as instructed — note that Finding A is nevertheless a property of that report's digest contract and must be resolved before candidate evidence can be materialized.
