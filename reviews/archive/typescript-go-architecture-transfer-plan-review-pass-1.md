I have enough evidence. Below is the focused review.

---

# Plan Review: TypeScript-Go Compiler Architecture Transfer

## Verdict

**The plan is acceptable for starting M0 exactly as written, but M0's milestone description needs tightening before the implementation PR is opened.** No prerequisite change is required first.

## Evidence verified against the issue

- `sifr_source` crate does not exist (no entry under `crates/`); must be created.
- `SourceMapView::text_position_to_span` and `span_to_text_range` return `None` as stubs at `crates/sifr_frontend/src/graph_cache_and_queries.rs:227` and `:237`.
- `interactive.source_map_lookup` at `crates/sifr_frontend/src/bin/frontend_query_bench.rs:255-274` calls the stub and discards the result (`let _ = source_map.text_position_to_span(...)`).
- Three competing source-position authorities exist today: `crates/sifr_syntax/src/lib.rs:87` (UTF-8 byte line starts, `Vec<usize>`), `crates/sifr_diagnostics/src/source_map/mod.rs:61-65` (separate `SourceMap` with `Vec<u32>` line starts), and `crates/sifr_frontend/src/graph_cache_and_queries.rs:77` (string wrapper, no line map). This is exactly the split D0-2 describes.
- `ruff_text_size` is already a workspace dep (`Cargo.toml` declares `third_party/ruff/crates/ruff_text_size`), so the locked decision's "may depend only on `std` and source-position primitives such as `ruff_text_size`" is workable.
- Dependency direction in the locked decision is consistent: `sifr_source` is at the bottom; `sifr_syntax` already depends on `sifr_diagnostics` (not the other way around), so the new crate can sit below both without creating cycles.
- Milestone `depends on` chains (M0→M1→…→M17, with M11/M14/M15/M16/M17 fanning into M5/M10) are coherent; no milestone is gated on work a later milestone delivers.

## Correctness risks to address before opening the M0 PR

1. **M0 closeout is too soft to be a gate.** "`no valid source-map lookup returns `None` because the method is stubbed`" can be read two ways (always-`Some` vs. validity-aware). Pin to: conversions return `Some` for positions on character boundaries inside registered source files, and return `None` only for genuinely invalid positions (unregistered file, byte inside multi-byte scalar, non-boundary offset).

2. **M0 scope omits an explicit "out of scope" list.** Without one, M0 risks pulling in overlays, snapshots, fingerprints, or scheduler hooks "since they touch the same code". Lock the exclusion list: no `SourceProvider`, no `WorkspaceSession`, no `WorkspaceSnapshot`, no `DirtyScope`, no cache reuse, no scheduler changes, no LSP request flow changes.

3. **M0 scope does not enumerate LSP/syntax/diagnostics migration sites.** The spec says "migrate syntax, diagnostics, frontend, and LSP conversion" but doesn't list files. Add a concrete inventory before implementation: `crates/sifr_lsp/src/conversion.rs:45,69,391`; `crates/sifr_lsp/src/capabilities.rs:28`; the parser-side `sifr_syntax::SourceText` consumers; the `sifr_diagnostics::SourceMap` consumers that need a re-pointing, not a rewrite. Without an inventory, the PR will either under-migrate (leaving the stub reachable) or over-migrate (touching the JSON schema, which the locked decision forbids).

4. **`sifr_syntax::SourceText` re-export vs rename is undecided.** The locked decision says `sifr_source` sits below `sifr_syntax`; that implies `sifr_syntax::SourceText` either becomes a re-export of `sifr_source::SourceText` or wraps it. Either is fine, but M0 must commit to one. The `Vec<usize>` line-start field at `crates/sifr_syntax/src/lib.rs:89` is a clean opportunity to delegate to `sifr_source::LineMap`. Pick a name and pin it in the M0 PR.

5. **No `sifr_source` dep-direction guardrail.** The locked decision states `sifr_source` must not depend on `sifr_syntax`/`sifr_diagnostics`/`sifr_frontend`/`sifr_analysis`/`sifr_lsp`/etc., but the issue defers the actual guardrail to M1. Risk: M0 lands, a later PR re-adds a dep, no automation catches it. Recommend M0 itself land a small `scripts/check_source_crate_dependency_direction.py` (or extend the existing `check_hir_maintainability_guardrails.py`) and have CI fail on a violation. This is two hours of work and saves a future archeology session.

6. **The perf-bench rewrite alone is not a correctness test.** `frontend_query_bench.rs:266` currently does `let _ = source_map.text_position_to_span(...)`. Rewriting it to assert a real round trip is fine, but a perf bench is allowed to no-op on a regression. M0 should also add a dedicated unit test in `crates/sifr_frontend` (and/or a `crates/sifr_source` test) covering UTF-8, UTF-16, and UTF-32 round trips across multibyte, CRLF, EOF, surrogate-pair interior, and non-boundary offsets, with snapshot baselines. The bench becomes a perf assertion; the unit test becomes the correctness gate.

7. **EOF, CRLF, multibyte, and rendered-diagnostic parity tests are mentioned in scope but not in closeout.** Move them from "scope" into the closeout acceptance list. A milestone can be closed with passing tests omitted otherwise.

8. **`FrontendContext::load_project` reads entrypoint and project dir via `std::fs` directly at `crates/sifr_frontend/src/graph_cache_and_queries.rs:426` and `:436`.** M0 does not need to change this; M2 does. But if M0 reworks `ModuleState` to carry `sifr_source::SourceText`/`LineMap` (it should, otherwise the new types don't reach the rest of the frontend), then `module_state` becomes a touch point. Make that construction explicit in the M0 PR so D0-3 and the "direct-read inventory" promise from M1 are not papered over.

9. **LSP UTF-16 negotiation must keep working.** `crates/sifr_lsp/src/capabilities.rs:28` advertises UTF-8 position encoding. M0's `PositionEncoding` migration must not break the negotiation. Closeout should require an LSP test that round-trips a UTF-16 position through the new conversion layer.

10. **D0-11 (docs overstate target architecture as reality) is not in M0's scope.** M0 won't fix `internal_docs/lsp_server.md`. That's fine — M1 is the right home — but flag it: if M0 ships first, reviewers should not assume the docs will be corrected in the same PR. Track as M1 acceptance.

## Dependency sequencing observations

- The M0→M1→M2→M3→M4→M5 chain is defensible. M0 closes the source-position hole before any session/snapshot work depends on it, which is the only ordering that prevents the "snapshot built on stubbed conversions" anti-pattern.
- M11 depending on both M5 and M10 is correct: the scheduler can only be "real" once the session owns the snapshot and the snapshot has identity that can be captured per request.
- M15 depending on M3, M5, M6, and M10 is correct: residency needs the session (M3), the LSP layer (M5), event compaction (M6), and snapshot reuse (M10) all in place.
- M17 depending on M5, M12, M14, and M16 is correct: the marker-based editor corpus needs the LSP session, per-request budgets, bucketed indexes, and trace/status to exist before it can validate their behavior.
- No milestone in the chain has a missing dependency. None depends on work that M0 alone delivers (i.e., M0 is genuinely the first PR; later milestones don't need anything M0 doesn't promise).

## Missing acceptance gates worth adding now

- A "LSP conversion sites migrated" gate: list of file:line call sites that must move from `sifr_syntax::SourceText`/raw byte math to `sifr_source` conversion APIs.
- A "`sifr_source` public API" lock: explicit list of types and functions the crate exports, so syntax/diagnostics/frontend/LSP migration PRs have a stable target.
- A "dep-direction guardrail in CI" gate (item 5 above).
- A "bench is no longer a no-op" gate, ideally backed by a unit test (item 6 above).
- An "M0 out of scope" list (item 2 above) inside the milestone description.

## Recommendation

Approve M0 as the first implementation PR. Before opening it, tighten the M0 description in `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md` (lines 482-500) to: enumerate the migration call sites, list `sifr_source`'s locked public API, add an explicit out-of-scope list, add the unit-test gates alongside the perf-bench rewrite, and require the dep-direction guardrail as part of M0 (not deferred to M1). These are doc-only edits to the issue and the execution checklist; they do not require reopening the planning review. The architecture decisions themselves are sound and M0 is correctly placed.
