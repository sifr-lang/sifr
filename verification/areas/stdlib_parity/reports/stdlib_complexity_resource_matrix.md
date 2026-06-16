# Phase 30 Complexity and Resource Matrix

This file is the canonical milestone_30_2 artifact for API-level complexity and resource parity.

Primary machine-readable source:
- `verification/areas/stdlib_parity/data/stdlib_complexity_resource_inventory.json`

Validator:
- `python3 verification/areas/stdlib_parity/tools/check_complexity_resource_inventory.py`

## Canonical Check Patterns

| pattern | input sweep | acceptance rule |
| --- | --- | --- |
| `o1_wrapper` | `1, 8, 64, 512` | adjacent growth ratio `<= 1.25x` |
| `linear_scan` | `64, 256, 1024, 4096` | adjacent growth ratio `<= 2.8x` |
| `ordered_insert_search` | `128, 512, 2048, 8192` | search paths align with `O(log n)`, insertion paths align with `O(n)` |
| `host_io_bound` | `1, 10, 100` | panic-free behavior and bounded temporary allocation; constant-factor deltas may be waived |

## Module API-Class Coverage

| api_class | modules |
| --- | --- |
| `o1_wrapper` | `env`, `math`, `datetime`, `time`, `platform`, `uuid` |
| `linear_scan` | `bytes`, `base64`, `hashlib`, `statistics`, `string`, `textwrap`, `fnmatch`, `re`, `collections`, `itertools`, `json`, `csv`, `timeit` |
| `ordered_insert_search` | `bisect`, `heapq` |
| `host_io_bound` | `io`, `os`, `pathlib`, `glob`, `tempfile`, `shutil`, `logging` |

All 28 Phase 30 modules are covered in the inventory and mapped to a complexity/resource check pattern.

## Asymptotic Comparison Status

Inventory status summary (from `stdlib_complexity_resource_inventory.json`):
- `28/28` modules have explicit expected and observed asymptotic classifications.
- All entries are aligned with approved Phase 30 API scope.
- No unresolved asymptotic mismatch remains open.

## Constant-Factor Delta and Waiver Inventory

Accepted constant-factor waivers (host-variant behavior with owner/rationale/revisit rule in inventory):
- `re`
- `io`
- `os`
- `pathlib`
- `glob`
- `tempfile`
- `shutil`
- `logging`
- `time`
- `timeit`
- `platform`

Non-waived modules use explicit delta bands (`within_2x`, `within_5x`, or `within_10x`) in the inventory.

## Governance Contract

- Every inventory entry has:
  - module identifier
  - API class
  - representative APIs
  - expected/observed asymptotic behavior
  - explicit constant-factor delta band
  - resource budget note
  - waiver metadata when delta is waived
- Waivers are invalid unless all of these fields are present:
  - `owner`
  - `rationale`
  - `tracking_issue`
  - `revisit_rule`
