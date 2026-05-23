

## M5 Review — Ready

### Blocking Issues

**None.** M5 passes the bar.

### Non-Blocking Notes & Observations

**1. `dependency_import_root` silently discards non-table dependencies**

`crates/sifr_package/src/graph/scopes.rs:219` — `SifrDependency::Version(_)` silently produces `None`. If someone writes a bare version string in `[dependencies]` with an `import` key, it will be silently ignored. The manifest TOML would produce a syntax error if the user writes `foo = "1.0.0" { import = "bar" }` (inline table on a string literal), but if the table parsing code ever emits `Version` for a dependency that also has `import` metadata, it would be silently dropped. Low probability in practice; worth a note.

**2. `dependency_kind` is a raw Cargo string (e.g. `"dev"`, `"build"`)**

`crates/sifr_package/src/graph/derive.rs:245-259` — `dependency_kind_for_edge` extracts `.kind` directly from Cargo metadata without normalization. `is_none()` works as a proxy for "normal" because Cargo only sets `kind` for non-normal dependencies. This is correct but not obvious from reading the code — a comment would help future readers understand that `None` = normal, `Some(...)` = dev/build.

**3. FNV-1a hash for namespace isolation**

`crates/sifr_driver/src/project/discovery.rs:608-615` — Uses 64-bit FNV-1a. The 16-digit hex gives ~64 bits of entropy, which is sufficient for package-instance namespace isolation (collision probability ~1/2^64 per pair). No action needed, but if this namespace is ever used in security-sensitive contexts, a stronger hash would be worth considering.

**4. `rewrite_import_from_dependency` only handles `from X import Y` at statement level**

`crates/sifr_driver/src/project/discovery.rs:670-710` — The rewrite is shallow (only `Stmt::ImportFrom` at top level of `stmts`). Nested imports inside a function body would not be rewritten. The scope derivation and graph walking only visit top-level imports, so this is correctly scoped for the current system. If the language ever allows imports inside functions, this would need a recursive visitor.

**5. No test for `--workspace` + `-p` + `--exclude` together end-to-end**

The CLI parsing test (`test_package_cli_parses_check_message_format_and_tree_args`) covers the argument parsing. The session test (`package_session_plans_check_workspace_package_selection`) tests the session layer with all three. The integration appears covered, but no test wires all the way from CLI → session → cargo plan with actual workspace metadata. Low risk since the layering is clean.

**6. `duplicate_workspace_sifr_names` duplicates diagnostic logic with `duplicate_workspace_import_roots`**

Both functions follow the identical pattern: group by key, filter where group size > 1, map to diagnostic. Could be refactored to a shared helper in a follow-up. Not blocking.

**7. Test `test_check_package_project_uses_sifr_manifest_dependency_aliases` doesn't assert the specific alias resolution path**

The test asserts `errors.is_empty()` but doesn't verify that `demo_json_v1` is resolved via the sifr.toml `[dependencies]` route vs. a Cargo metadata alias. In this specific test, there's no Cargo metadata alias (only the manifest alias is added), so the test correctly exercises the manifest-only path. A future Cargo metadata alias in the same package could make this ambiguous. Minor.

### Verification Summary

| Component | Status |
|---|---|
| SIFR-PACKAGE-0607 (duplicate Sifr name) | Correct — called in both `select_sifr_workspace_members` and `explicit_package_selection`. Test + docs complete. |
| CLI forwarding (`--workspace`, `-p`, `--exclude`) | Correct — `CargoPackageSelection` exported, `plan_check` updated, args built correctly. |
| `dependency_kind.is_none()` non-normal filter | Correct — excludes dev/build-only deps from scope derivation. |
| Sifr manifest `[dependencies].import` alias derivation | Correct — supplements Cargo metadata aliases, deduplicates, sorts. |
| Sifr manifest dependency alias test | Correct — covers the full resolution path from sifr.toml → scope → compile name. |
| Namespace hashing (FNV-1a) for transitive deps | Correct — consistent hash per package ID, sufficient entropy. |
| Import AST rewrite for scoped compile names | Correct — rewrites `from X import Y` statements to use scoped compile module names. |
| `dependency_kind_for_edge` lookup | Correct — matches both `dependency_name` and `package` fields. |
