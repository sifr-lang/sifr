Verification of the two demo fixes that are **not** flagged:

- **`demos/bytes_module/main.sifr:54`** (`"true"` → `"false"`): legitimate. Payload string was renamed from `"bytes-phase30"` to `"bytes-bytes_module"` in commit `accf5012` without updating the expected vector; `"bytes-bytes_module".ends_with("e30")` is `false`.
- **`demos/bytes_errors/main.sifr:24`** (`"latin-1"` → `"definitely-not-a-codec"`): legitimate. `latin-1` is a first-class codec (`crates/sifr_runtime/src/encoding.rs:432`), so `.encode("latin-1")` succeeded and `assert bad_codec` (line 38) would have failed. The new string is guaranteed unresolvable.

Top actionable concern: finding #1 — the `loads` → `toml_loads` rename in the parse_safety and structured_parsing_serialization demos plausibly hides an import-resolution defect in the Sifr HIR/lowering. Before merging, capture the actual diagnostic with the old imports (revert those two lines temporarily and run the demos) — that will confirm whether the fix is legitimate or is masking a compiler bug that violates AGENTS.md's "solve root causes" rule.
