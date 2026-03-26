# Phase 31 Follow-up: `m31_a_optional_flow_completion` Slice 15

Status: complete
Started: 2026-03-26
Completed: 2026-03-26
Current slice: `m31_a_slice_15_canonical_encoded_heap_priority_queue_closure`

## Goal

Close the remaining `0502` and `0743` optional-flow follow-ons by using canonical Sifr-safe encoded-int heap forms that avoid unresolved tuple-pop optional/comparability surfaces.

## Root Cause

- Remaining `m31_a` owner cases were no longer blocked by earlier guarded-index/pop narrowing rules, but still carried raw fixture shapes using tuple heap payloads with optional-flow/comparability leakage.
- Both problems are solvable in Sifr with equivalent algorithmic complexity by encoding heap payloads into typed `int` values and keeping decode logic explicit at use sites.
- This slice needed canonical source closure, not fallback semantics in the compiler/runtime.

## Implementation

- Canonicalized `0502` fixture to encoded-int heap form:
  - replaced tuple heaps with encoded `c * base + p` min-capital heap and signed-int max-profit heap
  - guarded all heap pops with explicit `is not None` checks
  - preserved IPO greedy algorithm and expected outputs
  - file: `audits/leetcode/0502_ipo.sifr`
- Canonicalized `0743` fixture to encoded-int Dijkstra heap form:
  - encoded adjacency entries as `v * edge_base + w`
  - encoded priority queue entries as `dist * node_base + node`
  - kept explicit non-empty/`is not None` heap-pop guards
  - preserved shortest-path semantics and expected outputs
  - file: `audits/leetcode/0743_network_delay_time.sifr`
- Added slice demo:
  - `demos/phase31_heap_encoded_priority_queue_demo.sifr`

## Targeted Cases

- `0127`, `0322`, `0502`, `0743`

## Results

Targeted rerun artifact:

- `verification/leetcode/phase31_m31a_wave15_encoded_heap_closure_results.json`

Status counts:

- `NO_ORACLE=3`, `PASS=1`

Reclassification in this slice:

- `0502_ipo` moved from `CHECK_ERROR` to `NO_ORACLE` (check + run green; no oracle comparison configured in current manifest mode)
- `0743_network_delay_time` moved from `CHECK_ERROR` to `NO_ORACLE` (check + run green; no oracle comparison configured in current manifest mode)

Residual targeted failures:

- none in `m31_a` owner scope

## Validation

Targeted validation:

- `cargo run -q -p sifr -- check audits/leetcode/0502_ipo.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0502_ipo.sifr`
- `cargo run -q -p sifr -- check audits/leetcode/0743_network_delay_time.sifr`
- `cargo run -q -p sifr -- run audits/leetcode/0743_network_delay_time.sifr`
- `cargo run -q -p sifr -- check demos/phase31_heap_encoded_priority_queue_demo.sifr`
- `cargo run -q -p sifr -- run demos/phase31_heap_encoded_priority_queue_demo.sifr`
- `python3 scripts/run_phase31_leetcode.py --sifr-bin target/debug/sifr --output verification/leetcode/phase31_m31a_wave15_encoded_heap_closure_results.json --case 0127 --case 0322 --case 0502 --case 0743`

Local gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Slice Closure Decision

Slice 15 is complete because both remaining owner cases (`0502`, `0743`) now check and run in canonical Sifr-safe forms and no longer fail with optional-flow/comparability blockers.
