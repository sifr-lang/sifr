# M39.12 Rust Interop Tooling, Diagnostics, Docs — Review Round 1

Branch: `phase39-rust-interop-m39-12`
Scope: items listed under `milestone_39_12` in `plans/phases/39_rust_interop.md`.
Result: **Not satisfied.** Two critical, one high, two medium, two low findings.

## Critical

### C1. `docs/rust-interop.mdx:100` — `@rust.async(thread_affinity=send)` is rejected by the compiler.

The async HTTP example writes:

```sifr
@rust.async(thread_affinity=send)
@rust(bridge.http.fetch_text, panic=map_error(bridge.http.map_panic))
async def fetch_text(url: str) -> Result[str, HttpError | RustPanicError]: ...
```

`crates/sifr_driver/src/build/rust_interop/async_validation.rs:56-65` only accepts `thread_affinity=none` or `thread_affinity=tokio_current_thread`; everything else emits `SIFR-RUST-ASYNC-*` with message ``thread_affinity=` must be none or tokio_current_thread`` (see test `package_rust_interop_async_rejects_unsupported_thread_affinity` in `rust_interop_async_contract_tests.rs:126`). A user copy-pasting the doc example fails the very contract this milestone documents.

Per DoD: "Public and internal docs are aligned with the architecture document." `internal_docs/rust_interop_architecture.md` does not list `send` anywhere as a `thread_affinity` symbol.

Remediation: drop the decorator entirely (Send is the default for async returns) or replace with `@rust.async(thread_affinity=tokio_current_thread)` and add a short sentence saying the default contract already requires `Send`, so the decorator is needed only for current-thread futures. Mirror the wording in `internal_docs/rust_interop_architecture.md` (Async section) so internal + public examples stay in lockstep.

### C2. `docs/rust-interop.mdx:156-164` — `@rust.callback(lifetime=threadsafe, …)` fails callback validation.

The callback registration example writes:

```sifr
@rust.callback(
    lifetime=threadsafe,
    backpressure=bounded(1024),
    overflow=error,
    shutdown=drain,
)
```

`crates/sifr_driver/src/build/rust_interop/callback_validation.rs:132-156` accepts only `backpressure`, `overflow`, `shutdown` and returns ``unsupported `@rust.callback(...)` key `<other>` `` for anything else. `internal_docs/rust_interop_architecture.md:698-723` shows the same example *without* a `lifetime=` key and explicitly enumerates the three legal keys; M39.11's contract grammar never accepted `lifetime` for callbacks. The public example will produce `SIFR-RUST-CB-0001`.

This is the user-facing example called out by the DoD ("user-facing examples for … callback registration") and is the only callback walkthrough in public docs.

Remediation: delete the `lifetime=threadsafe,` line so the example matches the architecture doc verbatim. Replace the surrounding prose ("Thread-safe callback registration requires an explicit callback contract.") with: "Backpressure, overflow, and shutdown behavior must be visible in the declaration; thread-safety is part of the callback type (`ThreadsafeCallback[...]` or `Callable[...]` with an explicit `@rust.callback(...)` contract)." Optional: switch the parameter type to `ThreadsafeCallback[[Event], Result[None, EventError]]` so the example mirrors the canonical architecture form rather than the more general `Callable` (see `internal_docs/rust_interop_architecture.md:704-707`).

## High

### H1. `crates/sifr_analysis/src/completion.rs:113-141` — policy-key completion fires uniformly regardless of decorator kind.

`rust_interop_policy_key_candidates()` returns the union of every policy key across `@rust`, `@rust.async`, `@rust.opaque`, `@rust.zero_copy`, `@rust.view`, and `@rust.callback`. `host/implementation.rs:186` appends that union whenever `prefix.contains('(')` (completion.rs:41-44), so:

- Inside `@rust.callback(...)`, the LSP offers `panic`, `type`, `send`, `sync`, `clone`, `close`, `borrow`, `owner`, `view`, `lifetime`, `mutability` — twelve keys that the validator (`callback_validation.rs`) will reject as ``unsupported `@rust.callback(...)` key``.
- Inside `@rust.opaque(...)`, the LSP offers `panic`, `owner`, `view`, `lifetime`, `mutability`, `backpressure`, `overflow`, `shutdown`. `opaque_contract.rs:84-143` rejects all eight.
- Symmetrically for `@rust.zero_copy`, `@rust.view`, `@rust.async`, and `@rust`.

This directly contradicts the architecture's tooling contract: ``Complete canonical Rust interop decorator dotted paths and policy keys (`@rust`, `@rust.async`, `@rust.opaque`, `@rust.zero_copy`, `@rust.view`, `@rust.callback`, and their policy arguments)`` — "their policy arguments" reads naturally as "the policy arguments of that decorator," not "the union of all decorator policy arguments." It also confused C2 above: the docs author appears to have copied a key that the LSP would have happily suggested.

Remediation: extract the decorator kind from the prefix (parse what follows `@rust` up to `(`) and switch the per-decorator key set inside `rust_interop_completion_candidates`. The valid sets are already encoded as match arms in the corresponding validators (`callback_validation.rs:136-156`, `opaque_contract.rs:84-143`, `zero_copy_validation.rs:230-244`, `async_validation.rs:56-65`); centralise them so completion and validation share one table. Add a completion test per decorator that asserts the *negative* set as well (e.g. `@rust.callback(...)` must not surface `lifetime`).

## Medium

### M1. `crates/sifr_analysis/src/completion.rs:32-38` — decorator completion does not trigger on continuation lines.

`prefix` is built from the start of the current line up to the cursor. Multi-line decorator calls (which the milestone's own docs use heavily — see `docs/rust-interop.mdx:113-125` and `:128-147`) put each named argument on its own line, and those lines do not start with `@ru`. The completion shortcuts return `Vec::new()`, so:

- Typing the second argument of `@rust.view(\n    owner=input,\n    lifetime=|` gets zero policy-key suggestions.
- Every Arrow, DLPack, and view example shipped in `docs/rust-interop.mdx` is multi-line; the LSP cannot help with any of them past the first line.

The DoD says "LSP completion and validation for Rust decorator dotted paths." Completion that disappears for the canonical decorator shape advertised in the same PR's docs is a real gap, not a future enhancement.

Remediation: when the current line's trimmed prefix does not start with `@ru`, scan backwards through preceding lines (or use the parsed module tokens already available via `editor_facts.tokens`) to determine whether the cursor is inside an unclosed `@rust*(...)` decorator call, and offer the policy keys for that decorator. The tokens stream already exposes parenthesis depth; reusing it avoids a brittle line-by-line scanner.

### M2. `crates/sifr_analysis/src/completion.rs:41-44` — policy keys offered immediately after `@rust(`, where the next token is the positional target path.

For `@rust(`, the first argument is always a `TargetPath` (e.g. `crc32fast.hash`), never a policy key. The current logic suggests `panic`, `thread_affinity`, etc. as soon as `(` appears, which steers users toward typing `panic=…` in the slot reserved for the target path. The validator then emits `SIFR-RUST-CONFIG-*` ("decorator requires a Rust target path"), making completion a foot-gun.

Remediation: only enable policy-key completion once we are past the first comma at depth 1 inside the decorator argument list. The token-aware traversal proposed in M1 gives this for free.

## Low

### L1. `crates/sifr_lsp/src/conversion.rs:191-198` — `completionItem` has no `insertText`/`textEdit`, so dotted labels can splice incorrectly.

Selecting `rust.callback` while the cursor sits mid-token at `@rust.cal|` lets the LSP client decide what range to replace. Most clients default to "current word", producing `@rust.rust.callback`. This is pre-existing infrastructure, but it becomes user-visible the moment dotted labels (`rust.async`, `rust.opaque`, `rust.zero_copy`, `rust.view`, `rust.callback`) ship as completion items.

Remediation: emit `filterText: "callback"` (or the last dotted segment) and `insertText: "rust.callback"` for dotted decorator labels, or supply a `textEdit` whose range is the decorator identifier including the `rust.` prefix. The conversion test added at `conversion.rs:580-595` should grow an assertion for the inserted-range behavior so the contract is locked.

### L2. No end-to-end test asserts that `sifr bridge check` and `sifr check` produce identical diagnostics on the same input.

`crates/sifr/src/bridge_cli_tests.rs` covers argument parsing only. Routing is structurally a thin wrapper around `cmd_check` (`cli_model_and_entrypoint.rs:421-443`), so equivalence holds by construction today, but the DoD ("Tooling surfaces the same target resolution and diagnostics as the compiler") is a behavioral promise. A regression that diverges the two paths in a later milestone would slip past the existing argument-parsing tests.

Remediation: add one integration test that drives both commands against a fixture with a known `SIFR-RUST-RESOLVE-*` or `SIFR-RUST-CONFIG-*` violation and asserts the rendered diagnostic codes/messages are identical. Reuse the temp-package scaffolding in `mode_resolution_tests.rs`.

## Observations (non-blocking)

- `docs/rust-interop.mdx:60` references `bridge.hash.map_panic` but the doc never shows a `src/bridges/hash.rs` companion. The crc32fast example two lines above is self-contained; the blake3 example would benefit from a sibling Rust snippet or a forward reference to the tokenizer section so users know where `map_panic` lives.
- `docs/rust-interop.mdx:163`: the canonical thread-safe-callback example in `internal_docs/rust_interop_architecture.md:704-707` uses `ThreadsafeCallback[[Message], Result[None, KafkaError]]`. The public docs use `Callable[[Event], Result[None, EventError]]`. Both are accepted today; preferring the explicit `ThreadsafeCallback` form mirrors the architecture and makes the threading contract visible at the type site.
- `crates/sifr_analysis/src/host/tests.rs:397-403` only asserts that `host.completion` returns a result with `AnalysisQueryKind::Completion`. There is no host-level test that the Rust interop candidates appear through the full pipeline (symbol index + interop candidates + ranker). The unit tests in `completion.rs:192-246` exercise the helper in isolation; adding one host-level test would prevent a silent regression if the `host/implementation.rs:186` `candidates.extend(...)` line is ever dropped.
- `crates/sifr_analysis/Cargo.toml` and `crates/sifr_lint/Cargo.toml` carry `[lib] doctest = false` in this branch. The review brief flagged these as unrelated worktree noise; calling out here only so the PR author confirms they were intentional before merge (they unrelated to milestone scope and should either land in their own PR or be reverted).

## What is good

- `sifr bridge check` routes through the same `cmd_check` → `package_session_for_cwd` → `session.plan_check` → `execute_cargo_plan` / `cmd_check_package_file` pipeline that backs `sifr check` (`cli_model_and_entrypoint.rs:421-443`). There is no parallel diagnostic stack.
- Rejected designs (`docs/rust-interop.mdx:187-209`) are framed as forbidden forms, not "alternate styles": `@rust("…")` string targets, `extern rust`, `from rust import`, and the `crate=`/`path=` shape are listed without remediation prose that would imply they are configurable.
- `docs/docs.json:71-75` adds `rust-interop` to the get-started group and the Rust developers landing page (`docs/guides/rust-developers/index.mdx:22-24`) cross-links into it; users with Rust backgrounds land on the right doc.
- Diagnostic family table in `docs/diagnostics/error-codes.mdx:140-156` now links each code to `/errors/SIFR-RUST-*-0001`, and every linked page exists under `docs/errors/`.
- Trust-policy keys quoted in the docs (`rust-build-scripts`, `rust-proc-macros`, `native-links`, `unsafe-rust-bridges`, `rust-no-panic`, `rust-panic-abort`) match the schema in `crates/sifr_package/src/manifest/sifr_fields.rs:73-109`. `[rust].bridge-version` and `[rust].bridges` likewise match `sifr_fields.rs:148-172`.
- LSP completion kind mapping (`crates/sifr_lsp/src/conversion.rs:511-521`) and its test (`:580-595`) cover the two new kinds (`decorator` → 15, `property` → 10) introduced for Rust interop.

## Required for Round 2

C1, C2, and H1 must be resolved before the milestone can be closed. M1 and M2 should be addressed in the same PR if H1 is fixed by sharing a decorator-kind dispatcher (they collapse into one change). L1, L2, and the observations may be deferred to follow-ups but should be tracked.
