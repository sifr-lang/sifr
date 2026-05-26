Now I have a comprehensive picture. Let me compile my final review findings.

---

# Final Production-Readiness Review: Ad Hoc Production-Grade Sifr Formatter

## Review Status

**Phase is APPROVED for final local validation and closure.**

No blocking issues found. All 15 acceptance criteria are implemented or documented. All locked capability matrix rows are satisfied. The single formatter core invariant holds. Parser/AST extension blocking is enforced. Docs and evidence are sufficient.

---

## 1. Acceptance Criteria AC-1 through AC-15

### AC-1: Ruff-backed `format_source`
- `crates/sifr_format/src/lib.rs:71-88` — `format_source` calls `ruff_python_formatter::format_sifr_module_source`, then re-parses through `sifr_syntax::parse_module` for roundtrip verification.
- ✅ **IMPLEMENTED**

### AC-2: `--check` exits cleanly / nonzero on drift
- `crates/sifr_format/src/lib.rs:90-105` — `check_source` returns diagnostics via `formatting_drift_diagnostic` (`crates/sifr_format/src/lib.rs:404-434`) with stable code `SIFR-FMT-0001` when drift exists.
- ✅ **IMPLEMENTED**

### AC-3: Idempotence
- `crates/sifr_format/src/lib.rs:523-536` — Unit test `formatter_is_ruff_backed_and_preserves_string_contents` runs two passes and asserts equality.
- ✅ **IMPLEMENTED**

### AC-4: Parser roundtrip
- `crates/sifr_format/src/lib.rs:80-83` — `parse_module(&formatted, ...)` is called on every format result. `crates/sifr_format/src/lib.rs:127-130` — `parse_module` also called for range formatting roundtrip.
- ✅ **IMPLEMENTED**

### AC-5: All Sifr syntax extensions have formatter coverage
- `verification/tooling/formatter_manifests/ast_coverage.json` — 11 coverage rows for param conventions, types, generics, match/case, collections, pragmas, docstring snippets.
- `verification/tooling/check_formatter_ast_coverage.py` — Fails when any extension is absent from both fork and wrapper fixtures.
- `crates/sifr_format/src/lib.rs:539-547` — Test `formatter_canonicalizes_sifr_parameter_conventions` validates `mut own` → `own mut`.
- ✅ **IMPLEMENTED**

### AC-6: `mut own` canonicalizes to `own mut`
- `crates/sifr_format/src/lib.rs:539-547` — Explicit test assertion `assert!(formatted.contains("def consume(own mut data: ..."))` and `assert!(!formatted.contains("mut own data"))`.
- ✅ **IMPLEMENTED**

### AC-7: Comments, blank lines, pragmas
- `crates/sifr_format/src/lib.rs:6` — `ruff_python_formatter::format_sifr_module_source` uses Ruff comment trivia.
- `verification/tooling/formatter_manifests/ast_coverage.json:15` — `formatter_pragmas` row requires `fmt: off`, `fmt: on`, `fmt: skip`, `yapf: disable/enable`.
- ✅ **IMPLEMENTED**

### AC-8: Formatter configuration
- `crates/sifr_format/src/config.rs` — Full config system with 11 keys, snake_case/kebab-case aliases, CLI override precedence, `--isolated`, unknown-key rejection (line 200-203), Python-only option rejection (line 195-198).
- ✅ **IMPLEMENTED**

### AC-9: CLI, analysis, LSP formatter equivalence
- `crates/sifr_format/src/lib.rs:71-88` — Single `format_source` entry.
- `crates/sifr_analysis/src/host/implementation.rs:394-425` — Both `format_document` and `format_range` call `sifr_format::format_source` / `sifr_format::format_range`.
- `crates/sifr_lsp/src/requests/formatting.rs:9-22,24-43` — LSP handlers delegate to `host.format_document` / `host.format_range` with identical `FormatOptions` conversion.
- ✅ **IMPLEMENTED**

### AC-10: Range formatting minimal/stable edits, no bypass
- `crates/sifr_format/src/lib.rs:107-135` — `format_range` returns `Vec<TextEdit>` from `ruff_format_sifr_range`, validates character boundaries (line 114), skips no-op edits (line 122), and roundtrip-verifies (line 125-130).
- ✅ **IMPLEMENTED**

### AC-11: Invalid source reports diagnostics
- `crates/sifr_format/src/lib.rs:313-347` — `format_module_error_diagnostic` maps `FormatModuleError::ParseError` to `"formatter could not parse Sifr source"`.
- `crates/sifr_format/src/lib.rs:574-582` — Test confirms parse errors return diagnostic, not partial output.
- ✅ **IMPLEMENTED**

### AC-12: All validation wired
- `scripts/run_all_tests.sh:140-145` — `check_formatter_contract.py`, `check_formatter_phase_manifests.py`, `check_formatter_ast_coverage.py` (each with self-test) all wired.
- `verification/performance/budgets.json:611,631` — `perf.formatter.corpus.project_check` and `perf.formatter.large_file.check` budgets present.
- ✅ **WIRED**

### AC-13: Docs cover all required areas
- `docs/formatter.md` (149 lines) — Commands, config keys, LSP-first editor behavior, preview, cache, Python-only options, guardrail commands.
- `docs/cli_command_semantics.md:56-61` — Formatter command semantics documented.
- ✅ **IMPLEMENTED**

### AC-14: AST coverage guardrail
- `verification/tooling/check_formatter_ast_coverage.py` — Requires fork fixture + wrapper fixture for each of 11 extension rows.
- `verification/tooling/formatter_manifests/ast_coverage.json` — 11 rows, each with `fork_fixture` and `sifr_wrapper_fixture`.
- ✅ **IMPLEMENTED**

### AC-15: All 5 editors document LSP formatter
- `docs/formatter.md:134-136` — Neovim, Zed, Helix, Emacs, VS Code all named; LSP-first.
- `internal_docs/editor_integrations.md:15-20` — LSP-first rule, `sifr fmt` reserved for CLI.
- `internal_docs/vscode_extension.md:46-47` — Extension "must not implement formatter logic".
- `verification/tooling/check_editor_assets.py:88-89` — Explicit check for direct formatter fallback wiring.
- ✅ **IMPLEMENTED**

---

## 2. Ruff Capability Matrix Parity

All 40 rows from `verification/tooling/formatter_manifests/capability_matrix.json` are implemented or correctly classified:

| Classification | Count | Status |
|---|---|---|
| `supported` | 26 | ✅ All implemented |
| `adapted` | 11 | ✅ All implemented |
| `not-applicable` | 2 (notebook, import sorting) | ✅ Documented rationale |
| `not-exposed` | 1 (`--target-version`) | ✅ Rejected in config (`config.rs:195-198`) and documented |

**Notable implementation verifications:**
- `config.rs:195-198` — Python-only `--target-version`/`--extension` are deterministically rejected
- `config.rs:195-198` — Unknown keys fail with diagnostic
- `lib.rs:268-273` — Line ending hardcoded to `LineEnding::LineFeed` (AC-8)

---

## 3. Single Formatter Core Invariant

The single formatter core (`ruff_python_formatter::format_sifr_module_source` / `format_sifr_range`) is shared by:

| Layer | Entry point | File:line |
|---|---|---|
| CLI `sifr fmt` | `sifr_format::format_source` | `lib.rs:71-88` |
| Analysis `format_document` | `sifr_format::format_source` | `host/implementation.rs:394-411` |
| Analysis `format_range` | `sifr_format::format_range` | `host/implementation.rs:414-425` |
| LSP document formatting | `host.format_document` | `formatting.rs:15-21` |
| LSP range formatting | `host.format_range` | `formatting.rs:36-41` |

**Invariant holds**: No layer bypasses `sifr_format`. LSP is a protocol adapter only. `format_options` conversion is shared between CLI and LSP (`formatting.rs:54-74`).

---

## 4. Parser/AST Extension Blocking

- `verification/tooling/check_formatter_ast_coverage.py:26-38` — `REQUIRED_EXTENSIONS` set of 11 Sifr-specific AST markers.
- `verification/tooling/check_formatter_ast_coverage.py:40-52` — `AST_MARKERS` maps each extension to file/pattern pairs in the Ruff fork.
- `verification/tooling/formatter_manifests/ast_coverage.json` — Every row has `fork_fixture` + `sifr_wrapper_fixture`.
- Future parser extensions without both fixtures **will fail the guardrail**.
- ✅ **ENFORCED**

---

## 5. Docs and Execution Evidence

| Item | Location | Finding |
|---|---|---|
| Public formatter guide | `docs/formatter.md` (149 lines) | ✅ Complete |
| CLI command semantics | `docs/cli_command_semantics.md:56-61` | ✅ Formatter section added |
| README link | `README.md:193` | ✅ Links to `docs/formatter.md` |
| Architecture | `internal_docs/architecture.md:276-294` | ✅ Formatter Architecture section updated |
| Tooling analysis | `internal_docs/tooling_analysis.md:103-123` | ✅ Ruff-backed, canonical `own mut` |
| Tooling verification | `internal_docs/tooling_verification.md:143-169` | ✅ Formatter hardening checks section added |
| LSP server | `internal_docs/lsp_server.md:83-93` | ✅ Settings and config discovery |
| Editor integrations | `internal_docs/editor_integrations.md:15-20` | ✅ LSP-first rule |
| VS Code extension | `internal_docs/vscode_extension.md:46-47` | ✅ Forbidden formatter behavior |
| Phase 36 reference | `internal_docs/phases/36_...md:20` | ✅ References formatter phase |
| Execution tracker | `issues/ad-hoc...-execution.md` | ✅ All 7 milestones, all PRs, showcase evidence |
| Editor integrations submodule | `editor_integrations/` at `8b0be19` | ✅ PRs #1, #2, #3 included |

**Submodule audit:**
```
third_party/ruff: 8b95ca3d88 (sifr-lang/ruff#3 — docstring snippet support, M6)
editor_integrations: 8b0be19 (PRs #1, #2, #3 — all M7 editor docs PRs merged)
```

---

## 6. Validation Gaps

The execution tracker (`issues/ad-hoc-production-grade-sifr-formatter-execution.md`) records this item as pending:
- `[ ] Full local validation recorded`
- `[ ] Final production-readiness review approved`

**These are the only remaining items before phase closure.** The targeted M7 validation passed:

```
python3 verification/tooling/check_editor_assets.py          # PASS
python3 verification/tooling/check_formatter_ast_coverage.py  # PASS
python3 verification/tooling/check_formatter_phase_manifests.py # PASS
cargo fmt -p sifr --check                                    # PASS
git diff --check                                              # PASS
```

The full validation (`scripts/run_all_tests.sh --profile quick` and `scripts/run_all_tests.sh`) must be recorded before closing, but **no implementation blockers exist**.

---

## Minor Observations (Non-Blocking)

1. **`crates/sifr_format/src/lib.rs` is 628 lines and `config.rs` is 312 lines** — Both under the 900-line cap.
2. **`docs/formatter.md` is new** — No prior version to diff; content verified against execution tracker validation logs and implementation cross-reference.
3. **`verification/tooling/check_formatter_contract.py` self-test passed** — Corroborated by execution tracker.
4. **`perf.formatter.corpus.project_check` and `perf.formatter.large_file.check`** are the two named budget entries per `verification/performance/budgets.json:611,631`.

---

## Closeout Checklist

- [x] AC-1: Ruff-backed `format_source` (`lib.rs:71-88`)
- [x] AC-2: `--check` with deterministic diagnostics (`lib.rs:90-105`)
- [x] AC-3: Idempotence test (`lib.rs:523-536`)
- [x] AC-4: Parser roundtrip on every format (`lib.rs:80-83, 127-130`)
- [x] AC-5: 11 AST coverage rows with fixtures
- [x] AC-6: `mut own` → `own mut` canonicalization (`lib.rs:539-547`)
- [x] AC-7: Comments and pragmas via Ruff formatter
- [x] AC-8: Full config system with precedence and diagnostics
- [x] AC-9: Single formatter core shared by CLI/analysis/LSP
- [x] AC-10: Range formatting with no bypass (`lib.rs:107-135`)
- [x] AC-11: Invalid source diagnostic, no partial output
- [x] AC-12: Validation wired: contract, manifests, AST coverage, budgets
- [x] AC-13: All required documentation areas covered
- [x] AC-14: AST coverage guardrail implemented and wired
- [x] AC-15: All 5 editors document LSP-only formatting
- [x] All 40 capability matrix rows satisfied
- [x] Single formatter core invariant verified across 4 layers
- [x] Parser/AST extension blocking enforced
- [x] Docs complete and consistent with implementation
- [x] Execution tracker has all PR links and showcase evidence
- [x] Submodule pointers at M6 commits: ruff `8b95ca3d`, editor_integrations `8b0be19`
- [ ] Full local validation (`--profile quick` + full) recorded
- [ ] This review artifact finalized

---

**The phase is approved for final local validation and closure. No blocking issues found.**
