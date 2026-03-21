# `wave_clone_3` Generic Hardening Traceability

Phase: `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md`

## Scope

`wave_clone_3` completes generic hardening and closure for the ownership-aware collection lowering phase:

- Fix Option[List]/Option[Dict] union-case indexing hardcoded `.cloned()` (FINDING-1 from wave_clone_2 pass-1)
- Fix set symmetric_difference hardcoded `.cloned()` (FINDING-2 from wave_clone_2 pass-1)
- Document conservative generic handling as explicit and deterministic
- Regression coverage for all targeted surfaces

## Key implementation points

### Finding-1: Option union-case indexing (`intrinsic_method_emitters.rs`)

In `try_lower_registry_expr_strict`, the Option-wrapped `Type::Dict` and `Type::List` arms now use `option_projection_method_for_owned_type` instead of hardcoded `"cloned"`:

- `Type::Dict(key_ty, value_ty)` → `option_projection_method_for_owned_type(value_ty.as_ref())` → `"copied"` for Copy types, `"cloned"` for Move types
- `Type::List(element_ty)` → `option_projection_method_for_owned_type(element_ty.as_ref())` → `"copied"` for Copy types, `"cloned"` for Move types

### Finding-2: Set symmetric_difference (`intrinsic_method_emitters.rs`)

In `try_lower_registry_set_method_call_expr`, the symmetric_difference projection now derives from the set's element type:

- Function signature updated to accept `object_ty: &Type`
- `resolved_object_ty = resolve_alias_type_for_plain_call(object_ty)`
- `set_element_projection = option_projection_method_for_owned_type(elem_ty.as_ref())`
- `method: set_element_projection` instead of `method: "cloned".to_string()`

Note: The `methods/set.rs::lower_symmetric_difference` fallback also hardcodes `.cloned()` but is never reached for typed `HirExpr` calls since the registry path handles `symmetric_difference`.

## Evidence: clone-elision outcomes

Command:
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/wave_clone_3_generic_hardening.sifr`

Observed shape:

- `Option[List[int]]` safe indexing uses copy extraction: `__sifr_index_list.get(__sifr_index_norm).copied()`
- `Option[Dict[str, int]]` safe indexing uses copy extraction: `maybe_scores.get("alice").copied()`
- `Option[Dict[str, str]]` safe indexing uses clone extraction: `maybe_strs.get("a").cloned()`
- `set[int].symmetric_difference` uses copy projection: `.symmetric_difference(&__other).copied().collect::<HashSet<_>>()`
- `set[str].symmetric_difference` uses clone projection: `.symmetric_difference(&__other).cloned().collect::<HashSet<_>>()`

## Wave artifacts

- pass fixture: `crates/sifr/tests/e2e/pass/wave_clone_3_generic_hardening.sifr`
- demo: `demos/ad_hoc_clone_wave3_generic_hardening_demo.sifr`

## Validation snapshot

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/wave_clone_3_generic_hardening.sifr` -> PASS
- `cargo run -q -p sifr -- run demos/ad_hoc_clone_wave3_generic_hardening_demo.sifr` -> PASS
- `cargo test -p sifr_codegen helpers::tests` -> 18 tests, all pass
- `cargo build -p sifr` -> compiles cleanly
- `cargo clippy -p sifr_codegen -- -D warnings` -> 2 pre-existing errors (not from this wave)
