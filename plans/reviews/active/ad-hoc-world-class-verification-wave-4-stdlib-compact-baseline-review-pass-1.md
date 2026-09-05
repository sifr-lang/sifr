# Wave 4 stdlib compact baseline — review pass 1

Reviewer: agent
Branch: codex/wave-4-stdlib-compact-baseline
Scope: Tenth Wave 4 diagnostics-baseline slice — purpose-built SIFR-STDLIB-0001 compact baseline + tightened SIFR-STDLIB-0003/0004 deferral rationales.

## Verdict

No blockers. Slice is ready for PR submission after `scripts/run_all_tests.sh --profile create-pr` and merge-gate runs pass. No additional review round required.

## Findings

### [verified] Purpose-built SIFR-STDLIB-0001 coverage, not incidental
- `verification/areas/diagnostics/fixtures/diagnostics/e2e_stdlib_defaultdict_keyword_constructor/main.sifr` imports `defaultdict` from `sifr.collections` and calls it with `default_factory=list`, which is exactly the user-reachable path the lowering check at `crates/sifr_lowering/src/lower/builtin_calls/constructors.rs:552-557` rejects with `STDLIB_UNSUPPORTED_SURFACE` and the message `"defaultdict() does not support keyword arguments"`.
- Compact stderr (`baselines/check-compact.stderr.txt`) shows exactly one diagnostic — `E SIFR-STDLIB-0001 ... main.sifr:6:26 defaultdict() does not support keyword arguments` — with empty stdout and exit code `1`, matching the expectation declared on line 1 of the fixture (`expect-error[col=26]: SIFR-STDLIB-0001`) and `name.range()` resolving to the keyword name `default_factory` at column 26 of line 6.
- The pre-existing incidental fixture `e2e_bare_defaultdict_constructor_rejected` emits SIFR-STDLIB-0001 indirectly via the type-`Any` `append` path while primarily targeting SIFR-NAME-0002; the new fixture cleanly isolates STDLIB-0001 with a single diagnostic, satisfying "purpose-built" ownership.

### [verified] Manifest / metadata / coverage / source-hash internal consistency
- `manifest.json` registers `e2e_stdlib_defaultdict_keyword_constructor` with `command=check`, `expect_exit_code=1`, `diagnostic_formats=["compact"]`. This is the only renderer claimed in the coverage entry (`renderer_formats=["compact"]`), so manifest and coverage agree.
- `baseline_metadata.json` records `source_hash=sha256:dcf4e2f768e51ce646f4a2af23514664177cc7b3021488385e575b0583fc92e0`. Independently computing `shasum -a 256 main.sifr` returns the same digest.
- The metadata `normalizers` list (`workspace-path`, `tmp-path`, `crlf`, `artifact-cache-lines`) matches the pattern used by surrounding Wave 4 entries, and the renderer/owner/bless_reason/bless_reference fields are populated coherently.
- The coverage entry has `deferral=null`, `multi_error_recovery_fixture=null`, `suggestion_rendering_fixture=null`, `renderer_formats=["compact"]` — internally consistent and matches what the baseline directory contains.
- All three baseline files exist (`check-compact.exit-code.txt`, `check-compact.stderr.txt`, `check-compact.stdout.txt`); stdout is empty as required.

### [verified] Tracker coverage counts and remaining deferral-family counts
- Total stable active codes in `code_baseline_coverage.json`: **170**. Codes with `baseline_fixture_id`: **123**. Codes with `deferral`: **47**. Codes with neither: **0**. These exactly match the tracker's "123 now have rendered baseline coverage and 47 carry Wave 4 deferrals" line for the Tenth slice.
- Deferred family breakdown from the JSON: `BUILD=6, INTERNAL=1, PACKAGE=34, STDLIB=2, WORKSPACE=4` — identical to the tracker text.

### [verified] SIFR-STDLIB-0003 / SIFR-STDLIB-0004 deferral rationales are technically honest
- `SIFR-STDLIB-0003` (`STDLIB_BOOTSTRAP_FAILURE`) is emitted only by `crates/sifr_driver/src/stdlib/bootstrap.rs` while compiling baked-in `STDLIB_SOURCES` from the `sifr_stdlib` crate. No user `.sifr` input can reach those branches without first corrupting the compiled-in stdlib sources. The tightened reason ("no current public command can trigger this code without corrupting compiled-in stdlib sources") is accurate.
- `SIFR-STDLIB-0004` (`STDLIB_CACHE_FAILURE`) is only ever constructed in the internal test in `crates/sifr_driver/src/stdlib/cache.rs:53-56`, where a sentinel `Err` is hand-fed into the `OnceLock` cache. The cache module itself never synthesizes a fresh STDLIB-0004; bootstrap never emits it. So no user-driven code path produces STDLIB-0004 without explicit in-process fault injection. The tightened reason ("no current public command can deterministically trigger a cache error without injecting an internal cache failure") is accurate.
- Both rationales now name the concrete follow-up required (lower-level rendered harness or explicit coverage-policy decision), so this slice does not have to add that harness now.

### [info] Catalog representative_fixture vs. baseline_fixture_id
- `code_catalog.json` still records the existing `crates/sifr/tests/e2e/fail/defaultdict_keyword_constructor_unsupported.sifr` as STDLIB-0001's `representative_fixture`. That field is separate from the verification-area baseline coverage tracked in `code_baseline_coverage.json` and is not expected to change as part of this slice. Surfaced only because it could look like a divergence at first glance.

## Validation status

Focused validation already run by the implementer (all passed):

- Direct compact CLI check on `e2e_stdlib_defaultdict_keyword_constructor`: one SIFR-STDLIB-0001, exit 1, empty stdout.
- `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless` — 121 cases / 149 renderer variants.
- `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts` — 5 variants, 0 failures.
- `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` — 149 variants, 0 failures.
- `cargo test -p sifr_lowering defaultdict_keyword_constructor_unsupported_has_stdlib_code` — passed.
- `python3 scripts/check_file_size_guardrails.py` — passed.
- `git diff --check` — passed.

## Decision

Proceed to broad validation (`scripts/run_all_tests.sh --profile create-pr` then `scripts/run_all_tests.sh`) and PR submission. No follow-up review round is required if those gates pass.
