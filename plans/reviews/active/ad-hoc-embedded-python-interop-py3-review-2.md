# Review 2: milestone_py_3 (Opaque Object Operations and Errors)

Scope: verify whether the blocking findings from review 1 are addressed and call out residual DoD gaps. Branch `ad-hoc-python-interop-py3` at HEAD of the working tree.

Net assessment: B1, B3, M4, M5 are addressed. B2 is **partially** addressed (forgery is now infeasible, but the structural-opaqueness contract is still broken). B4 is **still dead code**; the redesigned data flow only changes the *source* of `native_import_roots`, not the aliasing that made the check unreachable. M1 and M3 are unaddressed; M2 is mostly hardened.

The trust-policy plumbing only flows into the **package-project** build path; **single-file** entrypoints still pass `LoweringOptions::default()`, so HIR-side static literal trust checks are silently disabled for `sifr build/run/check file.sifr`. That is the most consequential residual DoD gap.

---

## Status of prior blocking findings

### B1 — `@trust_python_dynamic` and static HIR import gating — ADDRESSED (with single-file caveat below)

- `crates/sifr_lowering/src/lower/expressions/regular_calls.rs:11-58` adds the HIR check on calls to functions tracked in `python_import_module_bindings`:
  - Literal-string argument → checked against `python_trust_policy.allowed_import_roots` and `trusted_import_roots`; failure → `PYTRUST_UNTRUSTED_IMPORT` (`SIFR-PYTRUST-0002`).
  - Non-literal argument → unless the enclosing function carries `@trust_python_dynamic`, emit `PYTRUST_DYNAMIC_IMPORT_REQUIRES_TRUST` (`SIFR-PYTRUST-0004`).
- `python_import_module_bindings` is populated when `from sifr.python import import_module` (alias-aware) is lowered: `crates/sifr_lowering/src/lower/mod_impl.rs:348-349`.
- `current_function_trusts_dynamic_python` is set from `has_decorator(func, "trust_python_dynamic")` at every function-lowering site: `crates/sifr_lowering/src/lower/typing_and_functions/annotations_and_function_lowering.rs:599`, `crates/sifr_lowering/src/lower/statements/statement_dispatch.rs:682`, `crates/sifr_lowering/src/lower/classes/class_body_lowering.rs:114,204,344`.
- Coverage: `crates/sifr_lowering/src/lower/python_trust_tests.rs:64-115` proves the three branches (untrusted literal → diagnostic; root wildcard → accepted; dynamic without decorator → diagnostic; dynamic with decorator → accepted).
- Diagnostic codes are registered (`crates/sifr_diagnostics/src/codes/registry.rs:75-79`, family entry at `:446-448`).

Caveat: the static-literal check is gated by `if let Some(policy) = &ctx.python_trust_policy`. Trust policy is only injected via `PackagePythonRuntime::lowering_options()` (`crates/sifr_driver/src/build/python_runtime.rs:69-77`), which only fires on the package-project build path (`entrypoint.rs:412-419`). Every single-file entrypoint constructs the plan with `LoweringOptions::default()` — see `entrypoint.rs:136`, `:168`, `:263`, `:561`, and `compile_single_file_entrypoint_with_metadata`/`check_single_file_entrypoint`. Result: a user running `cargo run -q -p sifr -- run /tmp/foo.sifr` that calls `py.import_module("anything")` gets no HIR-time trust enforcement; only the runtime check at `object_ops.rs:221-249` fires. The dynamic-import check is policy-independent and still fires.

This is a genuine DoD ambiguity, not just polish: the milestone says "Enforce `allow-imports`, `[trust] python` … with `SIFR-PYTRUST` diagnostics", but enforcement only happens when the build path runs through a package manifest. Single-file is a documented Sifr CLI surface. Either (a) wire trust policy through the single-file path with a sensible default (e.g., empty allowed roots → reject all literal `py.import_module`), or (b) explicitly carve out single-file mode in the phase doc.

### B2 — `py.Object` opaqueness — PARTIALLY ADDRESSED

- Forgery is now infeasible: `Object` carries `_handle: int` and a 64-bit `_token: int` derived from `RandomState::hash_one((handle, nonce))` at construction (`crates/sifr_runtime/src/python/object_ops.rs:11-12,287-290`). Runtime lookup requires the token to match (`object_ops.rs:209-218,276-285`), so `Object(0, 0)` and other guessed pairs deterministically fail with `SifrPythonClosedObject` — no UB, no Python state corruption.
- Spec lines 273-276 still hold the harder line: "`py.Object` is opaque and foreign … cannot be pattern-matched or structurally typed as a Sifr class." Today:
  - `Object` is a regular Sifr class declared in `lib/sifr/python.sifr:36-42` with public `_handle` and `_token` fields.
  - The stdlib type definition (`crates/sifr_stdlib/src/python.rs:9-19`) exposes both fields as `Type::Int` — user code can read them, pattern-match on them, or define functions taking `Object` and destructuring the fields.
  - The codegen relies on this public surface: `crates/sifr_codegen/src/intrinsics/registry/python.rs:60-66,85-90` reads `__sifr_python_arg._handle` / `._token` directly when packing call args.
  - The public constructor `Object(handle, token)` from `lib/sifr/python.sifr:40-42` accepts arbitrary ints; nothing in HIR prevents that call from being typed.

What the prior review asked for ("opaque/sealed class concept in HIR, or generate the `Object` type as an extern handle (no user-callable constructor) so only intrinsics can mint values") is still not in place. The implementation chose a runtime defense rather than a structural one. If the team decides the random-token gate is sufficient for the milestone — that is a defensible call (the practical blast radius is now "deterministic error", as in review 1, plus the upgrade that forgery is no longer feasible) — please record that decision in the phase doc and tighten spec lines 273-276 accordingly. As shipped, the doc and the code disagree.

### B3 — Root wildcard allowance vs dependency wildcard rejection — ADDRESSED

- `crates/sifr_package/src/python/environment.rs:313-341`: `validate_python_trust_policy` skips the root package (`if &package.package_id == root_package_id { continue; }`) before applying the wildcard rejection across all four lists. Library packages still get `PYTRUST_WILDCARD_REJECTED` (`SIFR-PYTRUST-0001`).
- The resolved environment threads the wildcards through: `declared_imports`/`native_imports` filter `*` out (`:231,260,287`); `allowed_imports` and `trusted_imports` keep `*` so the runtime can short-circuit; `trusted_native_imports` keeps `*` for the same reason (`:71,294-302`).
- The runtime side accepts `*` via `contains_root` (`crates/sifr_runtime/src/python/object_ops.rs:251-255`), so the per-import check at `:228-247` short-circuits when wildcards are present.
- Tests prove both directions: `crates/sifr_package/src/python/tests.rs:145-202` covers the dependency-reject and root-allow paths; `python_trust_requires_allowed_roots_to_be_trusted` and `python_trust_requires_native_roots_to_be_allowed` (`:204-250`) cover the remaining trust invariants.

### B4 — Native-trust runtime check — NOT ADDRESSED

The data-flow rewiring is real but does not make the runtime check reachable.

Today:
- `PackagePythonRuntime::native_import_roots` = `probe.native_imports.iter().filter(|i| i.ok).map(|i| i.root.clone())` (`crates/sifr_driver/src/build/python_runtime.rs:48-53`).
- `probe.native_imports` is populated only for roots passed in `request.native_imports` (`crates/sifr_package/src/python/environment.rs:152-160`), and `request.native_imports` comes from `resolved.native_imports` = `native_python_imports(graph)` = `trusted_python_native_imports(graph)` (`:69,278-292`). That is: only roots declared in `[trust].python-native`.
- `PackagePythonRuntime::trusted_native_roots` = `trusted_native_roots` arg = `resolved.trusted_native_imports` = `trusted_python_native_policy_imports(graph)` (`:72,294-303`) — the same `[trust].python-native` set, with wildcards retained.

The runtime check at `crates/sifr_runtime/src/python/object_ops.rs:240-247`:

```rust
if contains_root(&config.native_import_roots, root)
    && !contains_root(&config.trusted_native_roots, root)
{
    return Err(PythonError::trust(...));
}
```

For any root that ends up in `native_import_roots`, it must already be in `[trust].python-native` (because that is the only source), therefore it is in `trusted_native_roots`. The condition `contains_native && !contains_trusted` is unreachable by construction — the same property review 1 flagged. The only thing that changed is which step strips `*`.

This is still labelled HIGH in the prior review as a "silent security gap". In practice I would soften that to MEDIUM: the intended security property (a root that loads native code without `[trust].python-native` authorization is rejected) is enforced earlier — `PYTRUST_UNTRUSTED_NATIVE_IMPORT` at the package graph (`environment.rs:354-366`) and `PYENV_NATIVE_IMPORT_FAILED` at probe time (`probe_validation`). The runtime check is dead defense-in-depth, not a missing primary gate. But it is still dead code in a security-sensitive surface and should either be removed (with a comment pointing to the earlier enforcement) or rewired to provide an independent signal — e.g., `native_import_roots` derived from declared `allow-imports` roots that the probe observed loading native code, regardless of whether the trust list authorized it.

---

## Status of prior significant findings

### M1 — `exit_context` passes `(None, None, None)` to `__exit__` — NOT ADDRESSED

`crates/sifr_runtime/src/python/object_ops.rs:195-205` still hard-codes:

```rust
.call_method1("__exit__", (py.None(), py.None(), py.None()))
```

The helper has no parameter for an inbound Sifr/Python failure. Spec line 416 still requires the failure context to be passed in. As review 1 noted, even though `py.with` is deferred to milestone_py_6, the signature of this primitive is the migration footgun — the moment `py.with` lands it will need a different signature here, which will be a second touch on a finalized API. If the team intends to revisit during py_6, please add a "milestone_py_6 will plumb failure context" marker in the phase doc.

### M2 — `inject_python_runtime_bootstrap` substring search — MOSTLY ADDRESSED

`crates/sifr_driver/src/build/python_runtime.rs:162-168` now matches `\nfn main() {` (newline + exact `fn main() {`) or `strip_prefix("fn main() {")` at the file start. This eliminates `fn maintain`, `fn main_helper`, and similar fence-post collisions, and is robust against `// fn main()` comments because no preceding newline-then-`fn main()` literal exists in commented-out source from current codegen. Three remaining brittle properties to be aware of:

- A change in codegen formatting that puts a space before `()` or `{` (`fn main () {` or `fn main()\n{`) silently breaks the bootstrap.
- A `r"\nfn main() {"` literal in user source (e.g., a multi-line string in a `print` argument) would still be picked up.
- The detection is purely textual; an alternative would be a stable codegen marker (`// __SIFR_PYTHON_BOOTSTRAP_HERE__`) emitted by the entrypoint module itself.

Acceptable for milestone_py_3, but worth noting in the carry-over list for py_5 (where async-main shapes appear).

### M3 — Generated bootstrap uses `eprintln!` and `std::process::exit(1)` — NOT ADDRESSED

`crates/sifr_driver/src/build/python_runtime.rs:155-157` still embeds:

```text
eprintln!("Sifr Python runtime initialization failed: {}", __sifr_python_runtime_error);
std::process::exit(1);
```

Two distinct concerns from review 1:

- Workspace lint divergence — generated package projects do not inherit `print_stderr = "warn"`, but a user enabling pedantic lints on the generated project gets a lint failure they cannot edit. Easy fix: route through a `sifr_runtime`-owned stderr or diagnostic helper (the runtime crate already owns `print_stderr` allowances elsewhere).
- The `std::process::exit(1)` bypasses any Sifr-level recovery before user `main`. Spec allows this for environment failures, but it should be flagged in the phase doc so reviewers know init-time Python errors are not Result-able.

### M4 — `call_attr` previously discarded intermediate close errors — ADDRESSED

`crates/sifr_runtime/src/python/object_ops.rs:174-181`:

```rust
let callable = get_attr(object, name)?;
let result = call_object(callable, args, kwargs);
let close_result = close_object(callable);
match (result, close_result) {
    (Ok(value), Ok(())) => Ok(value),
    (Ok(_), Err(error)) | (Err(error), _) => Err(error),
}
```

When the call succeeds and the close fails, the close error is now surfaced. When the call fails, the call error is preferred (its stack/context is more useful). Reasonable, and consistent with the "no dropped close errors" goal of review 1.

### M5 — `store_object` handle counter saturation — ADDRESSED

`crates/sifr_runtime/src/python/object_ops.rs:257-274` now uses `checked_add` and returns a typed `PythonOperationFailed("Python object handle space exhausted")` (also for the nonce). Silent overwriting is no longer possible.

### M6 — `Object` constructor doesn't validate handle existence — MITIGATED, NOT FIXED

Constructor `Object(handle: int, token: int)` still accepts any `(int, int)` pair (`lib/sifr/python.sifr:40-42`). The mitigation is the random token: any forged Object that ends up in `clone_handle` returns a deterministic `SifrPythonClosedObject` instead of UB. If B2 is left in its current "partially addressed" state, this finding inherits the same treatment.

---

## Residual DoD gaps for milestone_py_3

1. Single-file builds skip HIR-side static trust enforcement (see B1 caveat). Either thread trust policy through `compile_single_file_*` / `build_cached_single_file_binary` or carve out single-file mode in the phase doc. As shipped, the milestone's "Enforce `allow-imports`, `[trust] python` … with `SIFR-PYTRUST` diagnostics" bullet is only enforced on the package path.
2. Structural opaqueness of `py.Object` is still violated (see B2). Either tighten the spec or wrap `Object` in a sealed/extern construct. Current implementation closes the security blast-radius but not the structural promise.
3. Native-trust runtime check is unreachable (see B4). Either remove the redundant check with a comment pointing to the earlier enforcement, or rewire `native_import_roots` from a source independent of `[trust].python-native`.
4. `exit_context` cannot relay Sifr/Python failure context to `__exit__` (see M1). Either accept a failure-triple parameter now (so milestone_py_6's `py.with` does not have to re-shape this API) or note the deferral in the phase doc.
5. Init-failure bootstrap exits before user code (see M3). Document the chosen behavior in the phase doc; clean up the `eprintln!` so the generated project stays lint-clean.

---

## Optional / future-work (not blocking py_3)

- L1 (cargo-cult `try/except/raise` in `lib/sifr/python.sifr`) — review 1's finding still applies; every wrapper does `try: ...; except PythonError as e: raise e`. Harmless if Sifr's error-handling syntax requires this shape; otherwise simplify.
- L3 (`sys_path` / `site_packages` appended twice into `sys.path`) — `crates/sifr_runtime/src/python.rs:235-251` still iterates `site_packages.iter().chain(sys_path.iter()).rev()` and prepends each, on top of `module_search_paths` set in `Py_InitializeFromConfig`. Cosmetic, not wrong.
- L4 (`runtime_config()` clones the full config on every `import_module` call) — `object_ops.rs:227`. Hot-path cost.
- L5 (two independent substring searches for `sifr_runtime::python::` to set the `PythonRuntime` feature, while the intrinsic registry already declares it) — `entrypoints.rs:166` and `lib_modules_and_codegen.rs:824` still duplicate the detection; `registry.rs:800` already maps `py_*` to `StdlibFeature::PythonRuntime`. Pick one source of truth.
- L7 (Sifr surface lacks `set_attr`, `set_item`, non-string `get_item`) — spec lines 304-308 list these as core operations; milestone_py_3 DoD ("import, attr, item, call, kwargs, close, and context manager") is technically met without setters, but writing this down as deferred would be honest.
- S1/S2 from review 1 (no `@blocking_io` enforcement on Python calls; no Python-specific Result-handling enforcement) — both are deferred to milestone_py_5 / general Result handling, but the phase doc still doesn't list them as known deferrals.
- S3 from review 1 — the phase doc (`plans/issues/active/ad-hoc-embedded-python-interop.md:28-37`) still lists milestone_py_3 as "review round 1 addressed, pending review round 2, PR, and merge"; update once this round is resolved.

---

## Summary

Of the four blocking items from review 1: B1 and B3 are landed and tested; B2 is mitigated but not structurally fixed; B4 is still unreachable code. Of the six MEDIUMs: M2/M4/M5 are landed, M1/M3 are not, M6 is downgraded to defense-in-depth via the new token.

Recommend not merging until B2 and B4 receive either an implementation change or an explicit, written scope decision recorded in the phase doc, and until the single-file-vs-package trust-policy gap in B1 is either closed or carved out. M1 and M3 are independent decisions but should be resolved before milestone_py_6 / milestone_py_5 land on top, where the API signatures get harder to change.
