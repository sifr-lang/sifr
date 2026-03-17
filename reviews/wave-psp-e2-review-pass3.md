# wave_psp_e2 Review - Pass 3

**Reviewer:** External Reviewer
**Scope:** Class-Heavy and Custom Cleanup modules (`argparse`, `ipaddress`, `uuid`, `graphlib`)
**Status:** Approved with no actionable implementation issues

---

## Executive Summary

The wave_psp_e2 implementation successfully closes parity gaps for four class-heavy Python stdlib modules:
- `argparse`: token-shape support (`--name=value`, `--` end-of-options)
- `ipaddress`: leading-zero rejection, input validation hardening
- `uuid`: URN/curly-brace form normalization
- `graphlib`: sparse-node ordering fix

All tests pass and the implementation follows the documented parity contract in the traceability matrix.

---

## Implementation Review

### 1. argparse Token-Shape Support

**File:** `lib/sifr/argparse.sifr`

**Changes Implemented:**
- Added `_split_inline_option()` helper function (lines 12-29) to parse `--name=value` format
- Added `--` (double-dash) handling in both helper functions and `ArgumentParser.parse_args()` (line 188-190, 58-59)
- Updated `parse_option()` to check for inline values (lines 41-43)
- Updated `parse_positional()` to handle `--` end-of-options marker (lines 51-59)

**Verification:**
```python
# From cpython_argparse_subset.sifr test
parsed_inline = parser.parse_args(["--foo=bar", "value.txt"])
# Correctly parses inline option value
parsed_double_dash = parser.parse_args(["--", "--literal.txt"])
# Correctly treats --literal.txt as positional after --
```

**Risk Assessment:** Low. Implementation correctly handles token shapes per CPython argparse behavior. Tests verify both inline (`--foo=bar`) and end-of-options (`--`) modes.

---

### 2. ipaddress Leading-Zero Rejection

**File:** `lib/sifr/ipaddress.sifr`

**Changes Implemented:**
- Added leading-zero rejection in `is_valid_ipv4()` at lines 19-22:
```python
if len(part) > 1:
    first_digit: str | None = part[0]
    if first_digit is not None and first_digit == "0":
        return False
```
- Added `AddressValueError` custom error class (lines 4-8)
- Added input validation to all classification functions (`is_private`, `is_loopback`, `is_multicast`, `is_global`)
- Added range validation to `int_to_ip()` (lines 103-105)
- Added `IPv4Address` class with typed factory functions `ip_address()` and `ipv4_address()`

**Verification:**
```python
# From cpython_ipaddress_subset.sifr test
is_valid_ipv4("01.2.3.40") == False  # Leading zero rejected
leading_zero_rejected: bool = False
try:
    _leading_zero: IPv4Address = ip_address("1.02.3.40")
except AddressValueError:
    leading_zero_rejected = True
# Correctly raises AddressValueError
```

**Risk Assessment:** Low. The implementation correctly rejects leading-zero octets per CPython behavior. The sentinel value approach for direct constructor (`packed_int() == -1`) is documented as intentional diff.

---

### 3. uuid Parser Normalization (URN/Curly Forms)

**File:** `lib/sifr/uuid.sifr`

**Changes Implemented:**
- Added `_starts_with()` helper function (lines 111-121)
- Added `_substring()` helper function (lines 101-109)
- Updated `_canonical_uuid_text()` to handle:
  - `urn:uuid:...` prefix (lines 125-126)
  - `{...}` curly brace forms (lines 127-131)
- Properly normalizes all inputs to canonical hyphenated lowercase form

**Verification:**
```python
# From cpython_uuid_subset.sifr test
parsed_urn = uuid_from_hex("urn:uuid:550E8400-E29B-41D4-A716-446655440000")
# Correctly normalizes to "550e8400-e29b-41d4-a716-446655440000"

parsed_curly = uuid_from_hex("{550E8400-E29B-41D4-A716-446655440000}")
# Correctly normalizes to "550e8400-e29b-41d4-a716-446655440000"
```

**Risk Assessment:** Low. Implementation correctly handles all documented UUID input formats (plain hex, hyphenated, URN, curly braces). The raw constructor passthrough is documented as intentional diff.

---

### 4. graphlib Sparse-Node Ordering Fix

**File:** `lib/sifr/graphlib.sifr`

**Changes Implemented:**
- Added `nodes: list[int]` field to track explicitly-added nodes (line 54)
- Added `_record_node()` method to track nodes (lines 71-75)
- Added `_filter_order()` method to filter results to only explicitly-added nodes (lines 96-101)
- Updated `add()` and `add_many()` to track nodes
- Fixed `max_node` initialization to `-1` instead of `0` (line 66)
- Updated `static_order()` to use filtered order (lines 157-164)

**Key Fix:** Previously, sparse graphs like `add_many(50, [30, 40])` would leak intermediate nodes (0-49) that were never explicitly added. Now only explicitly-added nodes are returned.

**Verification:**
```python
# From cpython_graphlib_subset.sifr test
sorter.add_many(50, [30, 40])
sorter.add(30, 10)
sorter.add(40, 10)
sorter.add_many(10, [])
order = sorter.static_order()
# Correctly returns [10, 30, 40, 50] - NOT [0, 1, ..., 50]
```

**Risk Assessment:** Low. The implementation correctly filters to only explicitly-added nodes. The one-node-at-a-time incremental flow is documented as intentional diff from CPython's multi-node frontier.

---

## Test Coverage Review

### Positive Coverage

- **argparse**: 11 test cases covering parser construction, option parsing, positional binding, token shapes, double-dash mode
- **ipaddress**: 9 test cases covering validation, leading-zero rejection, classification helpers, error types
- **uuid**: 14 test cases covering generation, parsing (4 formats), validation, class behavior
- **graphlib**: 16 test cases covering static order, incremental flow, cycle detection, sparse-node filtering

### Fail Tests

- `phase_psp_e2_argparse_parse_args_non_string_list.sifr`: Correctly rejects non-string args list
- `phase_psp_e2_ip_address_non_string.sifr`: Correctly rejects non-string address
- `phase_psp_e2_uuid_from_hex_non_string.sifr`: Correctly rejects non-string hex
- `phase_psp_e2_graphlib_add_non_int_predecessor.sifr`: Correctly rejects non-int predecessor

---

## Validation Evidence

### Demo Validation
```
cargo run -q -p sifr -- run demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr
# Output:
# argparse.strict = true
# argparse.mode = parity
# argparse.entry = main.sifr
# argparse.inline = inline
# argparse.literal = --literal.sifr
# ipaddress.value = 8.8.8.8 global=true
# uuid.version = 4 text=dc42e593-ff40-4ab5-a473-efea50724a55
# uuid.curly.parse = 550e8400-e29b-41d4-a716-446655440000
# graphlib.order = [10, 30, 40, 50]
```

### Test Validation
All pass:
- `phase_psp_e2_class_heavy_custom_cleanup.sifr`
- `cpython_argparse_subset.sifr`
- `cpython_ipaddress_subset.sifr`
- `cpython_uuid_subset.sifr`
- `cpython_graphlib_subset.sifr`

### Fail Test Validation
All correctly reject invalid input:
- `phase_psp_e2_argparse_parse_args_non_string_list.sifr`: Type error on non-string list
- `phase_psp_e2_ip_address_non_string.sifr`: Type error on non-string address
- `phase_psp_e2_uuid_from_hex_non_string.sifr`: Type error on non-string hex
- `phase_psp_e2_graphlib_add_non_int_predecessor.sifr`: Type error on non-int predecessor

---

## Traceability Matrix Review

The `wave_psp_e2_cpython_traceability.md` correctly documents:
- All four CPython test families reviewed
- Adapted surfaces with clear scope boundaries
- Explicit waivers for advanced features (argparse subparsers, ipaddress IPv6, uuid non-v4, graphlib multi-node frontier)
- Intentional diffs documented (direct constructor behavior, raw UUID construction)

---

## Risk Assessment

**Overall Risk:** Low

All four components implement the documented parity contract correctly. The implementation:
1. Passes all CPython-derived tests
2. Correctly rejects invalid inputs via type system and runtime checks
3. Documents all intentional deviations from CPython behavior
4. Follows the wave's adapted classification approach

---

## Conclusion

**Status:** Approved as production-ready with no actionable implementation issues.

The wave_psp_e2 implementation successfully closes class-heavy stdlib parity gaps for argparse, ipaddress, uuid, and graphlib. All tests pass, fail tests correctly reject invalid inputs, and the traceability matrix accurately documents the parity contract.
