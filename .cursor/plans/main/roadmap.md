# Sifr Compiler -- Roadmap

## Phase Summary

| # | Phase | Milestones | Status | What it unlocks |
|---|-------|-----------|--------|-----------------|
| 1 | Language Foundations | 6 (ergonomics → codegen_quality) | completed | Single-file programs with classes, error handling, safe indexing, imports |
| 2 | Type System Power | 6 (protocols → codegen_quality_v2) | completed | Generics, closures, generators, decorators, operator overloading |
| 3 | Standard Library | 4 (core_stdlib → codegen_quality_v3) | completed | 13 stdlib modules, test runner, extended collections |
| 4 | Language Hardening | 19 (codegen_fixes → ownership_v3) | completed | ~60% LeetCode compiles, nested functions, generics, forward refs |
| 5 | Borrow-by-Default | 2 (borrow_default, borrow_hardening) | superseded | Original plan moved to Phase 10 after safety audit findings |
| 6 | Stdlib Architecture | 6 (intrinsics → stdlib_classes) | completed | Stdlib rewritten as .sifr files, 37+ modules, class-in-stdlib pipeline |
| 7 | Stdlib Parity | 7 (compiler_hardening → cpython_tests) | completed | Import errors, `with` protocol, `Callable` fix, lazy iterators, generic stdlib, ~50 new functions, CPython-aligned names, 6 new classes, `datetime` operator overloading, ~500 CPython test assertions |
| 8 | Error Safety | 2 (error_safety, error_safety_stdlib_types) | completed | Built-in error classes, exhaustiveness checking on `except` arms, `Result[T, str]` eliminated, module-specific error type export pipeline |
| 9 | Stdlib Safety Remediation | 6 (io_safety → zero_panic_gate → error_subclasses) | completed | All ~45+ `.unwrap()` panic paths fixed, zero-panic gate enforced, safety scores 7/10+ per module, error subclass hierarchy (FileNotFoundError etc.) for compile-time checked fine-grained error handling |
| 10 | Borrow-by-Default | 3 (borrow_default, borrow_hardening, borrow_stdlib) | completed | Borrow-by-default params, exclusivity, escape analysis, consuming-self, for-loop semantics, stdlib ownership patterns |
| 11 | Stdlib Deepening | 4 (pure_expansion → class_deepening) | completed | ~38% → deep CPython parity, 8 new modules, `datetime`/`deque`/`Pattern` classes, API naming divergences documented |
| 12 | Stdlib Remediation | 1 (stdlib_remediation) | pending | `open()` built-in (text + binary modes), `datetime.time`/`timezone`, `CompletedProcess`, `Path.glob`, `re` flags, minor gaps |
| 13 | Type System Completion | 6 (auto_init → stdlib_generic_rewrite) | pending | Auto-init, user-facing generics, pattern matching, enums, bigint, generic stdlib |
| 14 | Codegen Architecture | 6 (rust_ir_types → codegen_structural_passes) | pending | Structured Rust IR, pretty-printer, preamble/stmt/expr/intrinsic migration, dead-code elimination, clone optimization |
| 15 | Async and Ecosystem Foundation | 5 (async_core → async_advanced) | pending | Async runtime, typed serde, networking stdlib, sync primitives, async generators |
| 16 | Web Stack | 6 (web_framework → web_services) | pending | Web framework, database, typed extractors, auth, production features, external services |
| 17 | Interoperability | 1 (ffi) | pending | Rust FFI, C FFI, unsafe boundary |
| 18 | Package Management | 1 (package_mgmt) | pending | sifr.toml, sifr.lock, PubGrub solver, dependency resolution |
| 19 | Developer Tools | 1 (dev_tooling) | pending | LSP, formatter, linter, doc generator |
| 20 | Data Science and ML | 2 (data_processing, ml_inference) | pending | Polars DataFrames, ML inference, LLM integration |
| 21 | Ecosystem | 2 (metaprogramming, ecosystem) | pending | Compile-time decorators, package registry, incremental compilation, REPL |

## Ordering Rationale

1. **Foundations → Type System Power**: Classes and error handling must exist before protocols
   and generics can define trait bounds and typed error hierarchies.

2. **Type System Power → Standard Library**: The full type system (generics, closures, decorators)
   is needed to design stdlib APIs with proper type signatures.

3. **Standard Library → Language Hardening**: Real-world usage (396 LeetCode problems + stdlib)
   exposed systemic gaps in codegen, narrowing, ownership, and iteration.

4. **Language Hardening → Stdlib Architecture**: The language must be fully stable before rewriting
   the stdlib as `.sifr` files using the three-tier hybrid architecture.

5. **Stdlib Architecture → Stdlib Parity**: The three-tier architecture and class-in-stdlib
   pipeline must be proven before expanding the stdlib surface. Parity fixes compiler gaps
   (import errors, `with` protocol), adds test infrastructure, expands functions, aligns
   naming with CPython, rolls out 6 new classes, and ports ~500 CPython test assertions.

6. **Stdlib Parity → Error Safety**: The error handling model (`Result[T, E]` where `E` extends
   `Error`, exhaustiveness checking on `except` arms) must be enforced by the compiler before
   stdlib functions can be migrated from panicking intrinsics to `Result`-returning functions.

7. **Error Safety → Stdlib Safety Remediation**: You cannot make intrinsics return `Result[T, IOError]`
   if the compiler doesn't enforce that `IOError` extends `Error`, doesn't do exhaustiveness
   checking, and all existing tests use `Result[T, str]`. The zero-panic gate at the end ensures
   no safety regressions leak into subsequent phases.

8. **Stdlib Safety Remediation → Borrow-by-Default**: Both touch the same codegen paths (`stdlib.rs`,
   `lib.rs`). Fixing safety first means borrow-by-default works on non-panicking code. The
   zero-panic gate ensures the foundation is solid. Error subclasses (`FileNotFoundError`, etc.)
   then refine the error types into a CPython-aligned hierarchy with compile-time exhaustiveness
   checking at sub-error granularity, before the parameter passing convention changes.

9. **Borrow-by-Default → Stdlib Deepening**: New stdlib functions should be written with the final
   ownership model from day one. Writing 50+ new functions with move-by-default and then
   retrofitting `mut`/`own` is wasteful.

10. **Stdlib Deepening → Stdlib Remediation**: Phase 11 gap analysis revealed unfinished items
    (`open()` built-in, `datetime.time`/`timezone`, `subprocess.CompletedProcess`, `Path.glob`,
    `re` flags). These must be closed before the type system completion phase rewrites the stdlib
    with generics — the generic rewrite should operate on a complete stdlib, not patch gaps
    simultaneously. Binary file modes (`"rb"`, `"wb"`) are included because Phase 20 (data
    processing with Parquet I/O) and Phase 16 (crypto with AES encryption) need them.

11. **Stdlib Remediation → Type System Completion**: The type system has critical gaps: incomplete
    user-facing generics (field/method substitution), monomorphic stdlib, no pattern matching,
    no enum type, no auto-generated constructors, and integer overflow contradicts the safety
    guarantee. Fixing these before async means the async runtime, typed serde, and web extractors
    all benefit from generics, pattern matching, and clean class definitions from day one. The
    `bigint` type resolves the integer overflow contradiction. The stdlib generic rewrite at the
    end of this phase eliminates all type-specific function duplicates.

12. **Type System Completion → Codegen Architecture**: The codegen is the single largest crate
    (9,805 lines) and is entirely string-based — every Rust construct is emitted via
    `self.write("...")` with no intermediate representation. This causes: no compile-time
    validation of generated code, manual indentation tracking, heuristic clone insertion via
    temporal-coupling boolean flags, a string-parsing dead-code eliminator, and 34 Clippy
    suppressions. Every subsequent phase (async, web, FFI) will add hundreds of new intrinsics
    and codegen patterns. Introducing a structured Rust IR now means all future codegen is
    built on a sound foundation. The type system must be complete first so the IR covers all
    type constructs (generics, enums, pattern matching, bigint).

13. **Codegen Architecture → Async and Ecosystem Foundation**: The async runtime and web framework
    will use generic stdlib functions, pattern matching for error handling, and enum types for
    state machines. Having a complete type system, generic stdlib, and structured codegen means
    fewer surprises when building async features on top. New async codegen patterns (async fn,
    .await, tokio runtime) are built on structured IR from day one. The async phase is split
    into 5 milestones: async core (minimum viable async), typed serde (web-independent, placed
    before networking so the HTTP client can return typed responses from day one), networking
    stdlib, sync primitives (Lock/Channel/Semaphore + Send/Sync checking), and advanced async
    (async with, generators).

14. **Async and Ecosystem Foundation → Web Stack**: The web framework requires async I/O. Web-specific
    extractors (`Json[T]`, `Path[T]`, etc.) depend on both the web framework and typed serde core.
    The web stack splits web framework and database into separate milestones — database access
    is independent and can be used by CLI tools without a web framework.

15. **Web Stack → Interoperability**: FFI formalizes what the intrinsic system already does — wrapping
    Rust crates. Placing it after the web stack means the language is feature-complete and stable.
    FFI is its own phase because it's a fundamental capability (the "escape hatch" for the entire
    Rust ecosystem), not polish.

16. **Interoperability → Package Management**: Package management needs to handle both Sifr packages
    and Rust crate dependencies (via FFI). Having FFI available means the package manager can
    properly resolve and build mixed Sifr/Rust dependency graphs.

17. **Package Management → Developer Tools**: The LSP needs to understand project structure from
    `sifr.toml`. The formatter and linter benefit from a stable language surface. Developer tools
    are a dedicated phase because they are substantial engineering efforts that deserve focused
    attention.

18. **Developer Tools → Data Science and ML**: Data processing and ML inference are independent of
    the web stack — they depend only on typed serde and the async runtime (both from Phase 15).
    Placing them after developer tools means the IDE support is available for data science
    workflows. This phase is separate from the web stack because data science is a distinct
    use case.

19. **Data Science and ML → Ecosystem**: The ecosystem phase (registry, incremental compilation,
    REPL, metaprogramming) is the capstone that turns Sifr from a language into a platform.
    It comes last because it benefits from every preceding phase being complete and stable.

Per-milestone ordering rationale is documented within each phase file in `phases/`.

## Milestone Dependency Diagram

```mermaid
flowchart TD
    subgraph done [Completed]
        milestone_core_language["milestone_core_language: Core Language\nVariables, functions, if/else,\nprimitives, print, CLI"]
        milestone_control_flow["milestone_control_flow: Control Flow + Data\nLoops, list, dict, tuple,\nstring ops, indexing"]
        milestone_type_system["milestone_type_system: Advanced Type System\nUnion types, literal types,\ntype narrowing, Unknown"]
    end
    subgraph phase1 [Language Foundations]
        milestone_ergonomics["milestone_ergonomics: Language Ergonomics\nTernary, kwargs, augmented assign,\nmethods, slicing, walrus"]
        milestone_classes["milestone_classes: Basic Classes\nstruct + impl, __init__,\nmethods, auto-derive"]
        milestone_error_handling["milestone_error_handling: Error Handling\nResult/Option, try/except,\ntyped errors, compiler auto-unwrap"]
        milestone_safe_indexing["milestone_safe_indexing: Safe Indexing\nOption returns, del,\nfallible methods"]
        milestone_imports["milestone_imports: Multi-file + Imports\nimport/from, visibility,\ncircular detection"]
        milestone_codegen_quality["milestone_codegen_quality: Codegen Quality\nRemove unnecessary mut,\nidiomatic println/format,\nclean string/HashMap emit"]
    end
    subgraph phase2 [Type System Power]
        milestone_protocols["milestone_protocols: Protocols + Operators\nTraits, operator overload,\ndiscriminated unions, patterns"]
        milestone_inheritance["milestone_inheritance: Inheritance\nsuper, classmethod,\nstaticmethod, property"]
        milestone_generics["milestone_generics: Generics + Closures\nType params, lambdas,\ncomprehensions, iterators"]
        milestone_generators["milestone_generators: Generators + With\nyield, yield from,\ncontext managers"]
        milestone_decorators["milestone_decorators: Decorators + Variadics\nFunction wrapping,\n*args/**kwargs"]
    end
    subgraph phase3 [Standard Library]
        milestone_core_stdlib["milestone_core_stdlib: Core Stdlib\nI/O, JSON, env, os,\ntoml, collections, open"]
        milestone_test_runner["milestone_test_runner: Test Runner\nsifr test, assertions,\ndiscovery, parallel"]
        milestone_ext_collections["milestone_ext_collections: Extended Collections\nfrozenset, Counter,\ndefaultdict, bytes"]
        milestone_ext_stdlib["milestone_ext_stdlib: Extended Stdlib\nmath, time, random, regex,\nhashlib, base64, stream, logging"]
        milestone_codegen_quality_v3["milestone_codegen_quality_v3: Codegen Quality v3\nPost-stdlib codegen cleanup"]
    end
    subgraph phaseHardening [Language Hardening]
        milestone_codegen_fixes["milestone_codegen_fixes: Codegen Fixes\nTuple indexing, union returns,\nint/int, print None, escapes"]
        milestone_narrowing_v2["milestone_narrowing_v2: Narrowing v2\nElif chains, early-return,\nand-narrowing, 3+ unions"]
        milestone_ownership_v2["milestone_ownership_v2: Ownership v2\nAuto-borrow for print,\nstop consuming values"]
        milestone_subscript_mutation["milestone_subscript_mutation: Subscript Mutation\nlist[i]=val, dict[key]=val,\nself.field += 1"]
        milestone_iteration_v2["milestone_iteration_v2: Iteration v2\nString/dict iteration,\ntuple unpack in for, dict comp"]
        milestone_builtins_v2["milestone_builtins_v2: Builtins v2\nmax/min 2-arg, range 3-arg,\nmixed arithmetic, module vars"]
        milestone_syntax_expansion["milestone_syntax_expansion: Syntax Expansion\nNested functions, closures,\nbitwise ops, multi-assign"]
        milestone_recursive_types["milestone_recursive_types: Recursive Types\nListNode, TreeNode,\nBox for self-referential"]
        milestone_inference_v2["milestone_inference_v2: Inference v2\nReturn type inference,\nparam inference, Result unwrap"]
        milestone_stdlib_hardening["milestone_stdlib_hardening: Stdlib Hardening\nset type, import aliases,\nmath/json/io/env gaps"]
        milestone_nested_functions["milestone_nested_functions: Nested Functions\ndef-inside-def, closures,\ncapture variables, recursive"]
        milestone_forward_refs["milestone_forward_refs: Forward Refs\nTwo-pass class registration,\nListNode, TreeNode, Node"]
        milestone_narrowing_v3["milestone_narrowing_v3: Narrowing v3\nEquality narrowing, field access,\nunion comparison, truthiness"]
        milestone_union_ops["milestone_union_ops: Union Ops\nArithmetic on T|None,\ndict.get, list concat"]
        milestone_subscript_v2["milestone_subscript_v2: Subscript v2\nNested subscript assign,\n&mut self, mutability"]
        milestone_comprehension_v2["milestone_comprehension_v2: Comprehension v2\nRange in comp, dict/set comp,\ntuple unpack in for"]
        milestone_generics_impl["milestone_generics_impl: Generics Impl\nTypeVar, generic fn/class,\nCallable, protocol bounds"]
        milestone_phase_fixes["milestone_phase_fixes: Phase Fixes\nProtocol dispatch, ctx mgr,\ncls calls, stdlib gaps"]
    end
    subgraph phaseStdlibArch [Stdlib Architecture]
        milestone_intrinsics["milestone_intrinsics: Intrinsics Layer\n_sifr.* primitives, .sifr embedding,\ntwo-phase compilation"]
        milestone_stdlib_migration["milestone_stdlib_migration: Stdlib Migration\nPort 13 modules to .sifr,\ndelete emit_stdlib_call"]
        milestone_stdlib_expansion["milestone_stdlib_expansion: Stdlib Expansion\n~14 new modules: algorithms,\nCLI, file utilities"]
        milestone_stdlib_parity_audit["milestone_stdlib_parity: Stdlib Parity\nGap closing, remaining modules,\nparity audit"]
        milestone_stdlib_polish["milestone_stdlib_polish: Stdlib Polish\nAPI alignment, perf_counter/monotonic,\ntimeit API, test coverage"]
        milestone_stdlib_classes["milestone_stdlib_classes: Stdlib Classes\ncollections.Counter class,\nclass-in-stdlib pipeline"]
    end
    subgraph phaseStdlibParity [Stdlib Parity]
        milestone_compiler_hardening["milestone_compiler_hardening: Compiler Hardening\nImport errors, with protocol,\nCallable-as-struct-field fix"]
        milestone_lazy_iterators["milestone_lazy_iterators: Lazy Iterators\nState machine codegen,\nIterator trait, lazy yield"]
        milestone_test_infra["milestone_test_infra: Test Infrastructure\nassert_almost_eq, assert_gt/lt,\nvariance bug fix"]
        milestone_stdlib_functions["milestone_stdlib_functions: Stdlib Functions\n~25 pure-Sifr + ~12 intrinsics,\ngeneric bisect/heapq/itertools"]
        milestone_stdlib_naming["milestone_stdlib_naming: Stdlib Naming\nCPython-compatible names,\nr# keyword handling"]
        milestone_stdlib_class_rollout["milestone_stdlib_class_rollout: Stdlib Class Rollout\nPath, Logger, Match, UUID,\nTopologicalSorter, datetime"]
        milestone_cpython_tests["milestone_cpython_tests: CPython Tests\n~500 assertions ported,\nbehavioral validation"]
    end
    subgraph phaseErrorSafety [Error Safety]
        milestone_error_safety["milestone_error_safety: Error Classes\nBuilt-in error types,\nexhaustiveness checking,\nResult[T,str] eliminated"]
        milestone_error_safety_stdlib_types["milestone_error_safety_stdlib_types:\nModule-specific error types\n(StatisticsError, CycleError)"]
    end
    subgraph phaseSafetyRemediation [Stdlib Safety Remediation]
        milestone_io_safety["milestone_io_safety: I/O Safety\nFile ops return Result[T, IOError],\nno .unwrap() panics"]
        milestone_parse_safety["milestone_parse_safety: Parse Safety\nJSON/TOML/regex return Result,\nspecific error types"]
        milestone_collection_safety["milestone_collection_safety: Collection Safety\nEmpty collection handling,\nmath domain errors"]
        milestone_edge_case_safety["milestone_edge_case_safety: Edge Cases\nInput validation, bounds checks,\ncycle detection"]
        milestone_zero_panic_gate["milestone_zero_panic_gate: Zero Panic Gate\nCI lint, safety audit 7/10+,\ncomprehensive E2E test"]
        milestone_error_subclasses["milestone_error_subclasses: Error Subclasses\nFileNotFoundError, PermissionError,\nenum variants, inheritance subtyping"]
    end
    subgraph phaseBorrow [Borrow-by-Default]
        milestone_borrow_default["milestone_borrow_default: Borrow Default\nParamConvention enum,\nmut/own syntax, codegen"]
        milestone_borrow_hardening["milestone_borrow_hardening: Borrow Hardening\nExclusivity, escape analysis,\nconsuming-self, for-loop semantics"]
        milestone_borrow_stdlib["milestone_borrow_stdlib: Stdlib Ownership\nheapq mut, bisect mut,\ngenerator+borrow fix"]
    end
    subgraph phaseStdlibDeepening [Stdlib Deepening]
        milestone_stdlib_pure_expansion["milestone_stdlib_pure_expansion: Pure Expansion\nmath/stats/random/functools/\nitertools additions, cleanup"]
        milestone_new_modules["milestone_new_modules: New Modules\nsubprocess, sys, html,\ngzip, zipfile, calendar, operator"]
        milestone_stdlib_intrinsic_expansion["milestone_stdlib_intrinsic_expansion: Intrinsic Expansion\nmath/os/hashlib/platform/\ntime/base64/shutil additions"]
        milestone_stdlib_class_deepening["milestone_stdlib_class_deepening: Class Deepening\nopen(), deque, datetime class,\nPath, Pattern, csv, logging"]
    end
    subgraph phaseStdlibRemediation [Stdlib Remediation]
        milestone_stdlib_remediation["milestone_stdlib_remediation: Gap Closure\nopen() built-in (text+binary),\ndatetime.time, CompletedProcess,\nPath.glob, re flags"]
    end
    subgraph phaseTypeSystem [Type System Completion]
        milestone_auto_init["milestone_auto_init: Auto Constructors\nAuto __init__/__eq__/__str__,\nfield defaults, inheritance"]
        milestone_generics_v2["milestone_generics_v2: Generics v2\nGeneric class substitution,\nbounds, inference, None standalone"]
        milestone_pattern_matching["milestone_pattern_matching: Pattern Matching\nmatch/case, exhaustiveness,\nclass/union/literal patterns"]
        milestone_enums["milestone_enums: Enum Types\nSimple enums, valued enums,\nmethods, exhaustive match"]
        milestone_integer_safety["milestone_integer_safety: Integer Safety\nbigint type, overflow warnings,\narbitrary-precision arithmetic"]
        milestone_stdlib_generic_rewrite["milestone_stdlib_generic_rewrite: Generic Stdlib\nitertools/functools/collections/\nheapq generic rewrite"]
    end
    subgraph phaseCodegenArch [Codegen Architecture]
        milestone_rust_ir_types["milestone_rust_ir_types: Rust IR Types\nRustExpr, RustStmt, RustItem,\nRustType, RawCode escape hatch"]
        milestone_rust_ir_renderer["milestone_rust_ir_renderer: IR Renderer\nPretty-printer, indentation,\nRawCode passthrough"]
        milestone_codegen_preamble_migration["milestone_codegen_preamble_migration: Preamble Migration\nError types, FileHandle,\nlogging, imports via IR"]
        milestone_codegen_stmt_expr_migration["milestone_codegen_stmt_expr_migration: Stmt/Expr Migration\nlower_stmt, lower_expr,\neliminate temporal flags"]
        milestone_codegen_intrinsic_migration["milestone_codegen_intrinsic_migration: Intrinsic Migration\n~80 intrinsics, ~50 methods,\nstructured IR bodies"]
        milestone_codegen_structural_passes["milestone_codegen_structural_passes: Structural Passes\nImport collection, dead-code\nelim, clone optimization"]
    end
    subgraph phaseAsync [Async and Ecosystem]
        milestone_async_core["milestone_async_core: Async Core\nasync/await, tokio,\ntask spawn/sleep/timeout"]
        milestone_typed_serde_core["milestone_typed_serde_core: Typed Serde\nAuto serde, dumps/loads,\nweb-independent"]
        milestone_networking_stdlib["milestone_networking_stdlib: Networking Stdlib\nsocket, http, subprocess async,\nurl parsing"]
        milestone_async_sync["milestone_async_sync: Async Sync\nLock, Channel, Semaphore,\nSend/Sync checking"]
        milestone_async_advanced["milestone_async_advanced: Async Advanced\nasync with, async generators,\nasync comprehensions"]
    end
    subgraph phaseWebStack [Web Stack]
        milestone_web_framework["milestone_web_framework: Web Framework\naxum routing, middleware,\ndecorators, shutdown"]
        milestone_database["milestone_database: Database\nSQLite, sqlx, pools,\ntransactions, migrations"]
        milestone_typed_web_extractors["milestone_typed_web_extractors: Web Extractors\nJson/Path/Query/Form,\nfile uploads, 422"]
        milestone_crypto_auth["milestone_crypto_auth: Crypto + Auth\nArgon2, JWT, AES-GCM, HMAC"]
        milestone_web_production["milestone_web_production: Production Web\nJSON logging, tracing,\nrate limiting, CORS"]
        milestone_web_services["milestone_web_services: External Services\nRedis, S3, email"]
    end
    subgraph phaseInterop [Interoperability]
        milestone_ffi["milestone_ffi: FFI + Interop\nRust FFI, C FFI,\nunsafe boundary"]
    end
    subgraph phasePackageMgmt [Package Management]
        milestone_package_mgmt["milestone_package_mgmt: Package Management\nsifr.toml, sifr.lock,\nPubGrub solver"]
    end
    subgraph phaseDevTools [Developer Tools]
        milestone_dev_tooling["milestone_dev_tooling: Developer Tooling\nLSP, formatter, linter,\ndoc generator"]
    end
    subgraph phaseDataML [Data Science and ML]
        milestone_data_processing["milestone_data_processing: Data Processing\npolars DataFrames,\nCSV/Parquet"]
        milestone_ml_inference["milestone_ml_inference: ML + Inference\nModel inference, tensors,\nLLM integration"]
    end
    subgraph phaseEcosystem [Ecosystem]
        milestone_metaprogramming["milestone_metaprogramming: Metaprogramming\nCompile-time decorators,\ndataclass, const eval"]
        milestone_ecosystem["milestone_ecosystem: Package Ecosystem\nRegistry, incremental\ncompilation, REPL"]
    end
    milestone_core_language --> milestone_control_flow --> milestone_type_system
    milestone_type_system --> milestone_ergonomics --> milestone_classes --> milestone_error_handling --> milestone_safe_indexing
    milestone_safe_indexing --> milestone_imports --> milestone_codegen_quality --> milestone_protocols
    milestone_protocols --> milestone_inheritance --> milestone_generics
    milestone_generics --> milestone_generators --> milestone_decorators --> milestone_core_stdlib
    milestone_core_stdlib --> milestone_test_runner --> milestone_ext_collections --> milestone_ext_stdlib
    milestone_ext_stdlib --> milestone_codegen_quality_v3 --> milestone_codegen_fixes
    milestone_codegen_fixes --> milestone_narrowing_v2 --> milestone_ownership_v2 --> milestone_subscript_mutation
    milestone_subscript_mutation --> milestone_iteration_v2 --> milestone_builtins_v2 --> milestone_syntax_expansion
    milestone_syntax_expansion --> milestone_recursive_types --> milestone_inference_v2 --> milestone_stdlib_hardening
    milestone_stdlib_hardening --> milestone_nested_functions --> milestone_forward_refs --> milestone_narrowing_v3
    milestone_narrowing_v3 --> milestone_union_ops --> milestone_subscript_v2 --> milestone_comprehension_v2
    milestone_comprehension_v2 --> milestone_generics_impl --> milestone_phase_fixes
    milestone_phase_fixes --> milestone_intrinsics --> milestone_stdlib_migration --> milestone_stdlib_expansion --> milestone_stdlib_parity_audit
    milestone_stdlib_parity_audit --> milestone_stdlib_polish --> milestone_stdlib_classes
    milestone_stdlib_classes --> milestone_compiler_hardening --> milestone_lazy_iterators --> milestone_test_infra --> milestone_stdlib_functions
    milestone_stdlib_functions --> milestone_stdlib_naming --> milestone_stdlib_class_rollout --> milestone_cpython_tests
    milestone_cpython_tests --> milestone_error_safety --> milestone_error_safety_stdlib_types
    milestone_error_safety_stdlib_types --> milestone_io_safety --> milestone_parse_safety --> milestone_collection_safety
    milestone_collection_safety --> milestone_edge_case_safety --> milestone_zero_panic_gate
    milestone_zero_panic_gate --> milestone_error_subclasses --> milestone_borrow_default --> milestone_borrow_hardening --> milestone_borrow_stdlib
    milestone_borrow_stdlib --> milestone_stdlib_pure_expansion --> milestone_new_modules
    milestone_new_modules --> milestone_stdlib_intrinsic_expansion --> milestone_stdlib_class_deepening
    milestone_stdlib_class_deepening --> milestone_stdlib_remediation
    milestone_stdlib_remediation --> milestone_auto_init --> milestone_generics_v2 --> milestone_pattern_matching
    milestone_pattern_matching --> milestone_enums --> milestone_integer_safety --> milestone_stdlib_generic_rewrite
    milestone_stdlib_generic_rewrite --> milestone_rust_ir_types --> milestone_rust_ir_renderer --> milestone_codegen_preamble_migration
    milestone_codegen_preamble_migration --> milestone_codegen_stmt_expr_migration --> milestone_codegen_intrinsic_migration --> milestone_codegen_structural_passes
    milestone_codegen_structural_passes --> milestone_async_core --> milestone_typed_serde_core
    milestone_typed_serde_core --> milestone_networking_stdlib
    milestone_async_core --> milestone_async_sync
    milestone_networking_stdlib --> milestone_async_advanced
    milestone_async_sync --> milestone_async_advanced
    milestone_async_core --> milestone_web_framework
    milestone_typed_serde_core --> milestone_web_framework
    milestone_async_core --> milestone_database
    milestone_web_framework --> milestone_typed_web_extractors --> milestone_crypto_auth
    milestone_crypto_auth --> milestone_web_production --> milestone_web_services
    milestone_web_services --> milestone_ffi
    milestone_ffi --> milestone_package_mgmt
    milestone_package_mgmt --> milestone_dev_tooling
    milestone_typed_serde_core --> milestone_data_processing
    milestone_data_processing --> milestone_ml_inference
    milestone_dev_tooling --> milestone_metaprogramming --> milestone_ecosystem
```

## Progress Narrative

After milestone_safe_indexing, Sifr has a complete safety story (no panics from data access). After milestone_imports, Sifr supports multi-file projects. After milestone_generics, the type system is fully expressive. After milestone_decorators, the language has all features needed for stdlib and framework design. After milestone_test_runner, Sifr can test itself (dogfooding).

After the Language Hardening phase, the core language compiles ~60% of LeetCode problems -- nested functions, forward references, generics, comprehensions, union operations, and all Phase 2/3 bugs are fixed.

After the Stdlib Architecture phase, Sifr's stdlib is rewritten as `.sifr` files using a three-tier hybrid architecture (Rust intrinsics → Sifr stdlib → user code), with 37+ modules. The legacy `emit_stdlib_call` codegen path is deleted, and the class-in-stdlib pipeline is proven end-to-end (collections.Counter).

After the Stdlib Parity phase, the compiler produces clear "unknown module" errors for bad imports, the `with` statement implements the full `ContextManager` protocol, `Callable` types work correctly in struct fields, generators produce lazy iterators via state machine codegen, the test infrastructure includes `assert_almost_eq` for float testing, ~50 new stdlib functions and 6 new classes are available, all API names are aligned with CPython conventions, and ~500 CPython test assertions validate behavioral correctness.

After the Error Safety phase, the compiler enforces that all `Result` error types must be classes extending `Error`, `Result[T, str]` is a compile error, and `except` arms are exhaustiveness-checked against all error types from the `try` body. Built-in error classes (`IOError`, `ParseError`, `ValueError`, etc.) are available without imports. Module-specific error types (`StatisticsError`, `CycleError`) are exportable from stdlib `.sifr` files.

After the Stdlib Safety Remediation phase, all ~45+ `.unwrap()` panic paths in intrinsics are eliminated. Every file I/O, parse/decode, collection, and edge case operation returns `Result` or `Option` with proper error types. Error subclasses (`FileNotFoundError`, `PermissionError`, `FileExistsError`, `IsADirectoryError`, `NotADirectoryError`) enable compile-time exhaustiveness checking at sub-error granularity — developers can handle specific I/O failure modes without string matching, and the compiler enforces coverage. The zero-panic gate enforces that no panic-inducing patterns remain in user-facing codegen, every module scores 7/10+ on the safety audit, and a comprehensive E2E test proves no stdlib function panics on invalid input.

After the Borrow-by-Default phase, Sifr uses borrow-by-default for function parameters with explicit `mut`/`own` opt-in. Escape analysis prevents silent `.clone()` insertion. Consuming-self method receivers work correctly. For-loop element semantics are resolved and documented. The 7 known codegen regressions are fixed. Stdlib functions (`heapq`, `bisect`) use `mut` parameters, proving the model works in real code.

After the Stdlib Deepening phase, the stdlib reaches deep CPython parity with 8 new modules (`subprocess`, `sys`, `html`, `gzip`, `zipfile`, `configparser`, `calendar`, `operator`), full `datetime`/`deque`/`Pattern` classes, and API naming divergences documented in `architecture.md`.

After the Stdlib Remediation phase, all gaps from the Phase 11 gap analysis are closed: the `open()` built-in with file object protocol, context manager support, and both text and binary modes (`"r"`, `"w"`, `"a"`, `"rb"`, `"wb"`, `"ab"`), `datetime.time` and `datetime.timezone` classes, `subprocess.run` returning a structured `CompletedProcess` object, `Path.glob`/`Path.rglob`, `re` flags support, and minor surface area gaps (`os.sep`/`os.linesep`/`os.name`, `time` wrapper functions, `random.choice` re-export). The stdlib is now complete and ready for the generic rewrite.

After the Type System Completion phase, Sifr's type system is fully expressive. Classes auto-generate `__init__`, `__eq__`, and `__str__` from field declarations (eliminating the most common boilerplate). User-facing generics are complete — generic classes with field/method substitution, type parameter inference, protocol bounds, and generic type aliases all work. `match`/`case` provides exhaustiveness-checked pattern matching on union types, literal unions, optional types, and class unions. Simple enum types provide namespaced constants with exhaustive matching. The `bigint` type provides arbitrary-precision arithmetic matching Python's `int` behavior, resolving the integer overflow contradiction with the safety guarantee. The compiler emits warnings for potential `int` overflow. The entire stdlib is rewritten with generics: `itertools`, `functools`, `collections.Counter[T]`, `collections.deque[T]`, `heapq`, `bisect`, `random`, and `test` all use generic type parameters. Type-specific duplicates (`chain_str`, `accumulate_float`) are deleted. `Counter` has operator overloads and works for any hashable type. `deque` is backed by `VecDeque` intrinsics with O(1) front operations. `None` works as a standalone value and type.

After the Codegen Architecture phase, the entire codegen pipeline uses a structured Rust IR instead of string templates. A purpose-built intermediate representation (`RustExpr`, `RustStmt`, `RustItem`, `RustType`) models the ~50 Rust constructs Sifr emits. A pretty-printer renders IR to formatted Rust source. The preamble (error types, `FileHandle` with its 10 methods, logging globals) is emitted via structured IR — the 500-650 character single-line string templates are gone. All statement and expression codegen builds IR nodes via `lower_stmt`/`lower_expr` functions. All ~80 intrinsic function match arms and ~50 method call match arms produce structured IR bodies. Temporal coupling flags (`suppress_field_clone`, `in_generator_closure`, `in_display_impl`) are eliminated or converted to explicit parameters. The string-parsing dead-code eliminator (`filter_rust_code_to_needed`) is replaced by a structural IR pass. Import collection is automatic (no more `needs_hashmap` boolean flags). A clone optimization pass removes unnecessary `.clone()` calls. At least 20 of the 34 Clippy suppressions are removed. All future codegen (async, web, FFI) is built on structured IR from day one.

After the Async and Ecosystem Foundation phase, Sifr has a full async story. The core async runtime (Tokio-backed) supports `async def`/`await`, task spawning, sleep, and timeouts. Web-independent typed serialization (`dumps`/`loads` with auto-derived serde) enables typed JSON roundtrips. Networking stdlib modules (HTTP client, sockets, subprocess async, URL parsing) use typed serde for response parsing. Synchronization primitives (Lock, Channel, Semaphore) and Send/Sync checking at spawn boundaries enable safe concurrent code. Advanced async features (async with, async generators, async comprehensions) complete the story.

After the Web Stack phase, Sifr can build production web applications. The web framework (axum wrapper) and database access (SQLite + sqlx) are separate milestones — database-backed CLI tools work without a web framework. Typed extractors, auth, production features (logging, tracing, rate limiting, CORS), and external services (Redis, S3, email) layer on top.

After the Interoperability phase, Sifr has formal FFI support for calling into Rust and C code. The `extern crate` mechanism adds Rust crate dependencies, `extern "C"` enables C function calls, and the `unsafe` keyword marks FFI boundaries. This formalizes what the intrinsic system already does and gives power users access to the entire Rust ecosystem (50,000+ crates on crates.io).

After the Package Management phase, Sifr projects can declare dependencies in `sifr.toml`, resolve versions with a PubGrub-based solver, and lock exact versions in `sifr.lock`. The `sifr add`/`sifr remove` commands manage dependencies. Before the package registry exists, dependencies are git-only or path-only.

After the Developer Tools phase, Sifr has a complete developer experience: an LSP server with autocomplete, go-to-definition, hover types, and real-time diagnostics; a formatter (`sifr fmt`); a linter (`sifr lint`); and a documentation generator (`sifr doc`).

After the Data Science and ML phase, Sifr can handle data science workflows with Polars DataFrames (CSV/Parquet I/O, lazy evaluation, expressions) and ML inference (model loading, tensor operations, LLM client integration).

After the Ecosystem phase, it is a complete language ecosystem with compile-time metaprogramming (`@dataclass` with ordering/frozen/field config, custom decorators), a package registry (`sifr.dev`), incremental compilation, and a REPL.
