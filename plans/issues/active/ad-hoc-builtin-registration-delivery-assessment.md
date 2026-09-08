# 12K-B21: independent builtin-registration delivery assessment

Date: 2026-09-08. Owner: emitted-code phase / incorporated Item12C within12B.
Scope: source/dependency assessment and documentation only. No implementation,
compiler tests, builds, benchmark acquisition or Sifr gates were performed.

## Decision and evidence boundary

The existing repair has a complete independent source closure on assessed main:
three codegen files, including both registry regressions. It does not require
B20's formatter changes, the retained Python integration, SQL changes, corpus
changes, or Item65 release-policy changes to type-check its API relationships.
This conclusion comes from exact source comparisons and complete caller tracing;
it is not an assertion that a future compiler candidate has passed qualification.
Deliver through the fresh B25 integration registration below, with normal full
qualification and no inherited failure waived. Do not cherry-pick either full
PR3694 or PR3717, or execute B25 during B21.

## Immutable inputs and actual applicability

| Input | Exact identity / result |
| --- | --- |
| Assessed main / B21 base | `6bd085f40e42c04b1d81a09ce1660a392541bd1e`, Item67 record PR3784 merged; verified from origin and GitHub |
| SQL reported candidate | `4f53a1ce39f612e5e8c26b8802e5ef798c07d026`; unavailable by GitHub SHA fetch, read-only fetched from `/private/tmp/sifr-sql-item1.T06Xmx/codebase` into owned `refs/assessment/sql` |
| Original builtin repair | `3f422b01633d23c2bc8d8ce8ca59057c6e56adea`, parent `d10f9459b1fb66d228e6f2a93e02d3d0e7d9c158` |
| Retained12B | [PR3694](https://github.com/sifr-lang/sifr/pull/3694), OPEN; head `8e532f15895e7005fae8c658739ba3c3a6818c18`, base `b475ebdcd37081aa2860d9c348ace4100b546eff` |
| Retained integration | [PR3717](https://github.com/sifr-lang/sifr/pull/3717), OPEN; head `98480c78587d6cbd99a7079d10c71825360bd468`, API base `156157242b0995c01c4fff03575624b5c471c0d8` (not the original integration review base) |
| B20 terminal | `96d051e4c16771ae1902f6e8aa77ecd836d45fa6`; [PR3783](https://github.com/sifr-lang/sifr/pull/3783) OPEN DRAFT, head `ac4c277b30c6cac046ab1746fe4020c04df7eaf1`, base `fb66ed00d7672b9f5c623e3291d1b8498d50ec45` |

SQL's actual receipt is `/private/tmp/sifr-sql-item1.T06Xmx/clippy.log`, SHA256
`d13d32cf4a9190517172cd86c1cb204d660a641c14ae550c56f9202ce2be6b75`.
Lines331–348 report `clippy::expect_used` at
`crates/sifr_codegen/src/project_stdlib_nominals.rs:45-46` and failure to compile
the codegen library. The entire offending nominal file and builtin catalog
are byte-identical between the reported SQL commit and assessed main. Thus the
source defect still applies; a new Clippy run was neither needed nor performed.
The log is not a prediction that no other workspace lint can fail afterward.

B20 terminal `/private/tmp/sifr-b20.bj6Au6/evidence/terminal.json` authenticates
as SHA256 `0b4fe5b32beb604527091fdfbeab9ff3b4b7dbd6d4d194e27ebb17d3260999dc`.
It records no merge/gate, one NOT SATISFIED review, invalid-isolation acquisition,
and zero remaining processes. Its eager-ignore regression and timeout cleanup
finding remain later owners; neither is a builtin-registration prerequisite.

## Complete source closure

All paths below are relative to the repository. The exact three-file source
reference is retained12B `8e532f15895e7005fae8c658739ba3c3a6818c18`.
It contains the original implementation with current descriptive test names.

| Required path | Main blob | Retained12B blob | Responsibility |
| --- | --- | --- | --- |
| `crates/sifr_codegen/src/builtin_errors.rs` | `5278964c88b9f8cdd0504cb099a00041e3abd1e8` | `3d89a8ff3124484edcbeac020f607f6fb25b9eb5` | Private-field catalog token, total canonical identity, unchanged partial lookup API for other consumers |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs` | `52d61d2c2efaf8a7b23c8a9dfbb2623fc178e500` | `712ada9980232b7a6692b292d01954512a29edb7` | Typed collection keys, both production registration callers, identity filtering, registry module wiring and existing shadow test adaptation |
| `crates/sifr_codegen/src/project_stdlib_nominals/registry.rs` | absent | `c181097844afd9561a968a2ba6a3f960e32dfded` | Registry ownership boundary and two positive/negative regressions |

Exact comparison of the original repair's parent with assessed main finds no
builtin catalog change and only the descriptive rename of
`item10a_builtin_registry_identity_does_not_replace_a_module_qualified_shadow`
in the nominal file. Original repair to retained12B changes only that test name
and the two `item12b_` registry names to `corpus_repair_`; production is identical.
Use these three paths as a reviewed source reference; do not import its records,
other compiler changes, demos or submodule pointer. Main's nominal file has882
lines; the retained registry extraction is a responsibility split preserving
headroom for tests, not an alphabetical or line-count split.

The contract and every registration path are:

1. `BuiltinError` is crate-private with private `&'static str` storage.
   `all()` constructs only catalog members; `from_name()` returns `None` for
   other strings. `identity()` is total and produces `sifr.builtin.<name>`.
   Deriving order over the stored name preserves the former string-key sort.
2. `builtin_error_identity(&str) -> Option<String>` delegates to that authority.
   Its re-export in `lib.rs:8` and consumer in `union_type_helpers.rs:10-25`
   retain their signatures and existing identity-less lookup behavior.
3. `project_stdlib_nominal_plan` collects union members, module function types,
   local declarations, class fields/parents/methods/operators and constants through
   `collect_function_nominals`, `collect_hir_function_nominals`,
   `collect_module_nominals` and recursive `collect_shared_nominals`.
   All four collectors carry `BTreeMap<BuiltinError, Type>` consistently.
   `compiler_builtin_error` validates the name while preserving the existing
   identity predicate; module-qualified shadows are rejected. Existing
   identity-less, `sifr.builtin.*` and global Rust nominal handling is preserved.
4. Referenced-error demand from `error_refs.rs` is intersected with catalog
   tokens, preserving local-error and IO-kind exclusions. The `Error` branch
   still collects async Python error-contract types. Final plan registration
   receives the token, preserves explicit nominal aliases, and registers the
   canonical identity without a second partial lookup or replacement panic.
5. `register_emitted_builtin_nominals` iterates catalog tokens, registers only
   definitions actually present in shared support, and preserves IO-kind
   exclusion. This is the second and only other production registration caller.
6. The third preexisting caller is the module-shadow unit test. The extracted
   registry's shared/crate-root methods retain behavior and use `pub(super)`
   access. `project_union_relocation.rs` accesses the same re-exported type and
   fields; no new external API, package dependency or visibility widening is needed.
7. Both consumers are already on main: `lib_project_codegen.rs:325,412` and
   `lib_test_project_codegen.rs:57,172` call plan/prelude assembly. Single-file
   generation keeps its existing behavior. These consumers and all type-system
   identity helpers are inspection dependencies, not required edits.

This repair validates the builtin name; it does not newly tighten all possible
HIR identity/name pairs. In particular, the preexisting prefix/global predicate
and identity-less union lookup remain unchanged. General malformed-HIR identity
hardening is the retained Item12 invariant work, not a concealed B25 requirement.

Retained integration has two extra nominal-file changes: `module_bindings`
export and constructor-expression type discovery in `collect_hir_function_nominals`.
They originate in M1 commit `d726ffc11258c49f0185fd2d49697988cf90972c`, integrated
by `204118d35d4030bb543f14e090ee21f1bca7012e`. Neither is referenced by the token,
registry or original typed collectors. They solve broader Python/error-contract
behavior and must not be copied for this closure. Registry production and catalog
are unchanged in retained integration. Lineage alone is not the independence proof.

## Evidence reuse and consumed allowances

Historical focused receipt `/private/tmp/sifr-item12b.akguMz/builtin-focused.log`
has SHA256 `14a84f96f3b45c5c7330b6f1298628b8fa7e26feabe8993ce713cfe6e6284628`:
two `item12b_builtin_registration_*` tests passed at the original checkpoint.
`builtin-clippy.log`, SHA256
`704d5dda6160c56d1f3ed725da3ea4dbb636cc97dd344d85318be09dacc5ceab`, records the
historical codegen-crate Clippy pass. These are source-history evidence only.
The current-main extraction changes the surrounding compiler, dependency and
validation inputs; neither receipt qualifies the future candidate.

Original12B's [initial review](https://github.com/sifr-lang/sifr/pull/3694#issuecomment-5554250479)
approved the typed builtin mechanism within the broader exact
`b475ebdcd37081aa2860d9c348ace4100b546eff` to
`6ce83824e0315e5f89383fc666344b99431e1e76` scope. Raw response SHA256
`56cf098d3bea6502e9380d26e906c9ea53951023ac985c8c331b07d12a446e9f`.
The [sole remediation review](https://github.com/sifr-lang/sifr/pull/3694#issuecomment-5554470254)
approved changed corpus/coverage inputs at
`a3198ab9f936986b5ca1f9ce3fa73d36ac9ab74d`, preserving unchanged compiler
attribution. Raw response SHA256
`9a7e4448af6db3da62c78f91380d6b089d4f6a5f9c9a4f6901176e915b840d17`.
Both responses are under `/private/tmp/sifr-item12b.akguMz/opus-<SHA>.<suffix>/response.md`;
the exact paths and hashes are retained in B21's external source audit.

12C was incorporated and has no independent unused review/gate budget.
12B retains two consumed reviews and two FAILED gates (initial6ce838 and
replacementa3198); no third12B review/gate is granted. Original12K, B13, B14,
B15 and B20 counters/failures also remain unchanged. B21's assessment approval
cannot certify inherited code. There is no passing exact-SHA merge receipt for
the uncreated main-based candidate to reuse.

The proposed B25 qualification is a changed-input main integration: three source
files against assessed main, with the broader retained stack absent. It is not
an unchanged12B/B13 gate retry, nor approval of PR3694/3717. Historical review
supports unchanged mechanism reasoning; the fresh review must cover actual
extraction, caller compatibility, negative coverage and final inputs. No broad
review/gate exemption is permissible just to clear SQL's Clippy diagnostic.

## Next-item registration: 12K-B25 (unstarted)

Owner: compiler builtin-error registration delivery, emitted-code phase/12B.
Dependency: merged B21 assessment, the three immutable retained source refs,
and fresh read-only current-main comparison by one independent owner. Standing
user authority covers the bounded delivery workflow; no additional permission
is missing. B21 registers it but does not dispatch or execute it. Heavy work
must be scheduled with coordinator `01a07d84-7c62-77f0-b7c7-ecf310829a11`, after
dependency65's active broad-validation window for its single merge gate at
`4c7068b36904e216b02778f5664d2b8fd1159a6a`. Only that coordinator releases the
window. B21 read-only/docs validation/review may continue; do not change or
implement release policy.

Implement the COMPLETE three-file closure above, preserving current main's
descriptive names and unrelated changes. Expand regression assertions inside
these test modules if needed to meet the coverage below. No arbitrary string
registration, fallback, suppressed lint, skipped case, replacement panic, new
dependency, fixture/lockfile/workflow or unrelated compiler edit belongs here.
If actual source comparison reveals a further necessary mechanism, record its
precise dependency before broadening; do not quietly import M1 or B20.

Required positive/negative coverage and acceptance:

- Retain both `corpus_repair_builtin_registration_preserves_identity` and
  `corpus_repair_builtin_registration_rejects_nonbuiltin_names` from the
  retained registry. The first checks every catalog member, canonical paths,
  absence of bare-name entries and preservation of a crate-root module shadow.
  The second rejects custom, qualified, suffix-lookalike and empty names plus
  a module-qualified ValueError, and accepts canonical/identity-less ValueError.
- Complete those assertions with catalog `from_name` round-trips and stable
  re-registration, and rejection through `builtin_error_identity` as well as
  the token constructor. These are additional assertions in the same named
  tests, not an unimplemented test command or broader identity redesign.
- Retain main's `builtin_registry_identity_does_not_replace_a_module_qualified_shadow`,
  `builtin_error_union_keeps_its_shared_module_definition`,
  `emitted_builtin_nominals_join_the_project_registry`,
  `direct_io_error_kind_type_does_not_create_a_project_nominal`,
  `io_error_kind_handlers_do_not_create_dangling_project_nominals`, transitive
  nominal, same-basename and task-scope tests. Strengthen emitted registration's
  existing test to assert an absent catalog definition and a custom emitted
  struct create no builtin registration. Preserve supported global Rust
  identities through the existing union/task-scope tests and add explicit
  `sifr.parallel.WorkerError`/`WorkerRuntimeError` token and plan alias assertions
  to the retained registry tests; module-qualified lookalikes must be rejected.
- Both native driver shadow tests must compile and run, preserving different
  local/builtin field shapes in ordinary and test projects. Registry assertions
  alone do not establish emitted Rust/native correctness.
- Full affected codegen tests, strict workspace Clippy, formatting, HIR/file-size
  guards and the sole final merge-profile gate pass without weakening policy.
  A focused Clippy pass alone is insufficient delivery evidence.

Exact future commands (verified by source inspection; NOT run in B21):

```bash
cargo test -p sifr_codegen corpus_repair_builtin_registration
cargo test -p sifr_codegen
cargo test -p sifr_driver --lib tests::project_build_check::build_project_preserves_module_scoped_builtin_error_shadow_identities -- --exact
cargo test -p sifr_driver --lib tests::test_runner::test_project_preserves_module_scoped_builtin_error_shadow_identities -- --exact
cargo clippy --workspace -- -D warnings
cargo fmt --check
python3 scripts/check_hir_maintainability_guardrails.py
python3 scripts/check_file_size_guardrails.py
git diff --check <exact-base> <exact-candidate>
# After satisfactory exact-SHA integration review, once on that final candidate:
scripts/run_all_tests.sh --profile merge
```

The focused filter exists in retained12B/integration, not main before delivery;
the historical `item12b_builtin_registration` filter would select zero tests
after the rename and must not be reused. The full codegen run covers all other
listed nominal tests. Cargo manifests and module declarations support the two
exact driver test paths. The shell facade supports `--profile merge` and runs
the governed full profile, including its performance requirements; no reduced
or SQL-only gate is substituted.

Finish all implementation before named tests, then one draft PR and one
exact-base/candidate Opus integration review with at most one remediation,
then the single merge-profile gate on the approved final SHA. This is B25's
bounded changed-input integration allowance at dispatch, not a reset of any
predecessor. No create-pr gate when merging that SHA in-session, no second gate,
no third review. Reuse any subsequently available passing exact-SHA evidence
only after complete input/scope authentication. A second review's new mechanism
defect becomes a later owning item and terminal stop; recurring findings need
adjudication. External gate failures remain with their existing owners and stop
delivery; no new retry allowance or unrelated fix follows automatically.

Use an owned clone/branch/index/TMPDIR and default private Cargo target, leaving
`CARGO_TARGET_DIR` unset to avoid merging outer/nested probe caches. Initialize
the exact required submodules, ensure actual `origin/main` exists, inspect free
disk/private target before the long gate, and use only the owner's safe cleanup
authority. Authenticate receipts and ended processes at terminal. These setup
requirements do not make B20's semantic or performance changes prerequisites.

On pass: merge only approved B25 inputs, publish exact base/candidate/merge,
update the phase and report availability to the SQL coordinator, then stop.
Do not close PR3694, PR3717, corpus48, owner3776, or the larger emitted-code phase
on the strength of this narrow delivery. SQL's owner owns its own fresh validation.

## B21 validation and terminal record

B21's source audit, immutable identities, command-name checks, named documentation
validation and bounded Opus review live under `/private/tmp/sifr-b21.csIcGN/evidence`.
The [phase terminal record](ad-hoc-emitted-rust-excellence.md) now records merged
B21 PR3787, exact candidate/base/merge and all review/validation hashes. One
initial Opus review is SATISFIED, with no blocker or remediation. Only
documentation changed; no Sifr gate ran. B21 is complete and stops here;
B25 remains unstarted and dependency65 retains its active validation window.
