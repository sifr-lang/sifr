I've reviewed all relevant files and reproduced the fixture invocation locally. Here is the review.

## Review summary

**Verdict: no blockers, ready for PR after `bless_reference` is replaced.**

### Correctness

- **Recursion trigger** — `sifr.toml`'s `dev = { command = "run", args = ["other"] }` with a sibling `other` script hits the `script.command == "run" && lookup_script(arg0).is_some()` branch in `crates/sifr_package/src/ops/session.rs:294-300`, producing `SIFR-PACKAGE-0714`. Confirmed by direct invocation: emits exactly one diagnostic with `<unknown>` origin (correct — the diagnostic uses `PackageDiagnosticOrigin::CargoMetadata { cargo_package_id: None }`).
- **Adapter dispatch** (`area_adapter.py:462-482`) — `package-run-script` correctly reuses `find_package_root`, the workspace-anchored `cargo run --manifest-path` form, and dispatches `run --script dev`. The error-message rename on `area_adapter.py:605` to "package command entry" is appropriate now that two commands share the path.

### Fixture fidelity

- `sifr.toml` source hash `6965acc4…1b6d636` matches `baseline_metadata.json`. Confirmed via `shasum -a 256`.
- Baselines under the package root (`baselines/`, not `src/baselines/`) follow logically from `entry = sifr.toml` — `baseline_artifact_paths` computes `entry.parent / "baselines"`. Choosing `sifr.toml` as `entry` (rather than `src/main.sifr`) is the right call here: the diagnostic is manifest-defined, so re-blessing is correctly gated on manifest changes via the source hash.
- `Cargo.toml` aligns with the Wave 13 normalization (`edition = "2021"`, empty `[workspace]`, no lockfile). `src/lib.rs` marker file matches peers.
- Compact baseline contents: stderr ends with trailing `\n` (matches peer convention and adapter's `normalize_string`), stdout is empty, exit-code is `1`. All three trio files present.

### Manifest / coverage / metadata consistency

- Coverage counts verified by counting `coverage` array: 170 total, 127 covered, 43 deferred, with family breakdown `BUILD 5, INTERNAL 1, PACKAGE 31, STDLIB 2, WORKSPACE 4` — matches plan note and the prior slice's deltas (PACKAGE 32 → 31; covered 126 → 127).
- `SIFR-PACKAGE-0714` coverage entry flips from `baseline_fixture_id: null` + deferral → `package_script_recursion` + `["compact"]` + `deferral: null`. Schema validators in `code_baseline_coverage.py:192-218` will accept this.
- Metadata entry is alphabetically placed between `package_explicit_file_outside_source_root` and `self_update_missing_receipt` — consistent with the file's existing ordering convention.
- Diagnostic codes registry (`codes/registry/registry_entries/package.rs:414-424`) already declared 0714 active; docs page `docs/errors/SIFR-PACKAGE-0714.md` exists. No additional registry work is needed in this slice.

### Validation adequacy

The validation footprint is appropriate for the slice scope: direct compact CLI reproduces the diagnostic and exit code; baselines `--bless` then `verify` round-trip (125/153); contracts pass (5/0); py_compile + file-size guardrail + `git diff --check` all green. The broad merge-gate validation is pending per the plan-doc "Status" line, as expected.

### Notes / minor

- **`bless_reference` placeholder** — `wave-4-package-script-recursion-baseline-pr` is flagged as intentional. Schema validator only requires truthiness, so contracts pass; replace with the PR URL after opening (matches Wave 13's resolution).
- **Hard-coded script name `dev`** — `area_adapter.py:482` literally passes `["run", "--script", "dev"]`, so every future `package-run-script` fixture must define a `dev` script. Acceptable as a single-purpose path for now; revisit if a second fixture lands.
- **Pre-existing, not for this PR**: the diagnostic format template in the registry (`"…is not allowed: {script}"`) differs from the actual emitted message (`"…is not allowed for '{script}'"`). Out of scope here; contracts already pass.

Recommendation: proceed to create-pr validation and PR opening; swap the `bless_reference` placeholder for the PR URL after creation.
