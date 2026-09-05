# M7 Review — Pass 3

Branch: `sifr-sysroot-stdlib-m7-lsp-tooling`
Reviewer: agent
Scope: Verify pass-2 remaining blockers B1 and B2 are resolved via explicit M7b deferral.

## Verdict: PASS

The remaining pass-2 blockers are now resolved by formal deferral. M7's scope, acceptance, and validation are consistent with what the code actually delivers; the deferred work has a named follow-up milestone (M7b) with concrete tasks, acceptance, and validation lines.

## Pass-2 required-to-close items

| # | Requirement | Pass 3 status | Evidence |
|---|---|---|---|
| 1 | Implement OR explicitly defer sysroot mismatch/broken diagnostic flow | **deferred** | `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:438-442, 482-512` — M7 prose names the deferral; M7b lists proactive diagnostics, mismatch comparison, observed paths, and development-sysroot LSP tests in tasks/acceptance/validation |
| 2 | Tag `GeneratedSupport`/`CompilerSynthetic` in production OR defer | **deferred** | `:452-453, 467-469, 495-496, 504-505` — M7 reserves the origin kinds; M7b owns production emission and assertions |
| 3 | Update issue plan M7 line to reflect actual scope | **done** | `:18-19` — M7 row scoped to source/navigation with explicit M7b call-out; `:438-442` section header renamed to "Source/Navigation Integration" with deferral preamble |
| 4 (optional) | Cache stdlib bucket, surface observed paths, on-disk hover test | not addressed | Pass-2 marked these as non-blocking |

## Verified consistency between code, plan, and architecture

- M7 acceptance bullet for source-map origins now reads "reserve generated/synthetic origin kinds for M7b production emission" (plan `:467-469`) — matches the diff state where those variants exist but only via a constructed `SourceMapView` test. No longer a contradiction.
- M7 no longer claims diagnostic publishing; the inspectable `sifr/sysroot` query is the only acceptance bullet (`:466`), which matches the existing `requests/mod.rs` arm and `sysroot_request_tests.rs` coverage.
- Architecture doc (`internal_docs/sifr_sysroot_and_stdlib_architecture.md:664-666`) records the M7/M7b origin-emission split, so the durable design reference does not promise more than M7 ships.
- Status table (`:18-19`) explicitly carries M7 as in-progress with the deferral note and M7b as not-started with the blocker contents named.
- M7b validation includes LSP-level broken-sysroot diagnostic tests, mismatch tests, development-sysroot equivalence, and production-path source-map tests for both synthetic origins (`:507-512`) — the items pass-2 demanded are intact, just owned by the next milestone.

## Concerns from pass 2 — closure summary

- B1 (mismatch/broken diagnostics): closed by deferral to M7b with concrete acceptance + validation lines.
- B2 (dead synthetic origin variants): closed by deferral to M7b; M7 acceptance now matches code.
- B3, H1, H2, H3, M2: resolved in pass 2 — no regression.
- N1–N5 from pass 2: still open but pre-classified as low/medium and non-blocking; N2 (stdlib bucket reparse on refresh) and N3 (observed paths in error body) are worth tracking but do not gate M7.

## Suggested follow-ups (non-blocking for M7 PR)

1. Add a pass-3 review note to the plan's PR/review trail when M7 lands so M7b inherits the open N1–N5 list.
2. When M7b is opened, lift N3 (observed paths in `sifr/sysroot` error body) into its task list — it directly serves the "include observed paths" acceptance bullet at `:493-494`.
3. Optional during M7b: cache the stdlib symbol bucket by auxiliary-source revision to avoid the per-refresh parse cost called out in N2.

## Bottom line

M7 PR is reviewable as scoped. The deferral is properly tracked: status table reflects the split, M7b has full tasks/acceptance/validation, architecture doc records the boundary, and M7 acceptance no longer asserts work the diff does not contain.
