# 12K-B52–B54: performance source batch and terminal qualification

Status: source implemented, focused tests PASS, grouped review SATISFIED;
canonical controlled acquisition FAILED; source UNMERGED / full gates HELD.
Owner: compiler/performance, evidence issue3776. Date: 2026-09-08.

## Fixed scope and source identity

This worker owns only B52–B54. The latest user direction supersedes the older
maximal-worker queue and per-item review boundary: one coherent batch, one
grouped exact-SHA review plus at most one remediation, one combined canonical
acquisition, then terminal handoff. No next-item source, new acceptance criterion,
baseline weakening or corpus reduction. Optional suggestions are in a
[separate backlog](ad-hoc-emitted-rust-optional-review-backlog.md), not phase
requirements. Existing integration and whole-phase obligations remain unchanged.

Independent HTTPS clone `/private/tmp/sifr-production-batch.MItiGD/sifr`, own Git
objects/index/refs without alternates, own target/TMP/evidence. Parent and all
predecessor worktrees, ledgers, caches and evidence remained read-only. Actual
production base B49 `30b25c551566bf0c146290c7ec38a55621c5d526`; no B50/B51 temporary
instrumentation ancestry. Observed main remained
`4b4cc339964baeeb6641e57dc669fef700a5fa24`, not this held source graph.

Pushed source branch `codex/production-batch-b52-MItiGD`, final reviewed SHA
`6a59be12ec4076e2801652ffbf99250e67e5a258`. This separate documentation record
starts directly at B49 and does not promote the unmerged source changes.

| Item | Final item SHA | Focused evidence | Source outcome |
| --- | --- | --- | --- |
| 12K-B52 | `dd691bf2759c96f2d8cd5cec96ca552109e9b698` | 13 filters, 26 cases PASS | Complete session-owned inline support validation reuse |
| 12K-B53 | `fb7bcc2578352453cfc9e144f7f9d8f3202b9962` | 9 filters, 15 cases PASS | Demand-aware SQL editor host lifecycle |
| 12K-B54 | `6a59be12ec4076e2801652ffbf99250e67e5a258` | 5 commands, 30 cases PASS | Remove discarded formatter token projections |

B53 has no semantic dependency on B52, and B54's mechanism is independent;
their shared branch was explicitly registered as a pending-review combined graph.
The final review covers the entire graph. All three remain unmerged and cannot
be represented as independently production-qualified by these focused tests.

## Implemented ownership contracts

B52: the Renderer accepts raw Rust, so IR validity does not replace syntax
validation. `sifr_codegen/src/inline_syntax.rs` owns a bootstrap-scoped exact-content
support session. It tokenizes the complete assembly, parses inner attributes and
ordinary items with syn, and learns support only after complete success proves
its exact source range consists of closed top-level items. Reuse requires an
actual syn item boundary and whole token trees exactly covering that range.
Unproven boundaries receive ordinary parsing, not error recovery; failed parses
are never cached. BOM/shebang syntax remains under `syn::parse_file`. Body/import/
attribute/raw-fragment validity, error mapping and rendered bytes/order remain.
Every existing stdlib module is still emitted and validated. The driver owns the
session across the complete bootstrap inventory; generic/structural metadata and
private support contracts are retained. No global or persistent cache, no debug-only
check, unused-module omission or grammar approximation. Architecture updated.

B53: `sifr_analysis/src/sql_editor_host.rs` reconciles host demand from prepared
profile registry entries. Empty registries do not allocate a ComponentHost.
Configured initialization stays eager and fallible. Enabling constructs before
publishing profiles; configured reload retains the host; disabling drops it.
`replace_profiles` propagates initialization failure through the existing
structured diagnostic. Query cancellation, incremental caching, source trust and
snapshot semantics remain. The missing-host invariant guard returns an error,
not an unwrap or swallowed initialization failure. No RSS-saving claim.

B54: two validation-only calls in `sifr_format/src/lib.rs` now use the existing
`parse_module_raw` instead of `parse_module`, which additionally allocated debug
strings for every token and a wrapper vector that these callers discarded. The
complete parse and all invalid/unsupported syntax checks remain. Source/range
output and errors are preserved. Discovery rules, root-relative/ancestor invariance,
escaped/compound selection and cache-off semantics are unchanged. B46's existing
discovery optimization was not reimplemented. No diagnostic or budget adjustment.

## Named tests and unchanged-input reuse

Every item's complete source and test inputs were authored before its tests.
Exact argv, candidate, exit, counts and raw logs are in external
`source-validation.6a59be12ec4076e2801652ffbf99250e67e5a258.json`:
SHA256 `ce9e4a3c008cad7c14b14b25bdf5a937593192f43211ca000b42bd67a8f62f73`.
71 named case executions, not 71 unique tests, plus four final guards PASS.
Registrations and all failed/successful receipts are retained in the evidence root.

B52 commands use `cargo test --locked -p CRATE --lib FILTER -- --test-threads=1`.
Codegen filters: `inline_syntax_tests`, `support_assembly_codegen_tests`,
`structural_stdlib_impl_codegen_tests`,
`private_alias_registration_preserves_local_signatures_without_overwriting_origin`,
`public_import_roots_seed_matching_transitive_private_definitions`,
`generic_class_addable_methods_emit_their_support_contract`.
Driver filters: `stdlib_bootstrap_borrowed_emission_preserves_imports_and_generic_templates`,
`stdlib_structural_templates_retain_signatures_without_bodies`,
`stdlib_bootstrap_syntax_session_preserves_full_inventory_and_source_order`,
`private_interop_pending_inventory_preserves_lifetime_and_source_order`,
`stdlib_cache_shared_bundle_outlives_cache_and_keeps_definitions_isolated`,
`stdlib_interop_startup`, `stdlib_interop_demand`.
New oracles compare valid and invalid assemblies with the trusted full syn parser,
including raw boundaries, imports, attributes, generic/structural support, source
ordering, failure non-publication and complete reverse-order bootstrap inventory.

B53 same locked serial lib form. Analysis filters: `sql_editor_host::tests`,
`sql_editor_runtime_tests`, `sql_editor_tests`, `sql_incremental_cache::tests`,
`stale_snapshot_is_rejected_after_update`,
`stdlib_interop_startup_editor_retained_snapshot_after_defs_projection`.
Frontend: `cancellation_is_checked_at_every_provider_boundary`.
Driver: `package_compilation_prepares_profiles_offline_and_binds_source_bytes`,
`package_compilation_denies_ambient_schema_component_capabilities`.
Coverage includes actual lifecycle, empty/configured/enable/reload/disable,
injected initialization failure, drops, diagnostics, cancellation and snapshots.

B54: full `sifr_format --lib` (11), `sifr --bin sifr formatter_discovery` (7),
`sifr --test formatter_discovery` (9), and analysis lib filters
`analysis_snapshot_carries_workspace_state_and_query_metadata` (1) and
`sql_editor_runtime_tests` (2), all locked/serial. New full-parser format oracles,
invalid ranges, repeated cache-off and invalid-source preservation tests retain
all existing discovery contracts.

After B52 only analysis/formatter/CLI-test/docs inputs changed. Analysis and
formatter are not dependencies of codegen/driver; B52 evidence is reusable on the
final graph. SQL source is unchanged after B53, and final rebuilt analysis SQL
and snapshot formatting checks cover its changed formatter dependency. Final
guards: cargo fmt, B49-to-final diff whitespace, HIR and canonical file-size.
No broad full-crate/profile/Clippy or create-PR/merge gate was run.

Retained failures: initial B52 reuse test input did not demand support; its
replacement incorrectly assumed indexing returned Result instead of Option.
Both test-input receipts remain; final input uses an existing explicit ValueError
contract with required reuse assertion intact. No source mechanism correction.
B54 harness initially refused to overwrite B53's identically named receipt;
the fifth command had not launched. It alone ran under a fresh receipt key and
passed on the same SHA; the four previous passes were reused. No erased failure.

## One grouped exact-SHA review

Opus SATISFIED with no blocking findings at final source SHA; one initial request,
zero remediation reviews, zero provider errors. Read-only plan/project settings,
claude-opus-5, medium effort, no persistence; atomic response outside Git and
2400-second watchdog. Actual elapsed 362.245 seconds; owned release PASS.
[Full published review](https://github.com/sifr-lang/sifr/issues/3776#issuecomment-5591731875).
Raw `opus-6a59be12ec4076e2801652ffbf99250e67e5a258.N3L5Km/response.md`, SHA256
`a18d0099bd0d289abc4846ad6c8d89a53c38808b3235c3c5a26ffa3027211a65`.
No source changes after review. Approval is not performance acceptance or merge.

## Single canonical acquisition: FAILED, no external retry

Police explicitly released the frozen combined candidate after dependency workers
reported all resolver/download/native processes released and new bursts paused.
Private target 7.3 GiB, free 102366699520 bytes, AC power, no thermal warning,
10 logical CPUs; this is admission evidence, not a zero-load claim. Ordinary
locked binary/verification preparation preceded measurement. When own target
crossed 20 GiB, absence of users was checked and only this target was cargo-cleaned;
final binaries were rebuilt and frozen. No foreign cleanup or global tool changes.

Frozen source clean, both binaries and all 221 protected inputs identical to the
registered freeze; protected inventory exactly matches B49. Distinct pristine
qualification TMP. Freeze SHA256
`8dfe095fb9638e71c16729a75133fe604209990671f5ed6637edd54a1185ae9c`.
Sifr binary SHA256 `4378f0289ec40181fb61b801784090f61f4e328224ef9ad566e1b1874fd32a69`;
frontend helper `8f1259814e05b17a6254637e0650c6d8e98ab2371d43bf9f050c25dc4473eb08`.

Exact unchanged command:
`uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative`.
Original 10 cases, six rules, counts/warmup exclusion, internal maximum three
attempts, 120/60/30-second case limits. One outer 1800-second limit plus at most
120 seconds owned cleanup, within the existing 7200 allowance. No extra diagnostic,
subset run, warmed replay, repair-and-rerun or user-application intervention.

Actual invocation `performance-representative-1788901704922050000`.
Supervisor exit1 after 154.790451292 seconds, no timeout, no cleanup signals,
remaining owned processes empty, release PASS. Police notified immediately.
All six rules PASS. Five preceding cases accepted their first controlled attempt:
build-project additional-modules, build-single-file break-continue, check-project
project-graph, check-single-file arithmetic, diagnostic JSON schema. Formatter
corpus then exhausted all three original attempts:

| Attempt | Instruction CV | Existing limit | Disposition |
| --- | --- | --- | --- |
| 1 | 0.036634 | 0.02 | unstable-samples |
| 2 | 0.038932 | 0.02 | unstable-samples |
| 3 | 0.044864 | 0.02 | unstable-samples |

Concrete existing blocker: the unchanged controlled work-sample stability
criterion in `verification/areas/performance/benchmark_manifest.py:55`, enforced
by `controlled_sampling.py:27,82,138`, was not met by
`formatter-corpus-001-project-check`. Owner remains performance / issue3776.
This is a failed canonical acquisition, not a newly invented requirement or a
proven compiler mechanism defect. No attribution of instability to a process.

The actual `benchmark-subset` exit is1. The actual `budget-subset` has empty argv,
null exit, and `blocked-by-benchmark-subset`: the numeric checker DID NOT RUN.
No producer latest JSON was written. Four cases were unreached: formatter-large,
incremental unchanged-file update, warm-diagnostics query and LSP diagnostics.
No actual budget pass or numeric budget-failure count can be inferred. In
particular B49's nine original budget failures remain binding, not reclassified
as this run's sampling failure. Partial or rejected metrics are observational,
not performance savings, complete qualification or permission to promote source.

## Sealed evidence and terminal handoff

External evidence root `/private/tmp/sifr-production-batch.MItiGD/evidence`:

- `qualification-summary.json`: `50131d1d1357fed5e10690ea6a99b9dea8b2f1ecb02b20f6202b9f189975a36e`.
- `qualification-details.json`: `66737921c9cac1a28d840b900d4d76f65fd6c8fcaa7b5b754ee054d44226dc17`.
- `qualification-inventory.json`: `b9017fa1bee244e0302268abb77c5e6d92ba63221ac9eddec97049dcddf903ce`.
- Actual area result: `1b5048fb123b0715e53c056fec7e7e8b712b6f7c7b10e3ea43ba20b96441bc80`.
- Formatter control failure: `f4202576aa0f5c4f27364f341d62cec009901bdc7381d1a6af82d246ff81eac8`.

109 hashed canonical files, 48 raw sample receipts and 48 full stderr sidecars
verified. Immutable summary has zero finalized producer rows and null producer
ID because producer output was absent; supplemental details preserve actual argv
identity and the five partial accepted attempts without rerunning any checker.
Freeze integrity PASS, no changed inputs. All failure evidence retained.

Per-item implementation/tests/review are complete; combined canonical performance
qualification is blocked as above. PR: none. Full gates: zero. Merges: zero.
Full merge gate and PR publication/promotion remain held for the parent-owned
smallest complete integration candidate and original prerequisites. This worker
does not rerun exhausted gates, begin another item or perform whole-phase closure.
Exact next action: parent/Police retain this reviewed-source handoff and
criterion-bound failed acquisition; a fresh separately scoped worker owns any
authorized subsequent package. Batch worker stops after record publication.
