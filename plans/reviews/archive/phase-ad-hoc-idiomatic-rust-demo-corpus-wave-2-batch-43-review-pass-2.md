## type_system

OK

## union_ops

OK. The four outputs are:
1. `increment(Some(10))` -> `11`
2. `double(Some(3.14))` -> `6.28`
3. `safe_len(Some(&names))` -> `3`
4. `merge_lists([1, 2, 3], [4, 5, 6]).len()` -> `6`

## union_narrowing

OK. The Rust code correctly implements the route outputs, pet descriptions, `found`/`not found`/`true`, and `no items`/`3 items` behavior through direct enum and option narrowing.
