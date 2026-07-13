Based on my re-review of the diff after round-1 remediations:

**Verified remediations:**
1. **Plan closure**: `plans/issues/active/ad-hoc-declaration-first-python-interop.md:139` and `:422` are `[x]`; status line reads "M0 through M6 are implemented". Roadmap:129 aligns.
2. **Round-1 artifact**: `ad-hoc-python-interop-m6-milestone-closure-review-round1.md` contains the full consolidated findings.
3. **Demo marker**: `demos/m6_demo/run.sh` runs the ignored Rust integration test with `SIFR_M6_DEMO_MARKER_FILE`; `package_python_bridge_archive_tests.rs:75-78` writes the real captured `output.stdout` (post-assertion) to that file, and run.sh emits it via `sed -n '1p'`. No fabrication.
4. **Duplicate rejection**: `bridge_loader.rs:23-29` dedupes via `BTreeSet` before any Python-side install; test `duplicate_embedded_module_names_are_rejected_before_installation` asserts `__sifr_bridge_finder__` never appears on builtins.
5. **Dynamic-import detection**: `imports.rs:130-148` handles `*` alias for both `importlib` and `builtins`; `dynamic_callable_name` matches `importlib.__import__` (`:222-224`); tests `importlib_dunder`, `importlib_star`, `builtins_star` are present.
6. **Symlinked ancestor**: `filesystem.rs:43-50, 75-88` walks each package-relative ancestor; `symbolic_link_bridge_ancestor_is_rejected` covers `src/`.
7. **importlib.import_module guard**: `bridge_loader.rs:148-158` wraps `importlib.import_module` and re-hoists the finder; test at `:308-325` pops `sys.meta_path[0]` then invokes `importlib.import_module` and asserts recovery.
8. **PYRES scope**: Exit evidence lists only `SIFR-PYRES | 0002`.
9. **Cleanup evidence**: `package_bridge_evidence.json:18-23` states duplicate/collision failures occur before finder install — matches tests.
10. **Namespace leak**: rewriter emits `import <prefix>.helper as __sifr_bridge_imported` (throwaway); `<prefix> as bridge` only when the source had unaliased `import bridge.x` — no `__sifr_bridge__` binding leaks.
11. **Missing authority code**: `python_interop.rs:590` emits `PYIMP_INVALID_TARGET`.
12/14. Prefix-match retention is documented and covered by module-scoped (`bridge_authority_is_scoped_to_the_declaring_module`) and nested (`nested_inventoried_bridge_module_rewrites_to_the_resolved_package`) tests.
13. `bridge_resolution.rs:137-145` recomputes and verifies SHA-256 from raw bytes before plan construction.
15. `reset_for_tests` restores both `builtins.__import__` and `importlib.import_module`, deletes the finder/handles, and removes `__sifr_bridge_loader__` from `sys.modules`.

**Other checks:**
- Runtime bootstrap wires the loader at `python.rs:234` before `state.initialized = true`.
- Runner (`run.py:379-397`) validates capability/matrix rows and locks the biip marker.
- `declaration_capabilities.json` has `package-bridge` active with all required evidence `passing`; reserved rows remain for the self-test.
- File-size guardrail: largest touched file is `python_interop.rs` at 839 lines (< 900).
- Two-package isolation test is intact at `package_python_bridge_archive_tests.rs:91-153`.

No remaining actionable findings.

SATISFIED
