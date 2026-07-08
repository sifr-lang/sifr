Now I have enough context to render the verdict. Let me consolidate.

## M0e Review — Source-Origin Privacy for `_sifr.*` Imports

**Pass-1 blocker verified fixed:** `internal_docs/diagnostic_emission_inventory.md:297` now reads `forbidden private sysroot declaration import` and points at `crates/sifr/tests/e2e/fail/stdlib_intrinsic_direct_import.sifr`, matching the registry entry, the runtime message template, and every other diagnostic index.

### Correctness & acceptance-criteria checks

- **Source-origin policy is implemented, not an import-prefix rule.** `LoweringSourceOrigin` (`crates/sifr_lowering/src/lower/mod_context.rs:338-357`) has three variants (`User`, `SysrootPublicStdlib`, `SysrootPrivateDeclaration`) with the correct asymmetry: `is_sysroot_source()` covers both sysroot origins for stdlib-lowering behavior (`is_stdlib_lowering()`), while `can_import_private_stdlib_declarations()` narrows to `SysrootPublicStdlib` only.
- **Import gate is origin-driven.** `crates/sifr_lowering/src/lower/mod_impl.rs:266-276` now checks `!ctx.can_import_private_stdlib_declarations()` before resolving `_sifr.*` imports — user and private-declaration sources both hit `forbidden_intrinsic`. This is a real tightening for `SysrootPrivateDeclaration` (previously `allow_intrinsic_imports = true` for both entry points); grep over `stdlib/_sifr/**` finds zero `_sifr.*` imports, so no shipped source regresses.
- **Driver dispatch matches loader classification.** `lower_stdlib_source` (`crates/sifr_driver/src/stdlib/bootstrap.rs:497-510`) routes on `LoadedStdlibSourceKind::{Public,PrivateDeclaration}`, and the loader (`crates/sifr_stdlib_manifest/src/sources.rs:353,376`) sets those kinds from the on-disk `stdlib/sifr` vs `stdlib/_sifr` layout.
- **`return_lowering.rs:69` semantic parity.** `!ctx.is_stdlib_lowering()` is equivalent to the old `!ctx.allow_intrinsic_imports` because both prior stdlib entry points set that flag true; both new sysroot origins satisfy `is_sysroot_source()`.
- **Diagnostic surface is consistent everywhere.** Registry short desc + template (`parsing_names_and_types.rs:161-171`) match the runtime message in `import_diagnostics.rs:11-13`, the baseline `.../check-compact.stderr.txt`, docs (`docs/errors/SIFR-IMPORT-0001.{md,mdx}`, `docs/diagnostics/error-codes.mdx`, `docs/errors/diagnostic-codes.md`), and `internal_docs/{diagnostic_codes.md,diagnostic_emission_inventory.md}`.
- **Test coverage is positive + negative.** `name_import_diagnostics_tests.rs:117-153` adds `public_sysroot_stdlib_source_can_import_private_declarations` and `private_sysroot_declaration_source_cannot_import_private_declarations` plus updates the existing user-rejection test.
- **Doc updates land the rule.** `internal_docs/architecture.md:158-165` adds the source-origin explanation; `internal_docs/sifr_sysroot_and_stdlib_architecture.md:682-690` makes explicit that private declarations retain stdlib-lowering behavior but lose private-import capability.
- **No fallback path introduced.** No prefix-based shortcut, no dual policy — the gate reads a single boolean derived from a single origin field.

### Minor, non-blocking observations

- The constant `DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC` (`crates/sifr_diagnostics/src/codes/registry.rs:28`) keeps the old "intrinsic" name while the user-facing message now says "private sysroot declaration." Renaming the symbol would be churn; the diagnostic code (`SIFR-IMPORT-0001`) is the stable identifier. Note only.
- `user_interop` (`crates/sifr_driver/src/build/sysroot_interop_tests.rs:228-238`) still calls `lower_module_sysroot_private_declaration_with_externals` on a `/ws/app/...` source to synthesize an interop plan for downstream trust-policy tests. This predates M0e and is confined to unit tests; the rename in this PR is a mechanical follow-through. Out of scope.
- `lower_module_sysroot_public_stdlib` (no-externals) has no callers — kept for API symmetry with the previous `lower_module_stdlib` no-externals variant. Also pre-existing shape.

### Verdict

**READY** — pass-1's only blocker is resolved, all M0e acceptance criteria are satisfied (source-origin dispatch, private-declaration tightening, doc + diagnostic + baseline consistency), no fallback paths were introduced, and the shipped `create-pr` validation matches the code state on disk.
