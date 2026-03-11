## [Task] Replace Sifr file `# expect-stdout` checks with native Sifr assertions

## Goal
Replace stdout-based behavioral checks in `.sifr` files (`# expect-stdout`) with Sifr assertions so tests are expressed in language-native form
and stop depending on output matching via harness comments.

## Scope (current observed state)
- Total files with `# expect-stdout`: **631**
- Total expected lines: **2049**
- Location split:
  - `audits/leetcode`: **208** files / **450** expectations
  - `crates/sifr/tests/e2e/pass`: **387** files / **1138** expectations
  - `demos`: **36** files / **461** expectations
- Exclusions:
  - `# expect-error` only files (error-mode checks)
  - `# expect-stderr` files
  - Non-target runtime-fail fixture behavior unless clearly stdout-based

## Observations (important for migration safety)
- `# expect-stdout` files already importing `sifr.test`: **47**
- Files with `# expect-stdout` and no `print(...)`:
  - `crates/sifr/tests/e2e/pass/stdlib_logging.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_logging_class.sifr`
  - `crates/sifr/tests/e2e/pass/logging_basic_config.sifr`
- Bucket by print-vs-expected line count:
  - exact match (`print == expect`): **513** files
  - mismatch requiring manual intervention: **118** files

## Execution Plan

### 1) Preparation
- Create migration log in this issue with three buckets:
  - `simple` (exact 1:1 conversion)
  - `manual-extra-prints` (more prints than expects)
  - `manual-extra-expects` (more expects than prints)
- Freeze conversion rules:
  - `print(expr)` + `# expect-stdout: X` → `assert expr == X`
  - Keep `true/false` expectations as direct bool asserts where natural.
  - Preserve side-effect/logging behavior tests as-is if they are explicitly validating output channels.

### 2) Automated conversion for `simple` bucket
- Convert each `# expect-stdout` line to an assertion tied to the immediately related `print` sequence.
- Remove the consumed `# expect-stdout` comment lines.
- Validate no new semantic statements are introduced; keep existing control flow and variable names unchanged.
- Prioritize by smallest risk:
  1. `audits/leetcode` (mostly algorithmic I/O assertions)
  2. `crates/sifr/tests/e2e/pass`
  3. `demos`

### 3) Manual conversion for mismatch bucket
- `print > expect`:
  - remove/logical-noise prints OR keep prints and add asserts for expected lines where behavior is asserted.
- `expect > print`:
  - find missing assertions or implicit behavior and convert to explicit assertions.
- `0 print` files:
  - rewrite with capture-based assertions or replace with direct non-stdout validation where possible.

### 4) Post-migration consistency pass
- Remove all remaining `# expect-stdout` comments in converted files.
- Keep `# expect-error` and `# expect-stderr` untouched unless explicitly out-of-scope and approved.
- Deduplicate and normalize assertion style:
  - Prefer existing `sifr.test` helpers when file already imports them and behavior matches.
  - Prefer native `assert` form when replacing pure debug prints.

### 5) Validation
- Add/track checks in issue:
  - no `# expect-stdout` remains in converted files.
  - no behavior regressions observed in existing e2e command flow
  - per-folder conversion counts match target:
    - `audits/leetcode`
    - `crates/sifr/tests/e2e/pass`
    - `demos`

## Acceptance Criteria
- All targeted `# expect-stdout` usage removed.
- All previously validated outputs are represented via assertions in Sifr.
- No expected-output checks accidentally moved out of test intent (especially in exception/safety/demo paths).
- No regressions in existing `e2e` pass collection behavior.

## Rollout Strategy
- Do by folder batches to keep review easy:
  1. `demos` (36 files)
  2. `crates/sifr/tests/e2e/pass` (387 files)
  3. `audits/leetcode` (208 files)
- Commit per batch, with a short checklist for each batch:
  - converted files
  - skipped/manual exceptions
  - risky/owner-review items

## Notes
- This is a large mechanical migration and should be done in scripted passes with explicit exceptions list so intent is auditable.
- Any file requiring semantic interpretation beyond straightforward print replacement should be documented in this issue before changing.

## Migration To-Do (Execution Tracking)

### Rollout Checklist

- [x] Part 1 - demos: convert all `simple` files and review demos for behavior retention
- [x] Part 2 - crates/sifr/tests/e2e/pass: convert all `simple` files and review demos for behavior retention
- [x] Part 3 - audits/leetcode: convert all `simple` files and review semantic edge cases
- [x] Manual bucket audit: resolve `manual_*` files with explicit assertions or owner review
- [x] Part 5 - remove `# expect-*` markers from remaining `.sifr` fixtures:
  - [x] Part 5a: convert remaining `# expect-stdout` in pass fixtures to assertions
  - [x] Part 5b: convert remaining `# expect-stderr` in runtime fixtures to explicit failure assertions
  - [x] Part 5c: remove `# expect-error` markers from compile-fail fixtures and negative demo cases
  - [x] Part 5d: use `assert_err` directly for Result error assertions and restore explicit compile-fail `# expect-error` markers

### Current Mechanical Conversion Status

- `audits/leetcode`: 208 / 208 files converted (`# expect-stdout` removed from fixtures)
- `crates/sifr/tests/e2e/pass`: 387 / 387 files converted (`# expect-stdout` removed from fixtures)
- `demos`: 36 / 36 files converted (`# expect-stdout` removed from fixtures)

### Manual Mismatch Resolution Status

All manual mismatch buckets are resolved for the current repo snapshot. No remaining `# expect-stdout` comments in `.sifr` fixtures under:

- `audits/leetcode`
- `crates/sifr/tests/e2e/pass`
- `demos`

### Current Part Result

- Part 4 commit: `66e7541`
- Files converted in this part: 26
- Notes: remaining assertions were explicit behavior checks, including list-accumulation patterns and file-backed log capture where stdout was used only for diagnostics.

- Part 5d follow-up changes:
  - Removed the `assertRaises`/`assertError` wrappers and used `assert_err` directly.
  - Migrated `crates/sifr/tests/e2e/runtime_fail/assert_err_failure.sifr` to `assert_err`.
  - Restored explicit `# expect-error` markers on all `crates/sifr/tests/e2e/fail` fixtures (with `SIFR-*` or `[E####]` expectations).
  - Replaced runtime-panic fixtures `decimal_division_by_zero_runtime.sifr` and `bigdecimal_division_by_zero_runtime.sifr` with direct panic-triggering expressions.
  - Updated compile-fail harness in `crates/sifr/tests/e2e.rs` to assert expected diagnostic codes (`SIFR-*`) and message IDs like `E2507`.

### Note

Manual files should be individually documented with root-cause and intended assertion strategy before any scripted changes.
