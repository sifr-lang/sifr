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
