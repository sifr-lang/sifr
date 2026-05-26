Based on my audit of the Ruff codebase, here are my findings ordered by severity:

---

## Findings: File Discovery/Cache/Path Utilities

### CRITICAL — Must Reimplement (Python-Specific)

**1. Package Root Detection (`packaging.rs`)**
- **What:** `is_package()` checks for `__init__.py`, `detect_package_root()` walks ancestors
- **Why critical:** Sifr has no Python package semantics — `__init__.sifr` or similar would be non-standard
- **Action:** Delete entirely. Sifr should use simple directory/file hierarchy, not Python's package model

**2. Pyproject Configuration System (`resolver.rs` + `configuration.rs`)**
- **What:** `PyprojectConfig`, `PyprojectDiscoveryStrategy::Fixed|Hierarchical`, pyproject.toml loading
- **Why critical:** Tied to `pyproject.toml` structure with `[tool.ruff]` sections
- **Action:** Sifr needs its own config format (likely `sifr.toml`) with parallel structure but different semantics

**3. Python File Filtering (hardcoded in `settings.rs`)**
```rust
pub(crate) static INCLUDE: &[FilePattern] = &[
    FilePattern::Builtin("*.py"),
    FilePattern::Builtin("*.pyi"),
    FilePattern::Builtin("**/pyproject.toml"),
];
```
- **Why critical:** Extension mapping is Python-specific, notebooks (`.ipynb`) handling, Python target-version
- **Action:** Sifr needs `*.sifr` (or `.sifr`) as the primary include pattern

---

### HIGH — Extract and Adapt

**4. `ignore` crate Walker Integration (`resolver.rs:374-381`)**
```rust
let mut builder = WalkBuilder::new(first_path);
builder.standard_filters(resolver.respect_gitignore());
builder.hidden(false);
```
- **What:** Uses `ignore` crate's `WalkBuilder` for recursive file traversal with gitignore/ignore support
- **Why high:** This is language-neutral and well-designed
- **Action:** REUSE directly — `ignore` crate handles `.gitignore`, `.ignore`, custom ignore files
- **Sifr adaptation:** Use same `WalkBuilder` pattern but with Sifr-specific filters

**5. Glob Pattern Matching (`FilePattern`/`FilePatternSet` in `types.rs`)**
- **What:** `GlobSet` from `globset` crate, `FilePattern::Builtin|User`, `match_candidate_exclusion()`
- **Why high:** Foundation for include/exclude functionality
- **Action:** REUSE directly — this is language-neutral
- **Sifr adaptation:** Change default includes from `*.py` to `*.sifr`

**6. Path Normalization (`fs.rs`)**
```rust
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf
pub fn normalize_path_to<P, R>(path: P, project_root: R) -> PathBuf
pub fn relativize_path<P: AsRef<Path>>(path: P) -> String
pub fn relativize_path_to<P, R>(path: P, project_root: R) -> String
```
- **What:** Absolute/relative path conversion utilities
- **Why high:** Uses `path_absolutize` crate
- **Action:** REUSE directly — language-neutral

**7. File Exclusion Logic (`resolver.rs:400-414`)**
```rust
if match_candidate_exclusion(&file_path, &file_basename, &settings.file_resolver.exclude) { ... }
```
- **What:** Dual matching (full path + basename) for exclusion patterns
- **Why high:** Enables patterns like `foo.py` (exact) vs `*.py` (basename)
- **Action:** REUSE directly with Sifr patterns

---

### MEDIUM — Framework-Only (Extract Later)

**8. Cache Key System (`ruff_cache/src/cache_key.rs`)**
- **What:** `CacheKey` trait, `CacheKeyHasher` using `seahash`, implementations for primitives/collections/Path/Regex
- **Why medium:** Well-designed, portable hash for cache keys
- **Action:** REUSE directly when Sifr adds caching — requires `seahash` dependency
- **Sifr adaptation:** None needed

**9. Cache Persistence Layer (`ruff/src/cache.rs`)**
- **What:** Package-based cache organization, atomic writes, `FileCacheKey` with mtime/permissions
- **Why medium:** Complex but solid
- **Action:** REUSE architecture pattern when Sifr adds caching
- **Sifr adaptation:** Change from Python package model to Sifr project model

**10. Cache Directory Setup (`ruff_cache/src/lib.rs`)**
```rust
pub const CACHE_DIR_NAME: &str = ".ruff_cache";
pub fn cache_dir(project_root: &Path) -> PathBuf { ... }
```
- **Why medium:** Standard cache dir pattern
- **Action:** REUSE with Sifr naming (`.sifr_cache`?)

**11. `filetime` Integration (`ruff_cache/src/filetime.rs`)**
- **What:** `FileTime` for cache invalidation based on mtime
- **Action:** REUSE directly (via `filetime` crate)

---

### LOW — Straight Reuse

**12. Settings Structure (`ruff_workspace/src/settings.rs`)**
```rust
pub struct FileResolverSettings {
    pub exclude: FilePatternSet,
    pub extend_exclude: FilePatternSet,
    pub force_exclude: bool,
    pub include: FilePatternSet,
    pub extend_include: FilePatternSet,
    pub respect_gitignore: bool,
    pub project_root: PathBuf,
}
```
- **Action:** REUSE directly — just change defaults

**13. ResolvedFile Enum (`resolver.rs:496-539`)**
```rust
pub enum ResolvedFile {
    Root(PathBuf),   // Explicit CLI target
    Nested(PathBuf), // Discovered via walk
}
```
- **Action:** REUSE directly for Sifr

**14. ExclusionKind Enum (`resolver.rs:629-650`)**
- **Action:** Adapt for Sifr tool types (lint vs format)

---

## Strategy Assessment

**Sound strategy: YES**, with the following caveats:

| Component | Recommendation | Complexity |
|-----------|---------------|------------|
| `ignore` crate walker | **Direct reuse** | Low — just use it |
| Globset patterns | **Direct reuse** | Low — change defaults |
| Path utilities | **Direct reuse** | Low — copy `fs.rs` |
| Cache infrastructure | **Extract later** | Medium — requires `seahash` |
| Pyproject config | **Must reimplement** | High — Sifr needs `sifr.toml` |
| Package detection | **Delete** | N/A — Python-specific |

## Suggested Sifr Crate Structure

```
sifr_workspace/src/
├── resolver.rs      # Adapted from Ruff, uses ignore crate
├── settings.rs      # FileResolverSettings adapted
├── fs.rs           # Copy from Ruff (path normalization)
└── config.rs       # New: sifr.toml loading

sifr_cache/         # New crate when needed
├── cache_key.rs    # Copy from Ruff
└── lib.rs          # Adapted cache dir naming
```

## Key External Dependencies Sifr Needs

- `ignore` — file walking with gitignore support
- `globset` — glob pattern matching
- `path_absolutize` — path normalization  
- `seahash` — when adding cache support
