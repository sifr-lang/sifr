# Ad Hoc: Documentation Learning Path and Python Compatibility Narrative

Status: closed (documentation learning path completed; implementation waves merged in PR #2656, #2657, #2658, and #2659; final phase review approved on 2026-06-18; closure PR #2660)
Owner: Codex
Review artifacts:

- `plans/reviews/active/ad-hoc-documentation-learning-path-review-pass-1.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-review-pass-2.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-sonnet-review-pass-1.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-sonnet-review-pass-2.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-sonnet-reference-review-pass-1.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-sonnet-reference-review-pass-2.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-sonnet-guides-review-pass-1.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-sonnet-guides-review-pass-2.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-sonnet-mutability-review-pass-1.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-sonnet-mutability-review-pass-2.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-sonnet-concurrency-review-pass-1.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-sonnet-concurrency-review-pass-2.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-1-sonnet-review-pass-2.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-1-sonnet-review-pass-3.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-2-sonnet-review-pass-1.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-2-sonnet-review-pass-2.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-2-sonnet-review-pass-3.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-3-sonnet-review-pass-1.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-3-sonnet-review-pass-2.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-3-sonnet-review-pass-3.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-4-sonnet-review-pass-1.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-4-sonnet-review-pass-2.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-implementation-wave-4-sonnet-review-pass-3.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-final-phase-sonnet-review-pass-1.md`
- `plans/reviews/active/ad-hoc-documentation-learning-path-final-phase-sonnet-review-pass-2.md`

Implementation status:

- Wave 1: Merged in PR #2656. Scope: navigation foundation, `From Python`, `Status`, `Ownership and Mutability`, Slice 0 semantic fixes, and audience guide route placeholders. Validation: `cd docs && npx mint@latest validate` passed.
- Wave 2: Merged in PR #2657. Scope: core data model pages and dedicated Concurrency concept section. Validation: `cd docs && npx mint@latest validate` passed.
- Wave 3: Merged in PR #2658. Scope: standard-library module inventory, CLI package/workspace reference, and published one-page-per-code error reference. Validation: `cd docs && npx mint@latest validate` passed with Node 24.
- Wave 4: Merged in PR #2659. Scope: Python/Rust developer guide paths and language-doc wayfinding. Validation: `cd docs && npx mint@latest validate` passed with Node 24.
- Final closure: Full Mintlify validation passed with Node 24. Sonnet final phase review pass 1 approved closure with follow-up accuracy notes; pass 2 approved closure after OWN/IMPORT/PACKAGE error-page accuracy fixes. Closure PR #2660.

## Objective

Redesign the `Documentation` tab so a developer who is new to Sifr can build the right mental model quickly:

1. what Sifr is and when to use it;
2. how to install, run, check, and build a first program;
3. how Sifr maps familiar Python syntax onto a compiled, Rust-backed execution model;
4. which Python concepts carry over directly;
5. which Python concepts intentionally behave differently so programs stay panic-free and statically checked.

The finished docs should stay concise and polished. The current writing style is a strength: short sections, focused examples, card groups, steps, callouts, comparison tables, and practical snippets. The redesign should preserve that style instead of turning the docs into a dense language specification.

## Current State

The Mintlify docs currently expose one `Documentation` tab with these sidebar groups:

- Get Started: `index`, `introduction`, `installation`, `quickstart`
- Core Language: type system, error handling, ownership, classes, pattern matching, concurrency
- Commands: CLI reference pages
- Package management: project and package reference
- Modules: standard library overviews and module references
- Compiler errors: diagnostics overview and code reference

The existing pages are visually strong and individually useful, especially:

- `docs/introduction.mdx`: clear product thesis and core examples.
- `docs/quickstart.mdx`: concrete end-to-end walkthrough with `Steps`.
- `docs/language/type-system.mdx`: solid union/narrowing overview.
- `docs/language/error-handling.mdx`: strong explanation of `Result` and `try`/`except`.
- `docs/language/ownership.mdx`: readable borrow-by-default explanation, but it does not yet teach `mut` as clearly as `own`.
- `docs/language/concurrency.mdx`: strong structured-concurrency introduction, but large enough that it should become the overview for a dedicated Concurrency section.
- `docs/stdlib/concurrency.mdx`: useful API/reference material for `sifr.task`, `sifr.sync`, `sifr.parallel`, process, signal, runtime, and cleanup helpers.
- CLI and package pages: good command/reference coverage.

The gap is not presentation quality. The gap is sequencing, scope calibration, compatibility framing, and a small number of semantic contradictions that should be resolved before new learning pages are added.

## Codebase Scan Findings

The second scan covered `lib/sifr`, CLI sources under `crates/sifr/src`, verification inventories, and internal architecture docs. It changes the plan in several ways:

### Shipped standard library surface is broader than current docs

Current public stdlib docs cover only a representative subset:

- collections;
- I/O and filesystem;
- networking;
- concurrency;
- text and encoding.

The shipped `lib/sifr` namespace is much broader. It includes modules such as:

- Data and text: `json`, `csv`, `tomllib`, `configparser`, `html`, `textwrap`, `string`, `re`, `unicode`, `encoding`, `i18n`.
- Files and archives: `pathlib`, `os`, `shutil`, `tempfile`, `zipfile`, `gzip`.
- Algorithms and utilities: `math`, `statistics`, `random`, `secrets`, `hashlib`, `base64`, `heapq`, `bisect`, `itertools`, `functools`, `operator`, `difflib`, `fnmatch`, `glob`, `graphlib`, `ipaddress`, `uuid`.
- System and process: `sys`, `env`, `platform`, `time`, `datetime`, `calendar`, `process`, `signal`, `resource`, `runtime`, `ipc`, `timeit`.
- Developer-facing utilities: `argparse`, `logging`, `test`.
- Concurrency and parallelism: `task`, `sync`, `parallel`.
- Network substrate: `net`, `tls`, `url`, `http`.

This does not mean every module needs a full page immediately. It does mean the docs need a public module index or status matrix so users do not infer that only the currently documented modules exist.

### Core semantics from internal docs must be reflected carefully

Important source-of-truth concepts from `internal_docs` and verification inventories:

- `int` is intended to be exact/arbitrary precision, with explicit fixed-width integer types for storage, binary protocols, FFI, and dtype-sensitive work.
- Safe indexing and missing-key behavior must consistently use `Option`/`None` semantics where the compiler contract says they do.
- `Result`/`Option` handling is mandatory; ignoring fallible results is not normal control flow.
- `assert` is the intentional panic escape hatch for programmer invariants.
- Function parameters are immutable borrows by default. `mut` opts into a mutable borrow, `own` transfers ownership, and `own mut` transfers ownership while allowing local mutation.
- Bare CPython module imports are not aliases for `sifr.*`; supported stdlib imports live under `sifr.*`.
- Async has one canonical public story: `async def`, `await`, `sifr.task`, `sifr.sync`, structured scopes, explicit offload, typed cancellation evidence.
- Bytes are a first-class source type and need a clear place in the learning path once data-model docs expand.

### CLI docs have reference gaps

Current CLI docs cover build/run/check/emit/fmt/lint/test/lsp well, but the CLI source also exposes:

- `init`
- `fetch`
- `repair`
- `tree`
- `package`
- `publish`
- `vendor`
- `self update` / `self version`
- `trace`

Some package commands are covered indirectly in package pages, but the `CLI Reference` group should eventually make the full command inventory discoverable. The first learning-path slice should not absorb this, but the issue should track it as a follow-up reference cleanup.

### Demos are useful content source material

The `demos/` tree is rich enough to seed guides and examples without inventing toy code. Particularly useful topics:

- `borrow_by_default`
- `error_handling`
- `indexing_rules`
- `integer_safety`
- `imports`, `local_imports`, `module_assembly`
- `io_safety`, `binary_files`, `bytes_*`
- `json`, `csv`, `config_json_csv`
- `network_tcp_echo`, `network_http_substrate`
- `async_*`, `blocking_offload_demo`, `cancellation_cleanup_demo`
- `package` and manifest-related demos

Future guide pages should mine these demos instead of writing examples from scratch.

### Concurrency docs already have strong source material

The current concurrency pages and demos are enough to support a dedicated conceptual section:

- `docs/language/concurrency.mdx`: structured tasks, `async def`, `await`, scopes, `gather`, `select`, `TaskGroup`, channels, shared state, and parallel maps.
- `docs/stdlib/concurrency.mdx`: reference-shaped API coverage for `sifr.task`, `sifr.sync`, `sifr.parallel`, `sifr.process`, `sifr.signal`, `sifr.runtime`, and `sifr.resource`.
- `demos/structured_concurrency_demo/main.sifr`: gather, select, TaskGroup, cancellation behavior.
- `demos/task_core_demo/main.sifr`: timeout and task basics.
- `demos/cancellation_cleanup_demo/main.sifr`: cancellation and cleanup.
- `demos/ownership_concurrency_demo/main.sifr`: ownership and task-boundary examples.
- `demos/async_subprocess_pipeline_demo/main.sifr`: async process I/O.

The implementation should split and reshape this material rather than inventing a second concurrency narrative.

## Audience Perspectives

These perspectives should inform the learning docs, but they should not all live inside `Documentation`. The audience-specific, task-shaped material belongs in `Guides`, where readers can choose the path that matches their background.

### Python Developer Evaluating Sifr

Questions they need answered:

- Is this "Python with a compiler", a Python subset, or a separate language with Python syntax?
- Can I use normal Python idioms?
- What breaks if I paste a Python file into Sifr?
- Where do exceptions, `None`, indexing, imports, classes, and async differ?
- Do I need to learn Rust to use Sifr?

### Python Developer Starting a First Project

Questions they need answered:

- What file layout should I use?
- What does `main()` mean?
- How do imports and packages work?
- What type annotations are required?
- How do I install dependencies?
- What do I do when the compiler reports a type, ownership, or `Result` error?

### Rust-Curious Systems Developer

Questions they need answered:

- What safety guarantees does Sifr inherit from Rust?
- How does borrow-by-default map to Rust ownership?
- When do I need `own` or `.clone()`?
- What runtime is produced, and what is not included?
- Can I inspect the generated Rust?

### Tooling, CI, and Production User

Questions they need answered:

- Which CLI commands belong in the local loop vs CI?
- How do diagnostics format for editors and automation?
- How do `fmt`, `lint`, LSP, and editor integration fit into the local loop?
- What are the preview/stability limits?
- What is safe to deploy today?

## Guide Tracks

### Sifr for Python Developers

Recommended route:

- `docs/guides/python-developers/index.mdx`
- Sidebar title: `Python Developers`

Purpose: help Python developers transfer syntax knowledge while replacing Python runtime assumptions with Sifr's compile-time contract.

This should be a guided track, not a second compatibility reference. It should answer:

- what parts of Python syntax transfer directly;
- where type hints become enforced types;
- how `None`, `Result`, `try`/`except`, and missing values work;
- why imports use `sifr.*` instead of bare CPython module names;
- how borrow-by-default differs from Python object references;
- how to structure a first package and run the CLI loop.

Recommended first guide pages:

- `Write your first Sifr program`
- `Handle errors the Sifr way`
- `Use typed values and collections`
- `Import Sifr modules`
- `Move from script to package`

Scope boundary:

- `docs/from-python.mdx`: compatibility bridge. It is table-first, focused on semantic differences, and should not contain task walkthroughs.
- `docs/guides/python-developers/index.mdx`: audience-track index. It gives one paragraph of orientation, a card or step list of guide pages, and a link back to `From Python` for the compatibility table. It should not explain compatibility row by row.
- First Python guide page: task walkthrough. It should help a Python developer write and run a first Sifr program using familiar habits carefully. It should not become a prose version of `From Python`.

### Sifr for Rust Developers

Recommended route:

- `docs/guides/rust-developers/index.mdx`
- Sidebar title: `Rust Developers`

For both audience tracks, use the short sidebar label in navigation and the full page title in the page itself: `Sifr for Python Developers` and `Sifr for Rust Developers`.

Purpose: help Rust developers understand which Rust safety ideas are visible in Sifr and which details Sifr intentionally hides behind Python-shaped syntax.

This title is better than `Rust Developer Guide` because the guide is not teaching Rust; it is orienting Rust developers inside Sifr.

This should answer:

- how Sifr's borrow-by-default model maps to Rust ownership and borrowing;
- when `own` and `.clone()` are needed;
- how `Result` and `Option` appear in source code;
- what generated Rust is for, and when to inspect it with `sifr emit`;
- which Rust concepts are not part of the Sifr user model;
- what runtime, package, and deployment assumptions differ from a normal Rust crate.

Recommended first guide pages:

- `Map Rust patterns to Sifr`
- `Ownership in Sifr terms`
- `Results and optional values`
- `Inspect generated Rust`
- `Build a native binary`

### Linking from Language Docs

Language pages should remain concept-first. They should not start with audience routing, but they can end with lightweight wayfinding.

Use one of these patterns:

- A short `Next steps` card group at the end of major language pages.
- One restrained `<Tip>` near a concept where audience context prevents confusion.
- A `Related guides` section after the page's primary explanation.

Examples:

- `language/type-system.mdx`: link to `Sifr for Python Developers` from the type-hints discussion and to `Sifr for Rust Developers` from `Result`/`Option` vocabulary.
- `language/error-handling.mdx`: link Python readers to the guide page on replacing exceptions, and Rust readers to the guide page on Sifr's `Result` surface.
- `language/ownership.mdx`: after it is renamed to `Ownership and Mutability`, link Python readers from borrow-by-default vs object references, and Rust readers from `mut`, `own`, and `own mut` mapping.
- `language/pattern-matching.mdx`: add a small Python-oriented note around exhaustiveness and link to the Python guide track if the page needs more migration context.
- `language/concurrency.mdx`: link Python readers from async/cancellation differences and Rust readers from structured task/runtime notes.

The links should be phrased as continuation paths, not warnings:

```mdx
<CardGroup cols={2}>
  <Card title="Coming from Python?" href="/guides/python-developers">
    Follow a guided path through your first program, typed errors, imports, ownership, and packages.
  </Card>
  <Card title="Coming from Rust?" href="/guides/rust-developers">
    Map ownership, `Result`, and generated Rust back to the Rust concepts you already know.
  </Card>
</CardGroup>
```

Do not add these cards to every page. Use them where the reader is likely to carry a strong Python or Rust assumption into Sifr. If icons are added during implementation, verify them against the project's configured Mintlify icon library before using them in MDX.

## Documentation Gaps

### 1. Missing "From Python to Sifr" bridge

There is no page that explicitly says: "If you know Python, here is what carries over, what changes, and why."

Add a short bridge page near the top of the `Get Started` group. It should be written as a compatibility orientation, not a warning dump.

Recommended page:

- `docs/from-python.mdx`
- Sidebar title: `From Python`
- Purpose: help Python developers transfer knowledge without being surprised.
- Sidebar placement: after `Quickstart`, not before it. A reader should see Sifr compile and enforce a real program before they read the differences table.

Core sections:

- "Same shape, different contract"
- "What carries over"
- "What changes at compile time"
- "Common surprises"
- "Where to go next"

Use a compact, table-first layout:

| Python concept | Sifr shape | Difference |
| --- | --- | --- |
| Type hints | Type annotations | Enforced by the compiler, not optional metadata |
| Exceptions | `Result[T, E]` plus `try`/`except` | No stack unwinding for recoverable errors |
| Imports | Sifr modules/packages | No arbitrary CPython runtime import surface |
| Integers | Exact `int` plus explicit fixed-width types | Ordinary arithmetic should be Python-simple; representation-sensitive widths are visible |
| Bytes | First-class `bytes` | Encode/decode boundaries are explicit typed operations; do not rely on platform-default text behavior |
| Async | `async def`, `await`, `sifr.task` | Structured task scopes; cancellation is typed evidence, not an ambient exception |
| Function calls | Borrow by default | Heap values are not moved unless `own` is used |
| `assert` | Assertion | Programmer invariant, not recoverable control flow |

After Slice 0 corrects the indexing and optional-value docs, add a "Missing values" row that cites the finalized language and stdlib pages rather than restating low-level rules inline.

### 2. Missing first-hour learning flow

The current `Core Language` pages are useful but reference-shaped. A newcomer needs a clear first-hour flow, but that should not be a new `Language Tour` page in the first slice. `Quickstart` already acts as the guided walkthrough, and adding another evolving-example page would duplicate it.

Instead:

- keep `Quickstart` as the workflow-shaped tour;
- make `From Python` the compatibility bridge after Quickstart;
- use `docs/index.mdx` cards to point readers into that path;
- if a non-CLI tour is still needed later, fold it into `introduction.mdx` as a compact "Tour at a glance" section with sequential sections or accordions, not `Steps`.

### 3. Missing current status and compatibility boundaries

The introduction says Sifr is in preview, but there is no clear "what works today" boundary for developers evaluating whether to try it.

Add a concise status page:

- `docs/status.mdx`
- Sidebar title: `Status`
- Sidebar placement: a bottom "Project" group or other low-priority location, not the initial `Get Started` sequence.

Sections:

- "Preview channels"
- "Supported platforms"
- "Language surface"
- "Standard library surface"
- "Tooling surface"
- "Known limits"
- "Stability expectations"

This page should avoid apology language. It should give practical boundaries and link to installation, stdlib, and diagnostics.

### 4. Missing core data model pages

The docs explain type system, ownership, and stdlib modules, but they do not yet explain the core everyday data model as a developer would search for it:

- numbers and bools;
- strings and indexing;
- lists, dicts, tuples, and sets;
- iteration and comprehensions;
- functions and callables;
- modules and imports.

Some of this appears indirectly in `stdlib/collections.mdx` or demos, but it should be findable in `Core Language` without duplicating stdlib reference pages.

Recommended follow-up pages:

- `language/values-and-collections.mdx`
- `language/iteration.mdx`
- later: `language/bytes-and-text.mdx` only when it can bridge core `bytes` semantics without duplicating `stdlib/text-encoding.mdx`

Do not add all at once if that would produce noise. The first implementation slice should add only the bridge pages that unlock the learning path:

1. `From Python`
2. `Status`

Defer `strings-and-bytes`, `functions`, and a standalone `modules-and-imports` page until there is content that does not already belong in the existing stdlib, ownership, type-system, package, or CLI pages.

Then expand the data model pages as content-backed follow-up work.

### 5. Missing "different from Python" callouts inside existing pages

Existing pages mention differences, but the signal is scattered. Add consistent callout patterns:

- `<Note>` for "If you know Python..."
- `<Warning>` only for real surprises or unsafe assumptions.
- `<Tip>` for migration ergonomics.

Examples:

- Type system page: Python type hints are optional runtime metadata; Sifr annotations are compile-time obligations.
- Error handling page: `raise` in Sifr does not unwind; it maps to `Err` inside `Result` functions.
- Ownership page: Python passes object references; Sifr borrows by default and may require `own` or `.clone()`.
- Ownership and Mutability page: Python permits ordinary parameter rebinding and mutation; Sifr requires `mut` or `own mut` when a parameter should be changed.
- Pattern matching page: Sifr uses exhaustiveness and type narrowing; Python `match` does not enforce the same compile-time completeness.
- Stdlib overview: Sifr does not expose arbitrary bare CPython module names; supported modules live under `sifr.*`.
- IO page: `open()` without explicit encoding is rejected; this avoids platform-dependent text behavior.

### 6. Ownership page should become "Ownership and Mutability"

The current `docs/language/ownership.mdx` explains borrow-by-default and `own`, but it does not give `mut` equal treatment. That leaves a major surprise for both Python and Rust readers:

- Python readers expect parameter rebinding and list/dict mutation to be ordinary local behavior.
- Rust readers expect a clear distinction between shared borrow, mutable borrow, move, and owned mutable binding.

Rename the page in navigation and frontmatter:

- File can remain `docs/language/ownership.mdx` to preserve the existing route.
- Page title: `Ownership and Mutability`
- Sidebar title: `Ownership and Mutability`
- Description: mention borrow-by-default, `mut`, `own`, and `own mut`.

The page should teach the four parameter conventions together:

| Parameter style | Meaning | Caller keeps value? | Callee can mutate? |
| --- | --- | --- | --- |
| `items: list[int]` | immutable borrow | yes | no |
| `mut items: list[int]` | mutable borrow | yes | yes, during the call |
| `own items: list[int]` | owned immutable binding | no | no, unless copied into a mutable local |
| `own mut items: list[int]` | owned mutable binding | no | yes |

Primary sections to add or reshape:

- "Immutable by default": bare parameters cannot be reassigned or mutated.
- "`mut` for mutable borrows": use when the function should update the caller's value in place.
- "`own mut` for consuming and changing a value": use when the function takes ownership and mutates before returning or dropping the value.

Rules and callouts to include without turning them into large sections:

- `<Warning>` for "Borrowed values cannot escape": `mut items: list[int]` can mutate the caller's list during the call, but it cannot be returned or stored as an owned value. Use `own`, `own mut`, or `.clone()` depending on intent.
- "Reassignment vs mutation": `mut` is required both when rebinding a parameter name and when mutating through a heap parameter. Rebinding `mut items` to another list still does not let the borrowed value escape.
- "Scalar parameters": `mut n: int`, `mut x: float`, and `mut ok: bool` permit local rebinding, but they do not represent an observable heap borrow or ownership transfer.
- "Formatter convention": `own mut` is canonical; the formatter rewrites `mut own` to `own mut`.
- "Bytes are immutable": `mut` does not make `bytes` subscript assignment legal. Keep this as a one-sentence note, not a full section.
- "Across `await`": mutable borrows cannot remain live across an await point; link to concurrency docs rather than explaining async borrowing deeply here.

Good source material:

- `internal_docs/architecture.md` parameter convention section.
- `demos/own_mut_appends/main.sifr`.
- `demos/string/main.sifr` and `demos/collections/main.sifr` for `mut target: list[bool]`.
- `crates/sifr_lowering/src/lower/own_mut_semantics_tests.rs` for rejected immutable parameter mutation/reassignment.
- `SIFR-OWN-0005`, `SIFR-OWN-0006`, `SIFR-OWN-0007`, `SIFR-OWN-0008`, and `SIFR-OWN-0009` diagnostics.

The page should stay elegant: lead with the mental model and a single strong table, then show one mutable-borrow example and one `own mut` example. Keep Rust lowering details out of the main flow; put them in a short note for Rust readers or link to the Rust guide track.

### 7. Missing Python docs reference strategy

The docs should reference Python documentation when it helps transfer familiar syntax, but Sifr docs must remain the source of truth for semantics.

Use Python references for:

- basic syntax vocabulary (`def`, `class`, `if`, `for`, expressions);
- structural pattern matching terminology;
- Python typing vocabulary such as union syntax and `TypeVar`;
- common library concepts where Sifr intentionally mirrors a Python-shaped surface, such as `pathlib`, text encodings, `async`/`await`, or `match`.

Do not rely on Python docs for:

- Sifr type enforcement;
- Sifr ownership and borrow rules;
- `Result` / `Option` behavior;
- panic-free indexing;
- package layout and `sifr.toml`;
- Sifr stdlib availability;
- generated Rust or deployment behavior.

Recommended pattern:

```mdx
<Note>
  If you know Python's `match`/`case` from [PEP 634](https://peps.python.org/pep-0634/), the syntax should look familiar. Sifr adds compile-time exhaustiveness for typed unions, so every possible variant must be handled.
</Note>
```

Reference links should be sparse and intentional. Put them near the concept they clarify, not in a large external-reference appendix. Prefer stable Python PEP links when the concept is defined by a PEP:

- PEP 484 for type hints and typing vocabulary.
- PEP 604 for `X | Y` union syntax.
- PEP 634 for structural pattern matching.

Use `docs.python.org` links for tutorial-level syntax or library concepts only when the Sifr page needs to say "this syntax should look familiar." Avoid meta copy such as "Python's documentation explains the syntax shape"; say the product difference directly.

### 8. Concurrency deserves its own conceptual section

Concurrency is big enough to be more than a single `Learn Sifr` page. It crosses language syntax, ownership, typed errors, cancellation, runtime behavior, synchronization primitives, CPU parallelism, subprocesses, and networking. A newcomer should not have to learn all of that from one long page or from a stdlib API reference.

Best shape:

1. Keep `docs/language/concurrency.mdx` as the stable conceptual overview route, but rewrite it as `Concurrency Overview`.
2. Add a dedicated `Concurrency` group under the `Documentation` tab, between `Learn Sifr` and `Standard Library`.
3. Keep `docs/stdlib/concurrency.mdx` as the API reference for modules and methods, but label it `Concurrency API` in the sidebar to avoid colliding with the conceptual section.
4. Use guides for task-shaped workflows such as building a server, adding timeouts, or doing CPU-heavy work.

The route asymmetry is intentional: the overview keeps the existing `docs/language/concurrency.mdx` path for stability, while new sibling pages can live under `docs/concurrency/`.

Recommended `Documentation > Concurrency` sidebar:

1. `Overview`
   - Structured concurrency in Sifr.
   - No fire-and-forget tasks.
   - No global event-loop object.
   - Every task belongs to a scope.
2. `Async and Await`
   - `async def`
   - `await`
   - real suspension requirement
   - async functions returning `Result[T, E]`
   - Python `asyncio` differences at a high level
3. `Structured Tasks`
   - `task.scope()`
   - `scope.spawn`
   - `TaskHandle` as a linear value
   - `gather`
   - `select`
   - `TaskGroup`
4. `Cancellation and Timeouts`
   - cancellation evidence
   - `task.timeout`
   - `deadline`
   - cleanup before cancellation is observed
   - sibling cancellation behavior
5. `Ownership Across Tasks`
   - values crossing task boundaries must be owned and sendable
   - borrowed values and lock guards do not cross task boundaries
   - mutable borrows cannot cross `await`
   - link back to `Ownership and Mutability`
6. `Channels and Shared State`
   - ownership transfer through channels
   - bounded channels and backpressure
   - `Shared[T]`
   - `Lock[T]`, `RwLock[T]`, `Semaphore`, `Notify`
   - keep details below API-reference depth
7. `Parallel Work`
   - when to use `sifr.parallel` instead of async tasks
   - `parallel.map`, `try_map`, `Pool`
   - CPU-heavy work and worker-boundary ownership
8. `Processes and Signals`
   - async subprocesses at a concept level
   - structured shutdown and signal streams
   - link to stdlib/process/signal reference material

Initial implementation should not create all eight pages at once. Start with:

- `Concurrency Overview`
- `Async and Await`
- `Structured Tasks`
- `Cancellation and Timeouts`
- `Ownership Across Tasks`

Defer `Channels and Shared State`, `Parallel Work`, and `Processes and Signals` if they would mostly copy `docs/stdlib/concurrency.mdx`. Those can begin as sections inside the overview and split out once the content is strong enough.

Content disposition:

| Existing material | Destination |
| --- | --- |
| Opening structured-concurrency guarantee from `language/concurrency.mdx` | Stays in `Concurrency Overview` |
| `Async Functions` examples | Move to `Async and Await`; expand with demo-sourced suspension examples from `async_generator_comprehension_demo`, `async_subprocess_pipeline_demo`, and `blocking_offload_demo` |
| `task.scope and Scoped Spawn`, `task.select`, `TaskGroup` | Move to `Structured Tasks` |
| Timeout, deadline, cancellation, cleanup, sibling cancellation | `Cancellation and Timeouts`; this is mostly demo-sourced net-new writing |
| Sendability, owned task-boundary captures, borrowed values, mutable borrow across `await` | `Ownership Across Tasks` |
| `sifr.sync` tabs | Remain brief in overview or defer; API details stay in `Concurrency API` |
| `sifr.parallel` section | Remain brief in overview or defer; API details stay in `Concurrency API` |
| Module table | Overview wayfinding cards plus links to `Concurrency API` |

`Concurrency Overview` should not remain a long all-in-one tutorial after sub-pages exist. It should become a mental-model entry point: structured-concurrency guarantees, common differences from Python `asyncio`, a small "which page should I read next?" card group, and links into the stdlib API reference.

`Async and Await` owns the Python `asyncio` comparison at the syntax/model level. Once that page exists, the warning in `docs/stdlib/concurrency.mdx` should be trimmed to a short cross-reference instead of carrying the conceptual explanation.

Boundary rules:

- Concept pages explain the mental model, core guarantees, and common surprises.
- Stdlib reference pages list APIs, parameters, return types, module coverage, and edge-case behavior.
- Guides show end-to-end tasks and should pull from demos.
- Diagnostics pages explain individual compiler errors such as `SIFR-ASYNC-*` and `SIFR-OWN-0009`.

The concurrency section should link smoothly to:

- `Ownership and Mutability` for `mut`, `own`, `own mut`, and borrowed-value escape rules.
- `Error Handling` for `Result` in async functions and task failures.
- `Networking` for async TCP/HTTP/TLS examples.
- `Stdlib > Concurrency` for API detail.
- `Reference > Error Codes` for async/ownership diagnostics.

### 9. Need clearer split between learning docs and reference docs

The redesigned site should keep the top navigation small while making the left sidebar do more of the information-architecture work.

Proposed top navigation:

- Documentation
- Guides
- Reference
- Blog
- Website
- Install Sifr

`Documentation` and `Guides` should sit on the left side of the header. Search should remain centered. `Blog`, `Website`, and `Install Sifr` should sit on the right side.

`Documentation` should focus on the learning path and conceptual docs. It can still include compact references where they are part of learning Sifr, but deeper catalogs should move to `Reference`.

Proposed sidebar shape:

1. Get Started
   - Welcome
   - Introduction
   - Install
   - Quickstart
   - From Python
2. Learn Sifr
   - Types and Narrowing
   - Error Handling
   - Ownership and Mutability
   - Values and Collections
   - Iteration
   - Classes
   - Pattern Matching
3. Concurrency
   - Overview
   - Async and Await
   - Structured Tasks
   - Cancellation and Timeouts
   - Ownership Across Tasks
   - Channels and Shared State
   - Parallel Work
   - Processes and Signals
4. Standard Library
   - Overview
   - Module Index
   - Collections
   - I/O and Filesystem
   - Networking
   - Concurrency API
   - Text and Encoding
5. Packages
   - Overview
   - Manifest
   - Dependencies
   - Publishing
6. Project
   - Status

`Reference` should hold catalog-shaped material:

1. CLI
   - Overview
   - Build and Run
   - Check and Emit
   - Format and Lint
   - Test
   - LSP
2. Error Codes
   - Overview
   - one page per diagnostic code

This keeps `Documentation` approachable for a newcomer while giving experienced users a predictable reference tab.

The CLI list above is the initial shape. Slice 4 should expand it to the full public command inventory once the current CLI gaps are documented.

### 10. Missing Reference tab and per-code diagnostic pages

The current `docs/diagnostics/error-codes.mdx` page is useful as a family index, but it does not yet give each diagnostic a stable, searchable explanation page. A developer who sees `SIFR-TYPE-0002` or `SIFR-RESULT-0001` should be able to open a direct URL and learn exactly what happened, why it happened, and what a fixed program looks like.

Rust's error index is the right product shape to study: short per-code pages with an erroneous example, a plain explanation, and a corrected example. Sifr should follow that shape without copying Rust's content or making the pages feel like compiler internals.

Current source material:

- `crates/sifr_diagnostics/src/codes/registry.rs` is the registry source of truth.
- `verification/areas/diagnostics/data/code_catalog.json` maps codes to metadata, owners, severity, stability, representative fixtures, and generated docs links.
- `docs/errors/*.md` already contains generated per-code metadata, but `docs/.mintignore` excludes `errors/` and the pages are not publication-ready. They are metadata inputs, not public docs.
- `docs/diagnostics/error-codes.mdx` includes family descriptions and the published index, but some fix guidance needs a semantic audit before it is reused. For example, `SIFR-RESULT-0001` currently mentions `unwrap()` and `?` syntax; that should only remain if those are validated as public Sifr mechanisms.

Recommended public structure:

- Keep `docs/diagnostics/error-codes.mdx` as the existing discoverable index route, and make each row link to the corresponding per-code page.
- Add a `Reference` top-level tab in `docs/docs.json`.
- Add a single `Reference > Error Codes` group with a public overview/index and per-code pages.
- Preserve the compiler-emitted diagnostic URL contract as the canonical route for individual codes: `https://sifr.sh/docs/errors/<CODE>`.
- Publish the pages from `docs/errors/<CODE>.mdx`, register them under the `Reference > Error Codes` group, and remove or narrow the `errors/` entry in `docs/.mintignore` only after generated `.md` metadata has been replaced with reader-facing MDX.
- Treat the existing `/diagnostics/error-codes` route as a compatibility entry point. It can either remain the canonical index or redirect readers clearly to the `Reference` index, but it should not become stale.

Each everyday source-level per-code page should use a consistent, skimmable structure:

1. `# SIFR-XXXX-0000`
2. One-sentence summary
3. `## Erroneous example`
4. `## What went wrong`
5. `## How to fix it`
6. `## Fixed example`
7. `## Related`

The examples should be reader-scale, not raw compiler-test fixtures. Representative fixtures from the catalog should seed the example, but the final code should be edited for documentation clarity. For project, package, backend, or formatter diagnostics where a single `.sifr` snippet is not the clearest form, use the smallest realistic file tree, command, manifest, or terminal example instead.

Do not force the full code-example template onto every diagnostic. Tier the pages by how developers encounter them:

- Tier 1: full treatment with erroneous code, explanation, fix, and fixed code for everyday programming diagnostics such as `TYPE`, `NAME`, `CALL`, `RESULT`, `MATCH`, `FLOW`, `OWN`, `ASYNC`, `IMPORT`, and `PARSE`.
- Tier 2: focused treatment with explanation and a minimal example for source-level but specialized diagnostics such as `INT`, `DECIMAL`, `IO`, `ENCODING`, `CLASS`, `PROTO`, `FMT`, and `LINT`.
- Tier 3: brief treatment with what happened and what to do next for environment, package, workspace, build, stdlib, and internal diagnostics. Use file trees, manifests, commands, or terminal output when that is clearer than source snippets.

`CODEGEN` currently has no active codes, so it should remain listed as having no active codes rather than gaining an empty page. `SIFR-INTERNAL-0002` is informational note output, so it should not use the erroneous/fixed template.

The index should remain compact:

- group by family;
- show severity and a one-line description;
- link every stable active code to its page;
- include a short note about `sifr --explain <CODE>`;
- avoid repeating each page's full explanation.

## Content Principles

1. Prefer one strong example over many tiny fragments.
2. Use comparison tables where Python developers would otherwise infer the wrong thing; do not make every difference a table row.
3. Keep paragraphs short; let code and callouts carry the detail.
4. Use `Steps` for workflow pages, not concept pages.
5. Use cards for wayfinding at the end of pages.
6. Every page should answer "what do I do next?"
7. Mention Python when it helps orientation; do not frame Sifr as a Python compatibility clone.
8. State differences as product decisions, not caveats.
9. Avoid exhaustive spec language in first-pass learning pages.
10. Move spec-level detail to diagnostics, CLI references, or architecture docs; learning pages link to those instead of inlining them.
11. Diagnostic reference pages should lead with the fix path. Do not bury the corrected example below internal compiler detail.
12. Error-code docs should describe the user-facing program shape that caused the diagnostic, not the internal pass that emitted it.

## Proposed Implementation Slices

### Slice 0: Semantic Consistency Audit

Files:

- `docs/introduction.mdx`
- `docs/quickstart.mdx`
- `docs/language/type-system.mdx`
- `docs/stdlib/collections.mdx`
- relevant compiler demos/tests/source areas that pin indexing, missing-key, and optional-value behavior

Acceptance:

- Correct the current dictionary/indexing contradiction before new pages are added:
  - `internal_docs/architecture.md` already defines the contract: where CPython raises `KeyError`, Sifr returns `Option[V]`; `dict["missing"]` returns `None`.
  - `docs/stdlib/collections.mdx` currently contradicts this by saying direct `scores["missing"]` produces a typed error if the key might be absent.
  - Update `collections.mdx` to match the architecture contract and align `introduction.mdx`, `quickstart.mdx`, `type-system.mdx`, and future `From Python` copy around the same rule.
- Verify the actual contract against existing compiler behavior and cite the demo, test, or source area that pins it.
- Write the resolved contract into `docs/language/type-system.mdx` and `docs/stdlib/collections.mdx` so Slice 1 can cite stable anchors instead of deciding semantics while drafting `From Python`.
- Fix `docs/language/type-system.mdx` "None Safety" examples so every `int | None` value is checked or intentionally narrowed before numeric/string operations.
- Fix `docs/language/type-system.mdx` built-in scalar table: `int` must not be described as a 64-bit signed integer; align it with the exact/arbitrary-precision source-level `int` design.
- Fix `docs/language/type-system.mdx` Copy-type note: `float` and `bool` are Copy-like, but source-level exact `int` should be described as value-semantic rather than Rust-`Copy`.
- Keep the panic-free/safe-indexing pitch precise and implementation-backed.

### Slice 1: Learning Path Skeleton

Files:

- `docs/docs.json`
- `docs/from-python.mdx`
- `docs/status.mdx`
- `docs/language/ownership.mdx`
- targeted edits to `docs/index.mdx`, `docs/introduction.mdx`, and `docs/quickstart.mdx`

Acceptance:

- After reading `Introduction`, `Quickstart`, and `From Python`, a Python developer can state:
  - what Sifr does that Python does not;
  - which familiar Python idioms have different semantics;
  - what command they should run next.
- Sidebar group names distinguish learning pages from reference pages.
- `From Python` is table-first and does not duplicate the value-proposition prose from `Introduction`.
- `Introduction` is trimmed or reshaped where necessary so it states the product thesis once and lets `From Python` own the detailed compatibility table.
- `docs/index.mdx` wayfinding points at the learning path: at minimum, one top card should point to `Quickstart` and one should point to `From Python`.
- `Status` lives outside the main Get Started learning sequence.
- `docs/language/ownership.mdx` is retitled in frontmatter and sidebar as `Ownership and Mutability` while keeping the file path stable.
- `Ownership and Mutability` teaches `mut`, `own`, and `own mut` together, with a compact convention table and examples sourced from existing demos/tests.
- The page explains that bare parameters are immutable by default, `mut` permits mutation through a borrow, `own` moves the value, and `own mut` moves the value while permitting local mutation.
- The page explains that mutable borrowed parameters cannot be returned or stored as owned values without `own`, `own mut`, or `.clone()`.
- The page notes that `mut` on scalar parameters permits local rebinding but does not involve observable heap borrowing or ownership transfer.
- The page avoids becoming a Rust lowering spec; Rust-specific mapping is a note or link to the Rust guide track.
- The imports row in `From Python` links to a concrete worked import example, either in `Quickstart`, `cli/overview.mdx`, or another existing page chosen during Slice 1.
- `Status` includes a short standard-library availability section that points to `stdlib/overview.mdx` and does not imply undocumented modules are unsupported.
- `npx mint@latest validate` passes from `docs/`.

### Slice 2: Core Data Model

Files:

- `docs/language/values-and-collections.mdx`
- `docs/language/iteration.mdx`
- targeted links from stdlib pages to the language pages.

Acceptance:

- Core language docs cover everyday values before module-level stdlib docs.
- Python differences around indexing, truthiness, iteration, and mutation are easy to find.
- Existing demos are used as source material where possible.
- `language/values-and-collections.mdx` covers scalars, exact `int`, explicit fixed-width integer types at a high level, list/dict/tuple/set basics, truthiness, and mutation rules.
- `stdlib/collections.mdx` remains the home for `Counter`, `deque`, and module-level helpers.

### Slice 3: Concurrency Section

Files:

- `docs/docs.json`
- `docs/language/concurrency.mdx`
- optional new conceptual pages under `docs/concurrency/`, such as:
  - `docs/concurrency/async-and-await.mdx`
  - `docs/concurrency/structured-tasks.mdx`
  - `docs/concurrency/cancellation-and-timeouts.mdx`
  - `docs/concurrency/ownership-across-tasks.mdx`
- targeted links from `docs/stdlib/concurrency.mdx`, `docs/stdlib/networking.mdx`, `docs/language/ownership.mdx`, and `docs/language/error-handling.mdx`

Acceptance:

- `Documentation` has a dedicated `Concurrency` sidebar group between `Learn Sifr` and `Standard Library`.
- `docs/language/concurrency.mdx` remains the stable conceptual overview route and is retitled `Concurrency Overview`.
- The first implementation creates or reshapes only the concept pages that have enough content:
  - `Overview`
  - `Async and Await`
  - `Structured Tasks`
  - `Cancellation and Timeouts`
  - `Ownership Across Tasks`
- `Channels and Shared State`, `Parallel Work`, and `Processes and Signals` are either concise sections in the overview or deferred pages; they should not be split out if doing so only duplicates the stdlib reference.
- Concept pages explain the mental model, guarantees, and common surprises. They do not list every method on `sifr.task`, `sifr.sync`, or `sifr.parallel`.
- `docs/stdlib/concurrency.mdx` remains the API reference and links back to the conceptual overview for learning material.
- `docs/stdlib/concurrency.mdx` uses the sidebar label `Concurrency API`, not `Concurrency`, to avoid competing with the conceptual group.
- The implementation follows the content disposition table above so existing `language/concurrency.mdx` material is moved, summarized, or linked rather than duplicated.
- `Concurrency Overview` is rewritten as a concise mental-model entry point with navigation cards; it is not merely the old long page with a new title.
- `Async and Await` and `Cancellation and Timeouts` are treated as partly net-new writing sourced from demos, not just extracted sections.
- The section makes Python `asyncio` differences explicit without framing Sifr as an `asyncio` clone.
- The section links to:
  - `Ownership and Mutability` for `mut`, `own`, `own mut`, sendability, and borrowed values across `await`;
  - `Error Handling` for `Result` in async functions and task failures;
  - `Networking` for async networking examples;
  - `Reference > Error Codes` for `SIFR-ASYNC-*` and `SIFR-OWN-0009`.
- Examples are sourced from existing demos, especially `structured_concurrency_demo`, `task_core_demo`, `cancellation_cleanup_demo`, and `ownership_concurrency_demo`.
- `npx mint@latest validate` passes from `docs/`.

### Slice 4: Standard Library Module Index

Files:

- `docs/stdlib/overview.mdx`
- optional `docs/stdlib/module-index.mdx`
- `docs/docs.json`

Acceptance:

- Users can discover the shipped `sifr.*` modules without needing to read `lib/sifr`.
- The module index groups modules by purpose and marks detailed docs as "available" vs "planned" without overpromising parity.
- The index can also mark small modules as available without a dedicated page when a full page would add noise.
- The index links to existing pages for collections, I/O, networking, concurrency reference, and text/encoding.
- The index explicitly states that Sifr docs own Sifr semantics even when a module mirrors a Python standard-library concept.

### Slice 5: CLI Reference Inventory

Files:

- `docs/cli/overview.mdx`
- package pages as needed
- optional targeted CLI pages for `init`, `fetch/tree/vendor`, `repair`, `self`, and `trace`

Acceptance:

- The CLI overview lists every public command exposed by `sifr`.
- The CLI overview covers global `--diagnostic-format` and `--explain <CODE>` flags alongside subcommands.
- Commands are grouped by workflow concern, not as a flat alphabetical list.
- Commands that already have deeper pages link to them.
- Package-management commands do not get duplicated prose if the package pages already explain them.
- `self update` and `trace` are discoverable, with status/preview caveats where appropriate.

### Slice 6: Reference Tab and Error Code Pages

Files:

- `docs/docs.json`
- `docs/diagnostics/error-codes.mdx`
- rewritten public `docs/errors/<CODE>.mdx` pages for stable active diagnostic codes
- optional validation script or docs fixture that compares public pages with `verification/areas/diagnostics/data/code_catalog.json`

Acceptance:

- A `Reference` top-level tab exists.
- `Reference` includes diagnostics and error-code material without crowding the `Documentation` learning path.
- The existing `/diagnostics/error-codes` route links to the per-code pages and does not become a dead or stale duplicate.
- Individual error-code pages keep the compiler-emitted `https://sifr.sh/docs/errors/<CODE>` URL contract unless the implementation intentionally updates `DiagnosticCode::docs_url()` and every checked diagnostic baseline in the same slice.
- Every stable active code in `verification/areas/diagnostics/data/code_catalog.json` has a public page or an explicitly documented reason it is not published yet.
- No public page is generated from `docs/errors/*.md` without rewriting the content into reader-facing MDX.
- Tier 1 pages explain what the code indicates, show an erroneous example, explain the fix, and show a fixed example.
- Tier 2 and Tier 3 pages use the smaller documented templates where the full source-code template would be noisy or misleading.
- Examples are sourced from representative fixtures where possible, but edited down to documentation-scale examples.
- The page template handles non-source diagnostics such as package, workspace, backend, formatter, and lint diagnostics with file tree, manifest, command, or terminal examples when code snippets would be misleading.
- No `CODEGEN` stub page is created while that family has no active codes.
- `SIFR-INTERNAL-0002` is treated as an informational note page, not as an erroneous/fixed-code tutorial.
- The `SIFR-RESULT-0001` guidance is audited before publication so it does not recommend mechanisms that are not part of Sifr's current public syntax.
- The index copy is cleaned up while it moves: remove internal commentary such as "largest family" wording and avoid migration-history phrasing unless it helps the user decide what to do next.
- A docs validation step reads `verification/areas/diagnostics/data/code_catalog.json`, checks `stability == "stable"` entries, catches missing pages for active codes, and checks the reverse direction so public error pages do not outlive removed codes.
- The validation step runs from the normal local facade, preferably through `scripts/run_all_tests.sh` under the docs/profile path rather than as a separate one-off command.
- `npx mint@latest validate` passes from `docs/`.

### Slice 7: Python Reference Callouts

Files:

- existing language and stdlib pages.

Acceptance:

- Each page that uses Python-shaped syntax has at most one or two purposeful Python reference callouts.
- No page delegates Sifr semantics to Python docs.
- Differences from Python are consistently signposted.
- Initial callout targets are type system, error handling, ownership and mutability, stdlib overview, and I/O.
- Do not add a generic Python callout to pattern matching unless it clarifies a concrete rule not already explained.

### Slice 8: Guides Expansion Plan

Files:

- `docs/guides/index.mdx`
- `docs/guides/python-developers/index.mdx`
- `docs/guides/rust-developers/index.mdx`
- future guide pages.

Acceptance:

- Guides remain how-to oriented, not concept reference.
- The Guides sidebar includes audience tracks:
  - `Sifr for Python Developers`
  - `Sifr for Rust Developers`
- The two audience guide indexes explain what the reader needs to know from their background without duplicating the language reference pages.
- Track index pages contain only:
  - a one-paragraph orientation;
  - the guide sequence as cards or numbered steps;
  - pointers to the relevant background reference, such as `From Python` for Python readers and `language/ownership.mdx` or `cli/check-emit.mdx` for Rust readers.
- Track index pages do not contain concept explanations, compatibility matrices, or comparison tables.
- `From Python`, `Sifr for Python Developers`, and the first Python guide page have distinct jobs:
  - `From Python` is the compatibility table.
  - `Sifr for Python Developers` is the route through the guide track.
  - The first Python guide page is a task walkthrough.
- Language docs link into the audience guides only where background assumptions are likely to matter.
- Cross-links from language pages use `Next steps`, `Related guides`, or a single focused tip/card group; they do not interrupt the main explanation.
- The guide track names stay reader-centered and clear. Prefer `Sifr for Python Developers` and `Sifr for Rust Developers` over generic titles such as `Python Developer Guide`.
- Guide pages that share a topic with a language page must be distinguishable by task orientation. A reviewer should not be able to summarize a guide page as only "explains concept X."
- General guide pages live under `docs/guides/*.mdx` unless a later slice deliberately introduces a deeper grouping. First general guide set should likely be:
  - "Build a CLI tool"
  - "Read and write files safely"
  - "Typed error handling"
  - "Create a package"
  - "Use Sifr from CI"
- Python developer guide seed pages should likely be:
  - "Write your first Sifr program"
  - "Handle errors the Sifr way"
  - "Use typed values and collections"
  - "Import Sifr modules"
  - "Move from script to package"
- Rust developer guide seed pages should likely be:
  - "Map Rust patterns to Sifr"
  - "Ownership in Sifr terms"
  - "Results and optional values"
  - "Inspect generated Rust"
- `Typed error handling` is the general how-to guide. `Handle errors the Sifr way` is the Python-audience guide that starts from exception habits and walks through the Sifr workflow.
- `Results and optional values` is the Rust-audience guide that starts from Rust `Result`/`Option` assumptions and maps them to Sifr syntax and workflow. It should not duplicate the general typed-error guide.
- Guide examples should be sourced from existing demos when possible.

## Open Questions

- Should "Status" live in Get Started, or in a separate "Project" group?
- Should `From Python` come before or after Quickstart?
- How explicit should the docs be about unsupported CPython runtime features before the compatibility matrix exists?
- Should the first data model page be a single "Values and Collections" page or several smaller pages?
- Should we add a short "Generated Rust" page for Rust-curious users, or keep that in CLI `emit` docs?

Current answers after Claude review pass 1:

- `Status` should not live in Get Started; use a bottom Project group or equivalent low-priority placement.
- `From Python` should come after Quickstart.
- The bridge page should link to `stdlib/overview.mdx` for supported `sifr.*` modules and avoid promising arbitrary CPython module compatibility.
- Start with `Values and Collections` plus `Iteration`, not six separate data-model pages.
- Do not add a separate Generated Rust page yet; `Quickstart` and `cli/check-emit.mdx` cover the current need.

## Non-Goals

- Do not rewrite every existing docs page in one PR.
- Do not add root-level Mintlify shims.
- Do not create a separate Python compatibility promise broader than Sifr can support.
- Do not claim CPython module parity unless the implementation and tests support it.
- Do not move internal compiler reference docs into public docs as part of this task.
- Do not treat existing internal `.md` files under `docs/` as Mintlify pages unless a later slice explicitly migrates them to MDX and adds them to `docs.json`.

## Validation Plan

For future implementation PRs:

```bash
cd docs
npx mint@latest validate
```

For content review:

- Manually preview the docs and inspect the left sidebar order.
- Check pages at desktop and mobile widths.
- Confirm "Ask Assistant" still appears in the expected Mintlify header/panel location.
- Ask Claude Opus to review the issue and proposed content structure before implementation.
