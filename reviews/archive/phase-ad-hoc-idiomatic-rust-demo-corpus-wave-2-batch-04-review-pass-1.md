All three demos compile and pass. Now let me provide my review findings.

---

## Review: Wave 2 Batch 04 Idiomatic Rust Demos

### `demos/os/idiomatic.rs` — **2 actionable issues**

**Issue 1: `run_command` discards stderr content on success (lines 14-15)**
When the command succeeds, `output.stderr` is silently discarded. This loses potentially useful diagnostic information. If a command produces stderr output even on success (e.g., warnings), this is lost. A stronger design would either:
- Return both stdout and stderr
- Log or include stderr in the return for debugging

**Issue 2: Cleanup ignores errors (lines 68-69)**
```rust
let _ = fs::remove_file(&file_path);
let _ = fs::remove_dir(&base);
```
Using `let _ =` silently discards cleanup errors. If these fail (e.g., permission issues, race conditions), the test proceeds anyway. This masks real problems. Consider at least logging errors or propagating them.

**Minor concern: Closure-based error handling is verbose (lines 72-84)**
The `|| -> Result<...> { ... }()` IIFE with `unwrap_or` is unusual. While functionally correct, using early returns with `?` operator would be more idiomatic and readable.

---

### `demos/platform/idiomatic.rs` — **1 actionable issue**

**Issue: `uname` silently swallows all errors (lines 30-38)**
```rust
fn uname(flag: &str) -> String {
    Command::new("uname")
        .arg(flag)
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| system())
}
```
If `uname` fails for any reason (command not found, permission denied, etc.), the error is silently swallowed and `system()` is returned as a fallback. This makes debugging difficult since there's no indication that `uname` failed. Consider logging or distinguishing between "uname not found" vs "empty output" cases.

---

### `demos/uuid/idiomatic.rs` — **2 actionable issues**

**Issue 1: `to_str` returns `String` instead of `&str` (lines 21-23)**
```rust
fn to_str(&self) -> String {
    self.raw.clone()
}
```
Returning `String` when the method already has `&self` access to a `String` is inefficient. Returning `&str` would avoid the clone and be more idiomatic with Rust's `Deref` coercion patterns. The Sifr reference returns a string type, and this implementation clones unnecessarily.

**Issue 2: `version` uses `-1` as error sentinel (lines 25-32)**
```rust
fn version(&self) -> i64 {
    self.raw
        .chars()
        .nth(14)
        .and_then(|ch| ch.to_digit(16))
        .map(i64::from)
        .unwrap_or(-1)
}
```
Using `-1` as an error sentinel is not idiomatic Rust. `Option<u32>` would be the correct type for a value that may not exist. Returning `i64` also suggests the version could be negative, which is semantically wrong (UUID versions are 1-7).

**Minor: Error message duplication loses context (lines 93-95)**
Both `normalize_uuid_text` failure and `ExternalUuid::parse_str` failure produce the same error message "badly formed hexadecimal UUID string", making it harder to debug which stage failed.

---

### Summary

| File | Actionable Issues |
|------|-------------------|
| `demos/os/idiomatic.rs` | 2 (stderr discard, silent cleanup errors) |
| `demos/platform/idiomatic.rs` | 1 (silent uname failure) |
| `demos/uuid/idiomatic.rs` | 2 (`to_str` clone, `-1` sentinel) |

All demos are functionally correct and pass their assertions. The issues above are API design and error handling quality concerns — they work but could be improved for production-quality Rust.
