# Round 4 corrective review — dev-profile per-package `opt-level = 1`

### The diff

Cargo.toml appends two lines to the existing dev-profile per-package block:

```
[profile.dev.package.sifr_lowering]
opt-level = 1
[profile.dev.package.sifr_type_system]
opt-level = 1
```

Nothing else changes in-tree. `insta`/`similar` remain at `opt-level = 3`; `ruff_python_parser` remains at `opt-level = 1` — the pattern the corrective extends. No override of `debug-assertions`, `overflow-checks`, `debug`, `lto`, or `codegen-units`; dev defaults (asserts on, overflow checks on, full debug info) are preserved.

### 1. Sound root fix?

Yes, for the stated symptom. The M1–M6 rearchitecture puts real source stdlib compilation on every debug CLI invocation (M5 deleted the fallback shortcut). Profiling identified `sifr_lowering` and, adjacently, `sifr_type_system` as the dominant CPU cost. Raising those two packages to `opt-level = 1` — while leaving the rest of the workspace at `0` — is the exact precedent already used for `ruff_python_parser`, which is the corresponding parser hot-crate on the same CLI path. The observed ~1.11 s medians vs 1.33–1.36 s absolute budgets (roughly 20 % headroom) are consistent with the typical debug→`opt-level=1` gain for clone-heavy compiler code.

The deeper alternative "root fix" (invalidation-safe stdlib cache) is a distinct architectural workstream, out of the M1–M6 boundary phase. Refusing to compile the stdlib each time is not what this phase closed on; making the compiled Rust code that does the compilation reasonably fast in dev is a legitimate targeted response, not a threshold-shift or waiver.

### 2. Could this mask correctness bugs or hurt test/compile-time validation?

No material risk.
- `opt-level = 1` does not disable `debug_assertions` or `overflow-checks`; those flags are independent, and the dev profile default (`true`, `true`) is retained. All `assert!` invariants and integer-overflow traps continue to fire in tests and in the CLI itself.
- Debug info generation is controlled by `debug`, not `opt-level`; backtraces and line numbers in test failures are unchanged.
- Level-1 LLVM passes are conservative (mem2reg, basic inlining, dead-code elim). They don't reorder observable side effects and don't skip UB checks that weren't already skipped at level 0. There is no plausible mechanism by which a real bug present at `opt-level=0` would silently disappear at `opt-level=1` — you'd need UB in Sifr's own Rust code, which clippy/pedantic and `unsafe_code = warn` already gate against.
- Compile-time impact is confined to two crates (plus the already-configured `ruff_python_parser`). Because these crates are large, expect a small absolute increase in `cargo build` time; but they're rebuilt only when their sources change, and the corrective is scoped narrowly rather than blanket-optimizing the workspace.
- No test binary depends on `opt-level=0` behavior of these two crates for its assertions.

### 3. Are additional tests or docs required before the full merge gate?

No.
- The change is a build-config knob with no semantic surface; the existing merge gate is the correct validation and it's exactly what's outstanding on the phase checklist.
- The Cargo.toml block is self-documenting via the `ruff_python_parser` precedent immediately above it; no comment is needed to convey intent.
- A brief note in the phase closure record (`plans/reviews/active/…-round4-performance-corrective.md`) capturing what changed and what evidence justified it is worthwhile bookkeeping — but the review artifact is already scaffolded (0-byte placeholder) and is a closure task, not a merge-gate prerequisite.

### 4. Blockers

None. The change is narrow, follows an in-tree precedent, cannot mask correctness bugs given the retained dev-profile invariants, and demonstrably resolves the absolute debug-CLI performance regression with substantial headroom. Local validations (isolated `-p sifr` build, focused 3-case benchmark run + budget checker, `fmt --check`, `diff --check`) all pass on this exact diff. The only remaining step — running the full unchanged merge gate — is the last checklist item on `M6` and on the phase issue.

VERDICT: SATISFIED
