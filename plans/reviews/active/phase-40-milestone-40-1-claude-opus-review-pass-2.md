## Review — Phase 40 / milestone_40_1, pass 2

Branch `codex/phase-40-milestone-40-1` (3 commits) vs `origin/main` after PR #3026 integration; 39 files, +3901/−136.

### Pass-1 findings: re-tested

| # | Pass-1 finding | Status |
|---|---|---|
| 1 | Vacuous digest-sensitivity test | **Still open** — see P1 below |
| 2 | `materialize_stable_plan` untested | **Fixed** — `test_materialized_planner_contract`, `test_planner_rejects_drift_cases`, `test_plan_digest_sensitivity` drive `plan-stable-release` end-to-end over a real fixture git tree |
| 3 | Raw `KeyError` on artifact ids | **Fixed** — `artifact_index.py:35-50,213-217` pins `EXPECTED_ARTIFACT_IDS`; probe: renamed id now rejected |
| 4 | Path traversal in `workflow_artifact_name` | **Fixed** at the string level (`artifact_index.py:162-166`, schema `pattern: ^[^/]+$`); residual symlink vector below |
| 5 | Report coverage `<` instead of exact | **Fixed** — `artifact_index.py:225-233`; extra report rejected |
| 6 | `installer_version`/`receipt_channel` echoes | **Fixed** — `planner.py:339-352` parses the aggregate installer's `APP_VERSION`/`APP_CHANNEL`; unit-probed accept/reject on all four cases. Transitively bound: plan `installer_version == version == APP_VERSION` (`release_plan.py:220-222`) |
| 7 | `"not-yet-materialized"` fallback | **Fixed** — `release-qualification.yml:164-166` fails closed |
| 8 | VSIX nondeterminism / unchecked `os.popen` | **Fixed** — `:167-169` requires exactly one VSIX; `:172-177` uses `subprocess.run(check=True)` |
| 9 | Bypassed governed build path / no `--locked` | **Fixed** — workflow uses `--cargo-build` (`:106`); `build_release_artifacts.sh:283` now `cargo build --locked --release` |
| 10 | Timing-dependent `assemble` inputs | **Fixed** — four explicit named downloads (`:215-237`) |
| 11 | Repository custody asserted | **Fixed** — `collect_qualification_artifacts.py:89-102,177` verifies `repository.full_name`, run id, attempt, event, and workflow name from the run API |
| 12 | Tracking not updated | **Fixed** — execution issue `:74-88,199-227`, pass-1 record populated |
| 13 | Rust boundary fallbacks | **Fixed** — `planner.py:430-475` pins one exact claims shape; the real `stable_support_claims.json` validates (23 ordered ids); `stable-candidate` registered in the manifest and in all four profiles |
| 14 | Missing capability demo | **Fixed** — `demos/stable_candidate_qualification_demo.sh` (executable, capability-named) |
| 15 | rc inventory disposition | **Fixed** — `stable_gate_inventory.json:31-45` |
| 16 | Dead branch in generated installer | **Fixed** — `generate_version_installer.sh:80-86,139` emits literal `APP_CHANNEL="stable"` |

Also fixed incidentally and worth noting: `release_report.py:345` changed `.strip()` → `.rstrip("\r\n")`, which repairs a real 40.0 bug — `git submodule status` output was previously left-stripped, truncating the first submodule's commit by one hex digit.

---

### P1 — Blocking

**1. `test_plan_digest_sensitivity` still cannot fail.**
`verification/areas/distribution_release/governance/qualification_selftest.py:568-604`. The test is now end-to-end (a real improvement over pass 1), but each variant calls `create_fixture_source(variant_root, variant=variant)`, which builds a **brand-new git repository** with a fresh commit. Two causes make the digest differ regardless of the input under test: `qualification_fixture.py:35-38` stamps `variant` into the submodule's `package.json` for every variant, and `create_fixture_source` sets no committer/author date, so even two identical fixtures get different commit SHAs. Verified:

```
same-variant commits equal? False 6f67206d4307... 37d23bc6e6c0...
control variant 'nochange' changes digest (should be False): True
```

A variant label that touches nothing at all passes the assertion. The DoD line *"Changing any commit, submodule, lockfile, version, target, artifact, sysroot, installer, or Rust-claim input owned by this milestone changes the fixture plan digest"* is therefore still unproven for `lock`, `target-artifact`, `sysroot`, `installer`, `rust-claims`, and `vsix` — the six variants that do **not** need a new source tree.

Fix: build one fixture source, then vary only `build_evidence_bundle(variant=...)` for those six; keep separate sources only for `source`/`submodule`/`lock`. Add a no-op control variant asserting the digest is **unchanged** — that control is what makes the other eight assertions mean something. (`version` needs no positive case; `test_planner_rejects_drift_cases` already rejects version drift, which is stronger.)

---

### P2 — Should fix before merge

**2. A symlinked upload container escapes the transported-artifact custody boundary.**
`verification/areas/distribution_release/governance/planner.py:239-243` builds `artifact_root / workflow_artifact_name / name` and checks `path.is_file() and not path.is_symlink()` — but `is_symlink()` inspects only the final component. The directory component is followed. Verified against a valid fixture bundle with the `-editor` container replaced by a symlink to a directory outside `--artifact-root`:

```
symlinked upload container -> 0 accepted
```

The planner hashed the VSIX and the editor qualification report from outside the artifact root and bound them as candidate evidence. This is the residual of pass-1 finding 4: the schema/validator now block traversal *in the index*, but not through the filesystem. Fix: `resolved = path.resolve()` and require `resolved.is_relative_to(artifact_root.resolve())`, or reject when any component of the container path is a symlink.

---

### P3 — Hardening and test isolation

**3. The index does not bind artifact `id` to its `target` or to its governed upload container.**
`artifact_index.py:193-233` checks id-set equality, per-kind target coverage, and container prefix/suffix independently, so a row's `id` can name one target while `target`/`workflow_artifact_name` name another. Three mutations accepted:

```
ACCEPTED: id<->target swap (id says binary-archive-aarch64-apple-darwin, target says x86_64-apple-darwin)
ACCEPTED: per-target report attributed to the assemble upload
ACCEPTED: installer attributed to a target upload
```

The planner fails closed downstream on digests and on missing files, so none of these is exploitable today — but `qualification-artifact-index.json` is the canonical custody record consumed at publication, and this is the same exactness class the 40.0 passes 1-5 chased. Fix: for the known ids, require `workflow_artifact_name == f"{prefix}{expected_suffix}"` and `target == id.rsplit("-", …)` derived from the id itself.

**4. The outside-the-repo output assertion is confounded.**
`qualification_selftest.py:400-417`: `missing_artifact.unlink()` at `:408` permanently invalidates `bundle`, and `:413` then reuses that same bundle for the in-repo output check. The non-zero exit proves the missing artifact, not the guard, and `inside_output.exists()` at `:414` is trivially false. The guard itself is correct — `release_governance.py:254-260` runs before `materialize_stable_plan`, verified with a *valid* bundle:

```
valid bundle + in-repo out -> 2 stable release evidence output must be outside the repository; wrote? False
```

Fix: use a fresh valid bundle for the guard case (and run it before the missing-artifact mutation).

**5. `retention_days` and `overwrite` are asserted constants, not derived custody.**
`collect_qualification_artifacts.py:178-179` writes `"retention_days": 30, "overwrite": False` literally, and `artifact_index.py:99-102` then validates those literals. The run API returns `created_at` and `expires_at`; retention could be derived and checked. The workflow-contract case pinning `retention-days: 30` / `overwrite: false` in the YAML is a real compensating control, so this is low — but it is the same "asserted, not verified" shape as pass-1 finding 11, which was fixed for `repository`.

---

### Validation performed (read-only; no repository files modified)

| Check | Result |
|---|---|
| `areas run --area distribution_release --suite qualification` | pass (8 self-tests, 1 variant, 18.4s) |
| `areas run --area distribution_release --suite representative` | pass (41 variants, 0 failures) |
| `areas run --area distribution_release --suite full` | pass (43 variants, 0 failures) |
| `areas run --area distribution_release --suite evidence-custody` | pass |
| `cases/release_qualification_workflow_contract.sh` | PASS |
| `cargo test -p sifr --bin sifr self_update_receipt::` | 16 passed, incl. `accepts_stable_receipt_for_read_only_version_evidence` |
| Real `stable_support_claims.json` → `stable_claim_ids` | accepted, 23 ordered ids (no fixture-vs-production divergence) |
| Collector output → checked-in JSON Schema | conforms (schema/validator agree) |
| `rust_interop` manifest suites vs `validate_rust_candidate_result` expectations | exact match, incl. both `stable-candidate` case ids; suite selected in create-pr/merge/nightly/release |
| Real generated installer identity lines vs `validate_installer_identity` | one `APP_VERSION="`, one `APP_CHANNEL="`; unit-probed good/beta/no-channel/wrong-version |
| Planner fail-closed probes (ref≠HEAD, installer channel, editor status fail, checksums row dropped, docs status fail) | all exit 2 with governed diagnostics, no tracebacks |
| Digest-sensitivity control variant | **accepted** — finding 1 |
| Symlinked upload container | **accepted** — finding 2 |
| Index id/target/container mutations | **3 accepted** — finding 3 |
| `check_file_size_guardrails.py` | PASS (2857 files, limit 900; largest touched 896) |
| `check_hir_maintainability_guardrails.py` | PASS |

Not run: `scripts/run_all_tests.sh --profile create-pr`/merge, `demos/stable_candidate_qualification_demo.sh` (it requires a clean checkout; the worktree carries an untracked pass-2 review file I was instructed not to touch — verified executable and `bash -n` clean instead), and the GitHub-hosted matrix jobs.

---

**CHANGES_REQUESTED**
