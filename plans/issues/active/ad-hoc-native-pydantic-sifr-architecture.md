# Ad Hoc Phase: Native Pydantic-Sifr Architecture

## Status

Architecture proper was approved on draft PR
[#3014](https://github.com/sifr-lang/sifr/pull/3014). Opus 5 pass 17 returned
`SATISFIED` and approved `milestone_ps_0`. The architecture, conformance
inventory, repository boundary, and demo ownership are approved.
`milestone_ps_1` and `milestone_ps_2` are implemented and merged. The required
`certification_pkg_resource_core` item is complete through merged
[PR #3123](https://github.com/sifr-lang/sifr/pull/3123). `milestone_ps_3` is
implemented and merged through [PR #3138](https://github.com/sifr-lang/sifr/pull/3138).
`milestone_ps_4` through `milestone_ps_6` are implemented and merged in the
companion repository. The package-neutral method-slot prerequisite for
`milestone_ps_7` is implemented and merged. The blocked callback rows are
tracked separately, and `milestone_ps_8` is active in the companion repository.

Review artifacts:

- [`native-pydantic-sifr-architecture-opus5-review-pass-1.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-1.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-2.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-2.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-3.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-3.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-4.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-4.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-5.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-5.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-6.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-6.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-7.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-7.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-8.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-8.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-9.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-9.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-10.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-10.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-11.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-11.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-12.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-12.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-13.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-13.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-14.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-14.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-15.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-15.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-16.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-16.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-17.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-17.md)
- [`native-pydantic-sifr-ps2-claude-opus-review-pass-1.md`](../../reviews/active/native-pydantic-sifr-ps2-claude-opus-review-pass-1.md)
- [`native-pydantic-sifr-ps2-claude-opus-review-pass-2.md`](../../reviews/active/native-pydantic-sifr-ps2-claude-opus-review-pass-2.md)
- [`native-pydantic-sifr-ps2-claude-opus-review-pass-3.md`](../../reviews/active/native-pydantic-sifr-ps2-claude-opus-review-pass-3.md)
- [`native-pydantic-sifr-ps3-claude-opus-review-pass-1.md`](../../reviews/active/native-pydantic-sifr-ps3-claude-opus-review-pass-1.md)
- [`native-pydantic-sifr-ps3-claude-opus-review-pass-2.md`](../../reviews/active/native-pydantic-sifr-ps3-claude-opus-review-pass-2.md)
- [`native-pydantic-sifr-ps3-claude-opus-review-pass-3.md`](../../reviews/active/native-pydantic-sifr-ps3-claude-opus-review-pass-3.md)

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
- Opus review of candidate `5b1601ded66556fe04b9674916153726b341b2c6`
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
- Opus reviewed eight published remediation candidates. The final exact-SHA
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
  guardrails passed. Opus review round 3 returned `SATISFIED` with no actionable
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
- Opus remediation passes closed recursive identity/cache and panic-contract
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
- The exact-SHA Opus review returned `SATISFIED` with no blocking findings.
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
- Three recorded Opus review passes found and closed unsupported-owner emission,
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
- The final Opus remediation review returned `SATISFIED` with no in-scope
  blocker. The canonical create-PR and authoritative merge gates passed. The
  merge gate ran 4,096 release property cases, compiled both fuzz targets, and
  left only a stale tracked fuzz lockfile from removed production dependencies.
- The exact one-file lock correction merged in companion-repository
  [PR #2](https://github.com/sifr-lang/pydantic-sifr/pull/2) at merge commit
  `c8200c9ae67e3b504674ea105836b4894413507b`. Its reviewed and gated candidate
  was `7185f538a57eb54a74f87f9d4d7ae2e8fcbfb387`; Opus returned
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
  scalar review rounds closed all blocking findings. The whole-milestone Opus
  review of the final candidate returned `SATISFIED` with no blockers. Its
  response remains outside the Git tree at
  `/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr-claude.CB0DN1/response.md`.
- The canonical create-PR gate and the single authoritative merge gate passed
  on the same candidate. The merge gate included release-mode tests and 1,000
  bounded runs for each scalar, collection, and special validation fuzz target.
- Opus follow-ups are non-blocking documentation and coverage work: expand the
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
- The final Opus implementation and evidence review returned `SATISFIED` with
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
- The exact-candidate Opus review returned `SATISFIED` with no blocker.
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
- The exact-candidate Opus review returned `SATISFIED` with no blocker.
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
- The exact-candidate Opus review returned `SATISFIED` with no blocking
  finding. The response remains outside the Git tree at
  `/var/folders/lq/l19_y_rn76b8vprfvdjn9zch0000gn/T/sifr-claude.fvHK1U/response.md`.
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
- Ten Opus review rounds closed duplicate imports and test-crate root mapping.
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
  Opus accepted the combined evidence without a second merge gate.
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
- Exact-SHA Opus implementation review and validation adjudication returned
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
- Exact-SHA Opus implementation and remediation reviews returned `SATISFIED`
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
- Opus accepted the combined validation evidence without a second merge gate.
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
- Exact-SHA Opus implementation review returned `SATISFIED` with no blocker.
  Its response SHA-256 is
  `ad130714c6981f6243de4735f5ccefad1402587db3e8e7b4c95283e576088e5c`.
- The canonical create-PR gate exited zero. Its receipt SHA-256 is
  `d3e4263e11d18d89d2f5703d0373ec008fa542ec0254922e7f9cdb030fa73721`.
- The single merge gate passed every executed functional lane. It reproduced
  the stale frontend helper problem from issue #3161.
- A fresh exact-candidate helper measured 925,442,120 median instructions
  against the unchanged 936,811,698 limit. Its accepted receipt SHA-256 is
  `82bec880219baf54c7c3408e45651fee03428db4f813a88ed2e45ed385b82710`.
- Opus accepted the combined validation evidence. Its adjudication response
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
- The final whole-milestone Opus review returned `SATISFIED` with no blockers.
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
- The final whole-candidate Opus review returned `SATISFIED` with no blockers.
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
- Three Opus review rounds corrected fixture identities, shape hashes, and
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
- The final exact-SHA Opus review returned `SATISFIED` with no blocking findings.
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
  union ordering. The final Opus review returned `SATISFIED` with no blockers.
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
- The exact-SHA Opus review returned `SATISFIED` with no blockers. Its response
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
- The first exact-SHA Opus review found one flattened `EmbeddedJson` layout
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
- A full Opus review then found the strict JSON mapping handoff defect. Its
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
- The exact-SHA Opus review found one documentation omission in the pending API
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
- A separate exact-current-state Opus audit confirmed that the fixed-arity
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
- Two exact-SHA Opus reviews found generic-substitution/operator coverage and
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
- The final exact-SHA Opus review returned `SATISFIED` with no blocking findings.
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
- The exact-SHA Opus review returned `SATISFIED` with no blocking findings. Its
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
- The exact-SHA Opus review returned `SATISFIED` with no blocking findings. Its
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
- The remediation exact-SHA Opus review returned `SATISFIED` with no blocking
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
- Exact-SHA Opus review returned `SATISFIED` with no blocking findings. Its
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
- Exact-SHA Opus review returned `SATISFIED` with no blocking findings. Its
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
- Exact-SHA Opus review returned `SATISFIED` with no blocking findings. Its
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
- Exact-SHA Opus review returned `SATISFIED` with no blocking findings. Its
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
- Exact-SHA Opus review returned `SATISFIED` with no blocking findings. Its
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
- Exact-SHA Opus review returned `SATISFIED` with no blocking findings. Its
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
- Exact-SHA Opus review returned `SATISFIED` with no blocking findings. Its
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
  owning issue links for blocked rows. Exact-SHA Opus review returned
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
- Exact-SHA Opus review returned `SATISFIED` with no blocking findings. Its
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
  workload, and reproduction command. Opus was SATISFIED with no blockers;
  response SHA-256 is
  `30d76666e7682eea122d38997d90f6c48f7b9aaa1350e9e8501436f7c9887694`.
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
- Exact-SHA Opus review returned `SATISFIED`; its response SHA-256 is
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
- `milestone_ps_9` is now active in the companion repository.
- Deferred follow-up work: align registry representative-fixture paths with diagnostic
  baselines; align the pre-existing lowering and codegen structural-eligibility
  predicates for fixed-width platform integers, metadata, and imported classes;
  audit pre-epoch fractional timestamp reconstruction; disambiguate imported
  structural metadata/default lookup from colliding local names; accept large negative
  const bounds in integer-boundary decorators; and keep shared floor-arithmetic semantics
  from drifting between frontend const evaluation and runtime execution.

This document is the single planning source of truth for:

- the general compiler capabilities required in `sifr-lang/sifr`,
- the separate `sifr-lang/pydantic-sifr` repository,
- the Sifr `pydantic` package,
- the native `pydantic_sifr_core` Rust crate,
- the Pydantic/Pydantic Core compatibility corpus, and
- the ordered delivery and acceptance gates.

After implementation stabilizes, durable compiler contracts belong in the
corresponding `internal_docs` architecture documents in `sifr-lang/sifr`, while
package/core contracts belong in the `pydantic-sifr` repository. This issue
remains the phase history and decision record.

## Objective

Provide a complete, native, Pydantic-like data contract API for Sifr with:

- statically derived schemas,
- validation and coercion from untrusted inputs,
- typed model construction,
- structured aggregate errors,
- serialization profiles,
- custom validators and serializers,
- type adapters,
- JSON Schema generation,
- Pydantic-familiar APIs where they fit Sifr,
- native performance with no Python runtime, and
- behavior grounded in the battle-tested Pydantic and Pydantic Core corpus.

The end state must preserve Sifr's guarantees:

- fallibility is expressed through `Result`, not exceptions,
- user-triggerable input cannot panic the process,
- exact Sifr integers are not silently narrowed,
- ownership and callback effects remain statically visible,
- package behavior does not require compiler special cases, and
- invalid static schemas fail during checking/build rather than on the first
  production request.

## Problem

Sifr can represent typed classes and compile them to native Rust, but a full
data-contract system requires more than JSON parsing:

- structural schema derivation,
- field metadata and defaults,
- strict and lax conversion policies,
- recursive validation,
- alias selection,
- union ranking and tagged unions,
- complete error locations and aggregation,
- custom validator execution,
- profile-aware serialization,
- schema description, and
- a safe native boundary capable of returning arbitrary validated Sifr types.

Implementing these behaviors directly in the compiler would make validation a
language special case and would force package policy into the compiler release
cycle. Implementing them all as ordinary Sifr code over copied JSON values
would move performance-sensitive recursive execution across the Rust bridge and
would fail to reuse the strongest part of Pydantic's architecture.

Directly depending on or lightly wrapping `pydantic-core` also does not solve
the problem. At the researched revision, its central input, validation,
serialization, error, and result representations are shaped around PyO3 and
Python objects. Removing that coupling would be a permanent high-drift fork,
not a small native adapter.

The required solution is a Sifr-native form of Pydantic's proven frontend/core
split, with a smaller schema designed around static Sifr types rather than
Python's dynamic object model.

## Research Baseline

The architecture was derived from complete local checkouts outside the Sifr
codebase:

| Repository | Researched revision | Role |
| --- | --- | --- |
| `pydantic/pydantic` | `f59e929c999e8b2efc7b12fd0bc1685c1a186be3` | Sole compatibility pin for Pydantic and its in-tree Pydantic Core 2.47.0 |
| `pydantic/pydantic-core` | `383eb95a19433754c0cecf7025b50c26b6d97a36` | Historical architecture/reuse research reference at 2.41.5; not a parity oracle |

Both upstream repositories are MIT licensed. Copied implementation fragments or
test data must retain the required notice and provenance.

The Pydantic checkout's tracked `pydantic-core/` component is the engine version
required by that same Pydantic revision and is therefore the sole engine
semantic oracle. The older standalone checkout informed architectural reuse
decisions only; its tests never enter the compatibility ledger and a behavior
present only there cannot define parity.

Existing Sifr contracts: [`rust_interop_architecture.md`](../../../internal_docs/rust_interop_architecture.md),
[`sifr_sysroot_and_stdlib_architecture.md`](../../../internal_docs/sifr_sysroot_and_stdlib_architecture.md),
[`integer_model.md`](../../../internal_docs/integer_model.md), and
[`architecture.md`](../../../internal_docs/architecture.md).

The compiler substrate also depends on the core rows tracked by
[`rust-interop-runtime-ecosystem-certification.md`](../archive/rust-interop-runtime-ecosystem-certification.md).

### Cross-document authority

This accepted ad hoc phase supersedes the draft implementation scope in
`plans/phases/41_typed_data_model_and_validation.md`; that file becomes a
redirect/history note, and the roadmap's Phase 41 row points here. Phase 42's
typed-extractor dependency is the released `pydantic-sifr` public model/error
contract through certified release `milestone_ps_11`, not a separate
in-compiler validator. Phase 42 remains blocked if that external release does
not occur; it may not add a fallback.

This design does not supersede `internal_docs/integer_model.md` or its locked
serialization-boundary artifact: all integer JSON, schema, and diagnostic
behavior defers to them. `ps_1` owns the accepted const-specialization
diagnostic contract, activates `SIFR-INT-0009`, publishes its error page, and
changes `integer_model.md` from Reserved to Active. `ps_9` owns the later
consumer/schema integration update to the locked boundary artifact before the
external package releases; it does not redefine the code. The
former Phase 41 `Serialize`/`Deserialize` and stdlib `dumps`/`loads` proposal is
intentionally subsumed by the one `TypeAdapter[T]`/`BaseModel` Core Schema
path, not retained as a second compiler or stdlib serialization authority.
`sifr-lang/sifr` deliberately keeps only its general `JsonValue` JSON API;
typed model JSON is owned exclusively by the external package. Phase 42 waits
for `milestone_ps_11` certification and remains blocked rather than adding a
fallback if that release is unavailable.
Rust bridge certification rows remain owned by
`rust-interop-runtime-ecosystem-certification.md`; this phase may consume only
passing rows or an explicitly transferred narrow row recorded in that issue
and the compatibility matrix.

## End-State Decisions

1. `pydantic-sifr` is an external Sifr package in a standalone public GitHub
   repository owned by the `sifr-lang` organization, with the planned location
   [`sifr-lang/pydantic-sifr`](https://github.com/sifr-lang/pydantic-sifr). It is
   not a directory, workspace member, vendored subtree, or submodule of
   `sifr-lang/sifr`.
2. The Sifr package and `pydantic_sifr_core` are separate components in that
   repository and normally release together.
3. The design reuses Pydantic's high-level architecture:
   public package -> declarative Core Schema -> compiled native execution plan.
4. The design does not fork, embed, link, or require Pydantic Core in
   production.
5. The Sifr compiler gains only general structural metaprogramming,
   specialization, callback, and native-package capabilities.
6. The compiler contains no Pydantic, validation, field, model, JSON, or schema
   special cases.
7. `pydantic-sifr` owns the public API, schema derivation, configuration, and
   typed Sifr integration.
8. `pydantic-sifr` owns schema canonicalization and semantic verification in
   deterministic compile-time Sifr code; `pydantic_sifr_core` owns execution,
   input adapters, aggregate errors, and performance-sensitive algorithms.
9. Derived static schemas become immutable schema programs during build.
   There is no runtime schema-compilation path.
10. Core Schema is the sole package authority for validation, serialization,
    and description, while embedding and obeying accepted language-wide
    contracts such as Sifr's locked integer JSON profiles. Serde, Schemars, and
    another validator are not parallel authorities.
11. The structural Rust bridge contract replaces the current versioned schema
    and adds one general, trait-bounded structural call contract. The
    implementation removes the `[rust] bridge-version` manifest field and all
    version-specific compiler/tooling paths; there is no compatibility mode,
    rewrite, shim, or fallback. The structural contract does not add
    Pydantic-specific bridge types or container exceptions.
12. Native decoding returns a validated value arena. The JSON parse tree and
    normalized arena are expected; no third copied bridge-object tree exists.
13. Compiler-generated structural traits materialize a validated source into
    the requested Sifr type and project typed Sifr values to native consumers.
14. `pydantic_sifr_core` invokes those traits through one monomorphized native
    call. It never imports package-generated bridge types.
15. `jiter`, without its Python feature, is the canonical JSON parser.
16. `speedate` is a temporal parsing mechanism where its behavior matches the
    selected Sifr contract; it is not a public temporal representation.
17. `sifr_runtime::json` supplies the authoritative integer-profile helpers;
    Serde and `serde_json` may provide other format/writer mechanisms but do
    not redefine validation, coercion, errors, or schemas.
18. Focused Rust crates are reused for regex, URL, UUID, Base64, IDNA, and
    arbitrary-precision numeric mechanisms.
19. Pydantic and Pydantic Core are development oracles and provenance sources,
    never dependencies of published artifacts.
20. Compatibility means equivalent behavior where Python and Sifr correspond,
    with every divergence documented.
21. The canonical Pydantic-Sifr demo is owned, tested, and released by the
    external `sifr-lang/pydantic-sifr` repository. No package-specific demo is
    added to `sifr-lang/sifr`.
22. Delivery is sequential: implement, validate, review, merge, and release a
    milestone before starting the next.

## Repository Ownership

This repository boundary is part of the architecture, not merely source-tree
organization. `sifr-lang/sifr` supplies and releases the general compiler,
sysroot, package, and native-interop capabilities first. The resulting
`pydantic` Sifr package is then developed, tested, reviewed, released, and
consumed as an external package from its own
[`sifr-lang/pydantic-sifr`](https://github.com/sifr-lang/pydantic-sifr) GitHub
repository.

After `milestone_ps_3`, production implementation work for the package and
native core is tracked and merged in that external repository. This planning
issue records cross-repository dependencies and links the resulting external
issues, pull requests, and releases; it does not move their source into the
Sifr compiler repository.

### `sifr-lang/sifr`

Owns only general language and package substrate:

- compile-time type shape inspection,
- compile-time declaration metadata,
- safe structural construction,
- safe structural projection/visitation,
- specialization of generic package code,
- typed package callback adapters,
- bounded const-specialization issues mapped to compiler-owned diagnostics,
- package-neutral integer-boundary descriptor verification,
- static data emission,
- Rust bridge support required by general native packages,
- package/compiler compatibility declarations, and
- compiler conformance fixtures.

The compiler must be able to explain these features without mentioning
Pydantic. Database mappers, RPC systems, command-line parsers, encoders,
decoders, and other packages must be able to consume the same substrate.

### External package repository: `sifr-lang/pydantic-sifr`

Owns:

- the public Sifr package,
- the native core crate,
- the versioned Core Schema contract,
- compatibility and differential tests,
- fuzz targets,
- benchmarks,
- upstream provenance,
- runnable package demos,
- package documentation, and
- releases.

Recommended layout:

```text
pydantic-sifr/
  Cargo.toml
  Cargo.lock
  sifr.toml
  src/
    __init__.sifr
    model.sifr
    fields.sifr
    adapters.sifr
    validators.sifr
    serializers.sifr
    errors.sifr
    json_schema.sifr
    bridges/
      mod.rs
      core.rs
  backend/
    pydantic_sifr_core/
      Cargo.toml
      src/
        lib.rs
        schema/
        input/
        validators/
        serializers/
        errors/
        arena/
  tests/
    native/
    compatibility/
    differential/
    fuzz/
    provenance/
      upstream_manifest.toml
      core_schema_kinds.toml
  benchmarks/
  demos/
    pydantic_sifr_demo.sifr
    README.md
  docs/
  LICENSE
```

The backend is a normal statically linked Rust package dependency under the
existing Rust interop architecture. Published artifacts do not contain or load
a `cdylib`, Python extension, CPython library, or runtime plugin.

## High-Level Architecture

```text
Sifr type T + package metadata
              |
              v
      pydantic-sifr frontend
              |
              v
      Sifr Core Schema graph
              |
 package const canonicalize/verify
              |
              v
    immutable Schema Program
              |
              v
      pydantic_sifr_core
       /              \
      /                \
 decode plan        serialize plan
    |                    |
external input      structural view of T
    |                    |
validated arena     format writer
    |
structural Construct[T]
    |
typed Sifr value T
```

The schema graph is the architectural boundary corresponding to Pydantic's
`CoreSchema`. The representation and node set are Sifr-owned and deliberately
exclude Python-only behavior.

## Compiler Substrate

### Compiler prerequisites

Milestones `ps_1` through `ps_3` create new compiler and sysroot capabilities;
they are not small extensions of the current generics, stdlib value types, or
Rust bridge implementation. Their gated prerequisites are:

- compile-time specialization of package generics for a concrete `T`,
- deterministic compile-time evaluation sufficient to derive and emit static
  data,
- `ConstSpecializationOutcome[T]` and bounded package issues for const specialization in
  `check`, build, tests, and editor analysis,
- a package-neutral `JsonIntegerBoundaryDescriptor` verifier,
- first-class field required/defaulted metadata rather than reconstruction from
  an `__init__` signature,
- exact recursive nominal identity, and
- general stdlib value types that losslessly support microsecond temporal
  precision, timezone-aware `time`, and immutable `frozenset[T]`,
- a native-backed compiled `re.Pattern` that preserves source and flags after
  the opaque-resource substrate exists, and
- the structural Rust bridge call contract and certified ecosystem-owned
  opaque-resource support described below.

C-like enums remain simple constants. In accordance with the accepted Sifr
decision in `internal_docs/architecture.md`, data-carrying variants use ordinary
unions of records. Core Schema tagged unions specialize that existing type
model; they do not require associated-data enums or create a second permanent
sum representation.

### Structural Rust bridge calls

The existing bridge-compatible value table remains closed. The structural contract
does not make tuple, set, arbitrary mapping, union payload, or specialized
scalar values directly cross the boundary as ad hoc bridge types.

Instead, `sifr_runtime` owns three stable, language-general traits. Native
producers implement `StructuralSource`; the compiler generates
`StructuralConstruct` and `StructuralProject` implementations for concrete Sifr
types:

```text
StructuralSource
    shape_identity() -> ShapeIdentity
    root() -> NodeId
    take/read nodes through a sealed stable interface

StructuralConstruct
    construct[S: StructuralSource](source: own S) -> Result[Self, ContractError]

StructuralProject
    project(self: &Self, visitor: StructuralVisitor) -> Result[None, VisitorError]
```

The names above are conceptual; the accepted structural bridge design fixes the
actual Rust/Sifr surface. The essential contract is:

- a native backend may call a generic function bounded by these compiler-owned
  traits,
- the call is monomorphized in the generated package crate for the concrete
  Sifr `T`,
- the backend crate depends only on the stable traits and its own stable opaque
  resources,
- package-local generated glue implements the traits for generated Sifr types,
- the backend crate never imports `crate::__sifr_bridge` types,
- construction consumes a sealed `StructuralSource` carrying a declared
  structural-shape identity,
- projection borrows the current typed value and emits a call-scoped visitor
  event stream,
  and
- the existing bridge rejects all unsupported ordinary direct crossings as
  before.

`StructuralSource` is language-neutral. Pydantic-Sifr's validated arena
implements it, but an RPC decoder or database row mapper can implement the same
trait for its own native resource and use the same construction path. Core
Schema identity is checked by Pydantic-Sifr before construction; it is not part
of the compiler trait contract.

Decoding uses one native generic call:

```text
pydantic_sifr_core::validate_and_construct[T: StructuralConstruct](...)
    -> Result[T, ValidationError]
```

The core owns an opaque `ValidatedArena` implementing `StructuralSource`;
package-local generated glue consumes its stable nodes while constructing `T`.
Strings, bytes, exact integers, and specialized scalar components move out
where ownership permits. Containers are constructed recursively inside the
monomorphized package crate rather than crossing the public bridge wholesale.

Serialization also uses one native generic call. The core serializer pulls
from `T: StructuralProject` through the call-scoped view and remains the sole
driver of alias, exclusion, representation, and writer policy. This avoids
per-field Sifr/Rust bridge calls and avoids a second generic output tree.

The structural bridge contract must specify:

- trait and opaque-resource ownership,
- generated implementation placement,
- lifetime and call-scoped view rules,
- generic signature probing and monomorphization,
- move-out and partial-failure cleanup,
- recursion and callback interaction,
- panic containment,
- cache/build identity, and
- installed/source package certification.

The contract is incomplete until it is merged into
`internal_docs/rust_interop_architecture.md` and its core certification rows
pass. Pydantic-Sifr cannot privately invent an alternate structural bridge.

### Structural shape

Package code must be able to inspect a statically known `T` during
specialization:

- primitive kind,
- exact nominal identity,
- type arguments,
- record/class fields in declaration order,
- field names and declared types,
- required versus defaulted fields,
- enum variants and package-declared scalar value metadata,
- tuple and collection elements,
- optional and union members,
- refined/newtype base type,
- recursive references, and
- package-defined declaration metadata.

The shape is compile-time information. It is not a public runtime reflection
object and does not make arbitrary type mutation possible.

### Declarative metadata

Packages need one general mechanism for typed, compile-time declaration
metadata. It must support metadata attached to:

- a type,
- a field,
- an enum variant,
- a function,
- a method, and
- a parameter.

Metadata values must be statically typed and compile-time evaluable. The
compiler preserves and exposes them to specializing package code; it does not
interpret `Field`, validator, serializer, or model configuration semantics.

This mechanism is the substrate for Pydantic-familiar `Field`,
`field_validator`, `model_validator`, `field_serializer`, `computed_field`,
and configuration declarations without hard-coding their names.

### Const-specialization diagnostics

A specializing package function returns
`ConstSpecializationOutcome[T]`, containing either a value plus zero or more
warnings, or one or more fatal `ConstPackageIssue` values and no value. These
are new frontend contracts; they neither reuse nor alter
`sifr_package::PackageDiagnostic` or the driver's existing `CompileResult`.
Those existing types remain confined to package-manager and driver
diagnostics.

`ConstPackageIssue` carries a package-qualified stable `reason_code`, static
package-template arguments, one primary source span supplied by the
specialization API, additional labels, and notes. Values must be
const-evaluable and bounded. Package argument names are checked against the
package's statically declared template and cannot use compiler/LSP-reserved
names such as `rule`.
The package reason is diagnostic context, not a new top-level compiler code.
The frontend maps a fatal issue to built-in `SIFR-META-0001`, a warning to
`SIFR-META-0002`, and a malformed declaration to `SIFR-META-0003`. These three
general metaprogramming codes are added to the closed Sifr diagnostic registry
in `ps_1`, so normal documentation URLs remain
`https://docs.sifr.sh/errors/<CODE>`. A warning may accompany a produced value
and does not make checking fail; a fatal issue cannot accompany a value. The
compiler, never the package, owns severity and top-level rendering.

The compiler diagnostic arguments remain closed: `SIFR-META-0001` and
`SIFR-META-0002` declare exactly `package` and `reason_code`;
`SIFR-META-0003` declares `package`, `reason_code`, and `declaration_problem`.
Package template arguments are rendered only into a bounded structured note
after static template validation; they are never forwarded as open arguments
to a registry entry. `SIFR-META-0002` is intentionally an unsuppressible
`hard` LSP warning because it arose during deterministic specialization, not a
lint rule; this classification is documented with the code in `ps_1`.

The frontend converts the outcome into the same structured CLI/LSP diagnostic
stream in `check`, build, tests, and editor analysis. It validates the package
namespace, reason code, spans, and template arguments and never executes a
package renderer or accepts arbitrary terminal text. A non-Pydantic fixture
package must prove fatal and warning emission, invalid-issue rejection,
source/installed parity, and identical CLI/LSP identity before
Pydantic-Sifr may depend on the channel.

### Structural construction

Specialized package code must be able to construct `T` from validated
components without:

- bypassing ownership checks,
- invoking user-visible validation a second time,
- creating an observably partially initialized value,
- using reflection at runtime, or
- cloning every field.

Construction succeeds only from a sealed `StructuralSource` whose declared
structural-shape identity matches `T`. Compiler-generated code moves owned
values where possible and rejects source/type mismatches as internal contract
errors. It has no knowledge of Core Schema or Pydantic.

### Structural projection

Specialized package code must expose an immutable typed value to native
serialization as a structural reader:

- record field enumeration,
- enum variant access,
- primitive borrowing,
- collection iteration,
- optional/union discrimination, and
- declaration-order field visitation.

Projection is pulled by the native consumer during one monomorphized call and
does not first allocate a second generic tree. The facility is general
structural visitation; Pydantic-specific alias and serialization policy remains
in the schema program.

### Typed callbacks

Custom validators and serializers are ordinary typed Sifr functions. Generated
adapters must preserve:

- input and output types,
- ownership and borrowing,
- `Result` error types,
- declared ordering,
- callback identity in the schema program,
- one optional concrete context type and immutable/mutable borrow mode,
- panic containment at the Rust boundary, and
- non-send/send restrictions.

There is no universal untyped callback receiving an arbitrary runtime object.

Context-aware entry points are specialized over one concrete caller-owned
context type `C`. Callbacks receive `&C` or `&mut C` according to their declared
effect; mutation is visible to the caller and follows ordinary Sifr borrowing.
Callbacks in one specialized schema must agree on `C`, or use an explicit typed
aggregate context. Calls without context specialize on `NoContext`. The native
core carries only a call-scoped opaque handle plus type identity and forwards
it through generated typed adapters; it never interprets, stores, erases, or
constructs context values.

### Static schema emission

For derived and otherwise statically declared schemas, specialization produces
a schema graph during `check`, `build`, editor analysis, and any other
specializing frontend mode. Package-owned deterministic Sifr const code
canonicalizes and semantically verifies that graph exactly once and asks the
compiler's general static-data facility to materialize the resulting immutable
schema program. `check` and editor analysis retain its identity and
diagnostics; build-like modes additionally embed it in the generated artifact.
The program contains stable node arrays, string tables, references,
constraints, policies, and typed callback slots.

The same schema program must have the same identity across `check`, `build`,
`run`, tests, cache keys, and editor analysis.

Const canonicalization and verification are incremental frontend queries keyed
by package/core-schema version, compiler structural-contract version, concrete
type identity, declaration metadata/configuration, and callback identities.
Editing an unrelated declaration reuses verified programs; dependency changes
invalidate only affected schemas. Check and editor execution must remain
within the repository's accepted frontend median/p95 budgets.

The only native entry point accepts a sealed compiler-emitted
`VerifiedSchemaProgram[T]`; package code cannot construct or mutate one at
runtime. The core borrows it directly and checks only its
header/version/hash/shape-identity envelope. It does not repeat semantic schema
verification, parse a graph, compile a schema, construct validators, or
populate a process/per-call cache. Corrupt artifact envelopes return an
internal load error before user data is processed.

## Public Package Model

The public surface should be familiar to Pydantic users while respecting
Sifr's static and fallible semantics.

Representative shape:

```sifr
from pydantic import BaseModel, Field, ValidationError

class User(BaseModel):
    id: int = Field(gt=0)
    name: str = Field(min_length=1, max_length=100)
    active: bool = True

def parse_user(payload: bytes) -> Result[User, ValidationError]:
    return User.model_validate_json(payload)
```

Sifr does not turn validation failures into exceptions. Familiar operations
therefore return `Result` where user input or custom behavior can fail.

The canonical capabilities are:

- validate a Sifr structural input as `T`,
- validate JSON bytes/text as `T`,
- validate a bare `str` or statically verified strings-leaf structural input
  as `T`,
- serialize `T` to a structural value,
- serialize `T` to JSON,
- obtain a reusable `TypeAdapter[T]`,
- obtain JSON Schema for the selected serialization/validation mode, and
- customize validation/serialization through typed declarations and optional
  caller-owned typed contexts.

Pydantic-style methods and a smaller functional API may coexist only as thin
views over the same Core Schema and execution engine. There is no second
functional validator implementation underneath convenience functions.

## Core Schema Contract

### Role

Core Schema is a declarative, versioned internal contract between
`pydantic-sifr` and `pydantic_sifr_core`.

It describes:

- accepted input forms,
- strict/lax conversions,
- output value shape,
- constraints,
- defaults,
- aliases,
- extra-field policy,
- union selection,
- recursion,
- validation callback positions,
- serialization behavior,
- description behavior, and
- stable error codes.

### Node families

The complete node algebra must cover:

| Family | Required nodes |
| --- | --- |
| Scalars | none, bool, exact integer, fixed integer, float, decimal, string, bytes |
| Specialized scalars | date, time, datetime, duration, UUID, URL, multi-host URL/DSN, pattern, exact rational `Fraction`, `Complex`, and package-provided scalar adapters |
| Constraints | numeric bounds/multiples, decimal total/fractional digit bounds, length bounds, pattern, finite and clock-relative temporal bounds |
| Products | record/model, tuple, typed mapping |
| Collections | list, set, frozen set, typed sequence policies, and lazy `ValidatedIterator[T]` |
| Sums | optional, literal, enum, smart/left-to-right ordinary union with explicit auto-collapse policy, field/path-discriminated tagged union and typed-callback-discriminated tagged union |
| Control | default, nullable, definitions, reference, recursion guard, strict/lax branch, JSON/structural-input branch, embedded-JSON child decoder, typed sequential chain and compositional error override |
| Transforms | before, after, wrap and plain typed validators; built-in string normalization |
| Serialization | alias, inclusion/exclusion, computed field, typed serializer and representation override |

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

### Program invariants

Core Schema verification rejects:

- dangling references,
- duplicate definition identities,
- impossible type/output relationships,
- callback signature mismatches,
- invalid constraint combinations,
- a missing or unsafe Sifr integer JSON boundary descriptor,
- serialization nodes incompatible with validation output,
- unbounded recursive entry,
- ambiguous discriminator maps or invalid typed discriminator callbacks,
- an error override whose custom code lacks a static message, whose code
  collides with a built-in code while changing that built-in message, or whose
  message/context is not statically serializable,
- defaults that do not validate under their declared policy, and
- unknown schema versions or node kinds.

An unvalidated default expression or factory must produce the declared field
output type. A validated default may instead produce any statically known
input type accepted by the field schema. A const-evaluable validated default
is executed during schema verification and a failure is a package/compiler
diagnostic. A non-const factory result is run through the same validator plan
at use time and either constructs the declared output type or returns its
ordinary validation errors. A configuration cannot defer this typing choice
to runtime.

Verification failures are package/compiler diagnostics. All public adapters
are specialized for a statically known `T`; no runtime schema builder or
alternate validator path exists.

### Versioning

The schema program begins with:

- schema-program format version,
- compiler structural-contract version,
- Rust bridge structural-call version,
- callback ABI version,
- feature bitmap, and
- payload identity hash.

`pydantic-sifr` and `pydantic_sifr_core` release together and require an exact
supported contract tuple. Core Schema is an internal build artifact, not a
cross-release wire protocol. A contract change increments the relevant version
and rebuilds dependents; the core does not carry backward interpreters.
Unknown or mismatched contracts fail during build before user data is
processed.

## Native Core

### Responsibilities

`pydantic_sifr_core` owns:

- runtime validation and serializer program execution without recompilation,
- verified-program envelope and structural-shape identity checks,
- JSON and structural input adapters,
- strict/lax scalar conversion,
- constraint execution,
- record and collection validation,
- aliases and extra-field handling,
- union ranking and discriminator dispatch,
- recursion guards,
- default handling,
- callback scheduling,
- aggregate errors,
- validated value storage,
- serializer-plan execution,
- JSON writing, and
- source positions needed by diagnostics.

The Sifr package owns the one semantic Core Schema canonicalizer and verifier,
implemented as deterministic const-evaluable Sifr code. The native core never
implements a second verifier; its envelope checks protect artifact integrity,
not schema semantics.

It does not own Sifr syntax, class lookup, package imports, compiler type
resolution, or user-facing declaration analysis.

### Input abstraction

The core uses one input abstraction over supported data sources. Input adapters
provide:

- kind inspection,
- exact primitive access,
- sequence iteration,
- mapping lookup and ordered iteration,
- source location,
- strictness-relevant origin, and
- safe replay required by unions.

Validation selects one of three input profiles over that abstraction:

- `native` preserves the source's native primitive kinds,
- `json` applies JSON-origin conversion and strictness rules, and
- `strings` accepts either a bare `str` root or a structural value whose scalar
  leaves are strings and applies the documented Pydantic-style strings-input
  conversions.

The strings-profile public entry point is generic over an input `S`.
Compile-time shape verification accepts `S` when it is `str`, or when every
terminal scalar in its structural projection, including mapping keys, is
`str`; nested records, mappings, and sequences are allowed. A bare `str` uses
the normal scalar projection. The compiler-generated projection for `S` is
therefore the input type—there is no `Any` or package-owned recursive value
tree. The profile reuses the native structural adapter with a leaf-kind
restriction and different conversion rules; it is not a third schema compiler,
value representation, or validation engine.

JSON validation uses a `jiter::JsonValue` document. The value-tree form is the
canonical path because aggregate errors, aliases, recursive records, and union
candidate evaluation require replay and random access. A second streaming
semantic engine is not maintained. The JSON document and normalized validated
arena are two intentional representations with different jobs; the rejected
representation is an additional copied bridge-object tree between the arena
and `T`.

Native Sifr structural inputs use a compiler-generated structural projection
rather than converting through JSON.

### Validated value arena

Python provides one universal object representation; Sifr does not. Successful
decode execution therefore produces a per-call arena containing normalized
validated values:

```text
ValidatedValue =
    None
  | Bool
  | ExactInt
  | FixedInt
  | Float
  | Decimal
  | String
  | Bytes
  | Sequence(range)
  | Mapping(range)
  | Record(range)
  | Variant(tag, child)
  | SpecializedScalar(kind, payload)
```

The arena:

- has one root value,
- uses compact indices rather than recursive bridge allocations,
- owns converted strings/bytes/numbers exactly once,
- supports move-out during `Construct[T]`,
- records schema identity,
- is invalidated after successful consuming construction, and
- has bounded recursion and collection limits.

The Sifr bridge exposes the arena as a sealed opaque resource. Package code
cannot forge nodes or reinterpret one schema's output as another type.

`SpecializedScalar` payloads are crate-neutral normalized components, never
public `speedate`, `chrono`, `uuid`, `url`, `rust_decimal`, or `bigdecimal`
crate values. Examples include calendar/time components plus offset and
microsecond precision, UUID bytes, normalized URL text/components,
compiled-pattern source and flags, and exact decimal coefficient and scale.
`StructuralConstruct` reconstructs the canonical Sifr stdlib type:

- `datetime`, `date`, `time`, and duration use the extended stdlib value types
  that preserve the validated microsecond and timezone-offset components,
- UUID and URL use the existing stdlib-backed types, and
- compiled string patterns use the native-backed `re.Pattern`, and
- the Core Schema `decimal` node uses Sifr `bigdecimal`, backed by
  `bigdecimal::BigDecimal`, because Pydantic decimal precision is unbounded;
  fixed-precision Sifr `decimal` is a separate package-provided scalar adapter
  with its own range and precision contract.

`jiter`, `speedate`, and focused crates parse or normalize these components;
they do not define the Sifr-facing type or schema contract.

### Validation state

One validation state carries:

- strict/lax mode,
- current error path,
- recursion stack,
- exactness class,
- internal successfully-validated-field count,
- partial error accumulator,
- input source kind and profile,
- optional call-scoped typed context handle, type identity, and borrow mode,
- one call-scoped UTC clock snapshot for relative temporal constraints,
- resource limits, and
- callback state.

Tagged and ordinary unions are separate schema nodes and algorithms. An
ordinary union declares `mode = smart | left_to_right` and defaults to
`smart`. A one-choice union collapses to that choice by default; disabling
auto-collapse preserves the union branch label and aggregate-error boundary.

A tagged union either reads a declared field/path or invokes one statically
typed discriminator callback exactly once. The result is a declared tag used
to select a branch from the same indexed discriminator map or return a
discriminator error. Map lookup remains indexed; the callback form adds only
the typed tag computation. A smart ordinary union follows pinned
`pydantic-core/src/validators/union.rs:117-191`, with exactness order from
`pydantic-core/src/validators/validation_state.rs:15-19`:

1. an exact successful candidate with no field-count data short-circuits;
2. otherwise every candidate is evaluated;
3. field counts decide only when both candidates carry a count and those
   counts differ, with the larger count winning;
4. otherwise exactness ranks `Lax < Strict < Exact`;
5. a remaining tie selects the earliest declared candidate;
6. the selected candidate bubbles its exactness floor and adds its successful
   field count to enclosing validation state, so nested record/model counts
   participate additively in an outer union; and
7. an internal `Omit` seen before any successful candidate is remembered, but
   ignored after a best match exists; if no candidate succeeds and any
   candidate omitted, the union omits, while other non-line internal errors
   propagate.

The smart algorithm is a declaration-order left fold against the current best,
not a sort or an order-independent ranking key: its mixed
counted/uncounted comparison is intentionally non-transitive. When all
candidates fail with ordinary line errors, the aggregate retains declaration
order. Each candidate uses its declared choice label when present and otherwise
falls back to the validator/schema name. In `left_to_right` mode
(`pydantic-core/src/validators/union.rs:194-212`) the first result other than
ordinary line errors wins
immediately; total line-error failure uses the same ordered, labelled
aggregate.

The internal field count is ephemeral validation state used only for ranking.
It is not a public `__pydantic_fields_set__` attribute and is not retained on
the constructed Sifr model.

The Sifr-native `core/smart_union_ranking` family discriminates mixed
counted/uncounted candidates, exactness ordering, declaration-order ties,
additive nested bubbling, `Omit`, choice labels, both modes, and auto-collapse.

Any intentional difference from the pinned Pydantic behavior is recorded in
the compatibility manifest.

### Serialization

The serializer plan drives one monomorphized native call over
`T: StructuralProject`. It pulls from the compiler-generated call-scoped view;
it does not read private generated-Rust fields by layout assumption, issue
per-field Sifr/Rust calls, or rely on a Sifr equivalent of Python `__dict__`.

The plan owns:

- aliases,
- validation versus serialization representation,
- inclusion/exclusion,
- default/none policies,
- computed fields,
- tagged-union representation,
- custom serializers,
- typed caller-owned serialization-context forwarding,
- exact integer output policy, and
- target-format constraints.

Include and exclude arguments use one package-owned recursive value language:

```text
Selection =
    All
  | Fields[ordered map[field name, Selection]]
  | Elements {
        default: Option[Selection],
        indices: ordered map[signed index, Selection]
    }
  | Entries {
        default: Option[Selection],
        keys: ordered map[declared key type, Selection]
    }
```

`All` selects or removes the current node; `Fields` recurses by field name.
For `Elements`, `default` applies to every element and a matching `indices`
entry overlays it for that element. Signed indices are normalized against the
node's pre-filter sequence length before lookup; when two declared indices
normalize to the same index, the later declaration wins. `Entries` applies the
same default/override model to typed
mappings, matching entries by validated key value without positional
normalization. A record-wide default desugars to its statically enumerated
`Fields`.

Overlay is recursive and deterministic: missing entries inherit the base;
branch maps merge by key; an explicit branch replaces a base `All`; and an
explicit `All` replaces a base branch. A `default` present on both sides
overlays recursively; a `default` present on only one side is inherited. This
makes an index-specific nested selection refine or replace the default while
unrelated nested keys remain combined.

Element-default overlay and schema/call composition are distinct operations.
Following pinned
`pydantic-core/src/serializers/filter.rs:150-257`, sequence and mapping
filters are resolved per node against
every original mapping key or pre-filter sequence index, in this precedence
order:

1. a call-time exclusion that terminally selects the entry removes it;
2. otherwise, when a call-time inclusion exists, the entry is emitted when
   that inclusion selects it, forwarding any nested selection it carries
   together with a nested call-time exclusion carried from clause 1, and is
   otherwise removed unless the schema-declared inclusion selects it, in which
   case clauses 3 and 4 decide;
3. otherwise, a call-time exclusion that selects the entry only with a nested
   selection emits it and forwards that nested selection; and
4. otherwise the schema-declared filter decides: the entry is emitted when its
   inclusion is absent or selects the entry and its exclusion does not.

At the same sequence/mapping node, schema and call-time inclusions combine by
union, while call-time inclusion can re-include an item removed only by a
schema exclusion. Schema-filter inclusion and exclusion combine as
intersection. The Sifr-native `core/selection_precedence` family discriminates
schema-include plus call-include union, schema include/exclude intersection,
call-include over schema-exclude, negative and positive-out-of-range index
normalization, and empty nested selections. For records, a
statically declared field-level serialization exclusion is unconditional: the
field is removed before call-time selection and cannot be re-included.
Remaining record fields use the call-time parts of the same rules with no
schema filter. Signed indices normalize against the pre-filter sequence
length. Every signed index is normalized by Euclidean modulo for a non-empty
sequence, including positive out-of-range indices; no index matches an empty
sequence
(`pydantic-core/src/serializers/filter.rs:20-56,102-103,282-283`).
`Elements` applies only to statically sized-at-serialization
collections such as lists and tuples. An index selection on
`ValidatedIterator[T]` is rejected because negative/modulo normalization would
require consuming an unsized iterator; iterator filtering uses an explicitly
streaming predicate callback instead. A nested selection beneath a scalar
leaf, including a structurally
incompatible `Fields`, `Elements`, or `Entries` value, is accepted and ignored
rather than rejected; shape checking applies only where the declared type has
fields, elements, or entries. An empty nested exclusion emits the composite
value and excludes no children. An empty nested inclusion on a composite
selects no children and therefore empties that subtree unless a
schema-declared inclusion independently selects a child under clause 2; it is
inert below a scalar leaf.

This replaces Python's overlapping
set/dict/list/dict-view/`True`/ellipsis/`__all__` spellings with one typed
representation while preserving portable default, signed-index, override, and
composition behavior. A Python `None`-valued entry desugars to `All` under an
inclusion and to an empty nested selection under an exclusion. Python
custom-membership and duck-typed `__contains__` precedence are
`not-applicable`; only their explicitly selected key/index results may be
represented as an adapted typed `Selection`.

The pinned filter first normalizes Python `dict`/`set` spellings and has a
separate unsized-iterable path that rejects negative indices. Those are oracle
harness mechanics, not public Sifr forms: typed `Selection` is the sole
representation, and `Elements` is rejected altogether for
`ValidatedIterator[T]` as specified above.

Serializer input/schema type mismatches are statically impossible through
`T: StructuralProject`. Pydantic Core's runtime warning-and-passthrough cases
are `not-applicable`; they are never a fallback behavior. A custom serializer
with an incompatible declared input or output type is a compile-time schema
diagnostic.

JSON output is streamed to a writer. It does not allocate a complete
`serde_json::Value` first. `serde_json` mechanisms may be reused for escaping
and scalar formatting, while Sifr's schema program remains the semantic
authority.

### Integer JSON profiles

Every model or adapter that can serialize an integer selects exactly one
accepted Sifr profile in its static configuration: `json.exact`, `json.web`, or
`json.string_ints`. Nested fields inherit the containing profile unless a field
declares a supported override. The schema program stores that selection and
the native core routes integer reading/writing through
`sifr_runtime::json`; it never reimplements or weakens those helpers.

The package's deterministic const schema emits a general compiler-owned
`JsonIntegerBoundaryDescriptor` containing the selected profile, declared
integer kind, static range if bounded, and source path. The compiler's
package-neutral boundary verifier checks that descriptor before sealing the
schema program. Missing or unsafe information activates the reserved built-in
diagnostic `SIFR-INT-0009`; `ps_1` owns its registry entry, documentation, and
CLI/LSP tests. Pydantic-Sifr supplies data to this general verifier but neither
owns nor emits the top-level code.

- `json.exact` emits canonical base-10 JSON numbers without precision loss.
- `json.web` emits numbers only within JavaScript's safe range; `int64`,
  `uint64`, and unbounded `int` default to decimal strings unless a static safe
  range authorizes numeric output. A violating runtime value returns
  `JsonIntegerRangeError` with the model path.
- `json.string_ints` emits every integer as a canonical decimal string.

JSON Schema generation consumes the same profile. Under `json.web`,
`int8/16/32` and `uint8/16/32` are JSON integers with their exact
`minimum`/`maximum`; `int64`, `uint64`, and unbounded `int` are decimal strings
unless a statically proven JavaScript-safe range authorizes the same bounded
integer form. Under `json.string_ints`, every integer is a decimal string.
String representations use the locked decimal-string pattern and
`x-sifr-format`. Under `json.exact`, numbers use `type: integer`,
`x-sifr-integer-profile: exact`, exact bounds where available, and a client
warning unless the declared schema target supports exact integer parsing.
Browser-facing schema must never claim an unbounded numeric integer.

An absent or insufficient profile fails at compile time with `SIFR-INT-0009`,
including path, boundary, selected-or-missing profile, static range when
known, and suggested policy.
Pydantic oracle expectations that assume an unbounded JSON integer are
`adapted` to this language-wide contract rather than treated as a competing
package policy.

## Error Contract

All user-data failures return one `ValidationError` containing an ordered list
of `ErrorDetail` values.

Each detail contains:

- stable machine-readable code,
- ordered location segments,
- human-readable message,
- expected contract summary,
- optional safe input summary, controlled by the error-disclosure policy,
- optional context,
- optional JSON byte/line/column position, and
- originating schema node identity for diagnostics and testing.

An `ErrorOverride` wraps any validation subgraph and replaces that subgraph's
failure aggregate with one error at the wrapper location. It may reference a
built-in Sifr error code and its canonical message, or declare a package-owned
custom code plus a required static message and typed static context. Built-in
codes and meanings remain Sifr-owned; custom codes occupy a distinct
package-qualified namespace and cannot redefine a built-in code.

The public `ErrorDisclosure` policy has `IncludeSafeInput` and `OmitInput`
modes, selected by static model/adapter configuration with a per-call
override. `IncludeSafeInput` uses bounded, redacted summaries that never invoke
user formatting code; `OmitInput` removes the field from every detail. The
choice does not change validation, ordering, codes, or locations.

Locations support:

- field names,
- aliases,
- sequence indices,
- mapping keys,
- union branches,
- validator stages, and
- root/model positions.

Syntax errors, validation errors, serialization errors, callback errors,
resource-limit errors, and contained Rust panics remain distinct typed errors.
Static schema verification failures are compiler/package diagnostics. Raw Rust
errors, PyO3 errors, and `serde_json::Error` values do not leak into the public
Sifr API.

Error collection is bounded by an explicit policy to prevent adversarial inputs
from allocating unbounded error lists. Reaching the limit produces a stable
truncation fact rather than panicking or silently claiming complete coverage.
Sifr's locked JSON boundary enforces the integer-digit budget before allocating
an unbounded value and returns the existing
`JsonLimitError { message: str, limit: int }` from `sifr_runtime::json`.
Separately, `pydantic_sifr_core` owns explicit input-byte, nesting,
collection-size, string-size, recursion, and accumulated-error limits and
returns package-owned `ResourceLimitError { kind, limit, location }`. These are
distinct authorities: the package reuses the language integer parser/error and
does not pretend the other package limits are locked Sifr runtime behavior.

## Reuse Policy

### Production dependencies

| Component | Decision | Boundary |
| --- | --- | --- |
| `jiter` | Reuse directly | JSON parsing, exact/lossless numbers and locations; Python feature disabled |
| `speedate` | Reuse directly | Temporal parsing into crate-neutral components reconstructed as canonical Sifr stdlib types |
| `serde` | Reuse selectively | Format interoperability and writer mechanisms, never schema authority |
| `serde_json` | Reuse selectively | JSON escaping/formatting or adapters, never canonical validation semantics |
| `regex` | Reuse directly | Pattern compilation/matching with bounded policy |
| `url` and IDNA crates | Reuse directly | URL/IDNA mechanism behind Sifr types and errors |
| `uuid` | Reuse directly | UUID parsing/formatting behind Sifr policy |
| `base64` | Reuse directly | Binary-text codecs behind schema policy |
| `num-bigint` | Reuse directly | Exact integer mechanism compatible with Sifr's integer model |
| `num-rational` | Reuse directly | Canonical exact rational mechanism behind package-owned `Fraction` |
| `num-complex` | Reuse directly | Complex-number representation and arithmetic behind package-owned `Complex` |

Dependency features must be minimal. Python, extension-module, dynamic loading,
and unused default features are disabled.

### Selective algorithm ports

Small algorithms may be ported or behaviorally reimplemented when their
semantics are selected:

- boolean and numeric conversion tables,
- integer-string normalization,
- exactness scoring,
- internal successfully-validated-field union scoring,
- alias-path lookup,
- recursion detection,
- constraint ordering,
- error-location construction, and
- serializer include/exclude decisions.

Whole Pydantic Core validator or serializer modules are not copied. A port must
be small enough to state its Sifr contract independently and must carry source
revision and license provenance.

### Rejected dependencies and approaches

| Approach | Decision | Reason |
| --- | --- | --- |
| Embed CPython/Pydantic Core | Reject | Violates native deployment and imports Python identity, GIL and packaging |
| Link `pydantic-core` as a Rust library | Reject | Its central interfaces and outputs are Python-shaped |
| Fork and remove PyO3 | Reject | Near-total rewrite plus permanent upstream drift |
| Serde derive as validation engine | Reject | Fail-fast format decoding cannot express the complete aggregate/coercive contract |
| Schemars as schema authority | Reject | Creates a second schema model and unstable output ownership |
| Garde/`validator` as core | Reject | Post-construction Rust validation duplicates schema and cannot own decoding |
| Per-model compiler validation lowering | Reject | Makes package behavior a compiler special case |
| Copied arena-to-model bridge tree | Reject | JSON already has a parse tree and normalized arena; a third recursive bridge-object tree adds no semantic value |
| Parallel streaming and tree validators | Reject | Duplicates semantics and substantially increases maintenance |

## Pydantic Compatibility and Test Reuse

### Compatibility classes

Every relevant upstream behavior is classified as:

- `same`: Sifr intentionally matches normalized Pydantic behavior,
- `adapted`: equivalent capability with a documented Sifr-safe difference,
- `not-applicable`: behavior depends on Python-only semantics, or
- `rejected`: behavior conflicts with Sifr's guarantees.

No test is silently omitted because it is inconvenient.

### Portable test categories

Port extensively:

- JSON primitive conversions,
- strict/lax matrices,
- exact and boundary numbers,
- string and byte constraints,
- date/time/datetime/duration cases,
- required, defaulted and nullable fields,
- aliases and alias paths,
- extra-field policies,
- lists, tuples, mappings and sets,
- literals and enums,
- tagged and untagged unions,
- nested and recursive models,
- aggregate locations and messages,
- custom validator ordering,
- serialization aliases and exclusion policies,
- JSON Schema examples,
- malformed/adversarial JSON,
- fuzz seeds, and
- portable benchmarks.

Fixed-width integer schemas have no Python/Pydantic oracle because Python
integers are arbitrary precision. The Sifr-native `core/fixed_integer` contract
covers `int8/16/32/64` and `uint8/16/32/64`: strict mode
accepts only integer-kind inputs; lax mode additionally accepts exactly the
lossless bool, canonical integer-string, and finite integral-float conversions
of the exact-integer node. Conversion first produces a mathematical exact
integer and then performs one target-bound check. Underflow or overflow returns
stable `integer_overflow` detail containing the target type and inclusive
bounds; it never wraps, truncates, saturates, or routes through float. Native
serialization preserves the fixed type; JSON representation follows the
selected Sifr integer profile.

Before the public-model facade exists, `core/pattern_value` is a Sifr-native
package contract for the Core Schema compiled-pattern node: construct a
native-backed `re.Pattern` from source and flags, preserve both components,
match without recompiling per call, return a stable invalid-pattern error, and
serialize the source form. The later `api/pattern` family differentially checks
the public field and JSON Schema behavior against Pydantic.

Five additional native contracts close places where the upstream fixture
does not by itself discriminate Sifr's complete rule:

- `core/string_pipeline_order` proves Unicode-scalar versus byte length and
  every stated normalization/check boundary;
- `core/decimal_digit_counting` proves raw/normalized, zero, trailing-zero,
  fractional, and saturating whole-digit cases;
- `core/fraction` proves normalized numerator/positive-denominator identity,
  exact integer/decimal/rational parsing, zero-denominator rejection,
  strict/lax and JSON/strings profiles, constraints, and canonical
  serialization before the public adapter exists;
- `core/smart_union_ranking` proves counted/uncounted comparison, exactness,
  stable ties and labels, nested additive bubbling, `Omit`, both union modes,
  and optional auto-collapse; and
- `core/selection_precedence` proves schema/call inclusion and exclusion,
  index normalization, and empty nested selection semantics.

Owned Sifr structural inputs cannot contain Python-style identity cycles.
Recursive-schema success behavior is grounded in portable upstream acyclic
cases, while recursion/resource guards use the Sifr-native
`core/recursion_limit` contract: generated acyclic inputs beyond the configured
depth must return a stable `recursion_limit` error without panic or
exponential work. Cyclic-object identity tests are `not-applicable`.

Do not port as Sifr behavior:

- Python object identity,
- Python subclass and duck-typing behavior,
- metaclass mutation,
- descriptors,
- `__dict__` and `__pydantic_fields_set__`,
- pickle,
- CPython garbage collection/reference counts,
- arbitrary `from_attributes` object access,
- Python exception wrapping, or
- extension-module import behavior.

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

### Neutral fixtures

Portable cases are stored in a language-neutral fixture format. Each fixture
records:

- an origin variant: `Upstream { repository, commit, test_identifier }` or
  `Native { contract_id, contract_version }`,
- normalized schema,
- input source and value,
- an optional fixed UTC clock instant for clock-relative validation,
- validation/serialization mode,
- expected normalized value or error list,
- compatibility class,
- reason for adaptation/rejection, and
- license/provenance notice when the origin is upstream.

Committed fixtures, not the layout of upstream pytest files, are the stable CI
input. Native-origin fixtures do not enter the upstream exact-set ledger.

### Differential oracle

A development-only differential runner executes the neutral corpus against:

1. pinned Pydantic/Pydantic Core, and
2. the native Sifr implementation.

It normalizes values, locations, codes, and intentional Result-versus-exception
differences before comparison. Published package builds do not invoke Python,
download Pydantic, or require the oracle.

An upstream-audit tool reports newly added or changed relevant upstream cases.
It never changes Sifr behavior or fixtures automatically.

### Upstream pin updates

An upstream revision changes only in a dedicated reviewed compatibility PR:

1. update the sole Pydantic compatibility commit and regenerate the complete
   sorted ledgers for its API and in-tree Core test roots;
2. fail on every added, removed, renamed, skipped, or newly `xfail` node until
   its manifest disposition and owning milestone are reviewed;
3. regenerate only the affected neutral fixtures and differential snapshots;
4. review semantic deltas, provenance, licenses, and benchmark/fuzz seeds;
5. reject automatic behavior changes—an intentional contract change requires
   its own design decision and public compatibility entry; and
6. merge the new pin only when the ledger has exact set equality, every
   retained anchor passes upstream, and both the native corpus and differential
   oracle pass.

Historical manifest revisions remain available so a dependency update cannot
erase why a case was adapted, rejected, or declared not applicable.

## Public Compatibility Policy

The package aims for Pydantic-familiar capability and naming, not Python runtime
emulation.

Permanent Sifr-safe differences include:

- validation and serialization failures return `Result`,
- schemas for statically known types are checked and emitted at build time,
- validators and serializers are statically typed,
- exact Sifr integer behavior is preserved,
- Pydantic's unbounded numeric JSON and JSON Schema expectations are adapted
  to Sifr's locked `json.exact`, `json.web`, or `json.string_ints` profile;
  browser-facing schemas never advertise an unsafe unbounded number,
- ownership and mutation effects remain visible,
- arbitrary runtime class monkey-patching is unsupported,
- Python object identity and attribute probing are unsupported,
- `extra='allow'` is adapted: it is available only when the model declares a
  typed extra-field mapping destination; otherwise extra fields are ignored or
  rejected according to the static model policy,
- `from_attributes`, ORM-style arbitrary attribute probing,
  `revalidate_instances`, and `arbitrary_types_allowed` are not applicable to
  fixed-layout Sifr values,
- `exclude_unset` is not applicable because Sifr models do not retain a
  Python-style per-instance field-set side channel; `exclude_defaults`,
  `exclude_none`, and explicit typed include/exclude selections remain,
- Python `TypedDict` optional-without-default key semantics (`NotRequired` and
  `total=False`) are not applicable to fixed-layout Sifr records: every
  declared field has a value or an explicit default/`Option` representation,
  and upstream assertions whose result omits such a declared key are
  classified `not-applicable`,
- cyclic runtime input objects are not representable by owned Sifr structural
  values; recursive schemas and arbitrarily deep acyclic inputs remain fully
  supported within resource limits,
- serializer wrong-type warnings and passthrough are statically impossible and
  therefore not applicable,
- Pydantic `Decimal` infinity/NaN and `allow_inf_nan` behavior is not
  applicable because Sifr `bigdecimal` is finite by definition,
- simultaneous lower/upper string case conversion is rejected as an
  incompatible static schema instead of inheriting Pydantic's silent
  lower-case precedence,
- `regex_engine='python-re'` is not applicable without a Python runtime;
  portable patterns use the single native Rust-regex contract, and
  `coerce_numbers_to_str` remains a first-class string-node policy,
- a precompiled Python pattern under Pydantic's default engine is adapted only
  when its source and flags translate exactly to Rust `regex`; Python-only
  constructs are rejected at compile-time pattern construction,
- experimental `allow_partial` validation is rejected because it can silently
  discard invalid input while claiming a complete `T`; `missing-sentinel`
  identity is normalized away into ordinary missing-input/default/`Option`
  semantics,
- include/exclude index selection is defined only for sized collections;
  lazy `ValidatedIterator[T]` values require a typed streaming predicate and
  never buffer solely to normalize negative indices,
- Python reflection, internal `repr`, `__dict__`, field-set, subclass-identity,
  and exception-construction assertions may be provenance scaffolding only and
  never define a retained neutral expectation,
- unsupported dynamic behavior fails explicitly rather than falling back, and
- error codes are Sifr-owned even when initially mapped from a Pydantic case.

The compatibility documentation includes a searchable API/behavior matrix.

## Safety and Resource Contract

The native core must:

- contain panics at every package-authored Rust boundary,
- use no data-dependent `unwrap`/`expect`,
- accept only sealed compiler-emitted verified programs and reject a corrupt or
  contract-mismatched envelope before execution,
- guard recursive input and recursive schemas,
- bound input bytes, nesting, collection size, string size, integer digits and
  accumulated errors through explicit policies,
- preserve exact integers without float round trips,
- avoid unsafe code unless separately justified, audited and fuzzed,
- never expose borrowed data beyond its document/arena lifetime,
- never construct partially valid Sifr models,
- avoid quadratic union/alias behavior where an indexed plan is possible, and
- produce deterministic results independent of hash iteration order.

## Performance and Maintainability Contract

- Static schema programs are not rebuilt for every validation call.
- Record field and alias lookup tables are compiled once.
- Tagged-union branch lookup is indexed after direct field/path extraction or
  one typed discriminator callback.
- Validated strings, bytes and big integers are allocated at most once before
  typed construction where ownership permits.
- JSON serialization streams output rather than building a second value tree.
- There is no process, dynamic-library or Python boundary.
- Schema and callback identities participate in build/cache keys.
- Incremental frontend queries cache verified schema programs at the same
  dependency granularity and obey the accepted edit-loop median/p95 budgets.
- Benchmarks separate parse, validate, construct, project and write costs.
- Representative comparisons against pinned Pydantic Core are published, but
  semantic correctness and Sifr safety are never weakened to win a benchmark.
- Once a milestone establishes its performance baseline, unexplained material
  regressions block subsequent milestone closure.
- Rust modules remain responsibility-oriented and below the repository's file
  size guardrail.
- Every schema node has one primary implementation owner, one specification
  table, and one or more focused supporting test families.

## Non-Goals

- Exact source or binary compatibility with Python Pydantic.
- A Python runtime, PyO3 extension, or Python object bridge.
- Supporting Pydantic plugins by executing Python.
- Reusing Pydantic's Python-specific Core Schema nodes.
- Making arbitrary Sifr values dynamically introspectable at runtime.
- Making Core Schema the normal beginner-facing API.
- Runtime model/schema construction or a runtime schema compiler.
- Adding JSON-specific rules to the Sifr compiler.
- Replacing Sifr's ordinary type checker with validation schemas.
- Implementing Pydantic Settings, web-framework integration, ORM behavior or
  unrelated ecosystem packages inside the core architecture. Those may be
  separate packages consuming the completed public contract.
- Supporting a temporary reduced public architecture that later requires a
  second validation engine or compatibility fallback.

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
  candidate `SATISFIED` Opus review before the create-PR and merge gates.

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
- Certify supported compiler/core/package version combinations.
- Add end-to-end demos and package documentation.
- Add and snapshot-test the canonical
  `demos/pydantic_sifr_demo.sifr` in the external `pydantic-sifr` repository.
- Perform independent whole-architecture and implementation review.

Exit gate: all acceptance criteria pass using released Sifr and
`pydantic-sifr` artifacts without access to the source checkout, Python, or the
upstream repositories, and the canonical demo runs from an installed package
without a Sifr compiler source checkout.

## Acceptance Criteria

### Architecture

- `sifr-lang/sifr` contains no Pydantic-specific compiler branch, type, schema
  node, decorator name or JSON validation policy.
- The external
  [`sifr-lang/pydantic-sifr`](https://github.com/sifr-lang/pydantic-sifr)
  repository contains the Sifr package and native core as separately owned
  components with one versioned schema contract.
- `sifr-lang/sifr` contains no production `pydantic` package or
  `pydantic_sifr_core` source as a workspace member, vendored subtree, or
  submodule.
- Validation, serialization and JSON Schema generation consume one Core Schema
  authority.
- Static schemas are verified and deterministically materialized during
  `check`, build, editor analysis, and every specializing frontend mode by the
  same package const implementation; build-like modes embed the result.
- There is no runtime schema compiler or alternate dynamic adapter path.
- The structural Rust bridge is a merged, certified general contract with a
  non-Pydantic conformance consumer.
- Package const specialization can emit bounded issues whose package reason is
  mapped to registry-owned `SIFR-META-*` diagnostics with identical CLI/LSP
  identity through a non-Pydantic conformance consumer.

### Native execution

- Published artifacts contain no Pydantic, Pydantic Core, PyO3, CPython, GIL or
  dynamic extension dependency.
- JSON input uses Python-free `jiter`.
- Exact Sifr integers survive parse, validation, construction and serialization.
- Integer JSON and generated-schema behavior route through one explicitly
  selected locked Sifr profile; missing/unsafe policy fails with
  `SIFR-INT-0009` or `JsonIntegerRangeError` as appropriate.
- Successful validation constructs the requested native Sifr type.
- Serialization observes the current typed value after mutation.
- Construction and serialization each use one monomorphized structural native
  call; the core never imports generated package bridge types.
- User-controlled data and callbacks cannot produce an uncaught Rust panic.

### Behavior

- Required Pydantic-equivalent features have neutral fixtures and provenance.
- Every relevant upstream case is classified as same, adapted, not applicable
  or rejected.
- The manifest's sorted file/node ledger exactly equals the API and in-tree
  Core test roots at the sole Pydantic pin; no upstream path, collected
  selector, or parameter can disappear without failing the audit.
- The generated Core Schema kind ledger exactly equals every pinned
  `CoreSchemaType` and `CoreSchemaFieldType` literal, with one accepted
  primary implementation/disposition owner and either a non-empty
  evidence-family set or the explicit `ps_0` disposition audit per kind.
- Every fixture family assigned to a milestone passes before that milestone
  releases; `ps_11` performs re-audit and certification, not deferred behavior
  implementation.
- Strict/lax behavior, union ranking, error ordering and serializer profiles are
  deterministic and documented.
- Validation returns aggregate typed errors with stable codes and locations.
- Intentional Sifr differences are public and tested.

### Maintainability

- No permanent fork of Pydantic or Pydantic Core exists.
- Mature focused Rust dependencies are reused at their natural boundary.
- No schema behavior is implemented independently in both Sifr and Rust.
- No third arena-to-model bridge tree or per-call schema rebuild exists.
- Dependency features, licenses and provenance are audited.
- Fuzzing covers schema verification, JSON input, validation plans and writers.
- Benchmarks and regression gates cover each execution stage.
- Compiler and package conformance tests prevent accidental coupling.

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
