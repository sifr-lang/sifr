# Review: milestone_diag_10 slice 4 - structured HIR warnings, pass 1

Reviewer: Claude
Date: 2026-05-03
Branch: `codex/diag-10-structured-hir-warnings`

## Verdict

Not satisfied.

The structured warning transport was mechanically sound, but the reviewer raised one required policy question and two follow-up items.

## Findings

1. `crates/sifr_hir/src/lower/typing_and_functions.rs` changed the old panic-boundary CFG fallback from a non-fatal warning into an internal compiler error. The reviewer initially treated this as a behavioral regression and recommended adding an internal warning variant.

2. `crates/sifr_driver/src/project/frontend.rs` needed confirmation that project frontend diagnostic emission did not drop child diagnostics/help while rendering structured warnings and reveal notes.

3. `crates/sifr_driver/src/tests/single_file_frontend.rs` had source-backed `type_check_source` coverage for arithmetic overflow warnings, but no symmetric source-backed unreachable-statement warning test.

## Resolution Plan

- Keep the CFG path fail-closed as an internal compiler error because this phase and repo policy explicitly reject fallback paths for invalid compiler state.
- Preserve child diagnostics/help in the project frontend emission path.
- Add the missing `type_check_source` unreachable-statement structured warning coverage.
- Run a second review round with the no-fallback policy made explicit.
