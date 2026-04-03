## type_system

OK: The code correctly produces:
- `42` (create_user returns the id)
- `Starting...` (direct `&str` matching for `"start"`)
- `Unknown command` (fallback pattern)
- `number: 43` (integer branch adds one)
- `text: hello` (string branch returns text)
- `Alice Smith` (`find_user` returns `Some` for `"alice"`)

## union_ops

OK - the code is correct. Outputs:
- `increment(Some(10))` -> `11`
- `double(Some(3.14))` -> `6.28`
- `safe_len(Some(&names))` -> `3`
- `merge_lists([1,2,3], [4,5,6]).len()` -> `6`

## union_narrowing

OK
