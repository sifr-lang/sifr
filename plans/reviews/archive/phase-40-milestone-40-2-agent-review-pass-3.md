## Phase 40 · Milestone 40.2 — Full re-review (read-only)

**State confirmed:** `HEAD = 38f79f5043222b7075b05276148cf60570a0a45d`; tracked tree clean (only untracked artifact is `plans/reviews/active/phase-40-milestone-40-2-agent-review-pass-3.md`). Reviewed the whole `e0cc278a5..HEAD` range (4 commits, 64 files).

### Independently re-run at head
| Gate | Result |
|---|---|
| `verification/areas/distribution_release/runner.py` | **variants=102, failures=0** |
| `cargo test -p sifr --bins self_update` | **48/48** (`self_update_metadata` 15/15) |
| `cargo clippy --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `check_hir_maintainability_guardrails.py` | PASS (`self_update_metadata.rs` 886 lines) |
| `demos/stable_self_update_demo.sh` | forced beta→stable, stable→stable, no-op, workflow-input check — all pass |

### Prior findings 1–8 — re-verified closed
Entrypoint identity conflict (`generate_dispatchers.sh` `ENTRYPOINT_NAME` gate), byte-preflight of the site workflow before any release mutation (ordering asserted in `preview_release_workflow_yaml_parses.sh` and `site_release_workflow_contract.sh`), read-only validate/build jobs with a caller `contents: write` ceiling counted exactly twice, immutable `sifr-release-site-m40-2` tag resolved twice and polling bound to `head_sha`+`display_title`+`created_at`, `--force` on exact cross-channel pins (`resolve_update_plan` now derives `switches_channel` from `resolved_channel`), post-GA alpha/beta publication preserving `channels.stable`, the distinct `sifr-site-publication-binding-v1` contract, active-status installer extraction, `release-publication.yml` in the rc sweep, and paginated time-filtered polling. All confirmed in code, not just in tests.

Scope/DoD coverage is complete: fail-closed active-status installer digest in both the dispatcher and the Rust runner (digest checked *before* `make_executable`, negative test asserts the installer never ran), no extraction/replacement in `self_update_runner.rs`, write-once version assets with `--clobber` used exactly once for `channels.json`, max-generation allocation with write-once snapshot published *before* index replacement, and read-only local planning.

---

## Findings

**1 — Medium · docs drift · `demos/preview_release_lifecycle/README.md:3,10,39,41`**
This first-party demo still prescribes `--real-run`, `--mutation-mode local`, and `--binary`, and describes dry-run output (`plan_sha256`, `dry_run_side_effects=none`, artifact names, "stable entrypoint status") that no longer exists. Every command in it now hard-fails at head — I ran the documented dry-run and got `create-new-version: --site-repo must name a Git checkout` (exit 2); the "Mocked Real Run" hits `local mutation and artifact modes are removed`. `internal_docs/distribution_pipeline.md` and `.cursor/commands/create-new-version.md` were updated by this diff; this one was missed.

**2 — Low/Medium · schema parity · `verification/areas/distribution_release/governance/release_plan.py:279-280`, `schemas/stable_release_plan.schema.json:125`**
This diff declares `sifr-lang/sifr-website` canonical (phase plan lines 33/192/262, issue tracker, `release-publication.yml`, `fixtures/site_release_contract.json`), and specifically rewrote the phase text binding "the exact `sifr-lang/sifr-website` base commit" into the stable release plan. The validator and JSON schema that *enforce* that binding still hard-require the constant `sifr-lang/sifr-blog-website` (mirrored in `selftest.py:173`, `schema_contracts.py:239`). A stable release plan naming the canonical repository would be rejected; only the redirecting legacy name validates. Not exercised in 40.2 (stable publication lands in 40.5), but it is a spec-vs-enforcement contradiction created by this change set.

**3 — Low · dead/redundant code · `crates/sifr/src/self_update_metadata.rs:68`, `:330`, `:371`+`:376`**
`let Some(prerelease) = prerelease else { return Err(...) }` at line 68 is unreachable — the `if prerelease.is_none()` early return two lines above already handles that case. Separately, `fn installer_sha256` (line 330) is a pure alias for `resolve_exact`, and the `TargetRequest::Version` arm calls `resolve_exact(&version)?` (line 371) and then `installer_sha256(&target_version)?` (line 376) on the same value — one redundant lookup and one redundant indirection.

**4 — Low · release provenance · `.github/workflows/release-publication.yml:329`**
`gh release create "${VERSION}" release-assets/*` omits `--target "${SOURCE_COMMIT}"`, so the created git tag points at the default branch tip rather than the validated source commit. Assets and the governed index record are unaffected (both bind `source_commit`, and neither the dispatcher nor self-update resolves through the git tag), but `git checkout <version>` can yield a tree that is not the released source if `main` advances during the run. This is carried over from the base workflow, not introduced here, but it sits inside the step this diff rewrote and cuts against the milestone's exact-pinning contract.

### Not findings (examined, judged sound)
- `create_new_version.sh` requiring `DEFAULT_CHANNEL="stable"` in the checked-in site dispatchers while the workflow deploys `beta` under `ga_status: preview` is intentional and documented (`internal_docs/distribution_pipeline.md`): the repo copy is canonical-stable, the deployed copy is GA-derived and regenerated at publication.
- Residual tag-move window between the pre-dispatch tag re-resolution and the `workflow_dispatch` POST is unavoidable (the API accepts only branch/tag refs); polling binds `head_sha` to the pinned commit, so a moved tag fails loud rather than silently correlating.
- `release_installer_sha_from_metadata`'s exact key-order regex fails closed on withdrawn records (the extra `incident_id` key breaks the match) — verified by `install_withdrawn_stable_rejected.sh`.
- `install_metadata_url_injection_ignored.sh` is somewhat tautological (the env names it sets were never read), but the generator-owned-constant property it asserts is genuinely enforced.

---

NOT APPROVED
