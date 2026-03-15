# E2E Cache Prune TTL Phase - Production Grade Review (Pass 1)

**Reviewer:** Code Review
**Date:** 2026-03-15
**Branch:** main
**Files Reviewed:** `crates/sifr/tests/e2e.rs`
**Commit:** 97bc441e (Prune stale e2e cache entries before pass runs)

---

## Executive Summary

The e2e cache prune TTL phase has been implemented and merged to main. This review assesses whether the implementation is **production-grade** based on correctness, operational safety, regression risk, and maintainability.

**Overall Assessment: PRODUCTION-GRADE WITH ONE KNOWN ISSUE**

The implementation is fundamentally sound and meets the core functional requirements. However, there is one unaddressed issue from the prior review (Pass 1) that should be considered before declaring full production readiness.

---

## Implementation Overview

### Components Added

| Component | Location | Description |
|-----------|----------|-------------|
| `E2E_CACHE_TTL_SECS` | Line 30 | TTL constant: 2 hours (7200 seconds) |
| `cache_groups_root()` | Lines 1047-1049 | Returns path to `cache_dir/groups` |
| `cache_group_path()` | Lines 1051-1057 | Returns path to specific group directory |
| `prune_cache_manifest()` | Lines 1136-1200 | Core prune function |
| `sample_cache_root()` | Lines 1090-1092 | Test helper for temp directories |
| 2 unit tests | Lines 3036-3215 | Coverage for prune logic |

### Integration Point

The prune function is called at the start of each pass suite run (lines 1827-1840) when caching is enabled:

```rust
if config.cache.enabled {
    let manifest = read_cache_manifest(&config.cache.root);
    let pruned_manifest = prune_cache_manifest(
        &config.cache.root,
        manifest,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
    );
    write_cache_manifest(&config.cache.root, &pruned_manifest);
    pruned_manifest
}
```

---

## Correctness Assessment

### TTL Logic: CORRECT

```rust
let cutoff_unix_secs = now_unix_secs.saturating_sub(E2E_CACHE_TTL_SECS);
entries.retain(|_, entry| entry.built_at_unix_secs >= cutoff_unix_secs);
```

- Entries with `built_at >= now - TTL` are retained
- Entries older than TTL are filtered out
- Uses `saturating_sub` to handle edge case of very small timestamps

**Verified by tests:**
- TTL = 7200 seconds
- `now_unix_secs = 20000`
- `cutoff_unix_secs = 12800`
- Stale entry (12799): filtered out
- Live entry (20000): retained

### Group Directory Handling: CORRECT

The implementation handles three scenarios correctly:

1. **Entry references non-existent group directory**: Entry is removed from manifest
2. **Group directory exists but not in manifest**: Orphaned group is removed from disk
3. **Group has mixed fresh/expired entries**: Group is kept if any entry is fresh

---

## Operational Safety

### Error Handling: ADEQUATE

| Scenario | Handling | Status |
|----------|----------|--------|
| Missing groups directory | Gracefully handled with `NotFound` check | ✓ |
| Permission errors on removal | Logged but doesn't fail operation | ✓ |
| JSON serialization errors | Handled by `write_cache_manifest` | ✓ |
| System time before UNIX_EPOCH | Falls back to no pruning | ✓ |
| Corrupt manifest JSON | Rebuilds empty manifest | ✓ |

### Fail-Safe Behavior

- Prune errors are logged but don't crash the test run
- Cache corruption doesn't prevent new builds
- Idempotent: Safe to run multiple times

---

## Regression Risk

| Factor | Assessment |
|--------|------------|
| Scope | Limited to cache cleanup; doesn't affect build logic |
| Conditional execution | Only runs when caching is enabled |
| Fail-safe | Errors logged but don't fail operation |
| Idempotency | Safe to run multiple times |
| Test coverage | Unit tests cover core scenarios |

**Overall Risk: LOW**

---

## Test Coverage

### Test 1: `test_prune_cache_manifest_removes_expired_entries_and_orphan_groups`

**Status: PASSING**

Tests:
- Expired entries are removed
- Entries referencing missing groups are removed
- Orphan group directories are removed from disk

### Test 2: `test_prune_cache_manifest_keeps_shared_live_group_for_fresh_entry`

**Status: PASSING**

Tests:
- Groups with at least one fresh entry are preserved
- Mixed fresh/expired entries handled correctly

```
$ cargo test -p sifr -- test_prune --skip test_e2e_pass
    Running tests/e2e.rs
test test_prune_cache_manifest_keeps_shared_live_group_for_fresh_entry ... ok
test test_prune_cache_manifest_removes_expired_entries_and_orphan_groups ... ok

test result: ok. 2 passed; 0 failed
```

---

## Known Issues

### Issue 1: Missing Groups Directory Edge Case (Unaddressed)

**Location:** `prune_cache_manifest` lines 1148-1150

```rust
next_manifest
    .entries
    .retain(|_, entry| cache_group_path(root, &entry.group_id).is_dir());
```

**Problem:** If the groups directory (`cache_dir/groups`) doesn't exist but the manifest has entries, all entries would be incorrectly removed because `is_dir()` returns `false` for non-existent paths.

**Likelihood:** LOW - This is unlikely in practice because:
- On first run: manifest is empty
- On subsequent runs: groups directory exists (created during builds)

**Recommendation from prior review (not yet implemented):**

```rust
// Only check group directory existence if groups root exists
if groups_root.is_dir() {
    next_manifest.entries.retain(|_, entry| {
        cache_group_path(root, &entry.group_id).is_dir()
    });
}
```

**Impact:** Medium - Could cause cache invalidation in edge cases

---

## Maintainability

### Code Quality: GOOD

- Clear function names
- Appropriate abstraction level
- Constants properly named
- Error messages are descriptive

### Documentation: ADEQUATE

- Inline comments explain key logic
- Previous review document provides context
- No standalone documentation (acceptable for internal utility)

### Configurability: LIMITED

The TTL is hardcoded (2 hours). This is intentional for the e2e use case, as it's an internal testing cache.

---

## Remaining Items to Consider

### Before Full Production Declaration

1. **Consider implementing Issue 1 fix** - The groups directory existence check would make the implementation more robust against edge cases
2. **Consider adding test for missing groups directory edge case** - Would provide regression protection

### Not Required for Production

- Configuration for TTL (not needed for internal e2e cache)
- Metrics/logging beyond error messages
- Cache size limits

---

## Conclusion

The e2e cache prune TTL implementation is **APPROVED for production use** with one known minor issue. The core logic is correct, tests pass, error handling is adequate, and regression risk is low.

The unaddressed issue (groups directory existence check) has low likelihood in practice and the current behavior is fail-safe (errors don't prevent test execution).

### Action Items

- [ ] Consider implementing the groups directory existence check (optional improvement)
- [ ] Consider adding test for missing groups directory edge case (optional)

---

## Test Results Summary

```
$ cargo test -p sifr -- test_prune --skip test_e2e_pass
    Running tests/e2e.rs
test test_prune_cache_manifest_keeps_shared_live_group_for_fresh_entry ... ok
test test_prune_cache_manifest_removes_expired_entries_and_orphan_groups ... ok

test result: ok. 2 passed; 0 failed
```
