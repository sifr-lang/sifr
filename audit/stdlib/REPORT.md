# Standard Library Audit Report

**4 PASS / 6 FAIL** out of 10 tests.

## Issues Found

### Issue 1: sifr.math Missing Functions
**Test:** 01 | `log`, `sin`, `cos`, `tan`, `abs_val`, `pow_val`, `min_val`, `max_val`, `round_val` not found. Only `sqrt`, `floor`, `ceil`, `pi`, `e` are exported.

### Issue 2: sifr.json Only Accepts Strings
**Test:** 02 | `json_dumps(42)` gives `expected 'str', got 'int'`. The `json_dumps` function only accepts `str` arguments, not arbitrary serializable types. In Python, `json.dumps` accepts any JSON-serializable value.

### Issue 3: sifr.collections.Set Not Exported
**Test:** 05 | `module 'sifr.collections' has no member 'Set'`. The Set type exists (per existing e2e test `stdlib_collections_set.sifr`) but may use a different import path.

### Issue 4: sifr.io Missing Functions
**Test:** 06 | `write_text`, `read_text`, `exists` not found. The module exists but the function names differ from what was tested.

### Issue 5: sifr.env Different API Names
**Test:** 08 | `env_get`, `env_set` not found. The actual function names are different.

### Issue 6: sifr.random Only Works with `list[int]`
**Test:** 09 | `random_choice` expects `list[int]`, not `list[str]`. The function is not generic -- it only works with integer lists.

### Issue 7: sifr.hash Missing `md5`
**Test:** 10 | Only `sha256` is exported, not `md5`.

## What Works
- `sifr.math`: `sqrt`, `floor`, `ceil`, `pi`, `e`
- `sifr.re`: `re_match`, `re_find`, `re_replace`
- `sifr.time`: `time_now`, `sleep`
- String methods: `upper`, `lower`, `find`, `count`, `startswith`, `endswith`, `replace`, `strip`, `split`, `isdigit`, `isalpha`, `isalnum`, `len`

## Notes
The stdlib function names follow a `module_function` convention (e.g., `json_dumps`, `re_match`, `time_now`) rather than Python's `module.function` convention. This is because `import sifr.X` with qualified access (`sifr.X.func()`) doesn't work -- only `from sifr.X import func` is supported.
