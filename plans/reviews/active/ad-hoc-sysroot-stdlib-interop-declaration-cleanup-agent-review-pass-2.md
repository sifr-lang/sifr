## Review pass 2 — remaining findings

Pass 1 findings are substantively addressed in the phase document and docs (M3's tail-append guard is now anchored on the emitted Rust body, M1 seeds `HirFunction.return_type` before body lowering, M2 reserves a distinct diagnostic for sysroot-only policy misuse against non-`sifr_stdlib` targets, M0's rg checks are concrete, Affected Inventory now enumerates the touched test files, ellipsis is documented as public Rust interop declaration syntax, and the review checklist adds the three previously missing items). The following remains.

## Actionable findings

### 1. M0's own validation rg does not match the doc language it certifies

Phase M0 Validation (`plans/issues/active/ad-hoc-sysroot-stdlib-interop-declaration-cleanup.md:168-172`):
```
rg -n 'ellipsis-only|effective compiler-owned no-panic|package-authored'
   internal_docs/rust_interop_architecture.md
   internal_docs/sifr_sysroot_and_stdlib_architecture.md
   docs/rust-interop.mdx
```

Actual matches today:
- `internal_docs/sifr_sysroot_and_stdlib_architecture.md`: hits `ellipsis-only` (:145, :775), `effective compiler-owned no-panic` (:154). ✓
- `internal_docs/rust_interop_architecture.md`: **no hit** for any of the three terms. It says "exactly one ellipsis statement" (:100) and "package-authored" (:570, :614) — but not the M0 phrase `package-authored` in combination with the ellipsis contract, and no `ellipsis-only`.
- `docs/rust-interop.mdx`: **no hit** for any of the three terms. It only says "The Sifr declaration body is exactly `...`" (:54-55).

This directly contradicts the M0 task "State in `docs/rust-interop.mdx` that package-authored Rust interop declarations use ellipsis-only stub bodies." A reviewer running M0's own validation rg cannot confirm the durable contract is present in the public docs or in `rust_interop_architecture.md`. Either (a) tighten the docs to use the phrases M0 certifies against (add "ellipsis-only stub body" and "package-authored" language in both files), or (b) reword the rg to match the wording the docs actually use. The docs update is preferable because "ellipsis-only" is the distinguishing property against a general Sifr body form.

### 2. M4's adapter-policy guard reverses an existing invariant without flagging the direction change

`crates/sifr_driver/src/stdlib/stateless_private_adapter_policy_tests.rs` today asserts private stateless declarations **carry** `panic=trusted_no_panic`; M4 flips this to a guard that they **must not** carry `panic=` (i.e., must rely on effective sysroot policy) — and additionally must use an ellipsis-only body. The new test-name suggestion (`completed_private_declarations_use_ellipsis_stub_and_no_panic_policy`) implicitly signals the reversal, but the phase does not call out that the direction of an existing invariant is being inverted. This matters for reviewers of the M4 PR: without the callout they may assume the new guard is additive, miss the removal of the prior positive check, and merge a state where neither direction is enforced during the intermediate refactor. Add a one-line M4 note that the prior "must have `panic=trusted_no_panic`" assertion is being replaced (not augmented) so PR review understands the invariant switch.

## Residual non-blocking risks

- **`sifr_sysroot_and_stdlib_architecture.md` Pre-Migration Baseline (§ starting :20)** still uses migration voice and a "current owner → final owner → migration blocker" table shape. It does not frame the declaration cleanup itself as old-vs-new, so it does not violate the user's specific criterion — but it does keep migration framing adjacent to the durable contract. This was flagged in pass 1 as non-blocking polish and remains untouched; leave it or retire it in a broader stdlib-arch cleanup pass, not this ad-hoc.
- **M5 origin link.** The phase doesn't state whether this ad-hoc is a follow-up to the recently archived `19e346f0f Archive sysroot stdlib toolchain phase` or independent, or where its closeout evidence should be linked from `plans/roadmap.md`. Pass 1 noted this; it's still unspecified. Non-blocking, but worth resolving before closeout.
- **`crates/sifr_codegen/src/rust_interop_direct.rs` unit tests** are covered implicitly by listing the source file, but no separate `_tests.rs` module is called out in Affected Inventory. If tests live in the source file, this is fine; if a sibling tests file exists, add it for navigability. (Confirm during M3.)
- **`_sifr.crypto.sifr`** file-comment cleanup is covered by M4's blanket "Update private stdlib file comments…" task but not called out by module. If more private modules retain migration-voice header comments beyond `_sifr.crypto`, the sweep should be explicit about scope so it is not silently limited to one file.
