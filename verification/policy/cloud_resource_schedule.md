# Shared-cloud resource schedule

The `cloud` profile inherits the live merge assertion inventory. Its Linux
cgroup-v2 scheduler changes preparation order and graph lifetime. The functional
verdict and candidate-bound paired performance verdict remain independent.

The canonical allocation policy is [cloud_resource_schedule.json](cloud_resource_schedule.json).
Values are prospective estimates, not benchmark observations or hardware minima.
Every admission records effective affinity/quota, memory capacity and availability,
free storage, tmpfs free space, pressure and cgroup counters. Resident and tmpfs
growth share the same memory budget. Available cgroup memory includes unused
capacity and clean inactive file cache that the kernel can reclaim, capped by
the existing limit and host availability; dirty/writeback and unevictable bytes
are excluded. File cache is never an additional capacity allowance.
Each disk admission includes additional
allocation, retained copies and an 8 GiB reserve; the existing SQL clean-build
entry guard is preserved. Workers cannot exceed effective CPU quota/affinity.

Cold preparation has a prospective two-hour safety deadline per named stage;
assertions retain a forty-minute command safety deadline and their existing
inner assertion contracts. Assertions receive no new whole-step deadline.
Inherited earlier durations and absolute deadlines always win. These are
safety bounds, not performance thresholds or claims about expected duration.

Execution order is inventory guardrails, locked dependency acquisition, isolated
sysroot source/package preparation, every selected sysroot assertion, artifact
retention and eligible graph retirement, selected SQL clean assertion, its test preparation and remaining
assertions, remaining canonical preparation, then the other original guards, areas
and toolchain assertions. Every assertion runs
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
Protected compiler paths come from each producer's actual configuration,
including the package's host-triple release directory. All E2E worker arguments,
including forwarded requests, are clamped without changing case selection.
The isolated source compiler graph disables Rust incremental edit caches:
its input is immutable throughout qualification, and its later consumers use
the same source build configuration. The contributor/compiler benchmark graph
and source assertion selection retain their existing contracts.
Lease-created target roots initialize Cargo's standard `CACHEDIR.TAG` before
preparation, since Cargo only initializes the tag when it creates the root.
Only a matching owned graph receives this tag; unknown caches remain untouched.

A session recovering its own interrupted graph can explicitly provide
`SIFR_VERIFY_GRAPH_OWNER` from its retained prior execution journal. The caller
must already own that prior run and worktree. The marker's owner, worktree,
device/inode and UID must still match, and the exclusive lease must be free.
This preserves build-cache ownership continuity; it does not reuse failed
assertions. Every consumer must pass again before cleanup. Unknown/other-owner
markers are never adopted automatically.

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

## Selected SQL clean-build allocation

Selecting `build-qualification` in `area_sql_platform` gives the clean-build
assertion part `sql-build-qualification` admission and disk monitoring. Its prospective growth is 6 GiB: compact retains its 2 GiB reserve
(8 GiB entry), and normal cloud retains its 8 GiB reserve (14 GiB entry).
The monitored floor is the greater of entry free space minus 6 GiB, the policy
reserve plus 1 GiB stopping headroom, and any stricter caller floor. Thus an
8 GiB compact entry allows at most 5 GiB growth before the 3 GiB floor.
Memory remains 6 GiB resident plus 2 GiB reserve. The SQL tool's independent
8 GiB clean-build entry guard, native A/reused-A/independent-B commands,
incremental setting, cases and deadlines stay unchanged. Other or unknown
selections keep their existing allocations and monitoring behavior. This models
the existing entry envelope prospectively; actual SQL peak growth, cold fit and
full acceptance remain unmeasured. It grants no assertion reuse or graph cleanup.

## Early whole-area SQL execution

When SQL selects `build-qualification`, resource-aware execution runs that actual
clean-build suite first, immediately after successful sysroot consumers. It needs
none of the 30 SQL no-run preparations. After its private graphs close, those
same 30 preparations run, followed by the other 18 suites. This retains all 19
canonical suites and 66 cases, including parser-major and SQLite probe native
assertions. The remainder uses the existing `remaining-assertions` allocation;
both parts keep disk monitoring, policy reserves, stopping headroom and stricter
caller floors. These are prospective attempts, not measured fit guarantees.

Fresh invocation-owned part results are checked against the exact manifest cases
and subprocess outcomes, then assembled in canonical suite/case order at the
original SQL result path. Missing or invalid execution becomes explicitly blocked
cases with its cause. One canonical area outcome covers the complete selection;
no part from another invocation can qualify it. An ordinary completed build-case
failure still runs the other SQL cases, as the existing SQL adapter does, before
outer fail-fast acts. Infrastructure without complete execution evidence may stop
earlier; `--no-fail-fast` continues independent admitted work and retains failure.
Later scheduling omits only this invocation's already handled SQL preparation
and assertions. Reports show the real phase chronology.

The two assertion commands share the original 2400-second cumulative safety
budget. Interleaved preparation time is excluded; preparation keeps its existing
limits. Tighter inherited command, step and absolute deadlines still win, and
environment limits are restored afterward. Canonical SQL elapsed time sums its
assertion parts. No profile selection, SQL build recipe, checkpoint, previous-pass
reuse or cleanup contract changes. SQL preparation and later native assertions
may still exhaust their forecasts; actual cold fit remains unproven.
