# E2E Cache Prune TTL Review - Pass 1

**Reviewer:** Code Review
**Date:** 2026-03-15
**Branch:** codex/e2e-cache-prune-ttl
**Files Reviewed:** `crates/sifr/tests/e2e.rs`

---

## Summary

The e2e cache prune TTL implementation adds automatic cleanup of stale cache entries (older than 2 hours) and orphaned group directories. The implementation is **fundamentally sound** with correct logic for TTL filtering, group existence checks, and orphan cleanup.

---

## Implementation Overview

### New Components

| Component | Description |
|-----------|-------------|
| `E2E_CACHE_TTL_SECS` | TTL constant: 2 hours (7200 seconds) |
| `cache_groups_root()` | Returns path to `cache_dir/groups` |
| `cache_group_path()` | Returns path to specific group directory |
| `prune_cache_manifest()` | Core prune function |
| `sample_cache_root()` | Test helper for temp directories |
| 2 unit tests | Coverage for prune logic |

### Integration Point

The prune function is called at the start of each pass suite run (line 1827-1840) when caching is enabled:

```rust
if config.cache.enabled {
    // ... create cache root ...
    let manifest = read_cache_manifest(&config.cache.root);
    let pruned_manifest = prune_cache_manifest(&config.cache.root, manifest, now_unix_secs);
    write_cache_manifest(&config.cache.root, &pruned_manifest);
    pruned_manifest
}
```

---

## Correctness Analysis

### TTL Logic: ✓ CORRECT

```rust
let cutoff_unix_secs = now_unix_secs.saturating_sub(E2E_CACHE_TTL_SECS);
entries.retain(|_, entry| entry.built_at_unix_secs >= cutoff_unix_secs);
```

- Entries with `built_at >= now - TTL` are retained
- Entries older than TTL are filtered out
- Uses `saturating_sub` to handle edge case of very small timestamps

**Test verification:**
- TTL = 7200 seconds
- `now_unix_secs = 20000`
- `cutoff_unix_secs = 12800`
- Stale entry (12799): filtered out ✓
- Live entry (20000): retained ✓

### Group Directory Handling: ✓ CORRECT

The implementation handles three scenarios:

1. **Entry references non-existent group directory**: Entry is removed from manifest
2. **Group directory exists but not in manifest**: Orphaned group is removed from disk
3. **Group has mixed fresh/expired entries**: Group is kept if any entry is fresh

### Error Handling: ✓ ADEQUATE

- Missing groups directory: Gracefully handled with `NotFound` check
- Permission errors on removal: Logged but doesn't fail the operation
- JSON serialization errors: Handled by `write_cache_manifest`
- System time before UNIX_EPOCH: Falls back to no pruning (all entries kept)

---

## Edge Cases Considered

| Edge Case | Handling | Status |
|-----------|----------|--------|
| Empty manifest | Returns empty manifest | ✓ Handled |
| Missing groups directory | NotFound error ignored | ✓ Handled |
| Entry references missing group | Entry removed from manifest | ✓ Handled |
| Orphan groups on disk | Removed from disk | ✓ Handled |
| All entries expired | Manifest emptied, no groups remain | ✓ Handled |
| System time < UNIX_EPOCH | No pruning occurs (graceful fallback) | ✓ Handled |
| Concurrent access | Not handled (acceptable for local e2e) | N/A |

---

## Test Coverage

### Test 1: `test_prune_cache_manifest_removes_expired_entries_and_orphan_groups`

**Purpose:** Verify expired entries, missing groups, and orphan groups are handled.

**Test scenario:**
- 3 groups on disk: stale-group, live-group, orphan-group
- 3 entries in manifest: stale-key, live-key, missing-key
- stale-key: expired (now - TTL - 1)
- live-key: fresh (now)
- missing-key: fresh but group doesn't exist

**Expected results:**
- live-key retained
- stale-key removed (expired)
- missing-key removed (group doesn't exist)
- stale-group removed (orphan)
- orphan-group removed (orphan)

**Status:** ✓ PASSES

### Test 2: `test_prune_cache_manifest_keeps_shared_live_group_for_fresh_entry`

**Purpose:** Verify shared groups are kept when at least one entry is fresh.

**Test scenario:**
- 1 group on disk: shared-group
- 2 entries: old-key (expired), fresh-key (fresh)

**Expected results:**
- fresh-key retained
- old-key removed (expired)
- shared-group directory preserved (has fresh entry)

**Status:** ✓ PASSES

---

## Potential Issues

### 1. Minor: Groups Directory Existence Check

**Location:** `prune_cache_manifest` lines 1148-1150

```rust
.next_manifest
    .entries
    .retain(|_, entry| cache_group_path(root, &entry.group_id).is_dir());
```

**Issue:** If the groups directory (`cache_dir/groups`) doesn't exist but the manifest has entries, all entries would be incorrectly removed because `is_dir()` returns `false` for non-existent paths.

**Impact:** Low - This is unlikely in practice because:
- On first run: manifest is empty
- On subsequent runs: groups directory exists (created during builds)

**Recommendation:** Add explicit check for groups directory existence before the retain operation:

```rust
// Only check group directory existence if groups root exists
if groups_root.is_dir() {
    next_manifest.entries.retain(|_, entry| {
        cache_group_path(root, &entry.group_id).is_dir()
    });
}
```

---

## Regression Risk Assessment

| Factor | Assessment |
|--------|------------|
| Scope | Limited to cache cleanup; doesn't affect build logic |
| Conditional execution | Only runs when caching is enabled |
| Fail-safe | Errors are logged but don't fail the operation |
| Idempotency | Safe to run multiple times |

**Overall Risk:** LOW

---

## Policy Alignment

### Intended Behavior
- Cache entries older than 2 hours should be automatically removed
- Orphaned group directories (not referenced by manifest) should be cleaned up
- No manual cache clearing should be required

### Implementation Matches Intent
- ✓ TTL of 2 hours implemented correctly
- ✓ Expired entries filtered out
- ✓ Orphan groups removed from disk
- ✓ Integration at pass suite start (automatic)

---

## Additional Observations

1. **Cleanup on failure**: If `remove_dir_all` fails for an orphan group, the error is logged but processing continues. This is appropriate.

2. **Manifest versioning**: The schema version is preserved during pruning, which is correct.

3. **No performance concerns**: The implementation uses standard Rust iterators and should be efficient for typical cache sizes.

---

## Conclusion

The e2e cache prune TTL implementation is **approved** with one minor suggestion for improvement. The core logic is correct, tests pass, and the implementation aligns with the intended cache cleanup policy.

### Action Items

- [ ] Consider adding explicit groups directory existence check (optional improvement)
- [ ] Consider adding test for empty groups directory edge case (optional)

---

## Test Results

```bash
$ cargo test -p sifr -- test_prune
    Running tests/e2e.rs
test test_prune_cache_manifest_keeps_shared_live_group_for_fresh_entry ... ok
test test_prune_cache_manifest_removes_expired_entries_and_orphan_groups ... ok

test result: ok. 2 passed; 0 failed
```
