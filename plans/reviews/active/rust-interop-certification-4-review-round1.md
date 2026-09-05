# Rust Interop Certification 4 Review — Round 1

Reviewer: agent (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer independently ran both generated-build evidence directions,
focused Rust interop tests, workspace Clippy, formatting, and count checks. It
confirmed the runtime output, pre-Cargo rejection ordering, and intended counts
after excluding the unrelated parallel `opaque_resource_matrix` hunk.

Blocking findings:

1. `rust_interop.rs`, `rust_interop_tests.rs`, and `_scenario_checks.py`
   exceeded the 900-line hand-maintained source cap.
2. The reqwest client inherited ambient proxy environment variables, so the
   loopback request could fail or leave the process.
3. The token scanner missed helper modules, imported aliases, and
   `block_in_place`, while also overmatching unrelated builders and local
   `block_on` functions.

Additional findings asked for clearer native-link trust recording and a
package-level undeclared-link negative test, task-termination-based cancellation
evidence, explicit negative-source provenance, a pre-probe assertion, suite and
profile provenance in the fixture README, and exclusion of fixture-local
`target/` build output.

Correction wave:

- Split trust validation and async scenario manifest policy into focused files;
  moved the new trust test so all hand-maintained files pass the size guard.
- Switched the runtime policy audit to `syn` AST traversal across package
  `src/`, including imported aliases, external `block_on` functions,
  `tokio::task::block_in_place`, and helper modules, while ignoring unrelated
  builders and local functions.
- Added reqwest `.no_proxy()`, runtime-ID checks, and server activity accounting
  that reaches zero only when the task future is dropped.
- Added the generated-package undeclared native-link rejection test, explicit
  pre-probe assertions, negative bridge-source provenance, and merge-suite
  provenance.
- Excluded `target/` from fixture copying and moved the local generated target
  artifact to a recoverable temporary location.

Round 2 is required after focused and authoritative validation.
