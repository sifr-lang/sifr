Now I have a clear picture. Let me write the review.

# M1 Implementation Review — Pass 2

## Validation status
All five gates the user reported pass. I independently re-ran them on the working tree:
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` → PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` → PASS
- `git diff --check` → exit 0
- `cargo fmt --check` → exit 0
- `python3 scripts/check_file_size_guardrails.py` → PASS
- `cargo clippy --workspace -- -D warnings` → exit 0
- `python3 scripts/check_source_crate_dependency_direction.py` → exit 0

I also spot-checked inventory line numbers (`validate.rs:14, :43, :44`; `source_map.rs:240, :254`; `layout.rs:30`; `build/workspace.rs:219, :282, :296`; `source_maps.rs:91, :102`) — all match real code. Pass-1 items 1 (stale `source_map.rs:254`) and the four missing entries are fixed. The M1 doc updates in `lsp_server.md`, `frontend_cache_invalidation.md`, `frontend_query_architecture.md`, `architecture.md`, and `performance_budgets.md` are all well-targeted and consistent with the parallel M0 mention at `architecture.md:271`. The execution tracker correctly flips M1 and records validation.

## Findings, ordered by severity

### Major (block PR)

**1. Guardrail regex is broken — `is_file()` / `is_dir()` probes are not caught.**

`verification/tooling/check_typescript_go_m1_guardrails.py:24`

```python
r"(?:std::fs::|fs::)(?:read_to_string|read_dir)|\\.is_file\\(\\)|\\.is_dir\\(\\)"
```

In a Python raw string, `\\` is two literal characters (backslash + backslash). The compiled regex therefore looks for `\\.is_file\\(\\)` in source lines, which never matches `.is_file()` or `.is_dir()`. I confirmed by importing the module and inspecting `mod.DIRECT_FS_PATTERN.pattern` — it reads `(?:std::fs::|fs::)(?:read_to_string|read_dir)|\\.is_file\\(\\)|\\.is_dir\\(\\)`. A `re.search(".is_file()")` against this pattern returns `False`.

The effect: `direct_fs_sites()` only enumerates `read_to_string` and `read_dir` sites. The script reports 30 sites and 0 missing — but the true production count is ~50 sites once you count `is_file()` / `is_dir()` probes. The script is silently missing every probe site, which is exactly the negative-coverage check the pass-1 review's item 5 asked for. Without this fix, M2 can land with a new `is_file()` probe in any inventoried crate and the guardrail will not catch it.

Fix is one character per pattern alternative: change `\\.` to `\.` and `\\(` to `\(` and `\\)` to `\)` in the raw string. After the fix, the script becomes a true negative-coverage check.

**2. `crates/sifr_package/src/cargo/lock_modes.rs:46` is not in the inventory.**

```rust
// crates/sifr_package/src/cargo/lock_modes.rs:46
if package_root.is_dir() {
    None
} else {
    Some(PackageDiagnostic::source_unavailable_offline(...))
}
```

This is a real production probe in the offline lock-mode diagnostic path — a `.sifrbuildinfo`/package-identity adjacent read. It's not enumerated in the inventory table and not named in the permitted-exceptions block. Because the regex is broken (item 1), the script can't catch it. Once the regex is fixed, item 2 will be caught automatically and the doc will need a new row such as "Package offline lock-mode probe" pointing at M15 deferred territory (or a permitted exception with a rationale).

This is the same class of miss pass-1 item 2 flagged. The author added most of the missing entries (and the build-metadata exception at `workspace.rs:296`), but this one slipped through.

### Minor (worth noting, not blocking)

**3. Inventory line `crates/sifr_format/src/lib.rs:456` documents a write.** Line 456 is `fs::write(path, source)` in the formatter. The M1 doc correctly classifies writes as "command-output effects" rather than semantic reads. The script's `read_to_string`/`read_dir` regex never matched it, and the doc names it in the "Formatter standalone input" row alongside reads. Acceptable, but worth a one-line note in the doc clarifying the read/write split so M2 doesn't accidentally try to route a write through a read-only provider.

**4. The `-> Option<TextRange> {\n        None` literal-string check is brittle.** It depends on exact whitespace. If `source_maps.rs` ever wraps the body in a `match` or moves the early-return to a different indentation, the check fires even though no regression has occurred. The current source code uses `?` on a method call, not a `None` literal return, so the check passes today. Acceptable for M1; consider a structural regex (e.g., `pub fn ... -> Option<TextRange(?:Utf)?> \{[^}]*Some\(`) for the next guardrail revision.

**5. The script's `validate_source_dep_guard` shells out to `scripts/check_source_crate_dependency_direction.py`.** If the source-dep script is renamed or moved, the M1 guardrail fails with an unhelpful message. Low priority — the dep script is stable — but worth a clearer error path.

**6. The doc lists six `crates/sifr_package/src/projection.rs` lines (`100, 109, 127, 129, 169, 187`) as permitted exceptions, but only two (`129, 187`) match the read regex.** The other four are `fs::write` calls and `.exists()` checks. They are correctly categorized as package-management output and repair-state effects, but since the regex doesn't catch writes or `exists()`, the script's enforcement of these exceptions is purely textual (the doc must contain those exact `path:line` substrings). Fine for M1; the future M2/M15 closeout will need to know these rows are write-side, not read-side.

## Summary of M1 contract items

- Locked terms (`sifr_source`, `SourceProvider`, `WorkspaceSession`, `WorkspaceSnapshot`, `DirtyScope`, `DirtyReason`, `ModuleSignature`, `CompilerFingerprint`, `CacheKeyFingerprint`, `FlowGraph`, `.sifrbuildinfo`, `QueryReadiness`): all present in the M1 doc. ✓
- Source-map guardrail (`byte_offset_with_encoding(position, encoding)`, `range_at(span, encoding)`, stub-pattern check): pins M0 correctness. ✓
- LSP current-state checks (`AnalysisHost::open_single_file` + `FrontendMode::SingleFile` in `document_store.rs`; `lane_for_method` + no `CancellationToken` in `scheduler.rs`; `pending: BTreeSet<String>` + `remove_pending` in `request_queue.rs`): all current. ✓
- Aggregate-only LSP budget pinned to `perf.lsp.request_families`: manifest has exactly one `lsp-query` case with that budget id. ✓
- M1-M4 serialization language: present in M1 doc and `lsp_server.md`. ✓
- Doc updates: `lsp_server.md` (status line, "is being migrated toward" wording, new "Current M1 Compiler-Service Caveats" section); `frontend_cache_invalidation.md` (6-line M1 note); `frontend_query_architecture.md` (M1 note deferring to M3/M4); `architecture.md` (M1 line at 271, parallel to M0 line); `performance_budgets.md` (3-line note). All on point. ✓
- M1 checker wired into `scripts/run_all_tests.sh`: ✓
- M1 checker self-test: ✓
- M1 execution-tracker update: ✓
- `sifr_source` dependency direction: ✓ (no behavior migration)
- No M2+ identifiers (`SourceProvider`, `WorkspaceSession`, `WorkspaceSnapshot`, `DirtyScope`, etc.) in `crates/`: not in scope of this review, but the pass-1 review already confirmed.

## Verdict

**NOT APPROVED.**

The M1 spec calls for the inventory to be the input list for M2's provider migration and for the script to catch drift on the inventory. Item 1 means the script is materially weaker than the doc claims, and item 2 means the inventory is genuinely incomplete. Both are one-line fixes. Once they land (regex fix in `verification/tooling/check_typescript_go_m1_guardrails.py:24` and a new row for `crates/sifr_package/src/cargo/lock_modes.rs:46` in the inventory table), M1 is ready to open as a PR.
