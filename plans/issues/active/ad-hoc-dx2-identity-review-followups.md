# DX.2 identity and review follow-ups

status: open; separate future work, not DX.2 blockers
source: [merged DX.2](../archive/ad-hoc-compiler-dx-and-toolchain-reuse.md)
implementation: [PR #3842](https://github.com/sifr-lang/sifr/pull/3842)

These observations do not authorize implementation in the completed DX.2 session.

- Measure and reduce the approximately 15.526-second warm outer aggregate input
  inventory overhead within an appropriate later optimization item.
- Investigate/test Cargo precedence when both .cargo/config and config.toml exist.
  This is an unconfirmed review suggestion, not an established defect.
- If generated projects gain multiple binaries, select Cargo JSON executable
  artifacts by target name/kind.
- Clarify source-size policy wording and create headroom for the existing
  900-line CLI source; the enforced guard accepts the current file.
- Clarify embedding documentation for LSP convenience entrypoints using compiled
  test identity; the normal CLI uses explicit product identity.
- Consider dependency-direction guard coverage for dev-dependencies and more
  specific build-only exceptions.
- Improve taxonomy self-test fixtures with actual newlines and architecture source
  citation precision; preserve all case IDs and acceptance obligations.
- If Python is added to generated Sifr test runners, carry the selected Python
  setup and loader contract there. That runner does not currently support Python.
- Improve developer-tooling guard diagnostics for a deleted split test module and
  require adjacent path-attribute/module registration.
- Preflight named performance reference and tool prerequisites before expensive
  qualification; retain incomparable outcomes and immutable budgets.
- Charge cold sysroot release packaging preparation explicitly; preserve deadline
  failures and avoid treating cold compilation as warm performance evidence.
- Existing SQL test-only Clippy type-complexity warnings remain with
  [their owner](ad-hoc-schema-first-sql-platform-review-follow-ups.md).

Final review: [Opus SATISFIED](https://github.com/sifr-lang/sifr/pull/3842#issuecomment-5709069845).
Raw evidence and earlier scoped reviews remain under
yaser5@yaser.tailaa73b4.ts.net:/home/yaser5/projects/sifr/dx2-evidence/.
