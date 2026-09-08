# 12K-B46 — startup and formatter performance repair

## Terminal disposition

**TERMINAL HOLD — implemented and reviewed, but the one canonical performance
qualification FAILED all nine original enforceable limits. Not delivered or
merged. No retry, post-failure source repair, PR, Sifr gate, or next item.**

Owner: [issue3776](https://github.com/sifr-lang/sifr/issues/3776). B27 owns joint
adjudication. This scoped record is the only record-branch change above the
unchanged reviewed source. The parent phase ledger is parent-owned/read-only;
registration, review, prelaunch, result and terminal callbacks go to parent
`01a06e86-414a-7e11-9256-1f45bdb5a6c7` and Police
`01a07d84-7c62-77f0-b7c7-ecf310829a11`.

| Identity | Value |
| --- | --- |
| Independent HTTPS clone | `/private/tmp/sifr-b46.rvEbWp/sifr` |
| Source base | `93111f3426245b3ba5484195b8154d3bfc769769` (B44 over B42 `4a867cb`) |
| Main at registration/prelaunch | `4b4cc339964baeeb6641e57dc669fef700a5fa24` |
| Final source | `df0056ad1b9d7fcbaacda3a3f0809da66279ea84` |
| Durable source branch | `codex/item12k-b46-rvEbWp` |
| Separate record branch | `codex/item12k-b46-record-rvEbWp` |
| Test/preparation TMPDIR | `/private/tmp/sifr-b46.rvEbWp/tmp` |
| Distinct qualification TMPDIR | `/private/tmp/sifr-b46.rvEbWp/qualification-tmp` |
| Own evidence | `/private/tmp/sifr-b46.rvEbWp/evidence` |

Private index, refs, objects and target; no alternates/shared target. Every
command explicitly bound its appropriate TMPDIR and unset CARGO_TARGET_DIR.
Parent/predecessor worktrees, indices, refs, caches and evidence were read-only.
Ruff `f19957111640fdee8055bfe5b6aa854259344473` and wasi-virt
`448f6df8f688cee5d6995e96b1ffc31f9bf00742` were initialized in this clone only.
No old3717/B13 stack or lockfile import.

The named phase-closure-loop and talk-to-claude-opus skills were read and used
with the explicit B46 overrides: one initial plus at most one remediation
source-only review, one full representative, zero create-PR/merge gates,
zero PR/merge, then held source/record/evidence handoff. Generic skill delivery
loops and B45 proposal-only wording do not override this implementation grant.

## Authenticated prerequisite

B45 final report and all three companions were read; its source-backed A+F
decision, B44 record/review/raw evidence and relevant predecessor chain were
authenticated from saved bytes, not replayed. `evidence/authentication.json`
binds910 distinct files, including B45's908-file inventory and all105 B44 raw
samples; no mismatch. Terminal sealing rechecks those same bytes.

B45 report SHA256 `462e46d9a25d48b8101520115b41b91a11e048cb76f7e5068eeb34a19ea8cae9`,
record `826679ca3a6f23af4f52ec88b132542e99372700`, terminal
`361e6d1ebe2d6188d7fbbec60113adbbc8b7041cffc68319b0151fcbc97e5295`.
B44 source is the exact base above; its terminal
`a1df46329bd6054851c81d9997161f63f5ff1eb463573fabda8a54b86bd5158b`
and700-file inventory
`86fcc80b3e189e7c4f3354420cb586981566025007f605b73e5527a22a1c17c4`
remain unchanged. B44's single budget failure is not reset by B46.

## Implemented A and F boundaries

1. `ModuleCodegenResult = CodegenResult<()>` is an explicit no-application-plan
   bootstrap/per-module result. `StdlibEmissionCode` excludes HIR inventory
   from emission signatures. Local structural policy and structured bootstrap
   diagnostics remain. Empty private modules are inserted before their early
   continue; completed HIR moves into the final shared Arc inventory after
   bootstrap consumers, instead of cloning retained lowering output.
2. Canonical `application_plan` runs once at final complete single-file,
   combined project, and actual support+test-project assembly. Per-module and
   bootstrap discarded selections are removed. The full private/global
   inventory remains present; there is no module-prefix heuristic or empty
   inventory fallback.
3. IR-owned `readonly_visit.rs` is an exhaustive immutable sibling of the
   existing type visitor. It traverses defaults, nested functions, statement
   types, patterns and expressions once without cloning each visited function.
   Existing recursive nominal/structural type demand remains authoritative.
   A per-inventory intrinsic index avoids repeated whole-inventory lookup.
4. Only selected declarations and needed metadata are projected into the
   existing contract builder. Borrowed name membership sets select declarations
   while original source vectors determine their order, preserving positional
   cache fragments and diagnostic order. No whole-module clone/retain, lossy
   type projection, new representation graph or probe-policy redesign.
5. Definitions-only cache projection clones only requested external definitions
   from the SAME success/error OnceLock. It preserves initialization, cached
   errors, global inventory, later applications and sysroot trust. This does
   **not** claim to free retained global cache data. The shared analysis
   regression preserves canonical editor diagnostics and old immutable
   snapshots/version/release behavior; no speculative LSP leak/cache rewrite.
6. Formatter discovery validates each rule once and lazily caches only relevant
   per-rule compiled matchers within that discovery invocation. It never caches
   a file decision. File-before-parent and ordered-rule precedence remain with
   the ignore engine; malformed diagnostics, negation, literal-filter safety,
   symlinks/outside-root, explicit bypass, force-exclude, no-respect and lazy
   load ordering remain. No performance fixture/config/ignore/workload change.

B44 typed closure, reexports, private calls, generated open/Python support,
cleanup/operators, selected-plan+sysroot cache and user trust isolation remain.
ExecuteAll signature/unsafe/SendSync/borrow/async/callback/panic contracts and
emit deferral are not bypassed. Project codegen tests were moved unchanged to
a responsibility-owned test file to respect the900-line guard. Architecture
documentation changed only for these actual ownership/result boundaries.

### Explicit pre-existing test-runner integration obligation

The complete selected plan is retained in `TestProjectCodegenResult.interop`
and `GeneratedTestRunnerProject.interop`. The pre-existing runtime consumer at
`crates/sifr_driver/src/test_runner/artifacts.rs:67-79` still constructs
`InteropBuildPlan::default()` rather than consuming that result. Therefore
metadata retention **does not make test-runner Rust interop execute correctly**.
Per Police's explicit ruling, a new test-runner probe/attachment/materialization
pipeline is outside B46. This missing consumer and its execution impact remain
an issue3776/B27 full-test-runner integration obligation; no new pipeline was
implemented or silently claimed here.

## Named validation and selected semantic proof

All implementation and all new test inputs existed at
`1ff68ba7f61fe2d503faa1f30679140f89ada360` before the first named test.
Failed `startup-01` used a test sample calling Object methods on an unhandled
Result; `5657dc99d09b1cd71b2d52ac8f1bf0b21a667908` added explicit try/except and
`startup-02` passed4. Failed `analysis-01` asserted mutation of initial overlays;
`e5a97a1c7cdc5650f142a7a1bc4fd96c192f9efb` instead checked the authoritative
frontend version/source map and `analysis-02` passed1. Both failures, complete
logs, source identities and process-release receipts remain saved.

After initial review, one bounded remediation restored source declaration
order and added an independent literal eleven-function filesystem-order
assertion to the existing hidden-edge test. All22 registered groups were then
rerun on exact final `df0056ad1`: **38tests PASS plus4guards PASS**. No new suite,
Clippy, workspace or Sifr gate was run. `evidence/named_checks.py` holds exact
argv; `freeze.df0056ad1b9d7fcbaacda3a3f0809da66279ea84.json` binds every final
pass to this exact SHA, SHA256
`a1d14706d846b2b2eaa22c3f67d5fc4e63ba593bb1b84769106e17a419c6d96a`.

All Cargo tests used `--locked`; driver and analysis commands were serial:

| Registered command/filter | Final evidence |
| --- | --- |
| `cargo test --locked -p sifr_driver --lib stdlib_interop_startup -- --test-threads=1` | startup-03:4PASS |
| `cargo test --locked -p sifr_analysis --lib stdlib_interop_startup_editor_retained_snapshot_after_defs_projection -- --test-threads=1` | analysis-03:1PASS |
| `cargo test --locked -p sifr_driver --lib stdlib_interop_demand -- --test-threads=1` | demand-02:5PASS |
| `private_stdlib_interop_resolves_sysroot_crate_target` | iteration02:1PASS |
| `merged_user_and_private_stdlib_interop_both_resolve` | iteration02:1PASS |
| `merged_user_and_private_stdlib_interop_keeps_user_trust_separate` | iteration02:1PASS |
| `package_rust_interop_direct_probe_checks_signature_shape` | iteration02:1PASS |
| `package_rust_interop_direct_probe_accepts_bridge_signature` | iteration02:1PASS |
| `package_rust_interop_direct_probe_rejects_unsafe_fn` | iteration02:1PASS |
| `package_rust_interop_opaque_probe_rejects_unsatisfied_send_obligation` | iteration02:1PASS |
| `package_rust_interop_cache_fragment_is_deterministic` | iteration02:1PASS |
| `package_rust_interop_cache_changes_with_trust_policy` | iteration02:1PASS |
| `private_stdlib_imports_resolve_only_from_compiled_source_exports` | iteration02:1PASS |
| `missing_private_stdlib_member_is_a_structured_bootstrap_failure` | iteration02:1PASS |
| `missing_private_stdlib_module_is_a_structured_bootstrap_failure` | iteration02:1PASS |
| `test_get_or_init_stdlib_cache_reuses_` | iteration02:success+error2PASS |
| `cargo test --locked -p sifr --bin sifr formatter_discovery` | formatter-unit-02:7PASS |
| `cargo test --locked -p sifr --test formatter_discovery` | formatter-cli-02:7PASS |
| `cargo fmt --check`; `git diff --check`; HIR and file-size guardrail scripts | each iteration02:PASS |

Bare driver filters above use the same `cargo test --locked -p sifr_driver
--lib FILTER -- --test-threads=1` form. Existing six formatter unit and six CLI
tests were retained, with one new same-rules/different-path decision regression
in each group. No optional general IR suite was added: exhaustive variant
matching compiled, and hidden-node behavior is tested by the registered startup
filter. Unchanged B42 warmup/raw/query and B23 source evidence was reused; their
embedded canonical rules ran only as part of the one full qualification.

`evidence/semantic-proof.md` records concrete assertions: real bootstrap
selection count0 with full/empty inventory and structured errors; single/project
and actual test-project count1 including last-module demand and both orders;
immutable statement/default/nested edges, open/Python support and invalid
selected signature rejection; SAME cache/errors/inventory A-B+B-A/sysroot
isolation; canonical editor diagnostics/version/old snapshot/release; lazy
matcher identity with different decisions/negation and later invalid-directory
diagnosis after earlier explicit-file bypass. These are semantic checks, not
timing tests or inferred performance savings.

## Exact-source Opus review

Named skill invocation used `claude --permission-mode plan --setting-sources
project --model claude-opus-5 --effort medium --no-session-persistence -p`, fresh
private request directories, atomic final response,2400s watchdog and owned
process supervisor. No partial response polling; reviewers ran no tests.

| Round | Candidate | Verdict and receipt |
| --- | --- | --- |
| Initial | `e5a97a1c7cdc5650f142a7a1bc4fd96c192f9efb` | SATISFIED;675.394935s; nonblocking source-order finding |
| Only remediation | `df0056ad1b9d7fcbaacda3a3f0809da66279ea84` | SATISFIED;185.807750s; explicitly no new mechanism defect |

Initial response SHA256
`10e2a84d0d8ae76543b3280184ac2330f4dc5125dd3eb1ed89210b722e122f5d`;
final response SHA256
`1b3e4cb88b7ce7a3b4c1f4d720c0426b3141f68bd33a42d144f5d4ec60d8351a`.
Both `evidence/opus-review.<SHA>.md` files, original request directories/prompts,
logs and review-disposition records are retained. Provider errors0; initial1
plus remediation1, no further review allowed. Each review supervisor sent one
SIGTERM only to its owned review group during cleanup and verified full release;
both exited0 without timeout. Historical review counters remain unchanged.

## One original full qualification

After exact named PASS and final SAT, source was pushed. Ordinary private locked
`cargo build --locked -p sifr` and `cargo build --locked -p sifr_frontend --bin
frontend_query_bench` passed in test/preparation TMP (26.178191s/42.244230s).
Inherited `cold-*-build-01` receipt labels do not claim a cold cache or performance
evidence. A separate empty qualification TMP was then created and verified
pristine. Prelaunch: AC80%,56GiB free, private target18GiB, no competing heavy
or benchmark command, all review groups gone. No target cleanup or extra quiet
window. Canonical host admission/monitoring remained authoritative.

Binary SHA256s:

- own `target/debug/sifr`: `1d34f40dde09932a852be1420a72493de548de9c14def52b792c45a528248424`;
- own `target/debug/frontend_query_bench`: `463cbc34982c8d8f0c480865c28d5914e800f1b345ea26ceaf46bc5c3fb477bb`.

Actual LSP is own `target/debug/sifr lsp --stdio`; no override/extra LSP binary.
Prelaunch SHA256 `9694ac3bda0e0707720066712de84422ca866198a6c316ad9cad400f7cc73351`;
launch-binding SHA256 `fc847f8fe5bfdf3dace8e9da52bb1d70edd996c524b6ca3e51e28d7e81ce4c98`.
Binding and actual argv were sent to both parent and Police before execution.

```sh
python3 /private/tmp/sifr-b46.rvEbWp/evidence/run_receipt.py representative-01 1800 uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative
```

Original all10 cases,6 rules, counts, internal max3 controlled attempts, budgets,
host monitoring and actual case limits remain: **120s build;60s check/formatter/
LSP;30s frontend queries**, not blanket120. External1800s plus at most120s owned
cleanup, at most1920s inside7200. Supervisor is byte-identical to authenticated
B44, SHA256 `c4d26ef72e6a12f14c47b980461c3ce32b77262c8522a856edbff7a1a0fb55c1`;
B23 owned-group cleanup source remains unchanged.

Actual generated ID **`performance-representative-1788887990611838000`**;
run root `target/performance/bench-1788887990-69451`. All10 accepted, all6 rules
PASS, producer exit0 in188891.610ms. Nine cases accepted attempt1. Formatter
large-file attempt1 was rejected for instruction CV0.029976; the canonical
allowed internal attempt2 accepted CV0.011954. All accepted work CVs are below
0.02. This internal retry is not an external second acquisition; both attempts
and all rejected samples are retained.

**Actual SAME-ID budget consumer exit1 in60.672ms: nine failures across six
cases.** Full command exit1 after196.331495666s, no timeout.111 raw process
receipts:42 original command receipts plus6 rejected formatter receipts, and
63 query aggregate/work receipts. Query aggregate retains3warmups+20measured;
20 separate work processes retain3+1 internal iterations. No sample timed out;
the diagnostic case's expected exit1 is preserved. Full CLI instructions are
not compared to inner request latency. No external retry/subset/repair/extension
or reuse of banked old samples occurred after this terminal failure.

### Exact nine original limits — all remain FAIL

Instructions below are medians; RSS is bytes. Parenthesized instruction values
are the actual lower bounds used against limits after unchanged MAD uncertainty.

| Case / metric | B44 before | B46 after | Original limit |
| --- | ---: | ---: | ---: |
| single build RSS | 185466880 | 190169088 | 181649408 |
| project check instructions | 20705027673 (20667077619) | 19453233660 (19444966650) | 13938915299 |
| project check RSS | 185597952 | 185024512 | 179601408 |
| single check instructions | 20651201663 (20637475031) | 19394638768 (19365249772) | 13879420485 |
| single check RSS | 185139200 | 186023936 | 181436416 |
| JSON diagnostic instructions | 20645274502 (20634512512) | 19389012613 (19379235694) | 13874003870 |
| JSON diagnostic RSS | 185253888 | 185352192 | 180748288 |
| formatter corpus instructions | 46400530 (46170688) | 45331927 (45172711) | 38059926 |
| LSP diagnostics RSS | 150945792 | 155418624 | 108593152 |

All exact benchmark IDs, uncertainty values, commands/exits and actual original
limits are in `evidence/budget-disposition.json`. Original work budget SHA256
`56f91e711d78b4efb7e759e20868a3edec67c95f5b70368f08ddb5e70f98cc26` is unchanged.
Saved arithmetic comparisons are not a rerun of the checker; the canonical
budget-subset variant and log are authoritative. No checker rerun occurred.

### Four original controls — remain PASS

| Control / metric | B44 before | B46 after | Original limit |
| --- | ---: | ---: | ---: |
| additional-modules build instructions | 69816057092 | 68481102966 | 119984530852 |
| additional-modules build RSS | 208486400 | 213942272 | 241762304 |
| formatter large-file instructions | 56425524 | 56512549 | 59134110 |
| formatter large-file RSS | 13058048 | 13254656 | 44974080 |
| unchanged-file query instructions | 849224414 | 851923254 | 936811698 |
| unchanged-file query RSS | 6668288 | 6684672 | 38633472 |
| warm-diagnostics query instructions | 1430924064 | 1434319294 | 1480999372 |
| warm-diagnostics query RSS | 10551296 | 10682368 | 41517056 |

Both frontend query controls retain2300cache hits/2300misses. The previously
passing corpus RSS15024128<44793856 and LSP instructions366240014
(lower364408813.93)<390465660 also remain passing. There are no new budget
failures, but none of the nine original failures was cleared.

## Raw evidence, actual checker and release

All paths below are relative to `/private/tmp/sifr-b46.rvEbWp`:

| Artifact | SHA256 / content |
| --- | --- |
| `evidence/representative-01.json` | `04590e5fddeb4cee176856bb8520ca9d49a62708d37a6b1cccd039ebc89f71f2` — actual argv/deadlines/PID-start identities/release |
| `evidence/representative-01.log` | `cb9fb993c192b9d1a279d731534c0ed977101083127780e88b98363e66c7535f` — original actual checker errors |
| `evidence/representative-summary.json` | `a426755033dcba50135d2e53810f427d7a34862e1883b78972678c2180660bef` — all attempts111samples/actual variants/verified input hashes |
| `evidence/budget-disposition.json` | `5ab04a80725c205c70c9f8d6c1d8a69d90d82b22748713d323cd687e1b3414cf` — exact9comparisons+controls and actual checker exit |
| `sifr/target/performance/representative.budget.latest.json` | `947d19d275a8257afe0c39be2eb27cf2b6c8186ba4daccdf9be7c4b338e24c3f` — PRODUCER INPUT, **not a separate checker report** |
| `sifr/target/verification/areas/performance-results.json` | actual same-ID producer/checker argv and exits; hash in summary/inventory |
| `sifr/target/performance/bench-1788887990-69451/attempts/` | all111 raw receipts, stdout/stderr sidecars, accepted/rejected results and host snapshots |

Source, binaries, original workloads/config/ignore/lock/budget/runner inputs and
ancestor inputs were verified unchanged after qualification. The canonical
supervisor observed287 process identities in68 groups; all were absent at
completion, with no cleanup signal needed. Final read-only sealing checks all
named-test, review, preparation and canonical owned identities/groups again,
rehashes authenticated predecessors and all evidence, and writes
`evidence/final-process-release.json`, `evidence/evidence-inventory.json` and
`evidence/terminal.json` outside Git. The durable owner terminal callback gives
their final hashes and this record's commit SHA.

## Remaining acceptance and mandatory stop

The approved A/F mechanisms are implemented and source-reviewed; this does not
prove sufficient performance repair. The original nine limits remain valid and
enforceable, with no baseline/threshold/metric/policy change or waiver. Formatter
baseline zero-file capture remains UNPROVEN; missing f8c09d47 recovery was not
a repair prerequisite and was not attempted. LSP package-root drift alone does
not waive RSS; no new leak diagnosis is inferred. Historical formatter CV and
precise historical timeout cost/cause remain UNKNOWN. No causal savings claim
or automatic acceptance follows from a lower measured instruction count.

B42 admission failure, HOST first-warmup timeout, B44 budget failure and B45
saved assessment remain immutable. Original12K4FAILED+1RESOURCE, B13two,
B14one, B15two, exhausted B20initial+B22remediation, and separate B42/B44 initial
SAT counters remain. B46 adds exactly initial1+remediation1 SAT and one canonical
budget FAIL; it resets none of them.

B27 joint scope retains original B26 assessment/inputs,264fresh companions,
90native/411check, Python30, exact65SQL, all areas/toolchains, original emitted
quality and full delivery gate. Original3717draft remains unmerged; no blind
old-stack integration or65SQL-main-first work.12D/E/F, full12 and final12A
documentation/whole-phase review remain later obligations after all required
work is merged. No SQL/SDK-unblocked-main claim is made.

**Blocker: the one full representative failed the nine binding budgets.
Next authority is B27 joint adjudication, not another B46 attempt. Stop after
the durable source/record/owner/terminal handoff. No next item was started.**
