I completed the review. Baseline suites and gates all pass; two blocking defects remain.

## Review — Phase 40 / milestone_40_1, pass 5

Branch `codex/phase-40-milestone-40-1` (7 commits) vs `origin/main`; 42 files, +4843/−148.

### Pass-4 findings: re-tested

| # | Pass-4 finding | Status |
|---|---|---|
| 1 | Non-UTF-8 remediation stopped at the planner; three sibling read sites | **Partially fixed** — `release_report.py:171,378` now catch `UnicodeError`; `qualify_stable_target.py:233-237` adds `read_utf8`; `binary-release-profile` is a permanent case (`qualification_selftest.py:583-608`) and the checksum sidecar has a case (`artifact_stable_candidate_generation.sh:78-97`). Reproduced governed exit 2 for release profile, checksums, installer, and checksum sidecar. **The sysroot site is not fixed** — see P1 #1 |
| 2 | `validate_installer_identity` is a line-prefix heuristic | **Partially fixed** — `planner.py:338-359` now collects whole lines under `^\s*(?:export\s+)?(APP_VERSION\|APP_CHANNEL)\s*=` and requires exact equality with the canonical pair; the four pass-4 probes are permanent cases (`qualification_selftest.py:528-531,685-695`). All four now rejected. **Five other assignment forms still evade** — see P1 #2 |
| 3 | Collector accepted a symlinked upload container | **Fixed** — `collect_qualification_artifacts.py:205-218` resolves the container and each entry against `directory.parent.resolve()`; permanent case at `qualification_selftest.py:207-226`. Probes: symlinked container → rejected, symlinked file→outside → rejected, legitimately symlinked `--artifact-root` → correctly accepted |
| 4 | Pass-4 record and checklist not updated | **Fixed** — pass-4 review committed (`1e1b7a13`); execution issue `:234-242` records the round and `:247-252` names the new negatives |

---

### P1 — Blocking

**1. A non-UTF-8 `sysroot.toml` inside a candidate archive still produces a raw traceback, and the remediation added for it is unreachable.**

`scripts/distribution/qualify_stable_target.py:116-121` now routes the *extracted* `sysroot.toml` through `read_utf8`. But `verify_archive` runs first, at `:100`, and `scripts/distribution/verify_release_archive.py:142` decodes the same member unguarded:

```python
manifest_source = content.decode("utf-8")
```

`qualify_target` wraps that call in `except SystemExit` only (`:101`), and `main` catches `(GovernanceError, OSError, subprocess.SubprocessError)` (`:58`) — `UnicodeDecodeError` is none of those. Reproduced against a real host archive (built via `build_release_artifacts.sh`, `sysroot.toml` replaced with `\xff\xfebad`, sidecar recomputed):

```
EXIT=1
  File ".../qualify_stable_target.py", line 100, in qualify_target
    verify_archive(str(archive), version, target)
  File ".../verify_release_archive.py", line 142, in verify_archive
    manifest_source = content.decode("utf-8")
UnicodeDecodeError: 'utf-8' codec can't decode byte 0xff in position 0: invalid start byte
```

Exit 1 with a Python stack trace, not exit 2 with a governed diagnostic. `verify_release_archive.py:142` is now the *only* remaining unguarded `.decode(` in `scripts/distribution/**` and `governance/**` (`grep -rn '\.decode('`), and it is on two live qualification paths: the matrix job (`release-qualification.yml:118`) and the assemble job (`:267`). The `read_utf8` call at `qualify_stable_target.py:117` cannot be reached by this input class at all — the file is decoded and rejected upstream — so the pass-4 fix is dead code for the defect it was written for.

The execution issue's own claim is therefore inaccurate: `plans/issues/active/phase-40-stable-channel-ga-execution.md:236-238` states remediation "now governs every release-profile/checksum/**sysroot** text decode."

Fix: route `verify_release_archive.py:142` through a governed decode (raise `SystemExit`/`GovernanceError` with a diagnostic, consistent with the surrounding `raise SystemExit(...)` style at `:136,145,164`), and add a non-UTF-8-sysroot negative to `artifact_stable_candidate_generation.sh` alongside the checksum-sidecar case at `:78-97` — that case, written one line away, would have caught this.

**2. `validate_installer_identity` is still a line-anchored heuristic; five shell assignment forms that actually change the effective value are accepted.**

`verification/areas/distribution_release/governance/planner.py:343` anchors at line start with only `export` and leading whitespace tolerated. Anything else that assigns the same names is invisible, and the *last* assignment wins at runtime. Verified against coherently resealed bundles (installer bytes rewritten, index row `sha256`/`size_bytes`, `plan.installer_sha256`, `desired_release.installer_sha256`, and the qualification-index plan reference all recomputed):

```
semicolon-prefixed-channel   `true; APP_CHANNEL=beta`            ACCEPTED exit=0
readonly-channel             `readonly APP_CHANNEL="beta"`       ACCEPTED exit=0
eval-channel                 `eval 'APP_CHANNEL=beta'`           ACCEPTED exit=0
and-chained-channel          `true && APP_CHANNEL=beta`          ACCEPTED exit=0
if-block-channel             `if true; then APP_CHANNEL=beta; fi` ACCEPTED exit=0
```

Each of these resolves the channel to `beta` under the installer's actual interpreter (`#!/usr/bin/env sh`, `generate_version_installer.sh:130`), confirmed directly:

```
true; APP_CHANNEL=beta                   -> beta
readonly APP_CHANNEL="beta"              -> beta
eval 'APP_CHANNEL=beta'                  -> beta
true && APP_CHANNEL=beta                 -> beta
if true; then APP_CHANNEL=beta; fi       -> beta
```

In every case the planner emits a canonical plan whose `installer_sha256` is presented as evidence that the installer embeds the candidate stable version and channel. This is the sole binding between the published installer digest and candidate identity — it exists because pass-1 #6 found `installer_version` was a self-attested echo. The pass-4 remediation was fitted to the four probes that were named rather than to the defect class.

No false positive: the actual generated installer is accepted. I generated one through the governed path (`build_release_artifacts.sh` → `generate_version_installer.sh --version 0.1.0`) and ran `validate_installer_identity` on it directly:

```
real generated installer: ACCEPTED (correct, no false positive)
wrong version:            rejected
```

Note the real installer contains 14 further lines mentioning `APP_VERSION`/`APP_CHANNEL` (expansions at `:246-247,515-545,686`), none of which are assignments — so a stricter check must still discriminate assignment from use.

Fix: stop trying to parse arbitrary shell. Bind the digest to the governed producer instead — in the assemble job (or the planner), regenerate the installer from `generate_version_installer.sh` at the pinned `source_commit` with the plan's version and target digests, and require byte equality with the transported artifact. That closes the class rather than the next five probes. If a textual gate is retained as a cheap pre-check, add the five forms above as negatives.

---

### P3 — Test coverage

**3. The ordered-claims contract is not exercised by any fixture.**
`qualification_fixture.py` emits a single claim (`direct_crate_fixture`), so `planner.py:146` (`advertised_claim_ids != claim_ids`) can never fail in the suite — reversing a one-element list is a no-op. I confirmed the check itself is correct by injecting a second claim: reversed order → `exit 2 $.rust_interop.advertised_claim_ids: must exactly match the ordered stable support claims`; correct order → accepted. Production carries 23 ordered ids, so the ordering semantics matter. Add a second fixture claim and an order-reversal negative.

---

### Broad adversarial sweep — no further defects found

| Dimension | Probes | Result |
|---|---|---|
| Installer identity | 15 forms (trailing unquoted, `export`, tab/space indented, duplicate canonical, wrong version, CR suffix, `readonly`, `eval`, `;`-chained, `&&`-chained, `if`-block, `declare`, `printf -v`, dynamic `eval "$n=..."`) | 6 rejected, 5 real evasions → P1 #2 (`declare`/`printf -v` are bash-only, not reachable under `sh`) |
| Non-UTF-8 evidence | release profile (in-checkout, committed), aggregate checksums, installer, checksum sidecar, claims file, Rust result, documentation report, sysroot-in-archive | 7 governed exit 2 without traceback; sysroot → exit 1 traceback → P1 #1 |
| Filesystem custody (planner) | symlinked container→outside, symlinked file→outside, symlinked file→inside root, symlinked `--artifact-root` | 3 rejected; symlinked root correctly accepted |
| Filesystem custody (collector) | symlinked container→outside, symlinked file→outside, symlinked artifact root | 2 rejected; symlinked root correctly accepted |
| Release-report binding | `stable-candidate` suite dropped, `qualification` suite dropped, `result_artifacts` row dropped, submodules forged, non-canonical bytes | 5 rejected |
| Artifact-index custody | expiry in the past, run-attempt drift, forked repository, `overwrite: true`, duplicate JSON key | 5 rejected |
| Rust claims / results | duplicate claim id, `schema_version` drift, non-UTF-8 claims, non-UTF-8 result, 2-claim order reversal | 5 rejected |
| Editor / docs / target reports | `submodule_commit` forged, `installer_version` drift (resealed), non-UTF-8 docs report | 3 rejected |
| Aggregate checksums | sysroot row dropped (resealed) | rejected |
| Source identity | dirty checkout (untracked), plan submodules dropped, plan version drift | 3 rejected |
| GA vs normal semantics | predecessor on `ga-activation`, normal predecessor version drift, rollback-target mismatch, `normal` on first-GA index, `ga-activation` on populated index | 5 rejected; `normal` fixture bundle correctly accepted |
| Output custody | `--out` inside checkout (exit 2, nothing written), repeat write to an existing path | 2 rejected |
| Schema/validator parity | `qualification_artifact_index.schema.json` vs `artifact_index.py` | validator strictly stronger; 20-id exact set matches `minItems/maxItems: 20`; `artifact_contract` pins kind/target/upload-suffix/file-name per id |
| Workflow contract | permissions `contents:read`/`actions:read` only, 5-job topology, exact matrix, 4 uploads × `retention-days: 30` + `overwrite: false`, no `environment`, `github.sha == source_commit` gate, collector `head_sha` gate, `--locked` builder pin | all pinned and passing |
| Capability naming | `build_release_artifacts.sh`, `demos/stable_candidate_qualification_demo.sh`, `artifact_stable_candidate_generation.sh`, `release_qualification_workflow_contract.sh` — no phase/milestone identifiers | clean |
| Tracking | pass-4 record committed; evidence list updated. Two checklist boxes legitimately open (no PR yet, docs/VSIX materialize in 40.4) | acceptable, except the inaccurate "sysroot text decode" claim at `:236-238` |

### Validation performed (read-only; no repository files modified)

| Check | Result |
|---|---|
| `areas run --area distribution_release --suite qualification` | pass — 8 self-tests, 1 variant, 30.3 s |
| `areas run --area distribution_release --suite full` | pass — 43 variants, 0 failures |
| `areas run --area distribution_release --suite evidence-custody` | pass |
| `cases/release_qualification_workflow_contract.sh` | PASS |
| `cases/artifact_stable_candidate_generation.sh` | PASS |
| `scripts/check_file_size_guardrails.py` | PASS (2857 files, limit 900) |
| `scripts/check_hir_maintainability_guardrails.py` | PASS |
| Governed installer generated end-to-end and validated | accepted (no false positive) |
| Profile registration | `distribution_release:qualification` in merge/nightly/release; `rust_interop:stable-candidate` in create-pr/merge/nightly/release |

Not run: `scripts/run_all_tests.sh --profile create-pr`/`merge`, the GitHub-hosted matrix jobs, and `demos/stable_candidate_qualification_demo.sh` (it requires a clean checkout and the worktree carries the untracked pass-5 review file; its planner path is the one exercised end-to-end by `test_materialized_planner_contract`). `crates/sifr/src/self_update_receipt.rs` reviewed by inspection.

---

**CHANGES_REQUESTED**
