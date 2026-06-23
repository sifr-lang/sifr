## M4 Sysroot Stdlib Source Layout — Review Pass 2

### Blockers

None.

### Verification summary

- Public sources moved `lib/sifr/*.sifr` to `stdlib/sifr/*.sifr`; private placeholders were added under `stdlib/_sifr/*.sifr`.
- `SysrootPaths` now exposes public/private stdlib source roots and generated stdlib crate paths; sysroot validation enforces both source roots and both runtime/stdlib crate members.
- `load_stdlib_sources_from_sysroot` and `validate_stdlib_source_inventory` enforce public/private source inventory symmetry with missing/stale source tests.
- `compile_stdlib_uncached` resolves a sysroot and loads physical stdlib source files, threading on-disk paths into parser diagnostics.
- CLI `--print sysroot --json` exposes `stdlib_public_sources`, `stdlib_private_sources`, `stdlib_crate`, and `stdlib_crate_manifest`.
- Analysis/LSP stdlib tests continue through the same sysroot-loaded definitions as CLI/driver.
- Python runtime path verification now canonicalizes path-valued interpreter attributes so equivalent Homebrew Cellar and `opt` symlink paths do not fail runtime initialization.
- Touched first-party source files stay under the 900-line guardrail.

### Non-blocking observations

1. `STDLIB_SOURCES.source` still embeds file contents even though runtime now loads physical sysroot files; shrink to module names in a follow-up.
2. `stdlib_source_root` selects repo-vs-installed layout by filesystem probe; storing the selected mode on `ResolvedSysroot` would avoid ambiguity.
3. Add stale-public and duplicate-module tests for inventory parity in a follow-up.
4. `_sifr.io` and `_sifr.test` placeholders should either get architecture notes or be deferred until their public wrappers exist.
5. Add a focused tmp-symlink unit test for `python_path_value_matches`; current end-to-end Python interop validation covers the Homebrew Cellar/`opt` case indirectly.

### Verdict

review-satisfied
