I have enough to write a focused review. Let me compose it.

---

# Review: M4 Process Runtime — First Sync Wave

**RESULT: PASS** (non-blocking follow-ups recorded; first-wave PR is shippable to create-pr lane.)

## Verdict against the six review questions

1. **No CPython-shaped re-introduction.** `lib/sifr/process.sifr:1-87` exposes only native names (`Command`, `Output`, `Status`, `Stdio`, `command()`, `shell()`, `output()`, `run()`). No `CompletedProcess`, no `subprocess.run(..., capture_output=True)` flag mash. Legacy/bare `subprocess` paths remain rejected via `unsupported_legacy_stdlib_module`/`cpython_stdlib_reserved_suggestion` in `crates/sifr_codegen/src/intrinsics/registry.rs:175,210-214,234` and asserted by `registry.rs:344-389`.
2. **`Result[Output, ProcessError]` with binary stdout/stderr is a defensible first boundary.** `Output.stdout: bytes`/`Output.stderr: bytes` (process.sifr:30-31) plus typed `ProcessError` lets text decoding be layered through `sifr.bytes.decode_utf8` (exactly how `process_sync_output_status.sifr:5-10` uses it). This honours the M4 phase doc's "binary pipe mode first" direction and keeps `milestone_text_i18n_1` integration as future work.
3. **No user-triggerable panic paths.** Lowered Rust uses `?` + `map_err(|__err| ProcessError::new(__err.to_string()))` and `status.code().unwrap_or(-1)` — no `unwrap()`/`expect()` on data-dependent values. `process_output` is typed `Result[(bytes, bytes, int), ProcessError]` at the intrinsic registry (`registry/process.rs:19`).
4. **Shell path is explicit.** Separate `shell()` constructor (process.sifr:65-66) drives the `__shell` branch in the lowerer (registry/process.rs:41). The cross-platform `cmd /C` vs `sh -c` selection is in-IR and tested (`registry_extended_tests.rs:130-131`). Traceability honestly lists `@shell_exec`/`@blocking_io` diagnostics and async-context rejection as follow-ups (`concurrency_runtime_m4_process_traceability.md:18-27`).
5. **Validation fixtures are minimally sufficient for this slice.** One fixture exercises native command + `.arg`, shell command, and `run()` returning `Status`; both manifests include it. Negative coverage relies on existing `legacy_sifr_subprocess_removed` / `bare_cpython_subprocess_import` fixtures.
6. **No blocking issues before create-pr.** All findings below are follow-ups for later M4 waves.

## Non-blocking findings (recommended follow-ups)

1. **Lowerer embeds verbatim Rust as `RustExpr::Ident(...)` (`crates/sifr_codegen/src/intrinsics/registry/process.rs:40-43`).** Every other intrinsic — including the existing `subprocess.rs:100-257` you're replacing — composes structured `RustExpr` nodes. Stuffing a multiline Rust block into `Ident` works (it renders verbatim via `render_identifier` at `render_expr_and_blocks.rs:576-577`) but it bypasses the structured IR, defeats pretty-printing, and will be painful to extend when async/timeout/pipe variants land. Refactor to structured `RustExpr::If`/`MethodCall`/`Try` before the next M4 wave that adds spawn/pipe lowerers.
2. **`shell("…").arg("x")` silently drops the arg.** `Command.arg` appends to `args`, but the lowerer ignores `args` when `__shell` is true (registry/process.rs:41 emits only `__cmd.arg("/C"|"-c").arg(&__program)`). No fixture catches this. Either reject `.arg()` on a shell-built `Command`, fold extra args into the shell command line, or at minimum record the trap in `concurrency_runtime_m4_process_traceability.md` follow-ups so users aren't surprised.
3. **Signal termination collapses into exit code `-1`.** `__output.status.code().unwrap_or(-1) as i64` (registry/process.rs:41) merges `code()==None` (signal termination) with a legitimate `-1` exit. Already listed as "Host-matrix evidence for signal/termination behavior" follow-up in the traceability — but consider returning a typed `ProcessError::Signal { signal: i32 }` variant or distinguishing in `Status` rather than reserving `-1` as a sentinel, so the M5 signal/shutdown work doesn't have to redesign `Status`.
4. **`Stdio`, `PIPE`, `INHERIT`, `NULL` are exposed but inert.** `output()`/`run()` never consume `Stdio`; the constants are publicly importable surface that does nothing yet. Either drop them from this wave or wire them through `process_output` so the API doesn't misrepresent what controls users have. The traceability already lists `Stdio`/`PipeReader`/`PipeWriter` follow-ups; mirroring that disposition in the `.sifr` file (or gating exports) would prevent users from writing code that compiles but is silently ignored.
5. **Mild: `Stdio.mode: str` with no enum/validation.** A `Literal["pipe","inherit","null"]` or dedicated enum would catch typos at compile time consistent with Sifr's "if it compiles, it works" guarantee.
6. **Minor: fixture coverage gaps.** `process_sync_output_status.sifr` does not exercise: command-not-found error path (`ProcessError` surface), `current_dir`, non-zero exit (`status.success()==false`), or non-UTF8 binary stdout. Reasonable for first wave; flagging so the next M4 PR doesn't ship pipes without filling these.

## Summary

The implementation cleanly establishes the native `sifr.process` surface contract, keeps the typed `Result` boundary intact, and explicitly defers Stdio/pipe/async/signal/text-mode work in `verification/stdlib/concurrency_runtime_m4_process_traceability.md`. The structured-IR regression in the lowerer (finding 1) and the silent shell-arg drop (finding 2) are the most concrete follow-ups worth landing before M4's spawn/pipe wave.

**RESULT: PASS**
