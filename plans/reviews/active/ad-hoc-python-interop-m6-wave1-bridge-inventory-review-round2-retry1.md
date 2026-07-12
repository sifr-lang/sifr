Round-1 findings 1–13 are all addressed in the current diff:
- **#1** — projection.rs now generates `__sifr_inventory__.json` (repair_python_bridge_inventory wired at projection.rs:116-122, include entries in projection_bridge.rs:32-33 and projection.rs:293-294); tested end-to-end by `projection_repair_generates_canonical_python_bridge_inventory`. Convergence from missing manifest is real.
- **#2** — `DynamicImportVisitor::visit_expr` only fires on `Expr::Call`; references pass (verified by `dynamic_import_callable_reference_without_a_call_is_allowed`).
- **#3** — `ImportCollector` follows Assign/AnnAssign/Named plus tuple destructuring and canonical `getattr(importlib, 'import_module')` dispatch; 7-case parametric test locks it in.
- **#4** — escape message includes the offending import via `RawImport::display()`.
- **#5** — `ModuleNameError` splits reserved-root and invalid-path reasons.
- **#6** — the *relative-with-no-explicit-module* branch inserts base plus each name matched in `known_modules`.
- **#7** — `insert_same_package` inserts every ancestor prefix.
- **#8** — split `fs::read` + `String::from_utf8` produces "bridge source must be UTF-8 encoded".
- **#9** — `write_python_bridge_inventory` now returns `Result<Option<PathBuf>, PackageDiagnostic>`.
- **#10** — coverage added for root init, keyword identifier, duplicate modules, relative escape, symlink, non-UTF-8, mixed same-package imports, alias bypasses, non-call refs.
- **#11** — `pub use` grouped before `#[cfg(test)]` mods.
- **#12** — canonical-serialize failure becomes a real diagnostic.
- **#13** — `sha256` in modules, source digest, and inventory digest.

Byte-identical inventory across projection and packaging is preserved (both use `discover_python_bridge_inventory_at` and both produce content depending only on file bytes, module names, and imports — cargo_package_id/source_roots don't influence the JSON).

One remaining actionable issue: the round-1 finding #6 fix was applied only to the `from . import <names>` shape (module=None). The same "canonical `import`/`from ... import ...` inventory" contract still misses same-package submodule edges for both `from bridge.<pkg> import <names>` and `from .<subpkg> import <names>` (explicit modules), and the new ancestor test freezes this partial behavior by explicitly excluding `pkg.local`.

VERDICT: CHANGES_REQUESTED
