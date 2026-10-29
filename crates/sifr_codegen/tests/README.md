# Codegen test ownership

`emission_contracts.rs` exercises the public compiler emission API against the
normal `sifr_codegen` library. Its modules live in `contracts/`. Assertion and
embedded-fixture edits can rebuild that test executable while retaining the
production library. Shared source-to-emission helpers live in `support/`.

Private implementation invariants live in `unit/` and remain children of their
original Rust modules through `#[cfg(test)]` and explicit `#[path]` declarations.
They retain private access without adding public compiler APIs for tests. Editing
these tests still recompiles the library's unit-test executable. All test-owned
sources are outside the manifest's production `src` input root; they remain Cargo
inputs of the test targets that use them.

Choose the smallest appropriate check, through the same locked launcher and owned
target used for other local verification:

```sh
uv run --project verification --locked cargo test --locked -p sifr_codegen --test emission_contracts <filter>
uv run --project verification --locked cargo test --locked -p sifr_codegen --lib <filter>
```

The complete package check remains:

```sh
uv run --project verification --locked cargo test --locked -p sifr_codegen
```

It includes both targets, the native `project_slice_abi` tests, and doctests.
Neither focused command substitutes for complete candidate qualification. New
behavior tests should use the public API when it expresses the contract; private
invariant tests should stay with their owning module. Keep fixtures, assertions,
and shared helpers under test ownership, and update executable verification
inventories when relocating their source evidence.
