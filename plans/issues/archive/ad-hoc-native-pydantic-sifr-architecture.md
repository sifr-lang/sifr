# Ad Hoc Phase: Native Pydantic-Sifr

## Design Reference

The durable design and architecture are defined in
[`native_pydantic_sifr_architecture.md`](../../../internal_docs/native_pydantic_sifr_architecture.md).
This archived phase document owns milestone planning, delivery tracking,
validation and review evidence, blockers, deferred follow-up work, and closure.

## Status

Architecture proper was approved on draft PR
[#3014](https://github.com/sifr-lang/sifr/pull/3014). agent pass 17 returned
`SATISFIED` and approved `milestone_ps_0`. The architecture, conformance
inventory, repository boundary, and demo ownership are approved.
`milestone_ps_1` and `milestone_ps_2` are implemented and merged. The required
`certification_pkg_resource_core` item is complete through merged
[PR #3123](https://github.com/sifr-lang/sifr/pull/3123). `milestone_ps_3` is
implemented and merged through [PR #3138](https://github.com/sifr-lang/sifr/pull/3138).
`milestone_ps_4` through `milestone_ps_11` are implemented and merged in the
companion repository. The package-neutral compiler work is merged. Blocked
callback and temporal rows remain in their owning companion issues. Final
implementation review and both companion gates are complete. Installed
release-artifact acceptance is skipped under the phase rule because the Sifr
release sysroot omits its required structural-identity crate; Sifr issue #3233
owns that package-neutral release defect.

Review artifacts:

- [`native-pydantic-sifr-architecture-agent-review-pass-1.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-1.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-2.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-2.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-3.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-3.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-4.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-4.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-5.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-5.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-6.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-6.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-7.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-7.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-8.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-8.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-9.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-9.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-10.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-10.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-11.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-11.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-12.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-12.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-13.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-13.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-14.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-14.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-15.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-15.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-16.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-16.md)
- [`native-pydantic-sifr-architecture-agent-review-pass-17.md`](../../reviews/archive/native-pydantic-sifr-architecture-agent-review-pass-17.md)
- [`native-pydantic-sifr-ps2-agent-review-pass-1.md`](../../reviews/archive/native-pydantic-sifr-ps2-agent-review-pass-1.md)
- [`native-pydantic-sifr-ps2-agent-review-pass-2.md`](../../reviews/archive/native-pydantic-sifr-ps2-agent-review-pass-2.md)
- [`native-pydantic-sifr-ps2-agent-review-pass-3.md`](../../reviews/archive/native-pydantic-sifr-ps2-agent-review-pass-3.md)
- [`native-pydantic-sifr-ps3-agent-review-pass-1.md`](../../reviews/archive/native-pydantic-sifr-ps3-agent-review-pass-1.md)
- [`native-pydantic-sifr-ps3-agent-review-pass-2.md`](../../reviews/archive/native-pydantic-sifr-ps3-agent-review-pass-2.md)
- [`native-pydantic-sifr-ps3-agent-review-pass-3.md`](../../reviews/archive/native-pydantic-sifr-ps3-agent-review-pass-3.md)

Milestone delivery records:

- `milestone_ps_1` merged in [PR #3104](https://github.com/sifr-lang/sifr/pull/3104)
  at merge commit `7cc7f714e844e851a0ffe665fd18c669e0e21b99`; the reviewed candidate was
  `e23b80d94f67de3d3ced7dbcca7394efdf5ab6c1`.
- Validation passed workspace formatting/clippy, diagnostics/frontend/lowering/stdlib
  suites, the non-E2E CLI suite, diagnostics rules and baselines, file-size and HIR
  guardrails, two selected native E2E fixtures, and the positive/negative external
  specialization fixtures. The one merge gate reached the CPython differential lane;
  its first Sifr process timed out at 240 seconds under host-wide Cargo contention,
  while the remaining cases passed. The exact timed-out fixture then passed directly
  in 29.6 seconds with canonical output `[7,3,2,4,20]`.
- agent review of candidate `5b1601ded66556fe04b9674916153726b341b2c6`
  found augmented-assignment token handling and negative integer floor arithmetic
  omissions. Candidate `e23b80d94f67de3d3ced7dbcca7394efdf5ab6c1` corrected both;
  remediation review returned `SATISFIED` with no blocking findings.
- The `milestone_ps_2` contract candidate also repairs a PS1 test-adapter escape:
  the `cfg(test)` driver frontend adapter omitted five `LoweringResult` metadata
  fields added by PS1. The adapter now propagates the complete result, and all
  19 targeted `sifr_driver` Python-interop tests pass.
- The `milestone_ps_2` contract wave merged in
  [PR #3107](https://github.com/sifr-lang/sifr/pull/3107) at merge commit
  `44571561309afaabb7afb419804aa2cc00362193`; the reviewed candidate was
  `77442349c745ae1ad6ad1592be129572e37fb65c`.
- That wave accepted one unversioned structural Rust bridge contract and a
  repository-wide atomic removal plan for `[rust] bridge-version`, with no
  compatibility mode, rewrite, shim, fallback, or active `v2` name. It added
  separately gated future-owned evidence for structural calls and removed-field
  rejection without claiming implementation support.
- Contract validation passed formatting, 19 focused driver tests, all 10
  registered Rust-interop cases and their self-tests, the active-source naming
  sweep, and file-size/lowering maintainability guardrails. Earlier create-PR
  execution passed core, diagnostics, package, and stdlib lanes; its aggregate
  Python lane reproduced a host-contention timeout for a scenario that passed
  standalone.
- agent reviewed eight published remediation candidates. The final exact-SHA
  round confirmed all prior construction, identity, callback, projection,
  cutover-inventory, ownership, and current-state findings closed and returned
  `SATISFIED` with no blocking findings.
- The contract record accidentally leaked phase-delivery taxonomy into active
  Rust-interop fixtures, compatibility rows, and durable architecture surfaces,
  blocking the repository taxonomy guard on current `main`. The narrow repair
  merged in [PR #3112](https://github.com/sifr-lang/sifr/pull/3112) at merge
  commit `2452dca175d7b6b01068c40b3853c1d7b6d251f6`; its reviewed candidate was
  `087a399fd0d89f3066af04a26d4185c03497283f`.
- That repair moved the two future-owned compatibility rows to the exact durable
  Rust-interop architecture owner without weakening concrete-path or existence
  validation. Taxonomy and compatibility checks/self-tests, all 10 registered
  Rust-interop cases, stable claims, stale drafts, tiers, and file-size/HIR
  guardrails passed. agent review round 3 returned `SATISFIED` with no actionable
  findings. The create-PR profile then reached the externally owned Python
  readonly/doctor timeout tracked by PR #3110; #3110 was blocked on this taxonomy
  repair, so the fix merged to close the dependency cycle without absorbing its
  code.
- `milestone_ps_2` merged in [PR #3114](https://github.com/sifr-lang/sifr/pull/3114)
  at merge commit `fc2e7b29244e08399f6b59b60a927ea740af5c5b`; the reviewed candidate was
  `7c0aaebdb3c0c752a513c8d21735e67dd453678a`.
- The milestone completed the unversioned structural Rust bridge cutover,
  `sifr.meta.Structural` marker validation, safe structural construction and
  projection, typed callback generation, package-resource support, and the
  native-backed `re.Pattern` contract. It retained class generic bounds and
  excluded nominal newtypes from implicit structural eligibility. No legacy
  bridge mode, compatibility path, fallback, or versioned active name remains.
- Focused validation passed 972 codegen tests; 971 lowering tests with one
  ignored test; generated-runtime, abort-profile, generic-bound, newtype,
  collection, regex, formatting, HIR, file-size, taxonomy, coverage, readiness,
  profile, stable-claim, and all registered Rust-interop checks. The final merge
  lane passed every PS2-owned check and 23 of 25 Python-interop variants. The two
  failures were the externally owned PR #3110 readonly/doctor 120-second timeout
  and the host-sensitive binding-authoring 180-second timeout; the same binding
  path passed in 36.431 seconds with the controlled six-worker allocation.
- agent remediation passes closed recursive identity/cache and panic-contract
  gaps, restored generic class bounds, and removed duplicate stable-support
  claims. The final exact-candidate review returned `SATISFIED` with no blocking
  findings; per the reviewer skill, that final response remains outside the Git
  tree.
- `certification_pkg_resource_core` merged in
  [PR #3123](https://github.com/sifr-lang/sifr/pull/3123) at merge commit
  `c6f5748870471ba6baa7dcddbbadc1b627576140`. The reviewed candidate was
  `ad9e73fc6c589c9d25a8177b734b6d820ef63d9a`.
- The item certifies `opaque_resource_package_core` through a synthetic
  external package. It covers construction, lifecycle, panic redaction,
  direct-construction rejection, alias invalidation, and use after close.
- The exact-SHA agent review returned `SATISFIED` with no blocking findings.
  The one merge gate passed Python 25/25, Rust interop 10/10, generated builds
  70/70, E2E 694/694, and 268 hardening variants with zero failures.
- `milestone_ps_3` merged in
  [PR #3138](https://github.com/sifr-lang/sifr/pull/3138) at merge commit
  `2174be634d7d3f9e0053ff893dc9a1d2fc34f64d`. The exact reviewed and gated
  candidate was `1b1955f681ccce33b2ca65b130f381ef3e9f27ca`.
- The milestone adds deterministic compiler-owned static-program identities and
  bytes, a sealed typed program envelope, checked compact node indices, and a
  move-only structural arena. The synthetic package uses one generic structural
  signature probe and one monomorphized executor call. It proves exact and fixed
  integers, bytes, collections, records, moved payloads, typed errors, corrupt
  envelopes, invalid indices, cleanup, cache invalidation, and source/installed
  parity. No compatibility layer, fallback, or legacy static-program path exists.
- Three recorded agent review passes found and closed unsupported-owner emission,
  duplicate decorator handling, cache-order and cleanup evidence, and nested bytes
  encoding. A fresh exact-integration review then returned `SATISFIED` with no
  blocking or actionable milestone-owned findings; per the reviewer skill, that
  final response remains outside the Git tree.
- The unchanged warm create-PR gate passed. Its preceding cold run was
  functionally green and exceeded only the known external cold-artifact budget
  tracked by issue #3134. The warm receipt SHA-256 is
  `270742a07e27aa2cc704dd25693f93b64ef29b47e10f058037664b608f6db0c5`.
- The one authoritative merge gate exited zero. It passed Python interop 25/25,
  Rust interop 10/10, generated builds 72/72, E2E 694/694, hardening 268 variants
  with zero failures, distribution 66/66, sysroot 2/2, generated-code quality
  7/7, and all representative performance checks. The merge receipt SHA-256 is
  `caa0dc08aa5d6c855fcd49edd5516cc33b004d59d2f879e7a9443815b83b267e`.
- `milestone_ps_4` merged in companion-repository
  [PR #1](https://github.com/sifr-lang/pydantic-sifr/pull/1) at merge commit
  `4b2a1969022e7ab5220e036016a8340d96dba647`. The reviewed and gated candidate
  was `ade63892f5536a6e5cc1c52576ae0d499849ee7b`.
- The milestone established the public `sifr-lang/pydantic-sifr` repository,
  the Sifr package, and the Python-free `pydantic_sifr_core` backend. It pins
  Pydantic commit `f59e929c999e8b2efc7b12fd0bc1685c1a186be3`, tracks 310 source
  files and 12,754 API/Core nodes, and excludes the historical standalone Core
  checkout.
- Format 1 uses one deterministic Sifr canonicalizer and semantic verifier.
  The compiler seals its exact bytes, program identity, and shape identity.
  The Rust core checks only that envelope. It does not parse or verify the
  schema graph again, and it has no compatibility, fallback, or legacy path.
- The milestone also added the exact 53-schema-kind and four-field-kind ledger,
  nine unavailable dispositions, compositional error declarations, checked
  JSON/input arenas, plan foundations, Python-free jiter, licenses, provenance,
  property tests, two fuzz targets, and a benchmark harness.
- The final agent remediation review returned `SATISFIED` with no in-scope
  blocker. The canonical create-PR and authoritative merge gates passed. The
  merge gate ran 4,096 release property cases, compiled both fuzz targets, and
  left only a stale tracked fuzz lockfile from removed production dependencies.
- The exact one-file lock correction merged in companion-repository
  [PR #2](https://github.com/sifr-lang/pydantic-sifr/pull/2) at merge commit
  `c8200c9ae67e3b504674ea105836b4894413507b`. Its reviewed and gated candidate
  was `7185f538a57eb54a74f87f9d4d7ae2e8fcbfb387`; agent returned
  `SATISFIED`, both locked fuzz builds passed, and the create-PR and merge gates
  passed with a clean worktree.
- The released-Sifr round trip binds 689 exact static-program bytes to identity
  `d6591c059d855809f03be42c991d73a91a42e50c6c141810b3e6195c8efdca72`.
  Two invalid schema probes return stable `schema_invalid` diagnostics. JSON
  and schema foundation tests pass with no user-input panic path.
- `milestone_ps_5` merged in companion-repository
  [PR #3](https://github.com/sifr-lang/pydantic-sifr/pull/3) at merge commit
  `0ce1aecfd48af77f52b05578de10512a05914707`. The exact reviewed and gated
  candidate was `e09014c4926746442c4d174c09cac318500a48eb`.
- The milestone adds one bounded engine for native, JSON, and strings input.
  It validates exact and fixed integers, floats, decimals, fractions, complex
  values, text, bytes, temporal values, UUIDs, URLs, and compiled patterns. It
  also validates lists, tuples, mappings, sets, frozen sets, embedded JSON, and
  lazy iterators with stable deferred error locations.
- The exact compatibility ledger classifies all 19 required PS5 fixture
  families. It records the Rust-regex, numeric conversion, collection-kind,
  temporal, URL, bytes, and error-contract adaptations. Integer digit limits
  fail before large numeric allocation. The implementation has no Python,
  compatibility, fallback, or legacy runtime path.
- Four scalar review rounds, four collection review rounds, and three special
  scalar review rounds closed all blocking findings. The whole-milestone agent
  review of the final candidate returned `SATISFIED` with no blockers. Its
  response remains outside the Git tree at
  `/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr-agent.CB0DN1/response.md`.
- The canonical create-PR gate and the single authoritative merge gate passed
  on the same candidate. The merge gate included release-mode tests and 1,000
  bounded runs for each scalar, collection, and special validation fuzz target.
- agent follow-ups are non-blocking documentation and coverage work: expand the
  ledger prose for adapted temporal bounds, tuple arity errors, typed numeric
  collection identity, numeric coercion boundaries, and URL measurement; grow
  fuzz coverage beyond the required bounded crash smoke.
- The general typed static-program payload dependency for `milestone_ps_6`
  merged in [PR #3148](https://github.com/sifr-lang/sifr/pull/3148) at merge
  commit `027d1b63ef716b8dde1bffbc853a0224f0706c4a`. The exact reviewed candidate
  was `160e6b789349b20ae02df59ede2b4eb78929e561`.
- The compiler now emits one immutable borrowed `StaticProgramValue` for a
  structural static-program owner. Non-structural specialization does not add
  a runtime dependency or generated payload. No parser, cache, compatibility
  mode, fallback, legacy path, or versioned active name exists.
- The final agent implementation and evidence review returned `SATISFIED` with
  no blocker. The unchanged warm create-PR gate passed. The single merge gate
  stopped on one parse-only performance sample at 0.18% more than its limit.
  That benchmark cannot reach the changed path. Five controlled reruns passed
  with low variance.
- Exact-candidate continuation passed all crate tests and 694 E2E fixtures. It
  also passed 182 diagnostics, 17 project/workspace, five regression, 37
  fuzz/property, and 20 ecosystem hardening variants with zero failures.
- The review found a separate pre-existing ambiguity in structural canonical
  metadata. [Issue #3149](https://github.com/sifr-lang/sifr/issues/3149) owns
  that repair. PR #3148 did not absorb it.
- The structural keyword-field prerequisite merged in
  [PR #3152](https://github.com/sifr-lang/sifr/pull/3152) at merge commit
  `e98776bf6ce7eac1a149ced8203f4b8669fa0d3b`. The exact reviewed candidate
  was `6c7cbdbdeff40dc6a9330f4e13c70e0f36149380`.
- Generated structural construction and projection now render Rust keyword
  fields through the canonical identifier renderer. The structural wire name
  stays unchanged. A field named `type` proves construction and projection.
- The exact-candidate agent review returned `SATISFIED` with no blocker.
  Focused structural tests, all 984 codegen tests, codegen clippy, formatting,
  and file-size checks passed. The warm create-PR gate passed Python 19/19,
  Rust 10/10, and E2E 140/140.
- The one merge gate passed CPython differential 4/4, Python 25/25 with zero
  read-only mutations, Rust 10/10, and developer tooling 32/32. It stopped on
  one unrelated incremental frontend instruction sample at 0.20% more than
  its limit. Five independent controlled work-mode runs then passed the
  unchanged budget. Their 20-sample medians were 924.71 million to 925.85
  million instructions, with instruction variation below 0.14%.
- [Issue #3151](https://github.com/sifr-lang/sifr/issues/3151) owns the
  non-blocking reserved-root and synthesized-local collision hardening found
  during review. PR #3152 did not absorb it.
- The static structural return prerequisite merged in
  [PR #3155](https://github.com/sifr-lang/sifr/pull/3155) at merge commit
  `2e607c24d4b0731a51b3f0ae484b7728c3fa3b08`. The exact reviewed candidate
  was `776f7ff49c4855cb8f788b3a0136ba7115103151`.
- Generated structural records now expose their compiler-owned nominal
  identity through `StructuralType`. Return-only `T: StaticProgram` bridges
  use that identity without parsing schema data.
- Structural bridge demand enables one sysroot `sifr_runtime` identity in the
  probe and final generated crate. Ordinary builds do not enable the feature.
- The exact-candidate agent review returned `SATISFIED` with no blocker.
  Runtime, codegen, and probe suites passed 71, 984, and 28 tests.
- A companion integration proof passed specialization, probing, release
  compilation, arena construction, and typed value recovery. The recovered
  value was `7`.
- The warm create-PR gate passed. The first run passed all generated-code cases
  but exceeded the existing cold-artifact budget tracked by issue #3134.
- The single merge gate passed CPython differential, Python 25/25, Rust 10/10,
  developer tooling 32/32, and all five representative instruction budgets.
- The gate stopped when the formatter control rejected three unstable samples.
  [Issue #3120](https://github.com/sifr-lang/sifr/issues/3120) owns that external
  host-control condition. The unchanged gate was not repeated.
- [Issue #3154](https://github.com/sifr-lang/sifr/issues/3154) owns the
  non-blocking review and infrastructure follow-ups. PR #3155 did not absorb
  them.
- The package error-export prerequisite merged in
  [PR #3157](https://github.com/sifr-lang/sifr/pull/3157) at merge commit
  `cefa3eb2bd951dc814dfb3091c0339a4f80fe20d`. The exact reviewed and gated
  candidate was `3d00cb2fd71d4fc5d93b467ef49ef789bf9c3350`.
- Imported package-defined `Error` classes now keep their error status through
  public aliases and two-hop facade re-exports. Error markers are scoped by
  module and class, so an ordinary class with the same name does not inherit
  error behavior.
- Focused validation passed a native two-hop build and run, 459 driver tests,
  66 frontend tests, 976 lowering tests, affected-crate Clippy, formatting,
  maintainability, and file-size checks.
- The exact-candidate agent review returned `SATISFIED` with no blocking
  finding. The response remains outside the Git tree at
  `/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr-agent.fvHK1U/response.md`.
- The cold create-PR gate was functionally green and exceeded only the known
  generated-artifact cold budget tracked by issue #3134. Its receipt SHA-256
  is `7e3e43028447de5fa4cf4ed842b7530e81a066dbd604da6bcc05bf536e1b497c`.
  The unchanged warm create-PR gate passed; its receipt SHA-256 is
  `e58f8ef4a129d8b419535a1397a05160b486988c85ea498784c6f81514f6a79a`.
- The single authoritative merge gate exited zero. It passed Python interop
  25/25, Rust interop 10/10, representative performance 8/8, distribution
  66/66, and sysroot 2/2. Generated-code quality passed 7/7, and generated
  driver builds passed 74/74. E2E passed 694/694, and 268 hardening variants
  passed with zero failures. The merge receipt SHA-256 is
  `8390cef246f579a864f3676476cfee52b03e87d7421edabffc807e153c217a2d`.
- [Issue #3158](https://github.com/sifr-lang/sifr/issues/3158) owns the
  non-blocking reviewer suggestions for future stdlib alias propagation,
  helper consolidation, and direct cross-package coverage. PR #3157 did not
  absorb them.
- The composite-union prerequisite merged in
  [PR #3160](https://github.com/sifr-lang/sifr/pull/3160) at merge commit
  `ded05e97aba1c0fda3923c6fbd5e1a12f65ffb62`. The exact reviewed candidate
  was `a42d1806e496df0f959011db7659c52f38959903`.
- The compiler now keeps canonical identity for cross-module classes,
  protocols, newtypes, enums, and their unions. Binary and test-project code
  generation share the required union and nominal imports.
- A Sifr `main` support module now has an explicit generated test-crate path.
  Native tests cover two error types, `Result` unions, and composite unions.
- Ten agent review rounds closed duplicate imports and test-crate root mapping.
  The final whole-diff review returned `SATISFIED` with no blocker.
- The warm create-PR gate exited zero. Its receipt SHA-256 is
  `4cc51486c1a3ee528f8ce1e44f77a4e3e2ad8374d89d8458430e679f5e09ed83`.
- The single merge gate passed all executed functional lanes. Python interop
  passed 25/25, Rust interop passed 10/10, and developer tooling passed 32/32.
  The merge receipt SHA-256 is
  `76d95ad391c775e2aec22db093e83ba4a99937ebce4acb8bb9709e4995522873`.
- One frontend-query work sample used a stale local helper artifact and failed
  its instruction limit. A fresh governed build of the same candidate measured
  918,238,900 median instructions against a 936,811,698 limit.
- The unchanged budget checker accepted that fresh receipt. Its SHA-256 is
  `2baf1d90deb12990497132ef9ddd2f71966063dca1cc8fb11d4dbb83b9ae997b`.
  agent accepted the combined evidence without a second merge gate.
- [Issue #3161](https://github.com/sifr-lang/sifr/issues/3161) owns exact-artifact
  performance isolation. [Issue #3162](https://github.com/sifr-lang/sifr/issues/3162)
  owns non-blocking nominal and test-layout hardening.
- The distinct structural-generic prerequisite merged in
  [PR #3164](https://github.com/sifr-lang/sifr/pull/3164) at merge commit
  `76c3bcb10bc2a28940003dd9e1b1f92506b72d07`. The exact reviewed candidate
  was `7ed8e0d95f33859f86ca4033b139db228f3e3ff2`.
- Structural bridge functions now accept multiple generic parameters with
  separate `Structural` and `StaticProgram` bounds. The compiler keeps their
  canonical HIR order and applies each generated trait bound to its owner.
- Focused lowering, code generation, driver probe, native fixture, Clippy,
  formatting, maintainability, taxonomy, and file-size checks passed. The
  Rust-interop area passed 10/10.
- The warm create-PR gate exited zero. Its receipt SHA-256 is
  `e877039a90709e39e1950af2b47111e8acdfcc4d2c2891cdffe476dd08f6d229`.
  Python passed 19/19, Rust passed 10/10, and E2E passed 140/140.
- The single merge gate passed all executed functional lanes. It stopped on
  the stale-helper performance isolation problem that issue #3161 owns.
- A fresh exact-candidate target measured 925,520,586 median instructions
  against a 936,811,698 limit. Its accepted receipt SHA-256 is
  `dfe27a5f69dfb818a1506ef7f4baa6900fcd01dc05884a3fd42c96058f3b871e`.
- Exact-SHA agent implementation review and validation adjudication returned
  `SATISFIED` with no blocker. The adjudication response SHA-256 is
  `2db609c6aad0be3061f79e1d2b70d2ed4f95f6c0939af67a472773e6e777eae0`.
- The locked-package authority prerequisite merged in
  [PR #3166](https://github.com/sifr-lang/sifr/pull/3166) at merge commit
  `0e61c9889418a009b8dd611ecb005d5b27ca749f`. The exact reviewed candidate
  was `2a10cd9d9fec1c7cbc183171f6b0900a33e09198`.
- Prepared Cargo entries now use the union of package and sysroot locks as
  exact authorities. Initial resolution still gives the package lock priority.
- Unknown exact entries remain rejected. The fix adds no unlocked operation,
  compatibility path, or fallback.
- Focused Cargo-resolution tests passed 5/5. Clippy, build, format,
  maintainability, taxonomy, and the transfer guardrail also passed.
- The unchanged PS6 dependent package passed `fetch --locked` and
  `run --locked` with the candidate compiler.
- Exact-SHA agent implementation and remediation reviews returned `SATISFIED`
  with no blockers. The final response SHA-256 is
  `662bee48ebf47ce30a28473add6d9ef74fadb3e0f5131ff577ca28ce03dcd30b`.
- The cold create-PR gate was functionally green. It exceeded only the known
  generated-artifact budget from issue #3134. The unchanged warm gate exited
  zero with receipt SHA-256
  `9faa775c38f17fd0245003f4850784f856b85e2cddeb3b1fbc2d64495a40b5c1`.
- The single merge gate passed all functional lanes that it executed. It then
  reproduced the stale frontend helper problem from issue #3161.
- A fresh exact-candidate helper measured 925,032,774 median instructions
  against the unchanged 936,811,698 limit. The budget checker accepted receipt
  `8eef6f115108db21c7933ab5431d6c788f5a288679c973fd145547f71db59212`.
- agent accepted the combined validation evidence without a second merge gate.
  Its adjudication response SHA-256 is
  `5330a6e652cb1cd2d661c22f93ed5236b1601a6baa4a252dffaf3aa2deff878d`.
- The multiversion lock-authority prerequisite merged in
  [PR #3169](https://github.com/sifr-lang/sifr/pull/3169) at merge commit
  `89684aecbd2a321c92912a8cee051b9d9a4fc46a`. The exact reviewed candidate
  was `ed01b930307a9cca1074d4587df645fd5ca8a5c3`.
- Cargo lock seeding now replaces versions within one Cargo-compatible semver
  family. It preserves semver-incompatible versions from each authority.
- Exact prepared-entry validation remains fail-closed. The cache identity was
  advanced so name-wide seed results cannot be reused.
- Focused Cargo-resolution tests passed 7/7. Clippy, build, format,
  maintainability, taxonomy, file-size, and transfer guardrails passed.
- The unchanged PS6 demo passed `fetch --locked` and `run --locked`. Its
  release binary built without an unlocked retry or fallback.
- Exact-SHA agent implementation review returned `SATISFIED` with no blocker.
  Its response SHA-256 is
  `ad130714c6981f6243de4735f5ccefad1402587db3e8e7b4c95283e576088e5c`.
- The canonical create-PR gate exited zero. Its receipt SHA-256 is
  `d3e4263e11d18d89d2f5703d0373ec008fa542ec0254922e7f9cdb030fa73721`.
- The single merge gate passed every executed functional lane. It reproduced
  the stale frontend helper problem from issue #3161.
- A fresh exact-candidate helper measured 925,442,120 median instructions
  against the unchanged 936,811,698 limit. Its accepted receipt SHA-256 is
  `82bec880219baf54c7c3408e45651fee03428db4f813a88ed2e45ed385b82710`.
- agent accepted the combined validation evidence. Its adjudication response
  SHA-256 is
  `6ffc165d38afb41b380ac6bf0fbb7d19e080b843ebad9d5be5f8747c9fe13ceb`.
- `milestone_ps_6` merged in companion-repository
  [PR #4](https://github.com/sifr-lang/pydantic-sifr/pull/4) at merge commit
  `db98820655d0a6c50565e9b51a9c63a0008a7001`. The exact reviewed and gated
  candidate was `d2817e7ac267d842f32e7f996bec1c607e02849d`.
- The milestone validates model schemas, required, defaulted, and nullable
  fields, aliases and alias paths, and all extra-field policies. It constructs
  typed Sifr classes directly from the validated arena. It does not create a
  third model tree.
- Native, JSON, and strings entry points use one validation engine. They keep
  exact integers and move scalar values once. Mapping and set normalization is
  deterministic. Aggregate errors have stable codes and locations.
- The dependent demo, PS6 compatibility ledger, static-program identity
  fixture, typed-construction fuzz target, architecture text, and locked CI
  gate are included. All three Cargo graphs use exact Sifr merge
  `89684aecbd2a321c92912a8cee051b9d9a4fc46a`.
- The final whole-milestone agent review returned `SATISFIED` with no blockers.
  Its response SHA-256 is
  `f6aea6737e4284f0675c3579212602b89fa7d1e19a7f41940e4342013a45f0d5`.
- The canonical create-PR profile passed. The single merge gate also passed.
  It included release property tests and 1,000 bounded runs for each of four
  fuzz targets. The static-program round trip kept 1,464 exact bytes and
  identity `c24a89ff2f2a7d98471a23db41b127ca624c55eac318b04ce2788cca0841457c`.
- The implementation has no Python production dependency, compatibility path,
  fallback, legacy path, or versioned public API name.
- The `milestone_ps_7` structural-sum prerequisite merged in Sifr
  [PR #3173](https://github.com/sifr-lang/sifr/pull/3173) at merge commit
  `6bcce3876bdf4f07fab00c520c58462ec7b9c6ad`. The exact reviewed candidate was
  `8ed45b3b14122806d3d992de4c3522e8427264b8`.
- The prerequisite implements package-neutral enum and union construction and
  projection. It also implements deterministic union ownership and demand gates.
- The final whole-candidate agent review returned `SATISFIED` with no blockers.
  Its response SHA-256 is
  `c8bf3a4b223e509e3474a2f2e8b4b1b6268fa43600b07ceed0a6d5837530a4ed`.
- The canonical warm create-PR gate passed. The single merge gate passed every
  phase-owned lane, all performance cases, and all structural bridge cases.
- The merge continuation passed the remaining validation, runtime-platform,
  E2E, hardening, and extra-E2E steps. Hardening covered 268 variants with no
  errors.
- One current-main crate error reproduced on the clean base. Issue
  [#3179](https://github.com/sifr-lang/sifr/issues/3179) owns that nominal-path
  source collision.
- The prerequisite has no compatibility path, fallback, legacy path, or
  versioned public API name.
- The project-root structural identity repair merged in Sifr
  [PR #3182](https://github.com/sifr-lang/sifr/pull/3182) at merge commit
  `4f5492531e81385dd28efe25adfdd57dd678d2a9`. The exact reviewed candidate was
  `55210678160b1e43f7aab9245fc12bc9c6698f7d`.
- Project and test-project records now use their module-qualified nominal
  identity. Unnamed single-file records remain unqualified.
- Three agent review rounds corrected fixture identities, shape hashes, and
  stale assertions. The final review returned `SATISFIED` with no blockers.
- The final review response SHA-256 is
  `b6d7c534497f8ebf122a2a18287e56f8a34b51c8b4a4014df96d14c3fd5737ea`.
- The unchanged warm create-PR gate exited zero. It passed Python 19/19,
  Rust interop 10/10, generated quality 5/5, and E2E 140/140.
- The single merge gate passed every structural bridge case. It also passed
  Python 25/25, performance 8/8, distribution 66/66, sysroot 2/2, and
  generated quality 7/7.
- The gate passed 76 of 77 generated driver builds. The sole failure reproduced
  unchanged on exact base `6ac919f809bb966493c769a1c5ffb0e41420636b`.
- Issue [#3179](https://github.com/sifr-lang/sifr/issues/3179) continues to own
  that duplicate compiler-owned native-handle path defect.
- The repair adds no compatibility path, fallback, legacy path, or versioned
  public API name.
- The deterministic nominal-union prerequisite merged in Sifr
  [PR #3189](https://github.com/sifr-lang/sifr/pull/3189) at merge commit
  `f8857c4692903cffc5b150831263d31fb4822d5e`. The exact reviewed and gated
  candidate was `d62f9658319ee65481398afd9ac19bb6aa41020e`.
- Union ordering and deduplication now use stable nominal, structural, protocol,
  newtype, and enum identities. Same-bare-name members stay deterministic across
  modules and generic substitution. Raw generic and structural wrappers preserve
  nested optionality until a typed optional boundary flattens it.
- Project and test-project generation now derive crate-root ownership from the
  items actually hoisted into the generated crate root. An unrelated union can no
  longer strip a module-local compiler-owned nominal such as the native file
  handle.
- Focused validation passed 135 type-system tests, 1,023 codegen tests, strict
  affected-library Clippy, formatting, HIR maintainability, the file-size guard,
  and generated-build regressions for nominal collisions, module-local native
  handles, and `main`/root test layouts.
- The canonical create-PR gate exited zero. Its receipt SHA-256 is
  `0827cb4448778c7fbfb1c7fcae8ef1ff6911d1b94afd75e552c0229f85d2d916`.
  The single authoritative merge gate also exited zero. It passed all 78 generated
  driver builds, 1,023 codegen tests, 695 E2E fixtures, and 268 hardening variants
  with zero failures. Its receipt SHA-256 is
  `f4cd5b1d22494dd613f21eada04b2a30a30025c24947b89befe456b6fcfec1e8`.
- The final exact-SHA agent review returned `SATISFIED` with no blocking findings.
  Its response SHA-256 is
  `1edf024c54bb6cfd3812c76567c382aa4410a1875a7ee3d0dd291422685e12f5`.
- [Issue #3162](https://github.com/sifr-lang/sifr/issues/3162) continues to own
  non-blocking union-rendering, ownership-list consolidation, and test-layout
  hardening. PR #3189 did not absorb those follow-ups.
- The prerequisite adds no compatibility path, fallback, legacy path, or
  versioned public API name.
- The first `milestone_ps_7` companion wave merged in
  [PR #5](https://github.com/sifr-lang/pydantic-sifr/pull/5) at merge commit
  `d63bb8ea0a00b6229d8c4e4defab5045c2e2b24f`. The exact reviewed and gated
  candidate was `368553f93cbe2316686bfb600d54978a12e44bfd`.
- The wave implements literals, payload-free enums, nullable values, smart and
  left-to-right ordinary unions, and field/path tagged unions. It also adds
  labelled aggregate errors, error overrides, explicit auto-collapse, and one
  public static-program demo.
- Static smart-union ranking now checks model fields and mapping key/value
  schemas recursively. Focused tests cover coerced mappings, coerced models,
  and static left-to-right selection.
- Canonical union order now separates qualified structural identity from the
  compiler's bare nominal sort key. The guard covers class, newtype, and enum
  secondary keys. Cross-module model and enum tests assert the complete union
  identity.
- The provenance audit covers 310 files and 12,754 nodes. Its manifest is
  byte-identical on Python 3.12.5 and 3.13.1, with SHA-256
  `4dfdfed840829f0fd439b42ebba859f22c9c491f8f7e62595fe2fc4f19fedf0e`.
- The local create-PR gate and the cold GitHub create-PR gate passed on the
  exact candidate. Both demos, the static round trip, the Python-free graph,
  all workspace tests, the benchmark smoke test, and strict Clippy passed.
- The single merge gate exited zero. It added release-mode suites and 1,000
  bounded runs for each of four fuzz targets.
- Four remediation reviews corrected recursive static exactness and nominal
  union ordering. The final agent review returned `SATISFIED` with no blockers.
  Its response SHA-256 is
  `f9ab19720cdd215d6de5b78e5fbf4c74b51026a3b68dd753bebf358ccafae7b3`.
- Later PS7 waves own nested sum metadata, typed callback discriminators,
  definitions, recursion, control composition, validation context, and
  same-bare-name nominal collisions.
- The wave adds no compatibility path, fallback, legacy path, or versioned
  public API name.
- The second `milestone_ps_7` companion wave merged in
  [PR #6](https://github.com/sifr-lang/pydantic-sifr/pull/6) at merge commit
  `75788bab0e6efbc72533be260883dfa37cf64135`. The exact reviewed and gated
  candidate was `307eaf940dafcc2f6c01d3812c02d6e7eb18e6d7`.
- The wave executes owned and compiler-emitted definitions and references. One
  exact definition scope supports repeated scalar and model references.
- Recursive model validation now keeps stable nested error paths and enforces
  the configured depth limit. The public PS7 demo constructs a recursive model
  from a compiler-emitted static program.
- Definition construction rejects duplicate names, dangling references, and
  structural identity or canonical-kind mismatches before input validation.
- The canonical create-PR gate and the single merge gate passed. Evidence
  includes 103 workspace tests, release-mode suites, strict Clippy, both locked
  demos, the benchmark smoke test, and four fuzz targets with 1,000 runs each.
- The exact-SHA agent review returned `SATISFIED` with no blockers. Its response
  SHA-256 is
  `bf42f675e395d2722020142f950360db22758bdfd5169409e88f67f32ce266f3`.
- A later PS7 hardening wave owns scope propagation through embedded JSON,
  defaults, mapping keys, and lazy generators. It also owns reference-aware
  smart ranking, dead-definition checks, recursive sum ordering, and an active
  recursion-loop test.
- The wave adds no compatibility path, fallback, legacy path, or versioned
  public API name.
- The PS7 definition hardening wave merged in companion-repository
  [PR #7](https://github.com/sifr-lang/pydantic-sifr/pull/7) at merge commit
  `9777da116971acfea592bba566d9d873f5f20bca`. The exact reviewed and gated
  candidate was `78bc611f1f58218d11a2239e2b3eb70d1355b4e8`.
- Fresh parsed inputs, defaults, JSON object keys, and lazy generator items now
  keep their definition scope. Each fresh input starts a new arena-local
  recursion trace and keeps the shared depth budget.
- Owned and compiler-emitted smart unions resolve definition references during
  exactness ranking. Definition-wrapped sums use the same flattened canonical
  layout during selection and construction.
- Scope construction checks reachable and unreachable definitions. References
  cannot target flattened wrappers or nested definition scopes. Owned and
  compiler-emitted schemas enforce the same rule.
- Focused validation passed 20 definition tests, static reference rejection,
  nested depth accounting, strict Clippy, and the file-size guard. The refreshed
  canonical create-PR gate passed all package checks and both locked demos.
- The single authoritative merge gate exited zero. It passed all workspace and
  release foundation tests, six fuzz-target builds, four fuzz targets with
  1,000 runs each, the benchmark smoke test, and all provenance checks.
- The first exact-SHA agent review found one flattened `EmbeddedJson` layout
  blocker. Its response SHA-256 is
  `b3a91367fa7038a208d4cf8a844cb1cd6b2e4472255cce4b09168e7e5ef67608`.
- The remediation also closed all related static-parity, depth, recursion,
  scope-sharing, traversal, test, and documentation findings. The second
  exact-SHA review returned `SATISFIED` with no blockers. Its response SHA-256
  is `07c9ef47bfa2aafc867b5b4e13387bc6657df245914cf4a59aab45ac80e4189e`.
- Later PS7 work can borrow definition targets during ranking, share each scope
  map, resolve generator roots through references, and make extra-schema
  verification explicit. It can also simplify the internal validation entry
  point and expose owned reference targets through `SchemaRef` if required.
- The hardening wave adds no compatibility path, fallback, legacy path, or
  versioned public API name.
- The PS7 control-composition wave merged in companion-repository
  [PR #8](https://github.com/sifr-lang/pydantic-sifr/pull/8) at merge commit
  `c1a2f35dc48b9ba914d6c4a2d9912f91e0c51a11`. The exact reviewed and gated
  candidate was `8a8bde393d48eab8ebecdcd0be078502e34ad6bb`.
- The wave implements strict/lax and JSON/structural branch controls. It also
  implements flattened, nonempty typed chains with direct checked-arena
  handoff between steps.
- Control branches must have one structural output identity. Chain handoff
  keeps the input profile, definition scope, specialized scalar values, and
  aggregate values.
- Strict JSON mapping chains rebuild typed keys as JSON object keys. They do
  not expose the native mapping representation to the next JSON step.
- An earlier candidate passed the create-PR gate. Its merge-only fuzz build
  found one incomplete `ValidationOptions` initializer.
- The correction completed that initializer and preserved strict item checks
  when lazy generators relax only their container kind.
- A full agent review then found the strict JSON mapping handoff defect. Its
  response SHA-256 is
  `93ea5cd5d77868d39c65fe05a4b97c931f1b06bd18cf31856f641a38ab7f2cbc`.
- The exact remediation review returned `SATISFIED` with no blocking findings.
  Its response SHA-256 is
  `4e04d32eb0d027b6e285b4132e8fc51f2ab8308308e8608aa3a5694d57661a66`.
- The refreshed canonical create-PR gate passed on the final candidate. The
  single final merge gate also exited zero.
- Evidence includes 8 control tests, 15 collection tests, all workspace tests,
  release suites, strict Clippy, both locked demos, and the benchmark smoke.
- All six fuzz targets compiled. Scalar, collection, special, and typed
  construction targets each completed 1,000 bounded runs.
- The static-program round trip stayed byte-identical at 1,813 bytes. Its
  identity is
  `0a80954bcb655b487d33525efa0dfcd98fce189c12c863b21533e11e7970e4c9`.
- Reviewer follow-ups remain non-blocking. They cover pattern-key flags,
  non-finite float keys, stronger key assertions, and JSON key strictness
  documentation.
- The control wave adds no compatibility path, fallback, legacy path, or
  versioned public API name.
- The PS7 cumulative-ledger wave merged in companion-repository
  [PR #9](https://github.com/sifr-lang/pydantic-sifr/pull/9) at merge commit
  `bc4617c9535d49c25dacc0d345f69d23156ff686`. The exact reviewed and gated
  candidate was `8f04103ce82c14feb72e88cc3511af2d80b553f4`.
- The PS7 ledger now cumulatively certifies nine delivered families: literals,
  enums, ordinary and nullable unions, field/path tagged unions, definitions and
  recursion, control composition, recursion limits, and smart-union ranking.
  It binds every upstream-derived row to the pinned PS7 anchor set while keeping
  the two Sifr-native core families on direct local evidence.
- The exact-SHA agent review found one documentation omission in the pending API
  boundary. The remediation names validator, discriminated-union, and
  generic-recursion APIs as pending. The final review returned `SATISFIED` with
  no blockers. Its response SHA-256 is
  `6673912d688cf74ea97b82d1d8d1bdccdba6fa439678788fa26f7284c4e34cbc`.
- Targeted validation passed the PS7 ledger unit checks, 8 control tests, 20
  definitions/recursion tests, 17 sum tests, formatting, and the file-size
  guard. The canonical create-PR and single merge gates passed against the exact
  pinned Sifr source toolchain. The merge gate included both locked demos, all
  workspace and release suites, six fuzz-target builds, and four 1,000-run fuzz
  campaigns.
- A separate exact-current-state agent audit confirmed that the fixed-arity
  call-scoped callback bridge cannot carry the remaining heterogeneous validator
  slots or mutable caller context without prohibited erasure. The next
  package-neutral prerequisite is a static-program-indexed call-scoped typed
  callback slot table plus an opaque typed context handle. The audit response
  SHA-256 is
  `e3ac33233adb7e77ff27e63ed84ecb49ea716afb56ef1bda2ab97aecce0fb5d8`.
- Non-blocking ledger follow-ups can make the nullable-union evidence pointer
  more specific and replace the implicit `core/` anchor opt-out with an explicit
  row property. PR #9 did not absorb those suggestions.
- The wave adds no compatibility path, fallback, legacy path, Python production
  dependency, or versioned public API name.
- The PS7 annotated-method-shape prerequisite merged in Sifr
  [PR #3193](https://github.com/sifr-lang/sifr/pull/3193) at merge commit
  `ac8dfade387f8d9b5a6dcd390667c68686485620`. The exact reviewed and gated
  candidate was `d42dd6a09a46448bf71d4506fbae46e45b50be6b`.
- Static-program nominal shapes now carry deterministic descriptors for directly
  annotated local methods. Descriptors preserve source order, source spelling,
  method kind, receiver, async state, parameter conventions, keyword-only state,
  declared types, result type, and method or parameter metadata.
- Method contracts come from the specialized HIR declaration. Generic class
  arguments are substituted once from the concrete nominal owner, operators are
  included, and the lowered `new` constructor is serialized as source
  `__init__`. Unannotated helpers remain absent from the identity.
- This prerequisite changes const-specialization program identity only. It does
  not change runtime structural or wire identity, emit a callback runtime, or
  add a compiler special case for Pydantic.
- Two exact-SHA agent reviews found generic-substitution/operator coverage and
  constructor/async canonicalization blockers. The remediations closed them.
  The final exact-SHA review returned `SATISFIED` with no blockers; its response
  SHA-256 is
  `609fd6b07e81d6f3e481d433cee36538011305051231f7aa1b7afc4554c67cef`.
- Targeted compiler validation passed 71 frontend tests and 987 lowering tests
  with one pre-existing ignored test, plus formatting, maintainability, diff,
  and file-size guards. The first cold create-PR attempt passed every functional
  lane and exceeded only the tracked generated-artifact cold budget in #3134.
  The unchanged warm canonical create-PR gate then exited zero.
- The single authoritative merge gate exited zero. It passed Python interop
  25/25 with zero read-only mutations, Rust interop 10/10, E2E 695/695,
  hardening 268 variants with zero failures, all 78 ignored driver generated
  builds, and all compiler, release, performance, distribution, sysroot, and
  generated-code-quality lanes. The merge report SHA-256 is
  `b2ff478e156276bcddb350fff2f7efec35ef94042d226e36ec40f5bd4aeed0e8`.
- The prerequisite adds no compatibility path, fallback, legacy path, or
  versioned public API name. The next PS7 wave can build the static-program-
  indexed typed callback slot table and opaque typed context handle on this
  method-shape substrate.
- The PS7 imported-method-shape prerequisite merged in Sifr
  [PR #3196](https://github.com/sifr-lang/sifr/pull/3196) at merge commit
  `8901eeed5eafda9cd18ce8f9b4faab56b25125c6`. The exact reviewed and gated
  candidate was `28444669949a14bf8397ed4495c3e109e414bc26`.
- Imported and re-exported nominal shapes now preserve annotated method
  descriptors, generic substitutions, defaults, metadata, private nested
  identities, and direct enum and newtype source identities. Identity-qualified
  lookup prevents a colliding local class name from replacing imported shape
  metadata.
- Module export replacement and removal clear stale class defaults, declaration
  metadata, class type parameters, and structural method descriptors. Structural
  method storage and collection are demand-gated, so ordinary programs do not
  pay the imported-shape scan or allocation cost.
- The final exact-SHA agent review returned `SATISFIED` with no blocking findings.
  Its response SHA-256 is
  `bfbf35dffdfc04b559dccbb74ebb8a838b894f9259aa171e1917542e206ce6d1`.
- Targeted validation passed 77 frontend tests and 987 lowering tests with one
  ignored test, strict affected-library Clippy, formatting, HIR maintainability,
  and the file-size guard. Three governed representative performance runs passed
  all cases and budgets. Their evidence SHA-256 values are
  `ac99562e696d03a0e9c2b41c251b0073e7ed3b02581d6d172017720a972514b4`,
  `cc61b1913295db76baf8d8ac7e49f4545428194991d584bce87202a1e41c000e`,
  and `17a2204177edf19a949f0d8d786083760c68802c9ab1987634ddea2871c87802`.
- The canonical create-PR gate exited zero. It passed E2E 140/140, Python
  interop 19/19 with zero mutations, Rust interop 10/10, generated quality 5/5,
  and all guards. Its receipt SHA-256 is
  `767abbaec3e97ec42faeec44d5bcd4338fbb6531086386acc6b5d66119770361`.
- The single authoritative merge gate exited zero. It passed E2E 695/695,
  hardening 268 variants with zero failures, all 78 generated driver builds,
  and all compiler, release, performance, distribution, sysroot, and
  generated-code-quality lanes. Its receipt SHA-256 is
  `b3f2a072d8b7da43bfbcd921fe5f8727f8c8664bb10b96dbb6134280ed8030b6`.
- [Issue #3197](https://github.com/sifr-lang/sifr/issues/3197) owns the
  non-blocking method-export API, invalidation, sysroot-import, metadata-policy,
  maintainability, and stale-consumption follow-ups. PR #3196 did not absorb
  them.
- The prerequisite adds no compatibility path, fallback, legacy path, or
  versioned public API name.
- The PS7 static-program method-slot prerequisite merged in Sifr
  [PR #3199](https://github.com/sifr-lang/sifr/pull/3199) at merge commit
  `f3d28b8d16872baa396aa36e6b3f99a8e4735427`. The exact reviewed candidate
  was `6f12dd6eef1d756a9c7ce89559ed62ead7daa02b`.
- The compiler now resolves an ordered, identity-qualified slot list from one
  reserved specialization field. It emits monomorphic method dispatch for
  mutable, shared, and no-context calls. The runtime carries a call-scoped,
  current-thread-only handler and a typed context borrow. Slot identity and
  cache identity include the program, ordered signatures, context identity,
  and borrow mode.
- Six active `SIFR-RUST-SLOT-*` diagnostics reject malformed lists, missing
  methods, invalid signatures, bound or context conflicts, and handler lifetime
  or thread escape. Generated positive and negative package evidence covers
  dispatch, shared and mutable context, handler escape, and static-program
  envelope verification.
- The exact-SHA agent review returned `SATISFIED` with no blocking findings. Its
  response SHA-256 is
  `dbf474cecc17cad6270447dda290dea1fa6abc2a93b965ace72210bef4a10811`.
- Focused validation passed identity, runtime, frontend, lowering, codegen, and
  driver probe suites. The generated package passed check, build, and execution.
  Strict affected-library Clippy, formatting, diagnostics, Rust-interop
  inventories, taxonomy, HIR maintainability, and the file-size guard passed.
- The cold create-PR gate was functionally green and exceeded only the known
  generated-artifact cold budget tracked by issue #3134. The unchanged warm
  create-PR gate exited zero and passed E2E 140/140. Its receipt SHA-256 is
  `5aee199294e9b79ba52c34d560bb1e58e7687f2e2776608695a1a3cc31b4d3e5`.
- The single merge gate passed all functional lanes it executed, including
  Python interop 25/25, Rust interop 10/10, developer tooling 32/32, and all ten
  representative benchmark commands. One parse-only control reported a stable
  cross-run retired-instruction offset and rejected its budget. Two independent
  controlled repeats of the unchanged candidate passed at 920,717,367 and
  920,747,156 median instructions against the 936,811,698 limit. Their accepted
  receipt SHA-256 values are
  `053b764abb45be6f1d4a2a175cfe8d6e4874bf53dd1afdae78b6751bc160c94e`
  and `76ac60564b84c7c8c506a3173dcb398a4e94676501692b4a1872ee218104dcb2`.
- [Issue #3200](https://github.com/sifr-lang/sifr/issues/3200) owns the
  cross-run work-counter offset. [Issue #3201](https://github.com/sifr-lang/sifr/issues/3201)
  owns the reviewer suggestions for handler-bearing emission, structured slot
  errors, explicit header invariants, deterministic singleton ownership, and
  fixture evidence fidelity. PR #3199 did not absorb those follow-ups.
- The prerequisite adds no compatibility path, fallback, legacy path,
  Pydantic-specific compiler behavior, or versioned public API name.
- The typed-record field legality addendum merged in Sifr
  [PR #3203](https://github.com/sifr-lang/sifr/pull/3203) at merge commit
  `335390b8171fb0a5cda275c22d200a420380c777`. The exact reviewed candidate
  was `4eb576a46df7e9b9198abc26280152ff9b74ef78`.
- The reserved dotted specialization key was replaced atomically by the single
  legal compiler-owned field `sifr_method_slots`. The typed fixture now proves
  that a structural specialization result can declare and return the field.
  No alias, compatibility spelling, fallback, or legacy key remains.
- The exact-SHA agent review returned `SATISFIED` with no blocking findings. Its
  response SHA-256 is
  `440da0c5a279dd580d505d5879fc2c3303a6a3f3921ca1d4acc716ffe5299572`.
- Focused frontend and generated-driver tests, strict frontend Clippy, Rust
  interop inventories, taxonomy, formatting, HIR maintainability, and the
  file-size guard passed. The canonical create-PR gate passed every lane step;
  its receipt SHA-256 is
  `4fee2be48725607c8b62225bbec5ddaefdc27ba60ce724438d33ea8cd0680e1c`.
- The one merge gate passed every preceding functional lane and stopped only on
  the cross-run work-counter offset already tracked by issue #3200. The exact
  candidate then passed the unchanged controlled 20-sample case at 924,385,045
  median instructions with 0.000844 variation against the 936,811,698 limit.
  The accepted evidence SHA-256 is
  `d9467bdfaa644ee923d4ffc42eeb4faaa8ee68a252ecb0e0fc7550700ba9c6a0`.
- The empty method-slot list addendum merged in Sifr
  [PR #3205](https://github.com/sifr-lang/sifr/pull/3205) at merge commit
  `65674c1e182aaf6fafae132580641cd04e727d42`. The exact reviewed candidate was
  `514c018c09176fc9456e6f4b34530de2669497c2`.
- A typed static-program payload can now declare `sifr_method_slots` and return
  an empty list when it has no callbacks. Empty lists emit no method-slot table;
  malformed non-list values still produce `SIFR-RUST-SLOT-0001`. No alias,
  fallback, compatibility form, or Pydantic-specific compiler path was added.
- The remediation exact-SHA agent review returned `SATISFIED` with no blocking
  findings. Its response SHA-256 is
  `76bda413860ef97ff631a3992dc469f8122c44c190845f672f06c928a93abe66`.
- Focused frontend, driver, Rust-interop, diagnostics, strict Clippy,
  formatting, HIR-maintainability, taxonomy, and file-size checks passed. The
  create-PR receipt SHA-256 is
  `778db57497f4e1df41fd3df526748413ac414d60bb091810fe23f1444612204d`.
- The single merge gate passed every functional lane and all ten representative
  benchmark commands. Its only blocking verdict was the known #3200
  work-counter offset: 938,532,133 measured instructions against the
  936,811,698 threshold. The receipt SHA-256 is
  `845d7cb614b3e347078175c842572df2201ec98eb0f84ce083e3a205db724e9b`.
- `milestone_ps_7` is now active in the companion repository.
- The PS7 typed-callback and wrap rows that require new compiler capability are
  tracked by companion [issue #10](https://github.com/sifr-lang/pydantic-sifr/issues/10).
  The callback-aware specialization works monolithically, but imported const
  schema modules do not complete within bounded memory, and the current method
  dispatch does not emit the call-scoped handler needed for wrap semantics.
  These rows were skipped without adding a compiler-specific package path,
  fallback, compatibility layer, or legacy API.
- The first PS8 serializer-plan item merged in companion
  [PR #11](https://github.com/sifr-lang/pydantic-sifr/pull/11) at merge commit
  `5ea9bb4a1745001b435e6ce7d1e61daf3085611b`. The exact reviewed candidate
  was `6331b14a18b0f79b4d3282636f93e9f16eab0b21`.
- The core now compiles deterministic, depth-bounded serializer topology from
  the verified owned or static Core Schema. Plans retain the structural shape
  identity, model projection order, and control and collection children without
  retaining a validation arena or introducing another schema compiler.
- Exact-SHA agent review returned `SATISFIED` with no blocking findings. Its
  response SHA-256 is
  `e1b8c264806f5a18e495a725da24f48402c1bd515986620b16cc9c7ccad410d8`.
- Focused plan tests passed 2/2 with strict Clippy and the file-size guard. The
  exact-pin companion create-PR and single merge gates passed provenance,
  package checks, both locked demos, all workspace and release suites,
  benchmark smoke, strict Clippy, fuzz smoke, and merge-only foundations.
- The PS8 structural and streaming output item merged in companion
  [PR #12](https://github.com/sifr-lang/pydantic-sifr/pull/12) at merge commit
  `d4b039e70aa760613abbd038bc9e13d41ea41805`. The exact reviewed and gated
  candidate was `5cb74bf3d342876a524a3378524a4d21d540d09d`.
- Serialization now verifies the prepared structural shape before projecting
  current typed values. Structural output returns the crate-neutral value, and
  JSON output writes structural visitor events directly to bounded bytes with
  typed projection, limit, shape, and unsupported-policy errors. It does not
  retain a validation arena or build a generic JSON tree. Bytes, temporal
  representations, and integer-profile decisions remain explicit later PS8
  policy work; no fallback, compatibility, or legacy encoding was added.
- Exact-SHA agent review returned `SATISFIED` with no blocking findings. Its
  response SHA-256 is
  `4b6c131fc4ebc865b237b97922be2e8961fd8b1bf55c0d932eeaa57a2f174df6`.
- Focused output and plan tests passed 5/5 with strict Clippy, formatting, and
  the file-size guard. The exact-pin companion create-PR and single merge gates
  passed provenance, package checks, both locked demos, all workspace and
  release suites, benchmark smoke, strict Clippy, fuzz smoke, and merge-only
  foundations.
- The PS8 typed serialization-policy item merged in companion
  [PR #13](https://github.com/sifr-lang/pydantic-sifr/pull/13) at merge commit
  `59d62ef4d0b2c559d08019a08ec00812c0d23a90`. The exact reviewed and gated
  candidate was `4cb0a3c12f7f412d2a32994a293447236171a59b`.
- Plans now own field aliases and materialized defaults. Call options use typed
  field/index paths for recursive include and exclude selections, with exclude
  precedence, alias output, and none/default omission shared by structural and
  direct JSON serialization. Default comparison uses only a bounded raw byte
  capture for the active default-bearing field; JSON still does not construct
  a generic value tree. No fallback, compatibility API, or legacy signature
  remains.
- Exact-SHA agent review returned `SATISFIED` with no blocking findings. Its
  response SHA-256 is
  `9b5d1a30b86b6ac3427d27d9e68c740e15a160562354d5d8f4558a5abdbd66ee`.
- The full core suite, focused output and plan tests 7/7, strict Clippy,
  formatting, and file-size guard passed. The exact-pin companion create-PR
  and single merge gates passed provenance, package checks, both locked demos,
  all workspace and release suites, benchmark smoke, strict Clippy, fuzz
  smoke, and merge-only foundations.
- The PS8 custom serializer and computed-field row is blocked on the same
  package-neutral handler-bearing method-slot dispatch as PS7 callbacks.
  Companion [issue #14](https://github.com/sifr-lang/pydantic-sifr/issues/14)
  records the exact dependency and links the existing PS7 issue #10. The row
  was skipped without adding dynamic dispatch, a package-specific compiler
  path, fallback, compatibility behavior, or a legacy callback API.
- Caller-owned typed serialization-context forwarding is blocked by the same
  issue #14 because no serializer handler boundary exists to receive it. This
  row was also skipped; the package does not expose an unused context argument
  or an untyped context container as a placeholder.
- The PS8 integer JSON-profile item merged in companion
  [PR #16](https://github.com/sifr-lang/pydantic-sifr/pull/16) at merge commit
  `70cb8a42cd2229cc909e58ebe4e79c66fdf5f69d`. The exact reviewed and gated
  candidate was `a8466ed61490ed10446a055c782d4e13336a5aa4`.
- Each serializer plan now requires one explicit `json.exact`, `json.web`, or
  `json.string_ints` profile. Fixed-width integers, exact integers, and bounded
  default comparison all route through `sifr_runtime::json`. A `json.web`
  violation preserves the typed `JsonIntegerRangeError`, selected profile, and
  recursive model path. No call-time default, fallback, compatibility shim, or
  alternate integer encoder was added.
- Exact-SHA agent review returned `SATISFIED` with no blocking findings. Its
  response SHA-256 is
  `903792e47cfe664139aabb841e3cba29611b4369210d603d211e68d78100575c`.
- Focused serialization tests passed 8/8; the full workspace suite, strict
  Clippy, formatting, and file-size guard passed. The exact-pin companion
  create-PR and single merge gates passed provenance, package checks, both
  locked demos, all workspace and release suites, benchmark smoke, fuzz smoke,
  and merge-only foundations.
- Temporal output policy remains blocked until the package exposes current-value
  typed temporal projections at the serialization boundary. Companion
  [issue #15](https://github.com/sifr-lang/pydantic-sifr/issues/15) records the
  later item. The row was skipped without tuple-layout inference, retained
  validation state, fallback formatting, or compiler-specific assumptions.
- The delivered PS8 serialization corpus and benchmark bound merged in
  companion [PR #18](https://github.com/sifr-lang/pydantic-sifr/pull/18) at
  merge commit `4e47b23d1f1f87e76eb87e892bbc7570373adf04`. The exact final reviewed and
  gated candidate was `b55032ba352dd7012ae5f1734d2fa26aefb2805a`.
- Direct corpus tests cover JSON-native scalar and nullable values, sequences,
  tuples, sets, string-key mappings, structural/JSON projection agreement, and
  the prior model, alias, selection, default, and integer-profile cases. A
  guarded PS8 compatibility ledger records only the delivered family bound.
  Criterion smoke now measures both streaming JSON and structural output with
  plan and value setup outside the timed closure.
- The first exact-SHA review found one false canonical set-order claim. The
  bounded remediation records the actual structural projection-order contract;
  the one allowed remediation review returned `SATISFIED`. Its response
  SHA-256 is
  `79343450c190f54b4d8135f6d34be88998bbdc4597f75741e86bd43f84a2ca2f`.
- The full workspace and compatibility suites, three serializer-corpus tests,
  eleven existing output tests, all three benchmark smokes, strict Clippy,
  formatting, and file-size guard passed. The exact-pin companion create-PR
  and single merge gates passed all package, release, provenance, demo, fuzz,
  and merge-only checks.
- Remaining anchored serializer families are tracked by companion
  [issue #17](https://github.com/sifr-lang/pydantic-sifr/issues/17) until their
  PS9/PS10 typed public value surfaces exist; callback and temporal dependencies
  remain in issues #14 and #15. No placeholder serializer or generic-tree
  fallback was added.
- `milestone_ps_8` is now active in the companion repository.
- The first PS9 adapter item merged in companion
  [PR #19](https://github.com/sifr-lang/pydantic-sifr/pull/19) at merge commit
  `693fbfa11dd1e52647092c204369521d32a33c82`. The exact reviewed and gated
  candidate was `62fd16f08d64362f1845184da1518b2ac27eac97`.
- `TypeAdapter[T]` now prepares one Core Schema and one serialization plan,
  rejects a target shape mismatch at construction, and reuses that state for
  native, JSON, strings-profile, structural-output, and streaming-JSON calls.
  Its integer JSON profile is selected statically. No call-time default,
  fallback, compatibility layer, or legacy adapter path was added.
- Exact-SHA agent review returned `SATISFIED` with no blocking findings. Its
  response SHA-256 is
  `4fa9315c5ea23996dca5e2ecaf5d07114d3421b8c50fae87dcdf4ab78d39ee9f`.
- Focused adapter tests passed 3/3; the full workspace suite, strict Clippy,
  formatting, and file-size guard passed. The exact-pin companion create-PR
  and single merge gates passed all package, release, provenance, demo, fuzz,
  benchmark-smoke, and merge-only checks.
- The PS9 JSON Schema foundation merged in companion
  [PR #21](https://github.com/sifr-lang/pydantic-sifr/pull/21) at merge commit
  `7822214c301c03900d990eaa733bbdc27c9c16d8`. The exact final reviewed and
  gated candidate was `d8e997a53930749feb93987270d759fe6eabee12`.
- `TypeAdapter[T]` now generates validation- and serialization-mode JSON Schema
  by traversing its owned Core Schema. Ordinary scalar, literal, enum, sum,
  model, collection, temporal, control, and embedded-JSON nodes share that one
  authority. Unsupported recursive, specialized numeric, byte, and non-exact
  integer representations fail closed with a typed error.
- The first exact-SHA review found fixed integer bounds could be widened and
  byte length/encoding claims did not match the engine. The bounded remediation
  intersects target and declared bounds, rejects invalid `multipleOf`, and
  fails closed for byte and decimal representations. The one permitted
  remediation review returned `SATISFIED`; its response SHA-256 is
  `aa593676eb0942432172b79be65807b1fd33dee8c09d742d5d0b4e10fbe44b9a`.
- Six focused JSON Schema tests, the full core suite, strict Clippy,
  formatting, and file-size guard passed. The exact-pin companion create-PR
  and single merge gates passed all package, release, provenance, demo, fuzz,
  benchmark-smoke, and merge-only checks.
- The remediation review's later alias-key mechanism defect is recorded in
  companion [issue #20](https://github.com/sifr-lang/pydantic-sifr/issues/20)
  for the explicit PS9 aliases and mode-representation row.
- The PS9 integer-profile JSON Schema item merged in companion
  [PR #22](https://github.com/sifr-lang/pydantic-sifr/pull/22) at merge commit
  `d7620bcaef75580d548f00dfd7bcafb523efbf2a`. The exact final reviewed and
  gated candidate was `62b22ea7ea530d6fabde33f6d09afb351f3071ea`.
- Exact, web, and string-integer schemas now preserve their selected profile
  and complete static range. Web schemas require a wholly JavaScript-safe
  range and otherwise expose compiler-owned `SIFR-INT-0009` provenance.
  Integer literals and enum inputs use the same profile path as ordinary
  integer nodes. No profile fallback or implicit string encoding was added.
- The first exact-SHA review found integer literals and enum inputs bypassed
  the profile path. The bounded remediation made those representations
  profile-aware, and the one permitted remediation review returned
  `SATISFIED`; its response SHA-256 is
  `ca9ae1d6b69920eb98b44b41b73e3f28a200f0e5b6ea80d1097e0f68e048ae93`.
- Thirteen focused JSON Schema tests, the full core suite, strict Clippy,
  formatting, and file-size guard passed. The exact-pin companion create-PR
  and single merge gates passed all package, release, provenance, demo, fuzz,
  benchmark-smoke, and merge-only checks.
- The coordinated Sifr boundary artifact now names the implemented consumer,
  generated-client warning ownership, fail-closed web behavior, exact-profile
  marker, and four exact bounded JSON Schema snapshots.
- The PS9 definitions, recursion, aliases, constraints, and mode-specific
  representations item merged in companion
  [PR #23](https://github.com/sifr-lang/pydantic-sifr/pull/23) at merge commit
  `ee3b8d3a3a0a03f039b4812a18baa18186b4f7f1`. The exact reviewed and gated
  candidate was `9582dd47f6d3e2a5b04a0d4135ae8a3e09b046e2`.
- JSON Schema now emits deterministic root `$defs` and recursive `$ref`
  pointers, derives validation and serialization property names from explicit
  alias options, carries string mapping-key constraints through
  `propertyNames`, and fails closed for unsupported alias shapes, key types,
  or mode-specific representations. The legacy mode-only API was removed.
- Exact-SHA agent review returned `SATISFIED` with no blocking findings. Its
  response SHA-256 is
  `7d3f60c3becfae6370a274466c2913d2f82eefd78af818335dc7f9fed668ffd2`.
  Eighteen focused JSON Schema tests, the full core suite, strict Clippy,
  formatting, file-size guard, exact-pin companion create-PR gate, and the
  single authoritative merge gate passed.
- The PS9 public `Fraction` and `Complex` adapter/schema item merged in
  companion [PR #25](https://github.com/sifr-lang/pydantic-sifr/pull/25) at
  merge commit `c0ee5c77efdeccbd6380ef6abccf7e6612b1c533`. The exact final reviewed
  and gated candidate was `35ebd03a945133d2e998bdfd08e0162939244b94`.
- The companion now exports immutable specialized numeric values that reuse
  the prepared validation engine and serialize as canonical fraction and
  complex strings. Their mode-specific JSON Schemas retain exact rational and
  complex constraints as Sifr annotations and emit standard numeric keywords
  only when the exact value round-trips without rounding.
- The first exact-SHA review found rounded rational constraints could publish
  false numeric bounds. The bounded remediation made standard keywords
  round-trip exact, described actual Complex validation inputs, and pinned the
  strict-native versus serialized-string boundary. The one permitted
  remediation review returned `SATISFIED` with no blockers. Its new
  negative-sign NaN formatting mechanism finding is tracked separately by
  companion [issue #24](https://github.com/sifr-lang/pydantic-sifr/issues/24).
- The full core suite, strict all-target Clippy, formatting, file-size guard,
  exact-pin companion create-PR gate, and the single authoritative merge gate
  passed.
- The PS9 deterministic schema snapshots and dialect-conformance item merged
  in companion [PR #26](https://github.com/sifr-lang/pydantic-sifr/pull/26)
  at merge commit `f3ad56b9eb1d250e36c8e5f2929e31485c5dac75`. The exact reviewed and
  gated candidate was `337338b7506f9e514945413746fce4cf21cca391`.
- Every generated JSON Schema document now declares the Draft 2020-12 dialect
  at its root. Committed snapshots cover recursive definitions, aliased
  serialization, and specialized numeric validation. The independent pinned
  `jsonschema` test dependency meta-validates and compiles all three documents,
  and repeated generation has a byte-determinism check.
- Exact-SHA agent review returned `SATISFIED` with no blocking findings. Its
  response SHA-256 is
  `a22060c44878cae3e6a78fa2e5b16f51e2663850c4afab051b5542e0ac73f634`.
  The exact-pin companion create-PR gate and the single authoritative merge
  gate passed. No Sifr compiler source changed for this item.
- The PS10 API and behavior compatibility matrix merged in companion
  [PR #28](https://github.com/sifr-lang/pydantic-sifr/pull/28) at merge commit
  `98439b38a87f5e788e4b23f43814d0e9fd93db2c`. The exact reviewed and gated
  candidate was `f74ceb4410ee7367eba73e23bcbbde345e289cd4`.
- The public matrix and its machine-readable ledger classify twelve selected
  surfaces as `same`, `adapted`, or `blocked`. The focused unit gate enforces
  complete PS10 anchor-family coverage, local evidence for delivered rows, and
  owning issue links for blocked rows. Exact-SHA agent review returned
  `SATISFIED` with no blockers. Its response SHA-256 is
  `2c00d324f3ebfbb10f6deca2a8bbad6a64d0dcefea527c42666e2e9d7e985b08`.
- The focused unit suite, file-size guard, exact-pin companion create-PR gate,
  and the single authoritative merge gate passed. No Sifr compiler source
  changed for this item.
- The PS10 migration guide merged in companion
  [PR #29](https://github.com/sifr-lang/pydantic-sifr/pull/29) at merge commit
  `3fcfcc51b3d2544047ea016ff2df7a4d726bd854`. The exact reviewed and gated
  candidate was `7cc7dfe7cf5a2b834397a781285759578b6f48ef`.
- The guide maps familiar Pydantic workflows to the supported native Sifr
  surface, identifies adapted semantics and blocked callback surfaces, and
  gives direct migration examples without compatibility shims or fallback
  paths. Twenty focused documentation-contract tests passed.
- Exact-SHA agent review returned `SATISFIED` with no blocking findings. Its
  response SHA-256 is
  `f09f84c79fec512c9cc747c5ed7198f75ac0d2a27fd9613f5bd7aa10d73e7dc7`.
  The file-size guard, exact-pin companion create-PR gate, and the single
  authoritative merge gate passed. No Sifr compiler source changed.
- The PS10 one-engine proof merged in companion
  [PR #31](https://github.com/sifr-lang/pydantic-sifr/pull/31) at merge commit
  `ad3d511de1eaa3bd0681cc937066016ba37a1395`. The exact reviewed and gated
  candidate was `606dcd14bbfc74246186a9be3a94a58dd1c36ebc`.
- The executable PS6 demo now validates JSON and string-coercing inputs through
  both the functional entry points and thin class methods, then compares every
  constructed field. A source contract requires the exact three production
  Rust bridge declarations and verifies both class methods delegate.
- The one permitted remediation review returned `SATISFIED`; its response
  SHA-256 is
  `77bd1f020e9dba2759007bb6451a8c4700838525b02f861fde24c3f509d30c0f`.
  Its broader application-facade audit observation is tracked separately by
  companion [issue #30](https://github.com/sifr-lang/pydantic-sifr/issues/30).
  The exact-pin companion create-PR gate and single merge gate passed. No Sifr
  compiler source changed.
- The PS11 foundation benchmarks merged in companion
  [PR #38](https://github.com/sifr-lang/pydantic-sifr/pull/38) at merge commit
  `1e8601d3406cbf5a5cdc16c78902c6ac14432f98`. The exact reviewed and gated
  candidate was `5b519212d8418448eab2407f594c7657988e98e3`.
- Criterion now measures parse, validate, typed construction, and serialization
  as four named operations on one representative model. The published report
  binds its measured medians to source commit
  `f8ae63a6069186b0bf811c23649a74cdf5955b96` and records the host, toolchain,
  workload, and reproduction command. agent was SATISFIED with no blockers;
  response SHA-256 is
  `30d76666e7682eea122d38997d90f6c48f7b9aaa1350e9e8501436f7c9887694`.
  The companion create-PR gate and single merge gate passed. No Sifr compiler
  source changed.
- The PS11 canonical demo merged in companion
  [PR #41](https://github.com/sifr-lang/pydantic-sifr/pull/41) at merge commit
  `31a092bfa608ce832bc8d1b2edfa3d0fabb59220`. The exact reviewed and gated
  candidate was `b4bdc52542281266937d0c639af1c50f0e0e5f0e`.
- `demos/pydantic_sifr_demo.sifr` now exercises valid JSON, aliases, defaults,
  constraints, and a stable public error. Both companion gates copy that exact
  file into a fresh dependent package, format-check it, run it, and compare its
  complete standard output with the checked snapshot. agent was SATISFIED with
  no blockers; response SHA-256 is
  `103174097170fc196a30073a6bef53c9d11fd0ac8cf113d4fdc06abf3c589bd3`.
  The companion create-PR gate and single merge gate passed, including the
  exact-file snapshot lane. No Sifr compiler source changed.
- The PS11 end-to-end package guide merged in companion
  [PR #40](https://github.com/sifr-lang/pydantic-sifr/pull/40) at merge commit
  `a741654472a8dcc76f74bf4b436076c0aea3d5a1`. The exact reviewed and gated
  candidate was `934675f420f9149debf433d39f6fc009bf506ee9`.
- The public quick start now documents model declaration, JSON validation,
  input-profile selection, stable errors, and the certified commands. Both
  dependent applications have focused guides and remain mandatory in both
  companion gates. A unit contract binds the guide, apps, and gate invocations.
  agent was SATISFIED with no blockers; response SHA-256 is
  `f2b7048b73f4e11ae50bfb49e9ef67f9e5c07afc12134bad8b7a772c72119fb8`.
  The companion create-PR gate and single merge gate passed. No Sifr compiler
  source changed.
- The PS11 version certification merged in companion
  [PR #39](https://github.com/sifr-lang/pydantic-sifr/pull/39) at merge commit
  `da4f60a2b570cd28f32ea4a43c355cf72b05b3e0`. The exact reviewed and gated
  candidate was `99db12d8aaae542d8f1f2862aa7f59acb8dc3ab9`.
- One machine-readable tuple binds the Sifr compiler and runtime source at
  `4f5492531e81385dd28efe25adfdd57dd678d2a9`, CLI `0.0.0`, package requirement
  `>=0.3,<0.4`, and package/core `0.1.0-beta.1`. The companion gate rejects a
  second tuple, version or revision drift, an unbound compiler binary, and
  stale guide values. No compatibility or fallback path exists. The first
  review found the gate and guide bindings were not independently tested. The
  bounded remediation closed both blockers, and the second review was
  SATISFIED. Review response SHA-256 values are
  `b81ec4d538e512aca23b8707ca9099cb0ac049c920272de4532160fe96f11cb8` and
  `28667e0764608ca25fcb8b81aa0ed1361b758f5028a85f70143c2ef4acb45639`.
  The companion create-PR gate and single merge gate passed. No Sifr compiler
  source changed.
- The PS10 public construction-API cleanup merged in companion
  [PR #33](https://github.com/sifr-lang/pydantic-sifr/pull/33) at merge commit
  `49a8b8c7ce9923a7f73e2100ff5ca06a838db580`. The exact reviewed and gated
  candidate was `84529ab92c793b95b324f9c3089551321fa57867`.
- The temporary `VerifiedSchemaProgram` Sifr alias, root format-version export,
  and four unused construction-time version constants were removed. A
  fail-closed test now enforces the exact package-root exports and scans every
  package source for those removed names.
- The one permitted remediation review returned `SATISFIED`; its response
  SHA-256 is
  `1a826420dd557cfa7428b6be70e70f7aa8f5ea742a9b868c9ac8388088f46e57`.
  Broader contract-test self-coverage is tracked separately by companion
  [issue #32](https://github.com/sifr-lang/pydantic-sifr/issues/32).
  The exact-pin companion create-PR gate and single merge gate passed. No Sifr
  compiler source changed.
- The PS11 manifest and pin certification merged in companion
  [PR #34](https://github.com/sifr-lang/pydantic-sifr/pull/34) at merge commit
  `13f50e206a0bf4d109ba0df164f037183ca04c75`. The exact reviewed and gated
  candidate was `65ced904b23bb60e0cc61daa1ba69aadcec8dff7`.
- Clean-checkout regeneration reproduced the 310-file, 12,754-node upstream
  manifest and the 53-schema-kind, 4-field-kind Core Schema universe. The
  companion now documents exact Pydantic and Sifr revisions, ledger digests,
  and fail-closed update-pin procedures.
- Exact-SHA agent review returned `SATISFIED`; its response SHA-256 is
  `eb549e25506a808bb8225dff8f29fce089c5ae7224d7b9deb1dca170ebe26ee9`.
  The focused audits, file-size guard, exact-pin companion create-PR gate, and
  single merge gate passed. No Sifr compiler source changed.
- The PS11 live differential gate merged in companion
  [PR #35](https://github.com/sifr-lang/pydantic-sifr/pull/35) at merge commit
  `b9a5b2417cef2b1303928044cd51c0e2afb8b849`. The exact final reviewed and
  gated candidate was `e789643fd17372b44658c392c827bc25fee637cd`.
- Five shared cases compare canonical success values or stable error code and
  location against exact Pydantic Core 2.47.0 at the pinned upstream commit.
  They cover lax and strict integers, ordered string processing, collection
  coercion, and indexed errors. The certified outcome digest is
  `b4a31e7e54b284e6a7697ef6bf0ec958c65d9a40959afcc7f85b548fb347c6aa`.
- The first review found the native probe used the default native profile after
  parsing JSON. The bounded remediation set the explicit JSON profile. The
  one permitted remediation review returned `SATISFIED`; its response SHA-256
  is `8c0dea592e52b02766b9d422cd95fce502f8358f125830b4b99dacda6b23aff1`.
  The exact-pin companion create-PR gate and single merge gate passed. Python
  remains absent from the production dependency graph. No compiler changed.
- The PS11 bounded robustness certification merged in companion
  [PR #37](https://github.com/sifr-lang/pydantic-sifr/pull/37) at merge commit
  `dfd67f9e0854e2531ea388a44a2fd6174c865bb0`. The exact gated candidate was
  `a8504850970fcf74118633299177c0b1519d0829`.
- All six declared fuzz targets now compile and execute 1,000 bounded
  randomized inputs. Scalar and special validation joined the five release
  property suites at 4,096 cases. A contract derives the complete target set
  from both the fuzz manifest and target sources. Resource, recursion, and
  panic evidence remains mandatory.
- The first review found two property suites were missing and the target set
  was hardcoded. The bounded remediation closed both defects. The second review
  then found raw-byte property generation rarely reaches validation; under the
  phase review-limit rule this new mechanism is tracked separately by companion
  [issue #36](https://github.com/sifr-lang/pydantic-sifr/issues/36), with no
  third review round. Review response SHA-256 values are
  `e30bb393739f6fbb31cef201afa4b385096173abaa2209698c6ee1540984c132`
  and `6f289177fb01b7f98126287083ccc0ed76e372e19e63982e0ca0aa01ba2e674c`.
  The exact-pin companion create-PR gate and single merge gate passed. No Sifr
  compiler source changed.
- The whole-architecture remediation merged in companion
  [PR #42](https://github.com/sifr-lang/pydantic-sifr/pull/42) at merge commit
  `0c643a676d821b92ce4dfa824a8f6a5b98073d4c`. The exact final gated candidate
  was `7d80cbf3683205237a78ecbdc6c3dd24c5c08f62`.
- Whole-phase agent review of companion commit
  `31a092bfa608ce832bc8d1b2edfa3d0fabb59220` found missing public
  serialization/schema entry points, split schema authorities, fail-open
  constraint handling, incomplete error overrides, dead legacy authorities,
  and hand-built strings errors. Review response SHA-256 was
  `e19c3d7d0cc44232a474c110320c14d6e2546c2263e4b262958c45f89d5c890b`.
- The permitted remediation review of
  `06429bfa1da6e795e018a6f45d89f05be057b377` found remaining fail-open sum,
  literal, recursive-reference, and newtype paths; unbounded unrelated
  `$defs`; and stale certification language. Review response SHA-256 was
  `6623bc2d15227f6ddafa9b3edfd5d1a8689d17ab775a8ddb5c504e076d82dcfb`.
  The phase review-limit rule prohibited a third round.
- Final candidate `7d80cbf3683205237a78ecbdc6c3dd24c5c08f62` resolves every reported
  blocker. Constraints fail closed, unsupported newtypes are rejected,
  repeated model definitions share references, static JSON Schema emits only
  root-reachable definitions under a 4,096-node bound, optional `None`
  defaults retain requiredness, and error overrides preserve typed location,
  context, and protected resource/internal errors. The canonical demo proves
  validation, mutation, serialization, and matching JSON Schema through the
  public Sifr package.
- The companion create-PR gate and single authoritative merge gate passed on
  that exact candidate. The merge gate included all release property suites at
  4,096 cases and all six fuzz targets at 1,000 runs. No Sifr compiler source
  changed for the remediation.
- Installed-artifact acceptance used a verified extracted Sifr `0.0.0`
  `aarch64-apple-darwin` archive outside both source checkouts and exact
  companion merge `0c643a676d821b92ce4dfa824a8f6a5b98073d4c`. Package fetch and formatting
  passed. Structural bridge probing then failed because the archive contains
  `crates/sifr_runtime`, whose workspace dependency requires the omitted
  `crates/sifr_structural_identity`. The archive SHA-256 is
  `2d59b37ad880ed280b9ee2a04ae32087fdc5edfd891c450527c108fa2227cde3`;
  the isolated dependent lock SHA-256 is
  `124a69b7dfa07bbca88eccc731b3dab52a6ee98acf348f5d464d64e923d8f772`.
  Sifr [issue #3233](https://github.com/sifr-lang/sifr/issues/3233) owns the
  package-neutral release-artifact fix. The row is skipped rather than
  absorbed into this companion phase.
- `milestone_ps_11` and the implementation phase are closed. The incomplete
  callback, temporal, and installed-release rows remain explicit in their
  owning issues; no compatibility layer, fallback, legacy authority, or
  Pydantic-specific compiler behavior was added.
- Deferred follow-up work: align registry representative-fixture paths with diagnostic
  baselines; align the pre-existing lowering and codegen structural-eligibility
  predicates for fixed-width platform integers, metadata, and imported classes;
  audit pre-epoch fractional timestamp reconstruction; disambiguate imported
  structural metadata/default lookup from colliding local names; accept large negative
  const bounds in integer-boundary decorators; and keep shared floor-arithmetic semantics
  from drifting between frontend const evaluation and runtime execution.

## Milestone Compatibility Inventories

These accepted inventories assign compatibility evidence to delivery milestones.
The companion repository owns the executable current ledgers; this phase record
preserves the reviewed delivery allocation.

### Pin-derived node-kind disposition

The following table is total over the 53 literals in the sole oracle's
`CoreSchemaType` at
`pydantic-core/python/pydantic_core/core_schema.py:4247-4301`. The external
repository generates `tests/provenance/core_schema_kinds.toml` directly from
that literal and fails exact-set equality unless every kind has exactly one
row, compatibility class, normal form, and primary implementation owner, plus
a non-empty set of evidence families or the `ps_0` disposition audit.
Several evidence families may support one owner; they do not create a second
implementation owner. The same audit covers all four `CoreSchemaFieldType`
literals. A pin update regenerates the universe before any hand-authored
disposition changes, making a newly added kind a blocking review item rather
than a silent omission.

| Pydantic Core kind | Class | Sifr normal form or explicit disposition | Owner/evidence |
| --- | --- | --- | --- |
| `invalid` | `rejected` | Static schema construction fails; no executable invalid node exists | `ps_4` / `core/schema_contract` |
| `any` | `not-applicable` | No untyped runtime node exists; harness occurrences normalize to the smallest concrete child schema | `ps_0` disposition audit |
| `none` | `same` | none scalar | `ps_5` / `validators/none` |
| `bool` | `same` | bool scalar | `ps_5` / `validators/numeric` |
| `int` | `adapted` | exact/fixed integer plus locked Sifr JSON profile | `ps_5` / `validators/numeric`, `core/fixed_integer` |
| `float` | `same` | float scalar | `ps_5` / `validators/numeric` |
| `decimal` | `adapted` | finite exact `bigdecimal` scalar | `ps_5` / `validators/numeric`, `core/decimal_digit_counting` |
| `fraction` | `adapted` | package-owned exact `Fraction` scalar | `ps_5` / `core/fraction` |
| `str` | `adapted` | native string node and Rust-regex policy | `ps_5` / `validators/text_bytes`, `core/string_pipeline_order` |
| `bytes` | `same` | bytes scalar | `ps_5` / `validators/text_bytes` |
| `date` | `same` | date scalar | `ps_5` / `validators/temporal` |
| `time` | `adapted` | lossless Sifr time with declared offset policy | `ps_5` / `validators/temporal` |
| `datetime` | `adapted` | lossless Sifr datetime with declared offset policy | `ps_5` / `validators/temporal` |
| `timedelta` | `adapted` | Sifr duration | `ps_5` / `validators/temporal` |
| `literal` | `same` | literal sum node | `ps_7` / `validators/literal` |
| `missing-sentinel` | `not-applicable` | Missing-input state normalizes to defaults/`Option`; no identity value enters `T` | `ps_0` disposition audit |
| `enum` | `adapted` | enum variant tag plus declared scalar-value metadata | `ps_7` / `validators/enum` |
| `is-instance` | `not-applicable` | No runtime Python class identity or arbitrary object input | `ps_0` disposition audit |
| `is-subclass` | `not-applicable` | No runtime subclass reflection | `ps_0` disposition audit |
| `callable` | `not-applicable` | Sifr functions are statically typed callbacks, not user-data values | `ps_0` disposition audit |
| `list` | `same` | list collection | `ps_5` / `validators/collections` |
| `tuple` | `same` | tuple product | `ps_5` / `validators/collections` |
| `set` | `same` | set collection | `ps_5` / `validators/collections` |
| `frozenset` | `adapted` | immutable Sifr `frozenset[T]` | `ps_5` / `validators/collections` |
| `generator` | `adapted` | `ValidatedIterator[T]`; `next()` returns `Result[Option[T], ValidationError]` and never hides deferred failure | `ps_5` / `validators/generator` |
| `dict` | `same` | typed mapping | `ps_5` / `validators/collections` |
| `function-after` | `adapted` | typed after callback | `ps_7` / `validators/callbacks_context` |
| `function-before` | `adapted` | typed before callback | `ps_7` / `validators/callbacks_context` |
| `function-wrap` | `adapted` | typed wrap callback | `ps_7` / `validators/callbacks_context` |
| `function-plain` | `adapted` | typed plain callback | `ps_7` / `validators/callbacks_context` |
| `default` | `adapted` | statically typed const or validated factory default | `ps_6` / `validators/defaults` |
| `nullable` | `same` | nullable/optional control | `ps_6` / `validators/nullable` |
| `union` | `adapted` | smart or left-to-right ordinary union | `ps_7` / `validators/unions`, `core/smart_union_ranking` |
| `tagged-union` | `adapted` | field/path or typed-callback discriminator | `ps_7` / `validators/tagged_unions` |
| `chain` | `adapted` | flattened typed sequential chain | `ps_7` / `validators/control_composition` |
| `lax-or-strict` | `same` | strict/lax branch selected by static default or call override | `ps_7` / `validators/control_composition` |
| `json-or-python` | `adapted` | JSON/structural-input branch; no Python branch | `ps_7` / `validators/control_composition` |
| `typed-dict` | `adapted` | fixed-layout Sifr record | `ps_6` / `validators/typed_dict` |
| `model-fields` | `adapted` | record/model field plan | `ps_6` / `validators/model_fields` |
| `model` | `adapted` | ordinary Sifr class plus structural construction | `ps_6` / `core/json_models`, `api/base_model` |
| `dataclass-args` | `adapted` | record input/field plan; no separate constructor-argument object | `ps_6` / `validators/dataclasses` |
| `dataclass` | `adapted` | ordinary Sifr class; normalizes to the same model/record node | `ps_6` / `validators/dataclasses` |
| `arguments` | `not-applicable` | Python call-signature validation is static Sifr call checking | `ps_0` disposition audit |
| `arguments-v3` | `not-applicable` | Python call-signature validation is static Sifr call checking | `ps_0` disposition audit |
| `call` | `not-applicable` | `validate_call` is replaced by ordinary typed Sifr calls | `ps_0` disposition audit |
| `custom-error` | `adapted` | compositional `ErrorOverride` with static package code/message | `ps_4` / `core/schema_contract` |
| `json` | `adapted` | embedded-JSON decoder wrapping a typed child schema | `ps_5` / `validators/embedded_json` |
| `url` | `adapted` | package-owned URL scalar | `ps_5` / `validators/url` |
| `multi-host-url` | `adapted` | package-owned ordered multi-host URL/DSN scalar | `ps_10` / `core/multi_host_url_serialization`, `api/networks` |
| `definitions` | `same` | definitions table | `ps_7` / `validators/definitions_recursion` |
| `definition-ref` | `same` | typed definition reference | `ps_7` / `validators/definitions_recursion` |
| `uuid` | `adapted` | package-owned UUID scalar | `ps_5` / `validators/uuid` |
| `complex` | `adapted` | package-owned `Complex` scalar | `ps_5` / `validators/complex` |

The four field kinds normalize as follows:

| Pydantic Core field kind | Class | Sifr normal form | Owner/evidence |
| --- | --- | --- | --- |
| `model-field` | `adapted` | declared model field metadata | `ps_6` / `validators/model_fields` |
| `dataclass-field` | `adapted` | the same declared record field metadata | `ps_6` / `validators/dataclasses` |
| `typed-dict-field` | `adapted` | the same declared record field metadata | `ps_6` / `validators/typed_dict` |
| `computed-field` | `adapted` | serialization-only computed field | `ps_8` / `api/serialization` |

Nodes are orthogonal and compositional. For example, constrained integers are
an integer node plus constraints rather than independent positive-int,
negative-int, bounded-int, and strict-int validator implementations.
Set uniqueness follows from the set collection node rather than a second
constraint. User refinements compose a base node with existing constraints or
a typed validator; there is no redundant catch-all "typed refinement" node.

The pinned `lax-or-strict`, `json-or-python`, and `chain` kinds become general
control nodes instead of Python special cases. Sifr names the second
`json-or-structural`: JSON entry points select its JSON branch and native
structural entry points select its structural branch. Canonicalization flattens
nested typed chains and rejects an empty chain; it may erase a one-step chain.
Strict/lax selection obeys the adapter/model default plus the explicit per-call
override.

The package supplies ordinary typed `Fraction` and `Complex` values backed by
focused Rust numeric crates; they are not compiler primitives. Their scalar
nodes own strict/lax parsing, exact rational constraints, finite/non-finite
complex policy, and serialization through the same schema program.

Two pinned experimental capabilities deliberately do not survive as public
Sifr validation behavior. `missing-sentinel` is a real Core Schema node but is
`not-applicable`: missing input is validation state before construction and
public Sifr values use defaults or `Option[T]`, so no singleton identity leaks
into a model. `allow_partial` is not a Core Schema node; it is a validation-call
mode and is `rejected` for model/adapter validation because silently discarding
an invalid tail cannot produce the promised complete `T`. Incremental framing
is a separate streaming-input concern and never a fallback validation mode.

The enum node maps declared exact-integer or string input literals to ordinary
payload-free Sifr enum variant tags. The mapping is package declaration
metadata, not a Rust/Sifr enum discriminant, so string-valued and
arbitrary-precision integer-valued Pydantic enums do not require associated
data or an expanded compiler enum representation. Serialization consults the
same mapping; construction and projection carry only the Sifr variant tag.

The string node has built-in `strip_whitespace`, `to_upper`, `to_lower`,
`ascii_only`, and `coerce_numbers_to_str` policies rather than synthesizing
user callbacks. Its fixed pipeline, matching the pinned `string.rs` validator,
is input conversion, whitespace stripping, ASCII restriction, Unicode-scalar
length checks, pattern check, then case conversion
(`pydantic-core/src/validators/string.rs:110-178`). Byte lengths count bytes.
A Sifr-native discriminating fixture in `core/string_pipeline_order` contains
pairwise-conflicting inputs that prove strip before ASCII, ASCII before length
and pattern, length before pattern, and all checks before case conversion; it
also proves Unicode-scalar versus byte length. Rust `regex` syntax and flags
are the sole regular-expression engine. Explicit Python-regex mode is
`not-applicable`; a portable precompiled Python pattern is `adapted` by
compile-time translation of supported source and flags to native `re.Pattern`,
while Python-only syntax or flags are rejected at pattern construction.

Requesting both `to_upper` and `to_lower` is rejected as an intentional
stricter Sifr schema rule. Decimal constraints retain both raw and normalized
digit counts. `decimal_max_digits` and `decimal_max_places` are emitted only
when both the raw and normalized forms exceed their respective limit.
`decimal_whole_digits` applies only when both `max_digits` and
`decimal_places` are present: the allowed whole digits are
`max_digits.saturating_sub(decimal_places)`, each observed whole count is
`digits.saturating_sub(decimal_digits)`, and the error is emitted only when
both raw and normalized observed counts exceed the allowance. The native
`core/decimal_digit_counting` family fixes zero, trailing-zero, fractional,
and saturating-subtraction edge cases; it asserts all three exact error codes.
Constraint emission is first-match-wins in `max_digits`,
`decimal_places`, then `decimal_whole_digits` order, and each error context
carries the configured allowance rather than the observed count
(`pydantic-core/src/validators/decimal.rs:152-197`).

Clock-relative date/time constraints carry `past` or `future` plus the declared
UTC-offset policy. Validation captures one call-scoped UTC instant before
executing the plan. Production uses the package's `SystemClock`; tests and
differential fixtures inject a fixed typed clock through the same internal
capability. Every comparison in one validation call uses that snapshot, so the
result is deterministic for `(schema, input, clock)`.

### Pinned conformance manifest

The following tables are the normative minimum implementation scope at the
researched upstream revisions. Together with the total-set rule below, they
replace an open-ended instruction to "port Pydantic tests" with
milestone-owned evidence.

Immediately after `ps_4` creates the external repository and before any core
implementation begins, that repository stores the executable form as
`tests/provenance/upstream_manifest.toml`. A manifest entry identifies an exact
upstream commit, path, pytest selector, parameter identity, compatibility
class, owning milestone, normalized fixture destination, and disposition
reason. Its upstream file set is computed from the pinned Git trees, not from
the hand-written tables:

- tracked entries are enumerated from the pinned Git tree, not a recursive
  filesystem walk. The tracked `tests/pydantic_core` symlink is classified once
  as infrastructure and never followed; engine files are enumerated only from
  `pydantic-core/tests`;
- API and Core pytest collection run in two isolated processes with the
  Pydantic pin's committed `uv.lock`: pytest 9.1.1 and its locked plugins. API
  collection uses `<pydantic-pin>/tests` plus
  `--ignore=<pydantic-pin>/tests/pydantic_core`; Core collection separately
  uses `<pydantic-pin>/pydantic-core/tests`. Normalized identities are prefixed
  `api::` or `core::`, so duplicate basenames cannot collide or acquire a
  second owner. Neither process uses the repository root; explicit path
  arguments bypass the pin's `testpaths` default and prevent it from expanding
  the node universe;
- the standalone 2.41.5 Pydantic Core research checkout is recorded as an
  excluded research source outside the ledger, while the in-tree 2.47.0
  component and API share one immutable Git commit;
- every file is classified as collected conformance, benchmark, fixture,
  infrastructure, or not applicable;
- test collection runs before file-role classification; any file that yields a
  pytest node cannot be hidden as fixture or infrastructure, and every
  collected node and parameter identity in any file is classified as `same`,
  `adapted`, `not-applicable`, or `rejected`;
- sorted upstream paths, collected node identities, and parameter identities
  must exactly equal the manifest ledger; an added, removed, renamed, skipped,
  or unclassified path, node, or parameter fails the audit; and
- the manifest records a content hash for the complete sorted ledger, making
  silent omission from the hand-authored tables detectable.

A collected parameter case records the ordered identity of every contributing
parametrization source. Literal lists use each parameter's normalized
AST-content hash plus its zero-based occurrence among identical hashes.
Non-literal sources are evaluated through pytest collection and use a hash of
their normalized source-dependency closure plus the collected ordinal and
value fingerprint. The dependency closure recursively includes referenced
module constants, comprehensions, starred sources, factory functions, and
fixture parameter declarations. Value fingerprints recursively encode
primitives and containers; types/functions use module, qualified name, and
source hash. An opaque value with no deterministic source fingerprint makes
the audit fail rather than falling back to `repr` or object identity. The
ledger records multiplicity, so repeated equal parameters remain distinct and
a generated source/value change cannot silently preserve the old identity.

The tables then define required implementation anchors and milestone ownership.
The module column identifies the source of that row's anchors; it does not
assign every test in a broad upstream module to the same milestone. A
non-anchor `same` or `adapted` node is owned by the milestone that implements
its feature. These additional rules apply:

- every test and parameter is classified, including cases that are not ported;
- every `same` or `adapted` behavior maps to an explicit Core Schema node,
  error-contract capability, or named Sifr-native contract with one
  implementation milestone and gate; an uncovered capability fails the audit;
- every ordering, precedence, ranking, or normalization algorithm claimed as
  parity records its pinned implementation source and at least one
  discriminating assertion; a non-discriminating anchor cannot certify it;
- the selectors below are mandatory provenance anchors for portable behavior,
  not instructions to copy an upstream test body;
- an anchor must pass, without `xfail` or `skip`, at the pinned revision and
  must contain at least one observable behavioral assertion relevant to Sifr;
- a truthiness-only assertion, import/build smoke test, Python `repr`,
  reflection invariant, or assertion solely about a rejected/not-applicable
  mechanism cannot be a portable anchor;
- for a mixed upstream test, the manifest identifies each retained assertion
  or parameter by a stable AST-content hash, records its normalized Sifr
  expectation, and separately classifies every omitted assertion/parameter;
- a forbidden Python mechanism may appear only as replaceable test harness
  setup around a retained portable assertion, with the Sifr replacement and
  adaptation reason recorded; it cannot be the behavior being asserted;
- every `same` or `adapted` parameterization of an anchor is retained;
  omission is allowed only for an individually justified `not-applicable` or
  `rejected` parameter;
- `py_and_json` cases become both native structural-input and JSON-input
  fixtures where both modes have Sifr meaning;
- Python exception classes, object representations, and versioned message URLs
  are normalized to the selected Sifr error contract; and
- upstream arbitrary custom-error strings normalize to package-qualified Sifr
  custom codes while retaining the declared static message and wrapper
  location; and
- a missing, renamed, ambiguously qualified, or unclassified selector fails the
  upstream audit and the owning milestone gate.

Within a row, a bare filename inherits the directory of the preceding full
path. A bare selector is allowed only when it resolves in exactly one module in
that row; collisions use the full `path::selector`. Repeating an anchor source
module in a later milestone is allowed when different anchors become
executable later, but each retained assertion/parameter has exactly one owning
milestone.

#### Pydantic Core engine baseline

All paths in this table are relative to the authoritative in-tree engine root
`<pydantic checkout at f59e929c999e8b2efc7b12fd0bc1685c1a186be3>/pydantic-core`.
They never resolve against the standalone 2.41.5 research checkout.

| Milestone | Anchor source module | Mandatory portable selector anchors | Fixture family |
| --- | --- | --- | --- |
| `ps_4` | `tests/test_schema_functions.py` | `test_invalid_custom_error`, `test_invalid_custom_error_type`, `test_err_on_invalid` | `core/schema_contract` |
| `ps_4` | `tests/test_json.py` | `test_json_invalid` | `core/json_foundation` |
| `ps_5` | `tests/test_json.py` | `test_input_types`, `test_bool`, `test_int`, `test_float`, `test_json_bytes_base64_round_trip`, `test_json_bytes_base64_invalid` | `core/json_values` |
| `ps_5` | `tests/test_errors.py` | `test_error_json`, `test_error_json_loc`, `test_hide_input_in_error`, `test_error_type` | `core/validation_errors` |
| `ps_5` | `tests/validators/test_bool.py`, `test_int.py`, `test_float.py`, `test_decimal.py` | `test_bool`, `test_bool_strict`, `test_int_py_and_json`, `test_int_strict`, `test_int_kwargs`, `test_float`, `test_float_strict`, `test_float_kwargs`, `test_decimal`, `test_decimal_strict_json`, `test_decimal_kwargs`, `test_validate_max_digits_and_decimal_places` | `validators/numeric` |
| `ps_5` | `tests/validators/test_complex.py` | `test_complex_cases`, `test_complex_strict`, `test_json_complex`, `test_string_complex` | `validators/complex` |
| `ps_5` | `tests/validators/test_string.py`, `test_bytes.py` | `test_str`, `test_constrained_str`, `test_coerce_numbers_to_str`, `test_strict_bytes_validator`, `test_lax_bytes_validator`, `test_constrained_bytes` | `validators/text_bytes` |
| `ps_5` | `tests/validators/test_date.py`, `test_datetime.py`, `test_time.py`, `test_timedelta.py` | `test_date_json`, `test_date_strict_json`, `test_date_kwargs`, `test_datetime_json`, `test_datetime_strict_json`, `test_datetime_past`, `test_datetime_future`, `test_time_json`, `test_time_strict_json`, `test_timedelta_json`, `test_timedelta_strict_json`, `test_timedelta_kwargs` | `validators/temporal` |
| `ps_5` | `tests/validators/test_list.py`, `test_tuple.py`, `test_dict.py`, `test_set.py`, `test_frozenset.py` | `test_list_py_or_json`, `test_list_error`, `test_list_length_constraints`, `test_tuple_json`, `test_tuple_validate`, `test_multiple_missing`, `test_dict`, `test_dict_value_error`, `test_dict_length_constraints`, `test_set_ints_both`, `test_set_multiple_errors`, `test_frozenset_ints_both`, `test_frozenset_multiple_errors` | `validators/collections` |
| `ps_5` | `tests/validators/test_generator.py` | `test_generator_json_int`, `test_error_index`, `test_too_long`, `test_too_short` | `validators/generator` |
| `ps_5` | `tests/validators/test_json.py` | `test_any`, `test_list_int`, `test_dict_key` | `validators/embedded_json` |
| `ps_5` | `tests/validators/test_none.py` | `test_python_none`, `test_json_none` | `validators/none` |
| `ps_5` | `tests/validators/test_url.py` | `test_url_ok`, `test_url_error`, `test_max_length`, `test_allowed_schemes_ok`, `test_allowed_schemes_error`, `test_url_vulnerabilities` | `validators/url` |
| `ps_5` | `tests/validators/test_uuid.py` | `test_uuid`, `test_uuid_strict`, `test_uuid_version`, `test_uuid_json` | `validators/uuid` |
| `ps_5` | `tests/test_validate_strings.py` | `test_bool`, `test_validate_strings`, `test_dict` | `core/strings_profile` |
| `ps_6` | `tests/test_json.py` | `test_typed_dict`, `test_error_loc` | `core/json_models` |
| `ps_6` | `tests/test_errors.py` | `test_loc_with_dots` | `core/model_error_locations` |
| `ps_6` | `tests/test_validate_strings.py` | `test_typed_dict` | `core/strings_profile_models` |
| `ps_6` | `tests/validators/test_model_fields.py` | `test_simple`, `test_with_default`, `test_missing_error`, `test_fields_required_by_default`, `test_alias`, `test_alias_path`, `test_alias_error_loc_alias`, `test_ignore_extra`, `test_forbid_extra` | `validators/model_fields` |
| `ps_6` | `tests/validators/test_typed_dict.py` | `test_simple`, `test_with_default`, `test_missing_error`, `test_fields_required_by_default`, `test_model_deep`, `test_alias`, `test_alias_path`, `test_alias_error_loc_alias`, `test_ignore_extra`, `test_forbid_extra` | `validators/typed_dict` |
| `ps_6` | `tests/validators/test_dataclasses.py` | `test_dataclass_args`, `test_aliases`, `test_dataclass`, `test_dataclass_field_after_validator`, `test_dataclass_json` | `validators/dataclasses` |
| `ps_6` | `tests/validators/test_with_default.py` | `test_typed_dict_default` | `validators/defaults` |
| `ps_6` | `tests/validators/test_nullable.py` | `test_nullable` | `validators/nullable` |
| `ps_7` | `tests/validators/test_literal.py` | `test_literal_py_and_json`, `test_big_int` | `validators/literal` |
| `ps_7` | `tests/validators/test_enums.py` | `test_plain_enum`, `test_int_enum`, `test_str_enum`, `test_enum_exactness`, `test_big_int` | `validators/enum` |
| `ps_7` | `tests/validators/test_union.py` | `test_union_bool_int`, `test_int_float`, `test_left_to_right_union`, `test_smart_union_json_string_types`, `test_smart_union_model_field`, `test_td_smart_union_by_fields_set`, `test_smart_union_does_nested_model_field_counting`, `test_nested_unions_bubble_up_field_count`, `test_smart_union_validator_function`, `test_case_labels`, `test_custom_error` | `validators/unions` |
| `ps_7` | `tests/validators/test_lax_or_strict.py`, `test_json_or_python.py`, `test_chain.py` | `test_lax_or_strict`, `test_lax_or_strict_default_strict`, `test_json_or_python`, `test_chain`, `test_chain_many`, `test_chain_error`, `test_flatten`, `test_chain_empty`, `test_chain_one` | `validators/control_composition` |
| `ps_7` | `tests/validators/test_tagged_union.py` | `test_simple_tagged_union`, `test_discriminator_path`, `test_discriminator_function`, `test_custom_error` | `validators/tagged_unions` |
| `ps_7` | `tests/validators/test_nullable.py` | `test_union_nullable_bool_int` | `validators/nullable_union` |
| `ps_7` | `tests/validators/test_definitions.py`, `test_definitions_recursive.py` | `test_repeated_ref`, `test_deep`, `test_branch_nullable`, `test_recursion_branch`, `test_complex_recursive_type` | `validators/definitions_recursion` |
| `ps_7` | `tests/validators/test_function.py`, `tests/test_validation_context.py`, `tests/validators/test_with_default.py` | `test_function_before`, `test_function_wrap`, `test_function_after`, `test_function_plain`, `test_model_field_before_validator`, `test_model_field_after_validator`, `test_model_field_wrap_validator`, `test_after`, `test_mutable_context`, `test_typed_dict`, `test_wrap`, `test_validate_default_factory`, `test_default_value_validate_default_fail` | `validators/callbacks_context` |
| `ps_8` | `tests/serializers/test_simple.py`, `test_bytes.py`, `test_datetime.py`, `test_decimal.py`, `test_complex.py` | `test_simple_serializers`, `test_float_inf_and_nan_serializers`, `test_bytes`, `test_bytes_base64`, `test_bytes_hex`, `test_datetime`, `test_datetime_json`, `test_date`, `test_time`, `test_decimal`, `test_decimal_json`, `test_complex_json` | `serializers/scalars` |
| `ps_8` | `tests/serializers/test_timedelta.py` | `test_timedelta`, `test_timedelta_float`, `test_config_timedelta`, `test_timedelta_key` | `serializers/duration` |
| `ps_8` | `tests/serializers/test_list_tuple.py` | `test_list_any`, `test_include`, `test_exclude`, `test_filter`, `test_filter_runtime`, `test_filter_runtime_more`, `test_positional_tuple`, `test_filter_args_nested` | `serializers/sequences` |
| `ps_8` | `tests/serializers/test_dict.py` | `test_dict_str_int`, `test_include`, `test_exclude`, `test_filter`, `test_filter_runtime`, `test_filter_args_nested` | `serializers/mappings` |
| `ps_8` | `tests/serializers/test_set_frozenset.py` | `test_set_any`, `test_frozenset_any` | `serializers/sets` |
| `ps_8` | `tests/serializers/test_typed_dict.py` | `test_typed_dict`, `test_include_exclude_args`, `test_alias`, `test_exclude_none`, `test_exclude_default` | `serializers/typed_dict` |
| `ps_8` | `tests/serializers/test_model.py` | `test_model`, `test_include_exclude_args`, `test_alias`, `test_exclude_none`, `test_advanced_exclude_nested_lists`, `test_computed_field_exclude_none` | `serializers/models` |
| `ps_8` | `tests/serializers/test_enum.py`, `test_literal.py`, `test_none.py`, `test_nullable.py` | `test_plain_enum`, `test_int_enum`, `test_str_enum`, `test_int_literal`, `test_str_literal`, `test_none_fallback`, `test_nullable` | `serializers/sums` |
| `ps_8` | `tests/serializers/test_union.py`, `test_definitions_recursive.py`, `test_functions.py` | `test_union_bool_int`, `test_typed_dict_literal`, `test_tagged_union`, `test_tagged_union_with_aliases`, `test_branch_nullable`, `test_function_known_type`, `test_function_only_json` | `serializers/unions_callbacks_recursion` |
| `ps_8` | `tests/serializers/test_definitions.py` | `test_custom_ser`, `test_repeated_ref`, `test_deep`, `test_use_after` | `serializers/definitions` |
| `ps_8` | `tests/serializers/test_url.py`, `test_uuid.py` | `test_url`, `test_url_dict_keys`, `test_uuid`, `test_uuid_key`, `test_uuid_json` | `serializers/url_uuid` |
| `ps_10` | `tests/serializers/test_url.py` | `test_multi_host_url` | `core/multi_host_url_serialization` |

Tests whose Python harness uses `@property` or another descriptor to expose a
computed value are `adapted`: Sifr ports the computed-field serialization
behavior through its statically typed computed-field declaration, not the
descriptor mechanism. In
`serializers/test_model.py::test_computed_field_exclude_none`, the upstream
computed field is normalized to a declared nullable-integer computed field so
both the `None` value and `exclude_none` behavior remain observable without a
wrong-type serializer path.

Upstream `any_schema()`, untyped-container, and `Any`-annotated harnesses are
adapted per retained assertion to the smallest concrete Sifr structural type
that contains that assertion's values. A heterogeneous upstream mapping is
normalized to a declared union key type and concrete value type when those
types represent the retained values; otherwise that parameter is
`not-applicable` with its reason recorded. Neither adaptation introduces
`Any`, an untyped callback, or a recursive dynamic value tree.

Serializer anchors that mix correct typed values with wrong-type
warning/passthrough assertions retain only the correct-value assertions. The
wrong-type assertions are individually `not-applicable`. Likewise,
`test_simple_serializers` excludes subclass-identity parameters, and
`test_none_fallback` retains only parameters where `None` matches the declared
schema. These are assertion/parameter classifications in the manifest, not
alternate runtime paths.

`serializers/sequences::test_positional_tuple` is a mixed anchor: only its
correct declared-tuple serialization assertions are retained; warning,
passthrough, and Python-object `Any` cases are individually
`not-applicable`. `core/validation_errors::test_error_type` anchors canonical
error-code/message availability only; its direct constructor `.type` and
`.context` round-trips do not claim decimal-validator execution. Exact decimal
emission is instead required by `core/decimal_digit_counting` and
`api/constraints::test_decimal_validation`.

`validators/dataclasses` adapts declared field input, aliases, construction,
typed callbacks, and JSON behavior to ordinary Sifr classes. Python dataclass
subclass identity, generated Python initializer signatures, slots,
descriptors, `ArgsKwargs` positional-call harness cases, and object-layout
assertions are individually `not-applicable`; mapping/JSON input cases and
their normalized errors remain.

Enum-member identity assertions that distinguish an enum branch from its
underlying scalar branch are adapted to the selected union variant tag plus
value. The Python `is` relation itself remains `not-applicable`.
Python `str`-enum and beyond-`i64` integer-enum values become package-declared
variant value metadata; retained validation, error alternatives, union tags,
and serialization use those exact literals without changing Sifr's
payload-free enum representation.

Callable-form-only parameterizations, such as
`api/serialization::test_serialize_partial`, collapse to one typed callback
fixture when the retained input, callback effect, and output are identical.
Every collapsed Python callable-form parameter is still individually
classified as `adapted` and points to that shared neutral fixture.

Pydantic public-API anchors may use Python models as their harness, but the
neutral expectation retains only declared field values, serialized output, or
normalized errors. For context-aware anchors it may also retain typed callback
outputs, ordered callback traces, and caller-visible mutations of the declared
context. Assertions about `__dict__`, field-set state, runtime model rebuilds,
Python signatures, internal metadata/reprs, subclass identity, or exception
objects are excluded and classified separately. Heterogeneous Python contexts
become separate statically specialized fixtures with `NoContext` or one
concrete Sifr context type each.

The same projection rule applies to Pydantic Core model-field anchors whose
asserted tuple combines declared values, typed allowed extras, and Python
field-set state: tuple element 0 is retained, element 1 is retained only for a
declared typed `extra_behavior='allow'` destination, and element 2 is
`not-applicable`. The neutral fixture stores those retained observations
separately rather than preserving the Python harness tuple.

`validators/callbacks_context::test_default_value_validate_default_fail` is
adapted from its upstream runtime exception to the build-time diagnostic
required for its const-evaluable invalid default. Its error code and element
location remain provenance inputs; it does not introduce a runtime static
schema failure path.

The Core Schema decimal node is always finite because its Sifr output is
`bigdecimal`. The four `allow_inf_nan` parameters in
`validators/numeric::test_decimal_kwargs` are individually
`not-applicable`; non-finite numeric contracts use the float node rather than
a second decimal representation.

In smart-union anchors, an upstream `isinstance` assertion used only to
identify the chosen model arm is adapted to the corresponding Sifr union
variant tag. Python subclass/reflection behavior remains `not-applicable`; the
selected-arm observation is retained.

`validators/control_composition::test_json_or_python` adapts the Python branch
to Sifr's native structural-input branch. Its observable source-dependent
branch selection and output are retained; Python subclass mechanics and
runtime class reflection are not.

#### Pydantic Sifr-API baseline

All paths in this table are relative to the authoritative repository root
`<pydantic checkout at f59e929c999e8b2efc7b12fd0bc1685c1a186be3>`.

| Milestone | Anchor source module | Mandatory portable selector anchors | Fixture family |
| --- | --- | --- | --- |
| `ps_6` | `tests/test_types.py` | `test_constrained_bytes_good`, `test_constrained_bytes_too_long`, `test_constrained_list_good`, `test_constrained_list_too_long`, `test_constrained_set_good`, `test_constrained_set_too_short`, `test_constrained_str_good`, `test_constrained_str_too_long`, `test_string_too_long`, `test_string_too_short`, `test_string_constraints_ascii_only`, `test_decimal_validation` | `api/constraints` |
| `ps_6` | `tests/test_main.py` | `test_success`, `test_ultra_simple_missing`, `test_ultra_simple_failed`, `test_nullable_strings_success`, `test_parent_sub_model`, `test_required`, `test_default_factory_called_once_2`, `test_allow_extra`, `test_forbidden_extra_fails`, `test_model_validate_strict`, `test_model_validate_json_strict` | `api/base_model` |
| `ps_6` | `tests/test_aliases.py` | `test_basic_alias`, `test_alias_error_loc_by_alias`, `test_pop_by_field_name`, `test_validation_alias`, `test_validation_alias_parse_data`, `test_validation_alias_priority_json` | `api/aliases` |
| `ps_6` | `tests/test_config.py`, `tests/test_fields.py` | `test_config_inf_nan_disabled`, `test_hide_input_in_errors`, `test_populate_by_name_still_effective`, `test_coerce_numbers_to_str_field_option` | `api/config_fields` |
| `ps_7` | `tests/test_validators.py`, `tests/test_model_validator.py` | `test_annotated_validator_before`, `test_annotated_validator_after`, `test_annotated_validator_plain`, `test_annotated_validator_wrap`, `test_annotated_validator_runs_before_field_validators`, `test_validate_multiple`, `test_field_validator_validate_default`, `test_model_validator_before`, `test_model_validator_after`, `test_model_validator_wrap` | `api/validators` |
| `ps_7` | `tests/test_discriminated_union.py` | `test_discriminated_union_validation`, `test_discriminated_annotated_union`, `test_discriminated_union_int`, `test_nested_discriminated_union`, `test_callable_discriminated_union_recursive` | `api/discriminated_unions` |
| `ps_7` | `tests/test_generics.py`, `tests/test_forward_ref.py` | `test_value_validation`, `test_alongside_concrete_generics`, `test_complex_nesting`, `test_required_value`, `test_self_forward_ref_collection`, `test_recursive_model` | `api/generics_recursion` |
| `ps_8` | `tests/test_serialize.py`, `tests/test_computed_fields.py`, `tests/test_aliases.py` | `test_serializer_annotated_plain_json`, `test_serializer_annotated_wrap_json`, `test_model_serializer_plain`, `test_model_serializer_wrap`, `test_serialize_partial`, `test_serialize_json_context`, `test_computed_fields_get`, `test_include_exclude`, `test_exclude_none`, `test_computed_field_with_field_serializer`, `test_serialization_alias` | `api/serialization` |
| `ps_9` | `tests/test_type_adapter.py`, `tests/test_types.py`, `tests/test_serialize.py` | `test_types`, `test_validate_python_strict`, `test_validate_python_context`, `test_validate_json_context`, `test_validate_strings_dict`, `test_decimal_precision`, `test_type_adapter_dump_json` | `api/type_adapter` |
| `ps_9` | `tests/types/test_fraction.py`, `tests/test_types.py` | `test_fraction`, `test_fraction_validate_json`, `test_fraction_validate_strings`, `test_fraction_validation_error_strict`, `test_fraction_dump_json`, `test_strict_complex_field` | `api/specialized_numeric` |
| `ps_9` | `tests/test_json_schema.py`, `tests/test_aliases.py` | `test_by_alias`, `test_sub_model`, `test_optional`, `test_list_union_dict`, `test_constraints_schema_validation`, `test_constraints_schema_serialization`, `test_literal_schema`, `test_new_type_schema`, `test_schema_with_refs`, `test_nested_generic`, `test_discriminated_union`, `test_computed_field`, `test_type_adapter_json_schemas_title_description`, `test_aliases_json_schema` | `api/json_schema` |
| `ps_10` | `tests/test_networks.py` | `test_http_url_success`, `test_http_url_invalid`, `test_any_url_parts`, `test_postgres_dsns`, `test_multihost_postgres_dsns`, `test_json_schema`, `test_url_ser` | `api/networks` |
| `ps_10` | `tests/test_types.py` | `test_pattern` | `api/pattern` |
| `ps_10` | `tests/test_root_model.py` | `test_root_model_validation_error`, `test_root_model_nested`, `test_model_validator_before`, `test_model_validator_after`, `test_root_model_json_schema_meta` | `api/root_model` |
| `ps_10` | `tests/test_annotated.py` | `test_compatible_metadata_raises_correct_validation_error`, `test_decimal_constraints_after_annotation` | `api/field_metadata` |

For `api/pattern`, the string-pattern parameters retain compilation,
match/non-match behavior, and the regex JSON Schema format. Python class-name
reflection and object identity are `not-applicable`; the bytes-pattern
parameter is also `not-applicable` because the canonical Sifr `re.Pattern`
accepts strings.

The tables intentionally name engine and public-API anchors rather than copying
Python scaffolding, while the total-set ledger still enumerates every upstream
test file and collected node. Pydantic modules for pickle, private attributes,
metaclass dynamics, runtime model creation, Python call signatures, import
behavior, mypy plugins, and CPython object lifecycle therefore receive explicit
file/node-level `not-applicable` classifications; absence from the anchor tables
does not remove them from the audited universe.

## Prerequisites and Dependency Order

Phase 27's no-user-panic, stable diagnostic code/severity/span/URL/schema,
deterministic recovery, and CLI exit-code invariants bind every compiler
milestone. Each compiler PR must pass
`scripts/run_all_tests.sh --profile create-pr`; merge readiness requires
`scripts/run_all_tests.sh`. The external repository establishes equivalent
checked-in create-PR and merge gates before its first implementation commit.

The following Sifr capabilities must be merged and certified before the
companion repository depends on them:

| Required by | Prerequisite |
| --- | --- |
| `ps_1` | approved `ps_0` architecture and compatibility-inventory contract, plus released Phase 40 compiler/tooling foundations |
| `ps_2` | released `ps_1` compiler/sysroot containing compile-time specialization, deterministic const evaluation, `ConstSpecializationOutcome`/`ConstPackageIssue`, registry-owned `SIFR-META-*` and `SIFR-INT-0009`, field required/default metadata, recursive nominal shape identity, structural shape inspection, lossless microsecond/timezone temporal value types, and `frozenset[T]` |
| `ps_3` | released `ps_2` compiler/sysroot containing the merged structural Rust bridge call contract, the already-passing stdlib `opaque_resource_core`, plus completed certification item `certification_pkg_resource_core` with passing `opaque_resource_package_core`, `callbacks_call_scoped`, `panic_boundary_wrapper_emission`, typed construction/projection, callback adapters, and native-backed compiled `re.Pattern` |
| `ps_4` and later | released Sifr compiler/sysroot containing the certified `ps_1` through `ps_3` contracts |

The certification work is tracked by
[`rust-interop-runtime-ecosystem-certification.md`](../archive/rust-interop-runtime-ecosystem-certification.md).
Callback invocation, cleanup, and panic mapping are blocking prerequisites, not
assumed capabilities. No Pydantic-Sifr milestone privately implements or
bypasses an uncertified bridge row.

## Ordered Milestones

Each milestone follows the project workflow:

1. define the complete milestone checklist,
2. implement and validate locally,
3. open its PR,
4. review to satisfaction and merge,
5. release a compiler/package version when the next repository depends on it,
6. update this issue and durable documentation, and
7. only then begin the next milestone.

### milestone_ps_0: Architecture Lock and Compatibility Inventory

- Approve this architecture.
- Freeze the researched upstream revisions.
- Approve the pinned module and selector baseline in this document.
- Define the executable `upstream_manifest.toml` schema and total-set audit
  that expand the API and in-tree Core test roots at the one Pydantic pin into individually classified
  files, collected nodes, and parameters.
- Approve the upstream pin-update procedure.
- Approve the pin-derived 53-kind Core Schema and four-kind field disposition
  table plus its exact-set generated-manifest rule.
- Classify Python-only and portable behavior.
- Define initial compatibility, error-code and provenance tables.
- Define compiler/package/core version relationships.
- Add no production implementation.

Exit gate: independent architecture review finds no unresolved ownership,
semantic-authority, bridge, safety, or sequencing ambiguity; every required
feature family with a meaningful Pydantic oracle has pinned selector anchors,
Sifr-native families such as fixed-width integer overflow have explicit native
contracts, and the design makes an omitted upstream file or collected node
mechanically detectable.

### milestone_ps_1: Compile-Time Shape and Metadata

- Implement the prerequisite compile-time specialization and deterministic
  const-evaluation subsystems.
- Complete field required/default metadata.
- Implement general compile-time structural shape inspection.
- Implement typed declaration metadata.
- Implement `ConstSpecializationOutcome[T]` and bounded
  `ConstPackageIssue`, activating general built-in `SIFR-META-0001`,
  `SIFR-META-0002`, and `SIFR-META-0003` with CLI/LSP parity.
- Implement the package-neutral `JsonIntegerBoundaryDescriptor` verifier and
  activate reserved `SIFR-INT-0009` with registry tests, generated error page,
  and the corresponding `integer_model.md` status update.
- Repair the diagnostics code-coverage guard to validate the canonical `.mdx`
  error pages before adding the new active codes; the ps_1 gate must not inherit
  or mask a pre-existing red diagnostics surface.
- Cover fields, defaults, generics, unions, enums, newtypes/refinements and
  recursive identity.
- Extend the general stdlib with lossless microsecond and timezone-aware
  temporal value types and immutable `frozenset[T]`.
- Add a compiler conformance fixture that is not Pydantic-specific.
- Document the durable general compiler contract.

Exit gate: an external fixture package derives a deterministic static
description of representative types without compiler-known package names,
emits identical fatal/warning structured diagnostics in `check`, build, and
editor analysis, and proves invalid issue declarations are rejected. A second
non-Pydantic fixture proves missing/unsafe integer-boundary descriptors emit
registry-owned `SIFR-INT-0009`.

### milestone_ps_2: Construction, Projection and Typed Callbacks

- Specify and merge the monomorphized structural Rust bridge call contract
  into `internal_docs/rust_interop_architecture.md` before implementation.
- Consume the already-passing stdlib `opaque_resource_core`. This milestone
  creates general package-resource support but no Pydantic resource; after its
  release, the certification issue's sequential
  `certification_pkg_resource_core` item must create and pass
  `opaque_resource_package_core` before `ps_3` begins. Service-specific
  `opaque_resource_matrix` evidence remains out of scope.
- Block on the certification issue's passing `callbacks_call_scoped`, including
  callback-invocation panic mapping, and `panic_boundary_wrapper_emission`
  rows; this phase does not privately take their ownership.
- Implement the accepted structural Rust bridge contract.
- Atomically remove `[rust] bridge-version` from the manifest schema, every
  in-repository bridge manifest and fixture, managed projections, archive
  expectations, cache records, diagnostics, and generated-build assertions.
  Remove the Rust-interop fixture matrix's top-level `bridge_version` marker,
  the `check_fixture_matrix.py` and `_scenario_checks.py` assertions that
  require it, `_scenario_registry.py`'s literal token,
  `_matrix_inventory.py`'s required-fixture entry, and
  `runner/bridge_check.py`'s version parameter/default. Remove the
  `rust_interop_plan.rs` module/cache fields, package-graph digest field, and
  sysroot's synthesized `Some(1)`. Delete the complete `bridge_version_mismatch`
  fixture/scenario and its matrix, tier, and stable-claim entries rather than
  retaining legacy acceptance evidence.
- Delete the `bridge-version = 1` subsection and rewrite every remaining
  version-keyed statement in `internal_docs/**`, `docs/**`, active issues,
  `plans/phases/**`, and the roadmap. This explicitly includes
  `docs/packages/manifest.mdx`, `docs/rust-interop.mdx`, the Blake3 and Reqwest
  interop guides, and
  `internal_docs/sifr_sysroot_and_stdlib_architecture.md`. Dated reviews, issue
  archives, and frozen release-candidate evidence remain immutable history.
- Reject the removed field through an explicit diagnostic rather than relying
  on the manifest parser's unknown-key behavior, and provide no compatibility
  path, rewrite, shim, or fallback. Replace the current
  `bridge_version_mismatch` evidence with passing
  `bridge_version_field_removal` evidence in the same implementation PR. Bind
  its positive side to a driver contract test and prove the repo-wide cutover
  through the now-unversioned package/scenario examples.
- Promote `structural_bridge_calls` from future-owned to supported-through-bridge
  only when both directions pass; remove it from the stable-support runtime
  deferrals and update the public stable-claims documentation atomically.
- Add the compiler-owned `sifr.meta.Structural` marker, recognize
  `@rust.structural` as the sole bare Rust marker, and diagnose marker arguments,
  missing targets, duplicate markers, and invalid generic placement through
  `SIFR-RUST-CONFIG-0001` / `SIFR-RUST-TYPE-*` with targeted tests.
- Implement safe structural `Construct[T]`.
- Implement allocation-free structural projection/visitation.
- Implement typed callback adapter generation.
- Upgrade stdlib `re.Pattern` to use the general opaque-resource substrate,
  compiling once while preserving readable source and flags. Because Sifr has
  no overloads, retain both public names but correct both signatures
  atomically: `compile(pattern) -> Result[Pattern, RegexError]` and
  `compile_flags(pattern, flags) -> Result[Pattern, RegexError]`. Update all
  stdlib/compiler call sites and docs in the same pre-stable contract PR;
  invalid patterns fail at construction and no infallible compatibility path
  remains.
- Prove ownership, move, borrow, error and panic behavior.
- Extend non-Pydantic compiler conformance fixtures.

Exit gate: a fixture package round-trips nested generic/recursive values through
a native opaque resource without dynamic reflection, layout assumptions, or
untyped callbacks.

### milestone_ps_3: Static Program and Native Bridge Contract

- Implement deterministic static schema-program emission support.
- Implement sealed arena/document opaque resources and compact node indices.
- Add generic signature probes, installed/source parity, cleanup, and cache-key
  contracts while consuming the already-passing unversioned-manifest contract.
- Prove exact integer, bytes, collection and error crossings.
- Update Rust interop architecture and verification.

Implementation checklist:

- [x] Give each retained const-specialization result one deterministic program
  identity. Include its owner, package function, canonical value, structural
  contract, and declaration metadata in that identity.
- [x] Emit immutable program bytes and a sealed typed program envelope during
  build. Retain the same identity during check and editor analysis.
- [x] Include the program identity in generated-project cache keys. Prove that
  relevant program inputs invalidate the key and unrelated declarations do not.
- [x] Add reusable sealed document and validated-arena runtime types. Use
  checked compact node indices and one move-only structural source.
- [x] Add one synthetic external package that consumes the unversioned Rust
  manifest contract. It must use a generic structural signature probe and one
  monomorphized executor call.
- [x] Prove exact integers, fixed integers, bytes, lists, mappings, records,
  moved scalar payloads, typed errors, corrupt envelopes, and invalid indices.
- [x] Prove source and installed package parity, deterministic generated output,
  cache-key behavior, and cleanup after success and failure.
- [x] Register passing positive and negative Rust-interop evidence. Update the
  compatibility matrix, fixture matrix, tiers, and durable architecture.
- [x] Run targeted tests, open the milestone pull request, and obtain an exact
  candidate `SATISFIED` agent review before the create-PR and merge gates.

Exit gate: the merged and certified structural Rust bridge contract lets a
synthetic schema executor return a validated arena, construct a typed Sifr
value, and pull a structural view for output through one monomorphized call.

### milestone_ps_4: Companion Repository and Core Foundation

- [x] Require the released Sifr compiler/sysroot containing certified `ps_1`
  through `ps_3` contracts.
- [x] Create the standalone public GitHub repository
  [`sifr-lang/pydantic-sifr`](https://github.com/sifr-lang/pydantic-sifr) under
  the `sifr-lang` organization.
- [x] Establish the external Sifr package and Rust backend layouts there.
- [x] Track, review, merge, and release all package/core implementation from that
  repository from this milestone onward.
- [x] Materialize the total-set `upstream_manifest.toml` before core
  implementation; prove exact equality with both test roots at the sole
  Pydantic pin and explicitly exclude the historical standalone Core checkout.
- [x] Generate `tests/provenance/core_schema_kinds.toml` from the pinned
  `CoreSchemaType`/`CoreSchemaFieldType` literals and prove exact equality with
  the accepted disposition table before defining format version 1.
- [x] Define Core Schema/program format version 1.
- [x] Implement that canonicalizer/verifier once as deterministic Sifr package
  code and emit sealed `VerifiedSchemaProgram[T]` static data in every
  specializing frontend mode.
- [x] Define the built-in/custom error-code registry and verify compositional
  `ErrorOverride` declarations.
- [x] Add error, input, arena and plan foundations.
- [x] Integrate Python-free `jiter`.
- [x] Establish licenses, provenance, fuzzing and benchmark harnesses.

Exit gate: `core/schema_contract` and `core/json_foundation` pass; malformed
schemas and malformed JSON return stable typed errors with zero panics under
unit, property and fuzz tests; the upstream ledger has no missing path/node or
unclassified entry; and `core_schema_kinds.toml` is exact-set-equal to all
pinned Core Schema and field kinds with one accepted primary owner and evidence
set/disposition audit per row.

### milestone_ps_5: Scalar and Collection Validation

- [x] Implement scalar schema nodes and strict/lax conversion.
- [x] Implement exact/fixed integers, floats, decimals, exact rational fractions,
  complex values, strings and bytes.
- [x] Integrate temporal and focused scalar libraries, including the Core Schema
  compiled-pattern value node over stdlib `re.Pattern`.
- [x] Implement numeric/decimal, string-normalization, pattern, length, and
  call-scoped clock-relative temporal constraints with the specified ordering.
- [x] Implement lists, tuples, mappings, sets and frozen sets.
- [x] Implement lazy `ValidatedIterator[T]` with fallible `next`, stable deferred
  error indices, and length/resource limits; it is not silently collected.
- [x] Implement the embedded-JSON decoder after manifest adaptation supplies an
  explicit statically known child schema.
- [x] Implement native, JSON, and strings input profiles over one validation
  engine.
- [x] Port the corresponding neutral Pydantic Core corpus.

Exit gate: `core/json_values`, `validators/numeric`,
`validators/complex`, `validators/text_bytes`, `validators/temporal`, `validators/collections`,
`validators/generator`, `validators/embedded_json`, `validators/none`,
`validators/url`, `validators/uuid`,
`core/fixed_integer`, `core/pattern_value`, `core/string_pipeline_order`,
`core/decimal_digit_counting`, `core/fraction`, `core/strings_profile`, and
`core/validation_errors` pass; all intentional differences are recorded,
`JsonLimitError` covers integer-digit exhaustion before allocation, and all
resource limits are enforced.

### milestone_ps_6: Models, Fields, Defaults and Aliases

- [x] Implement model/record schemas.
- [x] Implement required/defaulted/nullable distinctions.
- [x] Implement field metadata, aliases and alias paths.
- [x] Implement extra-field policies and ephemeral validated-field-count tracking.
- [x] Implement typed construction into ordinary Sifr classes.
- [x] Expose the first complete `BaseModel` validation API, including JSON,
  structural, and strings-profile entry points.

Exit gate: nested models validate JSON and native structural inputs into typed
Sifr values with aggregate stable errors and no third arena-to-model bridge
tree; `core/json_models`, `validators/model_fields`,
`core/model_error_locations`, `core/strings_profile_models`,
`validators/typed_dict`, `validators/dataclasses`, `validators/defaults`, `validators/nullable`,
`api/base_model`, `api/aliases`, `api/config_fields`, and `api/constraints`
pass.

### milestone_ps_7: Unions, Recursion and Custom Validation

- Implement literals, enums, ordinary unions, field/path-discriminated tagged
  unions, and typed-callback-discriminated tagged unions.
- Implement deterministic `smart` and `left_to_right` ordinary-union modes,
  labelled aggregate errors, nested ranking-state bubbling, `Omit` handling,
  and the declared auto-collapse policy.
- Execute compositional error overrides, including union and tagged-union
  custom errors, through the single aggregate-error path.
- Implement definitions, references and recursion guards.
- Implement strict/lax, JSON/structural-input, and flattened typed-chain
  control composition.
- Implement before/after/wrap/plain typed validators.
- Implement field/model validator ordering and caller-owned typed validation
  context, including immutable and mutable borrow modes.
- Port the corresponding upstream behavior corpus.

Exit gate: ambiguous, recursive and callback-heavy cases have deterministic
success/error behavior, bounded execution and complete ownership coverage;
`validators/literal`, `validators/enum`, `validators/unions`,
`validators/control_composition`,
`validators/tagged_unions`, `validators/nullable_union`,
`validators/definitions_recursion`, `validators/callbacks_context`,
`core/recursion_limit`, `core/smart_union_ranking`, `api/validators`,
`api/discriminated_unions`, and `api/generics_recursion` pass.

### milestone_ps_8: Serialization

- [x] Implement serializer plans over structural projections.
- [x] Implement structural and streaming JSON outputs.
- [x] Implement aliases, typed recursive include/exclude selections, and
  default/none policies.
- [ ] Implement custom field/model serializers and computed fields. Blocked by
  companion issue #14 pending package-neutral handler-bearing method slots.
- [ ] Implement caller-owned typed serialization context forwarding. Blocked
  by companion issue #14 with the serializer callback boundary.
- [x] Implement Sifr's selected integer JSON profile through
  `sifr_runtime::json`, including typed range errors.
- [ ] Preserve temporal output policies. Blocked by companion issue #15 until
  typed temporal current-value projections exist at the serialization boundary.
- [ ] Port the remaining serialization tests after their typed public value
  surfaces land. The delivered corpus and benchmarks merged in companion PR
  #18; the remaining anchored families are tracked by companion issue #17.

Exit gate: mutated typed models serialize from their current values, not a
retained validation arena, and no full generic output tree is required for
JSON; every `serializers/*` fixture family named in the baseline and
`core/selection_precedence` and `api/serialization` pass.

### milestone_ps_9: TypeAdapter and JSON Schema

- [x] Implement reusable `TypeAdapter[T]`.
- [x] Implement native, JSON, and strings-profile validation plus serialization
  modes.
- [x] Generate JSON Schema from the same Core Schema.
- [x] Reflect the selected Sifr integer JSON profile and static range in every
  integer schema, failing closed with `SIFR-INT-0009` when ambiguous.
- [x] Before the external package release, merge a coordinated `sifr-lang/sifr`
  documentation/verification PR updating
  `verification/areas/core_language/data/integer_model/serialization_boundary_rules.md`
  with the implemented descriptor consumer, generated-client warning
  ownership, and exact bounded JSON Schema snapshots; update
  `internal_docs/integer_model.md` to name
  `x-sifr-integer-profile: exact` as the implemented exact-profile schema
  marker. `ps_1` already owns the diagnostic page and Reserved-to-Active
  diagnostic status change.
- [x] Support definitions, recursion, aliases, constraints and mode-specific
  representations.
- [x] Complete public `Fraction` and `Complex` adapter/schema representations.
- [x] Add deterministic schema snapshots and dialect conformance.

Exit gate: validation, serialization and description agree for every supported
schema node, with no Schemars or alternate metadata authority;
the coordinated Sifr boundary-artifact PR is merged and its snapshots pass;
`api/type_adapter`, `api/specialized_numeric`, and `api/json_schema` pass.

### milestone_ps_10: Full Pydantic-Familiar Surface

- Complete the selected `BaseModel`, `Field`, configuration, validator,
  serializer, computed-field and adapter APIs. The functional model,
  field/configuration metadata, and adapter surfaces are delivered. The
  Pydantic-familiar validator, serializer, and computed-field facade remains
  blocked on package-neutral handler-bearing method-slot dispatch, tracked by
  companion issues
  [#10](https://github.com/sifr-lang/pydantic-sifr/issues/10) and
  [#14](https://github.com/sifr-lang/pydantic-sifr/issues/14). This row is
  skipped under the phase blocker rule; no package-specific dispatch,
  fallback, or second engine is permitted.
- Complete the selected root-model, specialized network type, field-metadata,
  compiled-pattern field/API, and public error surfaces. Root adaptation,
  field metadata, URL/pattern validation, and structured errors are delivered
  in the shared core. The Sifr-visible nominal network and compiled-pattern
  values remain blocked on package-neutral structural mappings, tracked by
  companion [issue #27](https://github.com/sifr-lang/pydantic-sifr/issues/27).
  This row is skipped under the phase blocker rule; Rust-only wrappers, erased
  string substitutes, and package-specific compiler branches are not accepted.
- [x] Publish the API/behavior compatibility matrix.
- [x] Add migration documentation for Pydantic users.
- [x] Prove ordinary Sifr classes and the familiar facade use the same engine.
- [x] Remove any temporary internal API exposed during construction.

Exit gate: the documented end-state public API is complete; no public fallback,
temporary schema form or second validation path remains; `api/networks`,
`core/multi_host_url_serialization`, `api/pattern`, `api/root_model` and
`api/field_metadata` pass.

### milestone_ps_11: Certification and Release

- [x] Re-audit the already-complete manifest against its pinned revisions and the
  documented update-pin procedure; no compatibility coverage is deferred to
  this milestone.
- [x] Run differential validation against the pinned oracle.
- [x] Complete fuzz, property, adversarial resource and panic testing.
- [x] Publish parse/validate/construct/serialize benchmarks.
- [x] Certify supported compiler/core/package version combinations.
- [x] Add end-to-end demos and package documentation.
- [x] Add and snapshot-test the canonical
  `demos/pydantic_sifr_demo.sifr` in the external `pydantic-sifr` repository.
- [x] Perform independent whole-architecture and implementation review.

Exit gate: all acceptance criteria pass using released Sifr and
`pydantic-sifr` artifacts without access to the source checkout, Python, or the
upstream repositories, and the canonical demo runs from an installed package
without a Sifr compiler source checkout.

Installed-artifact execution is skipped under the phase blocker rule. The
verified installed Sifr sysroot is not self-contained for structural bridges;
[issue #3233](https://github.com/sifr-lang/sifr/issues/3233) owns the missing
package-neutral `sifr_structural_identity` release asset. Package fetch and
formatting passed before the installed compiler rejected that missing asset.

## Phase Acceptance Criteria

Phase completion requires the design invariants in
[`native_pydantic_sifr_architecture.md`](../../../internal_docs/native_pydantic_sifr_architecture.md)
and the delivery criteria below.

### Delivery

- Milestones are delivered sequentially through reviewed PRs.
- Corresponding durable architecture and status documents are updated after
  every milestone.
- The canonical `demos/pydantic_sifr_demo.sifr` lives in
  `sifr-lang/pydantic-sifr`, is covered by that repository's local validation,
  and builds and runs against installed/released artifacts.
- `sifr-lang/sifr` contains only package-neutral compiler conformance demos and
  fixtures; it contains no Pydantic-Sifr product demo.
- Authoritative local validation passes in both repositories.
- Independent final review confirms the implementation matches this end state
  without fallback paths or split semantic authority.

## Exit Gate

This ad hoc phase is complete only when a Sifr user can install the released
`pydantic-sifr` package and use a Pydantic-familiar, fully native API to:

1. derive a schema for a typed model,
2. validate hostile JSON into that model,
3. receive deterministic aggregate `Result` errors,
4. run typed custom validators,
5. mutate the resulting native model,
6. serialize its current state through a selected profile,
7. generate matching JSON Schema, and
8. do all of the above without Python, a Pydantic Core fork, compiler package
   special cases, duplicated schema authorities, or user-triggerable panics.
