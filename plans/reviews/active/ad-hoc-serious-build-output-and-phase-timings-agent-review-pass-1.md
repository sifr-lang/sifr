Here is the structured planning review. I have not modified any files.

# Review: Serious Build Output and Phase Timings

**Verdict:** Approve with revisions. The phase is well-shaped, the vocabulary is honest, and the scope/anti-scope split is clean. Three issues need to be fixed before implementation: (1) the verbose breakdown promises a `Checking types, ownership, and flow` timing that the driver cannot independently measure today, (2) the contract is silent on stdout vs stderr and `--diagnostic-format json|compact` even though the success line currently bypasses the renderer, and (3) `sifr run` is left as an open question even though it shares the exact same build path and will visibly inherit the new output. A handful of smaller tightening edits below.

## 1. Truthfulness and elegance of wording

**Strengths.** `Compiling`, `Finished`, `Binary`, `Loading Sifr standard library`, `Generating Rust project`, `Building release binary` are all real boundaries in the current driver (`RootedEntrypointPlan::from_entrypoint` in `crates/sifr_driver/src/build/entrypoint.rs`, plus materialization and `cargo build --release`). The "Preferred Text Vocabulary" / "Avoid" section is the strongest part of the issue — it directly heads off the most common failure mode (aspirational phase names).

**Problem — `Checking types, ownership, and flow` is not a real boundary today.** Frontend lowering, typecheck, ownership, and flow are all bundled inside `collect_project_hir_source_modules` (and the single-file equivalent). The issue tries to defend itself with "Splitting into `Checked types`, `Resolved lifetimes`… is not acceptable until those timings are measured independently" — but the *combined* label is also a claim, and it is mixed in with `Parsing source modules` which today is *also* inside the same call for the project path. Either:

  - Coalesce parsing+frontend into one line, e.g. `Analyzing 4 modules` or `Checking 4 modules`, until parsing is split out at the driver boundary, **or**
  - Add a Wave 1 subtask that explicitly splits the parse step from the lower/typecheck/ownership/flow step so the two lines reflect reality.

Pick one and commit. Right now the verbose example implies two timings that share a single timer.

**Minor wording polish:**
- `Finished release build in 42ms` reads slightly awkward. Cargo says `Finished \`release\` profile [optimized]`. I'd use `Finished release build (53ms)` or `Finished release build in 53 ms` (space before unit per SI; Cargo does this too).
- `Binary ./main` is fine, but `Binary: ./main` (with colon, as in the verbose block) reads better and matches the other key-value lines. Pick one style for both default and verbose so the default isn't a one-off.
- "Size" without a colon in the vocabulary list but `Size:   1.4 MB` in the example. Same colon-or-not decision needed.

## 2. Phase scope — implementation-ready?

Scope split is good. One overreach and one underreach:

**Overreach — binary size.** "Reporting binary size when the final binary exists and metadata is cheap to read" sounds cheap but `std::fs::metadata` after `cargo build --release` is racy on some filesystems and on package builds the artifact isn't always where the driver thinks (it's resolved through `cargo metadata`/target dir). Move binary size to verbose-only and make it tolerant of "size unavailable" — do not let a size read failure regress the success path. The "Open Decisions" item already hints at this; resolve it as **verbose only, best-effort, omit on error**.

**Underreach — `sifr run`.** `cmd_run_file` and the package equivalent go through the exact same `build_cached_*_binary` driver functions. That means the new output appears in `sifr run` automatically. The Open Decisions question "Should `sifr run` reuse the same build report?" is really three questions:

  - Does `sifr run` print the build report at all? (If not, you need to *suppress* it, which is a new requirement.)
  - If yes, does it print on cache hit, or only on cache miss?
  - Does it print the `Finished` / `Binary` footer when the user's intent is "run", or only `Compiling` while building and then nothing?

Decide this in Wave 0 alongside the flag policy, not as an open decision. A reasonable default: `sifr run` shows progress only when it actually compiles (cache miss), suppresses the final `Binary` line (since the program output follows immediately), and never shows it on cache hit.

## 3. Implementation waves and validation gates

The waves are reasonable but a few gates are thin.

**Wave 0 gap — output-stream contract.** The wave mentions deciding "stderr only vs split between stderr and stdout" and "the contract for `--diagnostic-format=json` and `--diagnostic-format=compact`," but the *current* implementation prints `compiled successfully` directly to stderr without going through `render_diagnostics`. That means today, JSON consumers piping `--diagnostic-format json` already get a stray human line on stderr. The new contract should *explicitly fix this regression* rather than preserve it. Recommend:

  - In `--diagnostic-format=json` or `--diagnostic-format=compact`, **suppress all human progress lines entirely** on both streams, regardless of `--verbose`. The flag is the user saying "I am a machine consumer."
  - Add a Wave 0 acceptance test: `sifr build --diagnostic-format json demos/own_mut_appends/main.sifr 2>/tmp/err >/tmp/out` → `/tmp/err` contains only valid JSON (or is empty if no diagnostics), `/tmp/out` is empty.

**Wave 1 gap — timer model and lock-step with vocabulary.** "Avoid micro-phase claims unless the implementation times them directly" is the right principle but the wave doesn't say *how* timing is recorded. A small concrete subtask: introduce a `PhaseTimer`/`PhaseReport` in `sifr_driver` that the existing `RootedEntrypointPlan::from_entrypoint` updates at the four real boundaries (stdlib, parse closure, frontend, codegen) plus materialization and Cargo. The CLI renders from that report. Without this, Wave 2 implementers will reach into the driver ad-hoc.

**Wave 2 gap — alignment math.** "Aligned text in human mode" is doing a lot of work. State the rule: right-align durations, left-align labels, fixed gap, computed from the longest label. Otherwise reviewers will argue about it in PR.

**Wave 3 gap — failure not double-rendered.** Good gate, but missing a concrete case: when `cargo build` fails, what does Sifr print after Cargo's stderr? Today nothing, because we abort before `compiled successfully`. After the change, ensure there is no `Finished` / `Binary` epilogue and that we don't add a redundant `error: cargo build failed` on top of Cargo's existing message. Add an explicit test for this.

**Validation list — missing items.**
- Add a test/check for the `NO_COLOR` env var path (Wave 2 says "respect `NO_COLOR` if color is added" but `if` should be "yes" if any color is added at all; otherwise drop color from the contract).
- Add `cargo run -q -p sifr -- build --verbose demos/own_mut_appends/main.sifr 2>&1 | cat` to confirm output is sane when stderr is redirected (no ANSI residue, no progress spinner).
- Add `sifr run` to the validation list since it inherits this output.
- Add a Cargo-failure fixture (e.g., a demo crafted to trigger codegen of code that rustc rejects) so the failure surface is tested end-to-end, not just in unit tests.

## 4. Cross-cutting concerns the issue under-addresses

**TTY/color.** "Color and alignment are allowed only when appropriate" is vague. Recommended explicit rule: detect `stderr.is_terminal()`; if `NO_COLOR` is set or stderr is not a TTY, emit no ANSI. If colors are used, color only the leading verb (`Compiling`, `Finished`, `Binary`), Cargo-style — never the path or duration.

**Scripting stability.** The issue says "stable in non-interactive contexts" but does not commit to a stability promise. Recommend wording: *"Human progress output is best-effort and may change between versions. Scripts must use `--diagnostic-format=json|compact` and parse the structured stream; do not grep `Finished` or `Binary`."* Otherwise you will be locked into the text forever after the first user pipeline.

**Cargo interop.** Two real concerns the issue misses:

  - **Cargo already prints `Compiling sifr_output v0.1.0`.** If we don't suppress Cargo's progress (we shouldn't), the user sees Sifr's `Compiling main.sifr` and Cargo's `Compiling sifr_output v0.1.0` interleaved on stderr. Decide: pass `--quiet` to `cargo build --release` by default in non-verbose mode (Cargo still prints errors with `--quiet`), and drop `--quiet` in `--verbose`. Document this in Wave 2.
  - **Cache hits.** When the cached binary is reused (no rebuild), what does the output look like? Today `compiled successfully:` still prints. Under the new contract, do we still print `Compiling main.sifr`+`Finished release build` for a no-op? Better: print only `Finished` with a duration near zero (or a `(cached)` suffix). Add to Open Decisions or, better, decide now.

**Diagnostic-format JSON.** Already covered above — promote it from "record the contract" to "machine-format suppresses human progress, full stop."

**Path quoting.** `Binary ./main` is fine until the binary path contains spaces. Decide whether to quote (`Binary "./My App"`) — recommend yes when the path contains whitespace, never otherwise.

## 5. Concrete edits to the issue text

Below are the most useful targeted replacements. I have not applied them — only proposed.

**Replace the second verbose-output block (lines 65–74)** with a version that reflects current driver granularity:

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

Rationale: matches the six measurable boundaries in `RootedEntrypointPlan::from_entrypoint` + materialize + cargo, swaps the misleading `Parsing source modules` for the actual `parse_import_closure_source_modules` boundary, uses `Analyzing` instead of `Checking` to avoid colliding with Cargo's `Checking` (which means `cargo check`).

**Add a new subsection after "Design Principles"** named **"Output Streams and Machine Formats"**:

> - Human progress lines are emitted to stderr only. Stdout is reserved for machine-readable surfaces.
> - When `--diagnostic-format` is `json` or `compact`, no human progress lines are emitted on either stream regardless of `--verbose`. Only diagnostics flow through the renderer.
> - Human progress text is not a stable public API. Scripts must consume `--diagnostic-format=json|compact`. This stability rule is documented in the user-facing CLI docs.

**Replace the "Open Decisions" section** with concrete decisions:

> ## Decisions
> - Flag is `--verbose` (single flag). `--timings` is not added; verbose mode already shows timings.
> - Default output includes `Compiling <path>`, `Finished …`, and `Binary …`. Three lines, all on stderr.
> - Successful build progress is stderr-only. Stdout is untouched.
> - Binary size appears only in `--verbose`, and only when readable; missing size is silently omitted.
> - `sifr run` prints `Compiling <path>` on cache miss only, suppresses the final `Binary` line (program output follows immediately), and prints nothing on cache hit.
> - Cache hits in `sifr build` print a single `Finished release build (cached)` line and the `Binary` line; no `Compiling` line.
> - In non-verbose mode the driver passes `--quiet` to `cargo build --release`; in verbose mode it does not.

**Tighten Wave 0 exit criteria** to:

> - Wave 0 PR includes accepted fixture baselines for: default success, verbose success, JSON-format build (no progress text on either stream), compact-format build (same), and cache-hit success.
> - Stability statement is added to the user-facing `sifr build` CLI docs in the same PR.

**Add to Wave 3** an explicit failure-output rule:

> - On Cargo failure, Sifr emits no additional summary line and no `Finished` / `Binary` footer; Cargo's stderr is passed through unchanged. A single Sifr line of context is allowed only when it points to the failing phase (e.g., `error: failed during: Generating Rust project`) and is not duplicated by an existing diagnostic.

**Append to "Avoid" list:**

> - `Compiling` for the native build step — that label belongs to the Sifr source step, since Cargo also prints `Compiling` for the generated crate and we should not collide.
> - Mixed unit styles (`42ms` vs `1.4 MB`). Use SI: `42 ms`, `1.4 MB`, space between number and unit.

## Findings summary

| # | Severity | Finding |
|---|----------|---------|
| 1 | **Must-fix** | `Checking types, ownership, and flow` and `Parsing source modules` cannot both be timed independently today. Either split the driver boundary or coalesce the labels. |
| 2 | **Must-fix** | `--diagnostic-format=json\|compact` interaction is left as "to be recorded"; today the success line leaks plain text into JSON consumers. Specify: machine formats suppress all human progress. |
| 3 | **Must-fix** | `sifr run` shares the build path verbatim. Decide and document its output behavior in Wave 0, not as an open decision. |
| 4 | Should-fix | Cargo's own `Compiling sifr_output` will appear alongside Sifr's progress. Pass `cargo build --quiet` by default; drop in `--verbose`. |
| 5 | Should-fix | Cache-hit behavior is undefined for both `sifr build` and `sifr run`. Specify a `(cached)` variant. |
| 6 | Should-fix | Binary size read is best-effort and verbose-only; cannot regress the success path on read failure. |
| 7 | Should-fix | Add stability disclaimer: human progress is not a stable API; scripts use `--diagnostic-format`. |
| 8 | Nit | Inconsistent punctuation: `Binary ./main` vs `Binary: ./main`; `42ms` vs `1.4 MB`. Pick one and apply everywhere. |
| 9 | Nit | "Aligned text" needs a stated rule (right-align durations, computed gap). |
| 10 | Nit | Wave 1 should name a concrete `PhaseReport` struct in `sifr_driver` so the CLI doesn't reach into private internals. |

Net assessment: the issue is one revision pass away from being implementation-ready. The current text is already markedly better than the existing `compiled successfully:` line and the design principles section is genuinely good — just needs the three must-fix items resolved and the open-decisions list converted into actual decisions before Wave 0 starts.
