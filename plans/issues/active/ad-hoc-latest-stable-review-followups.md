# Latest-stable review follow-ups

Status: open follow-ups, non-blocking for the frozen implementation phase.

Origin: initial read-only Opus review of root candidate
`7d3c4198585352504a2d0c82267d6a0824df50cf`, PR #3827, 2026-09-14.
The blocking policy-test registration omission is handled in the
[phase record](ad-hoc-latest-stable-release-convergence.md).
These four observations were classified as suggestions, infrastructure, or
pre-existing coverage; they do not reopen the frozen version selections.

## Probe identity header coupling

Owner: Rust bridge probe validation.

The package-name replacement currently matches the production manifest header.
The identity unit fixture duplicates that header, so a future production-header
edit could leave replacement unapplied without invalidating that fixture.
A bounded follow-up should test the actual production manifest builder and make
an internal header mismatch explicit. The current real-Cargo contract rejection
and artifact separation regressions remain passing.

## Maintained-demo preparation workspace retention

Owner: verification setup and build-storage lifecycle.

`prepare_maintained_demo_cache` allocates a persistent temporary workspace on
each run. Decide which compact reports and current build inputs must remain,
then clean or reuse the other session-owned preparation trees. Preserve actual
validation receipts and respect worktree ownership; do not remove another
session's target or rewrite historical evidence.

## PostgreSQL dialect terminology

Owner: PostgreSQL build documentation.

The build selects `c17` for MSVC and `gnu17` elsewhere. Clarify the broad
C17 wording as C17 with the GNU dialect on non-MSVC targets. No language-mode
change or parser upgrade is requested by this documentation follow-up.

## Continuous base-crate source inventory coverage

Owner: SQLite native-source provenance.

The maintained SQLite checker validates the four patched `sqlite3/*` inputs.
The receipt retains the authenticated base-crate archive hash, and the original
update evidence records the comparison of the unchanged base files. That is
distinct from continuous whole-tree enforcement by the maintained checker.
Consider extending that enforcement to unchanged base inputs, including the
SQLCipher source and bindings. The selected maintained feature graph does not
enable `bundled-sqlcipher`; enabling it requires separate qualification.

## Item 76 optional runner follow-ups

Source: scoped SATISFIED review of 20dbb1e4579b7dd25139b13853f3ccb0f057b9f0,
external response SHA-256
a170a81b556a38c9a0f1167ad6b41cd6b72ff28048da0502b4903c424dc90d59.
These are nonblocking suggestions outside the qualified POSIX runner repair.

- Owner: sysroot verification portability. If these gates gain a Windows host,
  handle text-mode TimeoutExpired streams that may already be strings there.
  Current required sysroot hosts are POSIX.
- Owner: sysroot diagnostic presentation. Consider preserving leading stderr
  indentation when appending the timeout diagnostic.
- Owner: verification setup logging. Consider whether the source preparation
  marker should be printed by the parent, matching sibling preparers; the
  child currently emits the equivalent marker.

## Item 77 compiler maintenance observations

Source: scoped SATISFIED review of b9a23066373762e0ea2a91913e326e94f64ee186,
external response SHA-256
86485e48f9a66083dbee2b12a5e4a0f369b71702004660d487e0a767e59dced5.
These are nonblocking follow-ups; the stale byte dependency snapshot is owned
separately by Item 78 in the convergence phase.

- Owner: codegen test lint maintenance. The optional all-targets Clippy check
  reports eight items-after-statements warnings and one needless generic
  argument borrow in four files unchanged by Item 77:
  generated_rust_canonicalizer/member_demand/record_variant_tests.rs,
  generated_rust_canonicalizer/syntax_cleanup/mutability_cleanup_tests.rs,
  generated_rust_canonicalizer_capture_tests.rs, and
  lib_codegen_tests/corpus_repair_codegen_tests.rs. All are under
  crates/sifr_codegen/src. Production codegen Clippy passes.
- Owner: stdlib metadata invariants. Document that the imported structural
  closure requires canonical identities. Production stdlib templates always
  have them; any future non-stdlib caller must uphold that contract.
- Owner: shared structural ownership. Consider narrowing import-only shared
  registration to declarations with emitted contracts. Current non-opaque
  registration is correct for the reviewed fixture and existing source model.
- Owner: compiler invariant diagnostics. If per-consumer structural rendering
  ever stops being module-independent, replace the existing conflicting-body
  assertion with a diagnostic. No divergence is present in current rendering.

## Items 78–84 review observations

These suggestions do not block the merged repair or reopen frozen selections.
The exact response hashes and candidates are in the phase's merged-item record.

- Owner: sysroot snapshot maintenance. Keep the byte-boundary dependency map
  synchronized with maintained imports while preserving the strict comparison.
- Owner: CLI failure-test maintenance. Document the intended failure stage when
  isolating Cargo from rustfmt, so a future prerequisite cannot mask that stage.
- Owner: phase record editing. Improve missing word spacing in historical prose
  when that prose is next edited; preserve the dated evidence and outcomes.
- Owner: callback codegen coverage. Add shared-borrow byte/list transport cases
  in a separate coverage item if useful. The review suggested coverage; it did
  not demonstrate a current defect in these unmodified paths.
- Owner: callback validation speed. Consider enrolling the negative capture
  fixture in a faster targeted selection without removing full integration checks.

## Mint importer README preservation

Origin: Item 34 SATISFIED review, candidate
`caa2174f0f4681710ec1459e8d3d0110153bbafb`.
Owner: documentation importer maintenance.
The existing README heredoc omits the unrelated Deployment and Assistant
Instructions sections. A future importer run could remove those sections.
Preserving them is a separate bounded follow-up, not part of exact Mint pinning.

## VSIX package size

Origin: native qualification 34895961655 at the final implementation candidate.
Owner: editor packaging maintenance.
The package succeeds with an advisory about 370 files/188 JavaScript files.
Consider bundling in a separate packaging task; no package policy was relaxed.

## Item 85 portable lockfile review observations

Origin: SATISFIED scoped review of
`5d7c6eeeded8e18d18d113aa4493a12f076e719c`, response SHA256
`13d5765f0fa0e7cd5a1ab67314f0ecb085bf27e3e0c7841a045f357d894ea17d`.

- Owner: portable sysroot dependency identity. If a published nonlocal runtime
  package is introduced in a future supported graph, consider tightening the
  optional-edge identity discriminator. The reviewed graph cannot reach that
  case and current missing-required-package checks fail closed. This suggestion
  does not authorize a compatibility path or a newly published runtime.
- Owner: verification build timing and storage lifecycle. Create-PR gate 11
  retained exit 124 for cold E2E compilation at 667.118/600 seconds despite all
  functional cases passing. The unchanged warm selection passed in 12.206
  seconds with all 46 cache hits. Preserve both results and investigate cold
  build preparation separately; do not relax thresholds or count the warm
  subset as a substitute for the canonical full merge gate.

## Item 86 — generated differential preparation follow-ups

Source: scoped Opus review of `26ae861e9f26f83fb6c052551c726cbb2cfb768f`,
response SHA256 `d4a3657b39e7706717a98680bb06a80f49258c908033747fa3ec8ebbbc48b727`.
Verdict SATISFIED, no blockers, initial review 1/remediation 0. These are
nonblocking future verification-runner work, outside the frozen convergence
batch and its final closure criteria.

- Infrastructure: give selected native-program preparation an aggregate
  deadline or fail-fast policy. Each case is bounded by 300 seconds, but the
  current loop can consume that limit for every selected case after failures.
- Suggestion: keep compiler verification within preparation uniformly locked
  and offline, or pass the already verified compiler record to the preparation
  helper. The current second verification follows a successful locked/offline
  build on an unchanged graph and takes approximately 0.4 seconds.
- Suggestion: preserve the oracle minimization artifact path when a behavioral
  regression is detected by preparation. Actual preparation output/status/time
  is retained, but preparation failure currently stops before oracle shrinking.

## Item 87 scoped review follow-ups

Review of 106bceb4813cdf7a847faace17308bf132cfa376 for PR 3833: initial 1, remediation 0, SATISFIED with no blockers. Response SHA256 85a6dfebf0205a058483975074a41c8734e1eea28902dc13bf3ad35b9e5fdf55.

- Negative Clippy controls reuse a fixed-name root outside positive-case invalidation. Each control asserts its own lint; consider applying uniform root invalidation in a separate verification-maintenance item.
- Document the definition-only invariant of rust_source_defined_item_names and separately assess the existing impl-only empty-residue early return.
- Consider preserving inner module attributes in root-binding import round trips and stopping after the unique compiler-owned nominal module. These were suggestions, not identified regressions.
- Preserve PRgate2 cold E2E budget failure (665.160 s, 0/46 cache hits, all 143 functional passes) and exact-source warm pass (11.783 s, 46/46 hits). Do not relax budgets or relabel the cold result.
- Track the 14 pre-existing optional all-targets Clippy test-style findings under their separate test-maintenance owner. Canonical production component Clippy passes.

Full merge1 and native qualification35021268957 pass at this source. Release1 passes generated quality then fails on stale package demo lockfile digests; Item88 owns that repair.

## Item 87 — optional test-target Clippy observations

An additional `cargo clippy -p sifr_codegen -p sifr_lowering --all-targets -- -D warnings`
check reported 14 pre-existing test-only diagnostics in unchanged source on the
Item 87 M6 working candidate. The canonical workspace lint step uses
`cargo clippy --workspace -- -D warnings`, without `--all-targets`; this extra
check is not a release/merge gate requirement and is not an external blocker.

Owner: compiler test maintainability. Follow up separately on five `borrow_as_ptr`
findings in `crates/sifr_lowering/src/lower/external_defs_context_tests.rs`,
eight `items_after_statements` findings across codegen record-variant,
mutability-cleanup and corpus-repair tests, and one
`needless_borrows_for_generic_args` finding in the codegen capture tests.
No Item 87 implementation or new regression-test path was diagnosed.

Preserve the failed extra check at
`20260915-generated-support/compiler-clippy1.log` and `.exit` (101),
without relabeling it as a pass or expanding the current generated-Rust repair
scope. Record the required compiler lint result separately at its actual final
source.

## Item 88 scoped review follow-ups

Initial 1/remediation 0 review of 700105f938 is SATISFIED, no blockers.
Preserve the cold PR E2E budget failure and warm pass separately. Disk pressure
was real; the review's inference that the target belonged to another worktree
was incorrect. This session owns it. Unused older revision artifacts were
removed after process-use checks, preserving current caches and generated inputs.
The registered Item 88 state is now reconciled with PR 3835 delivery. The final
release report identity failure is separately owned by Item 89.
