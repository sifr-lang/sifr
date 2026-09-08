# 12K-B49 — immutable stdlib bootstrap ownership

## Terminal disposition

Implementation and bounded source validation complete; **HOLD, representative budget FAIL**.
No PR, no merge, no full Sifr gate, no post-failure repair or external retry.
This record does not close the original emitted-Rust phase or authorize the next item.

- Exact base: `df0056ad1b9d7fcbaacda3a3f0809da66279ea84` (held B46 stack).
- Reviewed and measured source: `30b25c551566bf0c146290c7ec38a55621c5d526`.
- Source branch: `codex/item12k-b49-4WCpNp`, pushed to origin.
- This separate record branch: `codex/item12k-b49-record-4WCpNp`.
- Main observed before and after acquisition: `4b4cc339964baeeb6641e57dc669fef700a5fa24`; do not assume the held stack is on main.
- Owned independent HTTPS clone: `/private/tmp/sifr-b49.4WCpNp/sifr`; independent index, refs and objects, no alternates.
- Owned evidence: `/private/tmp/sifr-b49.4WCpNp/evidence`.
- Parent checkout, ledgers, predecessor source, caches and evidence stayed read-only.
- Owner: issue [3776](https://github.com/sifr-lang/sifr/issues/3776).

The latest concrete onboarding limited this item to implementation, named tests,
one exact-source Opus review plus at most one remediation, and one separately
released canonical representative acquisition. It superseded generic merge gates.
Police released source validation, ordinary locked binary preparation, and finally
the exact frozen representative command in separate coordination steps. Item41
provisioning/lock coordination preceded the heavy window; Item42 remained source-only
until this worker's actual process release.

## Complete ownership boundary

The source-supported redundant copies identified at B47's residual boundary were
removed coherently through the actual consumers, not attributed to a numeric cost:

- `StdlibEmissionView` borrows accumulated emission maps. Its bootstrap view retains
  the existing empty prior-module-source map. Module/project/deferred/support/demand
  consumers preserve that view through every actual module-source reader.
- Imported generic class selection remains at the structural consumer with the same
  explicit import-name predicate and order, removing bootstrap's intermediate copy.
- Generic HIR class templates are shared immutable `Arc<HirClass>` values through
  metadata, RustEmitter and aliases; newly owned templates transfer once.
- Canonicalized module HIR transfers into `Arc<HirModule>` immediately; pending private
  contracts share that HIR and borrow loaded source until full source-ordered contract
  construction completes. Canonical HIR is not deep-cloned for the pending inventory.
- The global cache holds `Result<Arc<StdlibCompiled>, Vec<RenderedDiagnostic>>`.
  `StdlibCompiled` no longer implements Clone. Rooted entrypoint, frontend and test
  assembly retain the shared bundle; mutable lowering copies only its required defs
  projection. Post-consumption interop consumers retain the bundle and borrow its plan.

Full parse/lower/canonicalization/emission/validation, private contract and trust
checks, diagnostic timing/order, source-ordered demand and retained snapshot lifetimes
remain intact. No inventory pruning by first caller, persisted cache, lazy failure,
fallback, semantic/codegen shortcut, fixture/budget/workload change, formatter change
or editor Wasmtime change was introduced. B46's previous HIR-map/discarded-plan/visitor
repairs are predecessor work, not claimed here. Architecture ownership documentation
was updated with the implementation. The exact base-to-source diff has 36 paths.

## Named source evidence

All implementation and test inputs were authored before the first test at
`96511e20a159f93806d4af24580a17e144276c94`. A named codegen test compile exposed one
missed cfg(test) caller (`E0308`); its failed receipt is retained. The final one-line
test-only migration to `emission_view()` produced source `30b25c551...`.

**37 named tests across 29 Cargo filter invocations and four guards passed.**
The final SHA reuses 35 driver/analysis results: the only subsequent change is in a
codegen-local cfg(test) file excluded from their production dependency/test inputs.
Both named codegen tests and all four guards passed on the final SHA. The exact
argv, counts, source bindings, failed receipt and log hashes are in the attached
source-validation artifact and the owned `named_checks.py` registry.

Driver command prefix: `cargo test --locked -p sifr_driver --lib FILTER -- --test-threads=1`.
The following exact filters were registered before edits:

- `stdlib_interop_startup` (4), `stdlib_interop_demand` (5).
- `private_stdlib_interop_resolves_sysroot_crate_target`.
- `merged_user_and_private_stdlib_interop_both_resolve`.
- `merged_user_and_private_stdlib_interop_keeps_user_trust_separate`.
- `package_rust_interop_direct_probe_checks_signature_shape`.
- `package_rust_interop_direct_probe_accepts_bridge_signature`.
- `package_rust_interop_direct_probe_rejects_unsafe_fn`.
- `package_rust_interop_opaque_probe_rejects_unsatisfied_send_obligation`.
- `package_rust_interop_cache_fragment_is_deterministic`.
- `package_rust_interop_cache_changes_with_trust_policy`.
- `private_stdlib_imports_resolve_only_from_compiled_source_exports`.
- `missing_private_stdlib_member_is_a_structured_bootstrap_failure`.
- `missing_private_stdlib_module_is_a_structured_bootstrap_failure`.
- `test_get_or_init_stdlib_cache_reuses_` (success and error).
- `stdlib_class_exports_preserve_parent_markers_and_generic_templates`.
- `stdlib_structural_templates_retain_signatures_without_bodies`.
- `stdlib_bootstrap_borrowed_emission_preserves_imports_and_generic_templates` (new).
- `stdlib_cache_shared_bundle_outlives_cache_and_keeps_definitions_isolated` (new).
- `private_interop_pending_inventory_preserves_lifetime_and_source_order` (new).
- `test_check_project_imports_generic_function_metadata_through_facade`.
- `test_build_project_imports_generic_function_metadata_through_facade` (plus `--include-ignored`).
- `test_build_project_keeps_aliased_same_name_generic_classes_distinct` (plus `--include-ignored`).
- `test_check_and_project_lowering_share_typecheck_rules`.
- `test_collect_project_modules_allows_non_main_stdlib_imports`.
- `test_imported_python_object_identity_reaches_check_and_compile`.

Analysis used the same locked lib/test-thread pattern with filter
`stdlib_interop_startup_editor_retained_snapshot_after_defs_projection`.
Codegen used `private_alias_registration_preserves_local_signatures_without_overwriting_origin`
and `public_import_roots_seed_matching_transitive_private_definitions`.
Guards: `cargo fmt --check`, `git diff --check`,
`python3 scripts/check_hir_maintainability_guardrails.py`, and
`python3 scripts/check_file_size_guardrails.py`.
Largest touched hand-maintained source is 899 lines, below the 900-line limit.
No broad crate/profile test, Clippy or full gate was run.

New coverage demonstrates shared-bundle lifetime after cache destruction, local defs
mutation isolation, real subsequent lowering/emission, pending private HIR lifetime,
non-lexical source order and complete signatures/source attribution, and borrowed
bootstrap generic/alias emission without nested prior module source. Existing tests
cover private failures, trust isolation, demands, project/native generic builds,
test-assembly metadata and retained editor snapshots. No allocation-count mirror test.

## Bounded exact-SHA review

One initial logical review, one provider attempt, zero provider errors, zero remediation
reviews. Model `claude-opus-5`, effort `medium`, read-only plan permissions, project
settings only, no session persistence, atomic private response, 2400-second watchdog.
The reviewer received exact base/candidate/changed paths/scope/named test evidence and
was instructed not to run tests or invent new requirements.

Verdict **SATISFIED**, no blocking findings, elapsed 253.837 seconds with owned process
release. Full exact-source review is outside the reviewed Git tree:
`/private/tmp/sifr-b49.4WCpNp/evidence/opus-review.30b25c551566bf0c146290c7ec38a55621c5d526.md`.
SHA256 `6930229617a45644b9b94330ae923e416af0394d3f4c434f6d759496ee917784`.
Published [review and source evidence](https://github.com/sifr-lang/sifr/issues/3776#issuecomment-5590283339).
No final review was committed into the approved source SHA. Predecessor review counters
are unchanged; B46's initial/remediation allowance remains exhausted.

Nonblocking review suggestions are later owner3776 work, not implemented here:
make the empty-source override harder to bypass through future Deref-coercion helpers;
consider uncached test-bootstrap cost only if measured; comments/style/cache-init
deduplication. Current actual source readers are correct. The pre-existing unused
StdlibCode test import remains a warning, not a new mechanism or remediation.

## Frozen representative acquisition — FAIL

Locked ordinary own `sifr` and `frontend_query_bench` binary preparation passed with
own test/preparation TMP and target. Prelaunch free space was 143911960576 bytes;
private target was about 15.69 GiB, with AC power and no thermal warning. Ordinary
desktop/background load was disclosed; canonical host admission/monitoring remained
authoritative and no apps/services were modified.

After explicit Police freeze authentication and release, exactly one command ran:

```sh
env -u CARGO_TARGET_DIR TMPDIR=/private/tmp/sifr-b49.4WCpNp/qualification-tmp \
python3 /private/tmp/sifr-b49.4WCpNp/evidence/run_receipt.py representative-01 1800 \
uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative
```

Qualification TMP was distinct and pristine; actual generated-artifact cache entries
were 0 before and 9 after. Source, both binaries and all 221 protected inputs were
verified unchanged after the run. Binary hashes:

- sifr: `41203eb21fcfe364925bdcc57309b0dbdcb8d5aef65bcbd2fb353e0aa6e26a12`.
- frontend_query_bench: `384e072c4f599b20d989545a333cf1bac86305e61bb14bf0d13bbbd730dae71f`.

Canonical invocation `performance-representative-1788894418571178000`;
raw run `bench-1788894418-36219`. All 10 cases accepted, six rules passed, producer
exit 0. The **actual** budget checker exited 1 with the SAME invocation ID:

```text
verification/.venv/bin/python verification/areas/performance/check_budgets.py
--results target/performance/representative.budget.latest.json
--allow-subset --expected-invocation-id performance-representative-1788894418571178000
```

The `representative.budget.latest.json` file is producer input to the checker,
not its verdict. Actual argv/exit are retained in
`target/verification/areas/performance-results.json`; full checker output is in
`/private/tmp/sifr-b49.4WCpNp/evidence/representative-01.log`.

| Actual failed metric | Measured | Unchanged threshold |
| --- | ---: | ---: |
| build-single break/continue RSS bytes | 185958400 | 181649408 |
| check-project graph instructions | 17467799652 | 13938915299 |
| check-project graph RSS bytes | 185860096 | 179601408 |
| check-single arithmetic instructions | 17409689048 | 13879420485 |
| check-single arithmetic RSS bytes | 185434112 | 181436416 |
| JSON diagnostic instructions | 17410040211 | 13874003870 |
| JSON diagnostic RSS bytes | 185630720 | 180748288 |
| formatter-corpus instructions | 46165395 | 38059926 |
| LSP diagnostics RSS bytes | 151437312 | 108593152 |

Instruction failures also exceed their thresholds after the canonical uncertainty
deduction; exact lower bounds and uncertainties are retained verbatim in the summary.
Four cases have no failing budget metric: build-project, large formatter, incremental
unchanged-file query and warm diagnostics query. This does not waive the nine failures.

The original large-formatter attempts 1 and 2 were rejected for work CV 0.026197 and
0.033774 against 0.02; attempt 3 was accepted at 0.012589. All other cases accepted
attempt 1. All **117 raw receipts** and sidecars are retained (including rejected
attempts); 316 evidence files are hash-indexed. Original command counts are 1 warmup
plus 5 measured; query aggregate counts are 3 warmups plus 20 measured, with 20
separate work processes. No external retry, profiler or prewarming was introduced.
Darwin RSS units are bytes, not KiB or footprint. Work-process and query-aggregate
measurements have different boundaries and are not cross-normalized or attributed.

Actual case limits remained 120 seconds BUILD, 60 CHECK/FORMATTER/LSP and 30 FRONTENDQUERY.
Outer limit 1800 seconds plus at most 120 cleanup (1920 total within 7200).
Actual elapsed 193.596 seconds, exit 1, no timeout, `release_pass=true`, `remaining=[]`.
The heavy window was released to Police immediately at terminal. No source change,
repair, qualification retry, gate, PR or merge followed the failure.

## Evidence custody and retained ownership

Attached machine-readable records:

- [Source validation](ad-hoc-emitted-rust-b49-artifacts/source-validation.json), SHA256
  `add66d3cd1cbb6f289ce45f6ad029ba962c5fda3a9de8680429c51ba6ad39571`.
- [Qualification summary](ad-hoc-emitted-rust-b49-artifacts/qualification-summary.json), SHA256
  `bbd46ee4118fca6adb3bdb5b7797e6d6cc906bfb3ea34b2f000d60ef03a011e2`.
- [Raw evidence inventory](ad-hoc-emitted-rust-b49-artifacts/qualification-inventory.json), SHA256
  `3e374a8594c5d8eb1026634d12e5b9ad5f53ebfe0722ff361e67683202f56d81`.
- Private freeze `freeze.30b25c551566bf0c146290c7ec38a55621c5d526.json`, SHA256
  `911ed746f78747727e902bd03ccc5a257903b5c9e5dd4901a676f940cb381a81`.

Unchanged B46 formatter14 and immutable IR visitor evidence are explicitly reused,
not rerun. B46 source `df0056...`, report record `bcbd5c8c71d1cb74ef849bd5c74fa104e149dcd5`,
and its terminal SHA256 `302f407be75cfaadc5f72ca714298f4a7f94163eed4d8588be552f815532a87d`
remain authoritative. B47's bounded source assessment (430 selected bindings/111 raw)
is reused without broad history scans; report SHA256
`a7e4721ac08c81596e934f375d63b551624bf99dcb800c65a3d7e8c6078b13a1`.
B48's closed control-only terminal SHA256
`cf9949b1504d91f62e80c18eb9c98eba3073d590bb879c4e0ae7f595b9a5b860`
supplies zero cost attribution. Historical baseline and all budgets remain unchanged.

The pre-existing `test_runner/artifacts.rs` empty runtime InteropBuildPlan remains
later B27 ownership: passing test-assembly metadata evidence is not runtime interop
execution delivery. Genuine list-repeat, smallest qualified builtin integration,
broad B27/full corpus, SQL qualification and other owners remain separate and open.
No historical f8 lookup, profiler retry or custody-harness repair was attempted.
The parent owns the top-level phase-ledger handoff; this worker did not edit it.
Final disposition: source repair complete, semantic/review evidence passed,
performance qualification failed, integration held. **Stop; no next item.**
