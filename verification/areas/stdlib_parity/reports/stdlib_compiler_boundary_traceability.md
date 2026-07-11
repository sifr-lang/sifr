# Stdlib/Compiler Boundary Traceability

This report maps each final boundary invariant to executable evidence. The
retained manifest is the only exception ledger; this report does not classify
or authorize compiler ownership.

| Invariant | Executable evidence |
| --- | --- |
| Compiled private source plus typed HIR is the sole callable-signature authority. | `sifr_hir` private-source bootstrap tests and `check_stdlib_native_intrinsic_allowlist.py` exact source-identity comparison. |
| Retained identities agree across source declarations, typed HIR, lowering, dispatch, and manifest ownership. | `check_stdlib_native_intrinsic_allowlist.py` and its negative self-test. |
| User and package source cannot declare `@compiler_intrinsic`. | HIR lowering tests plus `package_source_cannot_declare_compiler_intrinsics`. |
| Retained compiler callables are not first-class values, while former intrinsic names remain ordinary user names. | HIR lowering tests for first-class rejection, imported former names, and local shadowing. |
| Every production public native adapter is reachable or an explicit compiler substrate. | `check_stdlib_native_adapter_reachability.py`, its self-test, and `stdlib_native_adapter_reachability.toml`. |
| Every retained direct dependency feature is required by live typed-intrinsic codegen. | Native-intrinsic allowlist guard exact dependency-feature comparison and orphan-feature self-test. |
| Deleted fallback imports, APIs, crate paths, and schema fields cannot return. | Private-source bootstrap tests, manifest-schema self-test, and native-intrinsic allowlist deleted-token self-tests. |
| Missing private declaration source fails deterministically without recovery. | Stdlib bootstrap missing-source tests and deleted-fallback guard checks. |
| Source-tree and installed sysroots agree on retained/migrated behavior and generated dependency plans. | `sysroot_release:boundary-equivalence` fixture, normalized dependency comparison, and `stdlib_compiler_boundary_dependency_snapshot.json`. |

Core and merge validation invoke these guards and suites through the verification
profile runner; a prose-only update cannot satisfy the invariant.
