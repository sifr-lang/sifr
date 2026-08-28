# Sustained Coverage-Guided Fuzzing

The `sustained-fuzz` suite runs six libFuzzer targets. It covers parsing, lowering, ownership,
generated-Rust validation, diagnostic presentation, and package project graphs.

Nightly gives each target 45 seconds. Release gives each target 120 seconds. The committed manifest
owns these budgets. The suite runs outside merge and is non-blocking. A tool failure or compiler
finding remains visible in its machine-readable area result.

The runner writes corpora to `target/verification/fuzz/corpus/<target>` and crash artifacts to
`target/verification/fuzz/artifacts/<target>`. CI installs the pinned `cargo-fuzz` version and
fetches the excluded fuzz-project lockfile before the offline run.

Promote a finding only after it is stable and minimized. Put unresolved crashes in the crash
corpus. Put fixed defects in the fixed-bug registry. Include the fuzz target, source or input hash,
and exact reproduction command in the issue and pull request.
