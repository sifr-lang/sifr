Based on my comprehensive review of the M1 implementation, here are my findings ordered by severity.

## Critical (block M1 PR)

### 1. Stale direct-read line number in inventory
**File:** `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:47`

`crates/sifr_package/src/imports/source_map.rs:254` is documented as a "directory and source-root read" but the actual line 254 in that file is `if path.is_dir() {` — a path-type check, not a direct filesystem read. The only direct read in `discover_modules_recursive` is at line 240 (`std::fs::read_dir(directory)`). The M1 checker at `verification/tooling/check_typescript_go_m1_guardrails.py:48` enforces this stale line number, so the guardrail will permanently embed the wrong reference.

### 2. Inventory misses four production direct reads
**File:** `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:35-46`

The inventory is incomplete and the checker does not enforce inventory completeness. The following production direct reads are missing:

- `crates/sifr_format/src/lib.rs:197` — `fs::read_dir` in `collect_sifr_files_inner`
- `crates/sifr_format/src/config.rs:109` — `fs::read_to_string` for canonical config read
- `crates/sifr_package/src/source/layout.rs:30` — `std::fs::read_to_string` in `validate_pure_marker_file`
- `crates/sifr_package/src/imports/namespace_api.rs:32` — `std::fs::read_to_string(init_path)` in `parse_init_sifr_reexports`

Without these entries, M2 cannot fully route semantic reads through the typed provider. Consider extending the checker to require that no new `std::fs::` / `fs::read_*` calls appear in these crates without a corresponding inventory row.

### 3. Execution tracker M1 status not updated
**File:** `issues/ad-hoc-typescript-go-compiler-architecture-transfer-execution.md:12`

The M1 checklist row is still `[ ] M1 architecture contract and guardrails completed` while the validation log shows M1 has passed all required gates. M0 was already flipped to `[x]` after merge (line 11). Flip M1 to `[x]` before opening the PR, and add an M1 PR log entry mirroring the M0 entry at line 145.

## Major (should fix before merge)

### 4. `is_file()` calls characterized as "lookup dependencies" without a clear schema
**File:** `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:40`

`crates/sifr_driver/src/project/discovery.rs:90`, `:118`, `:180` are `is_file()` metadata probes, not content reads. The "Provider-tracked successful and failed lookup dependencies" framing is acceptable, but the inventory should make explicit that these are *probe* operations so M2 does not accidentally migrate them under a content-read path. Same applies to `is_file()` / `is_dir()` checks in sifr_format at `:177` and `:180`.

### 5. No negative-coverage assertion for the inventory
**File:** `verification/tooling/check_typescript_go_m1_guardrails.py`

The checker only asserts the inventory lists the entries it knows about. It does not assert "no new production `std::fs::` call appears outside this list" or "no inventory line drifted from the actual code." Both failures (item 1 and item 2 above) would have been caught by even a simple grep of `std::fs::|fs::read_to_string|fs::read_dir` over the affected crates compared to the listed line numbers.

## Minor (acceptable but worth noting)

### 6. Brittle stub pattern check
**File:** `verification/tooling/check_typescript_go_m1_guardrails.py:83-86`

The check `"-> Option<TextRange> {\n        None" not in text` requires exact whitespace. It would not catch a function that returns `None` on the first statement with a different indent, nor would it catch a future stub with a different return type. Acceptable for now, but consider evolving to a structural AST/regex check that any `Option<TextRange>`/`Option<TextRangeUtf>` function must contain real logic before its first `None`/`Some` return.

### 7. `DocumentState::rebuild` claim in M1 doc wording
**File:** `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:73-75` and `internal_docs/lsp_server.md:68-69`

Both files say `DocumentState::rebuild` "calls `AnalysisHost::open_single_file` with `FrontendMode::SingleFile` on open/change/save." Verified accurate: `document_store.rs:86` (new), `:101` (change_full), `:126` (change_incremental), `:136` (save). No correction needed.

## Verifications confirmed accurate

- Locked terms (`sifr_source`, `SourceProvider`, `WorkspaceSession`, `WorkspaceSnapshot`, `DirtyScope`, `DirtyReason`, `ModuleSignature`, `CompilerFingerprint`, `CacheKeyFingerprint`, `FlowGraph`, `.sifrbuildinfo`, `QueryReadiness`) are present in the M1 doc.
- Source-map guardrail snippets are real: `source_maps.rs:91` calls `byte_offset_with_encoding` and `:102` calls `range_at`.
- M1-M4 serialization language ("M1-M4 remain serialized") is recorded in the M1 doc and the lsp doc.
- `perf.lsp.request_families` is the only `lsp-query` case in `verification/performance/manifest.json`; budget id matches.
- `sifr_source` dependency direction guard passes (only `ruff_text_size`).
- No M2+ terms (`WorkspaceSession`, `WorkspaceSnapshot`, `SourceProvider`, `DirtyScope`, `can_replace_module_in_project`, `ModuleSignature`, `CompilerFingerprint`, `CacheKeyFingerprint`, `FlowGraph`, `sifrbuildinfo`) appear in any implementation under `crates/`. No behavior overreach.
- `scripts/run_all_tests.sh` now invokes the M1 guardrail; regressions would block PRs.
- `rustfmt --check` and `git diff --check` pass; LSP doc status header and "is being migrated" language are correct.

## M1 closeout assessment

The M1 closeout criteria are largely met (locked terms, direct-read inventory, four guardrails, M1-M4 serialization, aggregate-only LSP budget, target-vs-actual docs). The two inventory problems (stale line + missing entries) are factual defects in the artifact that M2 will use; they should be corrected before the M1 closeout gates M2.

---

**Not approved for PR as-is.** Address items 1-3 (stale `source_map.rs:254` line, four missing inventory entries, M1 tracker/PR log updates) and consider adding a completeness check to the M1 guardrail (item 5) before opening the M1 PR.
