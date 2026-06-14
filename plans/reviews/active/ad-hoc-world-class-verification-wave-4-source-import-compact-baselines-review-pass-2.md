# Wave 4 Source Import Compact Baselines — Review Pass 2

**Verdict: no blockers. No further review rounds required. Ready for PR after the standard `scripts/run_all_tests.sh --profile create-pr` (and full `scripts/run_all_tests.sh` if you want to refresh the merge-gate signature) — the same gate every prior Wave 4 slice ran post-review.**

Both pass-1 follow-ups were applied correctly and neither introduced any regression. The slice still does what it claims: SIFR-IMPORT-0004..0007 are cleared from deferral, each is bound to a unique compact baseline that literally renders its code, and the IMPORT family has zero remaining Wave 4 rendered-baseline deferrals.

## Blockers

None.

## Pass-1 follow-up verification

**Follow-up #1 (add fixture-local `sifr.toml` to `source_import_private_member`) — applied correctly.**

- File contents: `[source]\nroots = ["."]\n` at `verification/areas/diagnostics/fixtures/diagnostics/source_import_private_member/sifr.toml:1-2`. Matches pass-1's recommendation verbatim.
- Live reproduction: `cargo run --locked -q -p sifr -- --diagnostic-format compact check verification/areas/diagnostics/fixtures/diagnostics/source_import_private_member/main.sifr` still emits exactly `E SIFR-IMPORT-0004 …main.sifr:1:24 cannot import private name '_secret' from module 'local_math'`, exit code 1. Diffing live stderr against the recorded baseline returns `MATCH` — the fix-up did not perturb the rendered output, which is the expected outcome (entry-parent resolution already won over workspace inheritance, so the new local manifest just removes the implicit coupling).
- `source_hash` invariant preserved: `sha256(main.sifr) = 39bd96c8…2eb10` still matches `baseline_metadata.json:1925`. Adding `sifr.toml` does not affect the hash because the validator only hashes `case["entry"]` (`verification/areas/diagnostics/checks/code_baseline_coverage.py:304`) — that's the underlying limitation noted in pass-1 finding #3, and it's the reason this cleanup was safe to apply without re-blessing.
- The fixture is now self-contained and structurally consistent with the other three project-style fixtures. Pass-1's "leaning toward applying before merge" lands cleanly.

**Follow-up #4 (sort the four new manifest cases) — applied correctly.**

The four new entries under the diagnostics `baselines` suite are now strictly alphabetical: `source_import_ambiguous_module` → `source_import_cycle` → `source_import_namespace_collision` → `source_import_private_member` (`verification/areas/diagnostics/manifest.json:941, 950, 959, 968`). Matches the convention used by the existing `e2e_*` block. No automated check enforces ordering for the diagnostics manifest, so the impact is purely readability/consistency — exactly as pass-1 characterized it.

`baseline_metadata.json` is keyed by `(fixture_id, renderer)`; the order of entries inside the array does not affect lookup, and the four new entries were not reordered to match. That's fine — there is no convention requiring metadata-array ordering, and the validator does not assert one. Leaving as-is is correct.

## Did the cleanup introduce any new issue?

No.

- Adding `sifr.toml` to `source_import_private_member` does not shift the diagnostic, the column, the rendered path, or the exit code — confirmed by byte-for-byte diff.
- The new manifest ordering is structurally equivalent to the prior listing; no fixture id, command, exit code, or renderer-format set changed. The area adapter discovers cases by `id`, so traversal order has no semantic impact.
- The pre-existing `validate_coverage_baseline_evidence` check (`code_baseline_coverage.py:226-242`) still fires for every new row and still passes (re-grep confirms each compact stderr literally contains its claimed code; the user reports both contracts and baselines suites passed again post-cleanup, and a fresh `cargo run` over all four fixtures reproduces the recorded stderr exactly).
- No new files were introduced beyond the one-line `sifr.toml`; `git status` shows the same four fixture trees plus the modified `manifest.json`, `baseline_metadata.json`, `code_baseline_coverage.json`, and phase doc — nothing extraneous.

## Non-blocking findings carried over from pass 1

These remain open by design; none should block PR. Listing them only so the PR description can capture the deferral explicitly:

- **#2 — three fixtures are byte-identical copies of `project_workspace` canonicals.** Silent-drift risk; pass-1 mitigation options (`source_origin` field with byte-equality check, or symlink-style manifest reference) remain valid for a broader hygiene pass. Phase doc explicitly defers this as "non-blocking future hygiene rather than new scope in this slice."
- **#3 — `source_hash` only pins `main.sifr`.** Auxiliary `.sifr` modules and `sifr.toml` files (including the one just added) are not hash-tracked. This is a generic limitation, not a slice-specific regression; replay still catches drift via stderr diff. The cleanest fix is a `source_inputs_hash` (or hashing every `*.sifr`/`sifr.toml` under the fixture directory) across all diagnostics fixtures, not just this slice.
- **#5 — `bless_reference: "wave-4-source-import-compact-baselines-pr"` is a placeholder slug.** Per the slice-3 accepted convention; swap to the real PR URL after open.
- **#6 — `compiler/frontend` vs. `compiler/core-language` owner mismatch with parallel slices.** Both owners exist in `verification/owners.json`; no regression introduced here. Bundle into a future ownership sweep.

## Answers to the pass-2 questions

**Do SIFR-IMPORT-0004..0007 still map to compact baselines that render their codes?**
Yes. Live `cargo run … check` over each fixture's `main.sifr` emits exactly one diagnostic with the claimed code, the recorded path, column, and message, and exit code 1. Stderr diffs against the recorded baselines are byte-identical. The `validate_coverage_baseline_evidence` check re-confirms this at suite time and the user reports it passing after the cleanup.

**Are the fixture-local project files sufficient?**
Yes. Each fixture is now structurally self-sufficient:
- `source_import_private_member`: `main.sifr` + `local_math.sifr` + the newly added `sifr.toml` (roots = `.`). Entry-parent resolution finds `local_math.sifr` before consulting workspace roots, and the local manifest removes the hidden coupling to the repo-root `sifr.toml` that pass-1 flagged.
- `source_import_ambiguous_module`: `main.sifr` + `sifr.toml` (roots = `src_a`, `src_b`, `.`) + `src_a/helper.sifr` + `src_b/helper.sifr`. Both `helper.sifr` candidates are reachable, so SIFR-IMPORT-0005 fires deterministically.
- `source_import_namespace_collision`: `main.sifr` + `sifr.toml` (roots = `src`, `.`) + `src/helper.sifr` and `src/helper/value.sifr`. The flat module collides with the namespace package, firing SIFR-IMPORT-0006.
- `source_import_cycle`: `main.sifr` + `a.sifr` + `b.sifr` + `sifr.toml` (roots = `.`). `a` imports `b` and `b` imports `a`, firing SIFR-IMPORT-0007 at `a.sifr:1:6` — note that the rendered path is the cycle source, not `main.sifr`, which is the diagnostic's intended behavior.

No fixture depends on workspace inheritance from the repo root after the pass-1 cleanup.

**Does phase tracking remain honest?**
Yes. Reconciled the counts directly against `code_baseline_coverage.json`: 170 total = 101 active + 69 deferred. IMPORT family: 9 codes, 0 deferred. The phase-doc claim that "the `IMPORT` family no longer has Wave 4 rendered-baseline deferrals" is accurate. The slice paragraph at `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:869-874` correctly enumerates the four codes closed, the four fixture ids, both rounds of focused validation, the pass-1 outcome, the pass-1 cleanup that was applied, and the explicit deferral of non-blocking findings #2/#3/#6. The status header line at `:3` says "Wave 4 source import compact baseline slice locally focused-validated and ready for review," which doesn't overstate — full `scripts/run_all_tests.sh --profile create-pr` post-cleanup hasn't been recorded yet, matching the slice-3 pattern where the merge-gate run is added after pass 2.

## Required follow-up before PR

The pass-1 follow-ups have been applied and re-validated against the focused diagnostics suites. The remaining gating step is the standard merge-gate validation that every prior Wave 4 slice ran post-review:

- **Run `scripts/run_all_tests.sh --profile create-pr`** and record `e2e <N>/<N>`, signature, hardening counters, and any warm-budget advisory in the phase doc and PR description. Required by `AGENTS.md`.
- **Run `scripts/run_all_tests.sh`** (full merge) and record diagnostics baselines `variants=<N>` (should be 127 with this slice), hardening variants, and the merge-gate signature. Slice 3 closed at warm wall-time `1018.43s`; this slice adds four small project-style fixtures and is not expected to move the budget materially.
- **No additional review round is required.** The two pass-1 follow-ups are the only outstanding cleanup items; both are applied and re-validated. The non-blocking findings carried forward (#2/#3/#5/#6) are explicitly deferred by the phase doc and do not need a third review pass — they are scope for a separate hygiene slice.

Open the PR after the two `run_all_tests.sh` runs above land cleanly.
