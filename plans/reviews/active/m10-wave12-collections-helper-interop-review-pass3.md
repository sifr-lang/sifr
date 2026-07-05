## Findings

**Alignment of `own` convention on int params — satisfied.**

- `_set_add_impl`
  - Model: `("item".to_string(), Type::Int, ParamConvention::own())` ✓
  - Adapter: `def _set_add_impl(own s: list[int], own item: int)` ✓
- `_set_remove_impl`
  - Model: `("item".to_string(), Type::Int, ParamConvention::own())` ✓
  - Adapter: `def _set_remove_impl(own s: list[int], own item: int)` ✓

**Cross-checks on the rest of the surface (no drift introduced).**

- `_new_set_impl`: model `all_borrow(vec![], ...)`, adapter no params — consistent.
- `_set_from_list_impl`: model `own` on `items`; adapter `own items: list[int]` — consistent.
- `_set_contains_impl`, `_set_len_impl`: model `all_borrow`; adapter has no `own` markers — consistent (borrow default).
- `_set_union_impl`, `_set_intersection_impl`: model `own` on both `a`/`b`; adapter `own a, own b` — consistent.
- `_defaultdict_new_impl` / `_defaultdict_get_impl` / `_defaultdict_set_impl`: model `all_borrow`; adapter has no `own` — consistent.

**Other checks.**

- Naming: all identifiers are `_<name>_impl` on both sides, matching the private-adapter policy.
- `@rust(...)` targets under `sifr_stdlib.collections.*` line up 1:1 with the public helper names being adapted.
- No stray public declarations remain in the model for the migrated set/defaultdict surface; Counter operations correctly left in place (not part of this wave's cleanup).
- Validation gates cited (fmt-check plus the two targeted `sifr_driver` tests covering codegen-through-stdlib and adapter-policy syntax) are the right ones to catch a convention mismatch — both would fail if adapter `own` markers didn't match model `ParamConvention`.

## Verdict

**PASS.** The pass-2 optional cleanup is satisfied: `_set_add_impl` and `_set_remove_impl` now use `own item: int` in `stdlib/_sifr/collections.sifr`, matching `ParamConvention::own()` in `crates/sifr_stdlib_model/src/collections_bytes_time.rs`. No new inconsistencies introduced elsewhere in the diff. Cleared to merge.
