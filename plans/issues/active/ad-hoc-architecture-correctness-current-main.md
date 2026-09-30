# Architecture correctness and agent workflow: current-main supersession

Status: active; Preparation/F33 complete; residual delivery open
Audit baseline: main 3db97f2f7e354145224a582018cd2a978593a356; reconciliation assignment 2026-09-23
Historical branch: codex/architecture-audit-closure-m12k at 46618f73f1d98ffdb77403202f645c8da8250ff6

## Authority

This is the canonical current-main record for the residual architecture audit. It supersedes the execution order and completion claims in the [unmerged historical phase](https://github.com/sifr-lang/sifr/blob/46618f73f1d98ffdb77403202f645c8da8250ff6/plans/issues/active/ad-hoc-architecture-correctness-and-agent-workflow-closure.md). Preserve its branch, PRs, review receipts, failed gates and raw logs as history. None are current-main integration evidence.

The crosswalk uses F01-F34 in the supplied final recommendations (/home/yaser5/projects/sifr/architecture-closure-inputs-20260923/final_recommendations.md, SHA-256 bb2370e7685700456aba35f8b56b11c3ef2433ab6ccba2a1f7254fb8d70a894f). That report contains pinned source links and evidence limitations. Reported Rust reproductions were not rerun for this record. Each row below has one acceptance owner. Existing-owner rows are handoffs, not duplicate implementation authority. A completion claim needs the owner's merge SHA, exact validation and review evidence. A repaired prerequisite is not a passed dependent qualification.

## V01/F05 live selected-reference qualification (2026-09-28)

On exact merged main `8e8ec4255cf4ec3ffcab577adfc100efa8aa4a7b`, the
immutable `linux-i7-4720hq-12gb-dev-v1` reference was admitted live on its
captured Linux host. The selected reference digest was
`a33f46702c3e16b4f7e511a155b9c7a06880c19e703611e071e1b86103cee91d`;
Python 3.14.7, Rust/Cargo 1.98.1, two Cargo jobs, root ext4 target and temporary
storage, and the captured CPU frequency policy matched. The approved idle-host
window changed all eight CPU governors from `schedutil` to `performance` for
the run and restored `schedutil` afterward. The benchmark report records a
clean source tree, controlled latency mode, and no competing build process.

The exact `python3 verification/areas/performance/runner.py --suite representative`
selection passed **8/8** area variants with zero failures. Its producer ran
all **10/10** selected benchmark cases with the manifest sample counts and no
timeouts; the selected-reference subset budget check passed. Seven five-sample
cases did not claim p95 qualification because the policy requires at least 20
samples for that metric. The standalone live admission log has SHA-256
`1789864d62da777d2978a10c787f414eaed63832c8c007923bfd8f2fbd498b34`;
the area log, result JSON, benchmark report, and trend report have SHA-256
`3434e0a0805a24c7243e618815964e6d3ccb1e4adcb3690b66fd96be9450d642`,
`01a242ddce06a2857543bdd76323ba670fe64fa0fe3ec9e1d4d0b532a3c2aef4`,
`cd5226dfa7c7cd8343cce113a761278856131c85a6794037c4f7a18b9aacc6e8`,
and `7bac51a16e5fd9beb8b9b84e66587c5587bc0860c696f5111e84ff797a78e298`,
respectively. Raw candidate-keyed evidence is outside the Git tree at
`/data/sifr-architecture-v01-live-qualification-evidence-20260928/8e8ec4255cf4ec3ffcab577adfc100efa8aa4a7b/`.

The first standalone admission probe rejected the default tmpfs temporary
storage as `execution.temporary_storage` (raw log SHA-256
`d0255739737fcd60dee835a5cdd50d785f2476df071a484a0269eee7dcd9139a`);
admission passed after setting temporary storage on the captured root ext4
device. The first representative run then failed during setup because the new
worktree lacked the pinned Ruff submodule (raw log SHA-256
`37c49c7422161d68b4c91dc73526d3caf5d9d087187a7e4c1da232b906e81859`).
After initialization, the cold compiler build exceeded the benchmark runner's
180-second preparation deadline before measurements (raw log SHA-256
`f95cba8a8718decdef47e14d8378975725603fcdb20dc2cd66ecd8d062aac832`).
The compiler and frontend query helper were then built in the same owned Cargo
target; the passing run reused those warm artifacts. All three failed probes
and attempts remain separate from the passing evidence. The standalone probe
and passing run retain governor before/during/after receipts; the two failed
representative attempts retain their logs and exit statuses, while their
governor snapshots were overwritten by the passing run.

The record-only delivery is [PR #4058](https://github.com/sifr-lang/sifr/pull/4058).
This closes V01/F05's outstanding live admission and named representative
qualification prerequisite. Q01 still owns the final exact-candidate full
merge profile and whole-phase integration decision.

The record-only documentation structure check passed (raw log SHA-256
`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`),
the 900-line file-size guardrail passed for 4,245 files (raw log SHA-256
`2e10821206a8db09bbb2a00e21e9618f4f9f216f73b5c56afcd7ce13f9e819d2`),
and the diff check passed.

Scoped read-only Opus review of the earlier record candidate
`443965362753f49c576bcb02847066e286517dd5` found the omitted first
admission rejection and imprecise governor-receipt wording. Both are corrected
above; its response is preserved outside the Git tree (SHA-256
`94331217faaa3355b7c35f68c3d1f9351ccf600f6da3eb64c3ffa3044cf8c21a`).

## TypeScript-Go guardrail taxonomy blocker (2026-09-27)

The Emitted-Rust final qualifier's exact-main
`b5fb4b010ad485f521505382edcd6d9f50d045bb` full merge profile first
failed at `area_coverage_matrix` / `readiness/verification_taxonomy`.
The current `internal_docs/typescript_go_architecture_transfer_guardrails.md:55`
begins `The current-main V03/E01 CLI/driver follow-up`; the active-surface
taxonomy rule rejects that delivery label. The line was added in the
Architecture-owned TypeScript-Go inventory [PR #4041](https://github.com/sifr-lang/sifr/pull/4041)
(candidate `d36610a6694d3f23c5d71782b306801f88977755`). This is a
documentation wording defect in this issue's owned guardrail document.
Its owner should replace the delivery label with descriptive ownership text,
then run the exact `readiness/verification_taxonomy` selection and the
coverage-matrix readiness suite. No compiler, inventory or taxonomy-rule
relaxation is indicated. The Emitted-Rust qualifier did not change this
document; its full merge gate remains unqualified until the repair merges and
the gate is rerun on the resulting exact main.

The failed gate log, lane and coverage result are under
`/data/sifr-emitted-final-qualifier-retry-evidence-20260924/final-b5fb4b01/attempt13/`
with SHA-256 `c44e08960221342bfe2370939d74a0445eb418f1d3701dad11d070a5402a289e`,
`2a6541f5c7a8901437df81b779a9024676f7ac5fb363a50f9dc0073448aa43df`
and `ee64dd71f684a8c03474c9b683c3db46d2b8c70a1dadb205000976f78259dbdf`,
respectively. V01 reference admission, generated setup and preceding
guardrails passed; the approved governor window restored `schedutil`.

## TypeScript-Go guardrail taxonomy repair delivery (2026-09-27)

The Architecture-owned guardrail wording repair merged in [PR #4048](https://github.com/sifr-lang/sifr/pull/4048)
as `f8ba9c5728b5e3f1a877c70c4147b1ce771371d8` from exact tested and
reviewed candidate `836be73c03f3957842e27e1e1b644c9e1e49a4f7` (base
`b5cfa062924e3b52faebbf6a3d1026759566b505`). Guardrail line 55 now
names the CLI and driver transfer inventory by ownership instead of the
V03/E01 delivery label. The 32 scanner observations, their five paths,
pinned base and production/test split are unchanged. Only the Architecture-owned
source document changed; no taxonomy rule, inventory row or compiler code did.

On the exact candidate, `coverage_matrix`'s selected
`readiness/verification_taxonomy` case passed 1/1 (raw log SHA-256
`b37331b98c12c12e78533e0333d661214a1bafc0640bb76a6b72928854081aba`),
and its readiness suite passed 4/4 (raw log SHA-256
`dd3ba3f4f4b48aac84e13b0356b8480f348a6fa228c2a418be824694d229034a`).
Documentation structure passed after initializing the pinned nested editor
submodule (raw log SHA-256
`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`);
the two earlier missing-submodule setup failures remain preserved. The
900-line file-size guardrail passed for 4,245 files (raw log SHA-256
`2e10821206a8db09bbb2a00e21e9618f4f9f216f73b5c56afcd7ce13f9e819d2`),
and the committed diff check passed. Scoped read-only Claude Opus 5.5 review
returned **SATISFIED** with no blocking findings (response SHA-256
`9cc74432d7faa0f71210a9666b8fefaedfbbce09bb68daa1034044f37215fbec`).
Candidate-keyed raw logs and the review response are outside the Git tree at
`/data/sifr-architecture-tsgo-taxonomy-wording-evidence-20260927/836be73c03f3957842e27e1e1b644c9e1e49a4f7/`.
The reviewer noted the pre-existing accepted delivery label at guardrail
line 66 as a nonblocking documentation follow-up for this guardrail owner.
The Emitted-Rust qualifier still owns the final full merge-profile rerun on
merged main; this focused pass does not qualify the remaining gate.

## V03/E01 developer-tooling integration blocker (2026-09-27)

The Emitted-Rust qualifier's exact-main `a04cd9c49aaabdac93addb2c8e6a0a964aeae759`
merge profile passed diagnostics, runtime platform, and algorithmic
compatibility, then stopped in `area_developer_tooling`. Its first selected
failure, `typescript-go-transfer/typescript-go-transfer`, found 32 new
unlisted direct-read/probe sites in `crates/sifr/src/formatter_cache.rs` and
four `sifr_driver` files (`build/cargo_resolution_identity.rs`,
`cache_storage_windows.rs`, `project_cache/housekeeping.rs`, and
`test_runner/execution.rs`). The same guard still requires `Session` to own
the persistent LSP analysis workspace. The separate
`static/direct-filesystem-effects` check reports 165 unclassified and 72
stale site observations, including cache storage, test-runner and sysroot
paths. Both guard self-tests passed, so this is current source/inventory drift,
not a disabled scanner. V03 owns effect classification; the current CLI/driver
transfer-inventory owner must reconcile those source sites. E01/LSP ownership
must address the persistent-session guard. Do not mark either global guard
qualified from an item-scoped pass. The
Emitted-Rust qualifier did not edit these files and must resume its full gate
only after owner-scoped repairs and focused checks merge. A separate SQL
public-`bigint` guard failure is recorded in the
[SQL issue](ad-hoc-schema-first-sql-platform-review-follow-ups.md#public-bigint-compatibility-guard-integration-blocker-2026-09-27).

The exact failed gate log, lane and developer-tooling JSON are preserved under
`/data/sifr-emitted-final-qualifier-retry-evidence-20260924/final-a04cd9c4/attempt12/`
with SHA-256 `c22ee68d59c8cc0109abc261ef0f519dc76721732d88dd4b88c2093439970be1`,
`35091b86b7e0e934be64ac05d97fe8b0f7fb00df160c2b50851d2988af8796f5`,
and `875055dbd01b6be8c2659495e133698f28aa2781dba0536eca33ddb598ef2733`,
respectively. The approved governor window restored `schedutil`.

## V03/E01 CLI and driver transfer-inventory repair delivery (2026-09-27)

The current-main TypeScript-Go M1 CLI/driver direct-read/probe inventory
follow-up merged in [PR #4041](https://github.com/sifr-lang/sifr/pull/4041)
as 23802aa1492ea68b235bb9faf556846a88a3e323 from exact tested and
reviewed candidate d36610a6694d3f23c5d71782b306801f88977755
(base 88b992ffbda63f5680fa761d6e6b0febda32d9b1). The transfer
guardrail document now classifies all 32 newly observed scanner lines in the
assigned five CLI/driver paths: 26 production observations and six inline-test
observations. Cargo vendor/generated-manifest reads remain package/build
identity inputs; formatter markers and Windows/project cache probes remain
cache trust or output-storage observations; the test-runner probes are
generated-output assertions. Formatter source still passes through
SourceProvider. This inventory change does not claim source-provider migration
or classify the separate V03 static/direct-filesystem-effects guard.

The exact candidate passed the 32/32 path-and-line inventory comparison,
TypeScript-Go transfer guard self-test, documentation structure, 900-line
file-size guardrail for 4,245 files, and committed diff check. The full
transfer guard now fails only on the separately owned E01 persistent LSP
Session condition, with no direct-read/probe inventory omissions; it is not
recorded as a global pass. Scoped Claude Opus 5.5 review returned SATISFIED
with no blocking findings (response SHA-256
8ad0bc9b2c11dce937c11c084aeae2e4812558dfa04925ede77536a7e5c54508).
Candidate-keyed raw logs and the review response are outside the Git tree at
/data/sifr-typescript-go-direct-probe-inventory-evidence-20260927/d36610a6694d3f23c5d71782b306801f88977755/;
validation manifest SHA-256
4360f49a5104710f8be1b905150111f5c6899c1d9820b960382190db6097e9b5.
The initial documentation check failed because the pinned editor submodule
was absent; initialization followed by a passing check is recorded separately.
The automatic PR create-PR CI job failed and is not used as validation.
The phase-end full merge profile remains deferred under the approved
intermediate-item policy. E01 still owns persistent Session repair and V03
still owns the independent static filesystem-effect inventory.

The read-only review suggested clearer wording for two accurate classifications
and noted a pre-existing missing header on the following older table. These are
nonblocking documentation follow-ups for the transfer guardrail owner, not
additional acceptance for this batch.

## V03/F24 current-main filesystem-effect inventory repair delivery (2026-09-27)

The V03/F24 inventory drift in the developer-tooling blocker above merged in
[PR #4043](https://github.com/sifr-lang/sifr/pull/4043) as
`bd4ceb50931d646ffd47761b469f214903a461be` from exact tested and reviewed
candidate `a0baf01ed904dc6449426218d3dfba187864646a` (base
`e3f993019a57d6d2214567b9f09f4bff541b640c`, tree
`985026bce2fd1fdac797da37bc548bd1b67bf891`). The repair classifies
current site/symbol effects, preserves unchanged and moved classifications,
and removes stale source observations. Cache trust and provenance reads are
build identity, test-only reads are tooling input, metadata/source and
selected toolchain reads retain semantic-input classifications, and
filesystem mutations are output effects. Only the checked-in inventory
changed; no Rust implementation or other owner's guard changed. The blocker
counted 165 distinct unclassified and 72 distinct stale keys; the candidate
diff adds 168 and removes 74 rows because repeated keys have multiplicity.

On Linux x86_64, the exact full direct-filesystem-effects check passed for
3,013 sites, its negative self-test passed, the 900-line file-size guardrail
passed for 4,245 files, documentation structure passed after the pinned
editor submodule was initialized, and `git diff --check` passed. The initial
documentation setup failure is preserved separately. Scoped read-only Claude
Opus 5.5 review returned **SATISFIED** with no blocking findings (response
SHA-256 `5c0c23fe4e4d4169196c2bc86696896e54c4199773829877dce4aabecc556ecf`).
Candidate-keyed raw logs and the review response are outside the Git tree at
`/data/sifr-architecture-v03-f24-fs-inventory-evidence-20260927/`; validation
manifest SHA-256 is
`611e53db8e97406bf22c09bc02cc686e250288160b6c18e050fb7570345db03e`.
No Cargo target or shared worktree was cleaned. The approved intermediate-item
policy defers the full merge profile to Q01; this inventory pass does not
qualify E01's persistent LSP Session condition or the final integration gate.

## E01/F20-F21 persistent LSP Session guard prerequisite delivery (2026-09-27)

The current-main TypeScript-Go transfer guard's persistent Session condition
merged in [PR #4045](https://github.com/sifr-lang/sifr/pull/4045) as
`9a41c2402d54ddf24f379845562eefdfabdd46b7` from exact tested and
reviewed candidate `a05ab04899bfa902c954e7fb61ec15fab5494191` (base
`8391ec8ee2634ebbbc015432fbf24a762f32e26c`, tree
`a710b912f19ae046ca654977fca0bc06167e2888`). `Session` already
owned a persistent `LspAnalysisWorkspace` constructed with the explicit
compiler context; the guard incorrectly required its obsolete `default()`
constructor. The repaired assertion checks the owned field, compiler-bound
construction inside `with_compiler`, and document-query routing through
`self.analysis`. Negative self-tests reject a missing field, default
construction, and missing query routing. No LSP runtime code changed.

On the exact candidate, the selected
`developer_tooling/typescript-go-transfer` suite passed both the guard and
self-test (2/2). Three exact `sifr_lsp` regressions for unsaved overlay
analysis, changed overlay analysis, and project save ownership passed 1/1
each. The 900-line file-size guardrail passed for 4,245 files and the
committed diff check passed. Scoped read-only Claude Opus 5.5 review returned
**SATISFIED** with no blocking findings (response SHA-256
`355ecdbe07174ea477a3840e58c39e30e4bbf5e960e9f244f3b8b561c8a50d1f`).
Raw candidate-keyed evidence is outside the Git tree at
`/data/sifr-e01-persistent-session-evidence/a05ab04899bfa902c954e7fb61ec15fab5494191/`;
the validation manifest SHA-256 is
`d0e2203dd4c490a3872e8bc722139d631feb965ac834882d129fad43fe5ae415`.
The original single-condition guard failure is retained under the evidence
root; the initial test setup failures are described in the manifest. The
isolated Cargo target was kept warm; no shared target was cleaned.
The record-only documentation structure check passed after initializing
the pinned nested editor submodule; its first missing-submodule setup
attempt failed.

This closes only the persistent Session guard prerequisite from the
developer-tooling blocker. E01a-E01c below own the remaining external input,
watcher and fast-hit work. The item-scoped pass does not qualify the full
developer-tooling area or the phase-end merge profile; Q01 retains final
integration qualification.

## E01/F20-F21 needs-new-scope split (2026-09-28)

The broad E01 implementation assignment stopped at **needs-new-scope** on
clean main `ea82b1e4558b95addc75277f131a281575bc058f`. It produced no
implementation, test pass, review approval or E01 closure. Inspection found
that `Session::record_watcher_events` reduces notifications to a count,
`LspAnalysisWorkspace` applies that count across all projects, and the server
discards `Message::Response`. Existing authoring requests fingerprint some
package inputs, but the watcher transport has no registration acknowledgement
or root-specific change authority. A fast-hit reorder cannot safely precede
that authority. The merged persistent Session guard above remains a completed
prerequisite, not acceptance for the three items below.

Each row has one implementation owner and one merge/receipt. Start the next row
only after its predecessor merges. Run every listed path with
`cargo test --locked -p sifr_lsp --lib <path> -- --exact`; create the new named
cases where needed, then run focused regressions for touched APIs. Keep the
candidate, validation inputs, raw results and scoped review keyed by SHA.
The approved intermediate-item policy applies; Q01 owns the final full gate.

| Item and dependency | Bounded ownership and acceptance | Named focused positive and negative cases |
| --- | --- | --- |
| **E01a** after the persistent Session prerequisite and C02e | LSP/package external-input owner. Give each package root a snapshot and generation covering `sifr.toml`, `Cargo.toml`, `Cargo.lock`, relevant configuration, Python bridge sources/inventory, binding and certification artifacts, and selected Python environment and its declared files. Changed, newly present, absent, deleted, renamed, empty and unreadable inputs must change or invalidate the correct authority; unchanged inputs retain warm reuse. Invalidate analysis and Python declaration cache successes and failures for the affected root only. Do not add watcher protocol registration or reorder request fast hits. | `session::tests::e01_external_input_tests::unchanged_inputs_keep_the_owning_root_warm`; `session::tests::e01_external_input_tests::changed_and_renamed_inputs_advance_only_the_owning_root`; `session::tests::e01_external_input_tests::absent_empty_deleted_and_unreadable_inputs_invalidate_negative_and_positive_cache_entries`; `session::tests::e01_external_input_tests::environment_bridge_and_certification_drift_revalidate_without_source_edits`. Preserve exact existing `session::tests::python_declaration_tests::watcher_drift_revalidates_authoring_artifacts` and `session::tests::python_declaration_tests::live_bridge_sources_without_inventory_are_validated_and_fingerprinted`. |
| **E01b** after merged E01a | LSP server/protocol owner. Register watched files only when client capability permits; track request ID and acknowledgement or rejection, and route create/change/delete events and rename delete/create pairs to the owning root. Until acknowledgement, after rejection, and for unsupported clients, revalidate external inputs per request using E01a generations. Re-establish authority on reconnect and workspace-folder changes; coalesce storms without losing a change or invalidating unrelated roots. Do not change Python fast-hit ordering. | `server::tests::watcher_registration_acknowledgement_enables_root_scoped_events`; `server::tests::unconfirmed_rejected_and_unsupported_watchers_revalidate_per_request`; `server::tests::watcher_reconnect_and_workspace_folder_change_restore_authority`; `session::tests::e01_watcher_tests::create_delete_rename_and_storm_events_preserve_multi_root_isolation`. Preserve exact existing `session::tests::python_declaration_tests::stale_existing_lockfile_remains_a_package_error`. |
| **E01c** after merged E01b | LSP request/diagnostics owner. Reorder verified Python fast-hit checks only after source, configuration, external generation, package/root identity and cancellation are current. Prove warm hit reuse and full completion, hover and diagnostics publication after each relevant change; stale work must not publish over newer edits, close/reopen, root changes or cancelled requests. Include end-to-end multi-root acceptance. Do not take E02 canonical HIR/lint reuse or E03 measured 25-module cache deltas. | `session::tests::e01_fast_hit_tests::verified_unchanged_request_reuses_python_status`; `session::tests::e01_fast_hit_tests::source_config_and_external_changes_recompute_requests_and_publish_current_diagnostics`; `session::tests::e01_fast_hit_tests::cancelled_and_reopened_requests_cannot_publish_stale_results`; `session::tests::e01_fast_hit_tests::multi_root_requests_keep_generation_and_publication_isolated`. Preserve exact existing `session::tests::python_declaration_tests::python_declaration_diagnostics_and_source_drift_invalidate_cached_status`, `session::tests::python_declaration_tests::cancelled_python_declaration_request_stops_before_probe`, and `session::tests::dx11_editor_tests::dx11_stale_close_clear_cannot_overwrite_reopened_document`. |

E01/F20-F21 completion requires all three merged receipts and the named lifecycle
assertions on their respective exact candidates. E02 remains the separate
analysis/lint owner after C01; E03 remains the performance owner after E01c
and E02. Neither has a completion claim here.

The record-only scope split merged in [PR #4073](https://github.com/sifr-lang/sifr/pull/4073)
as `b8407d71bb1cf07155cc17ad392d434ebda6bfaa` from exact candidate
`57da2676e426090668708c434e307ee11a99d723` (base
`ea82b1e4558b95addc75277f131a281575bc058f`). Documentation structure
passed 1/1 (raw log SHA-256
`4601ca3d4aeb803a83ef2f7d831ac86a47372bea2a162ba46f1b1bf953a302c3`),
the 900-line file-size guardrail passed for 4,256 files (raw log SHA-256
`9dd0c274eab36227042a9f9ea2391ea0e47018a05c1f17e935d9e8d28c18f7d0`),
and the committed diff check passed. Candidate-keyed raw logs are outside the
Git tree at `/data/sifr-architecture-e01-scope-split-evidence-20260928/`.
The first documentation attempt failed during worktree setup because the pinned
editor submodule was absent and the runner requires an in-tree result path;
after initialization the supported command passed. This documentation-only
split required no broad gate or new external review. E01a is the next item;
no E01a, E01b, E01c, E02, E03 or Q01 acceptance is claimed.

## E01a/F20-F21 per-root external-input delivery receipt (2026-09-28)

[PR #4075](https://github.com/sifr-lang/sifr/pull/4075) merged as
`8dcb758fd11109fe205a4645713a2b232d59b8eb` from exact tested and
reviewed candidate `58e15d59fa33087ae60135e12b5b2e6c36a45641` (base
`20457942c26d97c6a5ab6e0293b2acc9b627abd1`, tree
`df435e31a54ce7dba0ccf58ecb2c234e7dbdb0f8`). The LSP now tracks a
snapshot and generation per package root for manifests, Cargo configuration
and locks, Python bridge sources/inventory, binding/certification artifacts,
and selected Python environment declarations. Request-time observation
advances only a changed root and refreshes its analysis owner and Python
declaration success/failure caches; stable inputs retain warm reuse. Newly
present and deleted nested manifests move document ownership between roots.
This item leaves watcher registration/acknowledgement to E01b and Python
fast-hit ordering to E01c.

On that candidate, all four new E01a cases, both preserved existing cases and
six focused ownership, source, lockfile and certification regressions passed
with one selected assertion each. Formatting, the 900-line file-size guard
(4,259 files), documentation structure (1/1) and diff checks passed. The
candidate-keyed raw logs and input identity are under
`/data/sifr-architecture-e01a-external-generations-evidence-20260928/58e15d59fa33087ae60135e12b5b2e6c36a45641/`;
validation manifest SHA-256 is
`9b8060fd4478ecb6af04291eb6f0cd35110e33300d4d49ccfbd54f380878a279`,
and the raw-log digest index SHA-256 is
`2aacec2b17c7a690f21c2b0f3f6217d01a536c4e69aa9171a6ec7d0f32611c79`.
The initial cold build and a subsequent selection ran zero new assertions
before the test module was registered; a later negative/positive analysis
assertion failed because its new Cargo fixture lacked `src/lib.rs`. These
attempts remain preserved as incomplete/failing evidence, followed by the
fixed fixture and twelve exact passing selections on the final candidate.
The isolated target and prepared Python environment stayed warm, with no
cache cleanup.

Scoped Opus review of that exact candidate returned **SATISFIED** with no
blocking findings (response SHA-256
`3010e913c1002b9d8e6f6ece2c7b4907272b521ff41428de5c538649c8cae0bc`).
Review suggestions concerning interpreter hashing cost and root retirement
belong to later E01b/E03 work; live distribution changes without declared-file
changes remain an existing environment-authority follow-up and are not
qualified by E01a. An optional strict scoped Clippy run stopped in
pre-existing `sifr_codegen/src/checked_place.rs:111,116` warnings, recorded
in the Emitted-Rust issue;  it was not an E01a acceptance gate (raw
log SHA-256 `08ddb955f2ed756160898d2991f362e16e9df6d114af5d8d20e4a39014d7487e`).
The prospective intermediate-item policy defers the full merge profile to
Q01. E01b is next; this receipt does not claim E01b, E01c, E02, E03 or Q01.

## E01b/F20-F21 watcher authority delivery receipt (2026-09-28)

[PR #4077](https://github.com/sifr-lang/sifr/pull/4077) merged as
`d3c1163a871a4e5de8ef54fcd78a57e754e867ab` from exact tested and
reviewed candidate `10d0aec0628de66d5d2eec17ed449a58c4523501` (base
`663946628335d49d70b7e6de237bcc29564e4ddd`, tree
`2a629723dda529a31049f48e44fcc1bcb30945fa`). The LSP now registers
watched files only when dynamic registration is supported, matches the client
response to its request ID, and treats acknowledgement as watcher authority
for current workspace folders. Pending, rejected, unsupported, malformed and
uncovered-root cases retain per-request E01a external-input revalidation.
Create/change/delete notifications and rename delete/create pairs route to
the owning package roots; storms coalesce per root without invalidating an
unrelated root. Folder changes and reconnects renew registration, including
unregistration of an old pending or acknowledged registration. Python fast-hit
ordering remains E01c work.

On that candidate, all four new E01b named cases, the preserved stale-lockfile
case and six focused watcher, Python, external-input and ownership regressions
passed with one selected assertion each. Production `cargo check --locked -p
sifr_lsp --lib`, formatting, the 900-line file-size guard (4,263 files),
documentation structure and diff checks passed. The candidate-keyed raw logs,
configuration identity and bundle are under
`/data/sifr-architecture-e01b-watcher-authority-evidence-20260928/10d0aec0628de66d5d2eec17ed449a58c4523501/`;
the raw-log digest index SHA-256 is
`dfabb1c9a263997c551c5e2ef1d47df9d099f5ed2621e56408c5efe5852e3ff2`.
The initial exact selection lacked the pinned Ruff submodule, a subsequent
compile found a test-module visibility error, and a focused watcher regression
failed before this worktree's Python verification environment was prepared.
Those raw failures remain preserved outside the final candidate directory.
No cache was cleaned; the prior inactive Cargo target was copied into this
worktree's own target before validation.

The first scoped Opus review returned **NOT SATISFIED** because the named
rename test could pass without its rename pair taking effect (response SHA-256
`c7d82aa87df65644e6c5e7bf44826094488032b99117bfcfc876ecf82e8bbf61`).
The repair captured the generation immediately before the pair and asserted
both roots advance on a cross-root move; it also retired registrations still
pending on folder changes. All eleven exact selections and affected checks
were rerun on the final candidate. The second scoped Opus review returned
**SATISFIED** with no blocking findings (response SHA-256
`ebbee34a76205b885a94e249a2ce33ca283c557d5428bd8c79024f59b432c6a4`).
Its follow-ups concern one observation on acknowledgement before E01c fast-hit
reordering, narrower watcher work and snapshot deduplication for E03, and
possible re-registration after malformed notifications. They do not qualify
E01c/E03. The prospective intermediate-item policy defers the full merge
profile to Q01. E01c is next; E01/F20-F21, E02, E03 and Q01 remain open.

## E01c/F20-F21 fast-hit delivery needs new scope (2026-09-28)

The E01c request/diagnostics implementation stopped at **needs-new-scope**.
[Draft PR #4079](https://github.com/sifr-lang/sifr/pull/4079) remains unmerged
at tested candidate `9ad0f8a34922046dc4fbdacb23f3d05170e3ecb3`
(base `34e78b959b069ddc0dc5f4ddf3fb0d8ddd67d30e`, tree
`9eeeb52654aa74044459ef3258637878d5ed4a8a`). The candidate moved
Python declaration cache hits ahead of interop-plan and workspace-diagnostic
queries, scoped source invalidation to a package root, and attempted current
request and diagnostics publication after external-input changes. This is
historical implementation evidence, not E01c acceptance or merge approval.

On that exact candidate, the four E01c named cases, all three preserved cases,
and ten focused regressions passed individually with exact
`cargo test --locked -p sifr_lsp --lib <case> -- --exact` selection.
`cargo fmt --check`, the 900-line guardrail (4,265 files), and the full PR
diff check passed. Candidate-keyed raw logs, compiler/lock/submodule/fixture
identity and the transport bundle are outside the Git tree under
`/data/sifr-architecture-e01c-fast-hit-publication-evidence-20260928/9ad0f8a34922046dc4fbdacb23f3d05170e3ecb3/`;
the validation-manifest SHA-256 is
`6ede8c0f104f5286ec4ea0a1f4cd9b1d1ab1dce973d2b0213b55be9c99c4f4c2`.
The initial missing-Python-venv assertion failures, corrected after copying
the prepared fixture into this worktree, and the first repair-loop failures
remain under the evidence root's `development/` sibling. No full merge
profile was run under the approved intermediate-item policy.

The first scoped read-only Opus 5.5 review was **NOT SATISFIED** (response
SHA-256 `d5c7129723d79b885125b8ca27c29299f908b08c2fd8e2a5d6e771b42f3453aa`):
request-time diagnostics errors could end the server and drop jobs, and the
tests missed package-root changes and cross-root external drift. Those findings
were repaired and retested. The second scoped review was also
**NOT SATISFIED** (response SHA-256
`5c9fed8adf275077e6edb64d94c407cf6d6a6cc134cad2d9be108125293272e1`)
with a new mechanism-level defect: after a nested package's `sifr.toml` is
removed, the fast-hit expected generation can come from the deepest
ever-tracked nested root while verification observes the current ancestor
root. Their independent generation counters can remain different, causing
every later completion, hover and diagnostics request to return
`ContentModified`; current diagnostics never publish. The named root-change
test removed the only manifest, so it did not exercise nested-to-ancestor
ownership. Under the phase-closure-loop second-review rule, this item stops
for rescope rather than another patch in this session.

The next E01c owner must establish one current package-root identity and
external-generation source for both fast-hit capture and verification,
including nested-to-ancestor transitions, then add an exact regression that
warms a nested package, deletes its manifest, rejects stale work once,
recomputes under the ancestor and publishes current diagnostics. Keep
cross-root isolation and request-time publication assertions. Review and
validate the new candidate independently; the draft commits may be selectively
ported but are not approved for merge. E01/F20-F21, E02, E03 and Q01 remain
open.

## E01c/F20-F21 verified fast-hit delivery receipt (2026-09-28)

[PR #4081](https://github.com/sifr-lang/sifr/pull/4081) merged as
`5130e8294f000faa5d69c3ac1da95a53b8668e9d` from independently tested and
reviewed candidate `15fe3c223f3f99b8f989b5ea94d5825394a74073` (base
`15e0452920e49bf0e8f995278e071914c8735d8d`, tree
`7641534485f35360c22aec4def3edcb323181ef9`). This branch selectively
ported draft #4079 code, then established one current package-root and external
input identity for Python fast-hit capture and verification. Removing a nested
`sifr.toml` now retires that root, reassigns open-source analysis to the
ancestor and rejects stale work once before completion, hover and diagnostics
resume. Unstable external inputs retain per-observation invalidation without a
permanent `ContentModified` loop: an unchanged observed failure state
completes, while changed input is rejected and recomputed. Request-time
diagnostics and watcher paths publish only current results. The unmerged draft
and its failed reviews above remain historical evidence, not approval for this
delivery.

On Linux x86_64 with rustc/Cargo 1.98.1, 24 exact
`cargo test --locked -p sifr_lsp --lib <case> -- --exact` selections passed on
the final candidate: all four E01c named cases, all three preserved cases, the
nested-to-ancestor and unstable-input regressions, and focused request,
publication, external-input, watcher, cancellation, close/reopen and multi-root
cases. Each selected one assertion. `cargo fmt --check`, the 900-line
file-size guardrail (4,265 files) and full PR diff check passed. The tracked
`Cargo.lock` and Ruff gitlink were unchanged; the prepared Python fixture and
separate worktree-owned warm Cargo target were retained. Candidate-keyed raw
logs, input identities and the exact-commit bundle are outside the Git tree
under
`/data/sifr-architecture-e01c-root-transition-evidence-20260928/15fe3c223f3f99b8f989b5ea94d5825394a74073/`;
the validation-manifest SHA-256 is
`d510a99f2e89d2b69066afacacd9effbc611025a38c850ab54a46342425b8032`.
Earlier fixture and repair failures remain in the `development/` sibling and
are not counted as passes. The approved intermediate-item policy leaves the
full merge profile to Q01.

The first read-only scoped Opus 5.5 review on candidate
`1c843f35991453eadf7f4c72a22a58d47444cd66` was **NOT SATISFIED**
(response SHA-256
`5af228d98039d4f5223178200646d9f75ae37a2414b0f5fd7dde0234f939114a`):
unstable inputs advanced their generation again at verification, permanently
rejecting requests. That finding was repaired and the affected selection plus
the full exact E01c validation reran. The second read-only scoped review on the
final candidate was **SATISFIED** with no blockers (response SHA-256
`33d7e9eb511ed8b55c2556a6fb4623c7cace9601d89c9aa4981ee7809f6aaf3c`).
Deferred follow-ups remain with their owners: E01a's transient unreadable-input
race, E01b's immediate watcher-side root reassignment, a later LSP diagnostics
scheduler item for idle pending jobs, and E03's unstable-root refresh cost and
measured editor performance. These are not E01c merge blockers. With E01a
[#4075](https://github.com/sifr-lang/sifr/pull/4075), E01b
[#4077](https://github.com/sifr-lang/sifr/pull/4077) and E01c #4081 merged
with their named assertions, E01/F20-F21 is complete. E02, E03 and Q01 remain
separately owned.

## E02/F22 canonical lint and current-action delivery receipt (2026-09-28)

[PR #4083](https://github.com/sifr-lang/sifr/pull/4083) merged as
339bb5096da4c2905b7fc07258ff877033dd1e8f from exact tested and
reviewed candidate d7ecb71f109153af26216f9d31f94115eab0b25c (base
f58a63f0c2196c06c347b6891e8810c1bc1ebf25, tree
016d8b2a2864c9233f7c0d23a197506de5242c46). On a lint miss,
analysis now passes its current frontend context's parsed syntax and HIR views
to the lint engine; a hit returns its existing per-file result. Current policy
code actions consume that result, reject a diagnostic moved off the requested
line, and insert suppression on the current diagnostic line. Fix-all applies
current diagnostics and rechecks changed text as a new lint source snapshot.
Standalone policy selection, per-file ignores, severity, suppression filtering
and sorting remain shared. Code actions moved to a separate analysis module to
keep hand-maintained files below 900 lines. The architecture note records this
authority boundary.

On Linux x86_64 with rustc/Cargo 1.98.1, all **13 exact one-assertion**
selections passed: four new sifr_lint cases, three new sifr_analysis
cases, five preserved analysis lint/action regressions, and one sifr_lsp
SQL editor action regression. Formatting, the 900-line file-size guardrail
(4,266 files), HIR maintainability, documentation structure, analysis
split-brain guard and committed diff checks passed. The tracked Cargo.lock
and Ruff gitlink were unchanged. The owned warm target reached 17 GiB with
71 GiB free on /data; no target was cleaned. The first documentation
attempt failed only because the new worktree lacked the pinned nested VS Code
submodule; after initialization the check passed. Candidate-keyed commands,
inputs and raw logs are outside the Git tree at
/data/sifr-architecture-e02-evidence-20260928/d7ecb71f109153af26216f9d31f94115eab0b25c/;
the validation manifest SHA-256 is
b8449adb5f4224b3bc48e24f7694d55ef535e2c136c83a1848cb024c7b0fdf41.

Scoped read-only Claude Opus 5.5 review returned **SATISFIED** with no
blocking findings (response SHA-256
1dd7235cac1c6469c753cb9afad48ae70491cfd67e739f6dc8ac85982f537620).
Its nonblocking follow-ups are a more explicit parse-failure view sentinel for
the lint API, the pre-existing end-of-line suppression edit behavior for the
lint/action owner, and a shared line index for large-file action lookup under
E03 performance ownership. Supplemental strict Clippy stopped in already
recorded sifr_codegen/src/checked_place.rs:111,116
items_after_statements warnings; allowing those exposed an older analysis
preview uninlined_format_args warning at implementation.rs:494 (original
preview code predates E02). A scoped Clippy run with only those two known
categories allowed passed. Neither failed attempt is E02 acceptance evidence
or a reason to absorb the other owners' code.

E02/F22 is complete on current main. E03 remains the measured-performance
owner after E01c and E02. Q01 retains the final exact-candidate full merge
profile and whole-phase integration decision.

## E03/F23 measured 25-module LSP cache delivery receipt (2026-09-28)

[PR #4085](https://github.com/sifr-lang/sifr/pull/4085) merged as
`4b319c0f784c4c5e71d93621de8331d41f98e3fc` from exact tested and
reviewed candidate `28381613b509ae45456895118991e1b3abc686ab` (base
`7fbf85e96895c99d5a6b4060417d93a0322f498b`, tree
`96da54cdc21f713f1196626161f77cf999945509`). The LSP exposes cumulative
Python declaration snapshot hits and completed misses through its debug
protocol; the LSP benchmark reads actual server deltas instead of fixed
values. The workspace fixture has 25 connected source modules, and workspace
benchmark scenarios enforce that minimum independently of its directory name.
The new live cache-delta case is selected by both create-PR and merge profiles.
The historical SHA-pinned DX.1 budget file is unchanged.

On that candidate, the live `performance/lsp-workspace-cache/lsp-cache-deltas`
case passed **1/1**. Its real 25-module server reported cold workspace query
**0 hits/1 miss**, unchanged completion and hover **2/0**, private-body edit
**0/1**, public API edit **0/1**, external manifest change **0/1**, cancellation
**0/0**, and the next current query **0/1**. The case uses diagnostics-off to
isolate request cache work; it does not claim a controlled-host latency budget.
The existing initialization-only cold-start benchmark observed **0/0** during
that interval, while the separately measured cold workspace query made the
miss. Focused completion, hover, cold-start, workspace-diagnostics, references
and rename benchmark invocations passed and emitted actual counters. An
undersized workspace was rejected and the 25-module input accepted.

All **seven exact one-assertion** `sifr_lsp` selections passed: four E01c
fast-hit lifecycle cases, cancelled Python probing, Python declaration source
drift, and unchanged external-input warm reuse. The compiler build,
performance manifest validation, benchmark runner and budget self-tests,
create-PR and merge profile plans, coverage readiness **4/4**, formatting,
HIR maintainability, the 900-line file-size guardrail (4,267 files), and the
committed diff check passed. The Ruff gitlink and tracked Cargo.lock did not
change. The owned target was 19 GiB with 46 GiB free on `/data`; no cache was
cleaned. The first Rust selection failed only before this worktree's Python
fixture environment was prepared, and an earlier exploratory edit to the
immutable historical budget failed its DX.1 self-test and was reverted before
the candidate. Those attempts are not counted as passing evidence.
Candidate-keyed raw logs, commands and input identity are outside the Git
tree at `/data/sifr-architecture-e03-evidence-20260928/28381613b509ae45456895118991e1b3abc686ab/`;
validation-manifest SHA-256 is
`54185808fab98122af0f7ad834459b61f0dc0a46374c105ad829853bd1fb886b`.
The prospective intermediate-item policy leaves the full merge profile to
Q01.

The first scoped read-only Opus 5.5 review returned **SATISFIED** with no
blockers (response SHA-256
`be6a1b1af86c2e9ea6dc84219c7625a2e628074dd1c91c2371e70427a9149944`).
The test was then hardened to cancel a cold query after project retirement,
and the module minimum was bound to workspace scenarios. The second scoped
review of the final candidate returned **SATISFIED** with no blockers
(response SHA-256
`52368d7921850c113c6f07d5baa6737f492acf1fe7af5a95390c9699e6f2b517`).
Its nonblocking follow-ups are a remaining theoretical cancellation timing
race on unusually fast hosts, separate frontend query-cache counters if future
performance work needs them, and Q01's reference/trend comparability decision
for the expanded workspace fixture. These do not qualify Q01. E03/F23 is
complete on current main; Q01 retains the final integration gate and
whole-phase review.

## E04/F34 resolved editor type-hierarchy delivery receipt (2026-09-28)

[PR #4087](https://github.com/sifr-lang/sifr/pull/4087) merged as
`b12ced85ebca4b6e1ec2c1ecff82f0bbd86351e4` from exact tested and
reviewed candidate `c57bb1320fdbcc3d6e0cabf6800ed38c5c363be6` (base
`29148ae25b56fda239078a5143694a00d7bf25c1`, tree
`d1ae5d17afe8fd8e4718cff88eda5131199f6d94`). Frontend module analysis
now exposes class identities and direct parent identities from resolved HIR.
The analysis host uses those identities for current class preparation and
nonempty direct supertype/subtype answers, with parsed syntax supplying only
source ranges. The LSP maps returned relation items to the owning current
file URI and source instead of discarding them. A standalone class without a
resolved relationship returns no prepared hierarchy item.

On the exact candidate, two `sifr_analysis` hierarchy cases passed **1/1**
each for lowercase classes, uppercase non-types, imported aliases, source
ranges and removal of stale edges after edits. The native `sifr_lsp` relation
and edit case passed **1/1**, and the existing frontend analysis-cache reuse
case passed **1/1**. The executable stdio LSP protocol smoke passed with a
nonempty parent/child fixture and edit invalidation. `cargo build --locked -p
sifr`, documentation structure, HIR maintainability, formatting, the
900-line file-size guardrail (4,268 files), and the committed diff check
passed. Exact command/log digests, compiler and input identities and the
transport bundle are outside the Git tree at
`/data/sifr-architecture-e04-f34-evidence-20260928/c57bb1320fdbcc3d6e0cabf6800ed38c5c363be6/`;
validation-manifest SHA-256 is
`14c9d8aa9540c56b09f5a86d3ab9ddc65532f449d71a80bc241dee3e0972191f`.
The owned Cargo target stayed warm; no cache was cleaned. The first
documentation check lacked the pinned nested VS Code submodule, an earlier
CLI build found a production/dev-dependency import mistake, and the first
protocol smoke consumed an unrelated queued diagnostic. The final candidate
fixed setup, import and fixture isolation; failed attempts remain distinct.
The automatic GitHub create-PR job failed and is not used as validation under
the approved intermediate-item policy; Q01 retains the full merge gate.

Scoped read-only Claude Opus 5.5 review returned **SATISFIED** with no
blocking findings (response SHA-256
`06391716af596927c9eb520b18f71f9308c647b1f1cbfb813b4a6e59ecced240`).
Nonblocking follow-ups are the UX choice to omit standalone types with no
edges, possible performance work under E03's latency budget, attribute-base
reference preparation, and a cross-file LSP conversion regression case.
These do not change E04 acceptance. E04/F34 is complete on current main;
Q01 retains final exact-candidate integration and whole-phase closure.

## C01 diagnostic docs sync integration blocker (2026-09-26)

The Emitted-Rust final qualifier's full merge profile on exact merged main
`bbdfd6cb5f2b61f35ecb6d12d5885dc8707bd96b` passed the repaired
Python-interop selection with complete combined certification, **30/30**
variants. Its first later failure was `area_diagnostics` ->
`rules/docs_sync`: `internal_docs/diagnostic_codes.md` and
`docs/errors/SIFR-IMPORT-0007.mdx` are out of sync with the diagnostic
registry. Four later diagnostics rules were blocked by fail-fast; later
areas and crate/E2E selections were not reached. The generated documents
still name `sifr_driver::project::compile_order`, whereas C01 commit
`c24f37c00302273d2388b4f9469b4da4e105cf25` changed the
`SIFR-IMPORT-0007` registry owner to
`sifr_frontend::compile_order`. This is C01-owned generated documentation
drift, not an Emitted-Rust codegen failure. The C01 owner should regenerate
the checked-in diagnostic documents, prove the exact
`python3 verification/areas/diagnostics/checks/docs_sync.py` selection
and relevant documentation checks, and merge the narrow repair. The
Emitted-Rust qualifier must then rerun its full merge profile on the new
merged candidate.

The gate log and copied lane/diagnostics JSON are under
`/data/sifr-emitted-final-qualifier-retry-evidence-20260924/final-bbdfd6cb/attempt10/`;
their SHA-256 values are
`ba2a7113289158022aaa1e72331176aacfe95d0661605fc9b72780ac624454f7`,
`3da6e728c3621a8ac21ae404b2d22e63466407827170ed44c5328c705a2dbab5`,
and `c0db2029e8cce232f70c30a2370a645cfae57600c02bb5df26cb9d0b4ee28b25`,
respectively. Gate exit was 1; the approved governor window restored
`schedutil`. No C01 code or generated docs were changed by this qualifier.

## C01 diagnostic docs sync repair delivery (2026-09-26)

The narrow generated-doc repair merged in [PR #4036](https://github.com/sifr-lang/sifr/pull/4036)
as `b2127c5f1143e6c7636bfbe897ce9d8ea1bc9cd6` from exact tested and
reviewed candidate `24c5248529756a0f717eb58bb7d266a5e8cbebb2` (base
`ac326a096250418c20e2cf511314e6c96f3eb7d8`, tree
`2a6c9dbd40e81e708acb78fab5104b5ba16b550e`). The merged tree matches
the candidate. Regeneration changed only the owner field for
`SIFR-IMPORT-0007` in `internal_docs/diagnostic_codes.md` and
`docs/errors/SIFR-IMPORT-0007.mdx` to `sifr_frontend::compile_order`.

On Linux x86_64, the exact
`python3 verification/areas/diagnostics/checks/docs_sync.py` selection
passed; its raw log SHA-256 is
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
Documentation structure passed (raw log SHA-256
`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`),
the 900-line file-size guardrail passed for 4,245 files (raw log SHA-256
`2e10821206a8db09bbb2a00e21e9618f4f9f216f73b5c56afcd7ce13f9e819d2`),
and diff check passed. The initial generator attempt failed because the new
worktree had no initialized Ruff submodule; after initializing the pinned
Ruff and nested editor submodules, generation and checks passed. Raw logs
and the read-only response are under
`/data/sifr-architecture-c01-diagnostic-docs-evidence-20260926/`, keyed by
candidate SHA. The scoped [Opus review](https://github.com/sifr-lang/sifr/pull/4036#issuecomment-5848656738)
returned **SATISFIED** with no blocking findings (response SHA-256
`4e7d667feb4d8f15af01ceb5a6bc2d0ed8eae2d4373a802ec49239e281429e5b`).

The generated-doc sync blocker is repaired. The Emitted-Rust qualifier must
rerun its full merge profile on a final merged candidate; this focused pass
does not qualify the later diagnostics rules, areas, crate suites, or E2E
selections blocked in the previous gate. A separate hand-maintained
`internal_docs/diagnostic_emission_inventory.md:91` reference still names
the former driver path for `SIFR-IMPORT-0007`; D01b documentation-map work
owns that current-path follow-up. It did not affect the generated-doc check.

## V02/V04 Python interop merge-gate safety-envelope blocker (2026-09-26)

The Emitted-Rust final qualifier reran the canonical merge profile on
`dec8ec7a2f5346bb839fa7489280bf64021e2804` after the V04/F27
self-test repair. The repaired guard passed. The first functional failure
was `area_python_interop`, whose one outer `run_command` process reached
the inherited 2,400-second safety deadline after 2,400,558 ms (exit 124,
`safety_deadline`). The selected area manifest had passed 24 cases through
`ml/ml-examples`, totaling 2,164,187 ms of case time; its next
`libraries/library-examples` case had no result at the deadline. The
focused exact `libraries` suite passed **1/1** in 296,945 ms on the same
candidate and warm target, with zero failures and explicitly
non-promotable filtered certification. Under the observed conditions,
the full selection cannot complete within the current 40-minute process
envelope; no missing preparation or library assertion failure was found.
Later async/runtime cases, other areas, crate suites and E2E remain
unqualified.

The verification runner/profile and V04 process owners should reconcile
the canonical Python-interop selection with a measured, bounded safety
contract while preserving actual selected assertions. Do not treat an
unchanged rerun or a bare timeout increase as qualification. The gate log
is `/data/sifr-emitted-final-qualifier-retry-evidence-20260924/final-dec8ec7a/attempt9/merge-dec8ec7a2f5346bb839fa7489280bf64021e2804.log`
(SHA-256 `014cdcd67a753b888d579328ddfdcb6a082a4fc6ab32a78cd9f2564fef4349e5`);
the copied lane JSON has SHA-256
`9b3f636d9c7dedb2b9d2ae5a284a978d7d001a972daa050ca6c37a1580511097`.
Focused log and JSON SHA-256 values are
`4dbfd11109591e303ccf64b135efae9bb672376accf1063dcddf2df1fff02210`
and `321e71d23843b2858f76e0bfc747011281af9df59a99d9030f903cf418119516`,
respectively, under the same attempt evidence and owned target.
The CPU governor was independently observed at `schedutil` after the
failed gate. The Emitted-Rust qualifier changed no verification code;
its full merge gate remains blocked pending the owning repair and rerun.

## V02/V04 Python interop safety-envelope repair delivery (2026-09-26)

The bounded area-process repair merged in [PR #4033](https://github.com/sifr-lang/sifr/pull/4033) as
`3f1e50f3b21fe4a6be190233dc5bbabd958c9b67` from exact tested and
reviewed candidate `5c396a23842562a4e10ce1a72cc67ff5a5f4b0ee`
(base `4be06aa137865c98004938dad8219d7b71f6fdb2`, tree
`fa459439e87e3916b13949b2c6be635567392995`). The merged tree matches
the candidate. The canonical profile now runs each selected Python-interop
suite in its own `run_command` process under the existing 2,400-second
per-process safety deadline, in manifest order. The runner removes stale
part results, requires each selected suite's exact cases and passing variants,
then checks one combined result and the existing full-selection compiled
certification. An explicitly configured step-wide absolute deadline still
narrows all children; the cache-aware step budget remains one aggregate
performance contract. Area manifest schema/name validation also remains
active for the live-only profile, and malformed manifest input fails as a
runner-owned step error.

On Linux x86_64 with Python 3.14.7, the full verification-runner foundation
self-test passed, including the three focused segmentation cases (raw log
SHA-256 `52feb22822ab4f7a5000708c5607c391a6ef8868ecb6c88e306da609b8fc2f84`).
The real `dependency-versions` suite passed through a bounded child and the
combined-result path, **1/1** (raw log SHA-256
`40d8688ad35c8e9399ddf789f75f7f75a39b0e4bccb784f06b78be5df47e971b`).
Python compilation, diff check, and the 900-line file-size guardrail passed
(guard raw SHA-256 `2e10821206a8db09bbb2a00e21e9618f4f9f216f73b5c56afcd7ce13f9e819d2`).
The first scoped Opus review on `c848ad804917c9eff58d91720d2cd0dbc1d064eb`
returned **SATISFIED** but identified live-profile manifest validation and
malformed-input classification regressions in follow-ups; both were repaired.
The final scoped [Opus review](https://github.com/sifr-lang/sifr/pull/4033#issuecomment-5847084150)
returned **SATISFIED**, no blocking findings (raw response SHA-256
`de4ed8464dab69349db751ff085385c95a50bb8280bb684989ae8b5fc0c430d5`).
Raw logs and both read-only responses are under
`/data/sifr-architecture-v02-v04-python-interop-envelope-evidence-20260926/`,
keyed by candidate SHA. The record-only documentation structure check passed
(raw SHA-256 `d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`)
after initializing the pinned nested editor submodule; the initial
missing-submodule failures remain preserved.

The automatic [create-PR job](https://github.com/sifr-lang/sifr/actions/runs/36248605010/job/108422307234)
stopped at the already recorded V01 `performance_reference_admission`
prerequisite because its runner did not select `SIFR_PERFORMANCE_REFERENCE`;
it reached no Python-interop assertion. The 30-suite merge-profile selection
and later areas, crate suites, and E2E remain unqualified. The final
integration qualifier must run the full merge profile on its final merged
candidate; this intermediate item supplies the process-contract repair and
focused evidence only. Nonblocking review suggestions for unrelated-manifest
error attribution, a future test-import dependency, and create-PR budget
headroom remain with the verification runner owner.

## V04/F27 merged self-test integration blocker (2026-09-26)

The Emitted-Rust qualifier's exact merged `faf0b9c86d55ef1423a17b1f5691bab6ed2598d2`
candidate ran the canonical merge profile after the approved V01 reference
and idle-host CPU policy were admitted. Its first functional failure was
`guardrail_verification_runner_foundation`: the F27
`ProcessTests.test_f27_per_process_default_does_not_limit_whole_step` and
`test_f27_step_deadline_covers_successive_commands` assert that
`SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC` is absent after a simulated
step. The nested runner copies the outer gate's inherited absolute deadline
into `ProfileRunner.env`, then correctly restores that value after the
step. These tests assume a clean parent environment and fail when selected
inside a real deadline-bounded gate. The [V04/F27 owner row](#preparationf33-crosswalk-and-bounded-delivery)
and [delivery receipt](#v04f27-delivery-receipt-2026-09-23) own this
failure. The standalone 48-case pass in that receipt did not qualify the
integrated selection; the [repair receipt](#v04f27-merged-self-test-integration-repair-2026-09-26)
records its later nested acceptance.

The raw gate log is
`/data/sifr-emitted-final-qualifier-retry-evidence-20260924/final-faf0b9c8/attempt8/merge-faf0b9c86d55ef1423a17b1f5691bab6ed2598d2.log`
(SHA-256 `18568bf34de52bfc46007faa18f7fa6ccbdeaedda8da0456c825db76321f1df3`);
its preserved `merge.lane.json` has SHA-256
`83cf2d19d4d910f2598396ac4b653abe691a3c53c30155eb52bf9b0007fd45c0`.
Reference admission, cache setup, demo freshness, and the repaired native
intrinsic allowlist passed before this failure. The outer gate restored
`schedutil` on exit. The Emitted-Rust qualifier did not change V04 code,
and its full merge gate remains unqualified. The V04/F27 tests were repaired
in [PR #4030](https://github.com/sifr-lang/sifr/pull/4030); the qualifier
must rerun the full gate on its final candidate.

## V04/F27 merged self-test integration repair (2026-09-26)

[PR #4030](https://github.com/sifr-lang/sifr/pull/4030) merged as
`8fe8a5f45cb22b6bfc3de7c3a8bc635561acea9e` from exact tested and
reviewed candidate `5ba207e03ff3fee143abc4e667adb116406d0dd9` (base
`87711adf5c14756fb8a237bb74f9f67e2b4bf8c6`). The two F27 tests
now compare the post-step deadline with the value inherited before the
nested `ProfileRunner` was constructed. A clean parent still expects no
value; a deadline-bounded parent expects its original absolute value to
be restored. Production deadline handling is unchanged.

On Linux x86_64 with Python 3.14.7, the exact two affected cases passed
2/2 without an inherited deadline (raw log SHA-256
`046beeeecd88cd8edf01651d10c0552ab64c665bef56d327058c7e7606920575`).
The named F27 selection of `ProcessTests`, `MetadataSetupTests`,
`NativeExecutionTests`, and `SysrootSetupPolicyTests` passed 48/48 with
`SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC=999999999999` (raw log SHA-256
`0aef50b55d4e853975d58548a042c61741036c7d1ec9d096dd3305524bb987a8`).
The guardrail's own `uv lock --project verification --check` passed (raw
SHA-256 `96377c696066a113e088108953d2074604421d823a8009f91723d71bb3c637c8`),
and `uv run --project verification --locked python -m sifr_verify --self-test`
passed under the same inherited deadline (raw SHA-256
`0e41eda47e461d121353c1d52c9ce431dab7703f4464eedf25bf850fcc60d66c`).
The first full self-test attempt failed in unrelated DX.4 fixture setup
because this new worktree had no `target/` directory (raw SHA-256
`1c644de3cd66dc147d474dfb6c70ee6c3b4b43885e0c1297fe727d4ba77be34a`);
creating that worktree-owned directory enabled the passing rerun. Python
compilation, the 900-line file-size guardrail (4,244 files), and diff
check passed. Raw logs and final read-only review are outside the Git
tree under `/home/yaser5/projects/sifr/architecture-v04-f27-gate-selftest-evidence-20260926/`.

The scoped Opus review on the exact candidate returned **SATISFIED** with
no blocking findings (response SHA-256
`19d3c4e86eb94ee7c595edfed60bbc4599010040f044447b1e6d75b42e1676f8`).
It noted nonblocking suggestions about strict absence checking and a
near-expiry inherited deadline, plus the unrelated initial fixture setup
failure. This intermediate repair does not qualify the Emitted-Rust full
merge gate; that owner must rerun on its final candidate. This
record-only update passed documentation structure and diff checks (raw
documentation log SHA-256
`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`).
The first documentation attempt failed because the pinned editor
integration submodules were not initialized; that setup failure is retained
(raw SHA-256
`2763f3d9cd4c14634ef98b55fdb11299909953df09d1c1002e3d5b87a53f391a`).

## Emitted-Rust final integration external blockers (2026-09-24)

The final Emitted-Rust candidate `eafa57e22df22d6d8172adf8c65aa5c42aea8acb`
ran the canonical merge profile in its own worktree. With the approved
`linux-i7-4720hq-12gb-dev-v1` reference explicitly selected, Python 3.14.7
and ext4 target/temporary storage matched, but V01 admission rejected the
live `schedutil` CPU governor against the captured `performance` policy.
The exact failed log is
`/home/yaser5/projects/sifr/emitted-rust-final-integration-evidence-20260924/merge-reference-eafa57e22d.log`
(SHA-256 `d66a526bb249720a2f8a9d22d357c399acd6212b2f587c383dff715b73ffe1e7`).
The gate stopped before Cargo or any selected assertion. V01 retains the
owner-controlled host-policy prerequisite described in its delivery receipt;
this qualifier did not change the global governor or approve another reference.

A separate exact merge-selected `coverage_matrix/readiness` area command
failed on two missing Cargo-package classifications:
`sifr_cache_storage` and `sifr_compiler_services`. Its failed JSON is
`/home/yaser5/projects/sifr/worktrees/emitted-rust-final-integration-20260924/target/verification/areas/coverage-final-eafa57e22d.json`
(SHA-256 `3417f0c1f555b9089a7cadd89cf702f15300a1d4632796dd38cfb360e68f4d3b`).
The X01 coverage registry owner repaired this prerequisite in
[PR #4024](https://github.com/sifr-lang/sifr/pull/4024). The failure remains
historical evidence; the Emitted-Rust qualifier must rerun its own selection
after this registry update. V01 host admission is a separate blocker.
The [emitted-Rust phase record](ad-hoc-emitted-rust-excellence.md#final-integration-qualifier-blocked-by-v01-host-admission-2026-09-24)
holds the separate passing and partial area receipts.

## Preparation/F33 crosswalk and bounded delivery

| ID | Owner / findings / status | Required acceptance evidence |
| --- | --- | --- |
| P00 | This record; F33; record-only preparation. | One owner, evidence class, bounded acceptance and order for F01-F34 and M1-M13. Documentation structure check. No named Sifr executable case. |
| X01 | Generated Rust owner in [Emitted Rust Excellence](ad-hoc-emitted-rust-excellence.md); F01; merged in [PR #3916](https://github.com/sifr-lang/sifr/pull/3916), receipt below. | Legal unwrap/expect names in check, emit, build, native, project and test flows; compiler-owned extraction still rejected. |
| X02 | Existing Emitted Rust owner, including retained Item 12; F02-F04 and codegen part of F32; open after X01. | Contextual Result failures at every public emission boundary, no partial output, origin-aware exhaustive IR/final-source validation, trusted macro/bridge policy, diagnostic and fixture checks. |
| V01 | Compiler/performance qualification owner; F05; fail-closed admission merged and live selected-reference representative qualification passed on main `8e8ec4255cf4ec3ffcab577adfc100efa8aa4a7b`. | Compatible reference selected before long work, controlled-clock freshness boundary tests, host/toolchain admission and actual selected reference check. Old Mac capture is expired; Linux reference is host-specific. |
| V02 | Verification runner owner; F06; merged in [PR #3921](https://github.com/sifr-lang/sifr/pull/3921), receipt below. | One Python-interop area ID across selection, receipt, export and required cache paths; mutated suite/cache inputs alter identity and cold classification. |
| V03 | Verification guard owner; F24; merged in [PR #3926](https://github.com/sifr-lang/sifr/pull/3926), receipt below. Current-main inventory drift was repaired in [PR #4043](https://github.com/sifr-lang/sifr/pull/4043), with C02b, C02c and C02d handoffs reconciled in [PR #4061](https://github.com/sifr-lang/sifr/pull/4061), [PR #4065](https://github.com/sifr-lang/sifr/pull/4065) and [PR #4069](https://github.com/sifr-lang/sifr/pull/4069), respectively. | Complete relevant Rust target roots and classify filesystem effects at site/symbol level as semantic input, build identity, tooling input or output effect. Detect builder reads and filesystem writes/mutations. Negative tests insert a new read in an already-listed file, an alias-only read, a byte read, a new crate/bin source, and representative builder/write sites. |
| V03b | Semantic parser guard owner; F25; merged in [PR #3929](https://github.com/sifr-lang/sifr/pull/3929), receipt below. | Detect aliased parse, parse_module_raw and parse_module_suite calls, and adjudicate existing parse-and-lower sites. Separate production from test-only code, including mixed files and cross-file module gating; negative tests cover aliased parse in mixed test/production source. |
| V04/F26 | Verification process owner; F26; merged in [PR #3931](https://github.com/sifr-lang/sifr/pull/3931), receipt below. | Validate thread and signal lifecycle constraints before spawn or establish guaranteed cleanup ownership immediately. Worker-thread signal setup, selector and callback failures leave no live child or descendants. Preserve current useful descendant/terminal behavior. Named acceptance: `sifr_verify.dx3_process_checks.ProcessTests` F26 negative cases and focused existing process cases. |
| V04/F27 | Verification process owner; F27; merged in [PR #3933](https://github.com/sifr-lang/sifr/pull/3933), receipt below. | Absolute deadlines cover nested work and lock waits; distinguish native exit 124, timeout and cancellation. Preserve useful descendant/terminal behavior. F26 setup cleanup evidence does not qualify F27. |
| N01/F15 | Driver test orchestration; supported build/test policy parity; merged in [PR #3939](https://github.com/sifr-lang/sifr/pull/3939), with package-root selection repaired in [PR #4006](https://github.com/sifr-lang/sifr/pull/4006), receipts below. Coordinates with existing [DX9-F3](ad-hoc-native-cargo-reuse-followups.md) without claiming its cache-scope acceptance. | Normal/locked/offline/frozen `sifr test` and build share authoritative resolution, sysroot configuration, native-link, tracing, hermetic environment and materialization policy. Classify unsupported modes explicitly. Focused exact crate/suite/case parity regressions. No implicit Python-test expansion or N02 warm-consumer identity claim. |
| N01/F16 | Driver native storage and test orchestration; merged in [PR #3943](https://github.com/sifr-lang/sifr/pull/3943), receipt below. | Alternating dependencies in one owned mutable root, same-key concurrency and immutable final output. Keep mutable leased family and immutable final snapshot distinct. |
| N01r/F15-F16 | Bounded filesystem-effect inventory follow-up; merged in [PR #3948](https://github.com/sifr-lang/sifr/pull/3948), receipt below. | Adjudicate the 27 N01-owned V03 guard failures from N02a, including one stale row; keep test orchestration, production package selection and filesystem mutations in their correct site/symbol effect classes. |
| N02a/F07 | Package-graph identity owner; first N02 batch, merged in [PR #3945](https://github.com/sifr-lang/sifr/pull/3945), receipt below. | Replace persisted package-owned FNV graph/metadata/source/probe/build-input identities with schema/domain-framed collision-resistant encodings of complete semantic inputs. Test field changes and absent/empty/unreadable package-source states. The package-owned digest producers must be ready for authority/payload verification by their consumers. |
| N02b/F07 | Driver, bridge and source-map/graph warm-consumer identity owner; merged in [PR #3951](https://github.com/sifr-lang/sifr/pull/3951), receipt below. | Replace correctness-sensitive persisted driver/bridge FNV identities with versioned, domain-framed collision-resistant encodings; bind every semantic field. Verify package graph/source-map authority and payload at relevant driver warm consumers, including absent, empty and unreadable states. Preserve Cargo-before-final-cache architecture and DX9-F7 behavior. Prepared-lock and unchanged-lock identities are N02b2. |
| N02b2/F07 | Prepared-lock and unchanged-lock identity owner; merged in [PR #3954](https://github.com/sifr-lang/sifr/pull/3954), receipt below. | Replace remaining persisted FNV lock identities with versioned, domain-framed collision-resistant encodings. Bind every semantic field and verify authority/payload at prepared and unchanged-lock consumers, including absent, empty and unreadable states. Preserve normal seed/reconciliation and Cargo-before-final-cache behavior. |
| N02c0/F08 | CLI trace-consumer handoff; merged in [PR #3960](https://github.com/sifr-lang/sifr/pull/3960), receipt below. | Preserve the four new fixed cache-status literals as distinct trace classifications; retain redaction for unknown or sensitive status values. Positive and negative CLI trace tests. No package/driver serialization implementation or F08 completion claim. |
| N02c/F08 | Package/driver serialization owner; merged in [PR #3964](https://github.com/sifr-lang/sifr/pull/3964), receipt below. | Serialization errors must return an error or disable optional reuse without publishing success. Distinguish absent, empty, unreadable and failed serialization at consumers; inject failure where reachable without inventing a production trigger. |
| N03 | Sysroot/native-context identity owner; F09; merged in [PR #3966](https://github.com/sifr-lang/sifr/pull/3966), receipt below. | Actual build context distinguishes declared variable absent, empty and literal <unset> with a versioned encoding. Prior Python model is not proof of a stale binary. |
| N04 | Driver interop/probe-input owner; F10-F11; merged in [PR #4008](https://github.com/sifr-lang/sifr/pull/4008), receipt below. | Required/optional roots, symlinks, errors and consumed bytes follow one snapshot policy; edit within one process invalidates digest. Direct probe still runs Cargo; DX9-F1 owns unused receipt disposition. |
| N05 | CLI formatter-cache owner; F12; merged in [PR #4010](https://github.com/sifr-lang/sifr/pull/4010), receipt below. | Marker binds implementation, options, source and path, has schema/ownership checks and atomic publication; revision-change and failure tests. |
| N06 | DX.3 storage owner and existing DX9-F5; F13-F14; merged in [PR #4012](https://github.com/sifr-lang/sifr/pull/4012), receipt below. | Complete owner inventory, lease-aware scoped pruning and bounded waits; wedged sibling, abandoned staging and live-child tests. Do not delete another session's target. |
| C01 | Frontend product owner; F17-F18; merged in [PR #4014](https://github.com/sifr-lang/sifr/pull/4014), receipt below. | Deterministic SCC/cycle diagnostics and one product for single/project/package/test retaining SQL, specialization, adapter, flow, exports and spans; same-snapshot CLI/analysis equivalence and measured memory. |
| C02a0 | Metadata production and storage owner; F19 first delivery after C01; merged in [PR #4018](https://github.com/sifr-lang/sifr/pull/4018), receipt below. | Move source stdlib bootstrap, metadata encoding/production, `PreparedMetadata` and project metadata transport into lower compiler services; move metadata lease/private-file/bounded-wait/atomic-publication primitives into lower cache storage. Keep the driver's current reader and `CompilerContext` as the sole provider/generation owner. Independent compile, named tests and merge below. |
| C02a1 | Metadata consumer and stdlib service owner; F19 after merged C02a0; merged in [PR #4022](https://github.com/sifr-lang/sifr/pull/4022), receipt below. | Move metadata selection/decoding/provider/navigation/qualification, tooling sysroot views, `stdlib_external_defs` and `CompilerContext` into lower compiler services. Driver reexports/delegates and removes originals without a second provider cache. Independent compile, named tests and merge below. |
| C02b | Python authoring service owner; F19 after C02a1; merged in [PR #4059](https://github.com/sifr-lang/sifr/pull/4059), receipt below. | Move editor runtime selection, environment/target probes, certification checks, interop-plan status and package diagnostic conversion below driver; preserve exact declaration identity and cancellation. Named tests and boundaries below. |
| C02c | SQL editor service owner; F19 after C02b; merged in [PR #4063](https://github.com/sifr-lang/sifr/pull/4063), receipt below. | Move editor profile discovery/preparation and profile import diagnostics below driver while retaining component and schema behavior. Named tests and boundaries below. |
| C02d | Preview and editor-check service owner; F19 after C02c; merged in [PR #4067](https://github.com/sifr-lang/sifr/pull/4067), receipt below. | Move generated Rust preview and editor check restore below driver using the C01 frontend product and explicit metadata/semantic identity. Named tests and boundaries below. |
| C02e | Dependency-direction guard owner; F19 after C02d; merged in [PR #4071](https://github.com/sifr-lang/sifr/pull/4071), receipt below. | Remove direct analysis/LSP driver dependencies and references, then enforce the lower-service boundary across manifest package aliases, optional features and explicit Cargo target paths. Negative tests and final C02 closure below. |
| E01a | LSP/package external-input owner; F20-F21; merged in [PR #4075](https://github.com/sifr-lang/sifr/pull/4075), receipt above. | Per-root external snapshots and generations; manifest/lock/config/bridge/binding/certification/environment transitions, positive and negative cache invalidation, unchanged warm reuse. Named cases above. |
| E01b | LSP watcher/protocol owner; F20-F21 after merged E01a; merged in [PR #4077](https://github.com/sifr-lang/sifr/pull/4077), receipt above. | Capability-gated registration and response acknowledgement; rejected/unsupported/unconfirmed per-request revalidation, lifecycle events, reconnect, storms and multi-root isolation. Named cases above. |
| E01c | LSP request/diagnostics owner; F20-F21 after merged E01b; merged in [PR #4081](https://github.com/sifr-lang/sifr/pull/4081), receipt above. | Verified fast-hit reorder and end-to-end current request/diagnostics publication under source/config/external changes, nested-to-ancestor root reassignment, unstable inputs, cancellation and multi-root lifecycle. Named cases above. |
| E02 | Analysis/lint owner; F22; merged in [PR #4083](https://github.com/sifr-lang/sifr/pull/4083), receipt above. | Current-revision diagnostics/actions reuse canonical parsed/HIR input; changed fix text gets new snapshot; policy/suppression equivalence. |
| E03 | LSP performance owner; F23; merged in [PR #4085](https://github.com/sifr-lang/sifr/pull/4085), receipt above. | Actual Python declaration cache deltas in 25 connected modules: cold, unchanged, private/API edit, external change and cancellation/recovery. |
| E04 | Separate editor correctness owner; F34; merged in [PR #4087](https://github.com/sifr-lang/sifr/pull/4087), receipt above. | Real type symbols and nonempty hierarchy edges, lowercase names, uppercase non-types, imported bases and edits; otherwise reconcile advertised capability/docs explicitly. |
| H01/F28 | Fuzz/property owner; needs-new-scope, split into H01a-H01i below; H01a-H01d and H01f-H01h merged in [PR #4090](https://github.com/sifr-lang/sifr/pull/4090), [PR #4093](https://github.com/sifr-lang/sifr/pull/4093), [PR #4096](https://github.com/sifr-lang/sifr/pull/4096), [PR #4100](https://github.com/sifr-lang/sifr/pull/4100), [PR #4104](https://github.com/sifr-lang/sifr/pull/4104), [PR #4107](https://github.com/sifr-lang/sifr/pull/4107), and [PR #4110](https://github.com/sifr-lang/sifr/pull/4110), receipts below. Whole H01 qualification remains open. | Real guided execution plus semantic normalization, narrowing, ownership, incremental/full and deterministic-codegen properties; separate build/tool/timeout/findings and minimized seeds. Complete only after all nine bounded items pass their named acceptance and merge. |
| H02/F29 | Typed lowering and unsafe bridge owners; needs-new-scope, split into H02a0, H02a1, H02b-H02c, H02d0-H02d1 and H02e-H02h below. H02a stopped at needs-new-scope in [PR #4115](https://github.com/sifr-lang/sifr/pull/4115); H02a0 merged in [PR #4118](https://github.com/sifr-lang/sifr/pull/4118), H02a1 merged in [PR #4120](https://github.com/sifr-lang/sifr/pull/4120), H02b merged in [PR #4122](https://github.com/sifr-lang/sifr/pull/4122), and H02c merged in [PR #4124](https://github.com/sifr-lang/sifr/pull/4124); H02d stopped needs-new-scope in [PR #4127](https://github.com/sifr-lang/sifr/pull/4127); H02d0 merged in [PR #4130](https://github.com/sifr-lang/sifr/pull/4130); H02d1 merged in [PR #4132](https://github.com/sifr-lang/sifr/pull/4132); H02e merged in [PR #4134](https://github.com/sifr-lang/sifr/pull/4134); H02f merged in [PR #4136](https://github.com/sifr-lang/sifr/pull/4136); H02g merged in [PR #4138](https://github.com/sifr-lang/sifr/pull/4138); H02h remains open. | Semantic dispatch classification, strict decline and ABI/lifetime/alias/ownership/callback runtime contracts. Regex counts are not acceptance. Complete only after all ten bounded items pass their named acceptance and merge. |
| H03 | Maintainability/flow owner; F30; open. | Current normalized ratchets and API/fan-out evidence; flow equivalence and resource measurements before removal. |
| D01a | Diagnostic and verification registry prerequisite; non-codegen F32; merged in [PR #3936](https://github.com/sifr-lang/sifr/pull/3936), receipt below. | Active code identity, owner-module and fixture references, related-span JSON, verification mutation inventory and negative drift tests. Do not change codegen diagnostics or historical numeric codes. |
| D01b | Documentation owner; F31 and final current maps; open after structural delivery. | Alias/target-aware maps, current API/path/link checks, active-named release-record status and links; historical receipts preserved. |
| D01c | Diagnostic registry completeness owner; residual non-codegen F32 follow-up, open after structural delivery. | Reconcile declared owner modules with actual emitters and replace bare-file representative references where a case-level reference exists; retain D01a negative guardrails. |
| Q01 | Final integration qualifier; all retained criteria; open after required owner merges. | Exact-candidate full merge profile, companion freshness, compatible performance reference and owner handoffs; whole-phase review and separate docs-only closure. |

Rows crossing packages specify a contract handoff; delivery splits at package ownership boundaries. Do not use this record to absorb another active issue.

### H02/F29 typed lowering and unsafe bridge scope split (2026-09-29)

H02 is **needs-new-scope** on current main. M9/#3561 was unmerged; its
reviews and gates are historical, not current-main acceptance. F29 requires
semantic dispatch classification and executable ABI contracts, not counts of
method names or `unsafe` tokens. Current main divides method behavior across
`sifr_lowering`, `sifr_frontend::const_evaluator`, and
`sifr_codegen::methods`; Python runtime callbacks, buffers, Arrow and
DLPack are separate unsafe bridges. The old M9 dispatch and unsafe policy
scripts are absent on current main. These bounded items replace the single
cross-subsystem H02 row. New case names below are reserved acceptance, not
existing passes. Each item gets its own exact-candidate tests, focused
regressions, scoped review and merge receipt before a dependent item begins.

| Item / owner | Dependency and bounded implementation | Named acceptance to add and run |
| --- | --- | --- |
| H02a0 / typed HIR and metadata carrier | Merged in [PR #4118](https://github.com/sifr-lang/sifr/pull/4118); first; one atomic, independently compiling contract prerequisite across `sifr_ir` (HIR field/type), `sifr_sysroot` (wire record, references and container version), and `sifr_compiler_services` (encoder/decoder). These are explicit package handoffs, not lowering or codegen semantics ownership. Add a typed authority field to `HirExpr::MethodCall` and its wire variant; preserve receiver convention, checked receiver target, mutable-argument places and source ranges. Mechanical constructor updates outside these packages are limited to making the new carrier compile; do not classify methods there. Carry an explicit unclassified transition state only until H02a1, never infer builtin authority from a missing field. Bump the private indexed metadata container version from 4 to 5 for the changed record shape; reject v4 and malformed/new unknown authority tags rather than decoding them as a default. Update the explicit record corpus. No frontend constant evaluation or codegen emission. | Add and run `cargo test -p sifr_ir --lib method_authority_tests::carrier_preserves_receiver_and_source -- --exact`; `cargo test -p sifr_sysroot --lib metadata::tests::h02a0_authority_record_roundtrip -- --exact`; `cargo test -p sifr_sysroot --lib metadata::tests::h02a0_old_version_and_invalid_authority_reject -- --exact`; `cargo test -p sifr_compiler_services --lib metadata::tests::h02a0_hir_authority_roundtrip -- --exact`; `cargo test -p sifr_compiler_services --lib metadata::tests::h02a0_corrupt_authority_rejects -- --exact`. Focused regressions: `cargo test -p sifr_sysroot --lib metadata::tests::all_explicit_record_families_and_variants_roundtrip -- --exact`, `cargo test -p sifr_sysroot --lib metadata::tests::dx15_physical_frames_are_bounded_canonical_and_versioned -- --exact`, `cargo test -p sifr_compiler_services --lib metadata::tests::incompatible_identity_rejects_provider -- --exact`, `cargo test -p sifr_lowering --lib lower::method_receiver_analysis_tests::builtin_method_calls_carry_canonical_receiver_conventions_and_source_ranges -- --exact`, and `cargo test -p sifr_lowering --lib name_resolution_snapshot_tests::name_resolution_snapshot_matrix_matches_lowered_name_facts -- --exact`. Handoff: independently compiling carrier; both direct and encoded/decoded HIR retain identical authority, receiver data and ranges; old/corrupt metadata declines; no generated dispatch changes. |
| H02a1 / `sifr_lowering` typed classification | Merged in [PR #4120](https://github.com/sifr-lang/sifr/pull/4120); after H02a0. At typed HIR construction, replace the transition state with semantic builtin/intrinsic, protocol, local/inherited nominal, imported and Rust-adapted authority, using resolved declaration identity rather than name alone. Preserve receiver ownership, convention and ranges; unresolved receivers and unsupported method/type pairs produce diagnostics. Assert every accepted source method call has a classified carrier, including imported/inherited and compiler-synthesized calls, before the HIR leaves lowering. No metadata schema change, frontend constant evaluation or codegen emission. | Add and run `cargo test -p sifr_lowering --lib method_authority_tests::typed_dispatch_classification -- --exact`; `cargo test -p sifr_lowering --lib method_authority_tests::unsupported_method_declines_with_diagnostic -- --exact`; `cargo test -p sifr_lowering --lib method_authority_tests::unclassified_method_cannot_leave_lowering -- --exact`. Focused existing `lower::method_receiver_analysis_tests`, `lower::own_mut_semantics_tests` and `lower::python_interop_callback_tests` selections must record resolved names/assertion counts. Handoff: H02b/H02c/H02d0 consume a merged typed HIR product whose method authority is populated, whose receiver data survived metadata reuse, and whose declines carry source diagnostics. |
| H02b / `sifr_frontend` constant evaluation | Merged in [PR #4122](https://github.com/sifr-lang/sifr/pull/4122) after H02a1. Classify the closed `@const_eval` method subset as compile-time semantics, compare results and failures with the corresponding runtime language contract, and decline unsupported or effectful methods. Preserve frontend product ownership. | `cargo test -p sifr_frontend --lib const_evaluator::method_authority_tests::supported_methods_match_runtime_semantics -- --exact`; `cargo test -p sifr_frontend --lib const_evaluator::method_authority_tests::unsupported_methods_decline -- --exact`; focused existing `const_evaluator` tests. |
| H02c / `sifr_codegen` method authority | Merged in [PR #4124](https://github.com/sifr-lang/sifr/pull/4124) after H02a1 and H02b; receipt below. Own typed builtin/intrinsic emission in one source-method authority. Classify other name branches as user/protocol dispatch, contextual Rust adaptation or Rust-IR consumption. On authority decline, only a proven user/protocol/contextual path may continue; unsupported builtins fail structurally. Reproduce historical list `append`/`cloned` strict-registry-decline risk before removing fallback. Reject a remaining unclassified carrier before source-method emission; mark codegen-created contextual calls explicitly rather than treating that state as a builtin fallback. No X02 generated-Rust safety rewrite. | `cargo test -p sifr_codegen --lib method_authority_tests::typed_builtin_dispatch_and_strict_decline -- --exact`; `cargo test -p sifr_codegen --lib method_authority_tests::user_protocol_and_contextual_paths -- --exact`; `cargo test -p sifr_codegen --lib method_authority_tests::list_append_cloned_decline_regression -- --exact`; focused `methods::tests` and `lib_codegen_tests::emitted_rust_quality_codegen_tests`, plus exact list-method E2E pass/fail fixtures through the existing runner. Assert behavior, diagnostic failure and no second builtin fallback. |
| H02d / original runtime-only boundary | **Needs-new-scope**, recorded in [PR #4127](https://github.com/sifr-lang/sifr/pull/4127) from audit base `eeb61f1c1710e2b587c46fbc47232076a5ac8544`. Superseded for execution by H02d0 and H02d1; this historical row is not an additional implementation item. | Source-audit findings below; no implementation or acceptance pass. |
| H02d0 / callback runtime API and generated caller contract | Merged in [PR #4130](https://github.com/sifr-lang/sifr/pull/4130) after H02a1; receipt below. No additional semantic dependency on H02b/H02c; the current base already includes their merges. One atomic, independently compiling handoff across `sifr_runtime::python::callbacks` (construction, admission, lifetime and teardown authority) and `sifr_codegen` (only the matching generated constructor/cleanup callers). Replace the safe lifetime-erased borrowed facade with a safe structural scope that survives forgetting exposed handles and drains on cancellation, or a narrow unsafe borrowed-construction contract used by compiler-generated scope glue with proved teardown. Keep owned `'static` callbacks safe. Prove borrowed captures stay live through all admitted decoding, execution and polling; normal return, conversion/setup error, escaped shell, early drop, cancellation, reentrant close and exact-once release must terminate safely. A cancelled closer must transfer or finish close authority so later closers terminate and captures release once. No compatibility facade, unrelated runtime-core audit, Buffer/Arrow/DLPack change, bridge-module change or X02 generated-Rust safety rewrite. | Add and run `cargo test -p sifr_runtime --features python --lib python::callbacks::h02_contract_tests::borrowed_callback_lifetime_and_thread_owner -- --exact`; `cargo test -p sifr_runtime --features python --lib python::callbacks::h02_contract_tests::close_cancel_reentrancy_releases_once -- --exact`; `cargo test -p sifr_runtime --features python --lib python::callbacks::h02_contract_tests::borrowed_callback_forget_and_escape_is_rejected_or_drained -- --exact`; `cargo test -p sifr_runtime --features python --lib python::callbacks::h02_contract_tests::cancelled_closer_handoff_releases_once -- --exact`. Run the three exact existing codegen caller cases and the focused runtime/codegen selections below. Require a compile-fail or structurally safe forget/escape proof, a cancelled-closer handoff regression, local thread/lifetime/alias/ownership contracts for changed unsafe boundaries, and independently compiling runtime plus generated callers. |
| H02d1 / `sifr_runtime::python` CPython core and remaining callback audit | Merged in [PR #4132](https://github.com/sifr-lang/sifr/pull/4132); receipt below; after merged H02d0, runtime-only. Finish the original initialization/GIL/foreign-object and callback-state audit against the repaired lifetime API: initialization/error state, GIL attachment/detachment, foreign-thread entry, object/refcount lifetime, `Send`/`Sync`, registration, in-flight close/cancel/reentrancy and capture release. Give each remaining owned unsafe boundary a local thread, lifetime, alias and ownership contract; narrow broad allowances to a function or cohesive ABI module. Reuse H02d0's established callback contract; do not reopen its codegen handoff or take buffer, Arrow, DLPack, bridge modules or X02. If another package contract must change, stop needs-new-scope. | Add and run `cargo test -p sifr_runtime --features python --lib python::h02_contract_tests::initialization_and_gil_entry_contract -- --exact`; `cargo test -p sifr_runtime --features python --lib python::h02_contract_tests::foreign_object_thread_lifetime_and_release_contract -- --exact`. Run `cargo test -p sifr_runtime --features python --lib python::tests::` and `cargo test -p sifr_runtime --features python --lib python::config_verify::tests::`, the four callback selections below, and all four exact H02d0 runtime cases on the final H02d1 candidate. Assert repeated/conflicting initialization, uninitialized and foreign-thread GIL entry, detached-thread release draining, no premature release or access after semantic close, and exact-once final release. |
| H02e / `sifr_runtime::python::buffer_ops` | Merged in [PR #4134](https://github.com/sifr-lang/sifr/pull/4134); receipt below. After merged H02d1 (and its H02d0 prerequisite). Own `Py_buffer` acquisition/release, shape/stride/bounds/alignment, writable alias admission, indirect access and exporter lifetime. Document unsafe pointer/read/write and `Send` contracts; reject conflicting aliases and malformed layouts before access. | `cargo test -p sifr_runtime --features python --lib python::buffer_ops::h02_contract_tests::layout_bounds_and_alias_admission -- --exact`; `cargo test -p sifr_runtime --features python --lib python::buffer_ops::h02_contract_tests::all_release_paths_are_exact_once -- --exact`; focused existing `python::buffer_ops::tests`, `python::buffer_ops::release_evidence_tests` and `python::buffer_ops::typed_access_evidence_tests`. Include negative strides, indirect pointers, shared storage, failure, explicit release and drop. |
| H02f / `sifr_runtime::python::arrow_ops` | Merged in [PR #4136](https://github.com/sifr-lang/sifr/pull/4136), receipt below; after merged H02d1 (and H02d0); independent of H02e. Own Arrow C Data/Stream/Device ABI layout, capsule identity, pointer lifetime, alias/transfer rules and release callbacks. Validate nullable callbacks and malformed capsules before dereference; distinguish borrowed observation from consumed ownership and prove exact-once release. | `cargo test -p sifr_runtime --features python --lib python::arrow_ops::h02_contract_tests::abi_layout_and_capsule_transfer -- --exact`; `cargo test -p sifr_runtime --features python --lib python::arrow_ops::h02_contract_tests::stream_callbacks_release_exactly_once -- --exact`; focused existing `python::arrow_ops::tests`, including malformed capsule, full/partial/failed consumption and stream/device release. |
| H02g / `sifr_runtime::python::dlpack_ops` | Merged in [PR #4138](https://github.com/sifr-lang/sifr/pull/4138), receipt below; after merged H02d1 (and H02d0); independent of H02e-H02f. Own legacy/versioned ABI layout, capsule one-shot consumption, device/stream validity, tensor lifetime, deleter transfer and exact-once release on rejection/reset/drop. Do not change declaration certification policy. | `cargo test -p sifr_runtime --features python --lib python::dlpack_ops::h02_contract_tests::legacy_versioned_layout_and_transfer -- --exact`; `cargo test -p sifr_runtime --features python --lib python::dlpack_ops::h02_contract_tests::rejection_and_reset_release_exactly_once -- --exact`; focused existing `python::dlpack_ops::declaration_tests`. |
| H02h / method and unsafe policy integration | After merged H02a0, H02a1, H02b, H02c, H02d0, H02d1, H02e, H02f and H02g. Record semantic site classifications and local unsafe contracts with normalized site fingerprints, not per-file counts. Detect changed/new/stale dispatch under renamed bindings; reject a second language-semantics owner, broad item-level unsafe allowances and operations without local contracts. Mask strings/comments/characters without hiding Rust lifetimes; self-tests work in a cold checkout. Assign SQL, cache, driver and generated-code sites to existing owners, without editing those packages. A static scan does not prove runtime safety. | `python3 scripts/check_method_dispatch_authority.py --self-test`; `python3 scripts/check_method_dispatch_authority.py`; `python3 scripts/check_unsafe_abi_contracts.py --self-test`; `python3 scripts/check_unsafe_abi_contracts.py`; exact H02a0-H02a1, H02b-H02c, H02d0-H02d1 and H02e-H02g named cases and focused list-method/Python bridge regressions on the integrated candidate. Negative fixtures cover renamed dispatch, second owner, stale fingerprint, broad allowance, missing contract and character-literal masking. |

H02d0 is the explicit exception to splitting delivery at a package boundary:
the runtime API and the generated caller/teardown proof are one atomic
contract. A runtime-only unsafe API breaks the existing safe callers; keeping
the old safe borrowed facade for a later codegen merge preserves the
unsoundness. Runtime owns its callback paths and codegen owns only their
construction/cleanup callers. The combined candidate must independently
compile and merge once before runtime-only H02d1 starts. This does not
authorize another cross-package safety rewrite.

H02d0 must run these existing exact caller checks (one selected case each):

- `cargo test -p sifr_codegen --lib python_interop_direct_tests::typed_current_callback_emits_checked_adapter_failure_reconciliation_and_cleanup -- --exact`
- `cargo test -p sifr_codegen --lib python_interop_async_tests::asyncio_callback_emits_owned_loop_factory_async_handler_and_async_drain -- --exact`
- `cargo test -p sifr_codegen --lib python_interop_async_tests::foreign_callback_in_async_wrapper_uses_nonblocking_drain -- --exact`

Its focused regressions are exactly
`cargo test -p sifr_runtime --features python --lib python::callbacks::tests::`,
`cargo test -p sifr_runtime --features python --lib python::callbacks::asyncio_tests::`,
`cargo test -p sifr_runtime --features python --lib python::callbacks::current_tests::`,
`cargo test -p sifr_runtime --features python --lib python::callbacks::ownership_tests::`,
`cargo test -p sifr_codegen --lib python_interop_async_tests::`, and these
exact callback caller regressions (each with `-- --exact`):

- `cargo test -p sifr_codegen --lib python_interop_direct_tests::retained_foreign_callback_is_aggregated_into_the_opaque_result_owner -- --exact`
- `cargo test -p sifr_codegen --lib python_interop_direct_tests::receiver_retained_callback_reuses_the_opaque_owner_slot -- --exact`
- `cargo test -p sifr_codegen --lib python_interop_direct_tests::retained_callback_owner_attribute_does_not_drop_an_absent_argument_frame -- --exact`
- `cargo test -p sifr_codegen --lib python_interop_direct_tests::retained_handler_failure_moves_into_typed_owner_sidecar_and_close_observes_it -- --exact`

These cover current/foreign/asyncio adapters, conversion and reconciliation,
call-scoped and retained ownership, unregister/rollback and asynchronous drain.

The forget/escape case must either run a Rust compile-fail fixture rejecting
safe borrowed construction/escape for the chosen unsafe contract, or exercise
the chosen safe structural scope: forget the exposed handle, retain a shell,
and prove no invocation/poll uses a capture after its borrow ends. Assert shell
rejection after teardown and drain of every admitted setup/invocation lease.
Do not execute a known unsound callback after freeing its capture. Comments,
normal-path close calls and a leaked target allocation are not lifetime proof.
The cancelled-closer case must admit an in-flight invocation, poll a closer to
its drain wait, cancel that elected closer, then prove another closer finishes
after invocation termination with no stranded `Closing` state, no remaining
poll of borrowed state and exactly one capture release.

H02d0's exact cases are implemented and qualified in its receipt below.
H02d1's exact cases remain reserved acceptance to add; the codegen caller checks
preceded H02d0. Resolve names with the same crate, features and filter
before running them; exact new and existing cases each require 1/1 selected
tests, and every focused selection requires nonzero resolved names and recorded
executed assertion counts. H02d1 and H02e-H02g likewise enable `--features python`
for all runtime named and focused selections. A feature-disabled or zero-test
success is not acceptance. Named evidence is prospective until the item's own
candidate is implemented, reviewed and merged.

H02c must add an H02-owned two-case pass manifest selecting
`list_append_extend_insert_registry` and `collection_cloning`, then run
`verification/runner/e2e/run_e2e_pass.sh --profile create-pr --fixture-manifest verification/areas/core_language/data/h02_method_e2e_manifest.json`.
The manifest and focused unit decline case form the exact list regression
selection; the current fail-fixture harness has no per-fixture selection.
The runtime items must run their exact named cases with `-- --exact`, and
each focused existing selection must record the resolved test names and
assertion counts in its own receipt. Keep Cargo targets warm in the owning
worktree; H02h may reuse prior passing evidence only if candidate,
configuration and validation inputs remain identical.

H02 closes only when H02a0, H02a1, H02b, H02c, H02d0, H02d1 and H02e-H02h merge in dependency order and integrated
typed dispatch preserves accepted behavior, strict decline, diagnostics and
receiver ownership. Runtime evidence must cover ABI layout, lifetime,
aliasing, transfer, callback entry and exact-once release in each owned
bridge. Native PostgreSQL FFI remains with the SQL owner, driver/cache
unsafe storage with its existing owner, and X02 owns generated-Rust safety.
Record failures in those packages in their owning issues. Q01 retains the
final exact-candidate full merge profile and whole-phase review; this split
claims no H02 test, review, gate or release.

The H02/F29 scope split merged in [PR #4113](https://github.com/sifr-lang/sifr/pull/4113)
as `d973fb5d46c249dcad55350b6902cc2cd17d468d` from candidate
`0a33a6188af984ddcca6bc0cd7b16f861d77c562` (base
`fc0e0d834e2b32169603eb93c610057c1be71d95`, tree
`678e7a847a9108cd9f1953c189e87d24026a8c8d`). Documentation structure,
the 900-line file-size guardrail and `git diff --check` passed for the
split and record; the guardrail source inputs were unchanged by the receipt. The documentation check initially stopped because the pinned nested
VS Code editor submodule was absent in this new worktree; initialization
restored the declared input, after which the check passed. No production
code, Cargo gate or external review was required for this documentation-only
scope split. H02a-H02h remain open; their named tests are prospective.

### H02a typed-method carrier boundary (2026-09-29)

H02a stopped at **needs-new-scope** on clean main
`6f2fb1ff8923d6c98c091539325c75944ff839e7`. It produced no
implementation, named test pass, review approval or typed-method closure.
The required authority cannot be retained within the assigned
`sifr_lowering` package alone. `HirExpr::MethodCall` is defined in
`crates/sifr_ir/src/hir_nodes.rs` with receiver convention, checked
receiver target and source ranges, but no dispatch-authority field.
`HirExpr::IntrinsicCall` distinguishes explicit intrinsics, while a
source method call carries no typed distinction among builtin,
protocol, local or inherited nominal, imported, and Rust-adapted
dispatch. The method call is serialized through
`crates/sifr_sysroot/src/metadata/hir_nodes.rs` and the
`crates/sifr_compiler_services/src/metadata/` encoder and decoder.
A lowering-only classification that is discarded at construction
would not preserve the typed decision for later consumers or metadata
reuse. Adding the field locally would break the shared HIR and wire
contracts, requiring changes in the other package owners.

The proposed bounded prerequisite is **H02a0**, owned by the HIR and
metadata-contract packages: add a typed method-authority carrier to
`HirExpr::MethodCall`, preserve the existing receiver ownership,
convention and source ranges, and round-trip the authority through
metadata with an explicit format/version decision and focused
positive/negative tests. Then **H02a1**, owned by `sifr_lowering`,
can classify accepted builtin/intrinsic, protocol, user/nominal,
imported and Rust-adapted calls at construction, reject unsupported
method/type pairs and unresolved receivers with diagnostics, and run
the two H02a named cases plus the three existing lowering regression
groups. H02b/H02c should depend on the merged H02a1 receipt. Neither
proposal authorizes frontend constant evaluation or codegen emission
in this item. No Cargo target was created or cleaned, and no broad
gate was attempted for this scope record.

The formal H02a0/H02a1 rows above replace the provisional H02a row and
make H02d depend on H02a1 as well. Neither replacement is implemented by
this documentation change.

### H02a0/H02a1 formal scope-split receipt (2026-09-29)

The H02a carrier/classification split merged in
[PR #4116](https://github.com/sifr-lang/sifr/pull/4116) as
`ebf8440fb71342ae5d038bda5d7d1ad4ad449121` from final documentation
candidate `b86b0d07f00fa849052620320e19f53653e0b996` (base
`1062a15a352e1f67599515d4fddd6d681a8754a5`). H02a0 owns the atomic
HIR/wire/codec carrier and metadata v5 handoff; H02a1 owns lowering
classification. H02b, H02c and H02d now depend on H02a1, with H02h and
whole-H02 order updated. Both child items are open; the listed cases are
prospective, not test passes.

The final documentation candidate passed
`python3 verification/areas/documentation/check_structure.py`,
`python3 scripts/check_hir_maintainability_guardrails.py`,
`python3 scripts/check_file_size_guardrails.py` (4,278 files), and
`git diff --check`. The first documentation check lacked the pinned nested
VS Code submodule; initializing `editor_integrations` recursively restored
that declared input and the check passed. A read-only scoped Opus review of
the initial docs candidate `b27c32d27d982fe6822a4f0b6d9583e9fe77ddab`
returned **SATISFIED**, with no blocking findings (response
`/tmp/sifr-h02a-formal-review.FtDDkq/response.md`, SHA-256
`e959d687e1e8b65b21c6f15609c077b417317c65fa604a71d014bc0cc1c11875`).
Its documentation suggestions were incorporated in the final docs-only
candidate; under the phase workflow, that edit needed documentation
rechecks, not a second external review. No production code, Cargo
test, full gate, or release qualification is claimed.

### H02a0 typed HIR and metadata carrier receipt (2026-09-29)

H02a0 merged in [PR #4118](https://github.com/sifr-lang/sifr/pull/4118) as
`c9770772fa1dba0d63eec2f5b5b36a0b3c72c7b0` from exact tested and reviewed
candidate `0a29f3ebaa173ff2f7937974df3f835357144c4a` (base
`af1218825ad16585a1330963192539da08464dc2`, tree
`4ddf2c5943ebf3be122803532b2c9f096c40a783`). `sifr_ir` now carries an
explicit `MethodAuthority` alongside the existing receiver convention, checked
receiver and argument places, and source ranges. The typed wire record and
compiler-service codec preserve those fields through encode/decode. The private
indexed metadata container is v5; v4 and malformed or unknown authority tags
decline. Constructors outside the carrier packages use the temporary explicit
`Unclassified` state. H02a1 still owns classification; this item changed no
frontend constant evaluation or generated method dispatch.

All five H02a0 named exact tests and all five specified focused regressions
passed one selected assertion each. The carrier/consumer package check and
codegen test compilation passed, as did generated encoder and decoder checks,
HIR maintainability, formatting, the 900-line file-size guardrail, diff check,
and documentation structure. Raw exact-command logs, digests, compiler and
submodule identities are in
`/data/sifr-h02a0-evidence-20260929/0a29f3ebaa173ff2f7937974df3f835357144c4a-validation.md`
(SHA-256 `bf424ead1e2a0064217adc048fd0d8d9bfae0245706772787221e35adef57e83`).

Read-only scoped Opus 5.5 review returned **SATISFIED** with no blocking
findings for that candidate (response
`/tmp/sifr-h02a0-review.Shb6GK/response.md`, SHA-256
`14497b5e30a26fe0b90cba98968d0de22be7410d51ce59ee8eab10f41fb19f29`).
It suggested that a future contract test forge an authority record through the
full compiler-service decoder and assert the precise unknown-tag failure after
record-integrity checks. The existing sysroot test verifies store-level forged
record rejection; these suggestions are nonblocking follow-up work. The
pre-existing DX.5 consumer-site inventory drift is recorded in its owning
[follow-up issue](ad-hoc-dx-metadata-review-followups.md). GitHub's create-PR
CI lane failed at performance-reference admission because
`SIFR_PERFORMANCE_REFERENCE` was unset; the intermediate-item exception did
not require that gate. No full merge gate or release qualification is claimed.

### H02a1 typed lowering classification receipt (2026-09-29)

H02a1 merged in [PR #4120](https://github.com/sifr-lang/sifr/pull/4120) as
`e26e60e9ed771b4050620e9ff736280aa3af3a0e` from exact tested and
reviewed candidate `789a9af137c86fdea26c9756b4ad99c12f117d40` (base
`46dfde674fe27b20bcfdc8e3ef3367915867d2db`, tree
`bbaa77166ab16bb7181d378c975e7af6fe449b13`). `sifr_lowering` now
classifies accepted typed method calls as builtin/intrinsic, protocol,
local/inherited nominal, imported or Rust-adapted at HIR construction. It uses
resolved receiver and declaration identity, preserves receiver convention and
source data, and emits diagnostics for unsupported pairs. Its verifier rejects
any remaining unclassified method call, including compiler-synthesized calls.
Synthetic task owners have their own identity so a same-named user class does
not acquire task authority. This item changed no metadata schema, frontend
constant evaluation or codegen emission.

All three H02a1 named exact tests passed one selected assertion each. Required
focused lowering selections passed: `method_receiver_analysis_tests` 27/27,
`own_mut_semantics_tests` 13/13 and `python_interop_callback_tests` 21/21.
Resolved test names are in the evidence directory. Additional focused task
lowering, method-call verification and task codegen selections passed 11/11,
4/4 and 15/15. Formatting, diff check, HIR maintainability, the 900-line
file-size guardrail and documentation structure passed. Exact candidate,
configuration, inputs and command results are recorded in
`/data/sifr-h02a1-evidence-20260929/789a9af137c86fdea26c9756b4ad99c12f117d40/validation.md`
(SHA-256 `9662cba07011ad79f0187d8d417d793870b32ebcc3d82eaa2fc85265f9074f89`).

The first scoped Opus 5.5 review returned **NOT SATISFIED** because user-defined
enum methods were classified as builtin. The candidate was repaired to resolve
enum methods by declaration, delegate newtype methods to the inner receiver and
use canonical attached-API bindings. Read-only remediation review of the final
candidate returned **SATISFIED**, with no blocking findings (response
`/data/sifr-h02a1-evidence-20260929/789a9af137c86fdea26c9756b4ad99c12f117d40/opus-review.md`,
SHA-256 `41b3a9b47dab15583e13239a24a1b1b4a427f9bb87094ba266de9a13c0be36d0`).
The reviewer suggested direct tests for the classifier's decline path, newtype
delegation and imported enum methods. Exact declaring-owner identity for an
inherited method on an imported class remains limited by exported metadata;
a future metadata-schema owner must add that origin if the semantic consumer
needs it. These are nonblocking follow-ups; this item does not expand metadata
scope. No full create-PR or merge gate or release qualification is claimed
under the intermediate-item exception.

**H02a1 lowering-owner follow-up (observed during H02b):** An optional
`cargo clippy -p sifr_frontend --lib -- -D warnings` fails on unchanged
`sifr_lowering/src/lower/method_authority.rs:237` with
`clippy::if_not_else`. The same command reproduced on exact H02b base
`54cb3d53161e4fb27b151a81fb99fd55f3720ab0`; raw baseline log SHA-256
`c552a7d27282487278faa9ff421201b4904f122b7d8d7cf64151cc61f2834802`
is under the H02b candidate-keyed evidence directory. This optional failure
does not revise the H02a1 named-test acceptance or the H02b acceptance.

### H02b frontend constant method evaluation receipt (2026-09-29)

H02b merged in [PR #4122](https://github.com/sifr-lang/sifr/pull/4122) as
`e6270454cc369a0c0e696614092049cdadfcf700` from exact tested and
reviewed candidate `52703f11582723e0705a3bdc5ade14c048535e63` (base
`54cb3d53161e4fb27b151a81fb99fd55f3720ab0`, tree
`fe09c1607e59028490c5504d3795a3856346ee44`). The frontend const
interpreter now requires the typed builtin declaration and matching receiver
type for its closed method subset. It evaluates `len` for strings, bytes,
tuples, lists and dicts, and `append` for a local list receiver. Unsupported,
effectful and nominal methods, mismatched declaration identities and mismatched
receiver values decline. String length counts Unicode scalar values, as in
runtime code generation; list append preserves mutation and the const
collection limit. No codegen or runtime implementation changed.

Both named exact H02b tests passed 1/1, and the focused
`const_evaluator::` selection passed 10/10 on the same candidate. Formatting,
the 900-line file-size guardrail (4,284 files) and diff check passed. Raw logs
are under
`/data/sifr-architecture-h02b-const-eval-evidence-20260929/52703f11582723e0705a3bdc5ade14c048535e63/`:
supported-method test SHA-256
`4051a61358169109eadb4445a0aa46752519d02b8cff97f876fbefb4b96e6a52`,
unsupported-method test
`e08a35546997019768474350e3d31b88137bb2cdcffd209d244224350179ca58`,
focused selection
`31fe0121fa89ed024f36be8785d1bad0b890d467602a59677cc8d996b6b6597b`,
and file-size guardrail
`bf52d5c724e4c999b5c36226a63751eced6ad6e011bb3b2990bb0fead1192f0b`.
Scoped read-only Claude Opus 5.5 review returned **SATISFIED** with no blocking
findings (response SHA-256
`64b378c616fb341844224815a1e68032b4c1d01b2784263685ded5f40acce534`).
Its suggestions to document future generic receiver support and a declined
`Option[str].len` fixture are nonblocking.

An optional strict frontend Clippy attempt failed in unchanged H02a1 lowering
code at `crates/sifr_lowering/src/lower/method_authority.rs:237` with
`clippy::if_not_else`. The same command and configuration reproduced the
failure on the unchanged H02b base (raw baseline SHA-256
`c552a7d27282487278faa9ff421201b4904f122b7d8d7cf64151cc61f2834802`).
This is a lowering-owner follow-up, not an H02b repair or passing Clippy claim.
The intermediate-item exception defers the full merge profile to H02 phase
integration; no release qualification is claimed. The record-only
documentation structure check passed after the pinned editor submodule was
initialized; its first missing-submodule setup attempt failed and is not a
passing check.

### H02c codegen method authority delivery receipt (2026-09-30)

[PR #4124](https://github.com/sifr-lang/sifr/pull/4124) merged as
`eba9f9adb83f647508b596936a0176efac34fed6` from exact tested and
reviewed candidate `9a7be02e543d7c386cf36acce75be979eb936205`
(original base `42fca77df7538cd04b3dde12d70b212c771b992c`, tree
`7a1c97ff4fc4d817402280bebbe9a709edaff6f2`). Codegen source-method
emission now uses the typed HIR authority. Unsupported builtin and
unclassified carriers fail structurally, while proven user, protocol and
contextual paths retain their dispatch. Nested receiver and argument errors
propagate through registry operand lowering; contextual checked indexed-list
length witnesses remain intact.

The fresh continuation used its own branch
`codex/architecture-h02c-e2e-retry-20260930` and worktree
`/data/sifr-architecture-h02c-e2e-retry-20260930`. It verified the exact
candidate/tree, clean source, pinned Ruff input, rustc/Cargo 1.98.1, Linux
target, empty rustflags, Cargo configuration and two compiler build jobs.
It also verified every prior validation/review log against its recorded hash.
No active compiler or open Cargo target lock was found before adopting the
explicitly transferred warm target
`/data/sifr-architecture-h02c-method-authority-20260929/target`.
The E01c owner had reclaimed 17,111,023,616 bytes from its own inactive
artifacts; the retry began with about 23 GiB free. This continuation cleaned
no target and preserved both earlier source worktrees.

The exact required H02-owned manifest command, with the same explicit E2E
cache as the failed attempt, passed **2/2** fixtures:
`list_append_extend_insert_registry` and `collection_cloning`.
The Rust `test_e2e_pass` harness passed 1/1. Both native groups were rebuilt
and executed, with **zero cache hits** and zero failed fixtures. The runner
reported compile 9,707 ms, build wall 23,109 ms and run 1 ms; its report
signature was `d2b5ed658a42c554`. These are functional-run observations,
not performance qualification. The compiler linked successfully and about
17 GiB remained free after the run.

Because implementation, configuration and validation inputs were unchanged,
the continuation reused the final candidate's three required exact method
cases (1/1 each), nested structural-error case (1/1), setdefault regression
(1/1), methods (12/12), emitted Rust quality (33/33), and selected codegen
library **1,739/1,739**. Exactly three independently reproduced H02a1
base failures remain excluded from 1,742 cases and separately owned in the
historical receipt below. The prior formatting, HIR maintainability, source
file-size and diff passes were also unchanged. Final scoped read-only Opus
5.5 approval remains **SATISFIED**, with no blocking findings, at response
SHA-256 `c7370218583843fd70be94b349a302796f1f41b9fe0c88699fc8c4d7bd9628f0`.
No repeated external review was required for this unchanged implementation.

Retry evidence is outside the reviewed tree at
`/data/sifr-architecture-h02c-e2e-retry-evidence-20260930/9a7be02e543d7c386cf36acce75be979eb936205/`.
Its validation manifest SHA-256 is
`9f2b36ab8662f241d748c0d3b5e200f6a28a88f11126c8df0088cd196c1203cc`;
stdout and stderr/timing/group-result SHA-256 values are
`be19d3efa3d902d029996d67af4dc3e9ea1954af80be49996f3d0fce6766f7e4`
and
`cb85151077dd949441bcfb3bcb6f8dffb9a55a9ae572a11d753b99346165fee7`.
The prior candidate manifest remains
`730ac17e8e184a5cb6e78005178c5f685b9dc8c1fe3ad70edc28b7de82f58508`.
The historical failed link, prior review/remediation attempts, three H02a1
exclusions, and CI admission failure below remain failures or separately
owned evidence. No unfiltered 1,742-case or historical 550-case pass is claimed.

H02c is closed with no current blocker. Codegen review suggestions and the
H02a1/CI follow-ups below remain separate work. The approved intermediate-item
policy defers the full merge profile to Q01; no broad gate, whole H02 closure
or release qualification is claimed. This session did not start H02d.

The documentation-only receipt passed documentation structure (SHA-256
`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`),
the 900-line source file-size guardrail for 4,285 files (SHA-256
`70059422f26550d91a3f601e32168bd70ffabd2006d6baf690cc05e9c8a2ddd5`),
and the diff check. Their inputs were unchanged by this evidence paragraph.
Raw documentation checks are under the retry evidence directory's `record/`
child. No repeated Cargo gate or external review was required.

### H02d0/H02d1 formal scope split receipt (2026-09-30)

The documentation-only scope split merged in
[PR #4128](https://github.com/sifr-lang/sifr/pull/4128) as
`a5ca8832a685e85c0ece1e6e27f08facbffe0930` from exact candidate
`e619507551a48cdf7b4e2f69341de7a89244633b` (base
`61cfcb15ec1d52699d5d34f3c6af052d9ea1857f`, tree
`2907da5277a88c51e12283ed8393012e396f461d`).
It formalizes H02d0 as one atomic runtime/generated-caller lifetime
prerequisite and H02d1 as the following runtime-only CPython/core audit.
H02e-H02g wait for merged H02d1; H02h waits for all ten bounded items.
The rows specify package ownership, exact Python-enabled runtime and
codegen caller checks, nonzero selections, scoped regressions,
forget/escape evidence and cancelled-closer authority handoff.

Documentation structure passed (raw log SHA-256
`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`),
the 900-line source guardrail passed for 4,285 files (SHA-256
`70059422f26550d91a3f601e32168bd70ffabd2006d6baf690cc05e9c8a2ddd5`),
and `git diff --check` passed. Raw logs and the candidate-keyed
validation manifest are outside the Git tree at
`/data/sifr-architecture-h02d-formal-split-evidence-20260930/`.
The pinned editor submodule and its nested VS Code input were initialized
before validation; their declared pins were unchanged. The execution host
had no `gh`, so a session-owned local bare repository relayed the exact
commit bundle; the published head matched the tested candidate.

This record changes only the canonical phase document. It claims no
runtime implementation, prospective acceptance pass, external review,
Cargo gate, H02d0/H02d1 completion or whole H02 closure. The original
H02d needs-new-scope findings and failed setup evidence below remain
historical evidence. Documentation checks also cover this record-only
receipt; source guardrail inputs remain unchanged.

The session owns worktree
`/data/sifr-architecture-h02d-formal-split-20260930`, split branch
`codex/architecture-h02d-formal-split-20260930` and record branch
`codex/architecture-h02d-formal-split-record-20260930`.
The formalization has no current blocker. The exact next action is a
separately assigned H02d0 callback runtime/generated-caller lifetime
prerequisite under its formal row. This session stops before implementation.

### H02d0 callback runtime/generated-caller contract receipt (2026-09-30)

[PR #4130](https://github.com/sifr-lang/sifr/pull/4130) merged as
`cf01ec278c0f5d571d61112cb3883c1a0d40f37c` from exact candidate
`2b57e785990e19e06939293360a935af376350c5` (base
`f685703bcfabf0da727c1b9ff221977370bb1015`, tree
`b8da1f7737168ca3ae94a90faac1d4862612c21b`). The execution host lacked
`gh`; a session-owned local clone relayed exact commit bundles and verified
candidate SHA/tree before publishing. Runtime and matching generated callers
merge together as the formal atomic H02d0 prerequisite.

Owned constructors require `'static` values/captures and remain safe. Borrowed
current/foreign/asyncio construction is explicitly unsafe with a private
compiler-local scope contract; generated construction establishes that scope,
and no safe borrowed facade remains. Current/foreign teardown drains admitted
synchronous use. Asyncio teardown closes/drains setup admission and revokes
pending worker futures and queued outputs under mutex exclusion before captures
expire, keeping each invocation lease through borrowed-state destruction.
Drop requests Python entry cancellation and schedules the remaining owned
owner drain if caller cancellation skips the close statement. Retained target
ownership changes only after successful owner publication. Provisional rollback
is restricted to owned `'static` wrappers and suppresses Drop only after terminal
cleanup. Cancelled owner closers relinquish election while admission stays
closed, allowing a successor to finish exact-once capture release.

The final candidate passed **71/71 selected assertion-case executions**
(**69 unique resolved names**), with each new/existing exact case resolving and
executing 1/1. Runtime selections all used `--features python`; a disabled or
empty selection was never counted. The four H02 runtime named cases passed.
Focused runtime groups passed `callbacks::tests` 25/25, `asyncio_tests` 11/11,
`current_tests` 1/1, `ownership_tests` 1/1 and the affected `foreign::tests` 7/7.
The three required exact codegen caller cases passed 1/1 each, the codegen asyncio
group passed 15/15, and all four exact retained-callback caller cases passed 1/1.
The compile-fail case builds the actual candidate library API, rejects safe
borrowed construction/forgetting and unauthorised unsafe scoped construction,
and rejects provisional rollback on a borrowed wrapper. The cancellation case
revokes an active borrowed future and a queued borrowed output after a forgotten
worker-slot handle, checks destructor/lease ordering, and observes owned drain
after dropping without polling a closer. Together with the cancelled-closer
case, it proves handoff, no remaining borrowed polling and exact-once release.
The added owned-rollback cancellation regression passes inside the 11-case
asyncio selection. The current caller assertion ties its scoped factory to the
emitted unsafe block.

Resolved names, commands, executed case counts, timings and raw-log SHA-256
values are outside the Git tree at
`/data/sifr-architecture-h02d0-evidence-20260930/2b57e785990e19e06939293360a935af376350c5/validation-20260930T001429/`.
The selection manifest SHA-256 is `d05acb50d038a157796f67eeb8311849eb8dc32cbb4375bc47b80d60557b03c5`.
Formatting, HIR maintainability, diff checks and the 900-line source guardrail
passed (4,288 files; the largest touched codegen test file is 894 lines).
Foreign implementation/tests were split by responsibility to retain that limit.
Validation used Rust 1.98.1 and existing canonical CPython 3.14.7. The inactive
H02c target was exclusively adopted after toolchain/configuration/lock/handle
checks; no target was cleaned, and no cold-build timing is performance evidence.

The initial completed Opus 5.5 review of candidate
`2ddb6e13b26e57fde7300f0bb808e1614329da2f` returned **NOT SATISFIED**:
borrowed wrappers could use safe provisional rollback, set `retained` before
await and then skip revocation on cancellation (response SHA-256
`3d1f063cf4e8b848b00a1d6a0df9f829148a05a4b681a26d2a58cf9acc01e5c7`).
The single remediation batch restricted rollback to `'static`, deferred that
flag until terminal cleanup, and added the rejection/cancellation regressions.
The final scoped Opus 5.5 remediation review on the exact candidate returned
**SATISFIED**, no blocking findings (response SHA-256 `28aa8583844f550d2c9662e41da262036f48fc60d8f1f61508400323ac2ab291`).
Its response is published outside the reviewed tree, keyed by candidate SHA.
Two earlier CLI requests used an invalid decimal API model spelling and produced
no accepted review; the valid reviewer identifier was `claude-opus-5-5`.

Historical failures remain under
`/data/sifr-architecture-h02d0-evidence-20260930/historical/`: the host default
Python lacked its development link, the pinned codegen corpus was initially
uninitialized, and initial candidate `5cfd7df47` ran the asyncio group 9/10
because its retained-drop test assumed Python cancellation implied immediate
spawned-closer completion. The failed exact case also passed unchanged; the
repaired regression passively observes bounded actual completion without
creating another closer. The overwritten initial suite log is preserved as an
excerpt restored from recorded command output, not reclassified as a pass.
The configured existing interpreter and pinned corpus resolved setup failures
without dependency changes.

The automatic `local-first-create-pr` job on PR #4130 candidate `2ddb6e13b`
(run `36648032661`) stopped before assertions
at V01 `performance_reference_admission`: no `SIFR_PERFORMANCE_REFERENCE`
was selected. This remains an out-of-scope CI prerequisite failure, not H02d0
acceptance or a passing profile. Its raw log SHA-256 is
`45a427fb8b6f00855bfbaf61230771918249d7e2b5dfa4681fe12fd8d670f581`.
The user-authorized intermediate policy requires named/focused tests and scoped
review; create-PR/full merge gates were not run locally. Q01 retains final
integration qualification. Record-only updates need documentation checks only.

H02d1's remaining callback audit owns the review's pre-existing mixed
foreign/asyncio cancellation finding: call-lifetime callbacks can share one
owner, and foreign Drop synchronously waits for Python async entries whose
supervisor must run on the same current-thread executor. A cancelled mixed
wrapper can therefore hang. Do not reinterpret this as a new H02d0 regression
or patch it in this session; H02d1 must adjudicate it and stop for a new atomic
scope if a generated-caller contract change is required. Matching the compile-
fail test's `rustc` to Cargo's selected toolchain and anchoring individual
borrow diagnostics remain nonblocking test-infrastructure suggestions. The
callback/codegen maintainability owners retain suggestions to document both
meanings of `retained` and match the unsafe construction through parsed `syn`
rather than a string split.

This session owns worktree
`/data/sifr-architecture-h02d0-callback-contract-20260930`, implementation branch
`codex/architecture-h02d0-callback-contract-20260930`, record branch
`codex/architecture-h02d0-record-20260930`, its adopted target and temporary paths.
H02d0 has no current blocker. H02d1, H02e-H02h and whole H02 remain open.
The exact next action is a separately assigned H02d1 runtime-core/remaining
callback audit against this merged contract. This session stops before that
batch; no whole-phase closure is claimed.


### H02d1 CPython core and remaining callback audit receipt (2026-09-30)

[PR #4132](https://github.com/sifr-lang/sifr/pull/4132) merged as
`0989439da10fde45453c2c5cca9cf299dc732c80` from exact tested and
reviewed candidate `273d9db7647358b808a06951d7c315a4f289d8b8` (base
`b5df9ffa4da8a75f79e1789aa19816cdb53e400c`, tree
`7a1332866dee4b28945426cce8fd7368eb3b0af7`). The execution host lacked
`gh`; this session's local clone relayed the exact commit bundle and verified
SHA/tree before publication. Only runtime Python/core/callback sources and the
architecture description changed; the merged H02d0 generated-caller contract
and the separate Buffer, Arrow, DLPack, bridge and X02 owners are preserved.

CPython configuration now has one cohesive ABI module with thread, lifetime,
alias and allocation ownership contracts. The outer module's broad unsafe
allowance is removed; callback allowances belong to individual functions and
marker implementations. Separate resource ABI owners retain their explicit
module allowances for their following audits. Invalid NUL-bearing configuration
is rejected before pinning the process environment. A later verification failure
keeps `initialized=false`, rejects attachment and replacement environments, and
permits the identical selected attempt to retry. CPython stays initialized for
the process lifetime; PyO3 attachment/detachment owns foreign-thread entry.
`ForeignObject` derives `Send`/`Sync` through its owned Py reference and mutex;
public semantic close claims its existing close lease before invoking Python,
rejects new alias access/duplicate cleanup and keeps admitted leases pinned.
Detached final release moves the sole tracked reference into the pending queue;
attached draining releases it exactly once.

The recorded mixed foreign/asyncio cancellation hang is repaired entirely in
runtime. Owner admission tracks synchronous setup, decoding, handler execution
and encoding separately from registered async entries. Synchronous wrapper
teardown closes shared admission and drains those synchronous uses with the GIL
released, while the remaining owned async leases finish shared-owner teardown.
It does not block the current-thread executor that must run the supervisor.
`ForeignCallback::close_call_scope` and `CurrentCallback::close` can therefore
return after their local target is safe while a mixed shared owner remains
`Closing`; the last lease or an explicit async closer completes `Closed` and
exact-once shared capture release. Capture-release reentrant close returns
`SifrCallbackCloseReentrancyError`. If attachment itself fails during public
semantic cleanup, the identity is poisoned rather than left open; this is the
intended fail-closed cleanup behavior noted by review.

The exact candidate passed **69/69 selected assertion-case executions**
(**69 unique resolved names**). The two exact H02d1 cases
`python::h02_contract_tests::initialization_and_gil_entry_contract` and
`python::h02_contract_tests::foreign_object_thread_lifetime_and_release_contract`
each resolved and executed 1/1. All four exact H02d0 runtime cases each passed
1/1. Focused selections passed `python::tests` 14/14,
`python::config_verify::tests` 2/2, `callbacks::tests` 25/25,
`callbacks::asyncio_tests` 11/11, `callbacks::current_tests` 1/1,
`callbacks::ownership_tests` 1/1, the affected `callbacks::foreign::tests` 7/7,
and `callbacks::h02_core_tests` 2/2. Every command uses `-p sifr_runtime`,
`--features python` and `--lib`; each exact command uses `-- --exact`, and
all focused groups resolve nonzero test names. The mixed cancellation regression
runs with a child-process wall deadline, both wrapper drop orders, a borrowed
pending future, no subsequent poll, escaped-shell rejection and exact-once
future/capture release. The concurrent semantic-close regression blocks the
first Python cleanup with a GIL-releasing event, rejects duplicate cleanup and
new alias access, then observes one Python cleanup call.

Resolved names, commands, case counts, timings, input digests and raw-log hashes
are outside the Git tree at
`/data/sifr-architecture-h02d1-evidence-20260930/273d9db7647358b808a06951d7c315a4f289d8b8/validation-20260930T005207/`.
The selection manifest SHA-256 is
`3925949944c79b0fb58234bd74f4d5c78ba33b82c82728a105c7ccd80bcedf46`.
Rust/Cargo 1.98.1 and the canonical existing CPython 3.14.7 were pinned;
the linked-library receipt resolves the selected `libpython3.14.so.1.0`, whose
SHA-256 is `6e2c606e547bd30b42062269747658ab009a181c6fd8541b311c5562bfbce8a3`.
The inactive H02d0 target was exclusively adopted after toolchain/configuration,
lockfile/process/handle and free-space checks; `/data` retained about 12 GiB
free for the focused incremental operation. No target was cleaned and no
cold-cache timing is performance evidence. Formatting, HIR maintainability,
committed diff and the 900-line source guardrail passed (4,291 files).
Documentation structure passed after initializing the pinned nested editor
submodule (log SHA-256
`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`).

Scoped read-only Claude Opus 5.5 review on the exact candidate returned
**SATISFIED** with no blocking findings (response SHA-256
`f1f26ec9580266c5febe0cab7b9423a4fb246cdef367710587ed8ef5c00d5083`).
The response is published outside the reviewed Git tree, keyed by candidate
SHA at `review/opus-5-5.md`. Its suggested documentation of local-close versus
shared-owner completion and attachment-failure poisoning is recorded above.
Workspace Clippy/full-profile qualification remains with Q01 under this phase's
approved intermediate-item policy; it is not claimed by these focused results.

Historical setup and fixture failures remain under the evidence root's
`historical/`: the first Cargo invocation lacked the pinned Ruff submodule;
an intermediate test fixture attempted to return a Python `Bound` across the
attachment token and failed compilation before assertions. Both were corrected
and the final selections passed; their saved tool-output excerpts are identified
as excerpts, not original redirected raw logs. A failed shell-edit parse and the
initial relay ref-selection error are retained there too. The review request
completed successfully; its orphaned watchdog sleep was terminated after the
atomic response publication to allow the SSH completion notification.

PR #4132's automatic `local-first-create-pr` job (run `36652507055`) stopped
before assertions at V01 `performance_reference_admission`: no
`SIFR_PERFORMANCE_REFERENCE` was selected. This is a preserved out-of-scope CI
prerequisite failure, not a passing profile or H02d1 acceptance. Raw log SHA-256
is `7895dae9d116f9c0bee3471b05c33a21bf2bfc230d625af21f883da27700a85e`.
The user-authorized intermediate policy requires named/focused tests and scoped
review; create-PR/full merge gates were not run locally. Q01 still owns final
integration qualification. This record-only update requires documentation checks
only, with no repeat implementation validation or external review.

This session owns worktree
`/data/sifr-architecture-h02d1-python-core-20260930`, implementation branch
`codex/architecture-h02d1-python-core`, record branch
`codex/architecture-h02d1-record-20260930`, its adopted target and temporary
paths. H02d1 has no current blocker. H02e-H02h and whole H02 remain open.
The exact next action is separately assigned H02e; this session stops here.

### H02e Python buffer address and release contract receipt (2026-09-30)

[PR #4134](https://github.com/sifr-lang/sifr/pull/4134) merged as
`61a6002e133577413ff73a12f680af9284ac0649` from exact tested and
reviewed candidate `db630987da8dc99e33808a67b673f39519f1a107` (base
`2e73a99c1755bcbd9cacfd11eef2ac4442d906c5`, tree
`5a1c278e309194a47257d4ae504fd244530db590`). The execution host lacked
`gh`; the session's separate local clone relayed the exact commit bundle and
verified SHA/tree before publication. Only the buffer owner and its parent
module's buffer-specific unsafe allowance changed. Arrow, DLPack, CPython
core/callbacks, bridge modules and H02h integration retain their own scope.

The buffer owner now checks metadata-vector alignment and address extents before
forming slices. Checked stride segments and signed offsets replace unchecked
`PyBuffer_GetPointer` traversal; indirect slots require non-null addresses,
alignment and valid numeric extents before their pointer read. Null row targets,
stride/address overflow and malformed shape/length layouts reject before element
access. The export pins an admitted logical-address snapshot, so later reads and
writes access exactly the allocations covered by storage-alias admission and do
not re-follow changed foreign metadata or indirect tables. Shared readers remain
valid; overlapping reader/writer and writer/writer admissions reject. Existing
negative strides, exact indirect/disjoint ranges and unaligned primitive element
access remain supported. Dense C/Fortran views retain compact address records;
Fortran elements iterate in logical C order.

The broad parent buffer-module unsafe allowance is removed. Cohesive pinned
CPython-export and checked PEP 3118 address modules own their ABI allowances;
primitive access helpers own function allowances. Local contracts state GIL,
mutex, export lifetime, alias and allocation ownership obligations, including
`Send` without `Sync`. Numeric checks cannot prove an arbitrary native
allocation's provenance: exporters must uphold CPython's allocation/lifetime
contract. No Rust references are formed to foreign element storage. The strong
export reference survives semantic foreign-handle close and releases exactly
once on explicit release or resource drop, including foreign-thread drop.

The exact candidate passed **33/33 selected assertion-case executions**
(**33 unique resolved names**). Both exact H02e acceptance cases
`python::buffer_ops::h02_contract_tests::layout_bounds_and_alias_admission` and
`python::buffer_ops::h02_contract_tests::all_release_paths_are_exact_once` each
resolved and executed 1/1. Focused selections passed
`python::buffer_ops::tests` 18/18,
`python::buffer_ops::release_evidence_tests` 5/5,
`python::buffer_ops::typed_access_evidence_tests` 1/1, and the affected
`python::buffer_ops::raw::tests` 7/7. Every command uses `-p sifr_runtime`,
`--features python` and `--lib`; each exact command uses `-- --exact`, and
all focused groups resolve nonzero names. The new cases include malformed
metadata/pointer alignment, null indirect rows, stride/address overflow, negative
strides, bounds, shared-storage admission, changed indirect tables, Fortran order,
unaligned elements, acquisition/validation/admission/storage failures, exporter
lifetime, explicit release, duplicate release, automatic drop and snapshot drop.
A failed acquisition transfers no export and invokes no release callback;
every successful acquisition rejection or completed live export releases once.

Resolved names, commands, case counts, timings, input digests and raw-log hashes
are outside the Git tree at
`/data/sifr-architecture-h02e-evidence-20260930/db630987da8dc99e33808a67b673f39519f1a107/validation/`.
The selection manifest SHA-256 is
`b46e5b834dc60390cd733a3623599524ef1fec9b7ddf180b8daa76300e0cb334`.
Rust/Cargo 1.98.1 and existing CPython 3.14.7 were pinned. The selected
`libpython3.14.so.1.0` SHA-256 is
`6e2c606e547bd30b42062269747658ab009a181c6fd8541b311c5562bfbce8a3`.
The inactive H02d1 target was exclusively adopted after matching
lockfile/configuration/toolchain and checking no Cargo/rustc processes or open
handles. It moved into this session's worktree; about 11 GiB remained free for
the focused incremental operation. No target was cleaned and no timing is
claimed as performance qualification. Formatting, HIR maintainability, committed
diff and the 900-line source guardrail passed (4,293 files). Documentation
structure passed after initializing the pinned nested editor submodule (log
SHA-256 `d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`).

Scoped read-only Claude Opus 5.5 review on the exact candidate returned
**SATISFIED** with no blocking findings (response SHA-256
`6b5111285688b012beb8ea43e9e28909e2c2d9019ce5a4ecb587ec14eb47bb1d`).
The response is published outside the reviewed tree, keyed by candidate SHA at
`review/opus-5-5.md`. Optional follow-ups are reserved for separately assigned
buffer work: compact persistent addresses for very large noncontiguous exports,
and additional programmer-invariant assertions linking dense offsets to their
validated extent. Neither changes H02e acceptance or starts another item.

Historical setup evidence stays under the external evidence root's `historical/`:
the first documentation check lacked the nested editor submodule and failed
before assertions; initialization corrected it and the final check passed.
The initial commit-bundle relay selected a nonexistent named ref; importing its
actual `HEAD` ref then verified the exact SHA/tree. Those shell/tool-output
excerpts are explicitly excerpts, separate from redirected raw logs. Earlier
successful intermediate layout/release runs are retained but do not substitute
for the final exact-candidate selections above.

PR #4134's automatic `local-first-create-pr` job (run `36654523284`, job
`109695857124`) failed before assertions at V01
`performance_reference_admission`: no `SIFR_PERFORMANCE_REFERENCE` was selected.
This preserved out-of-scope CI prerequisite failure is not a passing profile or
H02e acceptance failure; V01/Q01 retains qualification ownership. Its raw job
log SHA-256 is
`03fcb854ef83f38cfbca47aa28df022efcc0e0fb30593bc1e6aac9e05ccc4344`.
The PR merged normally with the exact approved head; no admin bypass was used.

The user-authorized intermediate policy required named/focused tests and scoped
review. Create-PR/full merge gates were not run locally; Q01 owns final integration
qualification. This receipt-only update requires documentation checks and no
repeat implementation validation or external review.

This session owns worktree
`/data/sifr-architecture-h02e-buffer-contract-20260930`, implementation branch
`codex/architecture-h02e-buffer-contract-20260930`, record branch
`codex/architecture-h02e-record-20260930`, its adopted target and temporary paths.
H02e has no current blocker. H02f-H02h and whole H02 remain open. The exact next
action is separately assigned H02f; this session stops without starting it.


### H02f Arrow ABI and capsule transfer delivery (2026-09-30)

H02f merged in [PR #4136](https://github.com/sifr-lang/sifr/pull/4136) as
e815a2d20779b1426ad94c9055cb4688d1561510, from exact tested and reviewed
candidate 145cb24d9996400811e2180110463d2efbdcd423 (base
63f98710f8e0e605513702f167f68f79d15c2fe4, tree
7a6fa8ab816c7fc640e8920dcb7c9b74d14e50d6).
Only sifr_runtime::python::arrow_ops and its owned ABI/transfer/test modules
changed. Declaration certification, buffers, callbacks and DLPack remain under
their separate owners.

Acquisition captures immutable capsule payload identities. Transfer admission
and finalization check those identities before ABI inspection; admission also
revalidates names, destructors, required nullable callback fields and live release
state. ABI inspection copies header values without references whose lifetimes
escape the live capsule. Native producers still own the allocation-validity and
protocol-serialization obligation: names/destructors cannot prove an arbitrary
native address safe, and the bridge does not dereference child/buffer pointers.
Negative array metadata rejects excessive null counts and length/offset overflow.
Borrowed metadata observations retain capsules without conferring transfer
authority. Prepared arguments and requested schemas share exclusive payload
admission while their Sifr transfer entries are live. A declined admission closes
the supplied handle and releases its references; it is not a nondestructive
probe. Finalization and drop revoke retained argument shells, and paired
schema/data finalization distinguishes full, partial and unconsumed ownership.
Reset drops capsules outside the store mutex, allowing destructor reentry.

The exact candidate passed **14/14 selected assertion-case executions** with
**14 unique resolved names**. Both reserved cases each resolved and ran 1/1:

- cargo test -p sifr_runtime --features python --lib python::arrow_ops::h02_contract_tests::abi_layout_and_capsule_transfer -- --exact
- cargo test -p sifr_runtime --features python --lib python::arrow_ops::h02_contract_tests::stream_callbacks_release_exactly_once -- --exact

The nine assertion-exercising existing python::arrow_ops::tests cases and
three python::arrow_ops::abi::tests cases also each ran individually with the
same crate/features/library selection and -- --exact. Selection resolved
nonzero names before execution and stopped on the first failing command.
The opt-in real_pyarrow_requested_schema_is_a_one_shot_transfer case was
explicitly excluded because its external producer was not configured; its
conditional no-op is not acceptance and no real-PyArrow qualification is claimed.

New coverage checks every field offset, size and alignment of C Data, Stream,
Device Array and Device Stream headers on the qualified 64-bit host against
Apache Arrow's apache-arrow-22.0.0 cpp/src/arrow/c/abi.h. No 32-bit ABI
qualification is claimed. It exercises overlapping transfer rejection, pointer
identity mutation before admission, revoked retained shells, consumed borrowed
observations, invalid shape/release metadata, device full/partial/drop paths,
stream/schema/chunk callbacks and independent output releases, failure before
nullable last-error observation, every missing required stream callback, and
explicit/consumed/unconsumed/failed-consumer/drop/reset cleanup. Existing focused
cases retain malformed-name/destructor rejection, paired full/partial/failed
consumption, one-shot proxies and requested-schema behavior.

Resolved names, exact commands, case counts, timing, source/configuration digests
and raw-log hashes are outside the Git tree at
/data/sifr-architecture-h02f-evidence-20260930/145cb24d9996400811e2180110463d2efbdcd423/.
Its selection SHA-256 is
f599dabb9dc2a42158d4e6ad10500bf054d3e68d59141b6a2b71bf3994300c5d.
Rust/Cargo 1.98.1 and existing canonical CPython 3.14.7 were pinned; the selected
libpython3.14.so.1.0 SHA-256 is
6e2c606e547bd30b42062269747658ab009a181c6fd8541b311c5562bfbce8a3.
After confirming the H02e target was inactive with no Cargo/rustc process, its
compatible target moved into this session's worktree for exclusive warm reuse.
Lockfile, workspace/runtime manifests and Cargo configuration matched. About
11 GiB was free before the operation and 9.7 GiB afterward; no target was cleaned
and timings are not performance qualification. Formatting, HIR maintainability,
committed diff and the 900-line guardrail passed (4,295 source files).

Scoped read-only Claude Opus 5.5 review returned **SATISFIED** with no blocking
findings on the exact candidate. The external review/opus-5-5.md response has
SHA-256 6fdaae683b0c62d190820af1f9f76d6eb7133873c6bad9ce50a01594f3894a3a.
Its nonblocking follow-ups are preserved for explicit later scope adjudication:
early errors skip the usual pending-release drain even though resources still
drop under attachment; and a pre-existing external callee can retain an
unconsumed exported capsule beyond the Sifr entry's transfer lease, allowing a
later borrowed alias to be admitted. This item proves its live-entry admission
boundary and does not claim to resolve external capsule retention policy.
The review's declined-handle and 64-bit evidence wording suggestions are
incorporated above. No follow-up implementation began.

Historical working/arrow-initial.log (SHA-256
01de91b7f4637e4e83e4c380ab24d1180643ea3f9287324f01d0c63633f7cc28)
preserves a new-test counter expectation failure after two added release
scenarios; the corrected final selection passed separately. The host's first
HTTPS push attempt lacked noninteractive credentials; the authenticated local
connection then pushed the exact branch without changing the candidate.
These are historical failures, not passing validation.

PR #4136's automatic local-first-create-pr job (run 36656380354, job
109701513510) failed before assertions in V01
performance_reference_admission because no SIFR_PERFORMANCE_REFERENCE was
selected. Its external raw job log has SHA-256
fd01f10acc8443e71aba7d1682561cadb0e8a7d16f026abf78a40e7279627bc0.
V01/Q01 retains ownership of this preserved out-of-scope CI prerequisite;
it is not a passing profile or H02f acceptance failure. The PR merged normally
with the exact approved head, without an admin bypass.

The assigned intermediate policy required named/focused tests and scoped review;
create-PR/full merge profiles were not run locally. Q01 owns final integration
qualification. This receipt-only update requires documentation checks and no
repeat implementation validation or external review.

This session owns worktree
/data/sifr-architecture-h02f-arrow-contract-20260930, implementation branch
codex/architecture-h02f-arrow-contract-20260930, record branch
codex/architecture-h02f-record-20260930, its adopted target and temporary paths.
H02f has no current blocker. H02g-H02h and whole H02 remain open. The exact next
action is separately assigned H02g; this session stops without starting it.

### H02g DLPack ABI and deleter transfer delivery (2026-09-30)

H02g merged in [PR #4138](https://github.com/sifr-lang/sifr/pull/4138) as
8bdbd1af98b9274d4d0141149237216031053785, from exact tested and reviewed
candidate e1bff98c99e1286dfe4276f25f4fed6e118066f4 (base
6a88c6ffc4f876b065ed533b67ddc488f8cc1657, tree
f429d408c8a735f364633bfea4deaeeaec4a9618).
The implementation changes only sifr_runtime::python::dlpack_ops and its owned
ABI, argument and test modules, plus removal of its broad unsafe allowance from
python.rs. Declaration certification policy is unchanged.

Capsule acquisition checks the managed-header alignment and transfers the
one-shot capsule to its used name before metadata inspection. The tracked owner
therefore invokes the deleter once on metadata rejection and insertion failure,
even while the exporter retains the capsule. Metadata inspection copies the C
header and checks shape/stride nullness, alignment and addressable extent before
constructing slices, then validates element-count overflow. Incompatible major
versions read only
the version and stable-prefix deleter. Device admission supports the existing
CPU/CUDA families, rejects negative ids and rechecks supplied stream metadata,
including ambiguous CUDA token 0, before calling the producer.

Prepared arguments structurally pin their capsule independently of exposed
ObjectHandles. Closing those handles or resetting semantic runtime state cannot
prevent attached cleanup. The argument capsule owns release until consumption or
reconciliation; reconciliation checks the payload identity and revokes retained
unconsumed shells before releasing. A consumer may release its managed header
before finish: consumed-state reconciliation reads only capsule state, never the
freed header. Reset drops stored entries outside the mutex, and detached drops
attach before foreign callbacks. Reset preserves monotonically advancing handle
identities. Unsafe operations have local thread, lifetime, alias and ownership
contracts and allowances at the cohesive ABI module, specific functions or tests.

Native producers still own live allocation validity, rank-sized initialized
metadata arrays, unique managed export ownership and protocol serialization.
Capsule names and alignment checks cannot prove arbitrary native addresses valid;
the bridge never dereferences tensor data. Header layout expectations use
[DLPack v1.2 dlpack.h](https://github.com/dmlc/dlpack/blob/v1.2/include/dlpack/dlpack.h)
and the [Python capsule protocol](https://dmlc.github.io/dlpack/latest/python_spec.html).
No 32-bit ABI or real external producer qualification is claimed.

The exact candidate passed **21/21 selected assertion-case executions** with
**21 unique resolved names**. Both reserved cases each resolved and ran 1/1:

- cargo test -p sifr_runtime --features python --lib python::dlpack_ops::h02_contract_tests::legacy_versioned_layout_and_transfer -- --exact
- cargo test -p sifr_runtime --features python --lib python::dlpack_ops::h02_contract_tests::rejection_and_reset_release_exactly_once -- --exact

All eleven existing python::dlpack_ops::declaration_tests, four ABI metadata tests
and four existing top-level DLPack tests also ran individually with the same
crate/features/library selection and -- --exact. Selection resolved nonzero
names before execution and stopped on the first failing command. New cases check
every C-header field offset, size and alignment on the qualified 64-bit host,
legacy/versioned transfer, nullable deleters, retained-shell revocation, producer
handle close, consumer release before finish, immediate copied/version/alignment
rejection, invalid stream/device admission, repeated release, reset identity,
stopped-runtime argument drop, and detached reset with attached, reentrant release
outside the store mutex. Existing cases retain signature/no-retry, dtype/device
mismatch, explicit stream, scalar/empty tensor and consumed/unconsumed coverage.

Resolved names, exact commands, case counts, timings, source/configuration digests
and raw-log hashes are outside the Git tree at
/data/sifr-architecture-h02g-evidence-20260930/e1bff98c99e1286dfe4276f25f4fed6e118066f4/.
Its selection SHA-256 is
b5846ec8cda63d1cd474ac1153eed949b0b06fc6833cdc9e876832615a1a1063.
Rust/Cargo 1.98.1 and existing canonical CPython 3.14.7 were pinned; the selected
libpython3.14.so.1.0 SHA-256 is
6e2c606e547bd30b42062269747658ab009a181c6fd8541b311c5562bfbce8a3.
After confirming no Cargo/rustc process was using the inactive H02f target, its
compatible target moved into this session's worktree for exclusive warm reuse.
Lockfile, workspace/runtime manifests and Cargo configuration matched. About
9.1 GiB was free before the operation and 8.8 GiB after validation; no target was
cleaned and these timings are not performance qualification. Formatting, HIR
maintainability, committed diff and the 900-line guardrail passed (4,296 files).

Scoped read-only Claude Opus 5.5 review returned **SATISFIED** with no blocking
findings on the exact candidate, published in the
[PR review receipt](https://github.com/sifr-lang/sifr/pull/4138#issuecomment-5902704124).
The external review/opus-5-5.md response has SHA-256
f567fa88d5e429042caffe3892b78189ad80303784e0848aac2ceddc78cc185f.
Deferred follow-up work remains separate from H02g acceptance: explicit unsafe
signatures for internal raw-pointer helpers are a policy suggestion for H02h;
CUDA stream -1 support is a future feature; absent/finalizing interpreter teardown
belongs to the CPython-core owner; and unknown consumer-name no-release coverage
is a test suggestion. No follow-up implementation began or new requirement was
added. Unavailable CPython and protocol-invalid consumer names cannot safely
confer foreign deleter authority; their teardown policies are not qualified here.

Historical working/initial-transfer.log (SHA-256 e50f34c926c9933dcc83b3f2fe30490a12b659808fdb860964a554e7fe0289bf)
preserves a setup failure before assertions because the new worktree lacked the
pinned Ruff submodule. Preparing the pinned Ruff/editor submodules corrected the
setup; the final selection passed separately. This failure is not passing evidence.

PR #4138's automatic local-first-create-pr job (run 36658330863, job
109707348319) failed before assertions in V01 performance reference admission
because no SIFR_PERFORMANCE_REFERENCE was selected. Its external raw job log has
SHA-256 91e28a61b47c3ffde5f52423a6ae8a306352e2a0a5273bf0d205b55be88fb73a.
V01/Q01 retains ownership of this preserved out-of-scope CI prerequisite; it is
not a passing profile or an H02g acceptance failure. The PR merged normally with
the exact approved head, without an admin bypass.

The assigned intermediate policy required named/focused tests and scoped review;
create-PR/full merge profiles were not run locally. Q01 owns final integration
qualification. This receipt-only update requires documentation checks and no
repeat implementation validation or external review.

This session owns worktree
/data/sifr-architecture-h02g-dlpack-contract-20260930, implementation branch
codex/architecture-h02g-dlpack-contract-20260930, record branch
codex/architecture-h02g-record-20260930, its adopted target and temporary paths.
H02g has no current blocker. H02h and whole H02 remain open. The exact next action
is separately assigned H02h; this session stops without starting it.


### H02d callback lifetime boundary: needs-new-scope (2026-09-30)

The H02d audit stopped **needs-new-scope** on clean main
`eeb61f1c1710e2b587c46fbc47232076a5ac8544`. No runtime implementation,
named acceptance pass, scoped implementation review, full gate or H02d
closure is claimed. H02e-H02g remain blocked on H02d; this session did
not start another batch.

The assigned runtime-only boundary cannot make the public borrowed callback
APIs sound without changing their generated callers. Concrete findings:

- `callbacks/current.rs:97-154` and `callbacks/foreign.rs:242-292`
  safely construct shells containing lifetime-erased pointers to targets that
  may borrow caller locals. `object()` exposes an independently cloneable
  handle. Safe `mem::forget(callback)` lets the local borrow end while the
  leaked box and escaped Python shell survive; later shell invocation reads
  an expired capture. Drop-based draining and leaking do not prove the
  borrowed capture lifetime.
- `callbacks/asyncio.rs:301-321,352,391-396` additionally lets ordinary
  early drop leak the borrowed target when an invocation is active, without
  cancelling or joining the worker. The erased future still polls after the
  caller may destroy its borrowed locals. A leaked target preserves its own
  allocation, not the data it borrows. The setup admission lease also needs
  a lifetime proof across concurrent drop and asynchronous teardown.
- `callbacks/state.rs:532-582` elects a closer by setting `Closing` before
  awaiting invocation drain. Cancelling that close future abandons the sole
  closer; subsequent closers wait for `Closed` indefinitely. Cancellation
  must transfer or finish close authority and release captures exactly once.
- `sifr_codegen/src/python_interop_callbacks.rs:77-87,122,450-470`
  emits the safe borrowed constructors and separate normal-path close calls.
  Making the existing constructors unsafe or replacing them with a structural
  scope changes this package contract; changing only runtime annotations
  would break generated Rust, and preserving a safe unsound facade is not
  acceptable.

These are source-audit findings, not executed undefined-behavior reproducers.
The exact source locations and base SHA are the evidence boundary. A bounded
replacement assignment should be **H02d0**, one atomic runtime/codegen
lifetime prerequisite: establish a safe structural scope that survives
forgetting exposed handles and drains on cancellation, or a narrow unsafe
borrowed-construction contract used only by compiler-generated scope glue
with proved teardown. This is an explicit atomic contract handoff across
package owners: the runtime API and generated constructor/cleanup callers
must merge in one independently compiling prerequisite. An intermediate
runtime-only unsafe API breaks callers; retaining the old safe borrowed
API until a later codegen merge preserves the unsound state. The codegen
portion is confined to this handoff, with runtime and codegen owning their
respective changed paths. Keep owned `static` callbacks safe; do not add a
compatibility facade. Include only callback runtime APIs, their generated
construction/cleanup, and focused callback tests. Prove normal return,
conversion/setup error, escaped shell, early drop, cancellation, reentrant
close and exact-once release. Do not treat comments or a leaked allocation
as a capture-lifetime proof. Reserve exact cases
`python::callbacks::h02_contract_tests::borrowed_callback_lifetime_and_thread_owner`
and
`python::callbacks::h02_contract_tests::close_cancel_reentrancy_releases_once`,
plus `python_interop_async_tests` and callback cases within
`python_interop_direct_tests` in `sifr_codegen`; resolve each existing
selection and assertion count before
claiming acceptance. The caller checks are exactly
`cargo test -p sifr_codegen --lib python_interop_direct_tests::typed_current_callback_emits_checked_adapter_failure_reconciliation_and_cleanup -- --exact`,
`cargo test -p sifr_codegen --lib python_interop_async_tests::asyncio_callback_emits_owned_loop_factory_async_handler_and_async_drain -- --exact`,
and
`cargo test -p sifr_codegen --lib python_interop_async_tests::foreign_callback_in_async_wrapper_uses_nonblocking_drain -- --exact`.
Add a compile-fail or structurally safe forget/escape
regression and a cancelled-closer handoff regression.

After that independently compiling prerequisite is merged, **H02d1** can
finish the original runtime-only initialization/GIL/foreign-object audit,
local unsafe thread/lifetime/alias/ownership contracts and allowance
narrowing. Buffer/Arrow/DLPack remain H02e-H02g; no bridge module or
unrelated generated-Rust safety change belongs to the prerequisite.
These proposed items are a handoff, not implementation acceptance.

The original H02d Cargo commands omit `--features python`, while
`sifr_runtime/src/lib.rs:25-26` gates the entire Python module on that
feature. Replacement acceptance must use
`cargo test -p sifr_runtime --features python --lib <exact-case> -- --exact`
and the same feature for the four focused callback selections. Require
nonzero resolved tests; a zero-selection invocation cannot qualify H02d.

The session-owned branch is
`codex/architecture-h02d-python-core-20260930`, worktree
`/data/sifr-architecture-h02d-python-core-20260930`. `/data` had 17 GiB
free at audit. The offered H02c target was inspected read-only; its available
runtime fixture fingerprints had no Python feature, so they supply no
callback acceptance evidence. No Cargo target ownership transfer, cleanup,
new compiler build or modification of another worktree occurred. The next
action is a separately assigned H02d0 lifetime prerequisite; H02d remains
open until its runtime and caller contracts are repaired and qualified.

The documentation-only record passed documentation structure (SHA-256
`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`),
the 900-line source guardrail for 4,285 files (SHA-256
`70059422f26550d91a3f601e32168bd70ffabd2006d6baf690cc05e9c8a2ddd5`),
and the diff check. Raw logs are outside the Git tree at
`/home/yaser5/projects/sifr/architecture-h02d-python-core-evidence-20260930`.
The initial documentation failure from the missing pinned nested editor
submodule is preserved (SHA-256
`4fe629d30459e0269ec1651cb2c35878fedfec3021d44dcde38ae0ae3fa0c26a`);
initializing the declared gitlinks restored the input, without changing
those pins. These checks do not qualify a runtime repair. No external
review or broad Cargo gate was required for this record-only update.

### H02c codegen method authority resource blocker (2026-09-30)

The prior continuation stopped **blocked by host storage**, with draft
[PR #4124](https://github.com/sifr-lang/sifr/pull/4124) at exact candidate
`9a7be02e543d7c386cf36acce75be979eb936205` (base
`42fca77df7538cd04b3dde12d70b212c771b992c`, tree
`7a1c97ff4fc4d817402280bebbe9a709edaff6f2`). At that checkpoint no H02c implementation
merge, E2E acceptance or full gate was claimed. H02d was not started.
The delivery receipt above supersedes this historical blocker.

The continuation owns branch `codex/architecture-h02c-resume-20260929`
and worktree `/data/sifr-architecture-h02c-resume-20260929`.
The inactive prior source worktree
`/data/sifr-architecture-h02c-method-authority-20260929` remains untouched,
including its two pending in-scope source edits. Their patch provenance was
preserved before adopting the fixture-carrier repair and removing the new
place-emitter fallback in the owned continuation. The first recovered scoped
Opus review of candidate `279b421d71c5fb5d8839991fa96b17fafaf44432`
was **NOT SATISFIED**: the ordinary setdefault unit test was unclassified and
new fallback expressions swallowed nested structural errors. The continuation
repaired both, added nested receiver/argument structural-error regression
coverage, and repaired two newly exposed indexed-list length regressions.
Historical failed and incomplete test/preparation attempts remain preserved.

On the final candidate, all three required exact
`method_authority_tests` cases passed 1/1 each; the additional
`nested_builtin_declines_preserve_structural_errors` case and exact
`lower_stmt::candidate_and_validation::tests::local_binding_setdefault_materializes_owned_key_and_default`
case also passed 1/1 each. Focused `methods::tests` passed 12/12 and
`lib_codegen_tests::emitted_rust_quality_codegen_tests` passed 33/33.
The selected `sifr_codegen --lib` run passed **1,739/1,739**, with exactly
three independently reproduced base failures excluded from 1,742 cases.
Formatting, HIR maintainability, diff check and the 900-line source guardrail
(4,285 files) passed. This does not claim an unfiltered 1,742-case or the
original worker's unfiltered 550-case pass.

Final read-only Opus 5.5 review returned **SATISFIED**, with no blocking
findings, on this exact candidate. Response SHA-256:
`c7370218583843fd70be94b349a302796f1f41b9fe0c88699fc8c4d7bd9628f0`.
It did not treat the concurrently pending E2E run as passing.
Codegen-owner suggestions remain separate follow-up work: avoid duplicate
operand lowering on special contextual paths, preserve early classification
diagnostics consistently, and add a well-formed place-decline regression
alongside the existing wrong-arity decline case. These are not new H02c
acceptance requirements.

Final evidence is outside the reviewed tree under
`/data/sifr-architecture-h02c-resume-evidence-20260929/9a7be02e543d7c386cf36acce75be979eb936205/`.
Its `validation.json` records the exact commands, configuration, inputs,
results and log hashes (SHA-256
`730ac17e8e184a5cb6e78005178c5f685b9dc8c1fe3ad70edc28b7de82f58508`).
The required command was:

```bash
CARGO_TARGET_DIR=/data/sifr-architecture-h02c-method-authority-20260929/target \
CARGO_BUILD_JOBS=2 \
verification/runner/e2e/run_e2e_pass.sh --profile create-pr \
  --fixture-manifest verification/areas/core_language/data/h02_method_e2e_manifest.json \
  --cache-dir /data/sifr-architecture-h02c-method-authority-20260929/target/sifr_e2e_cache/create-pr
```

That run exited 101 while building/linking the compiler: `cc` exited 1,
and `/data` simultaneously reported zero available bytes. The log ends
during the linker command and contains no complete trailing filesystem
diagnostic. Neither selected fixture executed. Preserve the exact incomplete
`e2e.log` (SHA-256
`6203cd824d616bc4765583bd75b06c907e593058287f52199bdeda51649ce9f8`);
the older candidate's 2/2 E2E result cannot qualify this changed candidate.

The parent explicitly transferred exclusive ownership of the inactive prior
Cargo target at
`/data/sifr-architecture-h02c-method-authority-20260929/target`.
No active prior compiler process or open target lock/handle was found;
rustc 1.98.1, Linux target, empty rustflags and Cargo configuration were
compatible. Source and Git index stayed in the owned continuation. Under
actual disk pressure, only obsolete completed incremental snapshots in this
adopted target were pruned after handle/lock checks, retaining current
snapshots. After the failed link stopped, its inactive 1,299,873,792-byte
temporary executable
`target/debug/deps/sifr-006e8ddf4da3b465.tmp93269e7` was removed solely to
make this documentation checkpoint possible. The resulting 999 MiB reserve
is insufficient to retry linking. Cleanup decisions and provenance are in
`pressure-cleanup.log`; no other session's target was touched.

**Historical next action (discharged above):** the parent coordinates sequential cleanup with completed
phase owners, then dispatches a fresh H02c retry worker. Verify target
exclusivity, candidate, configuration and validation inputs; reuse unchanged
passing evidence and warm artifacts, then rerun the exact required E2E
selection only after sufficient build/link reserve exists. Merge #4124 only
after required validation succeeds. This continuation stops at the resource
blocker under the repository's external-blocker rule.

**H02a1 lowering-owner follow-up (observed during H02c):** three optional
length cases fail in unchanged lowering on H02c base
`42fca77df7538cd04b3dde12d70b212c771b992c`, before codegen.
They remain excluded explicitly from the selected codegen run:

- `lib_codegen_tests::checked_place_read_codegen_tests::nested_list_string_compare_borrows_row_without_temporary_clone`:
  unsupported `len` on `None | list[str]`.
- `lib_codegen_tests::structured_path_codegen_tests::recursive_nested_checked_reads_use_the_injected_capture_binding`:
  unsupported `len` on `None | list[int]`.
- `lib_codegen_tests::union_representation_codegen_tests::optional_string_length_keeps_a_payload_compatible_callback`:
  unsupported `len` on `None | str`.

Base reproductions are
`/data/sifr-architecture-h02c-method-authority-evidence-20260929/base-42fca77df/optional-len-{1,2,3}.log`,
with SHA-256 respectively
`c12a7da21db837576bfe930c3e3fcd5da23542a5638a287c640f27818618bb99`,
`81e1f4af882a27a7aaa500445faf196e6863215d6b4bb5703a665749e23abbe3`,
and
`e6cecba821fe89e0739257a1b9595913087a5e4922edd1e8c1b7b105b07140c6`.
This records separate lowering work without revising historical H02a1
named-test acceptance.

**V01/CI admission-owner follow-up:** GitHub's
[create-PR job](https://github.com/sifr-lang/sifr/actions/runs/36634682383/job/109632314445)
failed before validation setup at `performance_reference_admission`:
`SIFR_PERFORMANCE_REFERENCE` was unset. Its normalized log is retained as
`ci-create-pr-failed.log` in the final candidate evidence directory. This is
an infrastructure failure, not H02c codegen evidence or a passing CI claim.

This blocker receipt changes documentation only. Documentation structure,
the source file-size guardrail (4,284 files on the documentation branch) and
diff check passed. No repeated Cargo gate or external review was required.

### H01/F28 needs-new-scope split (2026-09-28)

At the split, main had deterministic source-mutation smoke and repeated-output
checks in `verification/areas/fuzz_property`, but no Sifr-original
coverage-guided target execution or the five required semantic properties.
H01a supplies the parser target, and H01b supplies lowering and ownership
targets; the other guided targets and semantic properties remain open. The
historical M11/#3563
implementation is unmerged and its deferred findings are not passing current-main
evidence. H01 is **needs-new-scope**: nine bounded items below replace one
cross-subsystem implementation item. At the split these were planned test names
and commands, not existing passing tests. Each owner adds the named cases, runs its exact
selection and focused regressions, obtains scoped review, merges, and records an
independent receipt before the next dependent item starts. Preserve deterministic
mutation smoke throughout. Do not claim H01/F28 complete from one slice.

| Item / owner | Dependency and bounded implementation | Named acceptance to add and run |
| --- | --- | --- |
| H01a / verification runner and parser; merged in [PR #4090](https://github.com/sifr-lang/sifr/pull/4090), receipt below | First. Add a real instrumented, coverage-guided parser target and a sustained runner with a measured nonzero execution count and coverage feedback. Keep cold build preflight outside each target budget. Classify missing tool, offline dependency, instrumented build, target timeout, and compiler finding separately; retain bounded build-output tails, globally unique variant labels and input/tool/config identity. Minimize and preserve a stable reproducing seed before promotion; the current whitespace-only `minimize_seed.py` is not minimization evidence. Reconcile nightly/release timeouts and resource classes. Remove the historical fuzz workspace’s unused `serde_json` dependency unless a target genuinely consumes it. | `python3 -m unittest verification.runner.sifr_verify.hardening.test_coverage_fuzz` cases `test_missing_tool`, `test_offline_dependency_failure`, `test_instrumented_build_failure`, `test_target_timeout`, `test_compiler_finding_minimized_seed`, `test_nonzero_guided_executions_and_coverage`, `test_budget_and_unique_labels`, `test_no_unused_fuzz_dependencies`; `cargo +nightly fuzz run --fuzz-dir verification/fuzz parser <corpus> -- -max_total_time=10 -print_final_stats=1` with measured execution/coverage receipt. |
| H01b / frontend guided targets; merged in [PR #4093](https://github.com/sifr-lang/sifr/pull/4093), receipt below | After H01a. Add lowering and ownership targets through real compiler APIs, with diverse typed source grammar rather than fixed literal templates. Reuse H01a taxonomy, corpus and minimization contracts; do not change diagnostic, project or codegen target ownership. | `cargo +nightly fuzz run --fuzz-dir verification/fuzz lowering <corpus> -- -max_total_time=10 -print_final_stats=1` and the same exact command for `ownership`; `cargo test -p sifr_frontend --lib guided_target_replay_tests::lowering_seed_replay -- --exact` and `cargo test -p sifr_frontend --lib guided_target_replay_tests::ownership_seed_replay -- --exact`. Record nonzero executions, feedback, and stable minimized reproductions for findings. |
| H01c / diagnostics guided target; merged in [PR #4096](https://github.com/sifr-lang/sifr/pull/4096), receipt below | After H01a. Mutate structured rendered-diagnostic envelopes directly and exercise JSON, human and compact renderers. Use a specific active diagnostic identity; never use the forbidden catch-all `SIFR-TYPE-0001`. Preserve H01a failure taxonomy and minimized JSON artifacts. | `cargo +nightly fuzz run --fuzz-dir verification/fuzz diagnostics <corpus> -- -max_total_time=10 -print_final_stats=1`; `cargo test -p sifr_driver --lib tests::guided_target_replay::diagnostic_renderer -- --exact`; assert nonzero guided executions, all three renderers and a stable minimized structured seed. |
| H01d / project-graph guided target; merged in [PR #4100](https://github.com/sifr-lang/sifr/pull/4100), receipt below | After H01a. Generate bounded project/manifest/import graphs, create each fixture outside the fuzz hot loop, and exercise the canonical project compiler. Keep dependency/tool failures separate from compiler findings and preserve minimized project trees. | `cargo +nightly fuzz run --fuzz-dir verification/fuzz project_graph <corpus> -- -max_total_time=10 -print_final_stats=1`; `cargo test -p sifr_driver --lib tests::guided_target_replay::project_graph -- --exact`; assert nonzero guided executions, graph mutations and stable minimized project-tree replay. |
| H01e / codegen guided target; blocked pending resolution of automatic content review restriction, receipt below | After H01a and H01b. Generate varied *valid* typed programs and exercise codegen plus emitted-Rust validation without treating expected user diagnostics as findings. Do not alter X02 codegen safety ownership. Preserve minimized valid wrong-code/panic seeds. | `cargo +nightly fuzz run --fuzz-dir verification/fuzz codegen_validation <corpus> -- -max_total_time=10 -print_final_stats=1`; `cargo test -p sifr_driver --lib tests::guided_target_replay::codegen_validation -- --exact`; assert nonzero guided executions, valid-program diversity and stable minimized replay. |
| H01f / type-system semantic properties | After H01b; merged in [PR #4104](https://github.com/sifr-lang/sifr/pull/4104), receipt below. Add generative normalization idempotence/equivalence and narrowing partition/soundness properties using canonical type-system operations, including unions, optionals and complements. A repeated fixed fixture is insufficient. Own the Rust semantic-property one-run contract: reject an unused `repeat_runs` field on Rust test entries or execute the declared count. Leave ownership transfer and incremental sessions to H01g/H01h. | `cargo test -p sifr_type_system --lib semantic_property_tests::normalization_idempotent -- --exact` and `cargo test -p sifr_type_system --lib semantic_property_tests::narrowing_partition -- --exact`, plus existing `narrow::tests` regressions. Save deterministic failing seeds and replay commands. |
| H01g / frontend ownership property | After H01b and H01f; merged in [PR #4107](https://github.com/sifr-lang/sifr/pull/4107), receipt below. Generate move/borrow/mutability programs and compare accepted/rejected ownership behavior to explicit transfer invariants through the canonical frontend product. Include non-Copy values and branch joins. Do not substitute panic-free compilation for the ownership oracle. | `cargo test -p sifr_frontend --lib ownership_property_tests::move_borrow_branch_invariants -- --exact` plus focused existing ownership fixtures; record generated case count, deterministic random seed and minimized failing program. |
| H01h / frontend/session equivalence; merged in [PR #4110](https://github.com/sifr-lang/sifr/pull/4110), receipt below | After H01g and E01c. Compare incremental edits with fresh full compilation from the same source and external-input snapshot, including add/delete/rename, diagnostic spans, cancellation and recovery. Preserve E01 watcher and cache ownership; this item owns only the equivalence property. | `cargo test -p sifr_frontend --lib query_diagnostics_equivalence_tests::incremental_full_property -- --exact` and `cargo test -p sifr_lsp --lib incremental_full_property_tests::request_publication -- --exact`; record edit-sequence seed, matched products/diagnostics and minimized failing sequence. |
| H01i / codegen determinism and H01 integration | After H01e-H01h. Compile the same generated valid program in separate compiler processes and isolated output roots, compare emitted Rust and binary-observable behavior under varied process hash seeds, and verify reproducible diagnostics. Wire completed guided targets into nightly/release selection and keep local deterministic mutation smoke. At integration, rename the existing `fuzz-smoke` suite to `mutation-smoke` in the manifest and every caller; retain its seed execution and assertions. Map the existing five `*_entrypoint` ids to the six guided target names without duplicating ownership, reconcile the 10/30-minute sustained budgets with the area timeout/resource class, and update fuzz policy and release-evidence labels; do not claim a release checkpoint absent a release request. | `cargo test -p sifr_driver --lib tests::semantic_properties::cross_process_deterministic_codegen -- --exact`; `uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite property --suite mutation-smoke`; nightly/release selected `sustained-fuzz` runs with measured executions, unique labels, separated outcomes and corpus/minimized-seed receipts. |

The historical M11 review/gate follow-ups on build/tool/dependency/timeout
classification, output tails, budget/resource alignment, unused fuzz dependency,
semantic-property run contract, cross-process codegen, grammar diversity and
unique release labels are assigned above. `SIFR-INTERNAL-0003` registry coverage
remains D01c-owned; H01c must use an already active specific identity. H01a
must state the guided-tool/toolchain prerequisite before running its named live
selection; inability to install or execute that prerequisite is a recorded
external blocker, not a zero-execution pass. Each target owner preserves a
minimized seed and exact replay command for an actual finding; a clean target
records corpus identity and execution/coverage counters without inventing one.

### H01a/F28 parser-guided delivery receipt (2026-09-29)

[PR #4090](https://github.com/sifr-lang/sifr/pull/4090) merged as
`5b8df854023f7547e4604f6c7edad33d89660f46` from exact tested and
reviewed candidate `c5a4eb04f9541420a9d9638b6a38a4ab0a7f12cb` (base
`a2b80875d5e919f2dc0c43e58e2127cb453e3aa5`, tree
`158d34971f8f9c6475b51023fcc565f192da347f`). H01a adds the Sifr-original
instrumented parser target, three pinned source seeds, a standalone fuzz lock
without an unused direct `serde_json` dependency, and a runner with build
preflight outside the target budget. Nightly/release target budgets are 600/1800
seconds; the area reserves 3900 seconds and long-running/large-memory resources.
The `sustained-fuzz` suite is explicit-only until H01i integrates all targets.

The named H01a unittest selection passed all eight planned cases; two focused
regressions for old-artifact isolation and explicit suite selection passed as
well (**10/10**). The exact live selection
`cargo +nightly fuzz run --fuzz-dir verification/fuzz parser
verification/fuzz/corpus/parser -- -max_total_time=10 -print_final_stats=1`
completed **6,029 executions** with **4,985 coverage edges**. A repaired-runner
ten-second selection passed with **7,000 executions** and **5,117 edges**.
The receipt identifies cargo-fuzz 0.13.2, rustc 1.101.0-nightly, the manifest,
Cargo lock, target source and pinned-corpus hashes. It is retained under the
session-owned `target/verification/fuzz/h01a-parser-repair-receipt.json` and
summarized in [the PR review evidence](https://github.com/sifr-lang/sifr/pull/4090#issuecomment-5879533515).
No compiler finding appeared, so no minimized failing seed is claimed.
Area schema, documentation structure, nightly formatting, HIR maintainability,
diff checks and the under-900-line source guardrail passed. The guided prerequisite
was installed and executed: nightly Rust, cargo-fuzz 0.13.2, the Ruff submodule
and downloaded fuzz dependencies. The initial documentation check lacked the
nested VS Code submodule; after initialization the check passed. The first
Opus review found stale-artifact misclassification; the repair added isolated
per-run artifacts and a `target-run-failure` outcome, then affected tests and
runner execution passed. Final scoped read-only Opus 5.5 review returned
**SATISFIED** with no blocking findings (response SHA-256
`bc7e2ac8adc92abee9e78d0bea212b70f2db220073feaba2f6d388a075586997`).
Nonblocking runner hardening is tracked in [issue #4091](https://github.com/sifr-lang/sifr/issues/4091);
H01i retains profile mapping and selection. No create-PR or full merge gate ran
under the approved intermediate-item exception. H01/F28 remains open for H01b-H01i.

### H01b/F28 frontend-guided delivery receipt (2026-09-29)

[PR #4093](https://github.com/sifr-lang/sifr/pull/4093) merged as
`0b3a0c660550f21a2f1ff6315759cdb33944a4ba` from exact tested and
reviewed candidate `3095684321892bff688b23c30e5299cf26e1d805` (base
`041b551cc15185fc9411dad33ce4dbe9436f2f90`, tree
`96141c049e43f86851f147f0049ad7be530637ff`). H01b adds bounded,
byte-driven typed grammars for frontend lowering and ownership, with recursive
expressions, scoped bindings, generated helper signatures, nested control flow,
moves, borrows, reassignment, a branch join and a list call. Each target enters
the canonical frontend parse/HIR path. Seven pinned seeds have exact frontend
replay tests, including accepted lowering/borrow cases and rejected move/branch
cases. The H01a runner now selects parser, lowering or ownership with each
target's source and corpus identity. Its build/tool/timeout/finding taxonomy,
isolated working corpus, minimization and two-replay confirmation are reused.
H01i still owns nightly/release suite mapping; this item did not alter the
diagnostic, project or codegen targets.

Both named frontend replay tests passed. The H01a runner's focused unit suite
passed **12/12**, including per-target identity and classification. Ten-second
classified runner selections passed build and execution: lowering **479
executions/11,207 coverage edges**, ownership **1,325/9,774**. Both exact
acceptance cargo-fuzz commands also passed: lowering **449 executions/11,214
edges**, ownership **1,227/9,773**. No compiler finding appeared, so no
minimized failing seed is claimed. The candidate-keyed receipt, with nightly
rustc 1.101.0-nightly, cargo-fuzz 0.13.2, hashes for the manifest, Cargo lock,
target sources and pinned corpus, and the classified/exact runs, remains under
the session-owned
`target/verification/fuzz/h01b-3095684321892bff688b23c30e5299cf26e1d805-receipt.json`
(SHA-256 `c3fec3a09c52369957eb6399f925f0ac3f6efb90301a8110539e4c9c27641806`).
Scoped frontend Clippy, formatting, diff checks, the under-900-line source
guardrail and HIR maintainability guardrail passed.

The first scoped Opus 5.5 review identified a flat generator and parser-only
runner; both were repaired before the final candidate, and all affected
selections reran. Final read-only Opus 5.5 review returned **SATISFIED** with no
blocking findings (response SHA-256
`05e2ca2edb76138efc8d6660f7bbc46b57af5a15bfb81fe8b2834afc65e67f9b`;
[PR evidence](https://github.com/sifr-lang/sifr/pull/4093#issuecomment-5880105516)).
Nonblocking replay-assertion suggestions are tracked in
[issue #4094](https://github.com/sifr-lang/sifr/issues/4094). H01i retains
sustained-suite integration. The automatic PR `local-first-create-pr` job
[failed](https://github.com/sifr-lang/sifr/actions/runs/36494106250/job/109169514999)
at the already recorded V01 `performance_reference_admission` prerequisite
before Cargo because the runner did not select `SIFR_PERFORMANCE_REFERENCE`
(raw job log SHA-256 `904e3df0e99517ee78b6855259f10e1d608441923b4dc60c8a8e00adbe4245b6`).
It is not H01b validation evidence. No local create-PR or full merge gate ran
under the approved intermediate-item exception. H01/F28 remains open for
H01c-H01i.

### H01c/F28 diagnostics-guided delivery receipt (2026-09-29)

[PR #4096](https://github.com/sifr-lang/sifr/pull/4096) merged as
`1154d338c0050fa15beea25c4271ef4fb47e1f75` from exact tested and
reviewed candidate `070a87f070678ee85a563328210f8afe5eeb4bf1` (base
`38fa9140f676fcd6963e4af2c9825553eba4fe64`, tree
`8331738a66185413906d54d5c16fde388d5bdc71`). The target mutates
serialized `DiagnosticEnvelope` JSON directly and invokes JSON, human and
compact renderers. It uses active `SIFR-TYPE-0002`, with bounded envelope and
snippet-highlight sizes. Three pinned structured JSON seeds cover a minimal
envelope, source spans/children/suggestion edits, and edge span/argument forms;
the exact driver replay asserts all three renderers and stable output. No
`SIFR-TYPE-0001` seed or registry change was added. H01d/e targets remain
separately owned.

The H01a runner now selects `diagnostics`, preserves minimized diagnostic
findings as `.json` with two-replay confirmation, and retains its separate
tool, offline dependency, instrumented build, timeout and compiler-finding
outcomes. It captures execution and coverage counters from full LibFuzzer
output before retaining the bounded tail. The first classified run incorrectly
reported `no-coverage-feedback` because its tail contained only LibFuzzer's
recommended dictionary; the counter fix has a focused regression, and that
failed attempt remains separate from the final passing run.

On the exact candidate, the named `sifr_driver` replay passed **1/1** and the
focused runner unit suite passed **15/15**. The exact ten-second cargo-fuzz
selection passed **383,006 executions / 3,876 coverage edges** with no
finding. The classified runner separately passed build and target execution
at **415,658 / 3,866**, using cargo-fuzz 0.13.2 and rustc
1.101.0-nightly. Package-scoped nightly formatting, the 900-line file-size
guardrail (4,273 files), driver maintainability guardrail and committed diff
check passed. Candidate-keyed logs, input hashes and exact commands are
outside the Git tree at
`/home/yaser5/projects/sifr/h01c-evidence-20260929/070a87f070678ee85a563328210f8afe5eeb4bf1/`;
validation manifest SHA-256
`413eeff0cf4e8c61feeaca9265005c900a1f5dc62012db447c423f8144204d4a`.
No compiler finding occurred, so no minimized failing seed is claimed. The
first cold driver attempt lacked the pinned Ruff submodule; the exact replay
passed after initialization.

Scoped read-only Claude Opus 5.5 review returned **SATISFIED** with no
blocking findings (response SHA-256
`c81392aac77d7605f141b4e3b4b8d47dc0cc66393a10347c0529eae853cf1853`).
Its nonblocking suggestions on mutated identity filtering, possible future
float round-trip findings and an invalid-minimized-JSON negative test are
tracked separately in [issue #4097](https://github.com/sifr-lang/sifr/issues/4097).
The automatic PR `local-first-create-pr` [job](https://github.com/sifr-lang/sifr/actions/runs/36497751498/job/109181180304)
stopped at the already recorded V01 `performance_reference_admission`
prerequisite before Cargo because `SIFR_PERFORMANCE_REFERENCE` was not
selected (raw job log SHA-256
`6cf31238d07f5bd212977346c8f7ccd3b65eedbd5b3808a4a389b29a8bd5cbdc`).
It is not H01c validation evidence. No local create-PR or full merge gate ran
under the approved intermediate-item exception. H01/F28 remains open for
H01d-H01i; Q01 retains the final gate.

### H01d/F28 project-graph guided delivery receipt (2026-09-29)

[PR #4100](https://github.com/sifr-lang/sifr/pull/4100) merged as
`e36af9c50c48a8abe06fbe9b0933cb9a35f8dac1` from exact tested and
reviewed candidate `fcb3b08c9c3e275ea060206e96ae7508308ae2a4` (base
`c8f5dc10f95c7f93b009394f4d5345caa4c62906`, tree
`3a5e9c596b9a6aaa21ed4b86c5cf862bf6213b29`). Four input bytes select
at most four package manifests and six dependency edges, plus imports. The
checked-in fixture anchors paths; each fuzz iteration overlays its entire
project tree in memory, derives the canonical package graph and source map,
and calls `check_package_project`. Manifest, graph, and user-code diagnostics
remain outcomes rather than compiler findings. The runner registers this
target, retains H01a's separate build/tool/dependency/timeout taxonomy, and
exports a complete hashed package tree after stable finding minimization.
No compiler finding occurred in this item.

On Linux x86_64 with rustc 1.98.1, the exact driver replay passed **1/1**:
five pinned graph seeds have byte-for-byte 16-file exports and matching
disk-provider replay, and changing a manifest dependency changes an import
from a project diagnostic to an accepted project. The focused existing
project-graph regression passed **1/1** and runner unit tests passed
**16/16**. With rustc 1.101.0-nightly and cargo-fuzz 0.13.2, the named
10-second guided run passed **8 executions / 44,996 coverage edges**; a
separate classified 10-second run passed build and execution at
**7 / 44,985**. Formatting, committed diff, HIR maintainability and the
900-line touched-source guardrail passed. The first parallel instrumented
build was killed by the 11 GiB host's memory pressure in `cranelift-codegen`;
a single-job retry on the same isolated target passed. The earlier candidate
had a passing guided run but a **NOT SATISFIED** review because its generated
manifest did not reach compilation; those findings were repaired and that
run is not final-candidate evidence. Candidate-keyed logs, JSON and validation
manifest are outside the Git tree under
`/home/yaser5/projects/sifr/h01d-evidence-20260929/fcb3b08c9c3e275ea060206e96ae7508308ae2a4/`;
the validation manifest SHA-256 is
`5be1ddd76b5d8896f4dc3ce507d90965180c1c32aa6395e11a0f3e6346620134`.

Final scoped read-only Claude Opus 5.5 review returned **SATISFIED** with no
blocking findings (response SHA-256
`277c33d3a365d51cc6923af0ea5d5d2185983d22fd1cf2e0c945f2c64b95950a`).
Its nonblocking disk-replay, diagnostic-detail and fixture-documentation
suggestions are tracked in [issue #4101](https://github.com/sifr-lang/sifr/issues/4101).
No local create-PR or full merge gate ran under the approved intermediate-item
exception. H01/F28 remains open for H01e-H01i; Q01 retains the final gate.

### H01e/F28 codegen-guided external blocker (2026-09-29)

This docs-only blocker record is [PR #4103](https://github.com/sifr-lang/sifr/pull/4103).
Documentation structure, file-size guardrail and diff checks passed.

The H01e worker's exact driver replay selection,
`cargo test -p sifr_driver --lib tests::guided_target_replay::codegen_validation -- --exact`,
passed on its owned worktree. The required exact live guided selection,
`cargo +nightly fuzz run --fuzz-dir verification/fuzz codegen_validation <corpus> -- -max_total_time=10 -print_final_stats=1`,
was still in instrumented build when automatic content review stopped the
worker twice with the stated reason `possible cybersecurity risk`. Neither
interruption establishes an instrumented-build failure or a compiler finding.
The live selection has no passing execution or coverage receipt; H01e has no
scoped review, implementation PR, or completion claim.

The worker's edited worktree at
`/home/yaser5/projects/sifr/h01e-codegen-guided-20260929` and its target
are preserved. After the worker errors, the orchestrator stopped its orphaned
Cargo process. An earlier SSH outage and disk-pressure cleanup were setup
history, not the current blocker or a failed H01e acceptance result. H01e is
**blocked pending resolution of the automatic content review restriction**.
The exact next action is to resolve that restriction, resume the preserved H01e
candidate, complete the named live selection and scoped review, then prepare
and merge its implementation PR. H01f-H01i and whole H01/F28 qualification
remain open.

### H01f/F28 type-system semantic-property delivery receipt (2026-09-29)

[PR #4104](https://github.com/sifr-lang/sifr/pull/4104) merged as
`ebdc82132bfafe4b541a6a525030fda6e6078675` from exact tested and
reviewed candidate `01531b3487fea22b873c1c2982e3029d1c8456c8` (base
`2fbc376520962ce6ec6e9aea4de6a43942b9b585`, tree
`86d9791be89d7f289c9f236c03add8aa94ed299b`). Seeded generation
exercises canonical union normalization idempotence, permutation and duplicate
equivalence, and assignability witnesses over nested unions, optionals and
literals. A separate generated finite-domain oracle checks narrowing partition
and complement soundness for 2,048 domains, 11 conditions, seven witnesses and
both branches. The first failing seed was `0x483031664e415252`, case 0, mask
`0x76`: an impossible `is None` true branch produced `None`. The exact replay
is `cargo test -p sifr_type_system --lib semantic_property_tests::narrowing_partition -- --exact`; assertion output
includes seed, case, mask, condition, branch and witness. The repair also keeps
reachable alias-member, float equality and TypeVar branches conservative.

On the final candidate, both named exact semantic-property selections passed
**1/1** each, existing `narrow::tests` passed **14/14**, and Rust property
runner contract tests passed **3/3**. Both manifest Rust entries passed once
through the runner as `run-1`. Scoped Clippy, formatting, diff and 900-line
file-size checks passed. The optional full type-system library run became
unverified after an SSH stall and is not acceptance evidence. Candidate-keyed
validation and both read-only review responses are outside the Git tree under
`/home/yaser5/projects/sifr/h01f-evidence-20260929/01531b3487fea22b873c1c2982e3029d1c8456c8/`;
the validation manifest SHA-256 is
`f2bb07fbdb76e7d51bb6b6e6bf8d0108481ff48f7b36ecab249322d9926a1f86`.

The first scoped Opus 5.5 review found an in-scope alias/float/TypeVar omission,
which the second candidate repaired. Final read-only review returned
**SATISFIED** with no blocking findings (response SHA-256
`132843704912bcf37458ddbb0faa2a54d0eb7fbb0d2b232a934595440ea650e9`).
Pre-existing `IsInstance` alias-member overlap is tracked in
[issue #4105](https://github.com/sifr-lang/sifr/issues/4105); optional broader
property witnesses and dead-branch lowering coverage remain follow-up ideas,
not H01f claims. No local create-PR or full merge gate ran under the approved
intermediate-item exception. H01/F28 remains open; H01e remains blocked and
H01g-H01i retain their separate scope. Q01 owns the final gate.

### H01g/F28 frontend ownership-property delivery receipt (2026-09-29)

[PR #4107](https://github.com/sifr-lang/sifr/pull/4107) merged as
`232603110533bfabfe714707113868c8ef3159ab` from exact tested and
reviewed candidate `0888b14d2e3e03ea0646db2ec08acd08e6c7207c` (base
`0eba692fc7bbc8b0d8c6697e047e99498b59cf11`, tree
`dc4b444e32fc5b5a74d1c2bc61956cf0da309356`). Deterministic seed
`0x483031674f574e52` produced **512** ownership programs using non-Copy
`str` and `list[int]` values. Their explicit transfer oracle checks borrows,
moves into assignments and `own` parameters, rebindings, conditional branch
joins, and mutable/shared aliasing through `compile_frontend_product`.
Each generated case asserts the expected acceptance or a single exact ownership
diagnostic code. A failing case prints its source and can be replayed with
`SIFR_H01G_CASE=<index> cargo test -p sifr_frontend --lib ownership_property_tests::move_borrow_branch_invariants -- --exact --nocapture`.

The named exact selection passed **1/1** with all 512 generated cases. Focused
replay of eight existing ownership fixtures passed **1/1** (three accepted,
five rejected). Scoped Clippy, formatting, diff, 900-line file-size, and
lowering maintainability checks passed. Initial test-only Clippy style failures
were corrected before the final candidate. No generated compiler finding
occurred, so there is no minimized failing program. Candidate-keyed validation
and review evidence are outside the Git tree under
`/home/yaser5/projects/sifr/h01g-evidence-20260929/0888b14d2e3e03ea0646db2ec08acd08e6c7207c/`;
the validation manifest SHA-256 is
`73fd761c57d24a0f55023ddbc483fcaeb2a01a4f4835eba63a4d974b837cd98e`.

Scoped read-only Opus 5.5 review returned **SATISFIED** with no blocking
findings (response SHA-256
`05822c9106e0ae8ed96a0db3baa029cd28495ac0060cde60f51e832f6e220fd8`).
Broader mutability, nested branch, early move-use, diagnostic-span and
immutable-parameter generation were suggestions for separate follow-up work,
not H01g blockers. No local create-PR or full merge gate ran under the
approved intermediate-item exception. H01/F28 remains open; H01e remains
blocked, H01h-H01i retain their separate scope, and Q01 owns the final gate.

### H01h/F28 frontend/session equivalence delivery receipt (2026-09-29)

[PR #4110](https://github.com/sifr-lang/sifr/pull/4110) merged as
14f69a8a717a62d0435668b5643386b884b9dd2c from exact tested and reviewed
candidate c18732b7cecb096ee1f0c63c161f80c071bd411a (base
e113c89437e138acae4787b54ae3c1edee6b758b, tree
4496b3ee987f3acb11cb72b5fef303f7518ef6ea). The change adds only test
code and its test-module registration. E01 watcher, cache and publication
implementation ownership is unchanged.

Frontend seed 0x4830316845515549 runs 64 incremental project snapshots. Each
snapshot compares rendered diagnostics including spans, import-graph edges,
module membership and source-map bytes with a fresh project load from the
same files. The sequence covers source edits, version-only updates, warm
queries, add/delete/rename, an unresolved import and recovery. Valid added
and renamed modules resolve with the expected graph edge; deletion has the
specific SIFR-IMPORT-0002 diagnostic. LSP seed 0x483031685055424c runs
32 incremental document edits, comparing every published version and JSON
diagnostic/range with a fresh Session at the same disk and overlay snapshot.
Three further project transitions deliver create, delete and rename through
workspace/didChangeWatchedFiles and compare each resulting publication with
a fresh Session. The deleted import checks the exact 0:0–0:23 range. An
in-flight cancelled request returns RequestCanceled and later publication
recovers. Failure messages print seed, step, operation and source. No generated
failure remained, so there is no minimized failing sequence.

On the exact candidate, both named one-test selections passed 1/1. Focused
frontend project-edit and workspace-session regressions and the LSP watcher
regression passed 1/1 each. The 900-line file-size guardrail passed for
4,278 files; formatting and committed diff checks passed. Raw candidate-keyed
logs are outside Git at
/home/yaser5/projects/sifr/h01h-evidence-20260929/c18732b7cecb096ee1f0c63c161f80c071bd411a/.
The named frontend and LSP log SHA-256 values are
dedb04587405b9154e2d2f5d37d0b42225cfcb50b37ba109977650d0bf03da27 and
12b62a9fbea73471ab0325a319e62c8c70c1530b28d97a8df9d801e0207b765a.

The first scoped read-only Opus 5.5 review returned NOT SATISFIED because
structural fixture strings contained literal escape sequences and LSP
structural events were not exercised (response SHA-256
b9c9ad1f5f5eabfc8b76aed24382d1618b478e1f6dec270c1d923ad4765d7136).
Both findings were repaired in the final candidate. The remediation review
returned SATISFIED with no blocking findings (response SHA-256
b3953e03d5591269f724792f9f73fe238fe21f5573f269e9b96982d948f2e0f5).

An unchanged E01c cancellation/reopen regression failed identically on the
initial candidate and its exact base at the verified-completion assertion;
the base failure log SHA-256 is
d1b35343078afc940fc5a8e50351ab28956dd74faec1e8cbd30607adedd489e4.
It is recorded in [issue #4109](https://github.com/sifr-lang/sifr/issues/4109)
and did not block H01h's named equivalence acceptance. The reviewer noted a
pre-existing LSP/driver source-root discovery difference, recorded in
[issue #4111](https://github.com/sifr-lang/sifr/issues/4111).
No local create-PR or full merge gate ran under the approved intermediate-item
exception. H01/F28 remains open; H01e remains blocked, H01i retains its
separate scope, and Q01 owns the final gate.

### Resolved prerequisites and boundaries

- DX9-F7 lock drift: [PR #3903](https://github.com/sifr-lang/sifr/pull/3903), merge 439b5ebd5c6917872f58288453917a2427509192. Same-root alternation and resolution tests passed for that item. Retained Emitted Rust Item 12 remains unqualified.
- Taxonomy [issue #3898](https://github.com/sifr-lang/sifr/issues/3898) closed completed 2026-09-22 and is present on baseline main. The Emitted Rust header's blocker wording is stale; its owner updates that handoff before qualification.
- Solo-maintainer release approval: [PR #3827](https://github.com/sifr-lang/sifr/pull/3827), merge 33639f4ee3b7079ec4da889834cae0d55d763786. The [active-named record](ad-hoc-distinct-release-reviewer-restoration.md) describes permanent supersession. D01 owns correction of its status and links. Publication still needs approval of the exact protected run.
- Windows native storage/process behavior stays with [Windows driver portability](ad-hoc-windows-driver-portability.md), not V04/N06.
- The historical DX.10/B11 runner self-test inventory mismatch in the V01 receipt is resolved by [PR #3986](https://github.com/sifr-lang/sifr/pull/3986), merge 53664cbab52055b9bf8aa5df01b79743d57927d8. Its exact locked runner self-test passed on reviewed candidate 9e8740fc815c321c92b8cdf48d114c8e5a09e0a4; the prior failed log remains historical. SQL Item 4 still owns final-candidate qualification. The [DX.10 owning receipt](ad-hoc-dx10-profile-review-followups.md#reconciliation-2026-09-24) holds the raw evidence and scoped review.
- SQL Item 3 [PR #3975](https://github.com/sifr-lang/sifr/pull/3975), merged as `8bd81fcddd2226daf98a38c6e58e32a4d4ae4680`, had Windows CI pass 16 component cases, then failed with 55 pre-existing `sifr_driver` portability errors across seven modules (including `process_signals.rs`, omitted from the earlier inventory). N02c/F08 serialization changed no Unix-only storage operations. The Windows owner has [W1 secure storage, W2 process ownership and W3 integrated SQL qualification](ad-hoc-windows-driver-portability.md#2026-09-23-sql-item-3-ci-retriage-and-delivery-split) as separate sequential acceptance items; no Windows fix or passing qualification is claimed. The docs-only scope handoff merged in [PR #3978](https://github.com/sifr-lang/sifr/pull/3978) as `3fe57b88a1e0a778bbc6605e5f06a2275352faa9` from candidate `54476cc35b29c8dd1418a815109e3168f5c3bc60`; documentation structure and diff checks passed.

## Historical milestone and draft disposition

Historical M1-M12 drafts #3553-#3564 remain open on the unmerged stack at this review. Their candidate SHAs, validation and reviews remain historical evidence. Evaluate small deltas against current owners before any selective port; do not merge the stack or claim old tests pass on current main.

| Historical item / PR | Disposition |
| --- | --- |
| M1 #3553 | Cache authority/serialization retained in N02-N04 under current Cargo-before-final-cache design. |
| M2 #3554 | Supported build/test parity in N01; mutable leased family and immutable final snapshot remain distinct. |
| M3 #3555 | Effects/parser guards V03 and lifecycle V04; performance budgets are separate from kill deadlines. |
| M4 #3556 | Current docs/maps D01; Cargo aliases and targets defeat directory-only crate checks. |
| M5 #3557 | Urgent legal-name X01 then structural safety X02 under generated Rust owner. |
| M6 #3558 | Result propagation, no recovery output and executable diagnostics X02. |
| M7 #3559 | Frontend product/cycles C01 with current metadata and memory evidence. |
| M8 #3560 | Lower services C02, editor correctness E01-E02 and measured scale E03. |
| M9 #3561 | Typed method and unsafe contracts H02; no count-only closure. |
| M10 #3562 | Existing framed SHA/storage foundation retained; substantial N02-N06 residuals. No competing primitive/cache CLI. |
| M11 #3563 | Guided/semantic goals H01; deterministic mutation smoke remains useful. |
| M12 #3564 | Ratchets/flow H03, companions and registries D01/Q01. |
| M12A/C/E #3565/#3567/#3569 | Merged into historical M12 branch only. Keep descendant/terminal/cancellation scenarios in V04, not obsolete machinery. |
| M12B #3566 | List fallback regression useful in H02; old failure is not presumed live. |
| M12D #3568 | Old registry mismatch is noncurrent; D01 owns new consistency tests. |
| M12F #3570 | Regenerate current companions in Q01 after current changes; no old generated diff transplant. |
| M12G #3572 | Conditional on current C01/H02 lowering exports. |
| M12H #3573 | Old heading absent on main; retain taxonomy check, retire old failure. |
| M12I #3576 | Live Python-interop prefix defect V02; budgets measured independently. |
| M12J #3577 | Old 120-to-300-second budget is not portable; V04/Q01 qualify current selection/cache state. |
| M12K (no PR) | Controlled-host reference attempt blocked; V01 is early prerequisite. |
| M13 #3571 | Historical branch review is not current-main closure; Q01 owns final integration and whole-phase review. |

V03 scans every Rust source beneath each Cargo workspace member and manifest-declared target path, including build scripts, bins, examples, benches and tests. Four demos (`demos/csv`, `demos/parse_safety`, `demos/shutil`, `demos/stdlib_intrinsics`) are Cargo workspace members and are covered. Rust reference material in non-member demos remains outside this guard.

The unmerged [V03 draft PR #3923](https://github.com/sifr-lang/sifr/pull/3923) at candidate 45eaf9a43b217c75fe2c34ee7a6068da0d85f7cb is historical evidence only. Its second scoped review found new mechanism-level F24 and F25 omissions and required this F24/F25 split. The unmerged [F24 draft PR #3925](https://github.com/sifr-lang/sifr/pull/3925) ended at candidate 3d0ca988480ce1f2593b08e99d3f12ddb843cb8e after its second review found the OS symlink API boundary missing. V03 selectively ported and completed the filesystem guard in PR #3926; V03b completed parser work separately in PR #3929. The unrelated TypeScript-Go LSP guard failure remains with E01.

The unmerged [V03b draft PR #3928](https://github.com/sifr-lang/sifr/pull/3928) at candidate `143ea849fc6cf80482cdb52923ce2aeb4ef41649` is historical evidence. Its second review repeated the forbidden-call alias bypass. V03b rescope selectively ported its test/production classification and completed the callable boundary in PR #3929. Old draft status is historical candidate pending selective port or owner-approved closure, not approved for merge. Closing drafts is a separate repository action. Preserve failed/partial logs without relabeling.

## Order, gates and evidence

1. P00, then urgent X01.
2. Validation prerequisites V01-V03b, V04 and D01a registry guardrails; runner negative tests precede reliance on later green evidence.
3. Native/cache N01-N06; N01 depends on merged DX9-F7 behavior, N06 uses DX9-F5 ownership, and N02 establishes identities before warm-consumer claims.
4. X02 with generated Rust owner after X01, updating diagnostics/fixtures/materialization together.
5. C01, then C02a0 metadata production/storage, C02a1 metadata consumer/stdlib, C02b Python authoring, C02c SQL editor, C02d preview/editor checks, and C02e final boundary guard in that order.
6. E01a external generations, then E01b watcher authority, then E01c verified Python fast-hit reorder and publication. E02 remains separate after C01; E03 follows E01c and E02. E04 independently completes before editor closure.
7. H01, then H02a0, H02a1, H02b, H02c, atomic H02d0, runtime-only H02d1, H02e-H02g and H02h in their recorded dependency order, H03, D01b current documentation and D01c residual registry completion.
8. Q01 exact-candidate integration, owner handoff audit, whole-phase review and closure. Release qualification only on an actual request.

The 2026-09-23 assignment prospectively approves intermediate items with named acceptance tests, focused regressions and scoped Opus review, without per-item create-PR/full merge gates. Q01 runs one full merge profile on final merged work; repair the first in-scope cause, rerun failed/affected checks and the full gate until it passes. The whole-phase closer edits docs only and returns needs-implementation for defects. Other owners retain their own recorded rules.

Use exact crate/suite/case selection, fail fast by default and reuse compatible compiler/metadata/fixtures/Cargo target while running selected assertions. Evidence reuse needs unchanged implementation and validation inputs, not merely the same SHA: record tree, lock/submodule/config, compiler identity, command/selection, host/cache state where material, outcome and raw digest. Preserve failed/blocked/timeouts. Record-only updates need documentation checks, not broad gates or an extra external review after implementation review.

Scoped review names exact base/candidate SHAs, changed paths, scope, criteria and validation; it separates in-scope blockers from follow-ups. A second review with a new mechanism-level defect requires rescope. Merge one item, record PR/SHA/evidence and stop before the next batch. One session owns its worktree, branch, index and temporary paths.

## C01/F17-F18 frontend product and cycle delivery receipt (2026-09-24)

[PR #4014](https://github.com/sifr-lang/sifr/pull/4014) merged as `578c060352af1b323205f6a46fa620e9f2726d42` from exact tested and reviewed candidate `7c939798a8aad9a9505997e42c4969464e68f682` (base `5548b28460fd9d0934dc11ba1133d3623357ca65`, tree `433ec8a992d286f194e8aa2158f32fd5951b6d82`). The frontend now owns deterministic dependency/SCC order and one source-backed import-cycle diagnostic with related spans. Analysis reads that global cycle result without copying it into per-module diagnostic caches, so an unrelated module loses the cycle diagnostic when an edit repairs the cycle. The named self-cycle, multi-module cycle, broken-import and cycle-introduction/removal cases are covered.

One frontend compilation product serves supported single-file, project, package and test flows. It carries SQL, specialization, adapter, flow and diagnostic metadata; module exports are collected from the complete lowering result before HIR and flow are moved into retained output. CLI product and analysis diagnostics are compared on the same source snapshot. External definitions are prepared before both fresh lowering and HIR reuse hits.

On Linux x86_64 with rustc/Cargo 1.98.1, the exact frontend equivalence selection passed 6/6, SCC ordering 1/1, diagnostics behavior 7/7 and reuse 4/4; driver project graph passed 19/19, and SQL profile, adapter identity, test flow and diagnostic registry each passed 1/1. Frontend all-target Clippy, formatting, 900-line file-size, HIR maintainability, documentation structure and diff checks passed. The counting-allocator 200-function fixture retained 1,111,540 live heap bytes for the complete product and 1,067,278 bytes for HIR plus flow; dropping HIR and flow released all 1,067,278 measured bytes. These fixture measurements are not peak RSS or a general bound. Candidate-keyed raw logs and the validation manifest are under `/data/sifr-architecture-c01-evidence-20260924/`; the manifest is `7c939798a8aad9a9505997e42c4969464e68f682-validation.md` (SHA-256 `3b62bcfd688220ee0cc4acb5bac086217d194c7c5ea8b74baa254222e648d0c6`).

The first scoped read-only Opus review returned **NOT SATISFIED** because the cycle result could remain cached on an unrelated module after a repair edit (response SHA-256 `1ca3153531c46e9f7701ddda4f3fbe999a17d7c03db63575fa18e64f8b5b71ba`). The final remediation review returned **SATISFIED** with no blocking findings; its candidate-keyed response is `/data/sifr-architecture-c01-evidence-20260924/7c939798a8aad9a9505997e42c4969464e68f682-review.md` (SHA-256 `09797310e67b859fefa2b849f166462a07fb8930f577748f30dcb61339ee1600`). Failed and incomplete validation attempts remain preserved in the evidence directory, not recast as passes.

**V03/F24 owner handoff:** The complete filesystem-effect guard still reports exactly the 155 pre-existing out-of-scope sites, byte-identical to the N06 baseline (SHA-256 `bd1a91edc4bd340b036b323bc1a175c2b43be7683a53faca284ed3aa8a5adbd9`). The separate TypeScript-Go transfer guard remains byte-identical to the exact-base 36 direct-read inventory sites plus one persistent-session failure (SHA-256 `b60ce79fe9f6c7af3126842550b97f6c5194f3e9505407379374b9736519ed89`). No C01 path adds a site. Driver Clippy still reports six warnings in untouched native/cache files. E02 may decide whether module analysis views should hide unrelated cached analysis while a global cycle exists; diagnostics are correct. The test-only repeated dependency ordering and a cosmetic blank line remain frontend cleanup suggestions. X02 Emitted Rust stays with active [PR #3946](https://github.com/sifr-lang/sifr/pull/3946) for final closure. The approved intermediate-item policy defers the full merge profile to Q01; this C01 delivery does not qualify that final gate.

## C02a0/F19 metadata producer and storage delivery receipt (2026-09-24)

[PR #4018](https://github.com/sifr-lang/sifr/pull/4018) merged as `a6bec1b1e54253fa9c8e9b6c7deea9f09fbe7968` from final candidate `80459cbdba33bb9e0fd126fdbe11ea1156689ef7` (base `a339472777a909e9942cd77209f800bdf9490eb5`, tree `1bd5ccec3f9d5979e824a72a909bcafc4b7b2a6b`). The PR merge-ref tree matched the candidate. `sifr_compiler_services` now owns source stdlib bootstrap, indexed metadata encoding/production, `PreparedMetadata`, and project metadata transport. `sifr_cache_storage` owns the secure entry lease, private-file, Unix bounded wait, Windows cancellable lease polling, and atomic-publication primitives. The driver retains CLI cache policy, cache root/owner decisions, pruning, its metadata reader, and `CompilerContext` as the sole provider/generation owner; no lower crate depends upward. Cancellation reaches the producer through an explicit callback. The Windows security helper module moved as one unit because its ACL, path, and rename routines share private implementation; only the metadata-required primitives are called by the lower producer.

On the final candidate, `cargo check -p sifr_cache_storage -p sifr_compiler_services -p sifr_driver --locked` passed. The four required exact driver producer cases, focused process cancellation/retry, lower exact stale/corrupt override, three lower Unix storage cases, three project metadata transport and two stdlib projection regressions all passed. Dependency direction, formatting, 900-line file-size guardrail, encoder generation, DX.5 inventory, documentation structure and diff checks passed. The lower Windows storage target with test-support compiled; the native Windows workflow's three named cache-storage companion cases passed 1/1 each, in [run 36037986485](https://github.com/sifr-lang/sifr/actions/runs/36037986485) on the exact candidate. Raw logs and SHA-256 digests are in `/data/sifr-architecture-c02a0-evidence-20260924/80459cbdba33bb9e0fd126fdbe11ea1156689ef7/validation.md` (SHA-256 `148beb2d14f6db89b5af9c47fdfe506eb04ea35da030f21861e92991f8a79066`). The full Linux-to-Windows cross check could not complete without MSVC `lib.exe`; native Windows execution supplied the platform assertions.

The first scoped Opus review found that moved Windows test helpers were hidden behind the lower crate's `cfg(test)` and that a newly introduced Windows idle timeout could reject live holders. Both were repaired with a lower test-support feature and restoration of the prior cancellable Windows poll loop. The lease/test-helper repair review returned **SATISFIED** (response SHA-256 `8cbd6ac93a12aa2aa4fd62744cd664fe7cb3e33ebbbc35e8583dfa1e60955d6c`). A documentation-only commit refreshed one DX.5 site line number. The first native Windows run then exposed a missing `Read` trait import in the remaining driver Windows cache reader before any storage assertions; it was repaired by restoring the base import. The focused final Opus review returned **SATISFIED**, with no blocking findings (response SHA-256 `182c0ad6b955a1f9d0977222c1c1e7452a885a20efca025c49a0c35ae64289d1`). The prior native failure is preserved (log SHA-256 `d28db3e8df841d0999c69130e97754a18a9b9b13951326c2f2fa10eacf120d42`). The first failed review is preserved (SHA-256 `46e1f4dfcc717ab20ae50706cbcaea8e4bd0b387bcc23853b4770094260b5081`).

The final candidate’s automatic PR create-pr profile stopped at V01 `performance_reference_admission` before Cargo because its runner did not select `SIFR_PERFORMANCE_REFERENCE` (job 107762760072, log SHA-256 `c6ee0c727130878d02b42eebb0866ae7c70bdf3a1058aef10a747f0deba598a4`). The native Windows job passed the storage step and later failed in out-of-scope `sifr_sql_postgresql`/`libpg_query` C compilation under MSVC because Unix headers `unistd.h` and `dirent.h` were unavailable; the same failure occurs on the main baseline. Its raw log SHA-256 is `cb88c0e8498d769a095cab1db62e909d3b977a5e5cff648bd7a46340d9d22a78`. The deterministic report signature job timed out with exit 143 after roughly 33 minutes without an assertion failure. These broad runner issues are not C02a0 test evidence; the approved intermediate-item policy defers the full merge profile to Q01.

C02a1 still owns metadata selection/provider transfer and removal of the driver's test-only `metadata_features` allowance. Lower test hook API narrowing and duplicated diagnostic helpers remain follow-up suggestions, not C02a0 blockers. A future Windows bounded wait would require a reliable live-owner signal and native live/idle tests; C02a0 preserves the existing Windows cancellation contract. No C02a1 or later implementation or whole-phase qualification is claimed.

## C02a1/F19 metadata consumer and stdlib service delivery receipt (2026-09-24)

[PR #4022](https://github.com/sifr-lang/sifr/pull/4022) merged as `73e2674c7c1ee333230212e6350efa892e64df37` from exact tested and reviewed candidate `50bc733f8d3fef59abe8b8a604d9cd93bc515767` (base `eafa57e22df22d6d8172adf8c65aa5c42aea8acb`, tree `4fb05b5f0c25ce6b5856df5c508ebd5a657180e4`). Main advanced after the candidate only through an unrelated documentation receipt before this merge; the merged code matches the reviewed candidate. `sifr_compiler_services` now owns metadata selection, decoding, provider, navigation and qualification, tooling sysroot views, `stdlib_external_defs`, `ApplicationProfile` and `CompilerContext`. Driver keeps delegating reexports and test modules. The existing provider/generation cache moved into the lower context as the sole cache, with explicit compiler identity, resolved sysroot and cache owner. The driver test-only `metadata_features` allowance was removed. C02a0's producer/storage direction and C01's compilation product remain intact; analysis/LSP direct-import removal remains C02e.

On Linux x86_64 with rustc/Cargo 1.98.1, `cargo check -p sifr_compiler_services -p sifr_driver -p sifr_analysis -p sifr_lsp --locked` passed. The exact required driver override retry, installed relocation/generation switch and navigation-demand cases passed; the exact analysis public-to-private stdlib definition case and the two new lower identity/cache-owner cases passed. Focused driver source metadata diagnostics/emission and LSP development sysroot request cases passed. Formatting, the 900-line file-size guardrail, decoder generation, DX.5 metadata inventory, documentation links, HIR maintainability and diff whitespace checks passed. Candidate-keyed raw logs, input identity and SHA-256 digests are in `/data/sifr-architecture-c02a1-evidence-20260924/50bc733f8d3fef59abe8b8a604d9cd93bc515767/validation.md` (SHA-256 `732894407ee678d2aa80ec5d74336b64102705cc44ee27d519f419fb390961c3`). Earlier candidate checks that exposed a stale DX.5 inventory and a missing test-only driver import remain historical failures, not passes.

The scoped read-only Opus review returned **SATISFIED** with no blocking findings. Its exact-candidate response is `/data/sifr-architecture-c02a1-evidence-20260924/50bc733f8d3fef59abe8b8a604d9cd93bc515767/review.md` (SHA-256 `32669d84903e0007392db82cab28d775c209836a11db830ea246fef338f64ae7`). Follow-up suggestions concern consolidation of the hand-written default cache-root policy and narrowing reader internals widened for retained driver tests. The lower test helper identity omits driver/package/component/SQL tokens formerly carried by the driver helper; production CLI/analysis/LSP identities still extend the driver token set. The DX.5 inventory refresh also includes pre-existing line-number drift from the Item 12R merge. These do not change C02a1 acceptance. The separate verification taxonomy classification of the C02a0 lower crates belongs to its registry owner. The approved intermediate-item policy defers the full merge profile to Q01; this receipt does not claim final integration readiness.

## C02a0/C02a1 F19 frontend-syntax guard follow-up receipt (2026-09-27)

[PR #4051](https://github.com/sifr-lang/sifr/pull/4051) merged as `1d900872b1a83b54bf61a2902b11ab53ddfb3848` from tested and reviewed candidate `3b321284d579c10eb2ade690336b81202210637b` (base `f31a5e35fc5b69166b4e67f5a2fdaf88e23142b5`, tree `f9530527cea66bad9da7e1f9dae2da80945bbb8b`). The C02a0/C02a1 service extraction had left the V03b/F25 split-brain guard allowances at obsolete driver paths. Its exact guard failed on the four relocated production calls at `sifr_compiler_services/src/metadata/locations.rs:78` and `src/stdlib/bootstrap.rs:36,580,583`. These remain, respectively, checked declaration source-range syntax inspection and the canonical stdlib compilation parse/public-private lowering sites. The repair moved only the two allowlist path keys to the lower service; all four exact call lines, reasons, counts, and guard boundaries stayed fixed. The self-test now checks the current files and rejects a duplicate of each allowed call. No emitted-Rust code changed.

On the exact candidate, `python3 verification/areas/performance/check_split_brain_guardrail.py` passed (raw SHA-256 `205c08c1932627bb13ef4d94e111953e6c71797eaaddb21b75e7a355003ff7ee`), its `--self-test` passed (`77113b399b7f25a8ee1c8347f875b1bfbde4173438313252a1ebe3bca34f1ada`), the 900-line file-size guard passed for 4,245 files (`2e10821206a8db09bbb2a00e21e9618f4f9f216f73b5c56afcd7ce13f9e819d2`), and the diff check passed. Candidate-keyed raw logs are under `/data/sifr-architecture-c02-syntax-guard-drift-evidence-20260927/3b321284d579c10eb2ade690336b81202210637b/`. The scoped Opus 5.5 review returned **SATISFIED** with no blocking findings (response SHA-256 `ea25c3962c1ab57728c7de4f2d8c37013084099cb006630e0c9a513b21e10d84`). It offered nonblocking suggestions about self-test fixture independence and retained driver test scaffolding; neither changes this guard repair. The full merge profile remains assigned to Q01 under the approved intermediate-item policy.

## C02b/F19 Python authoring service delivery receipt (2026-09-28)

[PR #4059](https://github.com/sifr-lang/sifr/pull/4059) merged as 421312d148d67cf285995e5109c6b2e9ce66e5ad from exact tested and reviewed candidate 9dcea0da6f58a859414004b1c94954ca61cd689f (base 17ad40716357ce636e5afca30d21eaffafcbbb88, tree 2ff85bea6abbbc4dd8774ab3a60c74543f947d18). The lower sifr_compiler_services::python capability now resolves frozen editor package/environment state from explicit package graph and probe inputs, applies certification and target-inspection rules, checks cancellation before probes, owns interop status policy, and converts package diagnostics. Driver retains build/CLI execution and generated-project adaptation; LSP retains document ownership and cache invalidation. Declared-symbol identity, library deferral, read-only checking, diagnostic codes and source spans remain intact. LSP fast-hit ordering and E01 external-generation watching were not changed.

On Linux x86_64 with rustc/Cargo 1.98.1, the three reserved lower exact cases and the three named LSP exact cases passed 1/1 each. The named driver read-only deferral assertion passed 1/1 using its actual nested test path, build::python_interop::tests::read_only_check_defers_library_target_without_an_environment; the planning table omits python_interop::tests from that selector. Focused lower certification tests passed 2/2 and driver Python interop tests passed 16/16. Exact LSP alias identity, diagnostic source scope, library deferral and uninspectable status, plus driver package-origin and uninspectable target tests, each passed 1/1. The pinned Python 3.14.7 fixture was prepared from uv.lock offline in the owned worktree. The first LSP run failed because the fixture interpreter was absent; its failure and diagnostic rerun remain preserved, and the affected test passed after fixture preparation.

Strict no-deps Clippy passed for lower services and LSP. Driver Clippy passed with only four lint categories from untouched native/cache paths allowed; strict attempts failed in untouched sifr_codegen, native storage, rust interop digest, cache storage and project-cache housekeeping files. Rustfmt, the 900-line file-size guardrail, HIR maintainability, documentation structure and committed diff checks passed. Candidate-keyed commands, fixture/submodule identities, raw results and hashes are at /data/sifr-architecture-c02b-evidence-20260928/9dcea0da6f58a859414004b1c94954ca61cd689f/9dcea0da6f58a859414004b1c94954ca61cd689f-validation.md (SHA-256 dd1f17bc42d0658987abfaebd6cd17d07e875568edee8f9c20da0658dfcaa4df). The scoped read-only Opus 5.5 review returned SATISFIED with no blocking findings; response at the same candidate-keyed directory's review.md has SHA-256 7dae2b51656b98104895629963ab74bfbab4eedbbd6c62d125a4400a743ce597. Its nonblocking suggestions concern a more direct lower digest-choice assertion, a future caller-supplied probe path, inert binding identity, and Debug on the public error type.

**V03/F24 owner handoff:** the direct filesystem-effect guard reports exactly five newly located lower-service sites and four stale LSP rows after the move (raw log SHA-256 2132f9c20d60971cddb568235245be227e3b4c964f6bbf6a553493acf7174491). V03 owns classification and retirement of those inventory rows; C02b did not change guard semantics or inventory. The strict Clippy failures remain with the touched subsystem owners, not this Python extraction. The prospective intermediate-item exception deferred the full merge profile to Q01; no C02c or whole-phase qualification is claimed here.

## V03/F24 C02b filesystem-effect inventory handoff delivery (2026-09-28)

The C02b handoff above merged in [PR #4061](https://github.com/sifr-lang/sifr/pull/4061)
as `4b16fe5bb33d1936a9edff8857b91274f630660d` from exact tested and
reviewed candidate `2c00a3db24758d487a944021c46f592de81831a5` (base
`ac248ddb32ee6df80cc9a63eebbcab14690a1aa4`, tree
`5685ee6d559cc2491994cf3c51c64a735830c3a4`). The inventory now follows
four Python authoring filesystem probes from LSP into the lower compiler service,
retaining their `semantic-input` classifications and exact site/operation keys
under the new paths and symbols. It classifies the new lower-service test fixture
write as `output-effect` and retires the four stale LSP rows. Only the checked-in
inventory changed; the scanner, guard semantics, Rust source and C02c scope did
not change.

On Linux x86_64, the exact `python3 verification/areas/developer_tooling/check_direct_filesystem_effects.py` check
passed for 3,014 sites (raw log SHA-256
`ffdd9cdf148f83daea2a72c1604cb0971a90f78077f75ae5edbc26cf2deb12da`),
and its exact `--self-test` passed (raw log SHA-256
`8fa490515a807532f428fbe8fbef1de198850a48156fd4e318782e22aada6c20`).
The 900-line file-size guardrail passed for 4,250 files (raw log SHA-256
`2b6bd1126230ab80146304fc103843ccd69352a5a9f2ea20dca7635738224ab5`),
and the diff check passed. The raw baseline failure matches the C02b handoff
SHA-256 `2132f9c20d60971cddb568235245be227e3b4c964f6bbf6a553493acf7174491`.
Scoped read-only Claude Opus 5.5 review returned **SATISFIED** with no blocking
findings (response SHA-256
`d203fc92cfd6c69c98ce5241c0553740166717e9c4a001a4da0481501274a25a`).
Candidate-keyed evidence is outside the Git tree at
`/data/sifr-architecture-v03-c02b-inventory-evidence-20260928/2c00a3db24758d487a944021c46f592de81831a5/`.
The review suggested a future stronger key for repeated multi-line call sites;
this is a pre-existing guard-design suggestion, not a defect in these rows.
The approved intermediate-item policy defers the full merge profile to Q01.

## C02c/F19 SQL editor service delivery receipt (2026-09-28)

[PR #4063](https://github.com/sifr-lang/sifr/pull/4063) merged as `f33c86741aacf11ce6d6fae3598090ef7561bb45` from exact tested and reviewed candidate `c88fdeaf5bb7ec38652f5be81d7d6f4cf8f779bf` (base `b6b3b2bcb935b501de301aee8b27b9d21f032f03`, tree `cd34750f1a14e1cabf6672ffd00da114d727ed2f`). `sifr_compiler_services::sql_editor` now owns locked editor profile discovery, package-graph profile/component preparation, `PreparedSqlProfiles`, and source-backed profile-name import diagnostics. Analysis calls the lower capability directly; driver reexports it for build orchestration and native materialization. The move retains exact schema/component identity, lockfile-less deferral, initialization diagnostic isolation, live overlays, and import spans. No SQL language semantics changed.

On Linux x86_64 with rustc/Cargo 1.98.1, the three reserved lower-service exact negative cases and the three named analysis exact cases passed 1/1 each. Focused analysis initialization-isolation and component-host-residency cases plus driver offline preparation, import span, import alias/shadowing, and ambient component-capability cases each passed 1/1. Locked compiler-services/driver/analysis/LSP compilation passed. Production strict Clippy passed for compiler services and analysis; driver Clippy passed with only four warning categories from untouched native/cache/rust-interop files allowed. Strict driver Clippy still reports six such pre-existing warnings, and an optional strict lower-service `--tests` Clippy run reports 39 pre-existing metadata/stdlib/context test warnings under those owners. Rustfmt, the 900-line file-size guard, HIR maintainability, compiler and analysis split-brain guards, documentation structure, and committed diff checks passed. Candidate-keyed commands, inputs, setup history, raw logs, and hashes are at `/data/sifr-architecture-c02c-evidence-20260928/c88fdeaf5bb7ec38652f5be81d7d6f4cf8f779bf/c88fdeaf5bb7ec38652f5be81d7d6f4cf8f779bf-validation.md` (SHA-256 `5d945ad7ebb5148a02541d8fbf27d85932aa99ba9de0df66dd757febec4d3657`). Scoped read-only Opus 5.5 review returned **SATISFIED** with no blocking findings (response SHA-256 `15b1e70b9f4a30cad9b9a889df60556149cfab25502980912867c1b630018050`). Its nonblocking suggestions for a shared source-range diagnostic helper, explicit future compiler-version input, and a narrower build-cache accessor remain deferred follow-ups.

**V03/F24 owner handoff:** the direct filesystem-effect inventory reports two production `is_file` sites relocated from driver to the lower SQL editor service, five new lower test fixture/read sites, and the two stale driver rows (raw log SHA-256 `36b6e5ba8cb6c3a1e3e06fea13ac27c843014831ea4f1f52340c4f60c66dfb2c`). V03 owns classification and retirement of these rows; C02c did not change scanner or inventory semantics. The automatic PR create-profile CI job stopped before Cargo at the already recorded V01 `performance_reference_admission` prerequisite because `SIFR_PERFORMANCE_REFERENCE` was not selected (raw job log SHA-256 `7529369028c9edba8ff578ef2af91568adcdd4d0fd0b5c1c390d43ee9b833f75`). Neither failure is C02c acceptance evidence. The approved intermediate-item policy defers the full merge profile to Q01; no C02d or whole-phase qualification is claimed here.

## V03/F24 C02c filesystem-effect inventory handoff delivery (2026-09-28)

The C02c/F19 handoff merged in [PR #4065](https://github.com/sifr-lang/sifr/pull/4065)
as `dee1201676e29c7482cca90dbf0a9980346f76bf` from exact tested and
reviewed candidate `a46dd00bd2d3e47461e225820931e64892a3d41a` (base
`290c0d2775d6cfc1e022942bdb04dcb2c3627630`, tree
`1c11b74977ff9740e994c1994dfe9e38d6763c7b`). The inventory now follows
the two production `is_file` source sites (three scanner observations) from
driver into `sifr_compiler_services::sql_editor`, retaining `build-identity`
classifications and their site/symbol keys. Four new lower-service test fixture
mutations are `output-effect`; the missing-lock assertion is `tooling-input`.
The stale driver rows were retired. Only the checked-in inventory changed; Rust
source, scanner behavior, SQL semantics and C02d scope did not change.

On Linux x86_64, the exact direct-filesystem-effects check passed for 3,019
sites (raw SHA-256
`176a01cbc1be4033f6fcf5a2639ca9689162b60aa150520f737cebdc13649d18`)
and its exact `--self-test` passed (raw SHA-256
`8fa490515a807532f428fbe8fbef1de198850a48156fd4e318782e22aada6c20`).
The 900-line file-size guardrail passed for 4,252 files, and the diff check
passed. The original C02c failure remained preserved with SHA-256
`36b6e5ba8cb6c3a1e3e06fea13ac27c843014831ea4f1f52340c4f60c66dfb2c`.
Scoped read-only Claude Opus 5.5 review returned **SATISFIED** with no findings
(response SHA-256
`04a4932de2b35ac0576efd5b18d9ce5e94c670905719207f5717a5da19d433a4`).
Candidate-keyed evidence is outside the Git tree at
`/data/sifr-architecture-v03-c02c-inventory-evidence-20260928/a46dd00bd2d3e47461e225820931e64892a3d41a/`
(validation manifest SHA-256
`79a374428ac085caa92b140e40396c3e56a7e355ad8e973d98d9eca4d519477f`).
The approved intermediate-item policy defers the full merge profile to Q01.

## C02d/F19 preview and editor-check service delivery receipt (2026-09-28)

[PR #4067](https://github.com/sifr-lang/sifr/pull/4067) merged as `16bee0cd8028f797ef256630510637ff2cae89c1` from exact tested and reviewed candidate `66fad5cc1561406b762be1371ea309c0a1a51012` (base `8c20ab737d38cd918c3e6393aab635f8c0525b47`, tree `28e7d6ccfb2ff457739806b471239e8e9aa6cc55`). `sifr_compiler_services::editor` now owns generated Rust preview and read-only saved editor-check restore. Analysis calls that lower service directly rather than driver compilation or `project_cache`. Preview compiles the C01 frontend product with the same SQL query finalization, stdlib materialization and generated source-map projection used by driver, and rejects direct single-file Rust interop without package Cargo context before exposing source. The driver retains build/test orchestration and project-cache publication. Restore takes the compiler's identity, pinned metadata and cache root, verifies namespace ownership, leases, manifests and record hashes, then lets the frontend recheck disk observations and editor overlays before reuse.

On Linux x86_64 with rustc/Cargo 1.98.1, all four named existing exact cases and all three reserved lower-service exact negative cases passed 1/1 each. Focused preview/driver product equivalence, direct Rust interop failure parity, driver live restore/cache-owner/source rejection, SQL query artifact and portable-schema product cases also passed 1/1 each. The touched `sifr_cache_storage` crate passed an `x86_64-pc-windows-msvc` cross-target check. Rustfmt, the 900-line file-size guardrail, HIR maintainability, analysis split-brain and diff checks passed. Exact commands, candidate-keyed raw logs and hashes are at `/data/sifr-architecture-c02d-evidence-20260928/66fad5cc1561406b762be1371ea309c0a1a51012/validation.md` (SHA-256 `695daa987ed617aa34b3b2ead2983129b1fe4f34e411916359b81fd90acd2697`). The first scoped Opus 5.5 review found the direct Rust interop failure mismatch; its response (SHA-256 `8c1d68fc62765c99f76761bee65b24d62d76d93d1eaec47bef59affc4e807bf0`) and the repaired candidate's second read-only **SATISFIED** review (SHA-256 `29311b85782706028ddce1141887605d5f2eca2601c565ab0d537d992d92bd58`) are preserved outside the reviewed tree. Earlier test-import and Windows-import failures remain historical failed evidence. The broader Windows services cross-target probe could not run without this Linux host's missing MSVC `lib.exe`; it is not qualification evidence. The approved intermediate-item policy defers the full merge profile to Q01.

Separate follow-up work from review: share the driver's single-file metadata-resolution path with the lower preview so future sysroot-demand failures cannot drift; share saved-check reader schema, limits and lock-path checks with the driver publisher; extend corrupt-record and changed-metadata restore tests through the full service; and align custom cache-root pruning under the driver's cache-policy owner. These are not C02d completion claims or C02e dependency-guard work.

## V03/F24 C02d filesystem-effect inventory handoff (2026-09-28)

The direct filesystem-effect inventory on the exact C02d candidate reports 22 new unclassified observations from the relocated read-only saved-check reader, lower storage primitives and test fixtures (raw log `/data/sifr-architecture-c02d-evidence-20260928/66fad5cc1561406b762be1371ea309c0a1a51012/v03_inventory_handoff.log`, SHA-256 `0d5f49d8a51b51e296c6cf5a5857757adfb9588c55f8eaa9bcc09490d1d5e54b`). V03 owns classification and any inventory repair in a separate item; C02d did not change the scanner or inventory. The failed probe is preserved as failed evidence, not a C02d pass or a whole-phase gate result.

## V03/F24 C02d filesystem-effect inventory handoff delivery (2026-09-28)

The C02d handoff merged in [PR #4069](https://github.com/sifr-lang/sifr/pull/4069)
as `b66ac270f0745085c871eaca51f16d794a244ed8` from exact tested and
reviewed candidate `9785f0903c64d4a233a27a3b82ce946e693dde14` (base
`50e3efb6cad2b135b09ef8d109db2fb80fd1cfb2`). The inventory adds 23
rows for the 22 distinct unclassified site keys in the failed handoff probe;
the saved-check `read` metadata call contributes two scanner observations on
one line. No existing row was retired or changed. Cache payload and ownership
probes, lease metadata, and saved-check workspace identity are `build-identity`;
the two writable lease opens and fixture mutations are `output-effect`.
Site/symbol keys and scanner behavior are unchanged. No Rust implementation,
C02e boundary guard, or separate saved-check follow-up changed.

On Linux x86_64, the exact direct-filesystem-effects check passed for 3,042
sites (raw log SHA-256
`9f348a55966d929fee76abe4a507a1e2baf14dc628816a817eadde3ace3d09c2`),
and its negative `--self-test` passed (raw log SHA-256
`8fa490515a807532f428fbe8fbef1de198850a48156fd4e318782e22aada6c20`).
The 900-line file-size guardrail passed for 4,256 files (raw log SHA-256
`9dd0c274eab36227042a9f9ea2391ea0e47018a05c1f17e935d9e8d28c18f7d0`),
documentation structure passed after the pinned nested editor submodule was
initialized (raw log SHA-256
`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`),
and the committed diff check passed. The two earlier documentation setup
failures are retained separately (each raw log SHA-256
`93cfeff10def32526e57d73538d0f8d27a4b21fba4fa760108cb911fe2b9d3e9`).
Scoped read-only Claude Opus 5.5 review returned **SATISFIED** with no blocking
findings (response SHA-256
`1c1c9f19467e37fa56557e1dca1c3292459b65b3da1199b8b91736c014da82c6`).
Candidate-keyed raw logs and review response are outside the Git tree at
`/data/sifr-architecture-v03-c02d-inventory-evidence-20260928/9785f0903c64d4a233a27a3b82ce946e693dde14/`.
The approved intermediate-item policy defers the full merge profile to Q01.

## C02e/F19 dependency-direction guard delivery receipt (2026-09-28)

[PR #4071](https://github.com/sifr-lang/sifr/pull/4071) merged as
`69e417224a721906005f9ecad50bb2b0b3b14602` from exact tested and reviewed
candidate `2964a824e4e4e9272c37b4028570ecdb198c37bb` (base
`d3d3386da5a0087857f137c98a3f12c364e7b08b`, tree
`696521c27e7e750264004aafc27f0be365efcdbd`). Analysis and LSP now
take `CompilerContext`, stdlib/tooling services, Python interop plans and
startup identity directly from `sifr_compiler_services`; neither crate
declares or references `sifr_driver`. The lower service's compiled input
tokens cover the package, component and SQL inputs formerly inherited through
driver. Driver remains the CLI/build orchestration adapter. `Cargo.lock`
drops only the two removed dependency edges.

`check_source_crate_dependency_direction.py` now resolves Cargo package
identity for direct and workspace-inherited aliases, checks normal, build and
dev dependencies under top-level and target-specific sections, including
optional feature dependencies, and scans manifest-declared lib, bin, example,
test, bench and build-script Rust sources outside `src/`. Analysis and LSP
reject driver dependencies and source aliases in all configurations;
`sifr_compiler_services` and `sifr_cache_storage` reject upward edges. The
positive fixture, current tree and five independently named negative mutations
passed. The extra workspace-rename and lower-service negative cases passed.

On Linux x86_64 with rustc/Cargo 1.98.1, all three C02e named exact Cargo
cases passed 1/1 each. Five focused exact regressions for generated preview
source maps and product projection, Python plan file identity and LSP compiler
context identity passed 1/1 each. Rustfmt, the 900-line file-size guardrail,
HIR maintainability and diff checks passed. Candidate-keyed raw commands,
logs and hashes are outside the Git tree at
`/data/sifr-architecture-c02e-evidence-20260928/2964a824e4e4e9272c37b4028570ecdb198c37bb/validation.md`
(SHA-256 `35fa9498a39cabc39e5d6ae1f63b7162c42aef88c237e387c1e7c40263957a2d`).
The first LSP case failed only because this fresh worktree lacked the pinned
Python interop environment; after offline fixture preparation, the same exact
case passed without code changes. Both logs remain in the evidence directory.

The scoped read-only Opus 5.5 review returned **SATISFIED** with no blocking
findings (response SHA-256
`bf298cbf028b9594aa23ce6d728a9b60c6b428e8a3473684e9cd0d358bec8d1e`).
Follow-up guard hardening for Cargo's auto-discovered target files and
out-of-crate `#[path]` includes remains separate; no guarded crate currently
uses those auto-discovered target directories. The automatic create-PR CI job
failed at V01 performance-reference admission before C02e validation because
`SIFR_PERFORMANCE_REFERENCE` was absent. Its failure remains V01 evidence.
The approved intermediate-item policy defers the full merge profile to Q01;
this receipt does not claim whole-phase integration or review. The record-only
update passed documentation structure and diff checks.

## N06/F13-F14 and DX9-F5 generated-storage delivery receipt (2026-09-24)

[PR #4012](https://github.com/sifr-lang/sifr/pull/4012) merged as `7069cb8826efce2c78be4e1a60e96472f785bd5e` from exact tested and reviewed candidate `b7cb0064f195dcece63190644b4ec73f93ba0707` (base `ab53fba76f62398e9604e79b3c82dd27629315d7`, tree `ddece952e76d0fbf8538780944b3d9acbd1cc4c5`). The DX.3 cache inspection names the generated-artifact, prepared-resolution, probe-receipt, native-family, publication, metadata, project and fixture owners. Generic pressure pruning is scoped to the current canonical worktree and an uncontended permanent entry lease. It handles owned abandoned staging and inactive whole-family or prepared-resolution roots, protects unknown and orphan auxiliary roots without an ownership contract, and never removes another session's target or a live ownership lock. The cache CLI exposes the inspection and scoped pressure controls.

Unix generated-storage waits are cancellable and bounded by a 30-second idle deadline and a 41-minute hard deadline. Exclusive owners renew the diagnostic note while holding the lease; waiters reset the idle deadline on observed renewal. Timeout reports the lock path and last owner note from the held descriptor. The killed-parent/live-child case leaves an inherited lease intact; a wedged sibling eventually times out without breaking its lock. Prepared Cargo resolution now takes a shared lease for valid reuse and upgrades on a miss. Windows native storage and process behavior remains with its separate owner.

On Linux x86_64 with rustc/Cargo 1.98.1, exact focused selections passed: `sifr_driver --lib build::workspace::tests` 9/9, `build::cargo_resolution` 21/21, `build::native_reuse_tests` 7/7, `metadata_producer::tests` 12/12 with one intentional ignored, `reusable_scenario_preserves_identity_and_resets_authority` 1/1, and `sifr --bin sifr cache_cli_contract` 1/1. The storage suite includes wedged sibling, killed-parent/live-child, abandoned stage, orphan auxiliary root, live renewing holder, and scoped disk-pressure cases. Formatting, the 900-line file-size guardrail, driver and lowering maintainability, filesystem checker self-test, documentation structure and diff checks passed. The candidate-keyed validation manifest is `/data/sifr-architecture-n06-f13-f14-evidence-20260924/b7cb0064f195dcece63190644b4ec73f93ba0707-validation.md` (SHA-256 `28a7c5f976e35113f455e8ce1c7a8be4cc1274f4144b0918bb2cc341f6ea19d2`). `Cargo.lock` blob `935d5af418fcf3ce3292cfb1408985e54f3e1378` and Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1` were unchanged. The owned warm Cargo target stayed on `/data`; no shared or live target was cleaned. Failed setup and timing-sensitive test attempts remain historical failures, not pass evidence.

The first read-only scoped Opus review of `b064f9c848fc97cd2325c5ac114d0bccba1fc6c0` returned **NOT SATISFIED** because a fixed 30-second wait could reject healthy cold Cargo holders (response SHA-256 `d6e23c377f93d8378b4ecbf22ccdee010fdea4f7e8611cc80f3235c33e5234d5`). The final remediation review returned **SATISFIED** with no blockers; its response is `/data/sifr-architecture-n06-f13-f14-evidence-20260924/b7cb0064f195dcece63190644b4ec73f93ba0707-review.md` (SHA-256 `c664df4252384656f0accef60a3ad35ce6bda6246534348e96a1d22b89fe8944`). Follow-up suggestions concern shared-holder renewal, families spanning several Cargo subprocesses, renewal during unusually long prune operations, renewal error reporting and diagnostic note write atomicity. Prior suggestions about signal-cancellation state and a missing-lock race remain separate; legacy v9 prepared roots remain protected pending their lifecycle owner. These do not change the accepted N06/DX9-F5 boundary.

**V03/F24 owner handoff:** N06 changed-path filesystem-effect sites were reconciled. The complete guard still reports exactly the 155 pre-existing out-of-scope stale/unclassified sites, byte-identical to the N04/N05 baseline (`/data/sifr-architecture-n06-f13-f14-evidence-20260924/v03-repair.log`, SHA-256 `bd1a91edc4bd340b036b323bc1a175c2b43be7683a53faca284ed3aa8a5adbd9`). It is not a passing global guard. The intermediate-phase policy defers the full merge profile to Q01; this item and its automatic CI attempt do not qualify that final integration gate. The record-only update passed documentation structure and diff checks.

## N05/F12 CLI formatter-cache delivery receipt (2026-09-24)

[PR #4010](https://github.com/sifr-lang/sifr/pull/4010) merged as `a3b7529c46ef578c96f43a58f7c9d348a2b60e33` from exact tested and reviewed candidate `f41eafcd169a0e0529bf78fc27c33438222159d9` (base `74042d66dbd0f4d4a3def873a7e15bc9375b088b`, tree `e40b0c7ad08f036a8db9229916cabd3bec4d9292`). The old existence-only marker is replaced by a versioned, domain-framed identity covering the compiler build (including formatter and Ruff inputs), normalized path, source bytes and all effective formatter options. Cache reads require an exact owner/schema/identity payload in a regular file. Publication writes and syncs a temporary marker in the cache directory before atomically persisting it. The CLI binds publication to the formatter's returned output bytes, so an intervening disk edit cannot be stamped as formatted. This closes N05/F12 only; N06, DX9-F1 and V03 global guard work remain separately owned.

On Linux x86_64 with rustc/Cargo 1.98.1, the exact `sifr --bin sifr formatter_cache::tests` selection passed 6/6, including unchanged source across a revision, malformed/foreign markers, partial/failing publication and a post-format disk edit. The `sifr_format --lib` suite passed 11/11, two exact existing formatter CLI cases passed 2/2, and a real CLI smoke check passed for formatting, corrupt marker repair and source-edit reformatting. Formatting, HIR maintainability, the 900-line file-size guardrail, filesystem-effect self-test and diff checks passed. The candidate-keyed validation manifest is `/data/sifr-architecture-n05-f12-evidence-20260924/f41eafcd169a0e0529bf78fc27c33438222159d9-validation.md` (SHA-256 `20ca4c418b8d18782b5b1997e1f16796caf2e2b2722f3f5b8c95848d3f8f2f80`). `Cargo.lock` and the Ruff gitlink were unchanged. The worktree-owned warm target was retained; no cleanup occurred. The initial uninitialized-submodule and test-only moved-value attempts remain failed setup evidence, not passes.

The first read-only scoped Opus review of candidate `45ec91141d3641d5d96416adf4d253ca298b0a89` returned **SATISFIED** with no blockers (response SHA-256 `60e692c9f279707ce58e5e637bb20e81de72c1997268aad397d8c13339262222`), but identified the pre-existing post-format reread race. The final candidate corrected that race and reran affected checks. The second completed read-only review returned **SATISFIED** with no blockers or new mechanism-level defect; its response is `/data/sifr-architecture-n05-f12-evidence-20260924/claude-final-retry.qkhDEZ/response.md` (SHA-256 `30ee3e65090b7af6e94c130cc2cc551ec2eef81a799de0d33be0d9b9d86350e9`). An earlier second-review invocation left only a temporary output and was not counted as approval. Review suggestions for persist-step fault injection and an explicit truncated-marker test are deferred formatter-cache test work; marker pruning belongs to future cache-storage ownership, and formatter idempotence remains a separate formatter behavior question, with no observed failure here.

**V03/F24 owner handoff:** The complete filesystem-effect inventory still reports the same 155 out-of-scope stale/unclassified sites recorded by N04, with no N05 site. The N05 raw guard log at `/data/sifr-architecture-n05-f12-evidence-20260924/filesystem-guard-remediation.log` has SHA-256 `bd1a91edc4bd340b036b323bc1a175c2b43be7683a53faca284ed3aa8a5adbd9`, byte-identical to the N04 baseline. It is not a passing global guard. The prospective intermediate-phase policy defers the full merge profile to Q01; neither N05 nor the automatic CI attempt qualifies the final integration gate. This record-only update passed documentation structure and diff checks (raw documentation log SHA-256 `d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`).

## N04/F10-F11 Rust interop probe-input delivery receipt (2026-09-24)

[PR #4008](https://github.com/sifr-lang/sifr/pull/4008) merged as `77b6cb1218c5078f1432bc5d7b676a8237402c96` from exact tested and reviewed candidate `a5c8c0f3fb46a2177c531a7df9e2b548f85f2906` (base `3b1923cb7bbf912e7306527a90c05623b246ddf7`, tree `e1dfef15a8f7e802680117c814d3cf4e105a455e`). Required bridge and sysroot roots now fail on missing, unreadable or unsupported members. Optional SQLx metadata is absent only on NotFound; an empty directory, replacement file and read error are distinct. The walker includes empty directories, orders members deterministically, rejects links at the selected root or inside it, and allows linked ancestors that resolve the selected path (including macOS `/var`). The direct probe requires a Cargo manifest in every lock mode, recomputes mutable backend, runtime and SQLx input identities for each probe, and revalidates the key after Cargo. This closes N04/F10-F11 under the stated revalidation policy; direct probes still run Cargo and DX9-F1 retains unused receipt disposition.

On Linux x86_64 with rustc/Cargo 1.98.1, the exact `sifr_driver --lib build::rust_interop_digest::tests` suite passed 5/5; the exact same-process probe digest case, two exact SQLx metadata cases and the exact direct Cargo probe case with `--include-ignored` each passed 1/1. Formatting, the 900-line guardrail, filesystem-effect self-test, 59 N04-scoped inventory sites and diff check passed. The raw final validation log is outside the reviewed tree at `/home/yaser5/projects/sifr/architecture-n04-f10-f11-evidence-20260924/final-validation-a5c8c0f3.log` (SHA-256 `9cc1f008a456c47a861f1cb8929c8c53442ba476499b62962c9454688c703db0`). `Cargo.lock` SHA-256 was `8c34bb1735240e991ba00e82cf35e21c13d43457d1df1707004e4e29553a6c62`; its tracked blob `935d5af418fcf3ce3292cfb1408985e54f3e1378`, Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1`, and Cargo config blob `be538503060ed90da17949bb25d48a27e0b69178` were unchanged across the implementation diff. The worktree-owned warm Cargo target was retained; no cleanup occurred. The first focused attempt failed because its optional-root replacement assertion expected a digest instead of an error; it was corrected before the final candidate and is not counted as a pass.

The first read-only scoped Opus review of candidate `1cc3aadc4e66786d45ef5635def04bf2b32bb9a3` returned **SATISFIED** with no blockers (response SHA-256 `d413a2bad8092e069088a05cd0086d680fe0de5d7a61f989e25a1b71ed588cc4`), but identified the linked-ancestor macOS path issue that prompted the final adjustment. The final read-only scoped review returned **SATISFIED** with no blockers (response SHA-256 `e6b2695b6a55e1a42f16f20d1fbe7279ae14b246ef2fd48ba540e2fb6be61525`); both responses are keyed by candidate SHA under the evidence directory. Follow-up suggestions concern a transient member-swap race, confirmation of other `rust_interop_cargo_inputs.rs` consumers' post-Cargo boundaries, and root-run denial-test coverage. They do not extend this item's accepted boundary.

**V03/F24 owner handoff:** The whole current-main filesystem-effect inventory check still fails on 155 stale/unclassified sites outside N04 (notably Windows storage, native storage and project-cache paths). Its raw final log is `/home/yaser5/projects/sifr/architecture-n04-f10-f11-evidence-20260924/global-filesystem-guard-a5c8c0f3.log` (SHA-256 `bd1a91edc4bd340b036b323bc1a175c2b43be7683a53faca284ed3aa8a5adbd9`). The 59 N04 sites passed separately. V03 and each changed-path owner retain that inventory drift; N04 did not change their code or classify their sites. The intermediate-phase policy defers the full merge profile to Q01; this receipt does not claim N05, N06 or Q01. This docs-only record passed documentation structure and diff checks after the pinned editor integration submodule was initialized (raw log SHA-256 `d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`); the preceding missing-submodule setup failure is retained as a failure, not a pass.

## N03/F09 declared native environment identity delivery receipt (2026-09-23)

[PR #3966](https://github.com/sifr-lang/sifr/pull/3966) merged as `87229349db4b1294829e8366c2010236d86c9a47` from exact tested and reviewed candidate `da46f3f726b572aeb8a5600dca45d4a9da928c34` (base `d6684e2ffa6f6ad95febf3ca5498cb7642040da0`, tree `1dbde12eb53b873cf156d59390bdd8abbaffb796`). The declared native environment identity now uses a v2 domain and separately encodes each sorted declared name, a presence byte and raw value bytes. The actual `NativeBuildContext` identity differs for a declared variable that is absent, empty or the literal `<unset>`. The production context captures the same environment snapshot passed to Cargo. This receipt does not claim that the earlier Python model proved a stale binary, or claim N04-N06.

On Linux x86_64 with rustc/Cargo 1.98.1, the exact `sifr_sysroot --lib native_context_tests::declared_environment_absent_empty_and_literal_unset_have_distinct_build_ids` case passed 1/1 (raw log SHA-256 `9be001138f9f6e31604059b922b2f8a3e09a454b9d245695bc08e2e49c98ad7d`), and the focused `native_context_tests::` suite passed 8/8 (`743146fac64fcdc4c0b2cde39098cda0ac1f08d780ddfbccb00ece6c0b8723ff`). Formatting, diff check and the 900-line file-size guardrail passed (`8fadc3303178a95d046220f2bb32dad44a9355d917156d504827a7f8719d04d3`). Tracked `Cargo.lock` blob `e54d6c74360356d397fb5c3a99e11cf501cbc004` and Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1` were unchanged. Raw validation and review are outside the reviewed tree under `/home/yaser5/projects/sifr/architecture-n03-f09-evidence/`. The initial Cargo attempt stopped before compilation because Ruff was not initialized in this new worktree; the pinned submodule was initialized and the selected tests then passed. The worktree-owned Cargo target stayed warm and was not cleaned.

The read-only scoped Opus review returned **SATISFIED** with no blocking findings (response SHA-256 `8e44b03ec90c6ed5d0b7145953c053d1bdd21b3f760366cd1eafdcb4527e1a0d`). It noted an optional future assertion of the Cargo command environment, and a separate pre-existing absent/empty `python_loader_id` encoding whose values are currently nonempty digests. Neither changes N03 acceptance. The intermediate-phase policy defers the full merge profile to Q01. The docs-only receipt passed documentation structure after initializing the pinned nested editor submodule (raw log SHA-256 `d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`); the two earlier missing-submodule setup failures remain preserved (raw log SHA-256 `0009f921e9f856a5f47630c37523d073b7eae89e0a8b60b8403782cb033aee93`).

## N02c/F08 package and driver serialization delivery receipt (2026-09-23)

[PR #3964](https://github.com/sifr-lang/sifr/pull/3964) merged as `93d06c4484ad8704322ee631ca4bd0a2c15b155f` from published final candidate `a1878920b3746c526538af8ae9cd353ece459f76` (base `45a96258b815f32939028f87f52997e6ebaeaf47`, tree `e2106a19f1ed178692dbee96c8388d285884be2b`). The published tree matches local tested candidate `015795a7615c18e9c810f87e7e0e658a70c16fff`. This branch selectively reconciled the package/driver serialization changes from unmerged draft #3956 after the separately merged N02c0 trace handoff; #3956 and its failed second review remain historical evidence, not merge approval. Package graph, source-map, build-input, Cargo metadata and Python probe digests now propagate serialization errors. CLI, LSP and driver consumers return an error or disable optional reuse without hashing empty bytes or publishing success. Saved-check reporting distinguishes unavailable package identity or policy, record serialization and write failure. The project store publishes the same serialized record bytes used by the driver report. A non-UTF-8 record-path regression verifies that failed serialization publishes no cache entry. F07 identity behavior and Cargo-before-final-cache ordering remain intact.

On Linux x86_64 with rustc/Cargo 1.98.1, seven exact `sifr_package --lib` cases and nine exact `sifr_driver --lib` cases passed. The package selection was reused after an unrelated N02b Clippy base update and a driver-test-only addition because its implementation, tests, lockfile, Ruff pin and compiler were unchanged; the eight rebased driver selections and the final new serialization case ran separately. Strict `cargo clippy --locked --offline -p sifr_package -p sifr_driver -p sifr -p sifr_lsp --lib -- -D warnings` passed. The 900-line file-size, HIR/driver maintainability, direct-filesystem-effects, formatting and diff checks passed. Tracked `Cargo.lock` blob `e54d6c74360356d397fb5c3a99e11cf501cbc004` and Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1` were unchanged. The validation manifest outside the reviewed tree is `/home/yaser5/projects/sifr/architecture-n02c-f08-rescope-evidence/e2106a19f1ed178692dbee96c8388d285884be2b-validation.md` (SHA-256 `be4dc67008c60daec94f931efba0db1c6d2cfa8bb13ee393269b6037d8f5f8c5`), with raw log digests and preserved setup failures. The inactive compatible Cargo target was reused; no cleanup occurred. The prospective intermediate-phase policy defers the full merge profile to Q01.

The read-only scoped Opus review returned **SATISFIED** with no blockers. Its response is `/home/yaser5/projects/sifr/architecture-n02c-f08-rescope-evidence/a1878920b3746c526538af8ae9cd353ece459f76-review.md` (SHA-256 `3cbc88e2b21e94c15f8544c6cb50dbc7213c99365613dc89e8d42f8ee83e29cf`). Nonblocking follow-ups are a more specific outer source-snapshot serialization diagnostic, atomic application of LSP Arrow/DLPack certifications after a second setter failure, and separate treatment of pre-existing corrupt hint/owner deserialization from absence. This receipt closes N02c/F08; it does not claim N03-N06 or final Q01 qualification.

## N02c0/F08 CLI trace-consumer handoff (2026-09-23)

The unmerged [N02c draft PR #3956](https://github.com/sifr-lang/sifr/pull/3956) at published candidate `20f41e01fe1222ff58e04589667b5cd909667767` remains historical failed review evidence. Its second scoped Opus review returned **NOT SATISFIED**: the CLI `ProjectCacheReport.status` trace allowlist omitted four new fixed literals, causing all four to become `unrecognized-redacted`. The response is preserved outside the reviewed tree at `/home/yaser5/projects/sifr/architecture-n02c-serialization-evidence/20f41e01fe1222ff58e04589667b5cd909667767-review-remediation.md` (SHA-256 `94e3c3aae2771396ae96aeb39a3243eb5ad1b7aa603c8e3c16359b61b39314db`). N02c0 owns only the CLI trace-consumer contract for `identity-unavailable`, `policy-unavailable`, `serialization-unavailable`, and `interface-restored-serialization-unavailable`, plus redaction tests. The package/driver serialization implementation remains N02c, subject to its own tests and review after this handoff. Do not merge #3956 or reinterpret its failed review as approval.

[PR #3960](https://github.com/sifr-lang/sifr/pull/3960) merged as `094971d3011f76749cd6dcc96234972ac099b628` from exact tested and reviewed candidate `eff3079ba5201bf1e8bd64a3b94c924dba358a46` (base `ca093afb2c745633b18f5b0b503bf8b73121d08f`, tree `4b47bd4a9e6dedf7a49d5b60cedb80dd08c00d68`). The CLI trace allowlist now preserves the four fixed literals above and still redacts unknown or sensitive values. It does not contain the package/driver serialization changes from #3956; N02c/F08 remains open. Main advanced before merge only in an unrelated schema-first SQL issue document, so the implementation and validation inputs remained unchanged.

On Linux x86_64 with rustc/Cargo 1.98.1, the exact `sifr --bin sifr` new-status selection passed 2/2 and the focused `trace_artifacts::tests::` suite passed 5/5. The validation manifest outside the reviewed tree is `/home/yaser5/projects/sifr/architecture-n02c0-cli-trace-evidence/eff3079ba5201bf1e8bd64a3b94c924dba358a46-validation.md` (SHA-256 `7062e2d7219a50858e107304a9dfa731e6654989ace0054f4217807df2a5756c`); it identifies the raw logs, failed cold-worktree setup attempts, configuration and file-size/documentation checks. Formatting and diff checks passed; Cargo.lock and Ruff gitlink were unchanged. No target cleanup occurred. The read-only scoped Opus review returned **SATISFIED** with no blockers; its response is `/home/yaser5/projects/sifr/architecture-n02c0-cli-trace-evidence/eff3079ba5201bf1e8bd64a3b94c924dba358a46-review.md` (SHA-256 `b7f4aa75f29e42273887ca09f72dfbd68baa858a248a06b856b417812b68d5e2`). Its nonblocking suggestions concerned a redundant test assertion, a possible future sink-level test and pre-existing macro indentation. The intermediate-phase policy defers the full merge profile to Q01. This handoff ends here; N02c requires a separate assignment.

## N02b/N02b2 strict driver Clippy blocker repair (2026-09-23)

[PR #3962](https://github.com/sifr-lang/sifr/pull/3962) merged as `8a76f3c867131c5af49979cc2887abe3f1ebbd3f` from exact tested and reviewed candidate `372ef8a0bdcd19a688e073f3bed2735fc96bd048` (base `a3d5be6e5ed1cffdb4a8eb55577b82dc592260e6`, tree `62347a100912b60e079395ae82931c9e682b178c`). It repairs two strict `sifr_driver --lib` Clippy diagnostics introduced by N02b and N02b2 that blocked Emitted Rust Item 12R. The normalized manifest helper import is now test-only. SQLx metadata uses a vacant map entry, preserving a fallible digest only for a previously unseen root, unchanged error reporting, and existing cache identity encoding. This repair does not qualify Item 12R or any N02c work.

On Linux x86_64 with rustc/Cargo 1.98.1, `cargo clippy --locked -p sifr_driver --lib -- -D warnings` passed (raw SHA-256 `241294bc1b616c8a799e5f3846e59b641b3d8280b3b6a04776549a5a8652e28b`). Three exact `sifr_driver --lib` cases each selected and passed one assertion: prepared resolution manifest identity (`2ba287c172d172d2040552e00816a22eab2a8b04b01d4408bf1e459bdb33e523`), SQLx metadata cache participation (`25ffdafd3007d6e03665d71052b73630d9155968fa16a4e6589f0bb8b214fced`), and absent, empty, unreadable and repaired SQLx metadata identity (`32ba7d5e62c34778c1223baf1b55e6fdb70e1651f8e9f0b42993d84b3b9ac781`). Formatting, 900-line file-size guardrail (4,134 files), HIR and driver maintainability, and diff checks passed. Tracked `Cargo.lock` blob `e54d6c74360356d397fb5c3a99e11cf501cbc004` and Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1` were unchanged. The candidate-keyed validation manifest is at `/home/yaser5/projects/sifr/architecture-n02b-clippy-evidence/372ef8a0bdcd19a688e073f3bed2735fc96bd048-validation.md` (SHA-256 `a761b1d7f4c73d43621f140eafccc33e54e48196781815a3ce4245444703d735`), outside the reviewed tree. An initial Clippy setup attempt raced Ruff checkout and failed before compilation; the later completed-checkout run passed. Its first log path was reused, so the initial failure survives in the task transcript rather than a separate raw file.

Scoped read-only Opus 5.5 review returned **SATISFIED** with no blocking findings (response SHA-256 `538d9814a8badf312c343c5cb1285e758d38313e7a9abd8e858ade519980a270`, candidate-keyed outside the Git tree). Its optional style suggestion was deferred; the review's pending test note is discharged by the exact passes above. The full merge profile remains with Q01; Item 12R must rerun its own final strict validation on its own candidate. No blocker remains for this bounded repair.

## N02b2/F07 prepared and unchanged Cargo lock identity delivery receipt (2026-09-23)

[PR #3954](https://github.com/sifr-lang/sifr/pull/3954) merged as `894d5e42410ea45bcee3be3c97ae67e519d6156f` from the exact tested and reviewed candidate `ef9c304c75582c0dd0b10addf7f83f18539b0f87` (base `bf4ac918b6138c2c1b16f368707cfb611d942758`, tree `22a99f3812f6b252f9049a365f1dcf5fd4da36b0`). The normal authority seed, prepared resolution key and all remaining driver lock file components use versioned, domain-framed, length-framed SHA-256 identities. Prepared keys bind lock and vendor modes, toolchain, normalized manifest, ordered Cargo arguments and lock authorities, plus the path and current manifest/checksum inventory of each trusted vendor root. Package, sysroot and probe lock identities bind path and payload. Required lock or vendor authority failures now stop preparation or reuse; optional package locks distinguish absent, empty, changed and unreadable. Constrained-mode post-Cargo checks revalidate authority and vendor payload as well as the generated lock. Normal seeding and reconciliation, Cargo-before-final-cache behavior and DX9-F7 alternation are preserved. N02c/F08 serialization, N03 environment encoding and N04 probe inputs are not claimed.

On Linux x86_64 with rustc/Cargo 1.98.1, exact `sifr_driver --lib` selections passed: cargo resolution 21/21, Cargo inputs 8/8, lock digest 2/2, probe cache 5/5, and DX9-F7 alternating dependency 1/1. The direct-filesystem-effects guard passed 2,816 sites, the 900-line file-size guard passed 4,134 files, and formatting, HIR maintainability and diff checks passed. The candidate-SHA-keyed validation manifest `/home/yaser5/projects/sifr/architecture-n02b2-lock-identity-evidence/ef9c304c75582c0dd0b10addf7f83f18539b0f87-validation.md` has SHA-256 `e3382fbdee4dcb5b6a1050eebb197b206342d0f7b29b0471a862bfc24cec7a14` and records each raw log digest. The first DX9 run found an in-scope `[[bin]] path` classification regression; its failed log remains preserved, and the repaired exact case passed. Tracked `Cargo.lock` and Ruff gitlink were unchanged. No Cargo cleanup occurred.

The read-only scoped Opus review returned **SATISFIED** with no blocking findings (response SHA-256 `821ffa57b916360a9fb0c153a51bbf51c394c7f85be1878948a409d8267d6d5c`). The full response is outside the reviewed tree at `/home/yaser5/projects/sifr/architecture-n02b2-lock-identity-evidence/ef9c304c75582c0dd0b10addf7f83f18539b0f87-review.md`. Nonblocking review notes suggested a more specific diagnostic for non-file lock candidates, weighed the trusted-vendor rescan cost against concurrent-mutation detection, and noted an `authority-readable` field name; a pre-existing non-prepare normal-seed fragment still encodes unreadable authority by error kind while actual preparation fails closed. These are follow-ups, not N02b2 acceptance gaps. The intermediate-phase policy defers the full merge profile to Q01. This item ends here; N02c requires a separate assignment.

## N02b/N02b2 strict Clippy blocker found by emitted-Rust Item 12R (2026-09-23)

On emitted-Rust Item 12R integration candidate
`a998e53ac6ad5cee96a0fae452f25e5077193bde`, the exact
`cargo clippy --locked -p sifr_driver --lib -- -D warnings` command failed
with two diagnostics unchanged from merged main
`ca093afb2c745633b18f5b0b503bf8b73121d08f`:

- N02b2/F07 [PR #3954](https://github.com/sifr-lang/sifr/pull/3954)
  introduced a module-scope `normalized_manifest_cache_input` import at
  `crates/sifr_driver/src/build/cargo_resolution.rs:15` that is used only
  by tests. Strict Clippy reports `unused_imports`.
- N02b/F07 [PR #3951](https://github.com/sifr-lang/sifr/pull/3951)
  introduced `contains_key` followed by `insert` at
  `crates/sifr_driver/src/build/rust_interop_sqlx_offline.rs:73`.
  Strict Clippy reports `clippy::map_entry`. Repair must retain the
  fallible metadata digest only when the key is absent.

The raw evidence is
`/home/yaser5/projects/sifr/emitted-rust-item12r-evidence/clippy-driver-a998e53ac.log`
(SHA-256
`c87b6999a55a8ff9c7ac2faaa3e5d509b8105d891a7772294e8ca5a18586f642`).
Both diagnostics block Item 12R final strict validation and implementation
merge. The Item 12R worker made no code change for these external failures.

## N02b/F07 driver and bridge persisted identity delivery receipt (2026-09-23)

[PR #3951](https://github.com/sifr-lang/sifr/pull/3951) merged as `603f56cfd889d66ae1f276448083aee4b3cfb22c` from published final candidate `af8cd5773b35bca02dab533f1db2ae07d3b7cdff` (base `62ff95089bf1b120214c9978442eeef9b32979d4`). The tested local candidate `41e47ac66a7a4e06fe02a989c674461892b2fb40` and published candidate share tree `3aa980c8b313b3fb9df901315f04171d7dd23b23`; the GitHub connector recreated commit metadata because the host had no push credentials. Driver bridge trees, sysroot metadata, trust policy, combined Cargo inputs, probe markers and codegen bridge/Cargo fragments now use versioned, domain-framed SHA-256 identities over their semantic fields. Required bridge and sysroot source trees and the final SQLx metadata tree fail closed on unreadable input. The Rust interop final warm key binds the package source snapshot (source-map authority and module bytes); the pure-package project cache continues to validate current authority and source observations. Cargo still runs before final native reuse, and the DX9-F7 alternating-dependency regression passed. Prepared-lock and unchanged-lock identities remain in N02b2; N02c/F08 serialization, N03 environment encoding and N04 probe-input/symlink/memoization work are not claimed.

On Linux x86_64 with rustc and Cargo 1.98.1, 14 exact `sifr_driver --lib` cases and two exact `sifr_codegen --lib` cases passed, each selecting one assertion. They cover all Cargo/trust/sysroot and bridge-source fields, absent/empty/unreadable and repaired SQLx metadata, package source-map authority and source states, existing warm reuse, direct bridge probe, and DX9-F7. The candidate-SHA-keyed validation manifest at `/home/yaser5/projects/sifr/architecture-n02b-driver-bridge-evidence/af8cd5773b35bca02dab533f1db2ae07d3b7cdff-validation.md` records every case, command scope and raw log digest (manifest SHA-256 `4e600388faa6416046a566dd3988cb4c4b45e88a1b11efa2289fa4254498a8cd`). The direct-filesystem-effects guard passed 2,769 sites (raw SHA-256 `69175a1cbf855568b1d8a76db3ebcce48799387f04d6c4d589bb3acf2b5db864`) and its negative self-test passed (`8fa490515a807532f428fbe8fbef1de198850a48156fd4e318782e22aada6c20`). Formatting, 900-line file-size, HIR maintainability, documentation structure and diff checks passed. Tracked `Cargo.lock` blob `e54d6c74360356d397fb5c3a99e11cf501cbc004` and Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1` were unchanged. The worktree-owned target remained warm; 35 GiB remained free before review, and no cleanup occurred. Initial submodule setup, manifest compile and filesystem-inventory failures and a zero-test SQLx selection remain in the raw evidence as failed or incomplete attempts, not passes.

The first scoped Opus review found an in-scope unreadable-SQLx warm-key regression (response SHA-256 `b9b6abd6eed2f3c2397dc68bb55100365124c74f19d23f32d47923daa8c97689`). The repaired exact candidate received **SATISFIED** with no blocking findings (response SHA-256 `ead4503c7b453ec072d9bc6a7ad5651e49878dc3f4c1a7709418769e1d6e8392`). Both complete responses are keyed by candidate SHA under `/home/yaser5/projects/sifr/architecture-n02b-driver-bridge-evidence/`, outside the reviewed tree. Nonblocking inner lock FNV and absent/unreadable lock-state findings remain with N02b2; path-root symlink classification and process-global probe memoization remain with N04. The probe `.ok` marker is write-only and never bypasses Cargo. The full merge profile remains with Q01; no next batch is claimed.

## N02a/F07 package-graph persisted identity delivery receipt (2026-09-23)

[PR #3945](https://github.com/sifr-lang/sifr/pull/3945) merged as `a57e72c0a4dc5b94b4a16bedec99cba0f17e70e7` from published final candidate `c9b22cee04b4103bbe7450269a415c9a35a03fcf` (base `4734ffa03bebf926753221244ca118ff6b760583`). The published tree `b7cc50993910d691a812ddf5c83fa4c51e1a6da2` matches tested local candidate `1286635efe433a74dabfcbcc7da418a6da546976`; the GitHub connector recreated commit metadata because the host lacked push credentials. Package-owned persisted graph, normalized Cargo metadata, source map/snapshot, Python probe and build-input digests now use length-framed, versioned, domain-separated SHA-256. Full graph and Cargo structures bind fields omitted by the old projections; source snapshot binds module authority, public API state and source bytes, including ambiguous candidates. Python authoring and full environment probes have separate domains. Ordinary in-memory hashing was not changed. Cargo remains the authoritative resolver before final cache reuse; driver/bridge/lock fields and F08 serialization are separate N02b/N02c work. N03 environment encoding is not claimed.

On Linux x86_64 with rustc 1.98.1, the exact `sifr_package --lib` identity cases passed for build-key fields (raw SHA-256 `eb0df28c50df7a98ed5368b776bffe16fe18aa211dd5fa8806f47cc2369c1153`), normalized Cargo fields (`ad2fe4231a20b67e08f83d6843b1f613456001ad295d741b6242973fd3d2e764`), package graph fields (`38e045f92af3e651cb34cecac901ca41d2e0d6008e738617a22e245a9fe4e705`), absent/empty/unreadable package-source consumption after the final source change (`c051acc9a9be1a49a7b5ff6e9159a7bf4580cad3f5a2b4056e7db81ac751f1d4`), and Python request/probe fields after the final optional-value test (`efd704b2575419c5c7075e8c90cf6f1e596a3716b28f9131f8874b9380ef7948`). Framing, shuffled metadata stability, existing package build-key, three Python cache and SQL provider identity regressions passed. The authoring/full empty-probe domain distinction passed (`5fcf7f95c57eabd5a8254028622b15022155c678a6ea5b12c9e09d3f51c266ac`) along with its affected existing authoring case (`90eda5fcfa190306ceb8b6b18aa8707b2e7201a3fd76b01e1f6254cd3e9c01fa`). The five new filesystem sites were classified; focused comparison passed for all six sites in the two touched paths (`ca15aa29c2f4d81f4a145e25d0ec51b2c6e3bddd3cd8c56b99860f2295b93ba3`), and the guard's negative self-test passed. `cargo fmt --check`, file-size guardrail (4,128 files), lowering maintainability guardrail, documentation structure and diff check passed (`9620cb521e376ef61affbcf502c4265fd56ccb8c16088bcea2dc97df80002f95`). Tracked Cargo lock blob `e54d6c74360356d397fb5c3a99e11cf501cbc004` and Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1` were unchanged. The 5.2 GiB worktree-owned target remained warm with 58 GiB free; no cleanup occurred. Initial Ruff/editor submodule setup failures and an initial compile derive error were corrected before final checks and are not passes. Raw logs and SHA-keyed review responses are under `/home/yaser5/projects/sifr/architecture-n02-f07-package-evidence/`. The full merge gate remains with Q01.

The first scoped Opus review found the five missing filesystem inventory rows (response SHA-256 `57e214136fd39ea34aa9ec1052f0df4d5ad9141af19fc3d84e7033642918ad74`). After the one repair batch, final candidate review returned **SATISFIED** with no blocking findings (response SHA-256 `4b82abde38c1b82e57c957686a0e468f76a6c962c4caa22c61f7e9600669282f`). The full filesystem inventory still fails at 27 untouched N01/F15-F16 sites (raw SHA-256 `96ee425be807207a7aac840487ce53c2395e91448fd6f0c27dda83ef2e4bfcd0`); that failure is recorded under N01 above and is not N02a acceptance. N02b owns verifying package graph/source authority and persisted payloads at driver warm consumers plus bridge F07 identities; N02b2 separately owns prepared-lock and unchanged-lock F07 identities. N02c/F08 must resolve the `serde_json::to_vec(...).unwrap_or_default()` failure path: whole-struct `PathBuf` serialization can fail on non-UTF-8 paths and otherwise hash an empty payload as reusable success. Source-map candidate ordering and cross-host path separator normalization are follow-ups for their respective owners. This item ends here; the next batch requires a separate assignment.

## N01r/F15-F16 filesystem-effect inventory delivery receipt (2026-09-23)

[PR #3948](https://github.com/sifr-lang/sifr/pull/3948) merged as `f0308cb514dfb58280b515e3841b817d484e8073` from published candidate `349612393bcd0bdba80c02afcd6739f733dbbf06` (base `2fcc3fe560a6fd590bebd948d5854e013f4f2ec5`). The published tree `671678f823d678e8d8d735bacdf9f87d10e4c0af` matches tested local candidate `3b4765eb75fac09bde856ab4a2c30212f399849c`; the GitHub connector recreated commit metadata because the host lacked push credentials. The inventory now admits the 26 N01-owned sites and removes the one stale symbol row reported in the N02a receipt. Two production `cmd_test` canonicalizations that select the Sifr package are semantic inputs; 15 test reads and output assertions are tooling inputs; nine test fixture mutations are output effects. No Rust implementation or N02b identity was changed.

On Linux x86_64, the exact full `python3 verification/areas/developer_tooling/check_direct_filesystem_effects.py` passed for 2,737 sites (raw SHA-256 `1b3745640fb39f26c56944ff336613e5cdcc7dffd9f5b8ae172a37581edf2d79`); its negative `--self-test` passed (`8fa490515a807532f428fbe8fbef1de198850a48156fd4e318782e22aada6c20`). Documentation structure passed (`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`), the 900-line file-size guardrail passed for 4,128 files (`702b8108849d2f0948515796cffb5c8056d78997ee5ccdc79f502b26d4ba5775`), and `git diff --check` passed. Tracked `Cargo.lock` blob `e54d6c74360356d397fb5c3a99e11cf501cbc004` and Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1` were unchanged. Initial documentation attempts failed because the pinned editor and nested VS Code submodules were uninitialized; their raw logs remain preserved, and the check passed after initialization. The full merge gate remains with Q01.

The scoped read-only Opus review of the published candidate returned **SATISFIED** with no blocking findings (response SHA-256 `4c5c20e1e34f9b59710a9305df60c8d8ab20613c771a3c6b0a50a87d76324a64`). Raw logs and candidate-SHA-keyed review evidence are under `/home/yaser5/projects/sifr/architecture-n01r-inventory-evidence/`. The review's nonblocking suggestion to state the CLI package-selection classification boundary remains with V03 guard ownership. The N02a full-guard failure remains historical evidence of its earlier candidate; N01r closes that bounded follow-up only. No next batch is claimed.

## N01/F16 native storage and test orchestration delivery receipt (2026-09-23)

[PR #3943](https://github.com/sifr-lang/sifr/pull/3943) merged as `9bd32eca86ce93e8aadcd6db228ea7b2cce91f95` from published final candidate `b97c0f925daa20267fa6757eb4e17aa6d6c0d04c` (base `4e0f7140986d8b246f12294061beccebfc1f001a`). The published tree `06e168269b7005b64a3ea18d2193c3427ca92012` matches tested local candidate `428fe21477aaee88ab7926417f7894549edd68dc`; the GitHub connector recreated commit metadata because the host lacked push credentials. The test runner now removes retired generated support and bridge source files inside its leased native family before Cargo runs. Cargo's mutable project root retains its lock and target for freshness; the finalized entry captures only verified test executables, runtime libraries and its manifest. This does not claim DX9-F3 cache-scope acceptance, N02 warm-consumer identity or Python-test expansion.

On Linux x86_64 with rustc 1.98.1, exact `sifr_driver --lib` cases passed: `test_runner::execution::tests::test_runner_mutable_root_and_final_snapshot_stay_distinct` (raw SHA-256 `c5a80cf49ab28c3f85988cb364e529c339262f65d70683779970c12a4879b8d4`), `test_runner::execution::tests::test_runner_same_key_concurrency_publishes_one_snapshot` (`b958c973936f8ba84fe9ee280156db43c1a19ed014a7bca653b9a8c3b5ba2607`), DX9-F7 `build::native_reuse_tests::dx9_f7_locked_root_reconciles_alternating_runtime_dependency` (`deb62e7c472deffa1688beb0918c525bf23cfcde0ac4edcb0a6ed671e50fa918`), `build::native_reuse_tests::dx9_finalized_artifact_survives_same_root_edit_and_runs_again` (`d9cd1f4a6262750cbdccba235a0b16642e2adb93879df3c0ef0dc69c4e8b68a8`), and F15 `test_runner::execution::tests::test_runner_cargo_modes_use_build_resolution_and_trace_policy` (`c1ae9cd63a4d38b7acb2108e6d55a1f5c3a4ab9ff5fc9a36629ad6b7eda47ce8`). The new tests verify retired source removal in one mutable root, one miss and two hits for three concurrent same-key callers, unchanged first finalized executable bytes after a changed build, and no `Cargo.lock`, `target` or generated `src` in reusable final entries. `cargo fmt --all --check`, `git diff --check`, the 900-line file-size guardrail and driver maintainability check passed. Tracked `Cargo.lock` blob `e54d6c74360356d397fb5c3a99e11cf501cbc004` and Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1` were unchanged. The worktree-owned target was warm after its first build; 68 GiB remained free and no cleanup occurred. Initial setup failed on an uninitialized Ruff submodule, then the first compile found a private helper call; both failures remain in the raw logs and were corrected before final validation. The full merge gate remains with Q01.

The scoped read-only Opus review on published candidate `b97c0f925daa20267fa6757eb4e17aa6d6c0d04c` returned **SATISFIED**, no blocking findings (response SHA-256 `a24a3728eae561505ed7e902cc641d529687523707cfa39860316972f7c0ffd2`). Raw validation and SHA-keyed review evidence are stored outside the reviewed tree under `/home/yaser5/projects/sifr/architecture-n01-f16-evidence/`. Nonblocking storage suggestions about empty retired directories and literal root-name collision remain with driver storage; stronger same-key repeat and exhaustive final-entry inventory assertions remain with test orchestration alongside existing DX9-F3 scope work. The prior F15 directory-selection and diagnostic-code suggestions are unchanged. No next batch is claimed.

A later N02a scoped review found the direct-filesystem-effects inventory red for 27 sites in the N01/F15-F16 owned CLI/test-runner files (`crates/sifr/src/cargo_lock_mode_certification_tests.rs`, `crates/sifr/src/test_cli.rs`, and `crates/sifr_driver/src/test_runner/execution.rs`), including one stale inventory row. The N02a candidate classifies only its five new package-owned sites; the remaining full-guard failure is an N01 test-orchestration inventory follow-up and is not N02a acceptance. Raw failure: `/home/yaser5/projects/sifr/architecture-n02-f07-package-evidence/filesystem-full-candidate.log`.

## N01/F15 supported build/test policy parity delivery receipt (2026-09-23)

[PR #3939](https://github.com/sifr-lang/sifr/pull/3939) merged as `0dd4e076f4a043e19c292c55d1358839180485aa` from published final candidate `d6d2491c92a4b0b0012c157632db38121f8a2a45` (base `5509a0920f41783beb9c63ae8533af40e792fc08`, tested local candidate `ffbacdb9a74a63077a88682c181563dce35a720b`, identical tree `a2f09571d3117bc268096bd651423aebfca46eb7`). The GitHub connector recreated commit metadata because the execution host lacked push credentials, while preserving the tested tree. Package `sifr test` now carries build's package/sysroot Cargo policy through Rust interop and final materialization: authoritative locks, normal seed, locked/offline/frozen flags, vendor and sysroot configuration, declared toolchain environment, hermetic Cargo environment, native-link trust checks, and `final-test` tracing. Constrained manifestless test is rejected with the shared package-lock diagnostic; normal manifestless test remains supported. N01/F16 mutable-root concurrency and immutable final output, N02 warm-consumer identity, Python-test expansion, and DX9-F3 cache-scope acceptance are not claimed.

On Linux x86_64 with Python 3.14.4 and rustc 1.98.1, exact CLI cases passed: `cargo_lock_mode_certification_tests::test_lock_flags_parse_and_normalize_without_collapsing_frozen` (raw SHA-256 `da13dce358dc89cb2b3e23f1e4afba945e7934794b189d1098c8f2f145b92b95`), `constrained_modes_reject_manifestless_check_build_run_and_test` (`002480daa47ad7775c95a0bad2d87e53c3272d7ca99fe68cb9346edff48368d2`), and `sifr_test_modes_use_package_resolution_and_final_test_trace` (`3d497f24584cf3d9542d7cc6a15b162f494acf4cbf33333a02bd4e13c1720b72`). Exact driver four-mode test `test_runner::execution::tests::test_runner_cargo_modes_use_build_resolution_and_trace_policy` passed on the final candidate (`ba45492737861a2877b32a134b4413654df303f2e21540b87359347017d49b93`). Focused native-link, cache-key, warm reuse and source invalidation, DX9-F7 alternating-dependency, and Rust interop cases passed; their raw logs remain under `/home/yaser5/projects/sifr/architecture-n01-f15-evidence/`. `cargo fmt --check`, `git diff --check`, documentation structure, and the 900-line source guardrail passed. The tracked Cargo lock and submodule pins were unchanged. The worktree-owned Cargo target stayed warm; no cleanup was needed. The full merge gate remains for phase final integration under the prospective exception.

The scoped Opus review of published candidate `d6d2491c92a4b0b0012c157632db38121f8a2a45` returned **SATISFIED**, with no blocking findings (response SHA-256 `594f206048325b71fefbbd25283d374baf85b054bc165b1e5578649edfb7708e`). Its nonblocking test-directory/package-selection and diagnostic-code suggestions remain for test-orchestration ownership; the existing ignored `--release` test flag remains with DX9-F3. No DX9-F3 owner PR was open during this item, and no DX9-F3 acceptance was duplicated. N01/F16 remains open for the next batch.

## N01/F15 package-root selection follow-up receipt (2026-09-24)

Emitted-Rust Item12R [PR #3946](https://github.com/sifr-lang/sifr/pull/3946) found that `core_language:cfg_flow_behaviors` failed at `sifr test demos/control_flow_paths` and `project_workspace:project_graph_isolation` failed at `sifr test demos/graph_isolation`, both with SIFR-RUST-CARGO-0001. The failures trace to N01/F15 candidate `d6d2491c92a4b0b0012c157632db38121f8a2a45`: the CWD package graph had no package owning either test directory. [PR #4006](https://github.com/sifr-lang/sifr/pull/4006) merged the bounded repair as `af2fc49576f9018f6b0aee92c6c83daf2291db67` from tested and reviewed candidate `f5994db3a9a8a743eb288a92b70b2234d6786f17` (base `38bb6a1b941ea9b44364f140a243661e5ae7b7ad`). `sifr test` retains the CWD session and deepest graph-package ownership selection, falls through to the standalone path in normal mode when no graph package owns the directory, and retains the shared package-required diagnostic in constrained modes. Source-only `sifr.toml` and manifestless behavior remain supported under the same build/test policy.

On Linux x86_64 with rustc 1.98.1, the final candidate passed the exact `core_language:cfg_flow_behaviors` suite (4 rows, including `sifr test demos/control_flow_paths`; raw log SHA-256 `8b6ffb05f4c8ae25f057ff0ec9a2d7681e1e522e9b30a696ea4cc5fa53731c93`) and `project_workspace:project_graph_isolation` suite (5 rows, including `sifr test demos/graph_isolation`; `6ff90bff43241b575c86b183cdf5b1e5c755785f3487cbb5e9c4a474435fe878`). Exact CLI regressions `cargo_lock_mode_certification_tests::sifr_test_modes_use_package_resolution_and_final_test_trace` (`d4065409c10d55c8fce61c4a8ed1b1108fbb6014d8b932bd38b993b3151d4058`) and `constrained_modes_reject_manifestless_check_build_run_and_test` (`1c97983778f52570a7a87072e59468c8f47cafbedd0a7912b4fecc85734de7c1`) passed. `cargo fmt --all --check`, `git diff --check`, and the touched-file 900-line guardrail passed. Tracked `Cargo.lock` blob `935d5af418fcf3ce3292cfb1408985e54f3e1378` and Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1` were unchanged. The isolated Cargo target had 788 GiB free at validation; no cleanup occurred. Raw logs, JSON results, and review responses are under `/data/sifr-architecture-n01-f15-package-root-evidence/`.

The first candidate `b3b1f7fa6fa9ee8a01ae74b97251f25c7a85bb42` rerooted source-only manifests and failed its scoped review; its earlier core-language pass remains historical. The second scoped read-only Opus review on final candidate `f5994db3a9a8a743eb288a92b70b2234d6786f17` returned **SATISFIED** with no blocking findings (response SHA-256 `789591d81222032025e44e1c8bb26a678847642e04a1e1bd6ef2586c68bf4921`). Its nonblocking suggestions about the constrained no-owner diagnostic wording and dedicated no-owner unit coverage remain with test-orchestration ownership. The out-of-graph directory CLI contract is likewise a later policy question, not this repair's acceptance. The Item12R owner was notified in [PR #3946](https://github.com/sifr-lang/sifr/pull/3946#issuecomment-5812478380). The full merge gate remains with Q01; no next batch is claimed.

## D01a/F32 registry prerequisite delivery receipt (2026-09-23)

[PR #3936](https://github.com/sifr-lang/sifr/pull/3936) merged as `9c71301415299c54bd953d43faf621bd598035f3` from published final candidate `7b37ce77ab219aec185e0fff84b25f8a0435a209` (base `1b1002a5c7d077900518277077569071315f7a13`, tested local candidate `da0a101be7b299e05d151772e0c4056ef3327abe`, identical tree `f072a70e6b4c65a449540adb270a323600cb17ef`). The host lacked Git push credentials, so the GitHub connector recreated commit metadata while preserving the exact tested tree. D01a rejects duplicate or mismatched active code identities, missing owner-module paths, and stale representative fixture files or `::symbol` references through declared Rust modules. Registry and catalog fixtures, generated code pages, related-span JSON assertions and the verification input-inventory mutation case were synchronized. The phase row now separates later F31 maps (D01b), residual non-codegen F32 completeness (D01c), and codegen F32 (X02); the V03 demo-workspace statement is corrected without changing V03 code or historical receipts.

On Linux x86_64 with Python 3.14.4 and rustc 1.98.1, exact final selections passed: `python3 -m unittest discover -f -s verification/areas/diagnostics/checks -p code_coverage_test.py` (18 tests, raw SHA-256 `8c469ec8f1b929815d30b21fd0c00149c95cf6add1c68b44aaa2828aa91104a1`), `cargo test --locked -p sifr_diagnostics --lib` (32 tests, `c86760b774391e522a5e4e61b6159675c40eed1df1808de701bb2f46dafbabc9`), and `PYTHONPATH=verification/runner python3 -m unittest -f sifr_verify.dx4_fixture_checks.FixtureTests.test_r08_diagnostic_registry_and_manifest_mutations_invalidate_evidence` (1 test, `2bec6121e357fd35d60e3cbc489a6a290b466581e13cadeedd35f60dce5933bb`). Registry coverage, baseline coverage, diagnostic docs sync, documentation structure, `cargo fmt --check`, `git diff --check`, and the 900-line guardrail passed; documentation structure raw SHA-256 `d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`, file-size guardrail `06f18ed54c0fa6a16c0c2748f474f7904f025cdd960835b745eb106a777d77f8`. Validation used tracked `Cargo.lock` blob `e54d6c74360356d397fb5c3a99e11cf501cbc004`, Ruff gitlink `0f7e9ce63515fb45859f884452e5741eb19741c1`, and the worktree-owned 333 MiB Cargo target with 101 GiB free; no shared target was cleaned. Initial setup exposed an uninitialized Ruff submodule, and an initial fmt check found one layout adjustment; both were corrected before the final selections.

The first scoped Opus review on prior published candidate `9e9108cf9fee73851943889de3382be2dae302f7` found the missing `::symbol` check (preserved read-only response SHA-256 `474aa5faa77201e19537d8120d4682d86f0b711b4295862f7bd25a6e190b7b2b`). The final scoped review on `7b37ce77ab219aec185e0fff84b25f8a0435a209` returned **SATISFIED**, no blocking findings (response SHA-256 `a016c015cc87db59382d41581781d76a789123ec970f8c3682e5d8ac93d2c47c`). Both complete responses and raw validation logs are stored outside the reviewed tree under `/home/yaser5/projects/sifr/architecture-d01-registry-evidence/`. Follow-up findings remain with D01c: many declared owner paths resolve but do not contain the actual emission, and bare-file fixtures lack case-level identity. Suggestions about lexical symbol matching and nested `#[path]` traversal are deferred with that registry owner. X02 and D01b remain open; no next batch is claimed.

## V04/F27 delivery receipt (2026-09-23)

Verification process deadlines merged in [PR #3933](https://github.com/sifr-lang/sifr/pull/3933) as `28ca60d64b3c9abd99a2f4e5d3f54c548da56667` from reviewed candidate `b4c1b9720b8e4413accdf1bb257038f30affd950` (base `4788d4830cedd4bf5cccddd768787f94700d9e56`, tested tree `59ab76fc09565b927740022254cd78314efe7019`). The final tree matches the isolated local worktree commit `c184e605877adb2b218c4443612b268203ab7955`; the GitHub connector recreated commit metadata because the execution host had no push credentials. Nested commands inherit one narrowing absolute monotonic safety deadline, including lock waits and both stages of the standalone metadata helper. An explicit `SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS` opts a whole step into a shared deadline, while the default retains per-child safety bounds and separate performance budgets. Invalid/nonfinite deadlines fail before spawn; native exit 124, safety timeout and cancellation keep distinct causes. On deadline or cancellation, the runner stops waiting on pipes held by descendants outside the owned group, keeping bytes already read, and F26 setup cleanup remains covered.

On the Linux execution host with Python 3.14.4, the final exact selection `PYTHONPATH=verification/runner python3 -m unittest -f sifr_verify.dx3_process_checks.ProcessTests sifr_verify.metadata_setup_checks.MetadataSetupTests sifr_verify.native_test_execution_checks.NativeExecutionTests sifr_verify.cargo_sysroot_setup_checks.SysrootSetupPolicyTests` passed all 48 cases (raw SHA-256 `f5fade655a0f30c5b837d40db168ecdd2b22ca3e07b03aef784cff0ce70692c8`). These tests cover ignored termination, same-group and escaped pipe holders, bounded lock waits, standalone helper invocation, native exit 124 and F26 setup failures. Python compilation and diff check passed. The 900-line guardrail passed for 4,126 files (raw SHA-256 `06f18ed54c0fa6a16c0c2748f474f7904f025cdd960835b745eb106a777d77f8`). The first focused run failed once when an F26 descendant-readiness fixture did not start within two seconds (preserved raw SHA-256 `9a3799a066a8b79cb63be893de94d175a7f78aa83a89228685948de8d0b2d3dd`); that exact case passed on rerun (raw SHA-256 `e623ac621e1fd5a894e94ad223d507d7ebc71f25b8ad9cd1a7e500b5544c844c`) and passed in subsequent final selections. No Cargo build or compiler fixture preparation was needed for this Python-only selection; the tracked Cargo lock and submodule pins were unchanged.

The first scoped Opus review found a post-deadline CPU spin on an escaped pipe holder (response SHA-256 `393337f9e3d5627d81e934d18d507fa5ae7652fe726170cbf6a615a31ef922c6`). The repaired final candidate passed the second scoped review: **SATISFIED**, no blocking findings (response SHA-256 `48698abf4bbf6f1db4a912a4fa79bd1326429679c23243d8d691abe441dd825b`). Raw logs and review responses are under `/home/yaser5/projects/sifr/architecture-v04-f27-evidence`, keyed by candidate tree and SHA. The review noted nonblocking follow-ups for draining already buffered output after a safety outcome, making invalid optional step-limit configuration a runner-owned diagnostic, and reducing loaded-host timing sensitivity in two new process tests. Windows portability remains with its existing owner. The intermediate-phase policy defers the full merge gate to Q01.

## V04/F26 delivery receipt (2026-09-23)

Verification subprocess initialization and cleanup merged in [PR #3931](https://github.com/sifr-lang/sifr/pull/3931) as `3fbee160abfb2d3d29a63b7e0716aa5b3166f2fe` from reviewed candidate `e94631df5912215916d9ff2d29e505aac514c9ca` (base `6a3ef5cf0885644ae2d767276a20866c0b8c72b4`, tested tree `91ff51152591f3d7eef8fbafbcf23dc9f01b8a46`). The runner rejects worker-thread calls before spawning, installs signal handlers before spawning and restores partial setup, and owns process-group teardown and direct-child waiting from spawn through selector setup, stream callbacks and normal completion. Negative tests inject signal setup, selector creation, first and second registration, and callback failures; each post-spawn failure kills a real child and descendant. Normal output and direct-exit descendant cleanup remain covered. F27 absolute deadlines, lock waits and native exit-124 classification remain separately queued and unqualified.

The named `PYTHONPATH=verification/runner python3 -m unittest -f sifr_verify.dx3_process_checks.ProcessTests` passed all 20 cases on Python 3.14.4 (raw log SHA-256 `33ce9454b4b4cfde9891e2b34662e4b0d522a244c18ce99a5801d65e821fbd76`); focused initial negatives and existing behavior also passed (`cc4c9cfff656b2f3b08d815170a593699f3b620314ea5330fd14814878bef0c9`, `705683330057c023028b8a68e193bdbac1be93462efe1ea8d463bfa40742b4db`). Python compilation and diff check passed. The 900-line file-size guardrail passed for 4,126 files (`06f18ed54c0fa6a16c0c2748f474f7904f025cdd960835b745eb106a777d77f8`). Documentation structure passed after initializing the pinned nested editor submodule (`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`); the initial missing-submodule failure is preserved (`b2a67fa95ecd2c4c315fba0190c97f4373d7eb3fded81c211b20c7bde81c6186`). Raw logs and review are under `/home/yaser5/projects/sifr/architecture-v04-f26-evidence`, keyed by candidate SHA. The scoped Opus review returned **SATISFIED** with no blocking findings (response SHA-256 `2d6ed065d765a2caa81b3df6232b1250b96bff59f08eb3fdbfd01529d06dd45d`). The intermediate-phase policy defers the full merge gate to Q01.

The review identified two pre-existing V04 process follow-ups for separate scope adjudication: group signalling after the direct child is reaped can target a reused PID, and repeated 250 ms group teardown can delay draining inherited pipes. It also noted that pre-spawn handler installation can change an ignored signal's child disposition to default after exec. These are not claimed as F26 regressions or F27 acceptance. Windows process portability, V01 host policy, DX.10 and TypeScript-Go failures remain with their existing owners.

## V03b/F25 delivery receipt (2026-09-23)

The semantic parser guard merged in [PR #3929](https://github.com/sifr-lang/sifr/pull/3929) as `6e634e6556cb54adadd07e49c93e4b6e2b2348f6` from reviewed candidate `b3d1e9567778d3ee4fd95ebdb59514d75d39df8f` (base `2c22bf0f1d8edc1c9601b6ea4f75abf3a41eb47a`, tested tree `f162da93e96a0478f2b1e318a71d6ac5bbd9d8d8`). It guards the complete current exported `lower_module*` family, the prior forbidden parser/lowering names and `parse_module`, `parse_module_raw`, and `parse_module_suite`. Direct calls, crate aliases, grouped imports and renamed function imports are checked consistently. Production and test-only code are separated inside mixed files and across `#[cfg(test)]` module references. Existing syntax-only CLI, lint, formatter, bridge inventory and declaration-range uses remain individually adjudicated; canonical stdlib bootstrap parsing and lowering remain allowed at exact sites.

The named guard self-test passed with negatives for each parser entrypoint, each exported lowering callable, direct and aliased imports, non-root qualified calls, mixed production/test files and cross-file module gating (log SHA-256 `77113b399b7f25a8ee1c8347f875b1bfbde4173438313252a1ebe3bca34f1ada`). The complete repository guard scan passed (`205c08c1932627bb13ef4d94e111953e6c71797eaaddb21b75e7a355003ff7ee`); Python compilation, the 900-line file-size guardrail (`06f18ed54c0fa6a16c0c2748f474f7904f025cdd960835b745eb106a777d77f8`) and diff check passed. The first scoped Opus review found two in-scope omissions: five exported lowering siblings and non-root qualified calls. Both were repaired and revalidated. The final scoped review returned **SATISFIED** with no blocking findings (response SHA-256 `56a472616ef8539ddf2eb923c27171e0a1691ee42cb221657f836e15e6361251`). Raw logs and both review responses are under `/home/yaser5/projects/sifr/architecture-v03b-f25-rescope-evidence`, keyed by candidate SHA. The docs-only receipt passed the documentation structure check after initializing the pinned nested editor submodule (log SHA-256 `d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`); the initial missing-submodule setup failure remains recorded. The intermediate-phase policy defers the full merge gate to Q01.

Deferred guard follow-ups: V03's statement that demos are outside the Cargo workspace conflicts with four current demo workspace members and needs its owner to reconcile the coverage boundary. Integration-test roots are conservatively scanned as production; future guard ownership can refine their test-only classification if needed. Cross-file parser re-exports and function-item references remain outside this source admission check's first-order call/alias boundary. No V04, V01, DX.10 or TypeScript-Go implementation is claimed by this receipt.

## V03/F24 delivery receipt (2026-09-23)

The filesystem-effect guard merged in [PR #3926](https://github.com/sifr-lang/sifr/pull/3926) as `d3301943cb6cedad7d2860b54d72d5e1ae08edcb` from reviewed candidate `3c9fc534cda438f77d20c746ccf0fb3af2332c38` (base `30136206c94ee03784c9db93b2544de72116be88`, tested tree `48280e607d0e02b6eb900b36b0912aba4f73e522`). It inventories 2,707 direct filesystem sites across Cargo workspace source and declared target paths, including `std::fs`, `Path`, `File`, builder and Unix/Windows OS-extension effects. Grouped and aliased imports are resolved; each site and symbol carries an explicit semantic-input, build-identity, tooling-input or output-effect class. The checked-in inventory can adjudicate read sites individually, while filesystem mutations must be output effects. F25 parser checking remains V03b, unimplemented here.

The direct guard self-test passed with negatives for listed-file new reads, aliases, byte reads, new crate/bin targets, builder opens and creates, handle writes, OS symlink creation and aliases, `fchown`, `Path::read_dir`, `File::set_modified` and nonliteral builder write flags (log SHA-256 `8fa490515a807532f428fbe8fbef1de198850a48156fd4e318782e22aada6c20`). The inventory passed at 2,707 sites (`5c79092c9af74c04510d26c973d86e205dba2c46852069d4568fde4600414e6c`). Documentation structure passed after initializing the pinned nested editor submodule (`d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`); the earlier setup failures remain preserved. The 900-line file-size guardrail passed for 4,126 files (`06f18ed54c0fa6a16c0c2748f474f7904f025cdd960835b745eb106a777d77f8`), and diff check passed. The first scoped Opus review found three in-scope API-table omissions; their repair and focused regressions were reviewed on the final candidate. The final scoped Opus review returned **SATISFIED** with no blocking findings (response SHA-256 `8de115b501639efd1525d1caeeafdbadeaa7760f5c5c37b97ed7952e86cb4d08`). Raw logs and both review responses are under `/home/yaser5/projects/sifr/architecture-v03-f24-full-evidence`, keyed by candidate SHA. The intermediate-phase policy deferred the full merge gate to Q01.

V01 live admission remains blocked by the shared-host CPU policy under its owner. The DX.10 fixture inventory mismatch and TypeScript-Go LSP guard failure remain with their recorded owners. V03/F24 ends here; the separate V03b/F25 delivery is recorded above.

## V02/F06 delivery receipt (2026-09-23)

The Python-interop area identity repair merged in [PR #3921](https://github.com/sifr-lang/sifr/pull/3921) as `39de4657fee6f305b009242c5009cba4de181c1d` from reviewed candidate `85d43376662091bde1308829bb2e2b11b83320f7` (base `af83d6565a565ecea5f44dd50d1004bac275278c`, tested tree `b9c2fc5fdb91f0b0e8dcb56fe69400c2b9c5e88c`). The cache-aware `area_python_interop` profile step now derives the canonical `python_interop` area ID once for suite selection, fingerprint and receipt identity, exported cache state, and required Python cache paths. The step name remains the report and budget key.

The named `step_budgets.run_self_test` passed on Python 3.14.7 (log SHA-256 `dc9d35452fd7205accda7401b31636eea1474e194cd16d0fbd98b17d60fd2452`). Its profile-step regression observes missing-receipt cold, exact-input warm, changed-suite fingerprint and cold classification, then required Python-cache removal returning cold with the exported state updated. The 900-line file-size guardrail passed (log SHA-256 `a03cdfdb2d4f2827bb2d57e519613df7b23f027963c374ddfc4561bd64c1fc9a`); diff check passed. The scoped [Opus review](https://github.com/sifr-lang/sifr/pull/3921#issuecomment-5787261570) returned **SATISFIED** with no blocking findings (response SHA-256 `718600d58470b448ab634bdd0cc35164e2ebe92ae8c56284c4ce7271c68fb270`). Raw evidence is under `/home/yaser5/projects/sifr/architecture-v02-evidence`, keyed by candidate SHA. The intermediate-phase policy deferred the full merge gate to Q01.

V01 live host admission remains blocked by the recorded shared-host CPU policy; its benchmark qualification is not claimed here. The unrelated DX.10 fixture inventory mismatch remains in its owning issue. V02 ends here; V03 and later work require separate assignments.

## V01/F05 delivery receipt (2026-09-23)

The automatic `local-first-create-pr` CI job on N05 [PR #4010](https://github.com/sifr-lang/sifr/pull/4010) failed at `performance_reference_admission` before compiler or N05 assertions: `SIFR_PERFORMANCE_REFERENCE` was not selected in that runner. This is V01's reference-admission prerequisite, not N05 evidence or a passing profile. The job log is `/data/sifr-architecture-n05-f12-evidence-20260924/ci-local-first-create-pr.log` (SHA-256 `6e8ccb8eedfcc0b0a56184c96b825d16eb60f0d727fd8004e34e2fb9f2387e49`). The approved intermediate-item policy uses N05's named tests and scoped review; Q01 retains final full-gate qualification.

The fail-closed reference admission merged in [PR #3919](https://github.com/sifr-lang/sifr/pull/3919) as `a00d39d8367d14ed9a4cfdcb5f017df9997f4737` from reviewed candidate `2217d0c99535a32890ddbb5f165ff4e861235ab8` (base `787d3ad8d471ad35fd409b76887014c5b3fe334c`, tested tree `ade9f2145df926e7069d4aabef090d9260945b5e`). The profile runner now admits the selected reference before Cargo setup; direct benchmark suites also check it. Admission checks the immutable reference's manifest binding, trend freshness, budget policy, live host and toolchain identity. An absent, expired or incompatible selection reports unavailable qualification. The historical Mac trend was not refreshed or reused. No reference JSON was changed.

The selected `linux-i7-4720hq-12gb-dev-v1` reference (file SHA-256 `f6c5b4421ab7ce520a504316a4162d1fb9458f8ced2e45a3577cd9d76cda80eb`) passed the actual named trend and budget policy commands, with log SHA-256 `3d7722d4de5fb76977ff424082134a4a53608d737399b207020a913d52d3d823` and `f169678ceef8ca53a0bd178720dccfcc0158db99a41c2e505ecb787b60cd9fbe`. The selected trend self-test passed (`3acb4d2ecd2c49e8b4b5dc73c0f511a2de0373c5c9bfd16f34f5721cca6ae3fd`). Controlled-clock admission tests passed 5/5, including the exact 45-day and two-day future-skew boundaries (`432afaa0ee9ec03aabebcb16a23253ed84d30a427412435785859d4aee836ef0`). After review repair, the affected generated Cargo and admission ordering suites passed 45/45 (`a25980c285d9fc34550c3ae20a8c05235264d202b7e92a1bcc98648687bd53a4`), and the benchmark runner self-test passed (`4354d65b495cecc9072eaad2e21d746d023b7a84c610eaac17917b41d1c325fe`). Documentation structure, file-size guardrail and diff checks passed. The first scoped review found a simulated-test regression; its correction and affected tests were re-reviewed. The final [Opus review](https://github.com/sifr-lang/sifr/pull/3919#issuecomment-5787189453) returned **SATISFIED** with no blocking findings (response SHA-256 `784acbe2bf98f2f515794bafe9610dd64f8bf042d3bb75ca8048ffee04eed77c`). Raw evidence is under `/home/yaser5/projects/sifr/architecture-v01-evidence`, with final review keyed by candidate SHA.

Live selected-reference admission on the shared Linux host is **blocked**, not passing: using the captured Python 3.14.7 and ext4 temporary storage, the host reports `schedutil` while the approved capture requires the `performance` CPU governor (failure log SHA-256 `6cb3493d55fb5fb165a8914a627653de3862059d4fbb1092c9ad80c5f9c79ff4`). No benchmark qualification was claimed and the shared governor was not changed. The safe continuation is an owner-controlled host window with the captured power policy, matching toolchain and storage, followed by fresh live admission and the named performance qualification on the exact candidate. The reference must still be within its governed freshness window at that time; otherwise capture a new approved, immutable version. An optional broad runner self-test encountered a separate pre-existing DX.10 fixture inventory mismatch, preserved and recorded in [its owning issue](ad-hoc-dx10-profile-review-followups.md). The intermediate-phase policy deferred the full merge gate to Q01. V01's implementation delivery ends here; live qualification remains an explicit prerequisite for Q01.

## X01/F01 closure receipt (2026-09-23)

[PR #3916](https://github.com/sifr-lang/sifr/pull/3916) merged as `fa76440ee3ee9cd99b9a66a65c2e8483aa886c6a` from reviewed candidate `a3e4328d0468c02f1c16d84cf708e96b27e76488` (base `9522893db7986a7b4c6850d6cf1258d2a9793389`, tested tree `dbac1abadb90f7f00f2f8678e8bbc500ac9fe054`). The remote candidate tree matched the locally tested tree exactly. Both source HIR lowering paths now retain provenance for legal `unwrap`/`expect` names; IR validation still rejects compiler-owned method extraction and raw forbidden text. No X02/retained Item 12 implementation was merged.

Focused validation on that tree: `cargo test -p sifr_codegen --lib ir_validate::tests --locked --offline -- --nocapture` passed 12/12 (log SHA-256 `9052ae863a2d1b5d93673ce4531689f56dbab1665c35f99bcb68b1b3b8c14a80`); `cargo test -p sifr --test legal_failure_method_names --locked --offline -- --nocapture` passed 3/3 (log SHA-256 `adb98cd8b0fa269ed370c5bc54bcb6c79b68974e52a411e0b3a172d54009d524`). These cases exercise single-file check/emit/build/native, project check/emit/build/native/test, and inherited check/emit. Formatter, HIR maintainability, the 900-line file-size guardrail (4117 files), and diff checks passed. The scoped Opus review returned **SATISFIED** with no blocking findings; response SHA-256 `daa3058105ed21f2b930c99d14848376980c4b13714edc97e981dbb4363e750e`. Raw validation, historical setup failures, probes and review are preserved under `/home/yaser5/projects/sifr/architecture-x01-evidence`, keyed by candidate SHA.

Deferred follow-ups remain separate: inherited `super()` native const emission fails for unrelated method names too ([#3915](https://github.com/sifr-lang/sifr/issues/3915)); generic source-class `unwrap`/`expect` result typing in the pre-existing canonicalizer needs its own owner ([#3917](https://github.com/sifr-lang/sifr/issues/3917)). The reviewer also noted an unproven raw operator-method fallback path that fails closed if reached. X02 and retained Emitted Rust Item 12 preserve their existing scope and evidence. X01 ends here; the next batch requires separate assignment.

## C02/F19 delivery split (2026-09-24; scope record only)

The first C02 implementation attempt stopped at **needs-new-scope** on clean base `6f5b333380032eb14be694710d066a82daf14790`; it produced no implementation, test pass or review approval. On that base, both `sifr_analysis` and `sifr_lsp` directly declare `sifr_driver`, and their source has 103 `sifr_driver` references. `CompilerContext` combines identity, sysroot resolution, metadata selection and a process-local provider cache; `metadata_reader` and `metadata_producer` call driver `cache_storage` directly. Python declarations call driver probe/certification/diagnostic APIs; SQL editor construction calls driver profile APIs; generated preview calls driver compilation and editor construction calls driver project-cache restore. The existing `check_source_crate_dependency_direction.py` rule forbids `sifr_lowering` in analysis but does not forbid driver in analysis/LSP or resolve renamed Cargo packages and manifest-declared sources outside `src/`. This is a delivery plan, not a C02 completion claim. The historical M8 [PR #3560](https://github.com/sifr-lang/sifr/pull/3560) is a design reference only; its stacked candidate and validation do not qualify current main.

Each row below has one acceptance owner and one merge/receipt. A second clean-base C02a attempt on `b75ce643989ffbe9e5a473cb1bf0c32e2000cef8` stopped at **needs-new-scope**: the extraction spans more than 50 files, and a partial facade or lower reader with a duplicate provider cache would violate F19. It produced no PR, test pass or review approval. Sequential C02a0 and C02a1 replace C02a as independently compiling and merged items; together they retain C02a's final metadata/stdlib criterion. C02a0 establishes `sifr_compiler_services` and `sifr_cache_storage` (or equivalently lower crates) for the editor capabilities. Its public dependency graph must never point back to `sifr_driver`, `sifr_analysis` or `sifr_lsp`. Preserve C01's single frontend compilation product and source/metadata generation semantics. Use the exact named crate/suite/case selections below plus focused regressions for touched APIs; record raw candidate-keyed results and scoped review for implementation items. The next owner starts only after the predecessor is merged. The prospective intermediate-item gate exception above applies; Q01 alone qualifies the final full merge profile.

| Item and dependency | Owned implementation and bounded acceptance | Named acceptance and negative cases |
| --- | --- | --- |
| **C02a0** after C01 | One owner moves source stdlib bootstrap, metadata encoding and production, `PreparedMetadata`, and project metadata transport from driver into `sifr_compiler_services`. Move only the metadata-required lease, private-file, bounded-wait and atomic-publication primitives from driver `cache_storage` into `sifr_cache_storage`; driver keeps CLI cache policy, cache root/owner decisions, pruning and its existing metadata reader and `CompilerContext` as the sole provider/generation owner, consuming the lower producer. Compile acceptance on the C02a0 candidate: `cargo check -p sifr_cache_storage -p sifr_compiler_services -p sifr_driver --locked`; merge this boundary independently. The graph direction is `sifr_driver -> sifr_compiler_services -> sifr_cache_storage`, with no upward dependency. Pass cancellation through an explicit lower callback/interface and retain N06's live-holder versus idle-holder bounded wait, cancellation and owner safety semantics. Preserve the Windows secure storage/process companion; its native storage checks remain required when the platform path changes. No metadata selection/provider transfer, second provider cache or partial facade. | Existing exact cases: `cargo test -p sifr_driver --lib metadata_producer::tests::failure_recovery_stale_override_and_missing_entry -- --exact`; `cargo test -p sifr_driver --lib metadata_producer::tests::source_mutation_cannot_publish_mislabeled_metadata -- --exact`; `cargo test -p sifr_driver --lib metadata_producer::tests::threads_share_one_success_and_execute_independent_assertions -- --exact`; `cargo test -p sifr_driver --lib metadata_producer::tests::processes::process_waiters_warm_reuse_and_distinct_configurations -- --exact`. New lower case: `cargo test -p sifr_compiler_services --lib metadata::tests::stale_or_corrupt_override_retries -- --exact`. Focused producer cancellation/wait case `metadata_producer::tests::processes::wait_cancellation_and_cancelled_producer_retry`, lower storage lease/private-file/publication checks, and Windows companion `cache_storage::tests::windows_portability_extended_cache_path_publication`, `cache_storage::tests::windows_portability_private_acl_alias_and_lock_identity`, and `cache_storage::tests::windows_portability_pressure_prune_protects_leases_and_winners` when applicable. Preserve independent cold/warm and process-owner assertions. |
| **C02a1** after merged C02a0 | One owner transfers metadata selection and decoding, provider/navigation/qualification, tooling sysroot status/probe, `stdlib_external_defs`, and `CompilerContext` into `sifr_compiler_services`, taking explicit compiler identity, resolved sysroot and cache owner. Driver delegates/reexports the lower service APIs and drops its originals; the one provider/generation cache changes owner in place, with no duplicate cache or reader. Preserve installed/source-tree choice, override retry, concurrent generation, source-map navigation, demand loading and bounded metadata validation. Compile acceptance on the C02a1 candidate: `cargo check -p sifr_compiler_services -p sifr_driver -p sifr_analysis -p sifr_lsp --locked`; merge independently. Analysis/LSP direct-import removal remains C02e. No Python, SQL, preview or project-cache migration. | Existing exact cases: `cargo test -p sifr_driver --lib metadata_reader::tests::failed_override_retries_and_never_reuses_another_owner -- --exact`; `cargo test -p sifr_driver --lib metadata_reader::generation_tests::installed_relocation_and_generation_switch_pin_all_sources -- --exact`; `cargo test -p sifr_analysis --lib host::stdlib_tests::definition_inside_public_stdlib_can_link_to_private_declaration_file -- --exact`. New lower cases: `cargo test -p sifr_compiler_services --lib metadata::tests::incompatible_identity_rejects_provider -- --exact` and `cargo test -p sifr_compiler_services --lib metadata::tests::replaced_cache_owner_does_not_reuse_generation -- --exact`. Focused navigation-demand regression: `cargo test -p sifr_driver --lib metadata_reader::tests::dx7_navigation_demand_shares_index_and_reads_only_selected_source -- --exact`. Neither a stale provider nor another metadata generation may be reused. |
| **C02b** after merged C02a1 | One Python authoring service owner moves runtime/environment selection, interop plan/probe application, target inspection, distribution/protocol certification and package-diagnostic rendering needed by editor requests. Accept explicit package/environment snapshots and cancellation; retain declared-symbol identity, library deferral and read-only checks. Driver's build/CLI execution remains its own outer adapter. Do not reorder LSP fast-hit checks or own external-generation watching (E01). | `cargo test -p sifr_lsp --lib session::tests::python_declaration_tests::python_declaration_completion_hover_and_navigation_share_compiler_status -- --exact`; `cargo test -p sifr_lsp --lib session::tests::python_declaration_tests::certification_drift_is_checked_against_the_live_environment_digest -- --exact`; `cargo test -p sifr_lsp --lib session::tests::python_declaration_tests::cancelled_python_declaration_request_stops_before_probe -- --exact`; `cargo test -p sifr_driver --lib build::python_interop_tests::read_only_check_defers_library_target_without_an_environment -- --exact`. Add lower-service negative cases for wrong distribution/environment digest and uninspectable target; preserve package diagnostic codes and spans. |
| **C02c** after C02b | One SQL editor service owner moves profile discovery/preparation, `PreparedSqlProfiles` editor-facing state and profile-name import diagnostics into lower capabilities. Accept explicit package/profile/component inputs; keep build orchestration and native materialization in driver. Preserve lockfile-less deferral, initialization diagnostic isolation, component/schema identity and live overlay edits. No SQL language semantics or separate SQL phase implementation. | `cargo test -p sifr_analysis --lib host::tests::sql_editor_tests::lockfile_less_project_defers_sql_profiles_and_preserves_overlay_analysis -- --exact`; `cargo test -p sifr_analysis --lib host::tests::sql_editor_tests::configured_profile_import_diagnostic_tracks_editor_edits -- --exact`; `cargo test -p sifr_analysis --lib host::tests::sql_editor_tests::sql_templates_route_through_virtual_document_editor_queries -- --exact`. Add lower-service negative cases for missing lock, invalid profile/component and wrong import name, retaining source-backed diagnostics. |
| **C02d** after C02c | One preview/editor-check service owner supplies generated Rust preview and saved editor-check restore without analysis calling driver compilation or `project_cache`. Preview consumes the C01 frontend product with its SQL, specialization, adapter, flow, export, span and metadata identity; restore takes explicit semantic/cache owner and validates current source and metadata generations before reuse. Driver retains build/test orchestration and cache publication policy. No X02 emitted-Rust safety redesign, DX.5 metadata schema change or E02 lint/HIR reuse. | `cargo test -p sifr_analysis --lib host::generated_rust_preview_tests::generated_rust_preview_tracks_compiler_synthetic_source_map_entry -- --exact`; `cargo test -p sifr_analysis --lib host::generated_rust_preview_tests::generated_rust_preview_tracks_generated_support_source_map_entry -- --exact`; `cargo test -p sifr_driver --lib project_cache::tests::saved_check_policy_distinguishes_missing_empty_and_unserializable_paths -- --exact`; `cargo test -p sifr_driver --lib project_cache::package_reuse_tests::package_authority_changes_invalidate -- --exact`. Add negative cases for changed source/metadata/product identity, failed preview and corrupt/missing cached checks; no stale success or partial preview. |
| **C02e** after C02d | One dependency-guard owner removes the final direct `sifr_driver` dependencies/imports in analysis and LSP, including build identity and startup adapters, and extends `check_source_crate_dependency_direction.py` to resolve Cargo package identity behind aliases and scan feature-enabled, target-specific, build/dev dependencies plus `[lib]`, `[[bin]]`, `[[example]]`, `[[test]]`, `[[bench]]` and build-script source paths outside `src/`. Guard lower services against upward dependencies too; test/feature configurations may not bypass it. No unrelated V03 filesystem-effect classification. | `python3 scripts/check_source_crate_dependency_direction.py --self-test`; `python3 scripts/check_source_crate_dependency_direction.py`; `cargo test -p sifr_analysis --lib host::tests::all_editor_query_methods_expose_current_revision_metadata -- --exact`; `cargo test -p sifr_lsp --lib session::tests::python_declaration_tests::python_declaration_completion_hover_and_navigation_share_compiler_status -- --exact`; `cargo test -p sifr_driver --lib metadata_reader::generation_tests::installed_relocation_and_generation_switch_pin_all_sources -- --exact`. Negative self-tests insert direct and renamed `sifr_driver` dependencies, an optional feature/target-only alias, source imports through an alias, and a forbidden import in an explicit target file outside `src/`; each must fail while the positive fixture and current tree pass. |

For newly extracted service behavior, reserve these **exact new test names** in `sifr_compiler_services` (or the selected lower crate, with the same case names); each item runs its own names with `cargo test -p sifr_compiler_services --lib <module>::tests::<case> -- --exact` in addition to the existing commands above. C02a0: `metadata::tests::stale_or_corrupt_override_retries`. C02a1: `metadata::tests::incompatible_identity_rejects_provider`, `metadata::tests::replaced_cache_owner_does_not_reuse_generation`. C02b: `python::tests::wrong_distribution_or_environment_digest_rejects_certification`, `python::tests::uninspectable_target_remains_runtime_checked`, `python::tests::cancelled_probe_has_no_side_effect`. C02c: `sql_editor::tests::missing_lock_defers_profiles`, `sql_editor::tests::invalid_component_preserves_source_diagnostic`, `sql_editor::tests::wrong_profile_import_names_exact_target`. C02d: `editor::tests::changed_source_or_metadata_rejects_restored_check`, `editor::tests::corrupt_or_missing_check_is_a_miss`, `editor::tests::failed_preview_has_no_partial_output`. C02e's guard self-test must name and independently assert `direct-driver-dependency`, `renamed-driver-dependency`, `optional-feature-target-alias`, `aliased-source-import` and `explicit-target-outside-src`; run the exact guard self-test and current-tree commands above.

**Final C02 closure:** all six implementation PRs (C02a0, C02a1, C02b, C02c, C02d and C02e) must be merged in order with their own exact-candidate named tests, focused regressions and scoped review. On the resulting main, neither analysis nor LSP may declare or reference `sifr_driver` under any Cargo target/feature or source alias; the lower services have no upward dependency; driver remains the orchestration adapter; C01 product identity and editor behavior match the named positive and negative cases. C02e's guard must reject the listed mutation cases. Preserve distinct ownership: E01 owns Python fast-hit ordering and external input generations, E02 owns canonical HIR lint/action reuse, E03 owns measured editor scale, V03 owns filesystem effects, DX.5 retains metadata schema/decoder evidence, and X02 owns generated Rust safety. Q01 still owns whole-phase integration and review. No C02 item or whole-phase gate is claimed by this record.

## X01 coverage registry follow-up receipt (2026-09-23)

The SQL phase Item 2 named `coverage_matrix_readiness.py` check failed on its earlier current-main base `30136206c94ee03784c9db93b2544de72116be88`: `sifr: target lacks classification: test:legal_failure_method_names`. X01 introduced the target in [PR #3916](https://github.com/sifr-lang/sifr/pull/3916), but its Cargo metadata classification was omitted. The original SQL failure is preserved at `/home/yaser5/projects/sifr/sql-item2-evidence-20260923/coverage-matrix-readiness.log` (SHA-256 `48e8f082a5454db9a0a2de170c80c531231a94ccf8fb396dabde8b4d0a0a1e3d`); SQL Item 2 did not absorb or qualify this unrelated repair.

[PR #3940](https://github.com/sifr-lang/sifr/pull/3940) merged as `a141aa15a7d248b605420ea2438092f7aa5f6d10` from reviewed candidate `3c6c7c3411dab533c4d8a0821b00b95abaefd03b` (base `f31e79c3dc703efdc73edbafc6539de5eada708d`, tree `ae7cbcc6e9fc3f597f0b78048c172ba02a23c15a`). It classifies `test:legal_failure_method_names` as a `test_fixture` assigned to the merge profile, consistent with the other `sifr` integration targets. On the exact candidate, strict coverage readiness passed (raw log SHA-256 `2cec5c5176365897f41e225e2025921ef642ecee871a69ca327bf68506c2c741`); advisory coverage matrix (`0398ad40772f0d863d8e294b1f9764672a07323142175a9fd2d0a92c4b44fcc8`), readiness negative self-tests (58 cases, `5b1ba76460e237a2807bcd59c026a45601d1b2b04c94983216781f37019a9709`), profile assignment matrix (`a023719173c548c23734a0460d71238400af9bcc9c59bed5c4e366902142cfe2`), 900-line file-size guardrail (`06f18ed54c0fa6a16c0c2748f474f7904f025cdd960835b745eb106a777d77f8`), and diff check passed. The scoped Opus review returned **SATISFIED** with no blocking or follow-up findings (response SHA-256 `5d26491138129ee33e186debfa8c0487412ee25c251000779bd8f2e78998e61e`). Raw logs and review are under `/home/yaser5/projects/sifr/architecture-x01-coverage-registry-evidence`, keyed by candidate SHA. The full merge gate remains deferred to Q01 under this phase's intermediate-item policy. This follow-up ends here; SQL Item 2 must rerun its own named readiness check on its final candidate. The docs-only receipt passed documentation structure (raw log SHA-256 `d4e12f1bf8c85008bd1a0a66e6e112756b9cd5f9eb8696b1dfc81f7e6638cab0`) after initializing the pinned nested editor submodule; the initial missing-submodule setup failures are preserved.

## X01 C02a coverage registry follow-up receipt (2026-09-24)

The Emitted-Rust final candidate `eafa57e22df22d6d8172adf8c65aa5c42aea8acb` failed its exact merge-selected `coverage_matrix/readiness` selection solely because C02a0 introduced `sifr_cache_storage` and `sifr_compiler_services` without Cargo-package classifications. The failed JSON SHA-256 is `3417f0c1f555b9089a7cadd89cf702f15300a1d4632796dd38cfb360e68f4d3b`; it remains failed evidence. The earlier X01 target repair in [PR #3940](https://github.com/sifr-lang/sifr/pull/3940) provided the registry convention.

[PR #4024](https://github.com/sifr-lang/sifr/pull/4024) merged as `eb58977cfb26e4b682605864bc7531794e9fb62a` from reviewed candidate `0812ab592ffe1260f09d27534a8c20a11771f5c4` (base `af8b4bf551c8398a9cb80759541a4f8d3bccd998`, tree `4b732566ba464157ec5de747bbe9a5f38b8ab936`). Both packages, their exact Cargo targets, and the cache storage `test-support` feature now have permanent classifications. Full-mode blocking crate-test memberships are assigned across the delivery profiles; merge executes both compiler crates. On the exact candidate, the required `uv run --project verification --locked python -m sifr_verify areas run --area coverage_matrix --suite readiness` passed all four cases (strict readiness, profile assignment, 58-case negative self-tests, verification taxonomy; raw log SHA-256 `abccc8b658a85c2d8570686abc13ef4217112076d251b9891f7f86d221b6d490`). Focused advisory registry check (`25062ecc6f2b253c30e006f85dcc577374e32568dc4a875f7bae504671f5539b`), profile validation (`ed8badcc0f52226e25505b18c0664850159fe87425e14ea00b4e07c89bb59c74`), 900-line file-size guardrail (`cf3e51a2cc8d6b714bf05decbe551ded6bb37e821797058cbd318609d9e2bf0a`), HIR maintainability guardrail (`0371f7ff0407b48ec3c2b827e55899ed48ef9f856b1b3562c29558d86f65f945`) and diff check passed. The scoped Opus review returned **SATISFIED** with no blocking findings (response SHA-256 `79fbd1958960c0c7a5bfda4d9c0f448ac99006d7fdac6e163c2261e4ae31f4fc`); its performance-only compiler grouping suggestion belongs to DX.10. Candidate-keyed logs and review are under `/home/yaser5/projects/sifr/architecture-x01-c02a-registry-evidence`. The full merge gate remains with the final integration qualifier. This repair does not qualify the original Emitted-Rust candidate or change the V01 host-policy failure; the Emitted-Rust owner must rerun readiness on its final candidate. The docs-only record passed the documentation structure suite after initializing the pinned nested editor submodule; its initial missing-submodule setup failure remains preserved.

## Preparation/F33 closure receipt

The record-only crosswalk merged in [PR #3910](https://github.com/sifr-lang/sifr/pull/3910), merge d74a87eaa209e08986571bf027bd63342acba46c, from reviewed candidate 5888646741a98b2622437ac3b4a23980ec920fc4. The scoped [Opus review](https://github.com/sifr-lang/sifr/pull/3910#issuecomment-5785809006) returned SATISFIED with no blockers; response SHA-256 3b79c8dd77abb9d945a0d88b77bf7d62aa2ab978d4f896daf2ae54f98cd9b6ed. No implementation code or historical draft was merged.

Documentation structure suite passed 1 variant/0 failures on the reviewed candidate after initializing the pinned nested editor submodule; result SHA-256 42bbe33567b2b2bbfb3484b1539a1267d8d43e2d8926acf97aff6a6d340f1db8. Lowering maintainability and Git diff checks passed. The first documentation invocation failed only because that nested submodule was uninitialized; it is preserved as setup evidence, not recast as a pass. External review and result files are under /home/yaser5/projects/sifr/architecture-prep-f33-evidence, keyed by candidate SHA.

Deferred handoffs: Emitted Rust owner removes stale F7/#3898 blocker wording before its dependent qualification; D01 reconciles the active-named solo-maintainer record status and links. These are not P00 implementation changes. Next action is separately assigned X01 under the generated Rust owner. This session stops after the record-only receipt.
