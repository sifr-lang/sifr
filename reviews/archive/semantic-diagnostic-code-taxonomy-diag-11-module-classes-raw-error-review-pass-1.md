# Review: semantic-diagnostic-code-taxonomy-diag-11-module-classes-raw-error-review-pass-1

Branch: `codex/diag-11-raw-hir-module-classes`

Date: 2026-05-03

Reviewer: Claude (`.cursor/skills/talk-to-claude`)

## Scope

- Add `SIFR-IMPORT-0003` / `IMPORT_UNSUPPORTED_FORM`.
- Add `SIFR-IMPORT-0004` / `IMPORT_PRIVATE_MEMBER`.
- Add `SIFR-CLASS-0005` / `CLASS_INVALID_BASE`.
- Add `SIFR-CLASS-0006` / `CLASS_UNSUPPORTED_DECLARATION`.
- Migrate raw `ctx.error(String)` sites in `crates/sifr_hir/src/lower/mod.rs` and `crates/sifr_hir/src/lower/classes.rs` to structured code/range diagnostics.
- Add HIR tests, e2e fail fixtures, generated docs, and transport cleanup guardrail coverage.

## Findings

Required fixes: none.

The reviewer found the taxonomy correct:

- Unsupported import syntax belongs in the `IMPORT` family as `IMPORT_UNSUPPORTED_FORM`.
- Private imported members belong in the `IMPORT` family as `IMPORT_PRIVATE_MEMBER`.
- Invalid class bases belong in the `CLASS` family as `CLASS_INVALID_BASE`.
- Unsupported class-body declarations belong in the `CLASS` family as `CLASS_UNSUPPORTED_DECLARATION`.

The reviewer found the source ranges correct:

- Unsupported bare import statements point at the imported module alias.
- Unsupported relative and bare relative import forms point at the import statement.
- Private imported members point at the private imported name.
- Invalid class bases point at the parent class name where available.
- Unsupported field defaults point at the default expression.
- Unsupported class-body statements point at the unsupported statement.

The reviewer found the implementation clean:

- `classes.rs` and `mod.rs` have no remaining raw `ctx.error(` calls.
- Both files are listed in `RAW_HIR_ERROR_FREE_FILES`.
- The stdlib private-member path now emits the same structured private-import diagnostic instead of falling through to a missing-member diagnostic.

The reviewer found test and docs coverage complete:

- HIR tests cover unsupported import forms, private imports, unknown class bases, and unsupported class field defaults.
- E2E fixtures cover each new diagnostic code and expected primary column.
- Generated docs and diagnostic registry entries are consistent.

## Status

Approved / satisfied.

Claude note: the review completed successfully, but the reviewer process could not write this file directly due to permission restrictions. The review output reported no required fixes.
