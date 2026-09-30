# Method dispatch and unsafe ABI policy

H02h records reviewed sites in `method_dispatch_sites.json` and
`unsafe_abi_sites.json`. The guards discover first-party Rust files below
`crates/`, including tests and platform-specific files, without Cargo, a Git
index, or a prepared target. Native bundled inputs below `third_party`/`vendor`
and build outputs below `target` belong to their own owners.

Run the guards and their negative fixtures:

```sh
python3 scripts/check_method_dispatch_authority.py --self-test
python3 scripts/check_method_dispatch_authority.py
python3 scripts/check_unsafe_abi_contracts.py --self-test
python3 scripts/check_unsafe_abi_contracts.py
```

The existing verification profile runner invokes both commands for each guard
in create-pr, merge, nightly and release profiles. These checks supplement
executable H02 ABI and method tests. A static inventory cannot prove runtime
safety, foreign allocation validity or the semantics of arbitrary Rust programs.

## Dispatch ownership

The scanner inventories literal pattern matches, equality/inequality selectors,
`matches!` selectors, method-call carriers and typed authority matches. Literal
selectors are discovered regardless of their binding name. The deliberately
conservative scan also records host protocols and test assertions; they are
classified individually instead of being exempted by a per-file count.

| classification | authority |
| --- | --- |
| typed-lowering | `sifr_lowering` resolves declarations/types and publishes classified HIR |
| compile-time-semantics | `sifr_frontend::const_evaluator::eval_method_call` owns the closed pure subset |
| language-emission | `sifr_codegen::methods::lower_method_impl` owns the sole builtin/intrinsic runtime registry |
| user-protocol-dispatch | the resolved source declaration/protocol owns its call |
| contextual-rust-adaptation | codegen adapts previously selected contextual Rust protocols |
| rust-ir-consumption | codegen consumes already selected Rust operations |
| declaration-resolution | the frontend/type/editor product resolves facts and declarations |
| test-assertion | the containing test owns the assertion |
| package-protocol | the containing package owns its closed host protocol |
| sql-protocol / cache-protocol / driver-protocol | the existing SQL/cache/driver owner retains the site |
| generated-code | X02 retains generated Rust safety |

Only the exact canonical emission function can have `language-emission`
classification. The constant evaluator likewise has one exact classification
owner. Neither authority can be silently reclassified. Every other discovered
site requires an explicit owner, classification and local semantic contract.
Review must check that a classification describes the actual site: the scanner
cannot distinguish a maliciously mislabelled implementation of builtin semantics
from contextual Rust adaptation by interpreting its Rust body.

## Unsafe ownership

Each operation, unsafe ABI declaration, unsafe trait implementation, allowance
and emitted Rust string has a separate record. Its contract names its local
scope and thread, lifetime, alias and ownership obligations. Nearby source
comments are retained as review evidence when present. The reviewed H02 owners
are callbacks (H02d0), CPython core (H02d1), buffers (H02e), Arrow (H02f) and
DLPack (H02g). Other Python bridges keep their existing owner.

SQL FFI/WIT, cache storage, driver OS operations and emitted Rust remain assigned
to SQL, cache, driver and X02 respectively. Their inventory records describe the
required local obligations and ownership; they do not claim that H02h audited
or executed those owners' complete safety contracts. The runtime contract tests
remain the evidence for the repaired H02 bridges.

Allowances can cover a function, a particular unsafe Send/Sync implementation,
an expression, an explicitly registered cohesive ABI module, or a test module
with an adjacent `cfg(test)` guard. Ordinary impl/struct/enum/type/trait and
arbitrary file/module allowances are rejected. Marking a record as cohesive
cannot authorize an additional production module.

## Updating a site

The key is path, enclosing function, site kind and ordinal within that scope.
The SHA-256 fingerprint covers the site's complete lexical span, including
literal contents, without whitespace or comments. It does not use line numbers
or per-file counts. Added/removed sites fail as unclassified/stale, and edits or
binding renames fail as changed fingerprints. Duplicate records fail too.

When an intended change fails a guard, review the changed site against its owner
and executable contracts, then update that site's record. Do not regenerate the
inventory to conceal failures or relabel a new builtin registry as adaptation.
A moved/renamed site retires its old record and adds its reviewed replacement.

The lexer separately recognizes escaped/byte/Unicode characters, ordinary and
raw Rust strings (including byte/C strings), nested block comments and line
comments. Apostrophes introducing lifetimes and labels remain code. The cold
checkout self-tests cover renamed dispatch, new/stale/changed fingerprints,
second owners, broad allowances, missing local contracts, character masking and
lifetime-visible unsafe operations; they do not read repository inventories.
