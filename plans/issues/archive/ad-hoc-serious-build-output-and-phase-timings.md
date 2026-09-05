# Ad Hoc Phase: Serious Build Output and Phase Timings

Status: completed (2026-06-14; implementation PR #2555, closure PR #2556)
Owner: unassigned
Context: CLI polish phase for `sifr build` output, timing visibility, and compiler progress reporting

## Problem

`sifr build` currently succeeds with a single generic line:

```text
compiled successfully: ./sifr_output/target/release/sifr_output
```

That line is technically correct but undersells the compiler and gives users no useful sense of what happened. The adjacent `sifr trace` command already exposes deterministic compiler-service internals, but `trace` is debugging-oriented and too detailed for normal build feedback.

The gap is not cosmetic color or icons. The gap is that build output does not communicate:

- what input shape was built: single file, project, or package project
- whether the compiler completed frontend semantic work before native compilation
- which major build boundary failed when failures happen
- how long meaningful phases took
- where the final binary is and, when cheap to measure, how large it is

This phase designs and implements a serious compiler-style build output contract: phase-aware by default, terser under `--quiet`, truthful to actual compiler boundaries, stable in non-interactive contexts, and compatible with existing diagnostic formats.

## Current Implementation Facts

- `sifr build` only accepts `<FILE>`, `-o/--output`, global `--config`, and global `--isolated`; there is no quiet/progress flag today.
- `cmd_build` renders successful builds as `compiled successfully: {binary_path}` on stderr.
- Build execution currently goes through `compile_entrypoint`, resolves single-file vs project mode, builds a rooted entrypoint plan, emits frontend diagnostics, converts the plan to a generated Rust binary project, materializes files, and runs `cargo build --release`.
- Single-file builds parse one source and call `compile_single_frontend_module_with_source_and_options`.
- Project and package-project builds parse an import closure and call `collect_project_hir_source_modules`.
- Materialization writes `Cargo.toml`, `src/main.rs`, support modules, namespace modules, then invokes Cargo.
- `sifr trace` already owns detailed compiler-service trace vocabulary such as parse, lower, type_check, ownership, and flow; normal build output should not duplicate that full debugging surface.

## Design Principles

- **Truth over theater.** Output names must correspond to measured implementation boundaries, not aspirational compiler phases.
- **Useful default.** Successful default output should explain major work boundaries and durations without becoming a trace dump.
- **Quiet when requested.** `--quiet` should suppress phase detail while still leaving a short, useful success result for humans.
- **Diagnostics stay canonical.** Human build progress must not pollute JSON or compact diagnostic output.
- **TTY-aware presentation.** Color and alignment are allowed only when appropriate; output must remain readable with color disabled and when redirected.
- **No symbolic status gimmicks.** Use words such as `Compiling`, `Analyzing`, `Generating`, `Building`, `Finished`, `Binary`, `error`, and `warning`, not emoji or decorative glyphs.
- **Failure-first quality.** Each phase boundary must produce useful context on failure without hiding the existing diagnostic renderer or Cargo stderr.

## Output Streams and Machine Formats

- Human progress lines are emitted to stderr only. Stdout remains reserved for command surfaces that intentionally print program output, generated code, or machine-readable content.
- When `--diagnostic-format` is `json` or `compact`, no human progress lines are emitted on either stream, regardless of `--quiet`. Only diagnostics flow through the diagnostic renderer.
- Human progress text is not a stable public API. Scripts must use `--diagnostic-format=json` or `--diagnostic-format=compact` and parse the structured diagnostic stream instead of grepping `Finished`, `Binary`, or phase labels.
- If color is added, it is enabled only when stderr is a terminal and `NO_COLOR` is not set. Non-TTY output has no ANSI sequences.
- If paths contain whitespace, render them with stable shell-style quoting in human output.

## Proposed User-Facing Contract

Default successful build:

```text
sifr v0.1.0
input:  main.sifr
mode:   project
target: release native

   Loading Sifr standard library          8 ms
   Parsing import closure (4 modules)     3 ms
   Analyzing types, ownership, and flow   12 ms
   Generating Rust project                4 ms
   Materializing Cargo project            1 ms
   Building release binary                26 ms

Finished release build in 54 ms
Binary: ./main
Size:   1.4 MB
```

Quiet successful build:

```text
Finished release build in 54 ms
Binary: ./main
```

When the compiler can cheaply report module counts, prefer count-aware labels:

```text
   Parsing import closure (4 modules)
   Analyzing 4 modules
```

For default output, do not claim detailed sub-phases unless they are separately instrumented. `Analyzing types, ownership, and flow` is acceptable as one frontend semantic boundary only when the timer covers that combined semantic stage. Splitting it into `Checked types`, `Resolved lifetimes`, and `Verified ownership` is not acceptable until those timings are measured independently.

## Decisions

- The flag is `--quiet`. Do not add `--verbose` or `--timings` in this phase; phase timings are part of the default human build output.
- Default successful `sifr build` output includes the phase-aware summary, final duration, binary path, and best-effort binary size.
- Quiet successful `sifr build` output includes only `Finished release build in <duration>` and `Binary: <path>`.
- Human progress is stderr-only. Stdout is untouched by `sifr build` success rendering.
- Binary size appears in default human output only when readable. A size read failure must not fail or warn after a successful build. `--quiet` omits binary size.
- In `--diagnostic-format=json` and `--diagnostic-format=compact`, successful builds emit no human progress lines.
- `sifr run` prints build progress only on cache miss, suppresses the final `Binary:` line because program output follows, and prints no build progress on cache hit. `sifr run --quiet` suppresses build progress even on cache miss.
- `sifr build` continues to materialize into the caller-provided output
  directory and does not use the generated artifact cache in this phase.
- Sifr passes `--quiet` to Cargo for native builds so Cargo progress does not collide with Sifr progress. Cargo/rustc errors remain visible.

## Scope

This phase owns:

- adding the explicit `--quiet` build output mode
- introducing a build progress/timing data model in the driver or CLI boundary
- reporting elapsed time for major build stages
- reporting build mode and target shape in default human output
- reporting binary path in all success modes
- reporting binary size in default human output when the final binary exists and metadata is cheap to read
- preserving current diagnostic behavior for errors and warnings
- documenting output stability rules for scripts and snapshots

This phase does not own:

- changing generated Rust semantics
- changing release/debug compilation policy beyond labeling the current release build accurately
- replacing `sifr trace`
- exposing compiler-service trace internals in normal build output
- changing package manager behavior
- hiding Cargo errors or rewriting rustc diagnostics
- implementing a general telemetry framework

## Implementation Plan

### Wave 0: Output Contract Lock

- Add tests that capture the intended default success output shape.
- Add tests for `--quiet` help text.
- Add accepted fixture baselines for default success, quiet success, JSON-format success with no progress text, compact-format success with no progress text, and `sifr run` cache-hit success.
- Add the scripting-stability statement to user-facing CLI docs.
- Record the `sifr run` cache-hit and cache-miss output contract before touching shared build paths.

Exit criteria:

- The accepted text contract is present in tests or fixture baselines.
- The flag, stream, machine-format, cache, and `sifr run` policies are documented in this issue before implementation proceeds.

### Wave 1: Build Stage Instrumentation

- Introduce a small `BuildReport` / `BuildStageReport` data model in the driver that records:
  - entrypoint path
  - compilation mode: single-file, project, package-project
  - target profile: currently release native
  - binary path
  - optional binary size
  - total elapsed time
  - stage elapsed times
- Instrument real boundaries:
  - stdlib load/compile
  - source parsing/import closure, split from semantic analysis when project mode currently bundles the work too tightly
  - frontend semantic checking and HIR/flow production
  - Rust project generation
  - Cargo project materialization
  - native Cargo build
- Avoid micro-phase claims unless the implementation times them directly.

Exit criteria:

- Build code returns the same binary path as before.
- Timing collection can be disabled or ignored without changing build semantics.
- Internal stage names are stable enough for tests but not overexposed as public API unless explicitly documented.

### Wave 2: CLI Rendering

- Replace the existing `compiled successfully: ...` success line with the new default output.
- Render default phase lines with aligned text in human mode: left-align labels, right-align durations, and compute spacing from the longest label.
- Render `--quiet` success as the two-line quiet contract.
- Keep JSON and compact diagnostic modes free of human progress decorations.
- Make color TTY-aware and respect `NO_COLOR` if color is added.
- Ensure output remains readable without color.
- Pass `--quiet` to Cargo while preserving Cargo/rustc error output.

Exit criteria:

- Default output is bounded and readable.
- Default output is phase-aware and measurable.
- Quiet output is terse and stable enough for human terminal use.
- Non-human diagnostic formats remain parseable and unaffected by progress banners.

### Wave 3: Failure Surface Hardening

- Verify frontend diagnostics still render exactly through the diagnostic renderer.
- Verify Cargo failure output still includes actionable Cargo/rustc stderr.
- Verify Cargo failure output does not gain a redundant `Finished` or `Binary` footer and does not double-render an error summary already present in diagnostics.
- Add tests for failures at:
  - invalid input path or entrypoint resolution
  - frontend diagnostic failure
  - project materialization failure where feasible
  - Cargo build failure where feasible
- Ensure failed builds do not print a misleading final `Finished` or `Binary` line.

Exit criteria:

- Every failure boundary has either existing diagnostics or a targeted build-stage context line.
- No failure path double-renders diagnostics.
- Exit codes remain compatible with the current CLI contract.

## Preferred Text Vocabulary

Use:

- `Compiling <path>`
- `Loading Sifr standard library`
- `Parsing import closure`
- `Analyzing types, ownership, and flow`
- `Generating Rust project`
- `Materializing Cargo project`
- `Building release binary`
- `Finished release build in <duration>`
- `Binary: <path>`
- `Size: <size>`

Avoid:

- `optimized Rust source`, unless Sifr itself adds and measures an optimization pass
- `resolved lifetimes`, unless the compiler exposes that as a real separately measured pass
- `executed native compilation`, because it is less clear than `Building release binary`
- `Compiling` for the native build step, because Cargo also uses `Compiling` for generated crates
- emoji, symbolic checkmarks, and decorative glyphs
- trace-internal labels in ordinary build output
- mixed unit styles such as `42ms` beside `1.4 MB`; use `42 ms` and `1.4 MB`

## Testing and Validation

Minimum focused validation for implementation PRs:

```bash
cargo test -p sifr build_output
cargo run -q -p sifr -- build demos/own_mut_appends/main.sifr
cargo run -q -p sifr -- build --quiet demos/own_mut_appends/main.sifr
cargo run -q -p sifr -- --diagnostic-format json build demos/own_mut_appends/main.sifr
cargo run -q -p sifr -- --diagnostic-format compact build demos/own_mut_appends/main.sifr
cargo run -q -p sifr -- run demos/own_mut_appends/main.sifr
cargo run -q -p sifr -- run --quiet demos/own_mut_appends/main.sifr
cargo run -q -p sifr -- build demos/own_mut_appends/main.sifr 2>&1 | cat
NO_COLOR=1 cargo run -q -p sifr -- build demos/own_mut_appends/main.sifr
cargo run -q -p sifr -- trace demos/own_mut_appends/main.sifr
python3 scripts/check_hir_maintainability_guardrails.py
cargo fmt --check
```

Authoritative pre-PR gate remains:

```bash
scripts/run_all_tests.sh --profile create-pr
```

Merge gate remains:

```bash
scripts/run_all_tests.sh
```

## Review Log

- agent planning review pass 1: requested measured-boundary wording, explicit machine-format suppression, `sifr run` and cache-hit decisions, verbose-only best-effort binary size, Cargo `--quiet` policy, and unit-style cleanup.
- agent planning review pass 2: implementation-ready; only minor wording polish requested for default alignment, verbose Cargo quiet policy, and cached duration semantics.
- User follow-up on 2026-06-14: default human output should be the phase-aware summary; `--quiet` is the terse success mode.
- agent implementation review pass 1: shippable with recommended fixes for
  the cached `sifr build` doc contradiction, stale LeetCode benchmark parser,
  and cached-artifact status cleanup; suggested project-mode and
  warning-success machine-format coverage.
- agent implementation review pass 2: all pass-1 recommendations and
  testing gaps addressed; no further review rounds needed before PR validation.

## Implementation Log

- Implemented `BuildReport` / `BuildStageReport` in `sifr_driver` with measured
  timings for stdlib loading, parsing/import closure, semantic analysis, Rust
  project generation, Cargo project materialization, and release native Cargo
  build.
- Split generated Cargo project materialization from `cargo build --release`
  execution and pass Cargo `--quiet` for native builds.
- Added `sifr build --quiet` and `sifr run --quiet`.
- Replaced the legacy `compiled successfully: ...` line with default human
  phase output and quiet two-line success output.
- Suppressed human success progress for `--diagnostic-format json` and
  `--diagnostic-format compact`.
- Updated `sifr run` output so cache misses show build progress without a
  `Binary:` footer, cache hits are quiet, and the internal artifact-cache status
  line is no longer printed to users.
- Deferred cached `sifr build` output because `sifr build` still writes to the
  caller-provided output directory and does not use the hidden generated
  artifact cache; `sifr run` owns cache-hit reporting in this phase.
- Added process-level tests covering default/quiet/machine-format build output,
  run cache miss/hit behavior, run quiet behavior, frontend failure,
  materialization failure, Cargo invocation failure, project-mode module counts,
  and compact-format warning success without progress text.
- Removed the stale LeetCode benchmark parser for the retired
  `compiled successfully:` line in submodule PR
  https://github.com/sifr-lang/leetcode/pull/39.
- Updated TypeScript-Go M1 direct-read/probe inventory line references after
  touched-file line drift; the M1 guardrail passes again.
- Parent implementation PR: https://github.com/sifr-lang/sifr/pull/2555.

Focused validation run so far:

```bash
cargo test -p sifr build_output --no-fail-fast
cargo test -p sifr run_output --no-fail-fast
cargo test -p sifr failed_ --no-fail-fast
cargo test -p sifr --test build_output_behavior --no-fail-fast
cargo test -p sifr_driver build:: --no-fail-fast
cargo test -p sifr_driver test_cached_project_invalidates_when_workspace_helper_changes --no-fail-fast
cargo test -p sifr -- --skip test_e2e_pass
python3 verification/areas/developer_tooling/check_typescript_go_m1_guardrails.py
cargo fmt --check
cargo clippy --workspace -- -D warnings
scripts/run_all_tests.sh --profile create-pr
```

`scripts/run_all_tests.sh --profile create-pr` passed locally on 2026-06-14.
`scripts/run_all_tests.sh` passed locally on 2026-06-14 after a targeted
performance-budget rerun confirmed the initial representative-budget failure
was transient benchmark noise.
