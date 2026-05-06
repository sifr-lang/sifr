# Review — INT-2B Stdlib Const Integer Value Exports (pass 1)

Branch: `int-2b-stdlib-const-values`
Reference: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), [internal_docs/integer_model.md](../internal_docs/integer_model.md)
Reviewer scope: correctness, parity with project exports, test adequacy, dependency boundaries, stdlib bootstrap ordering, PR readiness.

## Scope verified

The slice closes the open follow-up flagged in
[reviews/integer-model-int-2b-cross-module-const-fitting-review-pass-1b.md](integer-model-int-2b-cross-module-const-fitting-review-pass-1b.md)
(note 1, "Stdlib bootstrap does not propagate `constant_integer_values`"):

- [crates/sifr_driver/src/stdlib/bootstrap.rs:147](../crates/sifr_driver/src/stdlib/bootstrap.rs:147) — collects per-module integer-value exports from `result.constant_integer_values`, gated by the public-name filter from `result.module.constants`.
- [crates/sifr_driver/src/stdlib/bootstrap.rs:336](../crates/sifr_driver/src/stdlib/bootstrap.rs:336) — inserts the per-module map into `stdlib_defs.constant_integer_values` only when non-empty, mirroring the project-side empty-map suppression.
- [crates/sifr_driver/src/stdlib/bootstrap.rs:390](../crates/sifr_driver/src/stdlib/bootstrap.rs:390) — new `collect_public_constant_integer_value_exports` helper, generic over `T: Clone`, intersecting an iterator of public names with the recorded-value map.
- [crates/sifr_driver/src/stdlib/bootstrap.rs:448](../crates/sifr_driver/src/stdlib/bootstrap.rs:448) — focused unit test using `i32` to keep the test out of the `num-bigint` blast radius.

No other files are touched.

## Correctness assessment

### Parity with project exports

The reference producer is
[crates/sifr_driver/src/project/exports.rs:93](../crates/sifr_driver/src/project/exports.rs:93).
Its loop over `module.constants` does two things in one pass: builds the
type-export map for non-`_`-prefixed names, and writes the integer-value
export when `lowering_result.constant_integer_values.get(name)` is `Some`.

The bootstrap mirrors this with two consecutive loops over the same source
of truth (`result.module.constants`):

1. `bootstrap.rs:142–146` — populate `const_exports` for non-`_`-prefixed
   names (pre-existing).
2. `bootstrap.rs:147–154` — feed those same public names into the helper,
   which filters by `result.constant_integer_values`.

Both passes start from the *locally-defined* `module.constants`, so neither
re-exports imported aliases — `from sifr.bar import LIMIT` in `sifr.foo`
will not surface `LIMIT` under `stdlib_defs.constant_integer_values["sifr.foo"]`.
That is the same rule the project-side enforces, and matches the
"export only constants the producer itself defined" rule called out in the
prior review's note 5.

The empty-map suppression at `bootstrap.rs:336` matches the equivalent
guard at `exports.rs:126`. The visibility filter (`!name.starts_with('_')`)
matches `exports.rs:94`. Module-key shape (`(*module_name).to_string()`)
matches the rest of the bootstrap and yields the same `String` keys the
consumer-side lookups in
[crates/sifr_hir/src/lower/imports.rs:104](../crates/sifr_hir/src/lower/imports.rs:104),
[crates/sifr_hir/src/lower/mod.rs:929](../crates/sifr_hir/src/lower/mod.rs:929),
[crates/sifr_hir/src/lower/mod.rs:1083](../crates/sifr_hir/src/lower/mod.rs:1083),
and [crates/sifr_hir/src/lower/compat_imports.rs:158](../crates/sifr_hir/src/lower/compat_imports.rs:158)
already use against `externals.constant_integer_values`. All four consumer
sites benefit from the new bootstrap output without further changes.

### Bootstrap ordering

`STDLIB_FILES` in
[crates/sifr_driver/src/stdlib/registry.rs:1](../crates/sifr_driver/src/stdlib/registry.rs:1)
is iterated in declaration order. When a later stdlib module imports an
integer constant from an earlier one, the earlier module's
`stdlib_defs.constant_integer_values["sifr.X"][NAME]` is already in place
when `lower_module_stdlib_with_externals` runs for the dependent module
([crates/sifr_hir/src/lower/mod.rs:505](../crates/sifr_hir/src/lower/mod.rs:505)).
The dependent module's `LoweringResult.constant_integer_values` may contain
both locally-defined and imported entries — but the new bootstrap export
filter only re-exports entries that are also in *its own*
`result.module.constants`, so transitive re-export does not accidentally
sneak in. This matches existing project-side behavior.

The bootstrap is also cached behind
[crates/sifr_driver/src/stdlib/cache.rs](../crates/sifr_driver/src/stdlib/cache.rs)
(`get_or_init_stdlib_cache(&STDLIB_COMPILED_CACHE, …)`), so the new field
becomes part of the cached `StdlibCompiled.defs` and is stable for the
lifetime of the process — no staleness or repeat-cost concerns.

### Dependency boundary

`sifr_driver/Cargo.toml` does not depend on `num-bigint`, and the change
preserves that. The helper is generic over `T: Clone`, so the bootstrap
can pass `&HashMap<String, BigInt>` (whose concrete type is owned by
`sifr_hir`) without `sifr_driver` needing to name `BigInt` directly. The
unit test exercises the helper with `i32`, which is consistent with the
slice description.

### Panic / no-user-path violations

No `unwrap`/`expect` on user-reachable paths. The helper uses `get(...).map(...)`
inside `filter_map`. Cloning a `BigInt` is allocating but infallible. No
new monolithic file or HIR guardrail risk.

### Stdlib coverage opportunity

There are several stdlib `.sifr` files with module-level integer constants
that this change makes foldable across the boundary:

- `lib/sifr/calendar.sifr` — `MONDAY..SUNDAY`
- `lib/sifr/logging.sifr` — `DEBUG`, `INFO`, `WARNING`, `ERROR`, `CRITICAL`, `NOTSET`
- `lib/sifr/subprocess.sifr` — `PIPE`, `STDOUT`, `DEVNULL`
- `lib/sifr/time.sifr` — `TIMEZONE`, `ALTZONE`, `DAYLIGHT`
- `lib/sifr/re.sifr` — `IGNORECASE`, …

After this slice, `from sifr.calendar import MONDAY` followed by
`day: uint8 = MONDAY` should fold to `IntLiteral(0)` via the existing
`fixed_width_fitting::const_integer_value` path. Worth highlighting that
the data this slice wires through is real and reachable, not theoretical.

## Test adequacy

### What the unit test does cover

- Recorded-value filter: `MISSING` is in `public_constant_names` but not in
  `values` → not in output. ✓
- Public-name list as filter: `STALE` has a recorded value but is not in
  `public_constant_names` → not in output. ✓
- Generic-over-`T` shape: the test uses `i32`, proving the helper does not
  pin `BigInt` into `sifr_driver`'s test surface. ✓

### What the unit test does NOT cover

The slice description says "Add a focused helper/unit test for **public-name
filtering** and recorded-value filtering". The helper itself does not
implement the public-name filter — that is enforced at the **call site**
in `bootstrap.rs:152` via
`filter_map(|(name, _, _)| (!name.starts_with('_')).then_some(name.as_str()))`.
The test name
`public_constant_integer_value_exports_filter_to_public_recorded_values`
implies the helper enforces both filters, but it only enforces the
intersection with `constant_integer_values`.

The `assert!(!exports.contains_key("_PRIVATE"))` line is therefore
effectively tautological: `_PRIVATE` was never passed in the
`public_constant_names` iterator (`["ANSWER", "MISSING"]`), so its absence
in the output proves nothing about the underscore-prefix discipline. The
real public-name filter at the bootstrap call site has no direct
regression-blocking test.

This is a real (small) gap. The simplest tightening is one of:

1. Rename the test (and possibly the helper) to clarify that the helper
   only does the recorded-value intersection, and have the test focus on
   that contract. The underscore-prefix discipline is then tested
   indirectly by the existing project-graph tests pattern.
2. Or, write a small wrapper in the test that runs the same call-site
   filter (`(!name.starts_with('_')).then_some(name.as_str())`) over a
   `&[(String, Type, HirExpr)]`-shaped fixture and asserts that
   `_PRIVATE` is dropped before reaching the helper.

### Missing integration coverage

The project-side analogue
([crates/sifr_driver/src/tests/project_graph.rs:602](../crates/sifr_driver/src/tests/project_graph.rs:602))
proves end-to-end that `BASE: int = 250 + 4` in a sibling module folds
into `value: uint8 = LIMIT + 1` → `IntLiteral(255)`. The stdlib side does
not get a symmetric integration test in this slice. A minimal addition
would be a project-graph test that imports a real stdlib constant
(e.g. `from sifr.calendar import MONDAY` or `from sifr.logging import DEBUG`)
and asserts the fitted let body collapses to a literal in a `uint8` /
`uint32` slot. Without this, the only signal that `stdlib_defs.constant_integer_values`
actually flows through the four consumer sites is by code inspection — no
test would fail if a future change broke one of them.

This is the most actionable improvement for a follow-up. Not a blocker
for the slice as scoped, but it would be a small, high-leverage addition.

## Style / minor notes (non-blocking)

1. **Helper naming.** `collect_public_constant_integer_value_exports` over-promises
   slightly — the helper itself only does the recorded-value intersection;
   the "public" framing belongs at the call site. A name like
   `select_constant_integer_value_exports` or `intersect_constant_integer_values`
   would be more accurate. Pure naming.
2. **Two-pass vs single-pass.** The producer-side `exports.rs:93` does the
   type-export and integer-value-export filters in a single loop. The
   bootstrap does them as two consecutive loops over the same source
   collection. Functionally equivalent; trivial cost. Bootstrap-side state
   already requires a separate intrinsic-import path for `const_exports`,
   so a two-pass shape is reasonable here.
3. **Helper signature.** `impl Iterator<Item = &'a str>` accepting the
   pre-filtered name view is a clean API choice — no `Vec` allocation, no
   `String` ownership transfer at the call site. Keeps the helper testable
   with simple `["...", "..."].into_iter()` literals.

## Validation review

The reported local validation set covers the relevant surfaces:

- `cargo fmt --check` — formatting clean (re-verified).
- `cargo test -p sifr_driver public_constant_integer_value_exports -- --nocapture`
  — passes (re-verified).
- `cargo test -p sifr_driver project_lowering -- --nocapture` — three tests
  pass, including the constant-export and import-fold tests (re-verified).
- `cargo clippy -p sifr_driver -- -D warnings` — clean (re-verified).
- `scripts/run_all_tests.sh --profile quick` — signature
  `e1bf653aaa770517`, wall time 63.44s.

No additional validation gaps for the slice's stated scope. The two test
gaps noted above (helper-test naming/coverage and missing stdlib-side
integration test) are about *what is tested*, not whether the validation
suite was run.

## Readiness

The change is small, focused, and accurately mirrors the producer-side
behavior in `project/exports.rs`. The `T: Clone` generic preserves the
no-`num-bigint`-in-`sifr_driver` discipline. Bootstrap iteration order is
sound for cross-stdlib-module integer-constant imports. All four
consumer-side lookup sites already read `externals.constant_integer_values`,
so this slice plugs in cleanly with no consumer-side edits.

The unit test has a minor mismatch between its name and what it actually
asserts (the `_PRIVATE` assertion is not exercising the helper's actual
contract), and there is no integration test that a stdlib integer constant
folds end-to-end. Both are small, non-blocking gaps appropriate for a
follow-up.

VERDICT: SATISFIED
