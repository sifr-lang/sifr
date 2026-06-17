Both nits confirmed in the current files:
- Line 652: standalone contract-check exit bullet [ok]
- Lines 239-242: inline PR-to-milestone labels ("belongs to") [ok]

**Final confirmation: READY**

No blocking implementation-readiness gaps remain. All three documents are internally consistent on ownership boundaries, sequential dependencies, and cross-repo validation contracts. Phase 35 and Phase 36 have clear entry/exit criteria, and the VS Code extension issue accurately reflects its `milestone_36_7` position within Phase 36.

**Non-blocking notes:**
1. LSP query budget defaults (Phase 36, lines 414-431) are phase-start values requiring baseline evidence before enforcement - by design.
2. `verification/tooling/vscode_extension_rules.json` is referenced across multiple documents (Phase 36 verification infra, issue cross-repo validation, issue PR sequence) - confirm the file name is stable when `milestone_36_1` locks the contract.
