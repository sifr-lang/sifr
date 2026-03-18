# wave_psp_d1 Review: CPython Parity Gap Analysis (R4)

**Review Date:** 2026-03-17
**Reviewer:** Claude Code
**Wave Scope:** filesystem, paths, archives (io, pathlib, glob, shutil, tempfile, gzip, zipfile)
**Status:** FINAL

---

## Executive Summary

wave_psp_d1 provides filesystem, path manipulation, and archive support for Sifr. All actionable gaps from previous review cycles have been resolved. The implementation fully satisfies the traceability contract.

**Key Findings:**
- All core surfaces implemented and functional
- All R1 issues resolved (is_absolute Windows support, gzip tests)
- All R3 issues resolved (zipfile missing-entry read error test added)
- All e2e pass tests execute successfully
- All error path tests correctly reject invalid inputs

---

## Previous Review Status

### R1 Issues - Status Update

| Issue | Status | Notes |
|-------|--------|-------|
| `is_absolute()` incomplete for POSIX/Windows | ✅ FIXED | Now handles Windows drive paths (C:/, C:\) and both / and \ (lib/sifr/pathlib.sifr:57-73) |
| Missing r+/w+/a+ modes | ✅ VERIFIED WAIVER | Correctly rejected with IOError - documented in traceability as `unsupported` |
| walk_dir intrinsic not tested | ✅ VERIFIED WAIVER | Documented as internal-only in traceability |
| gzip invalid data not fully tested | ✅ FIXED | Tests now cover invalid data and truncated data (cpython_gzip_subset.sifr:33-51) |

### R3 Issues - Status Update

| Issue | Status | Notes |
|-------|--------|-------|
| zipfile missing-entry read error not tested | ✅ FIXED | Test added at cpython_zipfile_subset.sifr:20-27 |
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
| `sifr.zipfile` | create/write/read/namelist, error paths | ZipFile class | ✅ MATCH |

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
| zipfile | ✅ cpython_zipfile_subset.sifr | ✅ phase_psp_d1_zipfile_write_non_string_content.sifr | ✅ Missing entry, missing archive | ✅ |

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

All pass tests run successfully:
- cpython_io_subset.sifr ✅
- cpython_pathlib_subset.sifr ✅
- cpython_glob_subset.sifr ✅
- cpython_shutil_subset.sifr ✅
- cpython_tempfile_subset.sifr ✅
- cpython_gzip_subset.sifr ✅
- cpython_zipfile_subset.sifr ✅
- phase_psp_d1_filesystem_paths_archives.sifr ✅

Fail tests correctly reject invalid inputs:
- `phase_psp_d1_io_open_non_string_mode.sifr` → type error
- `phase_psp_d1_glob_non_string_pattern.sifr` → type error
- `phase_psp_d1_shutil_copy_non_string_path.sifr` → type error
- `phase_psp_d1_zipfile_write_non_string_content.sifr` → type error

---

## Conclusion

**SATISFIED: no actionable gaps.**

wave_psp_d1 fully delivers on its traceability claims:
- ✅ Core functionality verified working
- ✅ All documented waivers correctly implemented
- ✅ All R1 issues (is_absolute, gzip tests) resolved
- ✅ All R3 issues (zipfile missing-entry test) resolved
- ✅ All error path tests pass

The wave is ready for advancement.

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
- `crates/sifr_codegen/src/intrinsics/zipfile.rs` - Zip handling

### Tests
- `crates/sifr/tests/e2e/pass/phase_psp_d1_filesystem_paths_archives.sifr`
- `crates/sifr/tests/e2e/pass/cpython_io_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_pathlib_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_glob_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_shutil_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_tempfile_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_gzip_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_zipfile_subset.sifr` (updated with missing-entry test)
- `crates/sifr/tests/e2e/fail/phase_psp_d1_*.sifr` (4 files)

### Documentation
- `verification/stdlib/wave_psp_d1_cpython_traceability.md`
