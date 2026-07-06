Two non-blocking findings; no blockers. Ready for PR.

- **Non-blocking (efficiency)** — `verify_release_archive.py:140` unconditionally reads every file member's content in the one-pass loop, forcing a full decompress + allocate for the ~40 MB `bin/sifr` binary that is then discarded. Gate on `name == "sysroot.toml" or is_sysroot_content_path(name)`.
- **Non-blocking (cosmetic)** — `compiler_surface_matrix.json:365` now uses `", "` where every other multi-suite row uses `","`; `split_suite_refs` tolerates whitespace so no consumer breaks.

All five pass-1 items scheduled for pass-2 fixes are correctly addressed (comma separator vs. `+`, home-leakage scan scoped to archive+emit only, `CARGO_TARGET_DIR` remap, repo root via `git rev-parse --show-toplevel`, `extractall(filter="data")` with fallback). The two intentional non-changes (`rust_interop_probe` relative fallback and LSP lifecycle-only smoke) are legitimate scope deferrals. **Satisfied for PR readiness.**
