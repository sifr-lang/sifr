# wave_psp_c1 Structured Parsing and Serialization - Review Pass 1

## Executive Summary

wave_psp_c1 covers structured parsing and serialization for `json`, `tomllib`, `csv`, and `configparser`. The implementation is largely complete and functional, with comprehensive test coverage and demos. However, several correctness gaps and production risks were identified that should be addressed.

**Status**: Implementation complete, tests passing, review identifies issues requiring remediation.

---

## Test Validation

All test files pass:

```
$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_c1_structured_parsing_serialization.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_csv_subset.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_configparser.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_tomllib_subset.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_json_subset.sifr
Exit: 0

$ cargo run -q -p sifr -- run demos/wave_psp_c1_structured_parsing_serialization_demo.sifr
Exit: 0
```

---

## Correctness Gaps

### 1. ConfigParser.read() Does Not Parse (Bug - High Severity)

**Location**: `lib/sifr/configparser.sifr` lines 217-223

**Issue**: The `read()` method reads a file but never parses its contents.

```sifr
def read(self, path: str) -> Result[list[str], IOError]:
    try:
        _: str = read_text(path)  # Reads but ignores content
        loaded_path: str = path + ""
        return [loaded_path]       # Returns path without parsing
    except IOError as e:
        raise e
```

**Expected behavior**: Should read the file and call `read_string()` to parse the INI content into the ConfigParser instance.

**Impact**: Users calling `parser.read("config.ini")` will get a successful return but the configuration will not be loaded.

**Recommendation**: Fix to call `read_string()` on the loaded content:

```sifr
def read(self, path: str) -> Result[list[str], IOError]:
    try:
        content: str = read_text(path)
        _: None = self.read_string(content)
        loaded_path: str = path + ""
        return [loaded_path]
    except IOError as e:
        raise e
    except ParsingError as e:
        raise Error(e.message)
```

---

### 2. JSON Serialization Silently Converts Errors to "null" (Production Risk)

**Location**: `crates/sifr_codegen/src/intrinsics/json.rs` lines 746-754

**Issue**: The `json_dumps_value` intrinsic uses `unwrap_or_else` with a closure that returns the string `"null"` on serialization failure, rather than propagating the error.

```rust
.method("unwrap_or_else".to_string(),
    args: vec![RustExpr::Closure {
        // ...
        body: Box::new(string_expr("null")),  // Returns "null" on error
        // ...
    }],
```

**Impact**:
- Serialization errors (e.g., cyclic references, invalid UTF-8) silently produce `"null"` output
- Users cannot distinguish between actual null values and serialization failures
- This differs from CPython's behavior which raises `JSONEncodeError`

**Current workaround**: The Sifr JSON module exports `dumps(value: JsonValue) -> str` (not `Result`), so this is a design decision, but it creates a production risk.

**Recommendation**: Either:
1. Change `dumps` to return `Result[str, JSONEncodeError]` and propagate errors
2. Document this as intentional adapted behavior in the traceability ledger

---

### 3. JSON Integer Overflow Handling May Be Too Strict

**Location**: `crates/sifr_codegen/src/intrinsics/json.rs` lines 267-276

**Issue**: When parsing JSON, unsigned 64-bit integers (u64) that exceed `i64::MAX` are rejected with an error rather than being converted to float.

```rust
RustStmt::If {
    cond: RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("n".to_string())),
        method: "is_u64".to_string(),
        args: vec![],
    },
    then_body: vec![RustStmt::Return(Some(err_expr(
        json_decode_error(
            string_expr("json integer out of range for sifr int"),
            // ...
        ),
    )))],
    // ...
},
```

**Impact**: JSON documents with large unsigned integers (> 9223372036854775807) will fail to parse even though they could be represented as floats.

**Note**: This is classified as "adapted" in the traceability doc. If this is intentional, it should remain. If not, consider converting u64 to float when it exceeds i64 range.

---

## Missing CPython Test Ports

The traceability document identifies the following CPython test families that should be harvested:

| Module | CPython Test Family | Status |
|--------|-------------------|--------|
| json | `test_json/test_fail.py`, `test_json/test_recursion.py`, `test_json/test_unicode.py`, `test_json/test_encode_basestring_ascii.py` | Covered via `phase_psp_c1_structured_parsing_serialization.sifr` |
| tomllib | `test_tomllib/test_data.py`, `test_tomllib/test_error.py`, `test_tomllib/test_misc.py` | Partially covered via `cpython_tomllib_subset.sifr` |
| csv | `test_csv.py` | Covered via `cpython_csv_subset.sifr` |
| configparser | `test_configparser.py` | Covered via `stdlib_configparser.sifr` |

**Coverage assessment**: The wave has good test coverage through the consolidated test file and existing subset tests. The test file `phase_psp_c1_structured_parsing_serialization.sifr` provides comprehensive coverage of:
- JSON: load, dumps, JsonValue construction, type checking, array/object access, error handling
- TOML: loads, nested table access, type checking
- CSV: Dialect, quoting, DictReader, DictWriter
- ConfigParser: defaults, strict mode, converters, mutation, error handling

**Gaps identified**: None critical - the adapted surface is well-tested.

---

## Ownership and Borrow Safety

**Assessment**: No ownership or borrow safety issues identified.

The codegen properly handles:
- **JSON intrinsics** (`crates/sifr_codegen/src/intrinsics/json.rs`): Uses references for read-only access when converting between Sifr's JsonValue and serde_json::Value. Clone operations are used appropriately.
- **JSON module** (`lib/sifr/json.sifr`): Uses value semantics throughout. The `own` annotations on internal helper functions (`_append_array_item`, `_append_object_item`) are correctly used.
- **CSV module** (`lib/sifr/csv.sifr`): Proper string cloning with `value + ""` pattern throughout.
- **ConfigParser** (`lib/sifr/configparser.sifr`): Proper copying of dictionaries with helper functions like `_copy_values`, `_copy_optional_str`.

---

## Production Risks

### High Priority

1. **ConfigParser.read() not functional** - This is a significant correctness bug that will cause production issues if users rely on this method.

### Medium Priority

1. **JSON silent serialization failures** - The "null" on error behavior could cause subtle bugs in production. Consider surfacing errors or documenting the adapted behavior clearly.

2. **No pretty-printing support** - CPython's `json.dumps()` supports `indent`, `separators`, `sort_keys` parameters. These are not exposed. The traceability doc classifies this as "unsupported" but it could be a production gap for users expecting formatted JSON output.

### Low Priority

1. **Limited CSV dialect registry** - CPython has a global dialect registry. Sifr uses direct `Dialect(...)` construction. This is classified as "unsupported" but is an adaptation that users should be aware of.

2. **ConfigParser interpolation not supported** - CPython's ConfigParser supports `${variable}` interpolation. This is classified as "unsupported" per the traceability doc.

---

## Regression Assessment

No regressions identified. All existing tests continue to pass. The implementation extends the surface without breaking existing functionality.

---

## Recommendations

1. **Fix ConfigParser.read()** - High priority bug fix required
2. **Document or fix JSON silent failures** - Production risk should be addressed
3. **Consider adding JSON pretty-print support** - Additional surface that may be needed
4. **Update traceability doc** - Ensure all adapted/unsupported classifications are accurate

---

## Files Modified

- `lib/sifr/json.sifr` - JsonValue class and JSON functions
- `lib/sifr/tomllib.sifr` - TomlValue class and TOML functions
- `lib/sifr/csv.sifr` - CSV reader/writer/Dialect implementation
- `lib/sifr/configparser.sifr` - ConfigParser implementation
- `crates/sifr_codegen/src/intrinsics/json.rs` - JSON intrinsics
- `crates/sifr_codegen/src/intrinsics/toml.rs` - TOML intrinsics
- `verification/stdlib/wave_psp_c1_cpython_traceability.md` - Traceability doc

---

## Conclusion

The wave_psp_c1 implementation provides solid structured parsing and serialization support with comprehensive test coverage. The main issues are:

1. A bug in `ConfigParser.read()` that needs fixing
2. Silent error handling in JSON serialization that poses a production risk

These should be addressed before considering the wave complete.