I've reviewed the diff, adjacent code (guard flow, `Self`-branch resolution, opaque contract collection, trust plumbing), and both the phase plan and the tests. Summary below.

## What the change does

`rust_interop.rs:284-298` — the private sysroot target-root guard now skips paths whose first segment is `"Self"`, so `Self.*` in a private `_sifr.*` method flows down to the existing `Self` branch at `rust_interop.rs:318-362`. That branch resolves `Self` to `SelfMethod { class_name }` only when the declaration's owner is `RustInteropOwner::Method` and the class is in `opaque_contracts`; otherwise it emits the "@rust.opaque" diagnostic. Non-`Self` roots in private sysroot modules still must be `sifr_runtime`/`sifr_stdlib`.

## Correctness

- The guard bypass is exactly `root == "Self"`. Every other non-canonical root still hits `SIFR-RUST-RESOLVE-0001`, e.g. `type=native.io.FileHandle` (test `sysroot_private_opaque_interop_rejects_non_sysroot_rust_type`).
- `Self.*` outside an opaque class still fails: after the guard is skipped, the `Self` match arm returns `SIFR-RUST-RESOLVE-0001` with the opaque guidance. The private-sysroot escape hatch is not a "trust everything named `Self`" hole — it just defers to the existing branch that already knew how to handle `Self`.
- `type=X` in an opaque class flows through `declaration_paths` (via `TargetPath` arguments), so it goes through the same guard and the non-sysroot rejection test confirms it.
- Trust: `canonical_trust_target_path` returns `"Self.close"`, and `is_trusted_sysroot_package` grants trust automatically for the sysroot package — the test asserts `required_entry == "Self.close" && trusted`, which matches the code at `rust_interop.rs:666-667`.
- Ordering: The SelfMethod resolution requires the opaque class declaration to be processed before its methods (so `opaque_contracts` is populated). This is a pre-existing invariant — `package_rust_interop_resolves_self_method_root` already relies on it — not introduced by this PR.

## Scope

Narrow and appropriate for M2b's "opaque resources … close/aclose lifecycle metadata" scope. One-line real change, ~180 lines of test/helper additions. File sizes (885 / 725) sit below the 900-line guardrail.

## Tests

Positive and negative sysroot paths are both covered:
- Positive: `sysroot_private_opaque_interop_resolves_self_close_method` asserts (a) the class's `type=sifr_stdlib.io.FileHandle` resolves to `SysrootCrate`, (b) the method's `Self.close` resolves to `SelfMethod { class_name: "FileHandle" }`, and (c) trust is granted.
- Negative: `sysroot_private_opaque_interop_rejects_non_sysroot_rust_type` confirms `type=native.io.FileHandle` still fails with the canonical-sysroot diagnostic.

Nice-to-have gap (not blocking): no sysroot-side test proves that a private `_sifr.*` method with `Self.X` on a **non-opaque** class produces the `@rust.opaque` diagnostic rather than the canonical-crate one. The behavior is correct by construction (the `Self` branch handles it), but an explicit assertion would make the guard-vs-Self-branch layering regression-proof. `package_rust_interop_rejects_self_method_root_without_opaque_class` covers this for the non-sysroot path.

## Safety to merge

Purely additive guard exception with clear tests. No behavior change for existing `_sifr.*` declarations. No ecosystem impact. Aligned with the M2b sub-PR breakdown in the phase plan.

READY
