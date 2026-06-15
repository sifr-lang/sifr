# Wave 4 Lint Compact Baselines — Review Pass 1

## Blockers

None. All six verification dimensions check out.

## Verified

1. **Lint diagnostics emit exactly the intended code per fixture, no incidental output.** Each `lint-compact.stderr.txt` contains the compact summary line plus a single `W SIFR-LINT-000X` for the matching code (`0001`/unknown, `0002`/unused, `0003`/blanket, `0004`/trailing-whitespace, `0005`/TODO, `0006`/boolean-positional, `0007`/large-parameter-list, `0008`/duplicate-import). All `lint-compact.stdout.txt` files are 0 bytes; all `exit-code.txt` files are `1`, matching `manifest.json` `expect_exit_code: 1` and `crates/sifr/src/lint_cli.rs:597-599` (`EXIT_USER_DIAGNOSTIC` for any diagnostic, warning or error). Column positions in baselines line up with the fixture sources (e.g., `lint_trailing_whitespace:1:12` — `def main():` is 11 chars, trailing whitespace starts at col 12; `lint_large_parameter_list:1:1` — function span anchored at the `def`).

2. **Adding `"lint"` to `BASELINE_COMMANDS` (verification/runner/sifr_verify/area_adapter.py:24) is the right shape.** The adapter only uses the set to gate `case["command"]` membership and to build `cargo run -- [--diagnostic-format <fmt>] <command> <entry>`. `lint` follows the same pipeline as `check`/`build`/`run`/`test`, and `crates/sifr/src/lint_cli.rs:172-174` correctly honors the global `--diagnostic-format compact` flag when `--output-format` is unset. No validation paths are relaxed; the unique-artifact-path collision check (area_adapter.py:391) still applies to lint variants because labels are now `lint-compact`, distinct from `check-*`.

3. **Coverage-checker label normalization is correct for both `check-*` and `lint-*`.** `label.rsplit("-", maxsplit=1)[-1]` (verification/areas/diagnostics/checks/code_baseline_coverage.py:117) maps:
   - `check-human → human`, `check-json → json`, `check-compact → compact`
   - `lint-compact → compact`
   
   Confirmed `find … -name "*.txt"` shows only these four labels exist under `verification/areas/diagnostics/fixtures/diagnostics/*/baselines/`. Renderers in `ALLOWED_RENDERERS = {"human","json","compact"}` are single-token, so a "last segment after final hyphen" rule is safe today.

4. **`.gitattributes` is narrowly scoped and justified.** Single line, single file path: `verification/areas/diagnostics/fixtures/diagnostics/lint_trailing_whitespace/main.sifr whitespace=-blank-at-eol`. The opt-out is necessary because `SIFR-LINT-0004` requires real trailing whitespace in the fixture, which `git diff --check` would otherwise flag. File has a trailing newline. Repo-wide `git diff --check` remains active for every other file.

5. **Manifest, coverage, metadata, and source hashes are consistent.**
   - All 8 lint manifest cases live in suite `baselines` with `command: "lint"`, `expect_exit_code: 1`, `diagnostic_formats: ["compact"]`, and entries under `lint_*/main.sifr`. They are placed in correct alphabetical position within the suite (between `hir_*` and `parser_*` cases).
   - Coverage entries flip the eight `SIFR-LINT-0001..0008` codes from `deferral` blocks to `baseline_fixture_id: lint_<name>` with `renderer_formats: ["compact"]`. Fixture IDs match the `code → fixture` mapping in the slice spec.
   - All 8 source hashes in `baseline_metadata.json` match recomputed `sha256(main.sifr)`:
     - `lint_blanket_suppression`: `8a6b…f5f` ✓
     - `lint_boolean_positional_argument`: `6635…706d` ✓
     - `lint_duplicate_import`: `9b4f…ff21` ✓
     - `lint_large_parameter_list`: `2090…5f8c` ✓
     - `lint_todo_comment`: `9b02…2af1` ✓
     - `lint_trailing_whitespace`: `0257…4792` ✓
     - `lint_unknown_suppression`: `3d2d…e283` ✓
     - `lint_unused_suppression`: `ffcd…3662` ✓
   - All eight metadata entries carry `owner: compiler/diagnostics`, `renderer: compact`, `suite: baselines`, and a non-empty `bless_reason`/`bless_reference`, satisfying `validate_baseline_metadata` (code_baseline_coverage.py:305-314).

6. **Tracker coverage counts and remaining deferral families are accurate.** Programmatic count of `code_baseline_coverage.json` yields `total=170, deferred=56, covered=114`. Deferred-family Counter: `BUILD=6, ENCODING=1, FMT=1, INTERNAL=1, IO=2, PACKAGE=34, STDLIB=3, WORKSPACE=8` → sums to 56, exactly matching the tracker prose at plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:889. No `LINT` deferrals remain.

## Non-blocking observations

- **Label normalization is positional, not enumerated.** `rsplit("-", maxsplit=1)[-1]` returns whatever comes after the last hyphen with no `ALLOWED_RENDERERS` cross-check at that site (verification/areas/diagnostics/checks/code_baseline_coverage.py:117). Today the only labels in tree are `check-*` and `lint-compact`, all single-hyphen, and downstream `validate_baseline_metadata` would catch a typo via the `metadata renderer is not in manifest formats` path. Still, an explicit `if renderer not in ALLOWED_RENDERERS: continue` (or an assertion) at line 117 would localize the contract and prevent silently-skipped files from masquerading as "no renderer for this fixture". Optional hardening, not required for this slice.
- **`synthetic_files` hardcodes `check-` prefix** (verification/areas/diagnostics/checks/code_baseline_coverage.py:271). Fine for the current synthetic baselines (all three are `check` command), but the slice introduces a precedent that future commands besides `check` might need synthetic coverage. If/when that lands, this line needs a command field on the metadata entry. Worth flagging but not in scope here.
- **All eight lint baselines are warning-only.** Since `cargo test -p sifr_lint` passed and the compact summary line shows `0 errors, 1 warning, 0 notes`, the warnings-exit-1 semantics in `render_lint_diagnostics` are exercised by the slice — small implicit win, no change requested.
- **Fixtures are minimally framed (mostly `def main(): pass`).** That's the correct minimum-surface-area choice for these baselines: anything richer risks triggering incidental lint diagnostics. Not a finding — just confirming the small fixtures are intentional.

## Validation status

Approved for PR with no required changes. Local focused validation already covers the path: `areas run --area diagnostics --suite baselines (--bless)` at 112 cases / 140 variants, `--suite contracts` passing, `cargo test -p sifr_lint` green, `python3 -m py_compile` on both modified Python files, file-size guardrail, `git diff --check`, and `--self-test`.

Before opening the PR, run `scripts/run_all_tests.sh --profile create-pr` per the project workflow (the tracker entry already references the merge-gate run but does not yet record a hash for this slice — that should land in the tracker once the merge-profile run completes, mirroring the format used by the fifth Wave 4 slice's entry).

No further review rounds required before PR submission.
