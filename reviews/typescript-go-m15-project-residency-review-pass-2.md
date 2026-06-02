The user hasn't approved writing the review file. I'll present the findings directly in the chat.

## M15 Pass-2 Review Findings

Verifying pass-1 findings (`reviews/typescript-go-m15-project-residency-review-pass-1.md`) against the working tree.

### Pass-1 finding resolutions

**F1 — watcher wipe on overlay close — RESOLVED.** `crates/sifr_frontend/src/workspace_residency.rs:170-176`. Renamed to `release_open_file_project`; only filters projects matching the closed path. The blanket `watchers.retain(... !SeenFile)` and `configs.retain(...)` calls are gone.

**F2 — `remove_overlay` skips residency refresh — RESOLVED.** `crates/sifr_frontend/src/workspace_session.rs:378-379`. `release_open_file_project` is now followed by `self.refresh_residency()`, which re-derives watchers/configs from current state.

**F3 — phantom config entries pruned on close — RESOLVED.** `workspace_residency.rs:178-192`. `or_insert` uses `ref_count: 1`, the close path no longer prunes configs, and `register_watch_path(Config)` runs up-front.

**F4 — `pending_reload` dropped on reload — RESOLVED.** `workspace_residency.rs:108, 178-188, 349-364`. New `pending_config_reloads: Vec<SourcePath>` field survives `refresh_after_reload`; `register_config` consults it to re-apply `pending_reload` after re-registration. (Note: there's no drain API yet — consistent with M15's contract that `reload()` doesn't actually re-read `sifr.toml`. Worth flagging for the milestone that adds real config reload.)

**F5 — ref_count semantics — PARTIALLY ADDRESSED, acceptable.** `workspace_residency.rs:381-396`. ref_count still counts mentions within one reload pass (because `watchers` map is cleared at the top of `refresh_after_reload`). Dedup is the user-visible promise and is now test-covered (`workspace_session_tests.rs:212-216`). No code comment explains ref_count semantics, but matches the doc's "ref counts" wording.

**F6 — `FailedLookup` parent-directory glob — RESOLVED.** `workspace_residency.rs:405-407`. Returns `{parent}/**`, falling back to bare path only when there's no parent.

**F7 — duplicate rejection reasons — RESOLVED.** `workspace_residency.rs:235-247, 418-422`. New `push_rejection` helper dedupes. Variant still unit-only (no path info), but pass-1 allowed "dedupe or extend."

**F8 — extra-source rejection — RESOLVED.** `workspace_residency.rs:74-81, 239-247`. New `SifrBuildInfoRejection::ExtraSource` variant; `verify_build_info` iterates `candidate.sources` and rejects when not in the active source map.

**F9 — static `snapshot_package_config_identity` — RESOLVED (comment-only).** `workspace_session.rs:486-488`. Code comment flags the handoff for the future live-config-reload milestone.

**F10 — watcher-survival test — RESOLVED.** `workspace_session_tests.rs:147-217` (`closing_one_overlay_preserves_other_overlay_watcher`). Opens project with two overlays, closes one, asserts (a) the other overlay's watcher survives, (b) closed-overlay project gone, (c) surviving-overlay project still present, (d) dedup invariant. Without F1+F2 fixes, watchers would either be wiped or never rebuilt — test would fail.

**F11 — pending-reload lifecycle across reload — RESOLVED.** `workspace_session_tests.rs:249-258`. Test calls `session.reload()` after `mark_config_pending_reload` and asserts `pending_reload` survives.

**F12 — dedup test — RESOLVED.** `workspace_session_tests.rs:212-216`. Asserts `BTreeSet`-of-globs length equals the Vec length on `residency.watchers`.

**F13 — build-info rejection coverage — RESOLVED within pass-2 scope.** `workspace_session_tests.rs:304-364` (`build_info_rejects_missing_extra_package_and_unloaded_inputs`) covers `MissingSource`, `ExtraSource`, `PackageConfigMismatch`, `WorkspaceNotLoaded`. Combined with `build_info_is_verified_against_current_workspace_fingerprints` (which covers `Verified` + `SourceHashMismatch`), five of six variants are exercised. `CompilerFingerprintMismatch` remains unexercised — informational only; the pass-2 ask enumerated "missing/source mismatch/extra/package/unloaded."

**F14 — substring-only guardrail — UNCHANGED.** `verification/tooling/check_typescript_go_m1_guardrails.py:404-431`. Pass-2 asks for accuracy; every pinned symbol exists in `workspace_residency.rs`. Hardening (asserting use sites, not just type names) is still a future LOW item.

**F15 — tracker/doc status — ALIGNED.** `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:24` shows `in progress | pending`; `internal_docs/typescript_go_architecture_transfer_m15_project_residency.md:3` shows `Status: in progress`. Reminder for merge: per M14, both flip to `merged` + PR link in the same commit.

**F16 — empty review files — partially addressed.** `…m15-…pass-1.md` is populated. The other untracked 0-byte review files (`m0-…pass-2`, `m1-…pass-1`, `m2-…pass-5`, `m8-…pass-2`, `m9-…pass-{1,2,3,5}`) remain empty; they are unrelated to M15 and look like leftover scaffolding. Recommend a housekeeping pass.

### Additional spot checks

- `verify_build_info` Verified-path retains generated artifact (`workspace_residency.rs:251-259`); the Verified-then-Rejected test asserts `build_info` is cleared on rejection (`workspace_session_tests.rs:295-301`).
- `refresh_after_reload` order: identity → overlays → dependencies → source_map → module_graph → retained stdlib roots → retained generated artifacts; dedup means later passes append reasons to existing entries.
- Minor doc wording nit: `…m15_project_residency.md:28-29` reads "Closing an overlay releases open-file residency and watcher/config entries that are no longer retained" — close itself doesn't prune watchers/configs; `refresh_residency` rebuilds them from current state. The end-state matches the wording but the mechanism described is slightly off. Not blocking.

### Residual informational items
- `pending_config_reloads` has no drain API; consistent with current contract, flag for the milestone that introduces real config reload.
- `CompilerFingerprintMismatch` rejection variant unexercised; defensive against future fingerprint divergence.
- Unrelated empty review files want a housekeeping sweep.

---

**SATISFIED.**

All pass-1 correctness fixes (F1–F4) and the high-priority test gap (F10) are in. The pass-2 scope items the user enumerated — overlay-close not wiping watchers, `remove_overlay` re-derives residency, `mark_config_pending_reload` retain+watch+reload-survival, watcher ref_count/dedup test coverage, failed-lookup parent-dir glob, build-info rejection cases (missing/source mismatch/extra/package/unloaded), and doc/guardrail accuracy — are all verified.
