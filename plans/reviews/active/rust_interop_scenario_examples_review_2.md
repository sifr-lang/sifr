Findings ordered by severity.

## HIGH

**1. All five new `sifr.toml` files are missing the required `package.sifr-version` field** — they would fail to parse as Sifr packages.

The Sifr manifest parser at `crates/sifr_package/src/manifest/sifr.rs:112-119` calls `required_string(..., "package.sifr-version")` and aborts with `PackageDiagnostic::invalid_sifr_manifest` if absent; `validate_compiler_requirement` (`crates/sifr_package/src/manifest/sifr_fields.rs:372`) then requires the value to contain `"0.3"` or equal `"*"`. Every canonical sifr.toml in the repo — root `/sifr.toml:5`, `verification/areas/package_management/corpora/demo_repositories/*/sifr.toml`, etc. — declares `sifr-version = ">=0.3,<0.4"`. The README contract (`verification/areas/rust_interop/README.md:42`) advertises these scenarios as "Sifr package config" examples, but as-is none of them parse:

- `verification/areas/rust_interop/fixtures/same_workspace_crate/examples/workspace_hash_crate/sifr.toml:4-7`
- `verification/areas/rust_interop/fixtures/shared_bridge_crate/examples/shared_hash_bridge/sifr.toml:4-7`
- `verification/areas/rust_interop/fixtures/cargo_locked_offline/examples/locked_offline_cache/sifr.toml:4-7`
- `verification/areas/rust_interop/fixtures/bridge_version_mismatch/examples/bridge_version_package/sifr.toml:4-7`
- `verification/areas/rust_interop/fixtures/panic_abort_profile/examples/abort_profile_package/sifr.toml:4-7`

The fixture-matrix checker doesn't catch it because `tomllib` doesn't enforce the Sifr schema and `_validate_scenario_manifests` (`check_fixture_matrix.py:479-521`) only asserts the `[rust]` and `[trust]` blocks. Two changes needed:

1. Add `sifr-version = ">=0.3,<0.4"` to each scenario `sifr.toml`.
2. Extend `_validate_scenario_manifests` to assert presence of `package.name`, `package.edition`, and `package.sifr-version`, and that `sifr-version` matches `validate_compiler_requirement` (contains `"0.3"` or equals `"*"`). Otherwise this drifts back the next time someone copies the pattern.

## MEDIUM

**2. Stray `version = "0.1.0"` in every scenario `sifr.toml` `[package]` block.** `parse` (`crates/sifr_package/src/manifest/sifr.rs:106-123`) reads only `name`, `edition`, `sifr-version`, and `default-run` from `[package]`; unknown keys are silently ignored (Sifr has no top-level `version` for packages — `sifr-version` is the compiler-window key). Drop it from all five sifr.toml files to avoid implying parity with Cargo's `[package].version`.

**3. Scenario verifier name is unconstrained.** `_validate_scenario_sifr_source` (`check_fixture_matrix.py:465-477`) only checks for any `def verify_` / `async def verify_` prefix, whereas package examples must declare `verify_<crate>_package` (`check_fixture_matrix.py:647-650`). A scenario could pass with `def verify_foo()` unrelated to the scenario name. Tighten to `verify_<scenario>` matching the directory key — the current five files already follow that convention, so it's a no-cost regression guard.

## LOW

**4. `check_fixture_matrix.py` is now 888 lines** (`wc -l` confirms), 12 lines under the 900 hand-maintained file cap (`AGENTS.md` file-size guardrail; `scripts/check_file_size_guardrails.py`). Adding the `sifr-version` assertion above would push it past the cap. Plan a responsibility-based split before the next addition — e.g. move scenario validation (`_validate_scenario_examples`, `_validate_scenario_example_dir`, `_validate_scenario_manifests`, `_require_path_dependency`, `_require_member`, `_require_trust_targets`) into a sibling `_scenario_checks.py` and re-export.

**5. Hand-written `cargo_locked_offline/.../Cargo.lock` uses lockfile `version = 3`** while the auto-generated lockfiles for the other four scenarios are `version = 4` (Cargo wrote them during the `cargo check` smoke runs). That's consistent today, but the inline comment at the top of that file doesn't acknowledge version drift — a future toolchain bump could refuse or rewrite it. Either pin the toolchain in that scenario (`rust-toolchain.toml`) or extend the comment to say version 3 is intentional and confirmed to work under `--locked --offline --frozen`.

## INFORMATIONAL (out of PR scope)

**6. Scenario examples model only the positive layout.** No scenario directory walks through the diagnostic side — e.g. `bridge-version = 2` for `bridge_version_mismatch`, a lockfile-drift feature delta for `cargo_locked_offline`, or a `rust-panic-abort` omission for `panic_abort_profile`. Existing positive/negative `.sifr` evidence already covers the diagnostic side abstractly, so this is a documentation expansion, not a correctness gap. Worth tracking as a follow-up.

**7. Other rust-interop fixtures with package/workspace claims that still lack scenario examples:** `local_bridge_blake3` (capability "package-local bridge binding") relies on `[rust] bridges = [...]` in the package manifest plus the generated `bridge.*` namespace — that is a package-layout claim that the current PR's pattern would naturally extend to. Out of scope here, but flagging for the follow-up so the inventory stays aligned with the user's "full scenario for every package/workspace/config claim" goal.

---

Summary: not satisfied — finding #1 is a real correctness issue (the scenario `sifr.toml` files don't satisfy the parser they're supposed to demonstrate). #2 and #3 are cheap-to-fix consistency improvements. The rest are watch-items.
