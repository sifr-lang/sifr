## Scope and state

- HEAD `f28b9d8fa0903f488f6636c54cb31de2038f6dfd` confirmed; tracked tree clean (only untracked `plans/reviews/active/phase-40-milestone-40-2-claude-opus-review-pass-6.md`).
- `git diff --check e0cc278a5..HEAD` clean — the two pass-5 EOF blank lines are gone.
- Diff reviewed in full: 71 files, +3075/−1198.

## Pass-5 closure reconfirmation (all intact)

- Ruleset pinning: `.github/workflows/release-publication.yml:104-121` and `:524-541` both assert `id`/`target`/`enforcement`/`updated_at`/exact include/empty exclude/`bypass_actors == []`/`current_user_can_bypass == "never"`/`rules == ["deletion","update"]`, before mutation and again immediately before dispatch, plus tag→`site_base_commit` resolution at both boundaries and a byte-level `SITE_WORKFLOW_SHA256` check (`:155-164`) ordered before the first publish step (asserted by `cases/site_release_workflow_contract.sh`).
- Bounded wall-clock poll with cancellation: `release-publication.yml:588-683`. `timeout --foreground "${remaining_seconds}s"` keeps every query inside the 1200 s deadline; status 124 breaks; non-timeout failures now increment `poll_query_failures` and abort after three consecutive failures with a distinct terminal error (`:621-636`); the success counter resets on a good query (`:637`); matching stays jq-first on title/head_sha/event/created_at (`:638-652`); a discovered non-terminal run is cancelled on exit (`:673-677`). Deadline and query-failure paths both exit 2.
- Exact stable pin rejected under preview metadata in Rust: `self_update_metadata.rs:314-332` (`resolve_exact` gates on `ga_active`), covered by `self_update_metadata_exact_pin_tests.rs:598-610`.
- Only the site Actions secret is passed to the steps that need it (`:73`, `:507`, `:591`); `GH_TOKEN` for the home repo stays `github.token`.
- No early-exit snapshot pipeline: `:263-268` feeds the loop via process substitution and `:439-451` re-checks snapshot-name uniqueness before write-once upload; generation is allocated as max(current, all snapshots)+1 with name/payload agreement enforced (`:250-262`).
- Canonical site repository parity: `sifr-lang/sifr-website` now consistent across `release_plan.py:279`, `schema_contracts.py:239`, `selftest.py:173`, `stable_release_plan.schema.json:125`; remaining `sifr-blog-website` mentions are explicit historical/redirect notes.
- Redundant exact-pin lookup removed; force semantics unified on `resolved_channel != receipt_channel` (`self_update_metadata.rs:273-285`), so exact cross-channel pins now require `--force`.
- Release tag targeting explicit: `gh release create ... --target "${SOURCE_COMMIT}"` with post-publish tag→commit verification and byte-for-byte re-download comparison (`:373-409`).

## Independent re-review of the full milestone diff

No blocking or actionable finding. Verified in particular:

- **Write-once/replace-only invariants**: existing release and existing tag both rejected pre-mutation (`:131-154`); asset set is exactly the 4 archives + 4 checksums + installer, diffed against `find` output (`:195-217`); snapshot uploaded before `channels.json`, `--clobber` used only for `channels.json`, and the activated index digest is re-verified after upload (`:453-470`).
- **Lease integrity**: `concurrency: sifr-release-index`, `cancel-in-progress: false`, plus generation+digest re-validation immediately before snapshot write (`:429-438`).
- **Input hardening**: channel/version agreement, 40-hex `source_commit` equal to the checked-out HEAD, and ancestry to protected `origin/main` (`:76-99`).
- **Dispatcher security** (`generate_dispatchers.sh`): metadata/installer URLs are generator-owned constants (env-override attempt proven ignored by `install_metadata_url_injection_ignored.sh`); canonical single-line schema-v2 shape enforced; `stable` resolution requires `ga_status: active`; the installer SHA-256 from the governed record is verified before `chmod +x`/execution; the record regex requires `"status":"active"`, so withdrawn records (which carry `incident_id`) can never match.
- **Rust metadata/runner**: `PreviewVersion::parse` accepts stable SemVer and still rejects `rc`, 4-part cores, empty/garbage prereleases; `ChannelMetadata::parse` requires exactly {alpha,beta} under `preview` and exactly {alpha,beta,stable} under `active`, validates every release record's field set, hex digests, and channel/status agreement; `validate_installer` hashes the download and fails closed before execution (`self_update_runner.rs:189-197`), proven by `rejects_installer_digest_mismatch_before_execution` (installer never runs). Channel rank ordering (stable > beta > alpha) is consistent between Rust and `generate_version_installer.sh` (rank 4 default after the `rc` rank removal).
- **`rc` removal** is complete across build, installer, trigger, dispatcher, and workflow surfaces and is guarded by `release_surfaces_reject_rc.sh`.
- **Docs/tracking**: `internal_docs/distribution_pipeline.md`, `.cursor/commands/create-new-version.md`, and the preview-lifecycle README match the shipped read-only planner and sole mutation authority; the phase/issue docs record both site PRs, the pinned commit, ruleset id, and the pass 1–5 history. The `index`-defaults-stable vs. live-beta divergence is explicitly documented (`distribution_pipeline.md:23-28`) and matched by `site_release_contract.json`'s `default_channel_by_ga_status`.
- **Guardrails**: every touched hand-maintained file is under 900 lines (largest: `selftest.py` 889, `self_update_metadata.rs` 882).

Validation I re-ran on this exact HEAD: `cargo test -p sifr self_update` → 49/49 passed; 27 targeted distribution cases (all new/renamed stable, checksum, withdrawn, legacy-shape, injection, entrypoint, site-facts, site-contract, workflow-YAML, rc, and create-new-version cases) → 27/27 passed.

## Informational (non-blocking, no action required for this milestone)

1. `docs/self_update.md:32,41,47-52` and `docs/cli_command_semantics.md:58-61` still describe self-update as alpha/beta-only with stable gated. This stays operationally true while the live index is `ga_status: preview` (a `--channel stable` request fails at resolution), and public GA documentation is explicitly milestone 40.4 scope in `plans/phases/40_...md`. It must be updated before GA activation.
2. `cases/release_surfaces_reject_rc.sh:31` depends on `rg`; if ripgrep is absent the `if rg ...` guard takes the pass branch. Every other case in the area uses `grep`.
3. `plans/issues/active/phase-40-stable-channel-ga-execution.md:81` flips a milestone 40.1 checkbox (that milestone is already recorded complete/merged) while 40.2's own boxes at `:88-92` stay unchecked pending merge — bookkeeping only, consistent with checking boxes at merge time.

APPROVED
