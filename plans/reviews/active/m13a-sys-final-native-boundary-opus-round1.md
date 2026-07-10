I have enough to review. One concrete defect surfaced during verification.

**Verdict: BLOCKED**

## Findings (severity-ordered)

### 1. [Blocker] `check_stdlib_migration_closure.py --self-test` now fails
`scripts/check_stdlib_migration_closure.py:517` uses `"run_command" => lower_run_command(args),` as the fixture for the "retained intrinsic seed should pass" branch. This diff moves `run_command` into `RETIRED_INTRINSICS` (line 281), so the self-test's own seed is now considered a resurrected retired intrinsic and the branch reports failure:

```
$ python3 scripts/check_stdlib_migration_closure.py --self-test
self-test retained intrinsic seed should pass
exit=1
```

The main-path validation still passes, so CI-visible guards are green - but a milestone whose whole point is to close out sys/fs is landing with the closure guard's own self-test broken. Swap the seed to a still-retained intrinsic that actually appears in `registry.rs` today (e.g. `"builtin_open" => file_handles::lower_builtin_open(args),`).

### 2. [Non-blocking] fs codegen test doesn't guard the new SifrIntBridge glue
`crates/sifr_driver/src/stdlib/stateless_fs_codegen_tests.rs:19-61` only asserts `sifr_stdlib::fs::{name}(` appears for each migrated function. It does not assert the bridge-int conversions:
- `stat_size` should produce `.map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())` (or equivalent)
- `disk_usage` should produce `.into_iter().map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating()).collect()` for `Vec<SifrIntBridge> -> list[int]`

Peer tests (`calendar_private_...` line 156-161, `i18n_private_...` line 812-814, `url_private_...` line 337) assert this explicitly. Adding one line each would prevent a future regression where the bridge glue silently disappears while the surface still compiles.

### 3. [Non-blocking] `legacy_subprocess_intrinsics_are_not_registered` name no longer reflects assertion
`crates/sifr_retained_intrinsics/src/lib.rs:129` used to check specific `subprocess_*` names in the `_sifr.sys` module. After this diff it just asserts `_sifr.sys` isn't registered at all. Rename to something like `legacy_sys_fallback_module_is_not_registered` (and drop "subprocess", since the removal covers `run_command/chdir/stat_size/disk_usage`, not subprocess helpers).

## Answers to the five review questions

1. **Removes final compiler-owned sys/fs public behavior?** Yes - `registry.rs` dispatch entries gone, `registry/os.rs` and `sys_fs.rs` deleted, `get_intrinsic_module` no longer maps `_sifr.sys`/`_sifr.fs`, and closure-guard `RETIRED_INTRINSICS` covers all four names. `_sifr.fs` retention is scoped correctly to `builtin_open`/`builtin_open_text` bridge glue only.

2. **`fs = ["dep:sifr_runtime"]` justified?** Yes. `stat_size` returns `Result<SifrIntBridge, io::Error>` and `disk_usage` returns `Vec<SifrIntBridge>`, matching the established stdlib int-bridge pattern in `random.rs`, `net.rs`, `crypto.sifr`, etc. The new impl is also strictly safer than the deleted lowering: `stat_size` uses `saturating_i64` instead of `as i64` (metadata.len() is `u64`), and `disk_usage` uses `saturating_mul(1024)` instead of `* 1024`.

3. **Manifest state transitions accurate?** Yes.
   - `_sifr.sys` retained -> closing: correct (row exists only to record the shared `os.rs` closure; removal_criteria matches).
   - `_sifr.fs` retained -> retained-by-design: correct - `builtin_open`/`builtin_open_text` are permanent bridge-glue.
   - `mixed-io-logging-random-preamble` retained -> retained-by-design: correct - only IOError + file-handle glue remain.

4. **Missing tests/guards?** See findings 1-3. Finding 1 is the only blocker; 2-3 are polish.

5. **Behavior parity with deleted `os.rs`?**
   - `run_command`: identical (`sh -c cmd -> trim(utf8_lossy(stdout))`, `?` bubbles the same `io::Error`).
   - `chdir`: identical.
   - `stat_size`: same happy path, strictly safer overflow handling.
   - `disk_usage`: same shape (missing metadata / df failure / <4 parts -> `[0,0,0]`; parses `df -k` field indices 1/2/3 x 1024), strictly safer arithmetic.

## Non-blocking notes
- The `sys.rs` `run_command` unit test (`printf 'sifr\n'`) is minimal but adequate for the trimmed-stdout invariant; consider a second case that exercises a non-zero exit or non-UTF-8 stdout if you want defense against future regressions.
- The `disk_usage` test only proves `.len() == 3` on a missing path (zero_usage). It doesn't cover the happy path (real path, four-field df output). Given `df -k` is host-dependent, that omission is defensible.
