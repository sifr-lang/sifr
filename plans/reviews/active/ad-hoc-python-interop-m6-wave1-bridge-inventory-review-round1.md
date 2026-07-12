Here is the review.

# Review — M6 Wave-1 Bridge Source/Inventory Substrate

Scope confirmed: parser fidelity (Ruff-based `sifr_syntax`, PY3.14), submodule pointer untouched, and the `Cargo.lock` delta matches the two added workspace deps (`sifr_syntax`, `sifr_python_ast`). Discovery is path-locked to `src/python_bridges/**`, misplaced-root scan composes candidates from manifest source_roots, symlinks/reserved paths/`__init__.py` are diagnosed, digests are stable across sorted BTree ordering, and the reserved runtime namespace (`__sifr_bridge__.*`) is correctly detected on both `Import` and `ImportFrom` forms including asname variants.

Findings ordered by severity, then by concreteness:

## MEDIUM

### 1. Wave-1 hard-fails packaging for any bridge source: `__sifr_inventory__.json` is required in the archive but no production code path emits it.
- `validate_package_archive` (`crates/sifr_package/src/cargo/package.rs:61`) now calls `discover_python_bridge_inventory` + `validate_python_bridge_inventory_manifest`, and `required_archive_entries` unconditionally adds `src/python_bridges/__sifr_inventory__.json` when any `.py` file exists (`crates/sifr_package/src/python/bridge_inventory.rs:162-165`).
- `write_python_bridge_inventory` is the only writer, and it has **zero production call sites** — only tests invoke it. `rg` across `crates/` confirms no build, driver, emit, or package command wires it up.
- **Failure scenario**: user creates `src/python_bridges/adapter.py` and runs the package archive validation. `discover_python_bridge_inventory` succeeds with one module, `validate_python_bridge_inventory_manifest` calls `fs::read_to_string` on the non-existent JSON → returns `SIFR-PYIMP-0002 "generated inventory is missing or unreadable: ..."`; simultaneously `required_entries.difference(&archive_entries)` emits `include_exclude_omits_source` for both the `.py` file and the JSON. The user has no CLI or documented method to produce the JSON, so packaging is blocked. Wave plan (`plans/issues/active/ad-hoc-declaration-first-python-interop.md:387-389`) says "Require every bridge source plus **its generated inventory manifest** in package archives" — but wave-1 does not generate it. Either wire `write_python_bridge_inventory` into the package/build path or narrow the archive gate to only fire when the manifest already exists, so wave-2 (which owns identity resolution and rewrite) can activate it atomically.

### 2. `DynamicImportVisitor` rejects any *reference* to `import_module`/`__import__`, not only calls — and reports it as "dynamic import call".
- `visit_expr` (`crates/sifr_package/src/python/bridge_inventory.rs:376-390`) computes `qualified_name` on every expression and flags a match regardless of context. The diagnostic message reads `dynamic import call '<name>' is not allowed` (line 249).
- **Failure scenario**: a bridge module that stashes a callable reference for test injection or attribute inspection — e.g. `resolver = importlib.import_module` inside a fixture, `if hasattr(importlib, "import_module"): ...`, or even a decorator like `@partial(importlib.import_module)` never invoked — trips `SIFR-PYIMP-0002` labeled as a "call" when there is no call. Either scope the walk to `Expr::Call.func` (correct semantic) or rename the message to "dynamic import reference" (honest labeling). The scoped-to-call version also fixes a subtler false positive: `hasattr(importlib, "import_module")` currently is safe (string literal, not qualified name), but `getattr(importlib, x)` isn't — the policy is "no dynamic dispatch surface at all", so the message must match the policy.

### 3. Alias-classifying dynamic-import bypass: `getattr`/tuple-import and `import importlib; loader = importlib` are not tracked.
- `ImportCollector` only tracks `importlib` via `Stmt::Import`/`Stmt::ImportFrom`. Rebindings via `Stmt::Assign` (e.g. `loader = importlib` or `loader = importlib.import_module`) are never propagated to `importlib_aliases`/`dynamic_function_aliases`, so the follow-up `loader.import_module(...)` / `loader(...)` calls are missed.
- **Failure scenario**: bridge source `import importlib\nloader = importlib\nx = loader.import_module('json')` passes inventory. Discovery emits no diagnostic; the manifest lists `importlib` as a `ThirdParty` root — but the dynamic call slipped through. This defeats the "alias-aware dynamic import rejection" scope bullet. Either extend the collector to follow simple `Name = <qualified>` assignments, or document that only import-time aliases are checked and defer to a stricter pass in a later wave — but the current partial coverage silently misses a straightforward bypass.

## LOW

### 4. `parse_bridge_source` short-circuits after the first invalid form and never reports which import statement escaped/collided.
- The relative-escape check at line 230 uses `iter().any(...)` and returns a single generic `"relative import escapes the package bridge source root"`. Dynamic and reserved failures also report only the first offender (`.into_iter().next()`).
- **Failure scenario**: a file with three offending `from ...` statements only shows the first flavor and never the file:line-precise offender. Users must binary-search the file. Collect and report all offenders per file (bridging the existing per-file-batch collection design), or at minimum include the offending module string in the escape message the way the dynamic message already does.

### 5. `module_name` conflates "not a valid Python identifier" and "root `__init__.py` is reserved" into one message.
- The error text (`crates/sifr_package/src/python/bridge_inventory.rs:53`) is fired for both cases: `src/python_bridges/__init__.py` (reserved) and `src/python_bridges/class.py` (keyword). Users cannot tell which condition applies.
- **Failure scenario**: user drops a `src/python_bridges/__init__.py` intending to make the root a package (a plausible mental model since Python packages need it). The message says "module paths must contain valid Python identifiers and root `__init__.py` is reserved" — a wall of two rules — instead of specifically pointing at the reserved-root rule. Split into two distinct reason strings inside `module_name`, or return an enum instead of `Option`, so the caller can format an unambiguous diagnostic.

### 6. Modeling gap in `from . import name1, name2` when `known_modules` partially matches.
- `classify_imports` at line 437-448: when the imported names are a mix of "one submodule that exists" and "one attribute of the base package", the loop sets `found_module = true` on the first match and then **skips** the else-arm that would have recorded `SamePackage { module: base }`. Result: the base-package dependency is lost.
- **Failure scenario**: `pkg/__init__.py` contains `NAME = 1`; `pkg/local.py` exists. In another module: `from . import local, NAME`. Inventory records only `SamePackage { module: "pkg.local" }` — the dependency on `pkg` itself is dropped, so downstream cache-invalidation and resolved-graph closure (M6 later waves) may miss that this file also depends on `pkg`'s init-time state. Fix by always inserting `SamePackage { module: base }` in addition to any matched submodules, or by explicitly modeling "package init" as a separate import edge.

### 7. `import bridge.foo.bar` records only the deepest module — intermediate ancestors are silently dropped.
- `classify_absolute` (line 460-471) inserts a single `SamePackage("foo.bar")` for the `bridge.foo.bar` prefix. In Python semantics, `import bridge.foo.bar` produces `bridge`, `bridge.foo`, and `bridge.foo.bar` in `sys.modules`.
- **Failure scenario**: wave-3 loader / embedded-table generation needs to know all module identities that participate in resolution. If wave-1's inventory is the source of truth, a module that only appears as an intermediate ancestor may be excluded from embedded tables → runtime `ModuleNotFoundError` for `bridge.foo` when a downstream import hits it directly. If ancestor closure is expected to be recomputed later, add a code comment; otherwise materialize the ancestors here.

### 8. Non-UTF-8 encoded bridge source produces an unhelpful message.
- `fs::read_to_string` at line 210 fails with "stream did not contain valid UTF-8" on legacy `# -*- coding: latin-1 -*-` sources; the diagnostic wraps this as `could not read source: ...`.
- **Failure scenario**: legacy Python source using Latin-1 gets a filesystem-flavored error rather than a helpful "bridge sources must be UTF-8; declare `# -*- coding: utf-8 -*-` and re-save". Optional but worth a distinct branch.

### 9. `write_python_bridge_inventory` returns `std::io::Error` instead of `PackageDiagnostic`.
- Every other `sifr_package` public surface returns `PackageDiagnostic` (or a Vec thereof). The new writer forces callers to translate a raw `io::Error` — which will diverge from the `SIFR-PYIMP-0002` convention when the future emit wire-up lands.
- **Failure scenario**: wave-2 wires the writer in the package pipeline and has to hand-roll conversion to `PackageDiagnostic::invalid_python_bridge_source`, duplicating the shape from `validate_python_bridge_inventory_manifest`. Change the return type now to keep the API homogeneous.

### 10. Test-coverage gaps for scope-declared behaviors.
- No test asserts: relative-import escape (`from .. import x` in a top-level bridge module), reserved root `__init__.py` rejection, duplicate module across two files with the *same* module path, symlink rejection, invalid-identifier filename (e.g. `class.py`), or a package's own relative import with a mixed submodule/attribute list.
- **Failure scenario**: regressions in `relative_base`, `misplaced_root_diagnostics`, or `module_name` could ship silently. Given wave-1 is designated the substrate, coverage for its own diagnostics is where wave-1 needs to be strongest. Add at minimum one test each for symlink rejection and relative-escape — the two most likely to regress under refactor.

## NITS

### 11. `pub use bridge_inventory::{...}` is placed *after* the `#[cfg(test)]` mod block in `crates/sifr_package/src/python/mod.rs:29-33`.
- The other `pub use`s are grouped at lines 8-17 before the test-only mods. Move the new one up to match. Purely stylistic.

### 12. `serde_json::to_vec(...).unwrap_or_default()` masks an unreachable serde failure with an empty digest input.
- Line 87. `to_vec` on `(&str, &'static str, &Vec<PythonBridgeModule>)` cannot fail for valid strings; the `.unwrap_or_default()` therefore either does nothing or (theoretically) produces the same empty digest for every collision. Prefer `.expect("bridge inventory digest input serialization must succeed")` — the invariant is real and worth encoding.

### 13. `fnv1a64` is a 64-bit hash — fine for a compiler-internal integrity check, but "digest_algorithm" is serialized into the manifest, so consumers can lean on the name.
- If wave-2/wave-4 rely on the digest for cache-invalidation identity across a large corpus (the plan lists "Fingerprint source/inventory digests"), consider swapping to a broader hash before it becomes a stable public contract; changing the algorithm later invalidates every archived JSON. Not blocking wave-1.

---

VERDICT: CHANGES_REQUESTED
