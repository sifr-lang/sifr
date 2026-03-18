# wave_psp_c1 Review - Gap Analysis and CPython Parity (2026-03-17 R2)

**Phase**: Structured Parsing and Serialization (PSP)
**Reviewer**: Claude Code
**Date**: 2026-03-17

## Executive Summary

The wave_psp_c1 implementation covers JSON, TOML, CSV, and ConfigParser modules. Core functionality works correctly, and most tests pass. This review identifies remaining gaps between the implementation and the stated traceability claims, focusing on actionable findings.

**Overall Status**: Functionally complete with minor correctness bugs and design adaptations.

---

## Verified: Fixed Issues

### 1. ConfigParser.read() — FIXED ✅

**Location**: `lib/sifr/configparser.sifr:217-227`

The bug identified in Pass 1 has been remediated. The method now properly:
- Reads file content via `read_text(path)`
- Calls `self.read_string(content)` to parse the INI content
- Converts `ParsingError` to `IOError` with appropriate message

```sifr
def read(self, path: str) -> Result[list[str], IOError]:
    try:
        content: str = read_text(path)
        try:
            _: None = self.read_string(content)
        except ParsingError as e:
            raise IOError("parse error on line " + str(e.line) + ": " + e.message)
        loaded_path: str = path + ""
        return [loaded_path]
    except IOError as e:
        raise e
```

---

## Remaining Correctness Issues

### 1. ConfigParser.has_option() Logic Bug — UNFIXED 🔴

**Location**: `lib/sifr/configparser.sifr:265-276`

**Severity**: Medium

**Issue**: The logic incorrectly falls back to checking defaults when the section exists but the option is not found in it. This can cause false positives.

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
        return normalized in self._defaults  # BUG: Returns True if option in defaults
    return False
```

**Expected behavior**:
- If section exists and contains option → return True
- If section exists but doesn't contain option → return False (NOT check defaults)
- If section doesn't exist → return False

**Current behavior**: When section exists but option is not in it, incorrectly checks defaults.

**Traceability claim**: The traceability document claims `configparser` surface is "adapted" but doesn't document this specific behavioral deviation.

---

### 2. JSON Serialization Silent Error Conversion — UNFIXED 🟡

**Location**: `crates/sifr_codegen/src/intrinsics/json.rs:746-754`

**Severity**: Low (by design adaptation)

**Issue**: Serialization errors produce `"null"` string instead of propagating an error.

```rust
method: "unwrap_or_else".to_string(),
args: vec![RustExpr::Closure {
    params: vec![RustParam::Named {
        name: "_err".to_string(),
        ty: RustType::Named("_".to_string()),
    }],
    body: Box::new(string_expr("null")),  // Returns "null" on error
    is_move: false,
}],
```

**Impact**: Users cannot distinguish between actual `null` values and serialization failures.

**Traceability status**: This is an "adapted" behavior. The `dumps` function returns `str` rather than `Result[str, JSONEncodeError]`. The traceability document classifies encoding hooks as "unsupported" but this specific silent error conversion is not explicitly documented.

---

### 3. CSV DictWriter.writeheader Redundant Assignment — UNFIXED 🟢

**Location**: `lib/sifr/csv.sifr:691-694`

**Severity**: Low (code quality)

```sifr
def writeheader(self) -> None:
    current_writer: writer = self._writer
    current_writer.writerow(self.fieldnames)
    self._writer = current_writer  # Redundant - same value
```

**Impact**: No functional impact, indicates potential confusion about mutation semantics.

---

### 4. TOMLDecodeError Position Always Zero — DOCUMENTED 🟢

**Location**: `crates/sifr_codegen/src/intrinsics/toml.rs:34-43`

**Severity**: Low (by design adaptation)

The error always reports line=0 and column=0:

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

**Status**: This is documented in Pass 2 review as acceptable adapted behavior per the traceability classification (callback-based TOML customization is out of scope).

---

## Traceability Validation

| Module | Traceability Claim | Actual Status | Gap |
|--------|-------------------|---------------|-----|
| json | `loads`, `load`, `dumps`, `dump`, JsonValue tree | Implemented | Silent error conversion not explicitly documented |
| tomllib | `loads`, TomlValue tree | Implemented | Error position always 0 (documented adaptation) |
| csv | Dialect, reader, writer, DictReader, DictWriter | Implemented | None |
| configparser | ConfigParser, RawConfigParser, DEFAULTSECT | Implemented | **has_option() behavior differs from CPython** |

---

## Test Coverage Assessment

All E2E tests pass:

```
$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_c1_structured_parsing_serialization.sifr
(passed)

$ cargo run -q -p sifr -- run demos/wave_psp_c1_structured_parsing_serialization_demo.sifr
(passed)

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_configparser_subset.sifr
(passed)

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_csv_consolidated.sifr
(passed)
```

**Note**: The existing tests do not exercise the `has_option()` bug scenario (checking an option that exists in defaults but not in a specific section).

---

## Actionable Findings

### Must Fix (Before Production)

1. **ConfigParser.has_option() logic bug** — This can cause incorrect behavior in production when:
   - A non-default section exists
   - The option is NOT in that section
   - But the option IS in DEFAULTSECT
   - Returns `True` (incorrectly) instead of `False`

### Should Document (Traceability)

2. **JSON silent error conversion** — Update traceability document to explicitly note that serialization errors produce `"null"` string rather than raising an exception.

### Nice to Have (Code Quality)

3. **CSV writeheader redundant assignment** — Remove `self._writer = current_writer` line.

---

## Recommendation

**Status**: APPROVED with one bug requiring remediation

The implementation is production-ready for the core use cases. The `ConfigParser.has_option()` bug should be fixed before considering the wave complete, as it can cause incorrect boolean results in common configuration patterns.

**Remediation required**:
1. Fix `has_option()` logic at `lib/sifr/configparser.sifr:265-276`

**Documentation updates recommended**:
1. Add JSON silent error conversion to traceability matrix as explicitly documented adaptation
