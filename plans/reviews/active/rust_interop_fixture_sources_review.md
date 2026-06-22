## Code Review: Rust Interop Fixture Source Completion

**No blocking findings.** The diff is shippable; one latent validator bug and a handful of contract-tightening opportunities below.

### Findings (by severity)

#### Low — defensive coding bug in validator

**1. `_validate_fixture_evidence_file` crashes (AttributeError) when `expected_result` is missing instead of reporting cleanly** — `verification/areas/rust_interop/checks/check_fixture_matrix.py:339-346`

```python
expected_result = manifest_evidence.get("expected_result")
if not isinstance(expected_result, str) or not expected_result:
    failures.append(f"{fixture_id}: evidence.{side}.expected_result is required")
status = matrix_evidence.get("status")
if expected_result.startswith("future-owned") and status == "passing":   # crashes if None / non-str
    ...
if status != "passing" and not expected_result.startswith("future-owned"):  # same
    ...
```

All current fixtures populate the field, so it's dormant — but the check is supposed to be defensive. Either `return` after the failure append, or guard the two `.startswith` calls with `isinstance(expected_result, str)`. Fix is one or two lines.

#### Info — validator/README contract gaps (not blockers)

**2. Validator doesn't enforce the `positive/<id>.sifr` / `negative/<id>.sifr` directory layout described in the README** — `check_fixture_matrix.py:305-318`. The validator accepts any in-tree `.sifr` path the manifest points to. A manifest with `evidence.positive.path: "stub.sifr"` at the top level (or worse, `negative/foo.sifr` declared as the positive evidence) would pass. Worth pinning to `Path(side) / f"{matrix_evidence['id']}.sifr"` so the layout match is enforced, not just documented.

**3. Headers in `.sifr` files aren't cross-checked against the manifest's `expected_result` / `execution_kind`** — `check_fixture_matrix.py:328-337`. The required headers are `fixture`/`evidence`/`evidence-status` and (for diagnostic results) `expected-diagnostic`. `# expected-result:` and `# execution-kind:` are present in every file but not validated against the manifest — they can drift silently. Natural extension of the existing header gate.

**4. The "non-empty stub" check is lexical and easy to satisfy** — `check_fixture_matrix.py:325-337`. A file with five comment lines, one of which contains the string `@rust`, `fixture-cargo:`, or `fixture-trust:`, passes. In practice all current fixtures have real decorator declarations, but the check should not be mistaken for "a real fixture exists" — it's "the words appear." Acceptable for contract tier; noting the limitation.

#### Nits

**5. No rationale comment alongside `[lib] doctest = false`** — `crates/sifr_lsp/Cargo.toml:24-25`. The change is consistent with existing precedent — `crates/sifr_analysis/Cargo.toml` and `crates/sifr_lint/Cargo.toml` both already set `doctest = false`, so this isn't unjustified. The only doc-style content in `crates/sifr_lsp/src/` is the module-level `//!` prose in `lib.rs` (no ` ``` ` code fences). A one-liner `# rustdoc tests hang in CI; no doc examples exist in this crate` would protect against a future revert. Not blocking; precedent crates don't have one either.

**6. README documents the four required files but not the `fixture.json` schema** — `verification/areas/rust_interop/README.md`. A fixture author has to read the validator to learn the required keys (`schema_version`, `diagnostic_family`, `evidence.{side}.{id,status,path,expected_result,expected_diagnostic?}`). Minor docs gap.

### What's verified clean

- 31 fixture directories ↔ 31 matrix entries, 1:1; counts match the validation summary (31 `fixture.json`, 31 positive `.sifr`, 31 negative `.sifr`).
- Every `fixture.json` I sampled mirrors the matrix `id`/`capability`/`tier`/`execution_kind`/`required_crates`/`features` and adds `schema_version`/`diagnostic_family` against the reserved SIFR-RUST-* set; validator confirms this for the full set.
- Headers in sampled `.sifr` files (across all tiers and both execution kinds) match the validator's required form, and `# fixture-cargo:` / `# fixture-trust:` annotations are used correctly where `@rust(...)` isn't the policy surface (`cargo_locked_offline`, `proc_macro_trust`, `native_build_script`).
- `status: planned` evidence consistently uses `expected_result: future-owned[-diagnostic]`; `status: passing` evidence consistently uses `pass` or `diagnostic`. The validator's pairing rule is satisfied across all 62 evidence entries.
- `crates/sifr_lsp/src/` contains no rustdoc code fences, so `doctest = false` doesn't lose coverage.

### Recommendation

Land as-is. Optionally fix finding #1 in this PR (5-line change) since the validator is a CI gate and crashing on malformed input is worse than reporting it. Findings #2–#4 can be tightened in a follow-up — they harden the contract but don't change current behavior.
