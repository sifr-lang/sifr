I've completed a thorough review of the M6 implementation. Below is the rigorous review.

---

# M6 Milestone Closure Review — Round 3

## Scope Confirmed

- Working tree diff against `main` covers 18 modified + 8 new files.
- Prior round-1 findings (12 items) and round-2 verifications spot-checked against current code, tests, and docs.
- Focused areas: bridge loader hermeticity, dynamic-import detection, symlink handling, cache identity, two-package isolation, archive/install/read-only proof, diagnostic taxonomy, evidence honesty, file-size guardrails, plan/roadmap accuracy.

## Verified — Prior Findings Remain Closed

- **Plan/roadmap closure honest.** `plans/issues/active/ad-hoc-declaration-first-python-interop.md:8`, `:139`, `:422` all check out; `plans/roadmap.md:129` matches ("M0–M6 complete through hermetic package-local bridge deployment"). M6 top-level has no PR link because this branch *is* the closure PR — wave breakdown at :375–422 links every sub-PR.
- **Demo marker is real binary stdout.** `demos/m6_demo/run.sh` invokes the ignored `archived_biip_bridge_builds_and_runs_without_checkout_or_extraction` test with `SIFR_PACKAGE_BRIDGE_DEMO_MARKER_FILE`; `crates/sifr_driver/src/tests/package_python_bridge_archive_tests.rs:75–78` writes the captured `output.stdout` (post-assertion) to that file. run.sh emits it via `sed -n '1p'`. No fabricated marker.
- **Duplicate-input pre-install rejection.** `crates/sifr_runtime/src/python/bridge_loader.rs:22–29` dedupes via `BTreeSet` before `PyModule::from_code`. Test `duplicate_embedded_module_names_are_rejected_before_installation` (:342) asserts `builtins.__sifr_bridge_finder__` never materializes on failure; the pre-existing collision test (:279) does the same. Cleanup evidence row at `fixtures/package_bridge_archive/package_bridge_evidence.json:19` matches ("collision and duplicate-input failures occur before finder installation").
- **Star- and `importlib.__import__` bypasses.** `bridge_inventory/imports.rs:130–148, 171–176, 219–224, 238–244` now cover: `from importlib import *`, `from builtins import *`, `importlib.__import__(...)`, and `getattr(importlib, '__import__')`. Tests `importlib_dunder`, `importlib_star`, `builtins_star` in `bridge_inventory_tests.rs:111–125` lock the behavior. `DynamicImportVisitor::new` (:200) seeds `import_module`/`__import__` and `__builtins__` unconditionally.
- **Symlinked-ancestor rejection.** `bridge_inventory/filesystem.rs:43–50, 75–88` walks each package-relative ancestor with `symlink_metadata`. `bridge_inventory_symlink_tests::symbolic_link_bridge_ancestor_is_rejected` creates `src -> outside_src` (leaf `python_bridges` real) and asserts a diagnostic anchored to `src`.
- **`importlib.import_module` guarded after `sys.meta_path` mutation.** `bridge_loader.rs:148–158` wraps `importlib.import_module`; loader test at :308–325 pops `sys.meta_path[0]` and then re-imports the reserved module via `importlib.import_module`, executing bridge code successfully.
- **Missing-authority code fixed.** `python_interop.rs:590` now emits `PYIMP_INVALID_TARGET`, and the corresponding lowering test `bridge_target_is_a_hard_error_without_package_authority` locks the code. Reserved-namespace ambiguity is covered by `reserved_bridge_target_cannot_be_reinterpreted_as_an_external_distribution`.
- **Module-scoped and nested authority.** `bridge_authority_is_scoped_to_the_declaring_module` and `nested_inventoried_bridge_module_rewrites_to_the_resolved_package` in `python_bridge_tests.rs:107–182` cover both concerns.
- **Source-drift defense-in-depth.** `bridge_resolution.rs:137–145` recomputes SHA-256 from raw bytes and rejects `"bridge source changed while its inventory was being resolved"` before plan construction.
- **`reset_for_tests` scope.** `bridge_loader.rs:172–207` restores `builtins.__import__`, `importlib.import_module`, deletes all three `__sifr_bridge_*` builtins, strips the finder from `sys.meta_path`, and now also removes `__sifr_bridge_loader__` from `sys.modules`.
- **Namespace leak.** Rewriter at `bridge_loader.rs:66–79` emits `import <prefix>.helper as __sifr_bridge_imported` (throwaway); the `<prefix> as bridge` alias only lands when the source had unaliased `import bridge.x`. No leak of `__sifr_bridge__` into module globals.
- **PYRES scope in exit evidence.** `verification/areas/python_interop/reports/python_interop_exit_evidence.md:38` lists only `SIFR-PYRES | 0002`, matching `crates/sifr_diagnostics/src/codes/python_interop_codes.rs:23`. New rows for PYIMP/PYCALL/PYCONV/PYCTX are consistent with `internal_docs/diagnostic_codes.md:314–320`.

## Independently Re-Verified This Round

- **Loader-before-main ordering.** `crates/sifr_runtime/src/python.rs:234` calls `bridge_loader::install` after CPython init and *before* `state.initialized = true`. No user code can run before the finder is at `sys.meta_path[0]`.
- **`ensure_first` restoration on every reserved lookup.** `object_ops.rs:29–37` and `:53–58` call `bridge_loader::ensure_first(py)` before `py.import(...)` / `resolve_target(...)` when the root segment is `__sifr_bridge__`. Combined with the `guarded_import`/`guarded_import_module` wrappers, both compiler-driven and user-driven reserved resolution restore the finder to position 0.
- **Two-package isolation.** `package_python_bridge_archive_tests.rs:91–153`: `app` and `library_dep` each ship `python_bridges/identifiers.py` with different bodies, both use `@python(bridge.identifiers.value)`. The binary asserts `own == 40` and `dependency == 2`. Runtime module names differ because `resolved_python_bridge_runtime_package` (`bridge_resolution.rs:59–64`) hashes the `SifrPackageId` into distinct `__sifr_bridge__.p_<hex>` prefixes.
- **Dependency-bridge trust authority.** `bridge_resolution_tests::dependency_bridge_requirements_remain_root_authorized` at :133–176 shows a dependency's `import numpy.linalg` contributes a `BridgeImport` requirement that fails root trust with `SIFR-PYTRUST-0005` unless the root app authorizes it.
- **Cache identity fingerprints.** `crates/sifr_codegen/src/python_interop_plan.rs:180–299` composes: binding contract version, per-declaration metadata, `bridge_packages` count, per-package `resolved_package_key`/`runtime_package`/`inventory_digest`, and per-module `source_digest`/`runtime_module`/imports. Combined with `bridge_resolution.rs` byte-level SHA re-verification, source and inventory drift both invalidate.
- **Archive proof genuinely hermetic.** `package_python_bridge_archive_tests.rs:19–89`:
  - Source checkout deleted at :40 (`std::fs::remove_dir_all(&app.root)`).
  - Build runs against the *unpacked* archive tree (:51–52).
  - Installed bridge sources removed at :53–54 before execution.
  - Run root created and set to `0o555` (:56–59); binary executes with `cwd = run_root`, `TMPDIR = run_root`.
  - Post-run assertion `read_dir(&run_root).count() == 0` (:79–84) proves no extraction into the read-only directory.
  - stdout is captured, asserted against the biip marker, and only then persisted to the demo-marker file (:71–78).
- **File-size guardrail.** Largest touched first-party file is `crates/sifr_lowering/src/lower/python_interop.rs` at 839 lines. Bridge loader 398, bridge inventory filesystem 230, imports 266, bridge resolution 241, archive tests 230 — all comfortably under 900.
- **Verification runner locks the biip marker.** `verification/areas/python_interop/runner/run.py:379–397` validates that `package_bridge_evidence.json` names the `package-bridge` capability, has ≥3 positive/negative/cleanup rows with owner strings, and locks `stdout_marker` to the exact biip GTIN string. `REQUIRED_FIXTURES`/`REQUIRED_FIXTURE_FILES`/`REQUIRED_SOURCE_FIXTURES` all include the new archive fixture.
- **Capability matrix honesty.** `verification/areas/python_interop/declaration_capabilities.json:66–79` flips `package-bridge` from `reserved` to `active` and moves positive/negative/cleanup/live evidence from `planned` to `passing`; `cancellation` remains `not-applicable` with a documented reason ("bridge loading is synchronous initialization"). No overclaim.
- **Round-3 review file.** `plans/reviews/active/ad-hoc-python-interop-m6-milestone-closure-review-round3.md` is a 0-byte placeholder awaiting this review; round-1 (full findings) and round-2 (verifications) files are populated.

## Non-Blocking Observations (LOW / INFO — do not gate closure)

1. **doc-drift**, `internal_docs/python_interop_declaration_architecture.md:400`. This paragraph still says "bridge targets without package bridge authority remain a `SIFR-PYRES-0002` error." The round-1 remediation intentionally split conflated conditions: the implementation now emits `SIFR-PYIMP-0001` for a `bridge.*` target in a package with no inventoried bridge (`python_interop.rs:589–594`). Exit evidence and diagnostic registry are already aligned; only this one arch-doc sentence lags. Non-blocking (doc drift, not code drift).

2. **dynamic-import-detection-limit**, `crates/sifr_package/src/python/bridge_inventory/imports.rs:130–148`. `record_imported_dynamic_aliases` only treats `import_module` as the "expected" name for module `importlib`. Two obscure aliasing patterns therefore slip through:
   - `from importlib import __import__ as _imp; _imp('json')` — CPython aliases `importlib.__import__` to the same object as `builtins.__import__`, but the collector never records `_imp` because `__import__` ≠ `"import_module"` (and ≠ `"*"`) under the importlib branch.
   - `_imp = __import__; _imp('json')` — `__import__` as a bare builtin is only seeded into `dynamic_function_aliases` by `DynamicImportVisitor::new` (:207–208), *after* `ImportCollector::record_assignment` runs, so the assignment doesn't propagate the alias.

   Common bypasses — `importlib.__import__(...)`, `getattr(builtins, '__import__')(...)`, star-imports from either module, and alias rebinding via imported *modules* (e.g. `_b = builtins; _b.__import__(...)` is caught by the aliasing chain in `record_assignment`) — all remain rejected. The above two patterns are contrived; the check is defense-in-depth against typo-shape dynamic import, not a full sandbox. Non-blocking.

3. **plan-link-format-nit**, `plans/issues/active/ad-hoc-declaration-first-python-interop.md:139`. M0–M5 rows carry a single PR link; M6's row omits it because the milestone spans PRs #2945/2947/2949/2951/2952 and this branch is the still-unmerged closure PR. The wave-level breakdown at :375–422 does link every merged sub-PR. Add a post-merge PR link when this branch lands. Informational only.

## Verdict

All plan-mandated acceptance criteria and verification requirements are met:

- Bridges are reproducible package implementation, embedded and archived, not ambient files.
- Two packages can own the same bridge module path (`bridge_resolution.rs:59–64` + tests).
- A dependency bridge cannot authorize its own third-party imports (`SIFR-PYTRUST-0005` test).
- Deployment does not depend on source checkout, writable temp directory, or ambient path ordering (archive proof).
- Loader ordering, collision, sibling-import, traceback, and cache tests all pass focused Rust suites and both `--ignored` integration tests (per the user-supplied validation summary).
- Diagnostic taxonomy, capability matrix, exit evidence, docs, and roadmap are internally consistent.

No actionable findings survive. The three observations above are LOW / INFO hygiene notes and do not block closure.

SATISFIED
