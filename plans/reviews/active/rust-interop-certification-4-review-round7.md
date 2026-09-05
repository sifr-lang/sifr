# Rust Interop Certification 4 Review — Round 7

Reviewer: agent (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer verified the round-6 relative glob, named re-export, type-alias,
and multi-thread runtime corrections, plus all runtime, trust, inventory, and
guardrail evidence.

Blocking finding:

- The fail-closed unresolved-glob rule covered only bare
  `Builder`/`Runtime::new*` calls. Glob-visible runtime/task modules, crate
  aliases, and type aliases could still hide multi-thread constructors or
  `task::block_in_place`.

Correction wave:

- Extended unresolved-glob rejection to bare alias constructor calls,
  `runtime::Builder|Runtime::new*`, crate-alias runtime constructor paths, and
  `task::block_in_place`.
- Added regression coverage for runtime/task module, crate, and type aliases
  exported through an intra-crate glob.
- Changed the generated negative bridge to construct through a runtime module
  re-exported by a non-Tokio glob declared after the function.
- Updated internal, public, and fixture documentation to describe the complete
  conservative rule.

Round 8 is required.
