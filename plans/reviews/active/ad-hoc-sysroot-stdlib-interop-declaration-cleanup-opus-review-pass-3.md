All five pass-2 fixes verify cleanly against the current files:

1. **`ellipsis-only` / `package-authored` in docs** — `rg -n 'ellipsis-only|effective compiler-owned no-panic|package-authored' internal_docs/rust_interop_architecture.md internal_docs/sifr_sysroot_and_stdlib_architecture.md docs/rust-interop.mdx` hits all three files: `docs/rust-interop.mdx:54` ("Package-authored… ellipsis-only stub body"), `internal_docs/rust_interop_architecture.md:99` ("ellipsis-only stub body") plus package-authored hits at :570/:614, `internal_docs/sifr_sysroot_and_stdlib_architecture.md:145,154,775`.

2. **M0 `@rust(sifr_stdlib…panic=` scope** — the phase's rg (line 166-167) targets `internal_docs docs plans/issues/active/ad-hoc-sysroot-stdlib-interop-declaration-cleanup.md`, i.e. no archive/review paths. Running that grep against the current tree returns zero matches, so canonical private-stdlib examples are clean.

3. **M4 replacement callout** — phase M4 line 349-351 explicitly says "Replace the existing adapter-policy panic assertion with this guard in the same change," making the invariant reversal visible to PR review.

4. **No implicit panic trust to user packages, `sifr_runtime`, `bridge`, `Self`, arbitrary roots** — anchored in three places: Design Rules line 25-33, M2 task line 250-251 ("Keep `sifr_runtime.*`, `bridge.*`, `Self`, and arbitrary Cargo roots outside the implicit policy boundary"), and Review Checklist line 412-413. The architecture doc echoes the same restriction at line 620-623.

5. **Docs state the durable contract directly** — `rust_interop_architecture.md:99-104`, `docs/rust-interop.mdx:53-56`, and `sifr_sysroot_and_stdlib_architecture.md:145-158/772-778` all read as present-tense declarative contract with no old-vs-new syntax framing.

## Remaining actionable findings

None. The phase and docs are ready for M0 signoff and M1 kickoff.

## Non-actionable residuals (unchanged from prior passes, explicitly deferred)

- `sifr_sysroot_and_stdlib_architecture.md` Pre-Migration Baseline (§ starting :20) still uses migration voice. Pass 1 and pass 2 both flagged this as non-blocking and out of scope for this ad-hoc — a broader stdlib-arch cleanup, not this phase.
- M5 doesn't yet say whether this ad-hoc is a follow-up of the archived `19e346f0f Archive sysroot stdlib toolchain phase` or where closeout evidence should link from `plans/roadmap.md`. Non-blocking closeout item; resolve when populating the M5 Closeout Notes.
