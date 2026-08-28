# Sustained Coverage-Guided Fuzzing

The `sustained-fuzz` suite runs six libFuzzer targets. It covers parsing, lowering, ownership,
generated-Rust validation, diagnostic presentation, and package project graphs.

Nightly gives each target 45 seconds. Release gives each target 120 seconds. The committed manifest
owns these budgets. The area has a 40-minute long-running resource envelope. This includes the
bounded 20-minute cold instrumented build and the six release target budgets. The suite runs
outside merge and is non-blocking.

The machine-readable result distinguishes a missing fuzz tool, an offline dependency failure, an
instrumented-build failure or timeout, a target timeout, a target execution failure, and a real
fuzz finding. Failed preflights and targets retain a bounded 16 KiB output tail. Every target uses
a suite-qualified evidence label.

`cargo-test` semantic properties have an explicit one-run contract. Cargo runs the named Rust test
once because the property itself owns its input matrix. Process-level determinism uses the
`PROP-SEM-0005` `emit` entry instead; the runner starts two compiler processes and compares their
normalized output.

The runner writes corpora to `target/verification/fuzz/corpus/<target>` and crash artifacts to
`target/verification/fuzz/artifacts/<target>`. CI installs the pinned `cargo-fuzz` version and
fetches the excluded fuzz-project lockfile before the offline run.

Promote a finding only after it is stable and minimized. Put unresolved crashes in the crash
corpus. Put fixed defects in the fixed-bug registry. Include the fuzz target, source or input hash,
and exact reproduction command in the issue and pull request.
