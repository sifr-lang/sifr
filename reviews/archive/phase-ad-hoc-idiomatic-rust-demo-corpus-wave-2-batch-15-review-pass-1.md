Review written to `/Users/yaseralnajjar/.codex/worktrees/9987/codebase/reviews/phase-ad-hoc-idiomatic-rust-demo-corpus-wave-2-batch-15-review-pass-1.md`.

**Summary:**

| File | Verdict |
|------|---------|
| `demos/json_values/idiomatic.rs` | **APPROVED** |
| `demos/random_hashing/idiomatic.rs` | **CONDITIONAL** (missing deps) |
| `demos/random_state/idiomatic.rs` | **APPROVED** |

**Blocking issue**: `random_hashing/idiomatic.rs` uses `base64` and `sha2` crates which are not in workspace dependencies. The file cannot compile as-is.

**Non-blocking notes**:
- `json_values`: `as_str()`/`as_int()` silently default on type mismatch
- `random_hashing`: Could use `hexdigest()` directly instead of `format!("{digest:x}")`
- `random_state`: Duplicates PCG `next_u64` implementation from `random_hashing`
