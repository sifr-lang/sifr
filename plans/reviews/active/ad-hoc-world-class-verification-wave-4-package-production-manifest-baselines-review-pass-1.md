I've reviewed the diff, fixtures, runner changes, and metadata. Now I'll write the review.

## Review summary

**Verdict: no blockers, ready for PR after `bless_reference` placeholders are swapped for the PR URL.**

### Correctness

- **Diagnostic emission sites** — `crates/sifr_package/src/diag/package.rs:10-29` (`manifest_exports_not_production`) and `:33-53` (`manifest_bins_not_production`) emit messages that match the baseline `stderr` byte-for-byte (`"production sifr.toml at '{}' uses [exports].modules"` / `"... uses [[bin]] target tables"`). The `<unknown>` token reflects that `SifrManifest` diagnostics lack a source span — correct for manifest-shape errors.
- **Adapter dispatch** (`area_adapter.py:463-484`) — `package-check-default` correctly joins the `find_package_root`/workspace-anchored `cargo run --manifest-path` form already used by `package-check` and `package-run-script`, then issues `check` with no positional arg. Since the cwd is the package root, `sifr check` discovers `sifr.toml` via its default lookup, exactly as intended for the "production manifest is the entry/source-hash evidence" semantic.
- **`BASELINE_COMMANDS` extension** (`area_adapter.py:24-33`) — the new command is allow-listed and the dispatch fall-through (`else: argv.extend([command_name, str(entry)])` at `:489`) cannot fire for it because the explicit `elif` branch matches first.

### Fixture fidelity

- Both `sifr.toml` source hashes match `baseline_metadata.json` (`b24dac3c…7998d3` for exports, `abf8bd06…47da39` for bin tables) — confirmed with `shasum -a 256`.
- Fixture layout follows the `package_script_recursion` pattern from the immediately prior slice: `Cargo.toml` with empty `[workspace]`, `[package.metadata.sifr] manifest = "sifr.toml"`, `src/lib.rs` marker, `src/main.sifr` stub, and `baselines/` at the package root (not under `src/`) — consistent with `entry.parent / "baselines"`.
- Compact baseline trio is complete for both fixtures (stdout empty, stderr ending with newline, exit-code `2`). Exit code `2` differs from the existing `package-check` fixtures (`1`); the manifest production-validation path returns 2, matching the focused validation. This is observed behavior, not a manifest-config defect.

### Manifest / coverage / metadata consistency

- Coverage flip for both codes is well-formed: `baseline_fixture_id` set, `deferral` nulled, `renderer_formats: ["compact"]`. The catalog declares `renderer_support` includes `compact` for both codes (`code_catalog.json:1929-1933`, `:2009-2013`), so the schema validators in `code_baseline_coverage.py:204-218` will accept it.
- Coverage counts verified by Python: total=170, covered=129, deferred=41, with families `BUILD 5, INTERNAL 1, PACKAGE 29, STDLIB 2, WORKSPACE 4` — matches the plan tracker text exactly.
- Manifest cases are placed adjacent to the existing `package_*` cluster; metadata entries appear between `package_explicit_file_outside_source_root` and `package_script_recursion`. Within the new pair, the file orders `exports` before `bin_tables`, which is reverse-alphabetical but consistent with the manifest's addition order — the validators don't enforce ordering, and the file's broader convention isn't strictly alphabetical either, so this is purely cosmetic.

### Validation adequacy

Direct compact CLI reproduces each diagnostic with exit 2 and empty stdout; `--bless` then verify round-trip is green (155/155); contracts pass (5/0); `py_compile`, file-size guardrail, and `git diff --check` all pass. Broad merge-gate validation is pending per the plan-doc "Status" line, as expected.

### Notes / minor

- **`bless_reference` placeholder** — `wave-4-package-production-manifest-baselines-pr` on both new metadata entries is intentional and matches the Wave 14 pattern. Schema validator only requires truthiness; swap for the PR URL after creating the PR.
- **Empty review file at `plans/reviews/active/ad-hoc-world-class-verification-wave-4-package-production-manifest-baselines-review-pass-1.md`** — this is the destination for this review; populate it with this summary before submitting.
- **Stylistic ordering nit** — within the new metadata pair, `exports` precedes `bin_tables` in the file; strict alpha would have `bin_tables` first. Non-blocking; consistent with the immediately preceding manifest insertion order.

**No blocking issues. No additional review round needed before PR submission after the broad gates (`run_all_tests.sh --profile create-pr` and `run_all_tests.sh`) pass.** Remaining open items are mechanical: replace the `bless_reference` placeholder once the PR is opened, and write the review summary into the empty review-pass-1 file if you want it tracked.
