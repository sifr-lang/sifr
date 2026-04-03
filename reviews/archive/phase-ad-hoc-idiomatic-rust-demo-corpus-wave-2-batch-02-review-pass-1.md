## Review: Wave 2 Batch 02 Idiomatic Rust Demos

After reviewing `demos/math/idiomatic.rs`, `demos/pathlib/idiomatic.rs`, and `demos/glob/idiomatic.rs`, I found **one behavioral issue** and several API design issues.

---

### Issue 1: `glob` silently swallows non-NotFound errors (Behavioral — `demos/glob/idiomatic.rs:10-13` and `demos/pathlib/idiomatic.rs:91-95`)

In `glob/idiomatic.rs:10-13`:
```rust
Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
Err(_) => return Vec::new(),  // Permission denied, etc. also returns empty!
```

Same pattern in `pathlib/idiomatic.rs:93-94`:
```rust
Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
Err(error) => return Err(error.into()),  // Returns the raw std::io::Error, not IOError
```

**Problem**: The `pathlib/idiomatic.rs` version returns `Err(error.into())` where `error` is `std::io::Error` but the function returns `Result<Vec<String>, IOError>`. The `From<std::io::Error> for IOError` impl exists at line 13-18, so this does work. However, callers of the public `glob` method (line 71-73) get an `IOError` that wraps the raw `std::io::Error`, losing the semantic distinction.

If an experienced Rust engineer wanted the same behavior (not-found returns empty, other errors propagate), they'd use the std `Error` type directly or propagate with `?`.

---

### Issue 2: Manual `IOError` wrapper is unnecessary boilerplate (`demos/pathlib/idiomatic.rs:8-27`)

```rust
#[derive(Debug, Clone)]
struct IOError {
    message: String,
}
```

`std::io::Error` already implements `std::error::Error`, `Debug`, `Clone`, and has a `to_string()` message. An experienced Rust engineer would either:
- Use `std::io::Error` directly as the error type
- Use `anyhow::Error` for ergonomic error handling
- Use `thiserror` for derive-based error types

Creating a hand-rolled `IOError` that just wraps the message string adds noise without benefit.

---

### Issue 3: `Path` uses `String` internally instead of `PathBuf` (`demos/pathlib/idiomatic.rs:30-31`)

```rust
struct Path {
    raw: String,
}
```

For a path-manipulation type, `std::path::PathBuf` would be more idiomatic. It provides `exists()`, `is_file()`, `is_dir()` natively, plus `parent()`, `file_name()`, `extension()` etc. An experienced Rust engineer would compose `std::path::PathBuf` rather than reimplementing these methods on top of a `String`.

---

### Issue 4: `Path::new` takes `impl Into<String>` instead of `impl AsRef<Path>` (`demos/pathlib/idiomatic.rs:35-37`)

```rust
fn new(path: impl Into<String>) -> Self {
    Self { raw: path.into() }
}
```

The idiomatic Rust pattern is `impl AsRef<Path>`, which accepts `String`, `&str`, `PathBuf`, `&Path`, `Path`, and other path-like types transparently. Using `Into<String>` forces conversion even when the input is already a `Path` type.

---

### Issue 5: `join_path` doesn't use `std::path::Path` (`demos/pathlib/idiomatic.rs:76-84`)

```rust
fn join_path(base: &str, child: &str) -> String {
    if base.is_empty() {
        child.to_string()
    } else if base.ends_with('/') {
        format!("{base}{child}")
    } else {
        format!("{base}/{child}")
    }
}
```

This manual string concatenation doesn't handle platform differences. `std::path::Path::join()` is the idiomatic solution and handles `\` on Windows.

---

### Minor: Redundant flag variable in `collect_path_class_actual` (`demos/pathlib/idiomatic.rs:163`)

`path_flow_ok` starts as `false` and is only set to `true` when the condition succeeds, then never changed back. It could be replaced by just checking if the Vec is non-empty at the end, but this is a style observation, not a bug.

---

### What's sound

- **`math/idiomatic.rs`**: No actionable issues. The `isclose`, `fsum` (Neumaier algorithm), `nextafter`, and `ulp` implementations are correct. Error handling is proper.
- **`glob/idiomatic.rs` wildcard_match**: The memoized recursive matching is correct, though simpler without memoization for a demo.
- **`pathlib/idiomatic.rs` glob_entries**: The logic is sound; it correctly handles hidden files and sorting.

---

### Summary

**Actionable issues found**: 5 (1 behavioral, 4 API design)

The most significant is the `glob` silent-error swallowing. An experienced Rust engineer would propagate errors explicitly or use `anyhow` for ergonomic propagation rather than returning empty vectors for non-not-found errors.
