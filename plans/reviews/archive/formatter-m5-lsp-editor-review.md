

## M5 Review: No Blockers Found

After reviewing the uncommitted superproject diff plus nested `editor_integrations` and `editor_integrations/vscode` branches, **Milestone 5 is approved to merge** and M6 may begin after updating tracker/PR links.

---

### Blocker Assessment

| # | Requirement | Status | Evidence |
|---|-------------|--------|----------|
| 1 | LSP advertises formatting only when `sifr.format.enable` is true | ✅ PASS | `capabilities.rs:26` — `server_capabilities(format_enable: bool)`; formatting capabilities conditionally added at lines 92-94. Smoke test `run_disabled_formatting_check()` confirms `formatEnable: False` omits both providers. |
| 2 | LSP rejects formatting when disabled post-init | ✅ PASS | `formatting.rs:45-51` — `ensure_formatting_enabled()` returns `LspError::method_not_found` (-32601). Stress test at lines 87-100 verifies toggle: disable → error, re-enable → valid edits. |
| 3 | LSP remains protocol shell (no direct `sifr_format` dep) | ✅ PASS | `sifr_lsp` has zero `sifr_format` imports. `sifr_analysis/src/lib.rs:380-384` exposes `format_options_for_path()` bridge. All formatting routes: `sifr_lsp` → `sifr_analysis::host` → `sifr_format`. |
| 4 | Analysis uses Ruff-backed `sifr_format` API | ✅ PASS | `implementation.rs:401-410` — `format_document()` calls `sifr_format::format_source()`, converts to single `TextEdit` with `full_range()`. `format_range()` at line 422 uses `sifr_format::format_range()` with proper range semantics. |
| 5 | Config discovery consistent with CLI | ✅ PASS | `sifr_format/src/config.rs:63-69` — `effective_format_options_for_file()` discovers `sifr.toml` via ancestors. `check_and_package_commands.rs:52-61` — `format_cli_overrides()` bridges CLI args to `FormatConfigOverrides`. Single source of truth. |
| 6 | Protocol tests cover formatting scenarios | ✅ PASS | Smoke: document/range formatting (lines 143-169), disabled capability (lines 172-190). Stress: invalid range error codes (lines 73-85), settings toggle + re-enable (lines 87-100). |
| 7 | Editor assets use LSP formatting only | ✅ PASS | Neovim: `vim.lsp.buf.format` + `BufWritePre` (sifr.lua:18-25). Zed: `formatter = { language_server }` (config.toml:8). Helix: `auto-format = true` (languages.toml:8). Emacs: `eglot-format-buffer` + save hook (sifr-mode.el:13-20). No `sifr fmt` fallback. |
| 8 | File-size guardrail adequate | ✅ PASS | 7,474 lines total across crates. New `sifr_format::config` (291 lines), `sifr_lsp::settings` (74 lines). Deleted `formatter_config.rs` (232 lines). Net -158 lines. |

---

### Validation Evidence (from your run)

All passed:
- `cargo fmt --check`
- `cargo test -p sifr_lsp -p sifr_analysis -p sifr_format`
- `cargo build -p sifr`
- `python3 verification/tooling/lsp_protocol_smoke.py`
- `python3 verification/tooling/lsp_protocol_stress.py`
- `python3 verification/tooling/check_editor_assets.py`
- `python3 verification/tooling/check_tooling_contract_lock.py`
- `scripts/run_all_tests.sh --profile quick`
- `python3 scripts/check_file_size_guardrails.py`
- `git diff --check`

---

### Next Steps

1. **Update tracker**: Mark M5 complete in `issues/ad-hoc-production-grade-sifr-formatter-execution.md`
2. **Merge nested PRs**:
   - `editor_integrations`: https://github.com/sifr-lang/editor-integrations/pull/1 (codex/m5-lsp-formatting-assets)
   - `editor_integrations/vscode`: https://github.com/sifr-lang/sifr-vscode/pull/2 (codex/m5-lsp-formatting-docs)
3. **Open superproject PR** for branch `codex/formatter-m5-lsp-editor-parity`
4. **Begin M6**: Production formatter formatting correctness enforcement
