# wave_psp_c1 Review - Gap Analysis: CPython Parity

**Review Date:** 2026-03-17
**Reviewer:** agent
**Phase:** Structured Parsing and Serialization (JSON, TOML, CSV, ConfigParser)

---

## Executive Summary

wave_psp_c1 provides structured parsing and serialization for JSON, TOML, CSV, and ConfigParser. The implementation is **largely functional** with core features working, but there are **gaps between traceability claims and shipped behavior** that require documentation updates or fixes.

**Key Findings:**
- ✅ Core functionality works (all tests pass)
- ⚠️ One correctness bug still exists (JSON silent serialization failures)
- ⚠️ Traceability claims are overstated relative to actual CPython test coverage
- ✅ ConfigParser.read() bug from review pass 1 is FIXED

---

## Verified: Items Fixed Since Review Pass 1

| Issue | Status | Notes |
|-------|--------|-------|
| ConfigParser.read() not parsing | **FIXED** | Now properly calls `read_string()` on loaded content (lines 217-227) |

---

## Remaining Correctness Issues

### 1. JSON Serialization Silently Converts Errors to "null" (Production Risk - MEDIUM)

**Location:** `crates/sifr_codegen/src/intrinsics/json.rs` lines 746-754

**Issue:** The `json_dumps_value` intrinsic uses `unwrap_or_else` with a closure that returns the string `"null"` on serialization failure, rather than propagating the error.

```rust
method: "unwrap_or_else".to_string(),
args: vec![RustExpr::Closure {
    // ...
    body: Box::new(string_expr("null")),  // Returns "null" on error
    // ...
}],
```

**Impact:**
- Serialization errors (e.g., cyclic references, invalid UTF-8) silently produce `"null"` output
- Users cannot distinguish between actual null values and serialization failures
- Differs from CPython's behavior which raises `JSONEncodeError`

**Current State:** The Sifr JSON module exports `dumps(value: JsonValue) -> str` (not `Result`), so this is a design decision that needs documentation.

**Recommendation:** Document this as intentional adapted behavior in the traceability ledger, or change `dumps` to return `Result[str, JSONEncodeError]`.

---

### 2. TOMLDecodeError Line/Column Always Zero (Minor Deviation)

**Location:** `crates/sifr_codegen/src/intrinsics/toml.rs` lines 34-43

**Issue:** The `TOMLDecodeError` always passes 0 for line and column:

```rust
fn toml_decode_error(message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "TOMLDecodeError".to_string(),
        fields: vec![
            ("message".to_string(), message),
            ("line".to_string(), RustExpr::Literal(RustLiteral::Int(0))),
            ("column".to_string(), RustExpr::Literal(RustLiteral::Int(0))),
        ],
    }
}
```

**Impact:** Error messages don't include position information, making debugging harder.

**Recommendation:** Classify as documented adaptation or extract position info from toml crate's error type in a future wave.

---

## Traceability Gap Analysis

### Traceability Claims vs. Shipped Test Coverage

The traceability document (`verification/stdlib/wave_psp_c1_cpython_traceability.md`) claims coverage from these CPython test families:

| Module | Claimed CPython Coverage | Actual Test Coverage |
|--------|--------------------------|----------------------|
| json | `test_json/test_fail.py`, `test_json/test_recursion.py`, `test_json/test_unicode.py`, `test_json/test_encode_basestring_ascii.py` | **Basic subset only** - Tests parsing, basic dumps, and simple error cases. No recursion, unicode escape sequences, or encode_basestring_ascii coverage. |
| tomllib | `test_tomllib/test_data.py`, `test_tomllib/test_error.py`, `test_tomllib/test_misc.py` | **Basic subset only** - Tests basic types, nested tables, simple error. No inline tables, array of tables, or datetime variant coverage. |
| csv | `test_csv.py` | **Moderate coverage** - Dialect, reader, writer, DictReader, DictWriter covered. |
| configparser | `test_configparser.py` | **Good coverage** - Defaults, strict mode, converters, mutation, read_file. |

### Gap Severity: MODERATE

The test files are functional but do not comprehensively exercise the CPython test families cited in the traceability document. This is an accuracy issue rather than a correctness issue—the tests that exist work, but they don't provide the depth of coverage suggested by the traceability claims.

**Specific gaps:**
- JSON: No recursion depth tests, no unicode escape sequence tests, no scientific notation tests
- TOML: No inline tables (`{key = "value"}`), no array of tables (`[[table]]`), no datetime variant tests (local, offset, naive)
- CSV: Dialect registry functions not covered (waived)
- ConfigParser: `read_file()`, `read_dict()` not tested (though `read()` is now functional)

---

## Verified Functional Test Results

All tests pass:

```bash
$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_c1_structured_parsing_serialization.sifr
(pass)

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_json_subset.sifr
(pass)

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_tomllib_subset.sifr
(pass)

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_csv_subset.sifr
(pass)

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_configparser_subset.sifr
(pass)

$ cargo run -q -p sifr -- run demos/wave_psp_c1_structured_parsing_serialization_demo.sifr
(pass)
```

---

## Classification of Waivers

The traceability document correctly classifies these as unsupported:

| Surface | Classification | Status |
|---------|---------------|--------|
| JSON encoder/decoder hooks (`default=`, `object_hook`, etc.) | unsupported | ✅ Correct |
| TOML `parse_float=` customization | unsupported | ✅ Correct |
| CSV lazy streaming and dialect registry | unsupported | ✅ Correct |
| ConfigParser interpolation and converter registration | unsupported | ✅ Correct |

---

## Recommendations

### Required Actions

1. **Update traceability document** to accurately reflect actual test coverage rather than claimed CPython test family coverage. The current claims are overstated.

2. **Document JSON silent failure behavior** as intentional adapted behavior, or change `dumps()` to return `Result`.

### Optional Improvements

1. Extract TOML error position from toml crate's error type in future wave
2. Add more comprehensive edge case tests (recursion depth, unicode, scientific notation, etc.)

---

## Conclusion

wave_psp_c1 is **functionally complete** for its stated scope with working implementations of JSON, TOML, CSV, and ConfigParser. The main issues are:

1. **Traceability accuracy**: Claims about CPython test family coverage are overstated
2. **Production risk**: JSON silent error-to-null conversion needs documentation
3. **Minor deviation**: TOML error position not captured

The ConfigParser.read() bug from review pass 1 has been fixed. The implementation is usable but the traceability document should be updated to reflect actual shipped coverage.
