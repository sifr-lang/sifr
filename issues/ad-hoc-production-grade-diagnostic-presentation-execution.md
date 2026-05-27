# Ad Hoc Phase Execution: Production-Grade Diagnostic Presentation

Status: planned on 2026-05-27

Phase contract: `issues/ad-hoc-production-grade-diagnostic-presentation.md`

## Checklist

- [x] Phase plan drafted
- [x] Phase plan reviewed and approved for implementation
- [x] Implementation-readiness review approved
- [ ] M1 renderer contract lock completed
- [ ] M2 human source-aware output completed
- [ ] M3 compact stable output completed
- [ ] M4 docs and closeout completed
- [ ] Full local validation recorded
- [ ] Final production-readiness review approved

## Planning Lock Addendum

This phase locks the diagnostic output-mode responsibilities before implementation starts. Changing the `human`, `compact`, or `json` mode contract requires a reviewed planning update.

### Required Implementation Work

| ID | Work item | Required closeout |
| --- | --- | --- |
| W-1 | Default human output drops primary source locations. | M2 renders file, line, column, source snippet, highlight, notes, help, suggestions, and docs URL for span-backed diagnostics. |
| W-2 | Compact output is verbose grouped text rather than stable one-line diagnostics. | M3 emits summary plus one physical line per retained diagnostic with stable fields. |
| W-3 | JSON is structurally useful but needs schema stability protected while CLI text formats change. | M1 and M4 confirm JSON schema compatibility and update only intentional text baselines. |
| W-4 | Renderer behavior is split between CLI-local rendering and `sifr_diagnostics` source-aware rendering. | M1 locks canonical `sifr_diagnostics` presentation ownership and tests CLI delegation after recovery limiting. |
| W-5 | Spanless internal diagnostics need a deliberate display contract. | M2 and M3 add spanless internal diagnostic fixtures for human and compact modes. |
| W-6 | Human target output requires visual highlight markers that the current canonical renderer does not print. | M2 adds a highlight renderer for single-line, multiline, and CRLF source snippets. |
| W-7 | Compact summary changes current help-count behavior. | M1 records severity-only compact summary counts as an intentional contract change and keeps help details in human/JSON. |
| W-8 | Snapshot path normalization affects portable baselines. | M1 documents `<WORKSPACE>` normalization for verification baselines while preserving live diagnostic display paths. |
| W-9 | Command-level format routing can drift across CLI entrypoints. | M1/M2/M3 add regression coverage proving `check`, `build`, `run`, and `emit` diagnostics keep respecting selected formats. |
| W-10 | Multiline diagnostics need explicit verification coverage. | M1 creates a multiline diagnostic verification fixture with locked `human`, `compact`, and `json` baselines. |
| W-11 | Compact mode must not inherit the old grouped `CompactKey` behavior. | M1 locks one-line-per-retained-diagnostic compact rendering after recovery limiting; M3 implements that contract. |
| W-12 | Related spans are present in the canonical diagnostic model but not rendered by current human presentation. | M2 renders related spans with labels/kinds after the primary span. |

### Locked Mode Decisions

| Mode | Locked decision |
| --- | --- |
| `human` | Default developer-facing view with source snippets and caret-style highlights. |
| `compact` | Terse line-oriented output for agents, CI, and scanning; no snippets or default URLs. |
| `json` | Canonical structured transport; preserve existing rendered diagnostic schema unless separately reviewed. |

## Review Log

- `2026-05-27`: Initial phase plan drafted from current CLI output inspection and desired output-mode responsibilities.
- `2026-05-27`: Claude phase review pass 1 found two planning blockers: unresolved CLI-vs-`sifr_diagnostics` delegation ownership and missing highlight-rendering infrastructure in the plan. The phase was updated to make `sifr_diagnostics` the canonical presentation owner after CLI recovery limiting, require caret/highlight rendering, require CRLF-safe snippet rendering, document severity-only compact summaries, add multiline fixtures, add command-level format routing tests, and document `<WORKSPACE>` baseline normalization.
- `2026-05-27`: Claude phase review pass 2 returned `SATISFIED` with no remaining blockers. One non-blocking precision edit was applied: M1 now requires a JSON schema-lock fixture enumerating the required `RenderedDiagnostic` fields. Review artifact: `reviews/diagnostic-presentation-phase-review-pass-2.md`.
- `2026-05-27`: Claude implementation-readiness review pass 1 returned `SATISFIED` with no blockers and four required precision edits. The phase was updated to require a new multiline verification fixture, clarify that command-format tests lock existing routing behavior, forbid inheriting old compact grouping behavior, and explicitly scope related-span rendering into M2. Review artifact: `reviews/diagnostic-presentation-implementation-readiness-review-pass-1.md`.
- `2026-05-27`: Claude implementation-readiness review pass 2 verified all pass-1 precision edits and returned `SATISFIED` with no blockers, unresolved decisions, or discovery gaps. Review artifact: `reviews/diagnostic-presentation-implementation-readiness-review-pass-2.md`.

## Validation Log

- Validation evidence will be recorded per implementation milestone.
- Planning PR validation starts with `git diff --check` and review artifact checks.

## PR Log

- Implementation PR links will be recorded per milestone after they are opened and merged.
