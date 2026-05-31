# Review: M1c — Trie Parity

**Branch**: `leetcode-trie-parity-m1c`
**Auditor**: Review pass 1
**Date**: 2026-05-30

---

## Summary

**Approved.** This milestone correctly implements its intent. No blockers, no hidden risks. The 6 changed files across Sifr sources, Python fixture generators, fixture inputs, metadata registry, and slowness seed form a coherent classification wave.

---

## Review Findings

### 1. Semantic Parity of Problem-Local Trie Implementations ✅

Both Sifr implementations (0208 and 0211) use the same dict-backed structure:
- `edges: list[dict[str, int]]` (Sifr emitted as `Vec<HashMap<String, i64>>`)
- `end: list[bool]` (Sifr emitted as `Vec<bool>`)
- Shared `_child` helper function with identical logic

**0208 (`src/0208_implement_trie_prefix_tree.sifr` lines 3–13, 15–56)**
- `Trie` class with problem-local dict-backing, replacing the prior `helpers.trie.Trie` dependency
- `insert`, `search`, `startsWith` methods match the Python reference at `src/0208_implement_trie_prefix_tree.py`
- The `insert` method clones `edges` and `end` on every call (lines 24–25), which is the `field_clone` source of slowness — intentional codegen behavior, not a correctness issue

**0211 (`src/0211_design_add_and_search_words_data_structure.sifr` lines 3–68)**
- `WordDictionary` class mirrors 0208's structure with the addition of wildcard search
- `_search_from` helper (lines 23–41) handles `.` pattern matching by recursing into all children
- Semantically identical to the Python reference at `src/0211_design_add_and_search_words_data_structure.py`
- The `addWord` method also clones `edges` and `end` (lines 52–53), consistent with the slowness tagging

**Codegen verification** (via `cargo run -q -p sifr -- emit`):
- Both emit `struct Trie { edges: Vec<HashMap<String, i64>>, end: Vec<bool> }` and `struct WordDictionary { ... }`
- The arena-style dict backing is correctly represented in generated Rust

---

### 2. Object-Op Fixture Fix ✅

**Root cause confirmed**: The generic `object_ops` runner (harnesses/generic.py line 600: `for line_index in range(1, len(lines))`) starts processing at line index 1, skipping line 0. Without `__init__`, the first operation was never processed on the Sifr side. The Python oracle processed all operations, generating incorrect expected checksums.

**Fix applied correctly**:
- `0208_implement_trie_prefix_tree.py` line 15: `lines = ["__init__"] + ...`
- `0211_design_add_and_search_words_data_structure.py` line 10: `lines = ["__init__"] + ...`

Both fixture generators now emit `__init__` as the first line. The Python harness (line 326–328) correctly handles this:
```python
if lines and lines[0].split()[0] == "__init__":
    constructor_args = parse_object_args(lines[0].split()[1:], ...)
    start_line = 1
obj = constructor(*constructor_args)
```

Both sides process the same operation range. The `__init__` line is a no-op constructor call (no args), so Python and Sifr behavior match.

**Fixture line counts confirm correct operation count**:
| Problem | Size | Lines | Expected ops |
|---------|------|-------|--------------|
| 0208 | 1000 | 3001 | 3000 (1000 insert + 1000 search + 1000 startsWith) |
| 0208 | 5000 | 15001 | 15000 |
| 0208 | 10000 | 30001 | 30000 |
| 0211 | 1000 | 3001 | 3000 (1000 addWord + 1000 search + 1000 .search) |
| 0211 | 5000 | 15001 | 15000 |
| 0211 | 10000 | 30001 | 30000 |

Expected files (`ops=0001000.expected`): `2000 2001000` for both 0208 and 0211, confirming 2000 result-producing operations (1000 searches + 1000 startsWith/.search calls), matching the fixture generators' operation mix.

---

### 3. Mixed/Equivalent Classification ✅

**0208** (`tries.json` lines 57–65, `slowness_seed.py` lines 53, 57):
- `primary_slowness_owner`: `mixed`
- `parity_status`: `equivalent`
- `slowness_tags`: `["trie_parity", "field_clone", "dict_clone", "stateful_object"]`

**0211** (`tries.json` lines 112–120, `slowness_seed.py` line 53):
- `primary_slowness_owner`: `mixed`
- `parity_status`: `equivalent`
- `slowness_tags`: `["trie_parity", "field_clone", "dict_clone", "stateful_object"]`

**Classification is appropriate**:
- `mixed` owner reflects that slowness is split between compiler codegen (field cloning behavior) and residual code structure (dict-backed arena with HashMap overhead)
- `equivalent` parity confirms correctness — fixture validation passes, demos run, benchmarks complete
- Severe slowness (0208 worst 0.005x, 0211 worst 0.009x) is accurately reflected in the measured ratios
- The `trie_parity` tag correctly identifies this as a parity wave removing the shared helper dependency

---

### 4. Hidden Harness, Fixture, and Codegen Risks ✅

**No risks found.** Verified:

- **Harness** (`harnesses/generic.py`): `object_ops_sifr_runner_body` correctly generates code that starts at `line_index = 1` (line 600), processing lines after `__init__`. Empty lines are skipped (line 603–604). Method dispatch correctly handles `__init__` via the pre-loop `init_parts` parse (lines 593–595).

- **Fixture generators**: Both produce deterministic output with proper `fixture_stem` format matching the registry. Line counts scale linearly with size.

- **Codegen**: Generated Rust uses `Vec<HashMap<String, i64>>` for edges and `Vec<bool>` for end markers — semantically correct dict-backed trie representation. The `field_clone` behavior (cloning both fields on every mutating method) is expected Sifr codegen output, not a bug.

- **Regression risk**: Direct Sifr demo runs (`cargo run -q -p sifr -- run`) pass for both 0208 and 0211. Correctness benchmarks validated. No evidence of regression.

---

## Validation Checks

| Check | Result |
|-------|--------|
| `python3 benchmarks/analyze_slowness.py --check-metadata` | ✅ Pass (no diagnostics) |
| `python3 -m py_compile` on all touched Python files | ✅ Pass (no errors) |
| `git diff --check` | ✅ Pass (no whitespace errors) |
| First-party file line counts under 900-line guardrail | ✅ Pass (max: 93 lines for 0211) |
| Fixture `__init__` presence verified | ✅ All 6 fixture files have `__init__` as line 1 |
| Direct Sifr demos for 0208 and 0211 | ✅ Pass |

---

## Milestone Status

**Approved.** All review criteria satisfied:

1. ✅ Semantic parity: Problem-local dict-backed trie implementations match Python references and generated fixtures
2. ✅ Object-op fixture fix: Adding `__init__` line and regenerating fixture inputs correctly addresses the runner/fixture mismatch
3. ✅ Classification: `mixed/equivalent` with appropriate `trie_parity`, `field_clone`, `dict_clone`, `stateful_object` tags is correct despite remaining severe slowness
4. ✅ No hidden risks: Harness, fixture, and codegen reviewed with no issues found

**No further review round needed.**