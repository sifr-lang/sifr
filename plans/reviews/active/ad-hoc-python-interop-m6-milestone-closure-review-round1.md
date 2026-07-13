Verified the plan is unchecked at lines 143 and 426, PYRES-0001 is reserved (not active), and the loader loop overwrites duplicates silently. Consolidating the findings from all five review streams.

## M6 Milestone Closure Review — Findings

### BLOCKER — closure gaps that prevent honestly declaring M6 done

1. **Plan checkboxes not flipped.** `plans/issues/active/ad-hoc-declaration-first-python-interop.md:143` still shows `- [ ] M6 hermetic package-local Python bridges` and line 426 still shows `- [ ] Complete bridge evidence and milestone closure` — the very wave this diff is supposed to close. Yet the uncommitted `plans/roadmap.md:129` proclaims "M0-M6 complete through hermetic package-local bridge deployment." The plan is the source of truth (per AGENTS.md workflow); leaving it unchecked while the roadmap claims completion is directly dishonest closure evidence.

2. **`plans/reviews/active/ad-hoc-python-interop-m6-milestone-closure-review-round1.md` is 0 bytes.** The M6 plan (line 436) explicitly mandates updating "review records"; M5 has a filled review; this milestone's review file is an empty placeholder in the working tree.

3. **`demos/m6_demo/run.sh:15` fabricates the milestone marker.** The final `printf 'sifr-python-interop:package-bridge:gtin=7032069804988:format=13:check=8'` runs unconditionally after `cargo test`. The archive test at `crates/sifr_driver/src/tests/package_python_bridge_archive_tests.rs:60-74` uses `Command::output()`, which captures the compiled binary's stdout into a variable and only asserts it — `--nocapture` doesn't restore it, so on success the child's real output is never printed. README (`demos/m6_demo/README.md:20-24`) implies "the command must finish with" that marker as if produced by the binary. The showcase demo is a shell echo, not proof.

### HIGH — hermeticity/correctness gaps in the shipped implementation

4. **`crates/sifr_runtime/src/python/bridge_loader.rs:22-33` — silent duplicate overwrite.** The install loop does `entries.set_item(&source.module, …)` in a `PyDict`; two `PythonBridgeSource` entries with the same `.module` string silently overwrite (last-writer-wins) with no `SIFR-PYIMP-0003`. `reject_reserved_collisions` (:192) only scans pre-existing `sys.modules`, not the input list. A malformed upstream planner (e.g. two packages happening to share `runtime_package` due to a resolver bug) would fail-silently, mis-executing the wrong bridge under a matching virtual `co_filename`.

5. **`crates/sifr_package/src/python/bridge_inventory/imports.rs` — dynamic-import detection gaps.**
   - Star imports: `from importlib import *; import_module("json")` — `record_imported_dynamic_aliases` only matches literal `"import_module"`/`"__import__"` aliases (`imports.rs:136`), not the star-imported symbol; `DynamicImportVisitor::new` (:202) does not seed `import_module`.
   - `importlib.__import__(...)` bypasses `dynamic_callable_name` (`imports.rs:217`) because the `__import__` branch only fires for prefixes in `builtins_aliases`.
   The contract at plan line 383 requires "reject … dynamic import calls as SIFR-PYIMP-0002"; these two patterns route around it.

6. **`crates/sifr_package/src/python/bridge_inventory/filesystem.rs:42` — symlink escape.** `symlink_metadata` only checks the leaf `src/python_bridges` node; a symlinked ancestor (`src/`) silently redirects discovery outside the package root, defeating "package-root src/python_bridges discovery" and rejection of "misplaced bridge roots." Existing coverage tests only the leaf.

### MEDIUM

7. **`crates/sifr_runtime/src/python/bridge_loader.rs:130-137` — retention only fires on `__import__`.** `guarded_import` re-hoists the finder to `sys.meta_path[0]` for calls routed through `builtins.__import__`, i.e. the `import` statement. `importlib.import_module` walks `sys.meta_path` directly via `_find_and_load` without going through `__import__`; a user-installed hostile finder at index 0 then services `__sifr_bridge__.p_*` for dynamic imports. Contract point "retain the reserved-name claim even after user `sys.meta_path` mutation" (plan line 412) is only partially satisfied.

8. **`verification/areas/python_interop/reports/python_interop_exit_evidence.md:38` — `SIFR-PYRES | 0001..0002` overclaims.** Only PYRES-0002 is active in `crates/sifr_diagnostics/src/codes/python_interop_codes.rs:23`; PYRES-0001 remains a reserved slot in `registry_entries/reserved.rs:42`. Listing it under "Active compiler diagnostic families" is misleading.

9. **`verification/areas/python_interop/fixtures/package_bridge_archive/package_bridge_evidence.json:19` — cleanup row overstates.** "bridge loader failed-install collision cleanup and reset_runtime_state_for_tests" — the loader test at `bridge_loader.rs:226` only asserts `install()` returns `Err`; the fixture (not the loader) then manually `del_item`s the reserved entry. No assertion that a failed install rolls back finder/module state.

### LOW — maintainability / test coverage

10. **`crates/sifr_runtime/src/python/bridge_loader.rs:58-68`** — the emitted `import <prefix>.helper` statement (without `asname`) binds the top-level `__sifr_bridge__` name into the executing bridge module's globals. Not a correctness bug, but a namespace leak of the private prefix.

11. **`crates/sifr_lowering/src/lower/python_interop.rs:589`** — the "package has no bridge authority" path emits `PYRES_UNIMPLEMENTED_DECLARATION`, whose canonical meaning is "reserved/not yet implemented," conflating two distinct error conditions.

12. **`crates/sifr_lowering/src/lower/python_interop.rs:597-601`** — `target_module_resolves` accepts any prefix match, so `@python(bridge.pkg.foo.compute)` when only `pkg` is inventoried compiles and fails at runtime rather than diagnosing.

13. **`crates/sifr_codegen/src/python_interop_plan.rs:284-299`** — per-module cache key encodes `source_digest` but not the raw source; consumers that fingerprint via `InteropBuildPlan::cache_key_fragment()` alone would miss source drift when digests are stale-but-matching (defense-in-depth today via `bridge_resolution.rs:137-145`).

14. **`crates/sifr_lowering/src/lower/python_bridge_tests.rs`** — no test asserts authority is module-scoped (a module without an authority entry must still error), and no test covers a nested inventoried module like `pkg.sub`.

15. **`crates/sifr_runtime/src/python/bridge_loader.rs:151-174`** — `reset_for_tests` scrubs `__sifr_bridge__` entries but leaves `sys.modules["__sifr_bridge_loader__"]` and the compiled loader module alive across test cases; complicates leak tracking, not a correctness bug.

---

NOT SATISFIED
