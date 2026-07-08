READY.

Verified the migration is coherent and self-consistent:

- Boundary is clean. `env_*`, `get_args`, `sys_exit`, `sys_version`, `sys_platform`, `sys_maxsize` were removed from the retained registry (`env.rs`/`sys.rs` deleted) and now route through `@rust(sifr_stdlib.sys.*)` declarations in `stdlib/_sifr/sys.sifr` backed by `crates/sifr_stdlib/src/sys.rs`.
- Retained set is correctly narrowed in `stdlib_retained_compiler_intrinsics.toml`: only the later helpers (`chdir`, `cpu_count`, `disk_usage`, `getpid`, `os_linesep/name/sep`, `run_command`, `stat_size`, `which`) remain, `registry_files` trimmed to `os.rs`, and the reason string documents the deferral.
- Semantics preserved. Invalid-key noop policy, NUL/`=` guards, and `SifrIntBridge` saturating conversion carried over intact, with unit tests covering both env and sys value paths.
- Signatures match between the `.sifr` declarations and the Rust functions (return types, arity).
- Manifest/features (`generated_stdlib_features.rs`, `features_tests.rs`, `feature_contract.rs`) and closure script updated in lockstep; create-pr profile passed.

No blockers.
