# Ad Hoc Phase: Sysroot Stdlib Interop Declaration Cleanup

## Status

Closeout review in progress.

## Objective

Make private sysroot stdlib Rust interop declarations read as pure checked
declarations. A canonical private `_sifr.*` declaration that targets
`sifr_stdlib.*` uses the compiler-owned sysroot no-panic trust policy, and its
body is an ellipsis-only Rust interop stub. User and package Rust interop keeps
its explicit panic-surface contract.

This phase makes the compiler-owned stdlib declaration surface concise while
preserving Sifr's no user-triggerable panic guarantee. It also makes the
ellipsis stub body a checked Rust interop declaration form instead of an
ordinary Sifr expression body.

## Design Rules

- A private sysroot stdlib declaration may omit `panic=` only when all of these
  are true:
  - the source module is a private `_sifr.*` stdlib module loaded from the
    resolved sysroot,
  - the declaration resolves through the compiler-owned synthetic sysroot
    package context,
  - the Rust target root is exactly `sifr_stdlib`,
  - the declaration is recorded as satisfying a sysroot-owned no-panic trust
    requirement.
- User package declarations, package-local bridge declarations, and non-stdlib
  private declarations continue to use the normal Rust interop panic-surface
  contract.
- A Rust interop declaration stub body is exactly one ellipsis statement:

  ```sifr
  @rust(sifr_stdlib.math.sqrt)
  def sqrt(x: float) -> float: ...
  ```

- Ellipsis is accepted only as the complete body of an eligible Rust interop
  declaration. It is not a general Sifr expression body, and mixed bodies such
  as `...` plus `return` are rejected.
- Lowering preserves the annotated function shape: parameters, return type,
  async flag, method receiver shape, decorators, source spans, and Rust interop
  metadata.
- Codegen must never infer behavior from an empty declaration body. Rust
  interop codegen emits the target call from Rust interop metadata for every
  supported Rust interop root.
- Trust metadata, cache fingerprints, probes, and diagnostics use the effective
  panic policy so omitted sysroot policy is still visible to compiler-owned
  validation and auditing.

## Non-Goals

- No change to public user/package panic-policy requirements or target-root
  semantics.
- No implicit panic trust for arbitrary Cargo dependencies.
- No new Rust interop target roots or bridge-version changes.
- No converter pipeline or callee-injection design.
- No migration of runtime, resource, async, callback, TLS, HTTP, or retained
  compiler-native stdlib surfaces.

## Implementation Status

| Milestone | Status | Evidence |
| --- | --- | --- |
| M0. Contract and Guardrail Documentation | completed | PR #2812 documented the canonical declaration form and validation boundaries. |
| M1. Lowering Support for Ellipsis Interop Stubs | completed | PR #2813 merged. Lowering implementation and focused tests are complete; Opus M1 pass 2 has no unresolved actionable findings. Validation: `cargo fmt --check`, `cargo test -p sifr_lowering rust_interop`, `cargo test -p sifr -- rust_interop`, `scripts/run_all_tests.sh --profile create-pr` in a clean detached worktree. |
| M2. Effective Sysroot Panic Policy | completed | PR #2814 merged. Effective panic policy implementation and focused tests are complete; Opus M2 pass 1 has no actionable findings. Validation: `cargo fmt --check`, `cargo test -p sifr_driver -- sysroot_interop`, `cargo test -p sifr_driver -- rust_interop_panic`, `cargo test -p sifr_driver -- rust_interop_tests`, `cargo test -p sifr -- sysroot_interop`, `cargo test -p sifr -- rust_interop_panic`, `python3 scripts/check_hir_maintainability_guardrails.py`, `scripts/run_all_tests.sh --profile create-pr`. |
| M3. Codegen and Plan Hardening | completed | PR #2816 merged. Bodyless direct/package-bridge/`Self` codegen and package bridge Cargo/cache metadata are complete; Opus M3 final pass is satisfied. Validation: `cargo fmt --check`, `cargo test -p sifr_driver rust_interop_tests`, `cargo test -p sifr_codegen rust_interop_direct`, `cargo test -p sifr_driver sysroot_interop`, `cargo test -p sifr_driver rust_interop_panic`, `cargo test -p sifr_driver generated_cargo_toml_includes_package_bridge_dependency_alias`, `python3 scripts/check_hir_maintainability_guardrails.py`, `python3 scripts/check_file_size_guardrails.py`, `scripts/run_all_tests.sh --profile create-pr`. |
| M4. Stdlib Source Migration and Executable Guards | completed | PR #2817 merged. Completed private stdlib `sifr_stdlib.*` declarations now use ellipsis-only stubs without explicit `panic=trusted_no_panic`; the stateless private adapter guard prevents drift. Validation: `cargo fmt --check`, focused stateless private adapter/codegen tests, representative migrated-demo checks, guardrails, and `scripts/run_all_tests.sh --profile create-pr`. |
| M5. Closeout Validation and Review | in progress | Final closeout PR #2818 opened; create-pr and merge validation passed; phase-level Opus follow-up review pending. |

## Affected Inventory

Private `sifr_stdlib.*` declarations are currently present in these stdlib
private modules and are in phase scope:

- `stdlib/_sifr/bytes.sifr`
- `stdlib/_sifr/calendar.sifr`
- `stdlib/_sifr/collections.sifr`
- `stdlib/_sifr/compress.sifr`
- `stdlib/_sifr/crypto.sifr`
- `stdlib/_sifr/datetime.sifr`
- `stdlib/_sifr/encoding.sifr`
- `stdlib/_sifr/html.sifr`
- `stdlib/_sifr/i18n.sifr`
- `stdlib/_sifr/json.sifr`
- `stdlib/_sifr/math.sifr`
- `stdlib/_sifr/platform.sifr`
- `stdlib/_sifr/regex.sifr`
- `stdlib/_sifr/toml.sifr`
- `stdlib/_sifr/unicode.sifr`
- `stdlib/_sifr/url.sifr`
- `stdlib/_sifr/uuid.sifr`

Private `_sifr` runtime/resource modules remain out of scope unless they
already target canonical `sifr_stdlib.*` direct declarations and satisfy the
same stateless/data-leaf boundary as the migrated modules.

Compiler surfaces expected to change:

- `crates/sifr_lowering/src/lower/rust_interop.rs`
- `crates/sifr_lowering/src/lower/typing_and_functions/annotations_and_function_lowering.rs`
- `crates/sifr_lowering/src/lower/statements/statement_dispatch.rs`
- `crates/sifr_lowering/src/lower/classes/class_body_lowering.rs`
- `crates/sifr_lowering/src/lower/expressions/core_and_calls.rs`
- `crates/sifr_driver/src/build/rust_interop/panic_validation.rs`
- `crates/sifr_driver/src/build/rust_interop.rs`
- `crates/sifr_driver/src/build/rust_interop_trust.rs`
- `crates/sifr_driver/src/build/sysroot_interop.rs`
- `crates/sifr_codegen/src/function_emitter/generator_bodies.rs`
- `crates/sifr_codegen/src/rust_interop_direct.rs`
- `crates/sifr_codegen/src/rust_interop_bridge_contract.rs`
- `crates/sifr_codegen/src/rust_interop_plan.rs`

Primary tests and guards expected to change:

- `crates/sifr_lowering/src/lower/rust_interop_tests.rs`
- `crates/sifr_driver/src/build/sysroot_interop_tests.rs`
- `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs`
- `crates/sifr_driver/src/build/rust_interop_tests.rs`
- `crates/sifr_driver/src/stdlib/stateless_private_adapter_policy_tests.rs`
- `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs`
- `crates/sifr_codegen/src/function_emitter/tests.rs`
- `crates/sifr_codegen/src/rust_interop_plan_tests.rs`
- `verification/areas/rust_interop/fixtures/**`

## Milestones

### M0. Contract and Guardrail Documentation

Lock the intended model before code movement.

Tasks:

- Document the canonical private stdlib Rust interop declaration form in
  `internal_docs/sifr_sysroot_and_stdlib_architecture.md`.
- Document the package/sysroot panic-surface distinction in
  `internal_docs/rust_interop_architecture.md`.
- State that private `_sifr.*` declarations targeting `sifr_stdlib.*` use a
  compiler-owned effective no-panic policy, recorded in trust metadata.
- State that the declaration body for Rust interop stubs is exactly `...`.
- State that user and package declarations continue to declare their own panic
  surface.
- State in `docs/rust-interop.mdx` that package-authored Rust interop
  declarations use ellipsis-only stub bodies.
- Keep public-facing docs focused on package-authored Rust interop; do not
  expose sysroot shorthand as a user feature.
- Sweep `internal_docs/architecture.md`, `plans/phases/`, and retained stdlib
  ownership documents for stale private-stdlib policy examples.

Acceptance:

- A reviewer can determine which declarations may omit a panic policy without
  reading implementation code.
- A reviewer can determine where ellipsis is legal and where it remains an
  error.
- Documentation describes the stable contract directly, without migration
  history or transitional forms.

Validation:

- Documentation review.
- `rg -n '@rust\(sifr_stdlib\.[^)]*panic=' internal_docs docs
  plans/issues/active/ad-hoc-sysroot-stdlib-interop-declaration-cleanup.md`
  should return no canonical private-stdlib examples.
- `rg -n 'ellipsis-only|effective compiler-owned no-panic|package-authored'
  internal_docs/rust_interop_architecture.md
  internal_docs/sifr_sysroot_and_stdlib_architecture.md docs/rust-interop.mdx`
  should show the durable contract in the architecture and public Rust interop
  docs.
- `rg -n 'panic=trusted_no_panic' docs/rust-interop.mdx
  internal_docs/rust_interop_architecture.md` should show only package-authored
  panic-policy examples.

### M1. Lowering Support for Ellipsis Interop Stubs

Teach lowering to recognize Rust interop declaration stubs before normal body
checking.

Tasks:

- Add a single helper that identifies an exact ellipsis-only function body from
  Ruff AST statements.
- Add a single helper that determines whether a function-like declaration has a
  Rust interop decorator, reusing `collect_rust_interop_declarations` parsing
  where possible to avoid drift.
- Apply the helper consistently to:
  - top-level functions,
  - nested functions,
  - class methods,
  - enum/newtype methods handled by class body lowering.
- For eligible Rust interop stubs, skip normal statement lowering, missing
  return checking, and return-type inference from body expressions.
- In top-level function lowering, bypass both `requires_exhaustive_return_annotation`
  and `infer_function_return_type` for eligible interop stubs because the
  declared return type is authoritative.
- Apply the same authoritative-return-type path to nested functions and class,
  enum, and newtype method lowering.
- Populate `HirFunction.return_type` from the annotation before body lowering
  runs, so ellipsis stubs still expose the `Result[T, E]` and error class
  metadata needed by direct Rust interop mapping.
- Preserve the explicit annotated return type as the HIR return type.
- Emit a targeted diagnostic when ellipsis appears in a non-interop function
  body.
- Emit a targeted diagnostic when a Rust interop stub mixes ellipsis with any
  other statement.
- Add a targeted diagnostic in expression lowering for `Expr::EllipsisLiteral`
  outside this specialized declaration path.

Acceptance:

- `@rust(...)\ndef f(...) -> T: ...` lowers without body diagnostics.
- `def f() -> int: ...` remains rejected.
- `@rust(...)\ndef f() -> int:\n    ...\n    return 1` is rejected.
- Existing non-stub Rust interop declarations continue to lower normally.

Validation:

- Focused lowering unit tests in
  `crates/sifr_lowering/src/lower/rust_interop_tests.rs`.
- `cargo test -p sifr_lowering rust_interop`
- `cargo test -p sifr -- rust_interop`

### M2. Effective Sysroot Panic Policy

Make omitted panic policy in canonical private stdlib declarations resolve to a
compiler-owned effective no-panic policy.

Tasks:

- Add an effective panic policy resolver that receives declaration context,
  resolved target path, package identity, and source origin.
- Return effective no-panic only for private sysroot `_sifr.*` declarations
  targeting direct `sifr_stdlib.*` paths.
- Keep explicit policy parsing unchanged for user/package declarations.
- Make panic validation consume the effective policy.
- Make trust requirement recording consume the effective policy.
- Include the effective trust requirement in the Rust interop plan and cache
  fingerprint.
- Ensure diagnostics distinguish user/package missing policy from invalid
  sysroot target/context.
- Reserve a targeted `SIFR-RUST-PANIC-*` or `SIFR-RUST-TRUST-*` diagnostic for
  private sysroot declarations that omit `panic=` outside canonical
  `sifr_stdlib.*` targets; the diagnostic should state that the sysroot
  no-panic policy applies only to canonical private `sifr_stdlib.*` targets.
- Keep `sifr_runtime.*`, `bridge.*`, `Self`, and arbitrary Cargo roots outside
  the implicit policy boundary.

Acceptance:

- Canonical private `_sifr.*` declarations targeting `sifr_stdlib.*` pass panic
  validation without a decorator-level policy argument.
- The resolved declaration still records a no-panic trust requirement satisfied
  by the sysroot trust policy.
- User/package non-`Result` declarations without a panic policy continue to
  fail with the existing Rust panic contract diagnostic.
- User/package `Result` declarations that do not expose or map
  `RustPanicError` continue to fail unless they declare an accepted panic
  policy.

Validation:

- Unit tests in `crates/sifr_driver/src/build/sysroot_interop_tests.rs`.
- Unit tests in
  `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs`.
- Unit tests in `crates/sifr_driver/src/build/rust_interop_tests.rs`.
- `cargo test -p sifr -- sysroot_interop`
- `cargo test -p sifr -- rust_interop_panic`

### M3. Codegen and Plan Hardening

Ensure bodyless declarations are represented as declarations, not as empty
implementation bodies.

Tasks:

- Audit `direct_rust_function_body`, package-local bridge emission, `Self`
  method emission, and the function body generator for declaration stubs.
- Prevent synthetic `Ok(())` insertion from being appended after a direct Rust
  interop call emitted for a bodyless `Result[None, E]` declaration.
- Implement that guard against the emitted Rust body source, not the original
  HIR body. When `direct_rust_function_body(func)` supplies the function body,
  the `Result[None, E]` tail-insertion path must be suppressed or delegated to
  the direct-body emitter.
- Emit bodyless package-local bridge calls from Rust interop metadata instead of
  relying on lowered Sifr statements.
- Emit bodyless `Self` calls for opaque method declarations from Rust interop
  metadata where that root is already part of the accepted interop contract.
- Reject bodyless declarations whose Rust interop target is not an accepted
  target root before Rust IR lowering.
- Ensure direct-interop error subclass field mapping derives from the annotated
  `Result[T, E]` return type and class metadata, not from a declaration body.
- Ensure Rust interop plan declarations expose enough policy/trust metadata for
  sysroot audits and cache keys.
- Add regression coverage for `Result[None, E]` direct stdlib calls.
- Add regression coverage for message-shaped error mappings such as regex and
  JSON decode errors with ellipsis-only declaration bodies.

Acceptance:

- Bodyless interop declarations generate exactly the Rust call wrapper body
  appropriate for their accepted target root and return type.
- No codegen panic is reachable from a bodyless declaration that lowering
  accepts.
- Unsupported Rust interop targets produce Sifr diagnostics before Rust IR
  lowering.
- Dependency-plan cache fingerprints change when effective trust policy changes.

Validation:

- Focused codegen unit tests for direct interop body generation.
- Focused codegen unit tests for package-local bridge body generation.
- Existing stdlib codegen tests in
  `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs`.
- Snapshot or assertion coverage for `zip_create`/`zip_add_file` style
  `Result[None, E]` declarations proving no duplicate `Ok(())` tail is emitted.
- Snapshot or assertion coverage for `re_find`/`re_replace` style
  `Result[..., RegexError]` declarations proving error fields still map from
  type metadata.
- Representative compile/run checks for `compress` and `json` declarations that
  return `Result[None, E]`.

### M4. Stdlib Source Migration and Executable Guards

Apply the canonical declaration form to migrated stateless/data private stdlib
modules and make drift impossible.

Tasks:

- Update every in-scope `stdlib/_sifr/*.sifr` declaration targeting
  `sifr_stdlib.*` to omit decorator-level panic policy.
- Replace implementation-shaped placeholder bodies in those declarations with
  ellipsis-only stub bodies.
- Keep retained compiler-native declarations and runtime/resource declarations
  unchanged unless they satisfy the phase's direct `sifr_stdlib.*` criteria.
- Strengthen
  `crates/sifr_driver/src/stdlib/stateless_private_adapter_policy_tests.rs`
  with a guard such as
  `completed_private_declarations_use_ellipsis_stub_and_no_panic_policy`, so
  each completed private declaration:
  - targets `sifr_stdlib.*`,
  - has no decorator-level panic policy,
  - uses an ellipsis-only body,
  - contains no placeholder `return` or `raise` body.
- Replace the existing adapter-policy panic assertion with this guard in the
  same change, so the completed-private-declaration invariant is enforced
  throughout the refactor.
- Update sysroot interop tests to use canonical private declaration examples.
- Keep user/package rust interop negative fixtures proving missing policy is
  still rejected.
- Update private stdlib file comments so completed declaration files describe
  their current ownership without incremental migration notes.

Acceptance:

- All completed private stateless/data declarations use the canonical form.
- The adapter policy guard fails on policy arguments or implementation-shaped
  placeholder bodies in completed private declaration files.
- Public stdlib behavior remains unchanged.

Validation:

- `cargo test -p sifr -- stateless_private_adapter_policy`
- `cargo test -p sifr -- stateless_private_codegen`
- `verification/runner/e2e/run_e2e_pass.sh`
- Representative `cargo run -q -p sifr -- check` on migrated stdlib-dependent
  demos.

### M5. Closeout Validation and Review

Close the phase only after local validation and external design review agree.

Tasks:

- Run Opus review on the phase, docs, and implementation.
- Address all actionable review findings or document why they are intentionally
  out of scope.
- Run the authoritative create-pr validation profile.
- Run the file-size guardrail and refactor any touched oversized hand-maintained
  source file by responsibility.
- Update this phase status with completed evidence and PR link after merge.
- Link closeout evidence from the archived sysroot stdlib toolchain phase or
  roadmap entry that owns follow-up cleanup for private stdlib interop.
- Update roadmap/phase tracking if this phase is added to a tracked milestone.

Acceptance:

- Opus review has no unresolved actionable findings.
- Local validation passes.
- The phase document records implementation evidence and merged PR link.

Validation:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh` before merge if the change touches shared compiler
  behavior broadly enough to warrant full gate coverage.

## Review Checklist

- [x] Ellipsis is accepted only for complete Rust interop stubs.
- [x] Ellipsis in a non-interop function body produces a targeted diagnostic.
- [x] Lowering handles top-level, nested, and method declarations consistently.
- [x] User/package missing panic policy remains rejected.
- [x] Private sysroot `sifr_stdlib.*` omitted policy records effective no-panic
      trust metadata.
- [x] `sifr_runtime.*`, `bridge.*`, `Self`, and arbitrary package roots do not
      receive implicit sysroot no-panic policy.
- [x] Bodyless direct interop codegen does not append synthetic `Ok(())`.
- [x] Direct-interop `Result[None, E]` declarations do not double-emit
      `Ok(())`.
- [x] Bodyless package-local bridge and accepted `Self` targets emit from
      metadata, not lowered Sifr statements.
- [x] Unsupported Rust interop targets fail before Rust IR lowering.
- [x] Cache fingerprints change when a declaration uses explicit package panic
      policy versus effective sysroot policy.
- [x] Completed private stdlib declarations are guarded against drift.
- [x] Architecture docs describe only the durable declaration contract.
- [x] Public user docs do not expose sysroot-only shorthand as package syntax.

## Closeout Notes

- PR: final closeout PR #2818.
- Opus review: pass 1 is recorded in
  `plans/reviews/active/ad-hoc-sysroot-stdlib-interop-declaration-cleanup-m5-opus-review-pass-1.md`;
  follow-up review is pending after the pass 1 findings were addressed.
- Opus pass 1 findings: the reviewer blocked closure until the local lint
  cleanup is committed with PR #2818 and this closeout record calls out the
  lint-driven changes that are not purely mechanical. The requested doc
  clarifications are recorded here before the follow-up review pass.
- Create-pr validation: passed on M5 closeout with final rerun of
  `scripts/run_all_tests.sh --profile create-pr`; e2e pass suite completed
  132/132 fixtures, blocking lanes passed, and there were no advisories
  (`wall_time=147.97s`, `cache_hits=44/44`).
- Additional M5 validation: `cargo fmt --check`,
  `cargo clippy --workspace -- -D warnings`,
  `python3 scripts/check_hir_maintainability_guardrails.py`,
  `python3 scripts/check_file_size_guardrails.py`, and
  `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py`.
- M5 lint sweep: current workspace `clippy -D warnings` required cleanup beyond
  the original phase implementation surfaces, touching analysis, LSP,
  package/sysroot helpers, frontend loaders, driver build-report construction,
  interop diagnostics, and bridge-contract helper signatures. Most changes are
  signature/borrowing or formatting cleanup. Two behavior-visible cleanups are
  intentional: a driver package-context invariant now returns an internal
  compiler diagnostic instead of panicking, and the Rust interop probe now uses
  the existing TOML string escaper for path dependencies instead of `Path`
  debug formatting.
- Merge-gate fixes exposed by M5 validation: package Python runtime native-link
  trust now includes the selected CPython framework's versioned link name
  (`python3.13` on the local runner), and the vendored `cc 1.2.63`
  `src/target` checksum files are restored so generated-code TLS builds can
  validate vendored checksums.
- Full validation: passed on M5 closeout with `scripts/run_all_tests.sh`;
  merge e2e completed 651/651 fixtures, blocking lanes passed, and the only
  advisories were warm wall-time budget exceeded and high e2e group skew
  (`wall_time=2222.06s`, `report_signature=ee5e5d44306f270c`).
- Remaining follow-up: none for this phase. Runtime/resource/callback stdlib
  migrations remain owned by
  `plans/issues/active/rust-interop-runtime-ecosystem-certification.md`.
