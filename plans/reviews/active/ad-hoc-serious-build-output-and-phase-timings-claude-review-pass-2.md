I read the revised issue and the pass-1 review. Going through each pass-1 item against the current text:

## Pass-1 items resolved

1. **Measured-boundary wording (Must-fix #1)** — Resolved. Verbose example now uses `Parsing import closure (4 modules)` + `Analyzing types, ownership, and flow`, and Wave 1 explicitly commits to splitting parsing from semantic analysis at the driver boundary (`crates/sifr_driver` work). The "acceptable as one frontend semantic boundary only when the timer covers that combined semantic stage" gate is the right shape.

2. **Machine-format suppression (Must-fix #2)** — Resolved in the "Output Streams and Machine Formats" subsection: no human progress on either stream when `--diagnostic-format` is `json` or `compact`, regardless of `--verbose`. Wave 0 fixtures explicitly include this.

3. **`sifr run` decision (Must-fix #3)** — Resolved in `Decisions`: progress on cache miss only, no `Binary:` footer, nothing on cache hit. Recorded in Wave 0 before touching shared paths.

4. **Cargo `--quiet` (Should #4)** — Resolved in `Decisions` and Wave 2.

5. **Cache-hit behavior (Should #5)** — Resolved in `Decisions`: `Finished release build in <duration> (cached)` + `Binary: <path>`, no `Compiling` line. `sifr run` cache hit prints nothing.

6. **Verbose-only best-effort size (Should #6)** — Resolved in `Decisions` and Scope: size is verbose-only, omitted on read failure, never warns after a successful build.

7. **Stability disclaimer (Should #7)** — Resolved: "Human progress text is not a stable public API…", with Wave 0 requiring it in CLI docs.

8. **Punctuation/units (Nit #8)** — Resolved: `Binary:` and `Size:` both use colons throughout; SI spacing (`42 ms`, `1.4 MB`) is in the Avoid list.

9. **Alignment math (Nit #9)** — Resolved for verbose: "left-align labels, right-align durations, compute spacing from the longest label" in Wave 2.

10. **PhaseReport struct (Nit #10)** — Resolved: Wave 1 names `BuildReport` / `BuildStageReport`.

## Remaining (non-blocking) wording observations

- **Wave 2 alignment rule covers verbose only.** The default example uses Cargo-style *right-aligned verbs* (`   Compiling`, `    Finished`, `     Binary:` — padded to 12 chars). The stated Wave 2 rule ("left-align labels, right-align durations") describes the verbose block but doesn't say how the default's leading-verb indent is computed. A one-line addendum like "In default mode, leading verbs are right-aligned to the longest verb (Cargo-style)" would close the ambiguity. Not a blocker; reviewers can pin it down in the PR.

- **Verbose Cargo policy uses soft "may".** `Decisions` says "Verbose mode may allow Cargo progress through." Given Pass 1's intent ("drop `--quiet` in `--verbose`"), tightening to "does not pass `--quiet`" removes interpretive room. Minor.

- **Cached `<duration>` semantics.** "`Finished release build in <duration> (cached)`" is unambiguous if `<duration>` is the wall-clock of the command (cache check + I/O). If it's intended to be the *original* compile duration, that needs storing in cache metadata. Worth a half-sentence clarification, but not blocking — implementer will pick the obvious wall-clock reading.

- **Verbose header punctuation.** The verbose top block has `sifr v0.1.0` (no colon) above `input: / mode: / target:` (with colons). Minor visual inconsistency; either drop the header entry or render it as `sifr:   v0.1.0`. Cosmetic.

## Verdict

**Implementation-ready.** All three must-fixes and all four should-fixes from pass 1 are resolved with concrete decisions, not deferred. The remaining items above are wording polish that an implementer can settle in the Wave 0 fixtures PR; none of them change the design, scope, or measured-boundary contract. Proceed to Wave 0.
