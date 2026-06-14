## Verdict

The phase is **implementation-ready**. I cannot find any remaining implementation-blocking ambiguity, contradiction, or missing decision that would force an implementer to invent policy. Sub-wave structure, status semantics, ownership conventions, hermetic/sandbox contracts, CPython oracle policy, red-blocker handling, and closeout gates are all explicit and self-consistent.

## Non-Blocking Polish (optional)

These are wording/clarity issues an implementer can resolve without inventing policy:

1. **`sifr_analysis` row in the cargo packages table (line 286)** says "merge crate test membership required," but per the Verified Facts (line 226), `sifr_analysis` is *not* in the omitted list, so it is presumably already in the hard-coded crate path. Recommend changing the phase decision text to: "already in current hard-coded crate path; keep explicit" — matching the wording used for `sifr_lowering`. Otherwise a reader may add it to the Wave 1 "add the omitted" task list.

2. **CPython exit-code policy (line 765)** says "precise integer equality only for documented exit-code-stable programs," but the Wave 6.0 policy-file checklist (lines 705–736) does not require an exit-code-stability table. Recommend adding to Wave 6.0's policy file requirement: "an enumerated table of exit-code-stable programs, with each program's allowed exit code(s) and rationale."

3. **Generated Rust clippy scope (line 489)** says "run clippy for merge-safe subsets and nightly full subsets" but does not define "merge-safe subset." Recommend adding to Wave 2.1..2.N: "merge-safe clippy subset is defined by an explicit allowlist in `verification/areas/generated_code_quality/data/clippy_merge_lints.json`; lints outside that file are nightly-only."

4. **LSP capability inventory artifact (Wave 9.1)** — the verification target matrix references "documented LSP capability inventory from `crates/sifr_lsp`" (line 137), but Wave 9.1 tasks (lines 908–931) never explicitly add that inventory file. Recommend adding as the first Wave 9.1 task: "Add `verification/areas/developer_tooling/data/lsp_capability_inventory.json`, populated from `crates/sifr_lsp` server-capability advertisement code."

5. **Crash sentinel `expiry` semantics (Wave 5.8, line 672)** — sentinels have `owner` and `expiry`, but the document does not say what happens at expiry. Recommend adding: "at expiry the sentinel is re-triaged: fix the crash, reclassify the surface, or extend the expiry with reason; an expired sentinel fails the regression suite."

6. **`not-applicable` matrix status (line 157)** — useful to include one concrete example so reviewers don't apply it to executable surfaces. Recommend appending: "Example: a data-only crate that ships only static JSON consumed by tests, with no executable code path."

7. **Profile assignment table (lines 88–107)** lacks rows present in the target matrix (CLI exit codes, suggestions/autofix, crash/ICE, distribution/release, incremental/determinism, local/CI parity). All are covered implicitly via other rows or by waves 8/10, but adding a footnote — "rows in the target matrix without an explicit profile row inherit their owning compiler-surface row's profile assignment" — would prevent reviewer confusion.

8. **`closes_in_wave` value space (line 76)** is described as "exactly one wave in 1-9." Recommend adding: "subwaves are expressed via `closes_in_subwave` (e.g., `closes_in_wave: 2`, `closes_in_subwave: final`); the matrix check rejects unknown wave/subwave names."

None of the above prevents starting Wave 0 implementation; they are clarifications the implementer would otherwise have to settle in PR review.
