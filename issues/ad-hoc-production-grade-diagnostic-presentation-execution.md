# Ad Hoc Phase Execution: Production-Grade Diagnostic Presentation

Status: planned on 2026-05-27

Phase contract: `issues/ad-hoc-production-grade-diagnostic-presentation.md`

## Checklist

- [x] Phase plan drafted
- [x] Phase plan reviewed and approved for implementation
- [x] Implementation-readiness review approved
- [x] M1 renderer contract lock completed
- [x] M2 human source-aware output completed
- [x] M3 compact stable output completed
- [x] Diagnostic presentation verification gate completed
- [x] M4 docs and closeout completed
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
| W-13 | The phase lacks a named mechanical verification gate. | M1 adds `verification/tooling/check_diagnostic_presentation_contract.py`, its `--self-test`, and `scripts/run_all_tests.sh --profile quick` wiring; M2/M3/M4 close all checker obligations. |

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
- `2026-05-27`: User review found the phase did not make verification a first-class deliverable. The phase was updated to require `verification/tooling/check_diagnostic_presentation_contract.py`, negative self-tests, quick-lane wiring, and per-mode fixture/baseline enforcement.
- `2026-05-27`: Claude verification-contract review pass 1 returned `SATISFIED` with no blockers and two required precision edits. The phase was updated to name `decimal_invalid_literal` as the existing locked single-line fixture and to clarify that `check_diagnostic_presentation_contract.py` is an M1 deliverable, not a pre-existing guardrail. Review artifact: `reviews/diagnostic-presentation-verification-contract-review-pass-1.md`.
- `2026-05-27`: Claude verification-contract review pass 2 verified the pass-1 precision edits and returned `SATISFIED` with no blockers, no required precision edits, and no remaining verification or discovery gaps. Review artifact: `reviews/diagnostic-presentation-verification-contract-review-pass-2.md`.
- `2026-05-27`: Implementation wave 1 moved CLI diagnostic formatting onto `sifr_diagnostics` presentation helpers after recovery limiting; added source-aware human rendering, stable compact one-line rendering, multiline and synthetic presentation contract fixtures, JSON schema lock evidence, and the phase-owned contract checker with negative self-tests.
- `2026-05-27`: Claude implementation review pass 4 returned `SATISFIED` with no blockers. It verified CLI delegation, human/compact/JSON contracts, fixtures, checker/self-test wiring, docs, and execution tracker evidence. Review artifact: `reviews/diagnostic-presentation-implementation-review-pass-4.md`.

## Validation Log

- `cargo test -p sifr_diagnostics -q` -> passed (`32 passed`), covering source-aware human rendering, multiline span rendering, CRLF terminal normalization, related spans, suggestions, spanless diagnostics, compact one-line output, and JSON preservation.
- `cargo test -p sifr -q diagnostic -- --nocapture` -> passed (`28 passed`), covering CLI diagnostic output routing, recovery-limited stream sharing, compact output snapshots, and command-level diagnostic format behavior.
- `python3 scripts/run_verification_hardening.py --profile pr --suite diagnostics --bless` -> passed and regenerated the locked diagnostics baselines for `decimal_invalid_literal` and `multiline_span_rendering`.
- `python3 scripts/run_verification_hardening.py --profile pr --suite diagnostics` -> passed (`variants=6, failures=0`).
- `python3 scripts/check_diagnostic_baseline_hygiene.py && python3 scripts/check_diagnostic_schema_sync.py && python3 scripts/check_diagnostic_docs_sync.py && python3 scripts/check_diagnostic_code_coverage.py` -> passed.
- `python3 verification/tooling/check_diagnostic_presentation_contract.py` -> passed.
- `python3 verification/tooling/check_diagnostic_presentation_contract.py --self-test` -> passed, including negative checks for missing fixture, missing baseline, missing schema field, and missing run-all wiring.
- `scripts/run_all_tests.sh --profile quick` -> completed through quick-lane guardrails, diagnostic presentation contract check/self-test, tooling checks, representative validation contracts, and quick e2e pass suite (`67 passed, 0 failed`). The latest validation report was written to `target/validation_lane_reports/quick.latest.json`.
- Phase-owned fixtures are `decimal_invalid_literal`, `multiline_span_rendering`, and `presentation_contract_cases`; the checker enforces severity-only compact summaries and one-line-per-retained-diagnostic output against those baselines.
- Phase-owned fixtures are `decimal_invalid_literal`, `multiline_span_rendering`, and `presentation_contract_cases`; the checker enforces severity-only compact summaries and one-line-per-retained-diagnostic output against those baselines.

## PR Log

- Implementation PR links will be recorded per milestone after they are opened and merged.
