# Shared-cloud resource schedule

The `cloud` profile inherits the live merge assertion inventory. Its Linux
cgroup-v2 scheduler changes preparation order and graph lifetime. The functional
verdict and candidate-bound paired performance verdict remain independent.

The canonical allocation policy is [cloud_resource_schedule.json](cloud_resource_schedule.json).
Values are prospective estimates, not benchmark observations or hardware minima.
Every admission records effective affinity/quota, memory capacity and availability,
free storage, tmpfs free space, pressure and cgroup counters. Resident and tmpfs
growth share the same memory budget. Each disk admission includes additional
allocation, retained copies and an 8 GiB reserve; the existing SQL clean-build
reserve is preserved. Workers cannot exceed effective CPU quota/affinity.

Cold preparation has a prospective two-hour safety deadline per named stage;
assertions retain a forty-minute enclosing safety deadline and their existing
inner assertion contracts. An inherited earlier deadline always wins. These are
safety bounds, not performance thresholds or claims about expected duration.

Execution order is inventory guardrails, locked dependency acquisition, isolated
sysroot source/package preparation, every selected sysroot assertion, artifact
retention and eligible graph retirement, remaining canonical preparation, then
the other original guards, areas and toolchain assertions. Every assertion runs
once. A failed prerequisite is blocked; failed/incomplete consumers prohibit
cleanup. `--no-fail-fast` continues independent remaining work when it can be
admitted, preserving all failures. Preparation never substitutes for assertions.

Only the two declared isolated sysroot Cargo targets may be retired. An exclusive
lease records session/worktree ownership and graph device/inode identity before
creation. Existing graphs without matching ownership may be consumed but never
reclaimed. Cleanup rejects live Cargo/rustc processes and symlinks, preserves
compiler bytes by verified independent immutable copies, then uses Cargo's
supported `clean --target-dir` operation. Net recovered space includes the cost
of retained copies; mutable Cargo outputs are never hardlinked for deduplication.

Each run publishes immutable observations under
`target/verification/execution-journals/<session>/`, including its source/runtime
key, live selection, admission policy, classifications, actual results and resource
observations. Source drift before or during a stage blocks qualification. Raw
sysroot results and retained compiler identities survive graph cleanup. Journal
observations are explicitly distinct from complete correctness evidence and
paired performance receipts. Checkpoint consumption remains disabled here until
each invocation's complete dependency closure and result accounting are proven;
the strict correctness-evidence schema owns the future consumption protocol.

Never convert missing results, infrastructure failures, compilation-only work or
an inconclusive required performance result into a phase-qualified pass. Preserve
earlier failures and change allocation/deadline policy only prospectively, with
the recorded cause and new candidate identity.
