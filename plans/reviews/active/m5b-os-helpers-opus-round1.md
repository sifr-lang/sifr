Verification complete. All claims check out against the actual diff:

- os.rs registry: retains only `run_command`, `chdir`, `stat_size`, `disk_usage`; the six lowerers are gone, no dangling references anywhere in `crates/`.
- sys.rs: `getpid`, `cpu_count`, `which`, `os_sep`, `os_linesep`, `os_name` all implemented and covered by unit tests.
- sys.sifr: native `@rust(sifr_stdlib.sys.*)` declarations for all six.
- imports: `os.sifr` pulls the six from `_sifr.sys`; `shutil.sifr` pulls `which` from `_sifr.sys`.
- manifest: `exact_intrinsics` tombstoned the six, reason text updated; feature routing adds `sifr.shutil -> {fs, sys}` in both `features.rs` and generated file.
- closure script: all six added to `RETIRED_INTRINSICS`.
- retained sys_fs.rs: the six type entries removed.
- tests: `registry_core_tests`, `stateless_private_codegen_tests`, `features_tests`, and both e2e fixture files updated consistently, including `sifr.shutil -> _sifr.sys` transitive-dep assertions.

One non-blocking note: `shutil.sifr` imports `which` but no function body uses it; it reads as a public re-export (`shutil.which`), and the full `create-pr` suite passed, so it is not a compile issue. Flagging only for awareness.

READY
