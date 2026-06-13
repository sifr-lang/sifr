# Review: semantic diagnostic taxonomy diag-11 type-alias raw HIR diagnostic migration

Status: Approved.

The reviewer is satisfied. No required fixes.

Key findings:

- Both raw `ctx.error` call sites in `type_aliases.rs` were migrated to `ctx.error_with_code_at` with `DiagnosticCode::TYPE_INVALID_ANNOTATION`.
- `value_range` was added to `TypeAliasDecl` so recursive-alias validation can report the alias RHS range.
- The affected recursive-alias tests assert `code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)` and the expected `primary_range`.
- `type_aliases.rs` was added to `RAW_HIR_ERROR_FREE_FILES`.
- Local validations passed: format checks, guardrails, diagnostic transport cleanup, cargo check, full HIR unit tests, clippy, and `scripts/run_all_tests.sh --profile quick`.

Reviewer verdict: satisfied; no required fixes.
