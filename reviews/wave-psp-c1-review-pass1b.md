# wave_psp_c1 Structured Parsing and Serialization Parity Review

**Review Date:** 2026-03-16
**Reviewer:** Claude Code
**Status:** Implementation Review - Pass 1b

## Executive Summary

The `wave_psp_c1` implementation provides structured parsing and serialization parity for JSON, TOML, CSV, and ConfigParser modules. The implementation follows a hybrid architecture where:
- **HIR layer** (`sifr_hir/src/stdlib/`) defines type signatures for intrinsics
- **Sifr stdlib** (`lib/sifr/*.sifr`) provides the user-facing module surface
- **Codegen** (`sifr_codegen/src/intrinsics/`) lowers intrinsics to Rust

The demo and E2E tests pass successfully, indicating core functionality works. However, several gaps and potential issues were identified.

---

## Modules Covered

| Module | Files | Status |
|--------|-------|--------|
| JSON | `json.sifr`, `intrinsics/json.rs` | Implemented |
| TOML | `tomllib.sifr`, `intrinsics/toml.rs` | Implemented |
| CSV | `csv.sifr` | Implemented |
| ConfigParser | `configparser.sifr` | Implemented |

---

## Correctness Gaps

### 1. ConfigParser.read() Not Implemented (Medium Risk)

**Location:** `lib/sifr/configparser.sifr:217-223`

```sifr
def read(self, path: str) -> Result[list[str], IOError]:
    try:
        _: str = read_text(path)
        loaded_path: str = path + ""
        return [loaded_path]
    except IOError as e:
        raise e
```

**Issue:** The `read()` method only returns the path without actually parsing the file content. CPython's `ConfigParser.read()` parses the file and returns a list of successfully read filenames.

**Impact:** Code relying on `parser.read(["file.ini"])` will get incorrect behavior (returns empty success but doesn't actually parse).

**Recommendation:** Implement actual file reading and parsing in the `read()` method.

---

### 2. CSV writer/DictWriter Redundant Assignment (Low Risk)

**Location:** `lib/sifr/csv.sifr:691-694`

```sifr
def writeheader(self) -> None:
    current_writer: writer = self._writer
    current_writer.writerow(self.fieldnames)
    self._writer = current_writer  # Redundant - same value
```

**Issue:** The assignment `self._writer = current_writer` is redundant since `current_writer` is a reference to the same object.

**Impact:** No functional impact, but indicates potential confusion about ownership or mutation semantics.

---

### 3. ConfigParser.has_option() Logic Inconsistency (Medium Risk)

**Location:** `lib/sifr/configparser.sifr:261-272`

```sifr
def has_option(self, section: str, option: str) -> bool:
    normalized: str = _normalize_option(option)
    default_section: str = _default_section()
    if section == default_section:
        return normalized in self._defaults
    for section_name, section_values in self._sections.items():
        if section_name != section:
            continue
        if _has_option_key(section_values, normalized):
            return True
        return False  # Early return after first section check
    return normalized in self._defaults
```

**Issue:** The logic `return False` inside the loop exits after checking the first section that doesn't match. If the section exists but the option isn't found in it, the function returns `False` without checking other sections or falling back to defaults properly.

**Correct behavior should be:**
- If section exists and has option → return True
- If section exists but doesn't have option → check defaults
- If section doesn't exist → return False

---

## Missing CPython Test Ports

### JSON
- **Missing:** Tests for float handling with scientific notation (e.g., `1e10`, `1.5e-3`)
- **Missing:** Tests for Unicode escape sequences in strings
- **Missing:** Recursion depth limits (currently rejected but without proper error type)

### TOML
- **Missing:** Inline tables parsing (`{key = "value"}`)
- **Missing:** Array of tables (`[[table]]`)
- **Missing:** Datetime variants (local, offset, naive)

### CSV
- **Missing:** Dialect registry functions (`register_dialect`, `unregister_dialect`)
- **Missing:** `Sniffer` class for auto-detecting dialects
- **Waived per traceability:** Lazy iterator behavior

### ConfigParser
- **Missing:** `read_file()` method
- **Missing:** `read_dict()` method
- **Missing:** `readfp()` (deprecated alias)
- **Waived per traceability:** Interpolation, converter registration

---

## Ownership/Borrow Safety Analysis

### Positive Findings

1. **JSON serialization** (`lib/sifr/json.sifr:148-157`): Uses `own` annotations correctly for array/object building:
   ```sifr
   def _append_array_item(own mut value: JsonValue, own item: JsonValue) -> JsonValue:
       value.array_items.append(item)
       return value
   ```

2. **Explicit copying patterns**: Extensive use of string copying (`field + ""`) and dictionary copying functions to ensure ownership semantics.

3. **Codegen handles ownership correctly**:
   - `lower_json_dumps_value` uses proper reference passing (`&value`)
   - `lower_json_loads` correctly handles conversions between Sifr and serde types

### Potential Issues

1. **ConfigParser mutation without ownership tracking** (`lib/sifr/configparser.sifr:324-336`):
   ```sifr
   def set(self, section: str, option: str, value: str | None) -> None:
       # ... modifies self._sections in place
   ```
   This works but could be more explicit about mutation semantics.

2. **List iteration in for-loops**: The Sifr stdlib uses iteration patterns that may involve copying (e.g., `for item in self.array_items: result.append(item)`). While correct, this creates unnecessary copies.

---

## Production Risks

### High Priority

1. **ConfigParser.read() stub**: Must be fixed before production use with ConfigParser file reading.

### Medium Priority

2. **ConfigParser.has_option() bug**: Can cause false negatives when checking for options in specific sections.

3. **Large file handling**: CSV parsing loads entire file into memory. No streaming support.

### Low Priority

4. **Test coverage gaps**: While E2E tests cover core functionality, edge cases (large numbers, nested depth, special characters) need more coverage.

5. **Error message quality**: JSON/TOML decode errors use generic messages from underlying libraries. Could improve with context-specific errors.

---

## Test Execution Results

### Demo File
```
$ cargo run -q -p sifr -- run demos/wave_psp_c1_structured_parsing_serialization_demo.sifr

wave_psp_c1 structured parsing and serialization demo
sifr
true
{"name":"sifr","items":[1,true]}
sifr
true
"alpha","beta"
[{"name": "alice", "age": "30"}]
name,age
alice,30
8080
true
None
```

### E2E Test
```
$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_c1_structured_parsing_serialization.sifr
(pass - no output in quiet mode)
```

---

## Recommendations

### Immediate Fixes (Before Merge)

1. **Fix ConfigParser.read()** - Implement actual file parsing
2. **Fix ConfigParser.has_option()** - Correct loop logic

### Post-Merge Improvements

1. Add more CPython test ports for edge cases
2. Consider adding file-based ConfigParser.read() that calls read_string internally
3. Document the CSV eager materialization design decision in user-facing docs

### Notes

- The "unsupported" features documented in `wave_psp_c1_cpython_traceability.md` are correctly marked as out of scope for this wave
- The implementation correctly handles structured value trees (JsonValue, TomlValue) instead of dynamic Python objects
- Error handling uses typed Result/Option patterns as expected for Sifr

---

## Conclusion

The wave_psp_c1 implementation is functionally complete for the stated scope but has two correctness bugs (ConfigParser.read() stub and has_option() logic) that should be fixed before production use. The structured value approach is sound and ownership semantics are correctly handled. The demo and E2E tests pass, indicating core functionality works as designed.

**Recommendation:** Fix the ConfigParser bugs, then merge. The gaps in test coverage are acceptable for the initial release given the explicit scope boundaries in the traceability document.