# Phase 36 m36.3 Review — AnalysisHost, Symbol Index, Session Model

**SATISFIED**

Branch: `phase36-m36-3-analysis-host-symbol-index`
Review scope: working tree only, code-review severity.

---

## Overall Assessment

The implementation is solid. `AnalysisHost` properly owns the session model, routes through `sifr_frontend` for all semantic answers, enforces monotonic document versions, rejects stale snapshots, builds `SymbolIndex` from canonical frontend views, and exposes every Phase 36 query method. The split-brain guardrails, verification scripts, and test coverage are sufficient. No blocking findings.

---

## Findings

### F1 — Informational: `SymbolIndexEntry::id` embeds revision, ensuring per-revision symbol identity

`host.rs:52–59` encodes `graph/source` revision into every `SymbolId`. A symbol will get a new identity after any source or graph change. This is the correct behavior — a symbol index is inherently tied to a revision. No action needed; the behavior matches how symbol indexes operate in comparable IDE toolchains (LSP servers typically rebuild the index on each change and don't provide cross-revision identity stability).

### F2 — Informational: no `FrozenSymbolIndex` or cross-snapshot symbol stability

`SymbolIndex` is rebuilt lazily on first access per-revision (`host.rs:395–416`). There is no mechanism to freeze a `SymbolIndex` alongside an `AnalysisSnapshot`. This is acceptable: the snapshot staleness check (`host.rs:488–503`) gates all queries that would use the symbol index. The `AnalysisSnapshot` methods (`snapshot.rs:468–485`) call `ensure_snapshot_current` before delegating, so stale snapshots can't return results derived from a stale symbol index. No action needed.

### F3 — Low: `DiagnosticExplanation::unavailable_reason` is always `None` in `explain_diagnostic`

`host.rs:374–378` constructs the explanation with `unavailable_reason: None` unconditionally. If a diagnostic code is not found in the workspace, the function returns `explanation` with `diagnostic: None` and `unavailable_reason: None`. This is technically correct (the diagnostic was simply not found, not that the explanation system is unavailable), but it may confuse callers who expect to distinguish "not found" from "system unavailable." The return type `DiagnosticExplanation` supports this distinction; the caller-side convention for handling `None, None` must be documented in `m36.4` when the full feature lands.

**File**: `crates/sifr_analysis/src/host.rs:378`
**Fix**: Clarify in `m36.4` that callers should treat `diagnostic: None, unavailable_reason: None` as "diagnostic not found in current workspace" and document the distinction explicitly.

### F4 — Low: `explain_diagnostic` iterates all workspace diagnostics on every call

`host.rs:368–378` calls `workspace_diagnostics()` on every `explain_diagnostic` call, which for large projects re-diagnoses every file. This is acceptable as a foundation (the contract doc says explain diagnostic enrichment lands in `m36.4`), but worth noting as a performance optimization target in `m36.4` when the feature is fleshed out.

**File**: `crates/sifr_analysis/src/host.rs:368`
**Fix**: In `m36.4`, add a diagnostic registry or indexed lookup rather than full re-diagnosis.

### F5 — Informational: `FrontendContext::index_for_module` panics on unknown module

`lib.rs:754–759` uses `unwrap_or_else(|| panic!(...))`. This is in an internal helper called exclusively after prior `module_for_file` checks that already validate the module exists. The panic cannot be triggered from the public `AnalysisHost` API because all public methods that accept `ModuleId`/`FileId` validate first. No user-path safety concern. This is acceptable as an internal invariant check; a `Result`-returning variant is not needed at this stage.

### F6 — Informational: `check_analysis_split_brain.py` allows `sifr_format::format_range` as a whitelisted snippet

`check_analysis_split_brain.py:30` explicitly permits `sifr_format::format_range` in `sifr_analysis`. This is the correct design — analysis owns the query API and delegates formatting to the formatter crate, which in turn uses `sifr_syntax`. This mirrors the `tooling_analysis.md` contract: analysis may call `sifr_format` but not the other way around for semantic queries.

### F7 — Informational: `generated_rust_preview` returns `unavailable_reason` sentinel

`host.rs:353–361` returns the full response shape with `rust: None` and `unavailable_reason` set to the m36.4 deferral message. This is the correct pattern — the query returns a structured response rather than an error, preserving the response schema while indicating the feature is not yet implemented. No action needed.

### F8 — Informational: test coverage for stale paths is comprehensive

The test suite in `host.rs:531–881` covers:
- `stale_document_version_is_rejected` — monotonic version enforcement (`host.rs:63–75`)
- `stale_snapshot_is_rejected_after_update` — snapshot staleness on `AnalysisSnapshot::diagnostics` and `AnalysisSnapshot::workspace_symbols`
- `single_file_session_updates_versions_and_invalidates_symbols` — symbol index refresh after document update
- `project_symbol_index_is_stable_for_workspace_queries` — stable results from same revision
- `all_editor_query_methods_expose_current_revision_metadata` — every query method returns metadata with the correct `AnalysisQueryKind`
- `completion_ranking_prefers_exact_then_prefix_then_substring` — completion scoring in `completion.rs:53–67`

These directly map to the m36.3 validation planning goals in the phase contract.

### F9 — Informational: verification scripts are correct

Both `check_analysis_snapshot_contract.py` and `check_analysis_split_brain.py` are correctly implemented:
- `check_analysis_snapshot_contract.py` runs the full `cargo test -p sifr_analysis` suite and asserts required test names appear in output
- `check_analysis_split_brain.py` uses line-by-line pattern matching (not regex) to catch forbidden imports/calls, with `sifr_format::format_range` properly allowlisted

Both scripts have negative self-tests that seed forbidden patterns and verify they are caught.

### F10 — Informational: `frontend_diagnostics` error flattening

`host.rs:523–529` extracts only the first diagnostic message when forwarding multiple `RenderedDiagnostic` errors. This is acceptable for the error surface — `AnalysisError` is a single message type. If `m36.4` needs multi-error forwarding, the error kind should be extended to carry a `Vec<AnalysisError>` or the host should use a different error transport. No action needed now.

---

## Residual Risks

1. **No cross-revision symbol identity**: `SymbolId` includes graph/source revision. Symbol identity changes across revisions. This is intentional and correct, but callers doing long-lived symbol tracking (e.g., LSP semantic token stream across edits) must re-resolve symbols after each snapshot invalidation. The `ensure_snapshot_current` gate in `AnalysisSnapshot` methods handles this for direct analysis users; LSP adapters must handle the re-resolution contract.

2. **`explain_diagnostic` re-diagnoses workspace**: As noted in F4, every `explain_diagnostic` call re-runs `workspace_diagnostics`. For `m36.4`, a cached diagnostic registry keyed by diagnostic code would be appropriate.

3. **No `FrozenSymbolIndex`**: While not needed today (staleness check gates all index usage), future multi-threaded or background query scenarios may need a frozen index. Design for this in `m36.5` when the scheduler adds background work lanes.

4. **Warm wall-time budget advisory**: The quick-lane validation report shows `wall_time=1121.72s` with advisory "warm wall-time budget exceeded." This is non-blocking but worth tracking as the test suite grows. The advisory was also present in `m36.1` and `m36.2` — it appears to be a systemic budget issue, not a regression from `m36.3`.

---

## Verification Evidence

| Check | Command | Result |
|---|---|---|
| Format/lint | `cargo fmt --check && git diff --check` | PASS |
| Python compile | `python3 -m py_compile verification/tooling/check_analysis_*.py` | PASS |
| Snapshot contract | `check_analysis_snapshot_contract.py && --self-test` | PASS |
| Split-brain guard | `check_analysis_split_brain.py && --self-test` | PASS |
| Cargo check | `cargo check -p sifr_frontend -p sifr_analysis` | PASS |
| Clippy | `cargo clippy -p sifr_frontend -p sifr_analysis -- -D warnings` | PASS |
| Tests | `cargo test -p sifr_frontend -p sifr_analysis` | PASS |
| Quick lane | `scripts/run_all_tests.sh --profile quick` | PASS |

---

## Conclusion

The implementation is ready for PR. All m36.3 scope items are implemented, validated, and documented. The residual risks are design observations, not defects, and are appropriately handled either by existing staleness gates or by explicitly deferring to `m36.4`. No changes are required.