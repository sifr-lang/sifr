# M4 Sync stdout/stderr Pipe Readers — Review Pass 2 (post-rebase)

Verdict: **PASS**

Scope under review: post-rebase working-tree diff against `origin/main` on branch
`codex/concurrency-runtime-m4-owned-pipes`, including the unstaged ledger update in
`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`. The
rebase landed PR #2348 stdin append semantics into the base; this review confirms
no regression of that evidence and no widening of the sync stdout/stderr pipe
readers wave invariants verified in pass 1.

## Rebase and conflict correctness

- Base now contains PR #2348 (`afffaa3f8e40b9af0bbdffe13bafb61e053afb03`). The
  ledger's "M4 stdin append semantics evidence" block (lines 740–759) is intact
  with its implementation, targeted local validation, review-loop, and merge
  link to PR #2348 preserved verbatim relative to the merged content.
- The pipe-readers wave evidence (lines 761–779) follows the stdin append block
  without disturbing it. Both waves coexist in the ledger; no conflict markers,
  duplicated headings, or stale "in progress" claims about PR #2348.
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md` status
  line lists, in order: PR #2331, #2334, #2336, #2337, #2341, #2344, #2345,
  PR #2348 stdin append, and "sync stdout/stderr pipe readers are in progress".
  Traceability table additions for `Command`, `Stdio`, `PipeReader`, and
  `Sync spawn/wait/Child.wait` add `process_spawn_pipe_readers` without
  reordering or dropping existing rows. Validation lanes (Create PR / Merge)
  add `process_spawn_pipe_readers` between `process_spawn_wait_status` and
  `process_timeout_status`. Follow-up boundaries replace the old "PipeReader,
  PipeWriter, owned stdout/stderr/stdin" bullet with the narrowed "PipeWriter,
  owned stdin pipe lifecycle, streaming PipeReader reads beyond one-shot
  read_all" follow-up, preserving the remaining open work.
- `verification/platform/supported_host_matrix.md` keeps the new "Sync subprocess
  stdout/stderr pipe readers" row (macOS arm64 / Linux x86_64 supported,
  Windows x86_64 host-limited pending a deterministic fixture) and no other
  process row regressed.
- `verification/validation_lanes/create_pr_e2e_manifest.json` and
  `merge_e2e_manifest.json` add `process_spawn_pipe_readers` immediately after
  `process_spawn_wait_status`; both parse clean with `python3 -m json.tool`.
- Ledger top-section "Implementation PRs" recap (line 416–422) adds the one
  new entry "M4 sync stdout/stderr pipe readers: pending PR." after
  PR #2345. PR #2348 and PR #2344 remain absent from this recap, matching
  `origin/main` (those evidence-only waves were not listed there either).
  Not a regression introduced by this branch.
- Ledger post-rebase rerun is recorded (`scripts/run_all_tests.sh
  --profile create-pr` -> PASS, `report_signature=559a90cf856fe902`,
  warm wall-time advisory `232.37s`, platform golden `pass=5 skip=2`,
  e2e `98 passed / 0 failed`, `cache_hits=25/26`) and matches the user's
  stated post-rebase numbers exactly.

## Original wave invariants reverified

- **Sync-only stdout/stderr pipe readers, not full communicate or async pipes.**
  No new async pipe surfaces, no `communicate`, no async `Child` /
  `PipeReader`. The pipe table is `std::sync::Mutex` over
  `Box<dyn std::io::Read + Send>` (not Tokio async); `__sifr_process_spawn`
  uses `std::process::Command`; no tokio `process` imports leak into the
  sync path.
- **`Command.stdout(mode: Stdio)` / `Command.stderr(mode: Stdio)`, default
  inherit.** `lib/sifr/process.sifr:157–158, 178–182` initialize
  `stdout_mode` / `stderr_mode` to `"inherit"` and overwrite via
  `mode.mode + ""`. Existing callers that never invoke `Command.stdout` /
  `Command.stderr` continue to inherit, matching prior behavior.
- **`Child.stdout()` / `Child.stderr()` transfer one-shot `PipeReader`
  handles.** `lib/sifr/process.sifr:108–122` returns `PipeReader(handle)`
  through `process_child_stdout` / `process_child_stderr`. The generated
  body in `crates/sifr_codegen/src/preamble/process_child_pipes.rs:86–176`
  uses `Option::take()` on the child's `stdout` / `stderr` field, returning
  a typed `ProcessError { message: "process … pipe is not available or
  already taken for child handle: …" }` on the second call.
- **`PipeReader.read_all()` reads bytes once and typed-errors on double
  read.** `lib/sifr/process.sifr:77–82` flips `_closed = True` *before*
  delegating to `process_pipe_read_all`, so any later call raises
  `ProcessError("process pipe reader is already closed")`. The generated
  `__sifr_process_pipe_read_all` (`process_child_pipes.rs:178–263`) removes
  the boxed reader inside a scoped `__pipes` block (mutex guard dropped
  before the IO call) and reads into a local `Vec<u8>` via
  `std::io::Read::read_to_end` with `map_err` into `ProcessError`.
- **Generated `__sifr_process_spawn`, `__sifr_process_stdio_from_mode`,
  `__SIFR_PROCESS_PIPE_READERS`, child stdout/stderr extraction, and
  `read_all` helper are panic-free on data paths and use typed
  `ProcessError`.** Verified by grepping the emitted Rust for
  `panic!|unwrap()|expect(`. Only matches are the structural
  `unreachable!("sifr try/except return capture fell through")` markers
  emitted by the compiler's try/except return-capture lowering — control
  flow that is statically unreachable, not data-dependent. The new
  helpers use `?` propagation with `process_map_err` and `ok_or_else` for
  every fallible call. The poisoned-lock recovery in
  `poisoned_lock_expr` (`process_child_pipes.rs:31–52`) uses
  `unwrap_or_else(|err| err.into_inner())`, so even a poisoned mutex
  yields the inner data rather than panicking.
- **Ordinary `Command.output()` does not emit child/pipe table helpers.**
  Reproduced locally:
  `cargo run -q -p sifr -- emit
   crates/sifr/tests/e2e/pass/process_sync_output_text.sifr | grep -E
   '__sifr_process_spawn|__sifr_process_stdio_from_mode|__SIFR_PROCESS_CHILDREN|__SIFR_PROCESS_PIPE_READERS|__sifr_process_child_stdout|__sifr_process_child_stderr|__sifr_process_pipe_read_all'`
  returns no matches. The filter gates in
  `crates/sifr_codegen/src/stdlib_filter/implementation.rs:282–328,
  351–367, 388–397` correctly opt programs in to the shared process child
  bundle only when they reference one of the spawn / pipe helpers.
- **Docs do not overclaim stdin `PipeWriter`, streaming reads, async
  pipes/communicate, timeout/cancellation/scoped supervision, or Windows
  support.** `PipeReader` row in traceability explicitly preserves
  "stdin `PipeWriter`, streaming reads, async pipes, and sendability/
  shareability checks remain later M4 work". Follow-up boundaries
  retain bullets for `PipeWriter` + owned stdin lifecycle, async
  spawn/wait/communicate, graceful terminate / non-Unix signal status,
  scoped `TaskGroup.spawn_process`, method-form `@blocking_io`, and
  Windows. Supported host matrix row marks Windows `host-limited`.
- **Traceability docs retain PR #2348 stdin append semantics while adding
  `process_spawn_pipe_readers` coverage.** Status line lists PR #2348
  stdin append between PR #2345 and the in-progress pipe readers wave;
  `Command` row keeps stdin append wording ("Repeated `stdin_bytes(...)`
  calls append in call order"); CPython mapping entry preserves stdin
  semantics references; Create-PR / Merge lanes carry the existing
  process fixtures plus `process_spawn_pipe_readers`.

## Local re-verification

- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr
  --quiet` -> clean (no output).
- `cargo fmt --check` -> clean.
- `python3 scripts/check_file_size_guardrails.py` -> PASS (2180 files,
  900-line cap). `process_runtime.rs` 853, `process_child_pipes.rs` 263,
  `registry/process.rs` 897, `registry.rs` 712, `stdlib_filter/
  implementation.rs` 722, `process.sifr` 413, `sifr_stdlib/process.rs`
  264 — all under the cap. Pass-1's standing note that
  `registry/process.rs` sits at 897/900 still applies and is still
  out-of-scope for this wave.
- `python3 -m json.tool verification/validation_lanes/
  create_pr_e2e_manifest.json` and `merge_e2e_manifest.json` -> PASS
  (both contain `process_spawn_pipe_readers`).
- `cargo run -q -p sifr -- run
  crates/sifr/tests/e2e/pass/process_spawn_pipe_readers.sifr` -> PASS
  (cache hit on the post-rebase run; no assertions fail; produces both
  `out` and `err`, double-read raises `"already closed"`, double child
  extraction raises `"already taken"`, status `kind=="success"`).
- Emit grep for `__sifr_process_spawn|__sifr_process_stdio_from_mode|
  __SIFR_PROCESS_CHILDREN|__SIFR_PROCESS_PIPE_READERS|
  __sifr_process_child_stdout|__sifr_process_child_stderr|
  __sifr_process_pipe_read_all|std::io::Read::read_to_end|ProcessError`
  on `process_spawn_pipe_readers.sifr` returns every expected helper
  with typed-error message bodies.

## Outstanding pass-1 non-blocking notes

Carried forward unchanged, none turning blocking after the rebase:
helper duplication across `preamble/process_runtime.rs`,
`preamble/process_child_pipes.rs`, and
`intrinsics/registry/process.rs`; `registry/process.rs` at 897/900;
`Command.stdin_bytes(...)` silently inert for `spawn`; `Stdio` mode
validation deferred to spawn; pipe table emitted for any `Child`-using
program (now also any program that calls `spawn`, since `spawn` is
keyed off the child handle table).

## Coverage attestation

The rebase preserved the stdin append semantics evidence (PR #2348) and
added the sync stdout/stderr pipe readers evidence without conflict or
overclaim. All original wave invariants verified in pass 1 still hold.
Generated code is panic-free on data paths, uses typed `ProcessError`,
keeps the pipe table behind a private gated bundle, and leaves async
pipes / communicate / streaming reads / timeout / cancellation / scoped
supervision / Windows explicitly open in docs and follow-ups.

**PASS** — ready to merge as the M4 sync stdout/stderr pipe readers
wave.
