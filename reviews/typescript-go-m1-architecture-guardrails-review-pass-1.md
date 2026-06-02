# M1 Review: TypeScript-Go Architecture Transfer Guardrails

## Validation status
`python3 verification/tooling/check_typescript_go_m1_guardrails.py` and `--self-test` pass. Local validation suite (clippy, fmt, quick profile, file-size guardrail, source-dep guardrail) all green per the execution tracker.

## 1. Scope fit and correctness

**Scope is correct.** The M1 work is exactly what the M1 milestone contract calls for: a pre-flight gate that locks terms, records a direct-read inventory, adds guardrails, updates overstated LSP docs, and serializes M1-M4. No behavior migration (no `SourceProvider`, `WorkspaceSession`, `WorkspaceSnapshot`, `DirtyScope`, cache reuse, scheduler queues, or LSP behavior changes) was attempted. The 5 modified files, 1 new guardrail doc, 1 new guardrail script, and 1 execution-tracker update match the M1 surface area.

**Doc updates are well-targeted but incomplete in coverage:**

- `internal_docs/lsp_server.md` — Status line, "Internal Layers" wording ("must expose" → "is being migrated toward"), and the new "Current M1 Compiler-Service Caveats" section are all on point.
- `internal_docs/frontend_cache_invalidation.md` — 6-line M1 note at top correctly pins the doc to the pre-session cache behavior and names the future milestones.
- `internal_docs/performance_budgets.md` — 3-line note at the `lsp-query-001-request-families` paragraph correctly tags it as aggregate smoke and defers to M12.
- `issues/ad-hoc-typescript-go-compiler-architecture-transfer-execution.md` — M0 marked complete with PR link; M1 validation log entries are reasonable.

**Doc coverage gap (medium):** `internal_docs/architecture.md` mentions M0 at line 271 ("TypeScript-Go architecture transfer M0: `sifr_source`...") but does not gain an M1 mention. `internal_docs/frontend_query_architecture.md` describes the same `FrontendContext` semantics that the M1 doc explicitly says the cache doc covers; it gets no M1 note. M1's goal "update docs that overstated target LSP layers as fully implemented" technically covers only LSP layers, but the architecture/frontend-query docs are the broader home for the same concepts and the parallel M0 mention at architecture.md:271 makes the omission conspicuous.

## 2. Direct-read inventory — materially incomplete

The 9-row inventory in `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md` covers 3 frontend reads, 3 driver resolution/discovery reads, 1 linter read, 3 formatter reads, 1 package manifest read, and 2 package source-map directory reads. Per the D0-3 locked decision, these categories are all in scope:

> "Frontend project loading, module import discovery, driver project discovery, package source reads, package manifest/config reads that affect compilation, lint source reads, format source reads, and LSP workspace reads go through the provider."

Production semantic direct reads I found that are **not in the inventory**:

**`crates/sifr_driver` (production):**
- `src/workspace/mod.rs:32` — `if manifest_path.is_file()` (workspace manifest existence)
- `src/workspace/mod.rs:49` — `std::fs::read_to_string(manifest_path)` (workspace manifest read)
- `src/workspace/mod.rs:156` — `if !absolute.is_dir()` (workspace path check)
- `src/project/package_discovery.rs:53` — `std::fs::read_to_string(&resolved.resolved_module.file_path)` (package module source)
- `src/build/workspace.rs:282, 296` — build metadata read/write (`.sifrbuildinfo` territory — M15; should be in the inventory as a deferred-but-listed row or permitted exception)

**`crates/sifr_format` (production):**
- `src/lib.rs:197` — `fs::read_dir(path)` in `collect_sifr_files_inner` (formatter directory walk — the inventory's "formatter standalone input" row cites lines 177/180/446 only and misses 197, 215, 456)
- `src/lib.rs:215` — `child.is_dir()` check
- `src/lib.rs:456` — `fs::write(path, source)` (formatter source write)
- `src/config.rs:85, 109` — `is_file()` candidate check and `fs::read_to_string(&canonical)` config read

**`crates/sifr_lint` (production):**
- `src/config.rs:48, 72` — `is_file()` candidate check and `fs::read_to_string(&canonical)` config read
- `src/discovery.rs:29, 33, 79` — `is_file`/`is_dir` lint file discovery

**`crates/sifr_package` (production):**
- `src/source/layout.rs:30` — `std::fs::read_to_string(path)` (pure marker source read)
- `src/manifest/validate.rs:14, 43, 44` — manifest validation (`is_dir`, `.sifr` file checks)
- `src/ops/session_discovery.rs:6, 13, 25` — `is_file()` and `fs::read_to_string(manifest)` package session reads
- `src/ops/session_targets.rs:17, 34, 42` — `is_file`, `read_dir`, `is_dir` package session targets
- `src/imports/namespace_api.rs:32, 264` — `__init__.sifr` read and `is_file` check
- `src/projection.rs:100, 111, 129, 169, 187` — Cargo projection reads/writes

**`crates/sifr_frontend` (production):**
- `src/bin/frontend_query_bench.rs:115, 182, 223, 290` — bench binary reads used by the `interactive.source_map_lookup` and `incremental.*` perf manifest cases

**`crates/sifr` (CLI binary):**
- `src/lint_cli.rs:496` — `if path.is_dir()`
- `src/check_and_package_commands.rs:409, 551, 590` — target path checks
- `src/cli_model_and_entrypoint.rs:716` — `parent.join(format!("{module_name}.sifr")).is_file()` module lookup

The M1 closeout criterion "guardrails fail on new semantic bypasses or untracked source-position forks" needs the inventory to be the input list for M2's provider migration. The current 9 rows will leave ~25+ production sites unmigrated in M2. The M1 spec says "no source-provider, session, snapshot, or LSP behavior migration starts until it passes" — meaning M2 is gated on this inventory, and an incomplete inventory means M2 will start with a partial migration plan.

At minimum, the inventory needs to add: `sifr_format` config reads (`config.rs:85, 109`), `sifr_lint` config reads (`config.rs:48, 72`), `sifr_driver` workspace manifest read (`workspace/mod.rs:32, 49`), `sifr_package` namespace API read (`namespace_api.rs:32`) and manifest validation, and the formatter directory walk (`lib.rs:197`). The build metadata and projection sites are defensible as M15/deferred or non-semantic, but should be explicitly listed or named in the permitted-exceptions block — not silently absent.

## 3. Guardrail script — too weak on inventory coverage, brittle on identifiers, no documented update path

**What's good:**
- The `byte_offset_with_encoding(position, encoding)` / `range_at(span, encoding)` checks plus the `-> Option<TextRange> {\n        None` literal-pattern check correctly catch a regression to the M0 stub.
- The LSP current-state checks (`AnalysisHost::open_single_file` + `FrontendMode::SingleFile` in document_store.rs; `lane_for_method` + absence of `CancellationToken` in scheduler.rs; `pending: BTreeSet<String>` + `remove_pending` in request_queue.rs) correctly pin the M1 reality and will fail when M5 changes the shape.
- The aggregate-only-LSP-budget check correctly pins M12 as a future milestone and will fail when M12 splits the scenario list.
- The `--self-test` correctly exercises the negative case (incomplete doc → failure).

**What's too brittle:**
- The `byte_offset_with_encoding(position, encoding)` and `range_at(span, encoding)` checks are tied to the literal parameter names of `sifr_source`. Any future signature change to `sifr_source::SourceText` methods (a normal refactor) breaks the check. The check should validate the *outcome* (valid registered file returns `Some`, invalid returns `None`) rather than the *implementation string*. At minimum, the script should comment that it pins the current `sifr_source` parameter shape.
- The LSP "current state" identifier strings (`DocumentState::rebuild`, `AnalysisHost::open_single_file`) are coupled to specific names. Acceptable for M1, but a future M5 refactor renaming `DocumentState::rebuild` would falsely fail the check; the check should be considered a "do not silently refactor without updating M1 doc + script" guardrail.

**What's too weak:**
- There is no negative check that production direct reads are limited to the inventory + permitted exceptions. The script trusts the doc. If someone adds a new `std::fs::read_to_string` in a non-inventoried production site, the script will not catch it. A stronger (but heavier) check would enumerate the production crates and verify each `std::fs::*` call site matches an inventory row or a permitted exception. Whether to add this in M1 or defer to M2's pre-migration audit is a judgment call — but M1 should at least document this as a known gap and commit M2 to running the negative audit before closure.

**No documented update path for M5 / M12 / M15:**
The M1 doc says "Later milestones may update this file and its checker when they intentionally replace one of these current limitations." This is correct but vague. Concretely:
- M5's closeout will break the "current LSP single-file rebuild" check by removing `AnalysisHost::open_single_file` and `DocumentState::rebuild` from `document_store.rs`.
- M12's closeout will break the "aggregate-only LSP budget" check by adding per-request scenarios.
- M9 / M10 (cache reuse) will be fine, since those don't touch the M1 checks.
- M13 (cancellation tokens) will need to update the "no `CancellationToken` in scheduler.rs" check, but the current script only checks *absence* of the term, so adding cancellation tokens will not break it.
- M15 (`.sifrbuildinfo`) will be fine since the script doesn't check the build metadata path.
- M17 (internal handles) will be fine.

So the script's update obligations are: M5, M12. The M1 doc should explicitly require those milestones' closeout PRs to update both the doc caveats and the corresponding script checks. Without this, M5/M12 can land while leaving the M1 script in a permanently failing state. The doc should also name which script check each future milestone owns.

## 4. Docs distinguish current vs target architecture

**Yes for the targeted docs.** `lsp_server.md` is clearly split: "Internal Layers" is now phrased as "is being migrated toward" with a separate "Current M1 Compiler-Service Caveats" section that names the current behavior and points at the future milestone (M5, M11, M13) for the upgrade. `frontend_cache_invalidation.md` and `performance_budgets.md` each have an M1-tagged note that defers to the future milestone. The M1 guardrail doc itself maintains the same current/target split: "Locked Terms" distinguishes what is locked now from "Not implemented in M1."

**Partial gap in non-targeted docs.** `internal_docs/architecture.md` line 271 still says "TypeScript-Go architecture transfer M0: sifr_source..." but gains no M1 line. `internal_docs/frontend_query_architecture.md` describes the soon-to-change `FrontendContext` semantics with no M1 caveat. These don't describe "LSP layers" per se, but the M0 mention in architecture.md sets a precedent the M1 work should follow for consistency. The frontend-query doc is the closer call — it accurately describes today's `FrontendContext`, so an M1 caveat is optional. Architecture.md should get a one-line M1 mention next to the M0 mention for parity.

## 5. Required changes for M1 approval

**Required (block PR):**

1. **Expand the direct-read inventory.** Add at minimum: `sifr_format/src/lib.rs:197, 215, 456` (formatter directory walk + write); `sifr_format/src/config.rs:85, 109` (config reads); `sifr_lint/src/config.rs:48, 72` (config reads); `sifr_lint/src/discovery.rs:29, 33, 79` (lint file discovery); `sifr_driver/src/workspace/mod.rs:32, 49` (workspace manifest); `sifr_package/src/manifest/validate.rs:14, 43, 44` (manifest validation); `sifr_package/src/imports/namespace_api.rs:32, 264` (init reads). Either add them to existing rows or add new rows. Add the build metadata read/write and projection ops as named deferred/permitted entries — not silently absent.
2. **Document the script's update obligations.** In the M1 doc, add a subsection naming M5, M12 (and any other relevant milestones) as owning the corresponding M1 guardrail updates as part of their closeout. Without this, M5/M12 closeout can leave the M1 script in a stale-failing state.
3. **Update the script's `REQUIRED_DOC_SNIPPETS` list** to reflect the expanded inventory (so the doc and script stay in sync — the snippet list currently enumerates the existing 9-row inventory; expanding the inventory will require expanding the snippet list too).

**Recommended (do not block PR):**

4. **Add an M1 mention in `internal_docs/architecture.md`** next to the M0 line at line 271, naming the M1 guardrail doc and the M1-LSP caveat in `lsp_server.md`. Keeps the architecture doc as a single source of truth for milestone status.
5. **Add a brief M1 note in `internal_docs/frontend_query_architecture.md`** explaining that the `FrontendContext` semantics described here are pre-session and will be re-expressed after M4.
6. **Add a one-line comment in the script** noting that the `byte_offset_with_encoding(position, encoding)` and `range_at(span, encoding)` literal-string checks pin the current `sifr_source` parameter shape and need updating if `sifr_source` signatures change. The current behavior is correct but looks like a magic-string check.

## 6. Approval verdict

**M1 is not approved for PR as-is.** The locked terms, source-map guardrail, and LSP/budget reality guardrails are correct. The script catches the core M0-regression risks. The doc updates on the targeted files (LSP, cache invalidation, performance budgets) are well-phrased. However, the direct-read inventory is materially incomplete — at least 15-20 production sites are not listed, which means M2 will start with a partial migration plan. Once the inventory is expanded to cover the missing `sifr_format`, `sifr_lint`, `sifr_driver` workspace, and `sifr_package` namespace/manifest reads (and the script's snippet list is updated to match), the script's M5/M12 update obligations are documented, and the optional architecture-doc mention is added, M1 is ready to open as a PR.
