

## Verdict: READY

The plan has been fully revised to address all three blockers and all four sequencing corrections identified in pass-1. Phase 36 is now a production-grade implementation contract, not an MVP. The remaining items are non-blocking nits.

---

### What changed since pass-1

**Phase 35 - all blockers resolved:**

The "Additional production tooling views required for Phase 36" section was added (lines 220-278) with all four missing contracts:
- `TypeDisplayView`, `SignatureView`, `ParameterView` for signature help and hover
- `SymbolTableView`, `SymbolDefinitionView`, `SymbolUseView` for references/rename
- `CodegenPreviewQuery` trait for generated Rust preview

Phase 35 exit gate (lines 282-284) now explicitly closes the split-brain path: "Phase 35 exit is incomplete if any Phase 36 production feature would require `sifr_lsp`, editor extensions, formatter/linter modules, or automation adapters to parse raw Ruff ASTs directly, traverse mutable HIR internals directly, run codegen independently, or derive diagnostics outside `sifr_frontend`/`sifr_diagnostics`."

**Phase 36 - complete rewrite:**

- All MVP language removed: "Phase 36 is not an MVP phase. Every capability listed in this file is required for phase exit unless a later reviewed planning PR explicitly changes the contract before implementation reaches that milestone."
- Expanded from 4 coarse milestones to 8 sequential milestones (36.1-36.8) matching the memo.
- Editor Query Contract covers all 19+ methods (completion, hover, signature help, definition, declaration, type definition, references, prepare-rename, rename, document symbols, workspace symbols, semantic tokens, inlay hints, document highlights, folding ranges, code actions, format, generated rust, explain diagnostic, discover tests, test command).
- Formatter is explicitly in scope (36.2), not a non-goal.
- VS Code Test Explorer is integrated: "VS Code Test Explorer integration backed by Sifr test discovery and CLI test commands" (lines 327-328, milestone_36_7 scope).
- Hard-vs-policy diagnostic classification is explicit (lines 268-281).
- LSP capability matrix lists every required method (lines 200-237).
- Split-brain guardrails are enforced per `tooling_reuse_strategy.md` (lines 189-194).
- Milestone ordering is now: 36.5 LSP Server -> 36.6 Multi-Editor -> 36.7 VS Code -> 36.8 Verification - the grammar-before-VS-code sequencing is correct.

---

### Non-blocking nits (no edits required, but note them)

1. **Roadmap scope note (low priority):** The Phase 36 row in `internal_docs/roadmap.md` could note the expansion to 8 milestones. Not required for implementation readiness.

2. **Extension repository decision (deferred to m36.1):** Phase 36 correctly defers the VS Code extension repo boundary decision to `milestone_36_1`. That's the right place for it.

3. **Test Explorer non-goal framing:** The plan says "test explorer integration is deferred" in non-goals but delivers it in milestone_36_7. The exit criteria (line 638) confirms the Test Explorer integration is required for phase exit. The framing is consistent - the deferral is only for the generic test-explorer protocol beyond VS Code, which is fine.

4. **Formatter milestone_36_2 exit gate clarity:** The definition of done for milestone 36.2 could explicitly name that the formatter must be verified against `sifr_syntax` tokenization fixtures before milestone_36_3 begins relying on it, but this is implied by the overall architecture and not a gap.

None of these block implementation. Phase 35 and Phase 36 are ready to begin.
