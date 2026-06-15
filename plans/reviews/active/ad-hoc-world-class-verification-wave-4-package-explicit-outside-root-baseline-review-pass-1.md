## Findings — Wave 4 package explicit-file-outside-source-root compact baseline slice

**Blockers:** none.
**Major:** none.

### Minor

1. **Fixture `Cargo.toml` is missing the `[workspace]` empty section that every sibling package fixture has.** `verification/areas/diagnostics/fixtures/diagnostics/package_explicit_file_outside_source_root/Cargo.toml` declares the package metadata but stops at `[package.metadata.sifr]`. Every other package-check fixture (`package_duplicate_public_api_symbol/Cargo.toml:9`, and every package-management area sibling — `package_ambiguous_import_canonical`, `package_diagnostic_help_preserved`, `package_bare_stdlib_import_canonical`, …) terminates with an empty `[workspace]` table. The pattern declares the fixture as its own workspace root so cargo cannot walk upward and discover the outer compiler workspace. In our exact code path it does not break anything — `area_adapter.run_sifr_variant` invokes `cargo run --manifest-path REPO_ROOT/Cargo.toml --locked -q -p sifr` so cargo resolves entirely from the explicit manifest and ignores the fixture's `Cargo.toml` (the focused validation already confirmed the diagnostic emits cleanly). Recommend adding the trailing `[workspace]` for consistency with siblings and so future tooling that walks cwd upward doesn't get surprised.

2. **`Cargo.toml` `edition = "2024"` differs from every sibling fixture (`edition = "2021"`).** Same code path as point 1 — the fixture's `Cargo.toml` is inert under `--manifest-path`, so the value cannot affect any check. Still purely cosmetic, but worth aligning for grep-ability.

3. **`bless_reference` is a placeholder, not a PR URL.** `baseline_metadata.json:923` uses `"bless_reference": "wave-4-package-explicit-outside-root-baseline-pr"`. This follows the precedent from the eleventh and twelfth slices where the reviewer marked the same swap-in as an optional follow-up after the PR is opened. Same treatment here.

### Info (process, not code)

4. **Broad-gate evidence is intentionally deferred.** Focused validation covers direct CLI emission (1× `SIFR-PACKAGE-0710`, exit 1, empty stdout), `baselines --bless` + `baselines` re-verify (124 cases / 152 renderer variants), `contracts` (5/0), `py_compile`, file-size guardrail, and `git diff --check`. This is sufficient to clear code review; `scripts/run_all_tests.sh --profile create-pr` and the full merge gate still need to run before PR/merge, matching the slice text's pending list.

### Verified correct

- **`package-check` command path** (`verification/runner/sifr_verify/area_adapter.py:453-498, 596-602`):
  - `find_package_root` walks `entry.parent` upward seeking both `Cargo.toml` *and* `sifr.toml`. From `tools/task.sifr` it skips `tools/` (no manifests) and resolves on the next iteration at the fixture root, which has both files. No ambiguity.
  - The resulting invocation `cargo run --manifest-path REPO_ROOT/Cargo.toml --locked -q -p sifr -- --diagnostic-format compact check tools/task.sifr` runs from the fixture-root cwd, so the user-visible `sifr check tools/task.sifr` is reproduced faithfully and the workspace `target/` cache is reused.

- **Nested `tools/baselines/` attribution** (`verification/areas/diagnostics/checks/code_baseline_coverage.py:87-121`):
  - `expected_baseline_files()` derives the baseline dir from `entry.parent / "baselines"` → `tools/baselines/`. Matches the on-disk layout, so the trio existence check and the `actual_files - allowed_files` orphan check both pass.
  - `actual_baseline_files()` globs `**/baselines/*.txt` under the fixture root, so the nested `tools/baselines/` is discovered identically to the existing `src/baselines/` precedent established by `package_duplicate_public_api_symbol`.
  - `baseline_file_keys` uses `path.relative_to(fixture_root).parts[0]` for the fixture id, which yields `package_explicit_file_outside_source_root` regardless of depth — so nested baselines correctly attribute to the top-level fixture id.
  - `validate_baseline_metadata` reads `source_path = case["entry"]` (`tools/task.sifr`), so the SHA is taken against the entry file used for the cargo invocation — see source-hash check below.

- **Diagnostic emission and scope discipline:**
  - Baseline stderr emits exactly one diagnostic — `E SIFR-PACKAGE-0710 <unknown> explicit file 'tools/task.sifr' is outside package source root '<WORKSPACE>/.../src'` — so `validate_coverage_baseline_evidence` (which substring-matches the code in the stderr file) passes.
  - `code_baseline_coverage.json` flips only the `SIFR-PACKAGE-0710` row from deferred to `compact`; no other PACKAGE codes change state — no over-claim.
  - `expect_exit_code: 1` in the manifest matches the `1\n` exit-code baseline.
  - Stdout baseline is exactly zero bytes (matches `normalize_string("")` output for empty stdout); stderr ends with a trailing newline (matches `normalize_string`'s `if not endswith "\n": += "\n"` guarantee).

- **Counts and hash:**
  - Computed from `code_baseline_coverage.json`: total active 170, covered 126, deferred 44, with PACKAGE 32, STDLIB 2, INTERNAL 1, BUILD 5, WORKSPACE 4 — matches the tracker note exactly (170 / 126 / 44; PACKAGE-down-by-one, others unchanged from the merged twelfth slice).
  - `shasum -a 256 tools/task.sifr → cde0429ba5478089419b8a36797f74089554914623c732054e04dd6eb2e49afd` matches `baseline_metadata.json:933` exactly.
  - Catalog `SIFR-PACKAGE-0710` has `renderer_support: [human, json, compact]` (`code_catalog.json:1988-2003`); coverage uses `compact`; manifest case lists `compact`; renderer_formats are a subset of catalog support — no validation gap.

- **Cross-file fixture-id consistency:** `package_explicit_file_outside_source_root` appears identically in `manifest.json:473`, `baseline_metadata.json:924` (`fixture_id`), `code_baseline_coverage.json:1364` (`baseline_fixture_id`), and `sifr.toml:2` (`name`). The new `Cargo.toml:2` declares a *fresh* `name = "package-explicit-file-outside-source-root"` rather than inheriting from a copy source — improvement over the twelfth slice, where the reviewer flagged inherited internal names.

- **Hygiene scope:** `baseline_hygiene.py` only blocks legacy `[Edddd]` pseudo-codes and the `SIFR-TYPE-0001` catch-all. Neither appears in the new fixture, and `git ls-files verification/**` will pick the new files up once staged so the contract suite continues to apply.

- **Source-root validity:** `sifr.toml` declares `[source] root = "src"` with `src/main.sifr` and `src/lib.rs` present — sifr can resolve the source root, so SIFR-PACKAGE-0710 (rather than a missing-source-root diagnostic) is the chosen error. The marker `src/lib.rs` and stub `src/main.sifr` follow the same precedent as the twelfth slice.

### Verdict

No blockers and no must-fix issues. The three minor items are cosmetic and follow precedent from prior slices that shipped with the same notes. **Another review round is not required** — remaining gates are mechanical (`scripts/run_all_tests.sh --profile create-pr`, merge gate, optional `bless_reference` swap, optional `[workspace]`/`edition` alignment).
