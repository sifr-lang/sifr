# wave_psp_d1 Review: CPython Parity Gap Analysis (R3)

**Review Date:** 2026-03-17
**Reviewer:** Claude Code
**Wave Scope:** filesystem, paths, archives (io, pathlib, glob, shutil, tempfile, gzip, zipfile)
**Status:** FINAL

---

## Executive Summary

wave_psp_d1 provides filesystem, path manipulation, and archive support for Sifr. The implementation delivers on all core surfaces claimed in the traceability document. Most gaps from previous review cycles have been addressed. One remaining test coverage gap exists.

**Key Findings:**
- All core surfaces implemented and functional
- Previous r1 issue (is_absolute Windows support) has been resolved
- 1 remaining test coverage gap
- All e2e pass tests execute successfully

---

## Previous Review Status

### R1 Issues - Status Update

| Issue | Status | Notes |
|-------|--------|-------|
| `is_absolute()` incomplete for POSIX/Windows | ✅ FIXED | Now handles Windows drive paths (C:/, C:\) and both / and \ (lib/sifr/pathlib.sifr:57-73) |
| Missing r+/w+/a+ modes | ✅ VERIFIED WAIVER | Correctly rejected with IOError - documented in traceability as `unsupported` |
| walk_dir intrinsic not tested | ✅ VERIFIED WAIVER | Documented as internal-only in traceability |
| gzip invalid data not fully tested | ✅ FIXED | Tests now cover invalid data and truncated data (cpython_gzip_subset.sifr:33-51) |

### R2 Issues - Status Update

| Issue | Status | Notes |
|-------|--------|-------|
| zipfile missing entry read error not tested | ⚠️ STILL OPEN | No test for reading non-existent entry from valid archive |
| io append mode via FileHandle not explicitly tested | ✅ VERIFIED | `append_text()` helper tested; underlying mode works |

---

## Traceability Validation

### ✅ Verified Surfaces (Match Claims)

| Surface | Traceability Claim | Implementation | Status |
|---------|-------------------|----------------|--------|
| `sifr.io` | text/binary reads/writes, context-managed, missing-file/invalid-mode error paths | FileHandle with read/write/read_bytes/write_bytes, context manager | ✅ MATCH |
| `sifr.pathlib` | exists, is_file, is_dir, read_text, write_text, glob, rglob, iterdir, resolve, with_name, with_suffix | Path class with all methods | ✅ MATCH |
| `sifr.glob` | wildcard matching (*, ?, prefix), hidden-entry handling, missing-root | glob(directory, pattern) returns sorted list | ✅ MATCH |
| `sifr.shutil` | copy, move_file, rmtree, which, disk_usage | All functions present | ✅ MATCH |
| `sifr.tempfile` | mkstemp, mkdtemp, unique path generation, missing-parent failure | All functions with collision safety | ✅ MATCH |
| `sifr.gzip` | compress/decompress string, invalid/truncated rejection | list[int] interface (documented waiver) | ✅ MATCH |
| `sifr.zipfile` | create/write/read/namelist, error paths | ZipFile class | ⚠️ PARTIAL |

---

## Actionable Finding

### 1. MEDIUM: zipfile Missing Entry Read Error Not Tested

**Location:** `crates/sifr/tests/e2e/pass/cpython_zipfile_subset.sifr`

**Issue:** The traceability document claims "missing-entry / missing-archive error paths" but the test does not verify reading a non-existent entry from a valid archive.

**Traceability Claim (line 13):** "create/write/read/namelist archive basics and missing-entry / missing-archive error paths"

**Current Test Coverage:**
- ✅ Creates archive and writes files
- ✅ Lists archive contents
- ✅ Tests missing-archive error (lines 28-33 - tests namelist after archive deleted)
- ❌ Does NOT test reading non-existent entry from valid archive

**Missing Test Case:**
```sifr
# Should test: zf.read("nonexistent.txt") returns IOError
zf: ZipFile = ZipFile(path)
_ = zf.create()
_ = zf.write("existing.txt", "content")
# This should return IOError:
_bad_read: str = zf.read("nonexistent.txt")
```

**Recommendation:** Add test case for reading non-existent entry to fully satisfy the "missing-entry error path" claim.

---

## Test Coverage Summary

| Module | Pass Tests | Fail Tests | Error Path Tests | Traceability Match |
|--------|------------|------------|------------------|---------------------|
| io | ✅ cpython_io_subset.sifr | ✅ phase_psp_d1_io_open_non_string_mode.sifr | ✅ Missing file, invalid mode, r+/w+/a+ | ✅ |
| pathlib | ✅ cpython_pathlib_subset.sifr | None | ✅ Missing read | ✅ |
| glob | ✅ cpython_glob_subset.sifr | ✅ phase_psp_d1_glob_non_string_pattern.sifr | ✅ Missing root | ✅ |
| shutil | ✅ cpython_shutil_subset.sifr | ✅ phase_psp_d1_shutil_copy_non_string_path.sifr | ✅ Missing copy/move/rmtree | ✅ |
| tempfile | ✅ cpython_tempfile_subset.sifr | None | ✅ Missing parent | ✅ |
| gzip | ✅ cpython_gzip_subset.sifr | None | ✅ Invalid/truncated data | ✅ |
| zipfile | ✅ cpython_zipfile_subset.sifr | ✅ phase_psp_d1_zipfile_write_non_string_content.sifr | ⚠️ Incomplete | ⚠️ |

---

## Verified Waivers (Intentional)

The following are correctly documented as unsupported in the traceability and are working as intended:

| Surface | Waiver State | Verification |
|---------|--------------|--------------|
| Read/write mixed modes (r+, w+, a+) | `unsupported` | Tests verify IOError is raised (cpython_io_subset.sifr:68-83) |
| pathlib PurePath/PosixPath/WindowsPath | `unsupported` | Single Path class present |
| glob recursive `**` | `unsupported` | Only non-recursive matching |
| shutil copy2, copytree, archive helpers | `unsupported` | Only copy/move/rmtree/which/disk_usage |
| tempfile NamedTemporaryFile, TemporaryDirectory | `unsupported` | Only path helpers |
| gzip GzipFile, file-object APIs | `unsupported` | Only compress/decompress |
| zipfile compression methods, extraction APIs | `unsupported` | Only create/write/read/namelist |
| walk_dir intrinsic | `unsupported` | Internal only |

---

## Verification

All tests verified to compile and run:

```bash
$ cargo run -q -p sifr -- run demos/wave_psp_d1_filesystem_paths_archives_demo.sifr
io.read_text = hello d1
pathlib.stem = note
glob("*.txt") = ["note.txt"]
shutil.move_file exists = true
tempfile.mkstemp = /var/.../sifr_wave_psp_d1_demo_*
tempfile.mkdtemp = /var/.../sifr_wave_psp_d1_demo_*
gzip roundtrip = archive sample
zipfile.read = inside-zip
zipfile.namelist = Ok(["inside.txt"])
```

Fail tests correctly reject invalid inputs:
- `phase_psp_d1_io_open_non_string_mode.sifr` → type error
- `phase_psp_d1_glob_non_string_pattern.sifr` → type error
- `phase_psp_d1_shutil_copy_non_string_path.sifr` → type error
- `phase_psp_d1_zipfile_write_non_string_content.sifr` → type error

---

## Conclusion

wave_psp_d1 is substantially complete. The implementation delivers on all core traceability claims with one minor test coverage gap:

- ✅ Core functionality verified working
- ✅ All documented waivers correctly implemented
- ✅ R1 issues (is_absolute, gzip tests) resolved
- ⚠️ 1 test coverage gap: zipfile missing-entry read error not tested

**Overall Assessment:** The wave delivers on its traceability claims with one verified gap in error path testing for zipfile. Recommend adding the missing test case to fully satisfy the "missing-entry error path" claim.

---

## Appendix: Files Reviewed

### Implementation
- `lib/sifr/io.sifr` - Public IO API
- `lib/sifr/pathlib.sifr` - Public pathlib API (lines 57-73 is_absolute fix verified)
- `lib/sifr/glob.sifr` - Public glob API
- `lib/sifr/shutil.sifr` - Public shutil API
- `lib/sifr/tempfile.sifr` - Public tempfile API
- `lib/sifr/gzip.sifr` - Public gzip API
- `lib/sifr/zipfile.sifr` - Public zipfile API
- `crates/sifr_codegen/src/intrinsics/file_handles.rs` - File mode handling

### Tests
- `crates/sifr/tests/e2e/pass/phase_psp_d1_filesystem_paths_archives.sifr`
- `crates/sifr/tests/e2e/pass/cpython_io_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_pathlib_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_glob_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_shutil_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_tempfile_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_gzip_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_zipfile_subset.sifr`
- `crates/sifr/tests/e2e/fail/phase_psp_d1_*.sifr` (4 files)

### Documentation
- `verification/stdlib/wave_psp_d1_cpython_traceability.md`
