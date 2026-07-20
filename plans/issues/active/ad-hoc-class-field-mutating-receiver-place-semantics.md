# Ad Hoc Issue: Class-Field Mutating Receiver Place Semantics

## Status

Open urgent follow-up identified during the M10 declaration-first Python
interop milestone review. The defect predates M10 and is not introduced by the
buffer implementation, but it affects the same shared-ownership place model.

## Problem

A mutating class method invoked through a class field, such as
`self.helper.bump()`, can lower the field read through the ordinary auto-clone
path. The method then mutates a temporary clone instead of the stored field, so
the source-visible mutation is lost even though the program type-checks.

The fix must establish one place-preserving receiver path for mutating method
calls. It must not add a special-case fallback for one field depth or one class
shape.

## Required scope

- Preserve the original storage place for mutating receivers rooted at
  `self`, mutable borrowed parameters, and supported nested field chains.
- Keep ordinary non-mutating field reads on the existing value/clone path.
- Define and enforce the aliasing rule when the same root is also read by
  another argument in the call.
- Define the supported boundary for indexed/sliced receiver places and reject
  unproved mutable places at check time instead of generating invalid or
  silently detached Rust.
- Keep receiver mutability inference, HIR ownership effects, codegen borrow
  emission, and diagnostics derived from the same convention metadata.

## Acceptance criteria

- A positive native regression proves that `self.helper.bump()` changes the
  stored helper and that a subsequent read observes the mutation.
- Positive coverage includes a supported nested field receiver and a mutable
  borrowed-parameter field receiver.
- Negative coverage rejects immutable roots, conflicting same-root reads, and
  unsupported indexed/sliced mutable receivers with stable diagnostics.
- Emitted Rust for accepted receivers contains no clone between the storage
  root and the mutating call.
- Focused lowering/codegen tests, E2E pass/fail fixtures, file-size and HIR
  guardrails, and the authoritative local validation gate pass.
- Repeated independent review confirms that no silent clone or alternate
  mutation path remains.
