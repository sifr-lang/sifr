## Review: wave2 batch11 pass2

### Severity: Low — Semantic Edge Case

#### Finding: `subprocess/idiomatic.rs` — `check_call` return value semantics differ from Python

**Severity:** Low  
**Location:** `check_call` function, line ~41

Python's `subprocess.check_call` returns `0` on success (the exit code is discarded), and raises `CalledProcessError` on failure. The Rust companion returns `Ok(result.returncode)` — the exit code itself — on success. The demo handles this correctly (it asserts `rc == 0`), but the return value semantics are not identical.

**Impact:** Low. No correctness issue in the demo's usage. This is a Rust-idiomatic shape (`Result` wrapping exit code) rather than Python API parity. Acceptable within "Rust-first companion" intent.

---

### Pass 1 Items — Status Recheck

| Finding | Status |
|---------|--------|
| Dead constants `PIPE`/`STDOUT`/`DEVNULL` in `subprocess` | Unchanged. Intentional-in-batch-scope. |
| `ZipFile::open` rejects `"rb"` | Unchanged. Deliberate text-mode-only constraint. |
| `NamedTemporaryFile` lacks `Drop` | Unchanged. Explicit `.close()` + `.cleanup()` pattern demonstrated. |

---

### No New Correctness or Maintainability Issues Found

Re-examined all three files for:
- Behavioral mismatches between companion and Python reference
- Edge-case mistakes in error handling and resource lifecycle
- Misleading API semantics
- Subtle ownership or lifetime issues

The implementation remains sound for its demonstrated scope. The `check_call` return semantics difference from Python is the only new item, and it does not warrant revision.

---

### Final Verdict

**Accepted.** Wave2 batch11 pass2 contains no findings that meet the re-check bar. The companion implementations correctly validate their targeted surface area. No behavioral regressions or correctness problems identified.
