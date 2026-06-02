# M15 Pass-3 Review — Project Residency, Watchers, And Build Info

Final verification before PR/merge. Pass 1 (`reviews/typescript-go-m15-project-residency-review-pass-1.md`) was CHANGES_REQUESTED; pass 2 (`reviews/typescript-go-m15-project-residency-review-pass-2.md`) was SATISFIED. Since pass 2 the diff only touched documentation/status wording and added M15 vocabulary to the M1 guardrail script. This pass re-verifies that the SATISFIED finding holds.

## Re-verification of pass-2 confirmations

- **F1 / F2 (overlay close)** — `crates/sifr_frontend/src/workspace_residency.rs:170-176` still scopes `release_open_file_project` to projects matching the closed path; no blanket `watchers.retain` / `configs.retain`. `crates/sifr_frontend/src/workspace_session.rs:373-388` invokes `release_open_file_project` then `refresh_residency()` so residency is rebuilt from current state. Confirmed by `closing_one_overlay_preserves_other_overlay_watcher` (`workspace_session_tests.rs:147-217`).
- **F3 / F4 (config pending reload)** — `mark_config_pending_reload` (`workspace_residency.rs:178-192`) uses `or_insert` with `ref_count: 1` and front-registers the config watcher; `pending_config_reloads` (`:108`) is the durable list re-applied through `register_config` (`:349-364`) after `refresh_after_reload`'s `self.configs.clear()`. Confirmed by `config_registry_pending_reload_and_extra_watch_roots_are_snapshot_visible:249-258` (asserts survival across `reload()`).
- **F6 (FailedLookup glob)** — `watch_glob` (`workspace_residency.rs:399-412`) returns `{parent}/**`, with a bare-path fallback only when there is no parent.
- **F7 (dedupe rejections)** — `push_rejection` helper (`:418-422`) deduplicates variants.
- **F8 (extra-source)** — `verify_build_info:239-247` iterates `candidate.sources` and emits `ExtraSource` when missing from the active source map.
- **F9 (static identity)** — Comment at `workspace_session.rs:486-488` flags the live-identity handoff for the future config-reload milestone.
- **F10–F13 (tests)** — All five tests are present (`workspace_session_tests.rs:147, 219, 260, 304, 366` plus dedup invariant `:212-216`). Five of six `SifrBuildInfoRejection` variants are exercised; `CompilerFingerprintMismatch` remains unexercised (informational only).
- **F14 (guardrail substring-only)** — `verification/tooling/check_typescript_go_m1_guardrails.py:404-431` plus the new M15 stanza assert presence of the residency / build-info vocabulary. Same hardening note as pass 2: still substring-only by design.
- **F15 (tracker / doc status)** — `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:24` shows `in progress | pending`; `internal_docs/typescript_go_architecture_transfer_m15_project_residency.md:3` shows `Status: in progress`. Aligned. Both must flip to `merged` + PR link in the same commit at merge time (M14 template).

## Changes since pass 2

1. `internal_docs/typescript_go_architecture_transfer_m15_project_residency.md:28-29` — wording corrected. Previously: "releases open-file residency and watcher/config entries that are no longer retained". Now: "releases its open-file residency and re-derives watcher/config snapshot state from the remaining retained session inputs." Accurate to the mechanism (the close itself only drops the project entry; `refresh_residency` does the rebuild). Pass-2 nit resolved.
2. `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md` — M15 row added to the "must be visible" list (lines 184-188) and the future-milestone obligation (lines 207-210) reworded from "must update" to "updated".
3. `verification/tooling/check_typescript_go_m1_guardrails.py` — adds `validate_m15_residency_state` (`:404-431`) plus required-doc-snippet entries (`M15 updated`, `ProjectResidencyKind`, `WatchRegistrationReason`, `SifrBuildInfoCandidate`).
4. `internal_docs/architecture.md` and `internal_docs/frontend_query_architecture.md` — M15 paragraphs added; descriptions match the implementation.
5. `crates/sifr_frontend/src/lib.rs` — `workspace_residency` module declared and re-exported. Symbols used in tests (`SifrBuildInfoCandidate`, `SifrBuildInfoVerification`, `SifrBuildInfoRejection`, `SifrBuildInfoSource`, `CompilerFingerprint`, `ProjectResidencyKind`, `WatchRegistrationReason`) are all visible at the crate root.
6. `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:24, 148-156` — M15 tracker row and validation log added.

## Local validation re-run

- `cargo test -p sifr_frontend workspace_session` → PASS, 8 tests
- `cargo test -p sifr_frontend` → PASS, 42 tests
- `cargo clippy -p sifr_frontend -- -D warnings` → PASS
- `cargo fmt --check` → PASS
- `git diff --check` → PASS (no whitespace issues)
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` → PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` → PASS
- `python3 scripts/check_file_size_guardrails.py` → PASS

## Residual non-blocking items

- `pending_config_reloads` has no drain API. Consistent with M15's contract that `reload()` does not re-read `sifr.toml`; flag for the milestone that introduces live config reload.
- `CompilerFingerprintMismatch` rejection variant remains test-unexercised. Defensive against future fingerprint divergence; not part of the pass-2 scope ask.
- Guardrail script remains substring-only. The new `validate_m15_residency_state` adds explicit name-presence assertions but does not assert use sites. Same future-hardening note as pass 2.
- Untracked, zero-byte review files unrelated to M15 remain in `reviews/` (`m0-…pass-2`, `m1-…pass-1`, `m2-…pass-5`, `m8-…pass-2`, `m9-…pass-{1,2,3,5}`). Housekeeping nit; not a merge blocker.

## Merge-time reminders

- Flip the tracker row (`issues/…:24`) and the doc status (`…m15_project_residency.md:3`) to `merged` with the PR link in the same commit.
- Append the validation log entry for `scripts/run_all_tests.sh --profile quick` once the PR's CI-parity run completes — the M15 validation block currently lacks the quick-profile line that M14's row has.

---

**Verdict: SATISFIED.**

All pass-1 correctness regressions stayed fixed; the pass-2 doc-wording nit was addressed; the new guardrail vocabulary is in place and validated; the full sifr_frontend test suite and all local guardrails are green. No new correctness regressions or unstated acceptance gaps found. The phase can proceed to PR / merge.
