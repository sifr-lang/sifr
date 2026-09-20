# Formatter integration-test lint qualification

Status: recorded, separately owned; no formatter behavior repair authorized here.
Owner: formatter discovery integration-test harness, following the
[formatter execution phase](../archive/ad-hoc-production-grade-sifr-formatter-execution.md).

The 2026-09-21 affected DXF lint reconciliation attempted default-feature
`cargo clippy --locked -p sifr_syntax -p sifr_frontend -p sifr_driver -p sifr_lsp -p sifr --all-targets -- -D warnings`
on Rust 1.98.1. After affected production/inline-test warnings were corrected,
Clippy reported 46 `unwrap_used` errors in
`crates/sifr/tests/formatter_discovery.rs` (lines 7–221). This standalone
integration target is unchanged by DXF.1–6 and distinct from the CLI binary's
inline tests. No compiler/product failure or formatter behavioral defect is
inferred from these test-harness diagnostics.

Evidence is retained under
`/home/yaser5/projects/sifr/affected-clippy-evidence/` in the candidate's
`preflight/clippy-cleanup-corrected.log`. The broader attempt remains failed.
The affected CLI test configuration is separately selected by
`cargo clippy --locked -p sifr --bin sifr --profile test -v -- -D warnings`;
its verbose compiler command verifies `--test`, so it is not an untested
production-only replacement. No full workspace/all-target clean claim is made.

Next action: in a bounded formatter-harness item, convert fallible fixture and
process helpers/tests to explicit propagated errors while preserving assertions,
then run `cargo clippy --locked -p sifr --test formatter_discovery -- -D warnings`
and the existing integration tests selected by
`cargo test --locked -p sifr --test formatter_discovery` (verify the actual
nonzero inventory before execution). Do not suppress warnings or infer that
other unselected integration targets are clean.
