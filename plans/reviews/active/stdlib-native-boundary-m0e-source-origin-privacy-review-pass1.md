## Findings

**1. `internal_docs/diagnostic_emission_inventory.md:297` — stale wording and fixture pointer** (low severity)
The SIFR-IMPORT-0001 row still reads `forbidden \`_sifr.*\` intrinsic import` and points at `crates/sifr/tests/e2e/fail/import_intrinsic.sifr`. Every other index (`internal_docs/diagnostic_codes.md`, `docs/errors/diagnostic-codes.md`, `docs/diagnostics/error-codes.mdx`, `docs/errors/SIFR-IMPORT-0001.{md,mdx}`) and the diagnostic registry now use the new wording and the registry's representative fixture is `stdlib_intrinsic_direct_import.sifr`. This inventory row is the only place the old wording survives and the description/fixture are out of sync with the registry entry. Failure scenario: an internal reader following the emission inventory to look up SIFR-IMPORT-0001 sees stale copy that contradicts both the diagnostic renderer and the newly-adopted trust-boundary language.

Everything else checked out:

- `LoweringSourceOrigin` has the three intended variants, `is_sysroot_source()` covers both `SysrootPublicStdlib` and `SysrootPrivateDeclaration`, and `can_import_private_stdlib_declarations()` narrows to `SysrootPublicStdlib` only — the intended asymmetry.
- `mod_impl.rs:272` now gates `_sifr.*` imports on `!can_import_private_stdlib_declarations()`; user + private-declaration sources both trigger `forbidden_intrinsic`. Since previously *both* sysroot variants set `allow_intrinsic_imports = true`, this is a real behavior tightening for `SysrootPrivateDeclaration` — exactly the M0e goal. No shipped stdlib private declaration currently imports `_sifr.*` (`grep` over `stdlib/_sifr/*.sifr` finds no such imports), so this tightening doesn't regress existing sources.
- `return_lowering.rs:69` switch from `!allow_intrinsic_imports` to `!is_stdlib_lowering()` is semantically equivalent: previously both stdlib entry points set `allow_intrinsic_imports = true`, and now both origins satisfy `is_sysroot_source()`. User paths remain gated.
- `sifr_driver::stdlib::bootstrap::lower_stdlib_source` correctly dispatches on `LoadedStdlibSourceKind::{Public,PrivateDeclaration}` to the matching lowering entry points.
- No leftover `lower_module_stdlib*` or `allow_intrinsic_imports` references anywhere in active `crates/` code; only archived plan/review docs still mention them (expected).
- Diagnostic registry (`SIFR-IMPORT-0001` message template + short description) matches the runtime `forbidden_intrinsic` string and the baseline `verification/areas/diagnostics/fixtures/diagnostics/e2e_import_intrinsic/baselines/check-compact.stderr.txt`.
- `name_import_diagnostics_tests.rs` adds both the positive (public sysroot can import `_sifr.*`) and negative (private declaration cannot) coverage, plus the existing user-rejection test with updated wording.
- `internal_docs/architecture.md` and `internal_docs/sifr_sysroot_and_stdlib_architecture.md` now state `_sifr.*` is a namespace convention and origin is the trust boundary — matches the runtime behavior.
- `LoweringSourceOrigin` is re-exported at `sifr_lowering::LoweringSourceOrigin` for public API completeness even though current external callers don't need it.

## Verdict

NEEDS_CHANGES — one low-severity cleanup blocker: update the SIFR-IMPORT-0001 row in `internal_docs/diagnostic_emission_inventory.md:297` to match the new wording ("forbidden private sysroot declaration import" or equivalent) and its representative fixture (`stdlib_intrinsic_direct_import.sifr`), so the internal diagnostic indexes are internally consistent. After that, and after `scripts/run_all_tests.sh --profile create-pr` passes as noted in the validation summary, this is ready for PR.
