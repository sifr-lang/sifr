# wave_psp_e2 Review: CPython Parity Quality (R2)

**Date:** 2026-03-17
**Reviewer:** agent (agent)
**Status:** SATISFIED - no actionable gaps

---

## Executive Summary

wave_psp_e2 targets "class-heavy and custom cleanup" surfaces covering 5 CPython test families: `argparse`, `ipaddress`, `uuid`, `graphlib`, and `test` (unittest-style assertions).

All implementations in the current mainline state align with the traceability contract. No actionable CPython parity gaps identified.

---

## 1. Implementation Verification

### 1.1 argparse — SATISFIED

| Feature | Contract | Implementation | Status |
|---------|----------|----------------|--------|
| Inline option values (`--option=value`) | adapt | `_split_inline_option()` at lib/sifr/argparse.sifr:12-29 | ✅ |
| End-of-options (`--`) | adapt | `force_positional` logic at lib/sifr/argparse.sifr:54-62, 202-222 | ✅ |
| Missing-option fallback | adapt | `_is_option_like_token()` at lib/sifr/argparse.sifr:141-156 | ✅ |

**Test Coverage:** 19 assertions in `cpython_argparse_subset.sifr`
- `collect_argument_parser_actual()`: 7 assertions
- `collect_option_token_shape_actual()`: 5 assertions (inline `--foo=bar`, double-dash `--`)
- `collect_missing_option_value_actual()`: 7 assertions (fallback + helper functions)

**Demo Verification:** `demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr` runs successfully:
```
argparse.strict = true
argparse.mode = parity
argparse.entry = main.sifr
argparse.inline = inline
argparse.literal = --literal.sifr
argparse.missing_mode = safe
argparse.missing_strict = true
```

---

### 1.2 ipaddress — SATISFIED

| Feature | Contract | Implementation | Status |
|---------|----------|----------------|--------|
| Leading-zero rejection | adapt | `is_valid_ipv4()` at lib/sifr/ipaddress.sifr:19-22 | ✅ |
| `is_link_local()` | adapt | lib/sifr/ipaddress.sifr:170-174 | ✅ |
| `is_reserved()` | adapt | lib/sifr/ipaddress.sifr:177-181 | ✅ |
| 100.64/10 (CGN) handling | adapt | `is_global()` at lib/sifr/ipaddress.sifr:165-166 | ✅ |
| 192.0.0.9/.10 exceptions | adapt | `_is_private_ipv4_value()` at lib/sifr/ipaddress.sifr:108-112 | ✅ |
| Full private range | adapt | `_is_private_ipv4_value()` at lib/sifr/ipaddress.sifr:76-113 | ✅ |

**Test Coverage:** 20 assertions in `cpython_ipaddress_subset.sifr`
- Leading-zero rejection: `01.2.3.40` rejected
- 169.254.x.x: classified as link-local and not global
- 100.64.x.x: classified as not global
- 192.0.0.9: classified as global, 192.0.0.255 not global
- 240.x.x.x: classified as reserved

**Demo Verification:**
```
ipaddress.value = 8.8.8.8 global=true
ipaddress.link_local = true global=false
ipaddress.multicast_global = true
```

---

### 1.3 uuid — SATISFIED

| Feature | Contract | Implementation | Status |
|---------|----------|----------------|--------|
| URN format (`urn:uuid:...`) | adapt | `_canonical_uuid_text()` at lib/sifr/uuid.sifr:125-126 | ✅ |
| Curly braces (`{...}`) | adapt | `_canonical_uuid_text()` at lib/sifr/uuid.sifr:127-131 | ✅ |
| Helper functions | adapt | `_substring()` at lib/sifr/uuid.sifr:101-109, `_starts_with()` at lib/sifr/uuid.sifr:111-121 | ✅ |

**Test Coverage:** 14 assertions in `cpython_uuid_subset.sifr`
- Parsing URN format: `urn:uuid:550E8400-E29B-41D4-A716-446655440000`
- Parsing curly braces: `{550E8400-E29B-41D4-A716-446655440000}`
- Parsing plain hex and hyphenated formats

**Demo Verification:**
```
uuid.version = 4 text=3e880803-867b-412e-bddc-c948d3874974
uuid.curly.parse = 550e8400-e29b-41d4-a716-446655440000
```

---

### 1.4 graphlib — SATISFIED

| Feature | Contract | Implementation | Status |
|---------|----------|----------------|--------|
| Explicit node tracking | adapt | `nodes` list at lib/sifr/graphlib.sifr:54, 63, 71-75 | ✅ |
| Sparse node filtering | adapt | `_filter_order()` at lib/sifr/graphlib.sifr:96-101 | ✅ |
| Empty graph handling | adapt | `max_node = -1` initialization at lib/sifr/graphlib.sifr:66, check at 107-109, 158-159 | ✅ |
| `static_order()` filtering | adapt | Uses `_filter_order()` at lib/sifr/graphlib.sifr:162 | ✅ |

**Test Coverage:** 16 assertions in `cpython_graphlib_subset.sifr`
- Sparse node handling: `[10, 30, 40, 50]` with nodes 10, 30, 40, 50 (no leaks)
- Empty graph: returns `[]`

**Demo Verification:**
```
graphlib.order = [10, 30, 40, 50]
```

---

### 1.5 test (unittest assertions) — SATISFIED

| Feature | Contract | Implementation | Status |
|---------|----------|----------------|--------|
| Assertion helpers | adapt | lib/sifr/test.sifr:3-84 | ✅ |

**Test Coverage:** `cpython_unittest_assertions_subset.sifr` - verified

---

## 2. Traceability Contract Alignment

| Module | Contract | Implementation | Gap |
|--------|----------|----------------|-----|
| argparse | "inline option values, end-of-options positional mode, and missing-option fallback" | All three implemented | None |
| ipaddress | "IPv4 classification was aligned with CPython special-range behavior... 100.64/10 and 192.0.0.9/.10 exceptions" | All special ranges handled | None |
| graphlib | "explicit added nodes and no longer leaks undeclared intermediary nodes" | Filtering implemented | None |
| uuid | "supports... `urn:uuid:...` and `{...}` forms" | Both formats supported | None |

---

## 3. Test Coverage Summary

| Test File | Assertions | Status |
|-----------|------------|--------|
| `cpython_argparse_subset.sifr` | 19 | ✅ Pass |
| `cpython_ipaddress_subset.sifr` | 20 | ✅ Pass |
| `cpython_uuid_subset.sifr` | 14 | ✅ Pass |
| `cpython_graphlib_subset.sifr` | 16 | ✅ Pass |
| `cpython_unittest_assertions_subset.sifr` | N | ✅ Pass |
| `phase_psp_e2_class_heavy_custom_cleanup.sifr` | N | ✅ Pass |
| `demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr` | N | ✅ Pass |

---

## 4. Fail Test Verification

| Test File | Expected Behavior | Result |
|-----------|-------------------|--------|
| `phase_psp_e2_argparse_parse_args_non_string_list.sifr` | Type error | ✅ `type error: argument 1 ('args')... expected 'list[str]', got 'list[int]'` |
| `phase_psp_e2_ip_address_non_string.sifr` | Type error | ✅ `type error: argument 1 ('addr')... expected 'str', got 'int'` |

---

## 5. Conclusion

**Status:** SATISFIED: no actionable gaps

All wave_psp_e2 implementations in the current mainline state are:
1. Aligned with the CPython traceability contract
2. Covered by test assertions
3. Verified via demo execution
4. Correctly handle edge cases (empty graphs, sparse nodes, format variations)

The implementation is ready for any future wave merge.
