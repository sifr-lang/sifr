I'll present the findings directly. Below are the M15 review findings (file/line references against the working tree).

## Findings — M15 (Project Residency, Watchers, Build Info)

### F1 — `release_open_file` wipes every `SeenFile` watcher, not just the closed overlay's — **High / correctness**

`crates/sifr_frontend/src/workspace_residency.rs:168-177`

```rust
pub(crate) fn release_open_file(&mut self, path: &Path) {
    let path = SourcePath::new(path.to_path_buf());
    self.projects.retain(|project| {
        !(project.kind == ProjectResidencyKind::OpenFileOwner
            && project.root.as_ref() == Some(&path))
    });
    self.watchers
        .retain(|_, watcher| !watcher.reasons.contains(&WatchRegistrationReason::SeenFile));
    self.configs.retain(|_, config| config.ref_count > 0);
}
```

The `path` argument is only used for the `projects` filter. The watcher filter blanket-deletes every entry whose reasons contain `SeenFile` — which on a freshly reloaded project is essentially all overlay-, dependency-, source-map-, module-graph-, and entrypoint-derived watchers (since `register_identity` registers the entrypoint as `SeenFile` at `workspace_residency.rs:331`). Closing a single overlay therefore drops the watcher set the host needs to keep observing the *other* open files. The M15 doc promises "watcher/config entries that are no longer retained" — the code deletes a whole class.

### F2 — `remove_overlay` does not refresh residency, so stale watcher state persists — **High / correctness**

`crates/sifr_frontend/src/workspace_session.rs:373-387`. After `release_open_file`, no `refresh_residency` call follows; only `reload()` (`workspace_session.rs:310`) re-derives the watcher map. Between the two, the snapshot reports the post-wipe state (F1) as the authoritative watcher set.

### F3 — `mark_config_pending_reload` can create phantom config entries that get pruned by `release_open_file` — **Medium / correctness**

`workspace_residency.rs:179-187`. The `or_insert` path inserts with `ref_count: 0` and no watcher. The next `release_open_file` call (`workspace_residency.rs:176`, `configs.retain(|_, config| config.ref_count > 0)`) drops it silently, losing the `pending_reload` flag. The single test (`workspace_session_tests.rs:140-169`) uses the path `register_identity` had already registered, so the bug doesn't surface.

### F4 — `pending_reload` flag is silently dropped on the next `reload()` — **Medium / behavioral**

`workspace_residency.rs:127` (`self.configs.clear()`) inside `refresh_after_reload`, followed by re-registration with `pending_reload: false` (`register_config` at line 335). Since M15's `reload()` doesn't actually re-read `sifr.toml`, a pending config change becomes invisible after any unrelated reload. The doc doesn't specify the lifecycle, and no test covers it.

### F5 — Watcher `ref_count` does not track lifetime; it counts hits per reload pass — **Medium / design**

`workspace_residency.rs:128` clears `self.watchers` at the top of `refresh_after_reload`; `register_watch_path` (`workspace_residency.rs:365-380`) then increments on every subsequent call. A single file usually appears in overlay + source map + module graph + a `FileRead` dependency, so its ref_count lands at ~4 immediately after `open_project`. The snapshot value is "multiplicity of mentions during this reload", not a retention count, and the only consumer of decrement-based release (the watcher cleanup in `release_open_file`) bypasses the counter anyway. Either rework to true ref-counting or document the meaning.

### F6 — `watch_glob` collapses `SeenDirectory` / `PackageRoot` / `StdlibRoot` into one string and treats `FailedLookup` as an exact-file watch — **Low / design**

`workspace_residency.rs:383-394`. Three distinct semantic categories produce the same `"<path>/**"` glob, so the host has to demux via the `reasons` vec; `FailedLookup` gets a bare path with no glob, which won't fire on parent-directory creation events for the missing file. Worth confirming against the intended watcher policy.

### F7 — `verify_build_info` aggregates duplicate rejection reasons with no file-level info — **Low / observability**

`workspace_residency.rs:223-233` pushes `SourceHashMismatch`/`MissingSource` per file, but the enum is unit-variant. A 5-file mismatch yields `[SourceHashMismatch, SourceHashMismatch, ...]` and no information about which file. Either dedupe or extend the enum with the offending path.

### F8 — `verify_build_info` is one-sided: extra candidate sources are silently tolerated — **Low / behavioral**

`workspace_residency.rs:223-232` iterates `source_map.files` only. A `.sifrbuildinfo` listing files no longer in the workspace is `Verified`. Conservatively correct (won't accept stale info that *omits* current files), but the doc's "source hashes match the active workspace" reads stricter than the implementation.

### F9 — `WorkspaceSession::reload()` uses static `snapshot_package_config_identity` — **Low / pre-emptive constraint**

`workspace_session.rs:485-493` passes `self.snapshot_package_config_identity.as_ref()` into `refresh_after_reload`. That field is set once in `new()` and never updated. Fine today, but when a later milestone makes `reload()` pick up config changes, this line will silently use the stale identity. Add a code comment or thread the live identity.

### F10 — Test does not exercise F1 (watcher wipe) — **High / test adequacy**

`workspace_session_tests.rs:118-138` removes an overlay and never inspects `removed_snapshot.residency.watchers`. Add a two-overlay test that asserts the other overlay's `SeenFile` watcher and the entrypoint's `SeenFile` watcher survive a single `remove_overlay`.

### F11 — No test for `pending_reload` lifecycle across `reload()` — **Medium / test adequacy** (couples F4)

`workspace_session_tests.rs:140-169` snapshots right after marking; it never calls `reload()` to pin the intended behavior.

### F12 — No test that watcher dedup actually dedupes — **Low / test adequacy**

Existing assertions check "*some* watcher with reason X". A regression turning the BTreeMap into a Vec would not be caught.

### F13 — Three of four `SifrBuildInfoRejection` variants are unexercised — **Low / test adequacy**

`workspace_session_tests.rs:171-213` covers `Verified` and `SourceHashMismatch`. `CompilerFingerprintMismatch`, `PackageConfigMismatch`, `MissingSource`, and `WorkspaceNotLoaded` are absent. The guardrail script pins these names as required vocabulary, so a regression that returns the wrong variant won't fail any test.

### F14 — Guardrail script is substring-only — **Low / docs/guardrail accuracy**

`verification/tooling/check_typescript_go_m1_guardrails.py:404-431` validates by `"<symbol>" in residency`. A regression that declared types but stopped using them, or aliased them, would still pass. Consider asserting at least one use site (`fn verify_build_info(.. candidate: SifrBuildInfoCandidate ..)`) instead of pure type-name presence.

### F15 — Tracker/doc status alignment — **Tracking accuracy**

`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:24` (`pending`) and `internal_docs/typescript_go_architecture_transfer_m15_project_residency.md:3` (`Status: in progress`) are consistent now. Flag for the merge step: both must flip together; M14 above is the template.

### F16 — Empty review file in tree — **Housekeeping**

`reviews/typescript-go-m15-project-residency-review-pass-1.md` is a 0-byte untracked file. Either delete it or populate it with this review. Same situation for the other untracked review files in the diff — they're 0 bytes and unrelated to M15.

## Summary

High-priority correctness: **F1**, **F2**, **F3**, **F4**.
High-priority test gap: **F10** (would have caught F1).
Medium: **F5**, **F11**.
Lower: **F6**–**F9**, **F12**–**F16**.

Not SATISFIED — recommend resolving F1, F2, F3, F4, and F10 before merge. I drafted the same content as a review file at `reviews/typescript-go-m15-project-residency-review-pass-1.md` but the Write call was denied; let me know if you want me to save it.
