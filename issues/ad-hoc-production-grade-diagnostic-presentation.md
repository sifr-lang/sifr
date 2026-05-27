# Ad Hoc Phase: Production-Grade Diagnostic Presentation

Status: planned on 2026-05-27

## Purpose

Make Sifr diagnostic output modes match their intended audiences:

- `human`: the primary developer-facing diagnostic view, optimized for fixing source problems quickly.
- `compact`: a terse, stable, line-oriented view for agents, CI summaries, and quick terminal scanning.
- `json`: the canonical structured diagnostic transport for tools and editor integrations.

The current compiler already carries source span data through diagnostics. The gap is presentation: default human output drops file, line, column, and snippet context, while compact and JSON expose the useful location data.

## Source Inputs

This phase is based on:

- Current diagnostic renderer implementation in `crates/sifr/src/diagnostic_rendering_and_run.rs`
- Canonical diagnostic model and source-map renderer in `crates/sifr_diagnostics`
- HIR diagnostic source ranges in `crates/sifr_hir/src/lower/diagnostic_types.rs`
- Frontend diagnostic source-range rendering in `crates/sifr_frontend/src/query_diagnostics.rs`
- Existing verification baselines under `crates/sifr/tests/verification/diagnostics`
- Public diagnostic code docs under `docs/errors`
- Project diagnostic architecture notes in `internal_docs/architecture.md`

## Current Behavior

For `crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/main.sifr`, the current modes behave as follows.

Human mode omits the source location:

```text
type error: [main] Decimal() received invalid exact literal '12.34.56'
```

Compact mode includes a source location:

```text
summary: 1 error(s), 0 warning(s), 0 note(s), 0 help item(s)
error [SIFR-DECIMAL-0001] [main] Decimal() received invalid exact literal '12.34.56' (x1)
  at crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/main.sifr:3:30
  url: https://sifr.sh/docs/errors/SIFR-DECIMAL-0001
```

JSON mode includes canonical structured span data:

```json
{
  "code": "SIFR-DECIMAL-0001",
  "spans": [
    {
      "file": "crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/main.sifr",
      "line": 3,
      "column": 30,
      "end_line": 3,
      "end_column": 40,
      "lines": [
        {
          "text": "    value: decimal = Decimal(\"12.34.56\")",
          "highlight_start": 30,
          "highlight_end": 40
        }
      ]
    }
  ]
}
```

## Product Decision

Sifr will keep three diagnostic modes, but their contracts must be explicit and mechanically tested.

The canonical presentation boundary is:

- `sifr_diagnostics` owns diagnostic presentation for `human`, `compact`, and `json` once diagnostics have been reduced to canonical `RenderedDiagnostic` values.
- The CLI may apply recovery limits, select the requested format, write to stdout/stderr, and choose command exit status.
- The CLI must not keep a second span-dropping diagnostic renderer for user-facing compiler diagnostics.
- CLI-only status messages such as successful build paths may remain in the CLI, but diagnostic formatting must flow through the canonical presentation renderer.

Implementation must converge the existing CLI-local renderer and the `sifr_diagnostics` presentation renderer into one source-aware path. If a future command needs CLI-specific diagnostic presentation, it must document why the canonical renderer cannot serve it and add a regression test proving no source span is dropped.

### Human mode

Human mode is the default and must be the most readable source-fixing experience.

Target shape:

```text
error[SIFR-DECIMAL-0001]: Decimal() received invalid exact literal '12.34.56'
  --> crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/main.sifr:3:30
   |
 3 |     value: decimal = Decimal("12.34.56")
   |                              ^^^^^^^^^^
   |
   = docs: https://sifr.sh/docs/errors/SIFR-DECIMAL-0001
```

Required behavior:

- Show severity, diagnostic code, and message on the first line.
- Show primary file, line, and column for every diagnostic with a primary span.
- Show source snippets and highlight ranges for primary spans.
- Render visual highlight markers from `DiagnosticSpanLine.highlight_start` and `DiagnosticSpanLine.highlight_end`.
- Handle multiline spans with clear per-line highlighting.
- Normalize or strip CRLF carriage returns before printing snippets to terminals while preserving JSON span text semantics.
- Show related spans when present, without hiding the primary source location.
- Show child notes, help, suggestions, and docs URL in a stable, readable format.
- Preserve spanless internal diagnostics with a clear fallback layout that does not pretend a source location exists.
- Avoid module-prefix noise when the file path already identifies the source context, unless multi-module ambiguity requires it.

### Compact mode

Compact mode is for terse scanning and agent/CI consumption. It should be stable, line-oriented, and easy to parse without being JSON.

Target shape for one diagnostic:

```text
1 error, 0 warnings, 0 notes
E SIFR-DECIMAL-0001 crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/main.sifr:3:30 Decimal() received invalid exact literal '12.34.56'
```

Target shape for multiple diagnostics:

```text
3 errors, 1 warning
E SIFR-NAME-0001 src/main.sifr:8:12 undefined variable `user_id`
E SIFR-TYPE-0002 src/main.sifr:14:20 expected `int`, found `str`
E SIFR-DECIMAL-0001 src/main.sifr:22:30 Decimal() received invalid exact literal '12.34.56'
W SIFR-FLOW-0001 src/main.sifr:31:5 unreachable statement ignored
```

Required behavior:

- Emit a single summary line.
- Emit one physical line per retained diagnostic.
- Use stable fields: severity abbreviation, code, location or `<unknown>`, message.
- Do not group retained diagnostics by message template, rendered message, or primary file. Recovery limiting may still add explicit omission-summary diagnostics before compact rendering.
- Do not emit snippets.
- Do not emit URLs by default unless a future reviewed flag requests verbose compact output.
- Count only diagnostics by severity in the summary. Help items remain visible in `human` and `json`, but are not counted separately in compact mode.
- Preserve deterministic diagnostic ordering and recovery-limit summaries.
- Keep output parseable under spaces in messages by making the first four fields stable.

### JSON mode

JSON mode remains the canonical structured output and should not be optimized for human aesthetics.

Required behavior:

- Preserve the existing `RenderedDiagnostic` schema unless a schema migration is explicitly reviewed.
- Preserve code, severity, message, template, args, URL, spans, children, help, and suggestions.
- Preserve byte ranges and 1-based UTF-8 character line/column positions.
- Preserve source snippets and highlight ranges.
- Continue to serve as the authoritative mode for editor/tool integration tests.

## Scope

In scope:

1. Replace the CLI-local human renderer with a source-aware human renderer.
2. Align CLI human output with `sifr_diagnostics` source-span rendering instead of dropping spans.
3. Redesign compact output into a stable one-line-per-diagnostic format.
4. Keep JSON output structurally stable.
5. Update verification baselines for `human`, `compact`, and `json` where output changes are intentional.
6. Add renderer tests for:
   - primary source span
   - multiline source span
   - related span
   - spanless internal diagnostic
   - help and child notes
   - suggestions
   - CRLF source snippets in human mode
   - compact deterministic ordering
   - compact recovery-limit summaries
   - command-level format-selection regression coverage for `check`, `build`, `run`, and `emit`
7. Update docs for `--diagnostic-format`.
8. Ensure LSP/editor diagnostic behavior is not regressed by CLI presentation changes.
9. Ensure generated docs URLs and diagnostic code identities remain unchanged.
10. Add a phase-owned verification gate that mechanically enforces the diagnostic presentation contract before closure.

Out of scope:

- Changing diagnostic codes or diagnostic family taxonomy.
- Changing type checker, HIR lowering, parser, ownership, or codegen diagnostic semantics.
- Changing the JSON schema except for a separately reviewed schema migration.
- Adding color output. Color can be a later terminal-UX phase.
- Adding machine-readable non-JSON formats.
- Reworking source-map storage or byte-range semantics.

## Milestones

### M1: Renderer Contract Lock

- Add `verification/tooling/check_diagnostic_presentation_contract.py` with positive and negative self-tests.
- Wire `check_diagnostic_presentation_contract.py` and its `--self-test` mode into `scripts/run_all_tests.sh --profile quick`.
- Add focused renderer contract tests for current canonical span data.
- Lock target human and compact output fixtures.
- Confirm JSON schema remains unchanged.
- Add a JSON schema-lock fixture that enumerates the required `RenderedDiagnostic` fields: `code`, `severity`, `message`, `message_template`, `args`, `url`, `spans`, `children`, `help`, and `suggestions`.
- Treat `crates/sifr/tests/verification/diagnostics/decimal_invalid_literal` as the existing locked single-line span fixture for `human`, `compact`, and `json`.
- Add a new multiline diagnostic verification fixture, for example `crates/sifr/tests/verification/diagnostics/multiline_span_rendering`, with locked `human`, `compact`, and `json` baselines.
- Lock snapshot path normalization behavior: verification baselines may use `<WORKSPACE>` for repository-root-relative portability, while live CLI output preserves the display path passed through the diagnostic span.
- Document and test that CLI diagnostic rendering delegates to `sifr_diagnostics` after recovery limiting, while CLI success/status messages remain CLI-owned.
- Record the compact summary format change from `summary: ... help item(s)` to severity-only counts as intentional.
- Record the compact grouping decision as intentional: compact renders one line for each retained diagnostic after recovery limiting and must not reuse the old grouped `CompactKey` behavior.
- Add regression tests proving `check`, `build`, `run`, and `emit` respect the selected diagnostic format; these tests lock existing routing behavior rather than adding new command routing.
- The verification gate must fail until required fixtures, baselines, and renderer contract tests exist.

### M2: Human Source-Aware Output

- Implement default human output with file, line, column, snippet, and highlight.
- Add a highlight renderer for `DiagnosticSpanLine.highlight_start` and `DiagnosticSpanLine.highlight_end`.
- Add CRLF-safe snippet rendering for terminals.
- Render related spans in human mode with their labels/kinds while keeping the primary span first.
- Preserve child notes, help, suggestions, and docs URL.
- Add spanless diagnostic coverage.
- Regenerate affected verification baselines.
- Extend `check_diagnostic_presentation_contract.py` to enforce human-mode source location, snippet highlight, related-span, CRLF, suggestion, and spanless-diagnostic fixture coverage.

### M3: Compact Stable Output

- Replace grouped verbose compact output with stable summary plus one-line diagnostics.
- Preserve deterministic ordering and recovery-limit summaries.
- Add tests covering paths, unknown locations, repeated diagnostics, and omitted diagnostics.
- Regenerate affected verification baselines.
- Extend `check_diagnostic_presentation_contract.py` to enforce compact severity-only summary, one-line-per-retained-diagnostic output, no default URLs/snippets, and no old grouped `CompactKey` behavior.

### M4: Docs And Closeout

- Update public CLI docs and internal diagnostic architecture notes.
- Close all `check_diagnostic_presentation_contract.py` obligations and keep the guardrail wired into `scripts/run_all_tests.sh`.
- Run local validation.
- Record validation evidence in the execution tracker.
- Complete final review before closure.

## Verification Contract

This phase owns a new mechanical verification gate:

```bash
python3 verification/tooling/check_diagnostic_presentation_contract.py
python3 verification/tooling/check_diagnostic_presentation_contract.py --self-test
```

The gate must be wired into `scripts/run_all_tests.sh --profile quick` in M1 and remain active through phase closure.

The checker must verify:

- The existing `crates/sifr/tests/verification/diagnostics/decimal_invalid_literal` fixture continues to provide locked single-line span baselines for `human`, `compact`, and `json`.
- Required verification fixture directories exist for single-line and multiline diagnostics.
- Required `human`, `compact`, and `json` baselines exist for each diagnostic-presentation fixture.
- JSON schema-lock coverage enumerates `RenderedDiagnostic` fields: `code`, `severity`, `message`, `message_template`, `args`, `url`, `spans`, `children`, `help`, and `suggestions`.
- Human-mode baselines include file/line/column, source snippet, visual highlight marker, docs URL, spanless diagnostic fallback, related-span rendering, child note/help rendering, suggestion rendering, and CRLF-safe output coverage.
- Compact-mode baselines use severity-only summaries, one retained diagnostic per line, stable first fields, and no default snippets or URLs.
- CLI regression coverage exists for `check`, `build`, `run`, and `emit` format selection.
- The phase docs and execution tracker mention every required fixture and guardrail.

The checker must include negative self-tests that prove it fails when a required fixture, baseline, field, or run-all wiring entry is missing.

## Acceptance Criteria

- `sifr check <file>` in default human mode shows a usable source location for span-backed diagnostics.
- Human output includes a highlighted source snippet for primary spans.
- Compact output is line-oriented and stable enough for agent/CI scanning.
- JSON output remains schema-compatible and keeps full span data.
- Verification baselines prove the three modes have distinct, intentional responsibilities.
- Verification baselines include both single-line and multiline source-span diagnostics.
- CLI tests prove `check`, `build`, `run`, and `emit` respect `--diagnostic-format` for diagnostics.
- `verification/tooling/check_diagnostic_presentation_contract.py` and `--self-test` pass and are wired into `scripts/run_all_tests.sh --profile quick`.
- No user-facing diagnostic loses code identity, severity, docs URL, or source span data.
- No fallback renderer path hides source-map rendering failures.

## Validation

Minimum validation for implementation PRs:

```bash
cargo fmt --check
cargo test -p sifr_diagnostics
cargo test -p sifr -- --skip test_e2e_pass
python3 verification/tooling/check_diagnostic_presentation_contract.py
python3 verification/tooling/check_diagnostic_presentation_contract.py --self-test
python3 scripts/check_file_size_guardrails.py
scripts/run_all_tests.sh --profile quick
```

Full phase closure validation:

```bash
scripts/run_all_tests.sh
```

If full validation is blocked by an unrelated inherited failure, the execution tracker must record the exact command, failure, and why this phase did not cause it.
