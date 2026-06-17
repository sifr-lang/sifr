All six precision items from pass-1 are addressed:

**Item 1 — `rejected_ruff_config_keys` population:** The schema (phase plan lines 274, 280) now defines it and the validation obligation explicitly references it. The field is `rejected_ruff_config_keys` (not `rejected_sifr_config_keys` — the naming is correct since these are rejected Ruff keys).

**Item 2 — `updated_by_milestone` format:** Phase plan line 567 defines it as `string milestone identifier that last changed the gate, such as "m1" or "m3"`, with `Absent until the first gate change.`

**Item 3 — M1 parser-aware API import check:** Phase plan line 570 adds: "verify that any Sifr rule module whose `suppression_complexity` is not `physical-line` imports or depends on the manifest's `parser_aware_api` path, initially `sifr_lint::suppression::ParserAwareSuppressions`."

**Item 4 — M2 lint CLI module split:** Phase plan line 597 adds: "split lint CLI argument modeling and execution into a dedicated `lint_cli.rs` module before expanding `cli_model_and_entrypoint.rs` or `check_and_package_commands.rs` beyond the hand-maintained file-size guardrail."

**Item 5 — M6 diagnostic-class guardrail:** Phase plan line 704 adds: "add `verification/tooling/check_linter_diagnostic_class.py` with a self-test, or extend `check_lsp_split_brain.py`, so validation fails if LSP code-action handlers offer suppression or fix actions for `Hard` class diagnostics."

**Item 6 — Execution tracker:** Line 76 records the pass-1 review with its precision edit requests and the fact that the phase was updated accordingly.

**No further discovery needed before starting M1.** All pass-1 precision items are encoded in the phase plan. M1 can proceed immediately with manifest encoding, `check_linter_reuse_rules.py` creation, and the suppression-gate manifest initialization.
