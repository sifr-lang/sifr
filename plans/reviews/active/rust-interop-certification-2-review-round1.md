# Rust Interop `certification_2` Review — Round 1

Reviewer: Claude Opus 5, medium effort
Verdict: `NEEDS REVISION`

## Verified behavior

The reviewer reproduced the generated release-package path and confirmed that:

- target calls are protected by `sifr_runtime::interop::catch_rust_panic`;
- ordinary bridge errors enter the declared ordinary union member;
- Rust mappers receive a cloned redacted `RustPanicErrorBridge` behind a
  second protected boundary;
- mapper panics rebuild the fallback from the original redacted panic without
  a move or payload leak;
- mapper signature probing occurs in the mapper target's Cargo context; and
- the focused area, inventory, formatting, lint, and guardrail checks were
  otherwise aligned.

## Findings

1. Panic-surface validation used a rendered-type substring while codegen used
   the nominal `RustPanicError` type. A class such as
   `RustPanicErrorish` could therefore pass validation without receiving a
   generated catch wrapper, allowing a private target panic payload to escape.
2. A `Result[T, RustPanicError]`-only channel allowed an ordinary Rust bridge
   error to be converted into `RustPanicError`, violating the rule that this
   type is reserved for generated wrapper failures.
3. The widened catch-all `bridge_error_expr` arm broke ordinary multi-field
   `Error` subclasses by constructing only `message`.
4. The widened alias arm attempted to construct a Rust struct using the alias
   name and failed for ordinary aliased errors.
5. Structurally valid aliases of `E | RustPanicError` were rejected as
   unrepresentable because validation inspected rendered text instead of the
   resolved type.
6. Async `panic=map_error(...)` declarations passed validation even though
   this milestone emits only synchronous wrappers.
7. The tracker recorded 96 matrix self-test cases while the final focused run
   reported 95.
8. The companion unrepresentable-fallback source was not registered as
   structured auxiliary negative evidence and declared an execution-kind
   header inconsistent with its runtime-observed fixture row.

## Required response

Replace rendered-string panic classification with structural contract
metadata, reject unsupported panic-only ordinary-error and async-map-error
surfaces, narrow error mapping without regressing unrelated declarations, and
make all negative evidence and counts mechanically registered and aligned.
