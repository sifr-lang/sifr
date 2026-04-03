## Review: Wave 2 Batch 03 Idiomatic Rust Demos

### demos/io/idiomatic.rs

**1. `FileHandle::close()` is a misleading no-op (line 19)**
```rust
fn close(self) {}
```
This is bad API design. It implies manual resource cleanup but does nothing—in Rust, the `File` is already closed when `FileHandle` is dropped. A reader might call `file.close()` expecting cleanup and be surprised. Either remove it or just drop the struct entirely. This is the most significant issue.

**2. `FileHandle` struct adds indirection without value (lines 10-12)**
The struct wraps `Lines<BufReader<File>>` but only exposes a single `readline()` method. An idiomatic Rust solution would either use the buffered lines directly or return a type that exposes more of the underlying `BufReader` interface. For a demo, this is acceptable but not maximally idiomatic.

**Minor:** `exists()` at line 36 uses `std::path::Path::exists()` which is fine for the demo.

---

### demos/csv/idiomatic.rs

**1. `parse_row()` and `format_row()` don't handle CSV escaping (lines 10-16, 22-24)**
Per RFC 4180, CSV fields containing commas, double quotes, or newlines must be quoted. The current implementation:
```rust
fn parse_row(row: &str) -> Vec<String> {
    if row.is_empty() {
        Vec::new()
    } else {
        row.split(',').map(str::to_string).collect()
    }
}
```
...will incorrectly parse `"a,b","c"` as three fields instead of two, and will corrupt output if a field contains a comma. An experienced Rust engineer would use the `csv` crate from crates.io for correct RFC 4180 compliance, or at minimum implement proper quoting logic.

**2. `CsvReader` and `CsvWriter` are in-memory only (lines 33-47, 49-66)**
```rust
struct CsvReader {
    rows: Vec<Vec<String>>,
}
```
This loads entire file contents into memory. For large files this is inefficient. The `csv` crate provides streaming readers/writers that are more idiomatic for production use.

---

### demos/shutil/idiomatic.rs

**1. `disk_usage()` uses external crate `fs2` without justification (lines 66-71)**
```rust
fn disk_usage(path: &str) -> Vec<u64> {
    let path = PathBuf::from(path);
    let total = fs2::total_space(&path).unwrap_or(0);
    let free = fs2::available_space(&path).unwrap_or(0);
    vec![total, total.saturating_sub(free), free]
}
```
This adds a external dependency (`fs2`). The standard library doesn't expose this directly, but an experienced Rust engineer would note that this requires an external crate. This may be intentional for the demo, but it's worth flagging.

**2. `which()` has incomplete PATH handling (lines 52-64)**
```rust
fn which(name: &str) -> Option<String> {
    let candidate = Path::new(name);
    if candidate.components().count() > 1 && candidate.is_file() {
        return Some(candidate.to_string_lossy().into_owned());
    }

    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|directory| directory.join(name))
            .find(|path| path.is_file())
            .map(|path| path.to_string_lossy().into_owned())
    })
}
```
This doesn't check if the file is executable (via `is_file()` alone). On Unix, you need `path.metadata().map(|m| m.permissions().readonly())` or similar. It also doesn't handle the case where PATH is empty.

**3. `mktemp_path()` could use `std::env::temp_dir()` combined with a UUID (lines 29-38)**
Using `SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(0)` as a unique identifier is fragile—`SystemTime` underflow can occur on systems with pre-UNIX epoch clocks. Using `uuid` crate or `rand` would be more idiomatic for unique temp paths.

---

### Summary

| File | Severity | Issue |
|------|----------|-------|
| io/idiomatic.rs | **High** | `close()` is a misleading no-op |
| csv/idiomatic.rs | **High** | No CSV escaping per RFC 4180 |
| csv/idiomatic.rs | Medium | In-memory-only reader/writer |
| shutil/idiomatic.rs | Medium | External `fs2` dependency |
| shutil/idiomatic.rs | Medium | `which()` doesn't verify executability |
| shutil/idiomatic.rs | Low | `mktemp_path()` fragile timestamp uniqueness |

The CSV escaping issue is the most consequential for behavioral correctness—inputs like `"a,b","c"` will produce wrong output. The `close()` no-op in io is the most misleading from an API design perspective.
