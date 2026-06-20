I have the full picture of the diff. Compiling my findings now.

# Phase 39 Rust Interop Design — Review (Round 1)

## Summary

The design is genuinely scoped: no MVP phrasing, no fallback wording (every "fallback" reference is an explicit rejection statement), no `extern rust` residue in active planning docs, and no dependency on Python interop or earlier interop drafts. Phase renumbering (39 Rust → 40 Stable GA → 41 Typed Data → 42 Web → 43 DS/ML) is consistent across `roadmap.md`, `index.md`, all referenced phase files, and the cross-references in 29/33/34/36/37. Old `43_interoperability.md` is deleted; the new `39_rust_interop.md` and `internal_docs/rust_interop_architecture.md` are self-consistent.

However, there are several **implementation-blocking gaps** in the architecture document that need resolution before milestone_39_1 can land, plus a handful of optional refinements. No hidden backwards-compatible path detected.

---

## Findings (severity ordered)

### B1 — Exact-integer bridge representation is undefined (Blocker)
**File:** `internal_docs/rust_interop_architecture.md` §Bridge Type Contract (lines 251), and `internal_docs/integer_model.md` line 486 (`explicit exact-integer bridge representation`).
The contract text says "A bridge may accept exact integers only through Sifr's exact integer bridge representation," and milestone_39_5 demands "exact integers through explicit exact representation." But no Rust type name, crate location, or layout/ownership semantics are given. Without that, milestone_39_5 type checking cannot be implemented and the bridge table is incomplete.
**Suggested change:** add a row to the bridge table for `int` mapping to a named concrete representation (e.g. `SifrIntBridge` in `sifr_runtime`), specifying parameter/owned/return forms, `Copy`/`Clone` status, and whether it round-trips through the boundary by reference or value. Reference the same name from `integer_model.md`.

### B2 — `dict[str, T]` → `HashMap<String, T>` loses Sifr/Python insertion-order semantics (Blocker)
**File:** `internal_docs/rust_interop_architecture.md` §Bridge Type Contract, table row for `dict[str, T]`.
Sifr inherits Python's insertion-ordered dict semantics. `HashMap` is unordered. Bridging through `HashMap` silently degrades ordering — a behavioral regression at the interop boundary that contradicts the doc's no-silent-downgrade rule.
**Suggested change:** map to an ordered Rust type (e.g. `IndexMap<String, T>` from the `indexmap` crate, or a generated `SifrDictBridge` wrapper) and explicitly document the order-preservation guarantee. Also clarify whether `dict[K, V]` with non-`str` keys is supported and which Rust hash/eq bounds apply.

### B3 — `@rust.view(...)` is referenced but never defined (Blocker)
**File:** `plans/phases/39_rust_interop.md` milestone_39_1 line 65 lists `@rust.view(...)` for parsing/lowering.
The architecture document only specifies `@rust.zero_copy(...)`. No `@rust.view` example, parameter set, or contract appears anywhere. The phase doc therefore promises behavior the normative architecture has not defined.
**Suggested change:** add a `@rust.view(...)` subsection to §Zero-Copy and Views with an example, the parameter set (`owner=`, `lifetime=`, mutability, Send/Sync), and the relationship to `@rust.zero_copy` (presumably `zero_copy` is a stricter `view`, or the two cover orthogonal aliasing modes). If they are the same surface, remove `@rust.view` from milestone_39_1.

### B4 — `catch_unwind` `UnwindSafe`/`AssertUnwindSafe` strategy is unspecified (Blocker)
**File:** `internal_docs/rust_interop_architecture.md` §Error Semantics, lines 273–283.
The example wraps a free function call in `catch_unwind`. In practice, generated wrappers will also call methods that take `&mut Handle<T>`, which is not `UnwindSafe`. Without an `AssertUnwindSafe` policy (and a documented "Rust bridge authors must preserve poisoning-safe state on panic" contract), the emitted code will fail to compile or will silently lose the panic boundary on opaque methods.
**Suggested change:** specify that generated wrappers use `AssertUnwindSafe` and add a paragraph requiring bridge implementations to leave handle state observably consistent or poisoned after a caught panic. Add a `SIFR-RUST-PANIC-*` code for the poisoned-handle re-entry path.

### B5 — Non-`Send` future rule contradicts current-thread Tokio model (Blocker)
**File:** `internal_docs/rust_interop_architecture.md` §Async and Tokio Runtime, lines 312–345.
Two adjacent rules conflict:
- "Current generated async entrypoints use the current-thread flavor" and "no assumption that `rt-multi-thread` is enabled" — i.e. !Send futures are legal on a current-thread runtime.
- "non-`Send` futures are rejected until Sifr has an explicit local-task surface" — i.e. all futures must be `Send`.

The `KafkaConsumer` example with `thread_affinity=tokio_current_thread` further implies a non-Send awaitable is required for `aclose`. This needs reconciliation before milestone_39_7 can be specified.
**Suggested change:** split the rule into (a) `Send` is required only for futures that may be spawned/handed off to multi-threaded executors, and (b) futures pinned to the entrypoint runtime may be !Send if their declarations carry `thread_affinity=tokio_current_thread` or an equivalent local-task annotation. Define the annotation here, don't defer it.

### B6 — Opaque method path resolution rule is incomplete (Blocker)
**File:** `internal_docs/rust_interop_architecture.md` §Path Resolution and §Opaque Types.
`@rust(Self.poll)` is used inside `KafkaConsumer` whose type is `bridge.kafka.Consumer`. The resolution table lists `Self` as "Method binding inside a `@rust.opaque` class" but never states the lowering rule. Concretely: does `Self.poll` resolve to `crate::bridges::kafka::Consumer::poll` (inherent impl), `<Consumer as SifrBridge>::poll` (trait impl), or either-with-precedence? What happens with `Self.aclose` when the underlying Rust type's `aclose` is an `async fn` on a non-`Self` trait?
**Suggested change:** add a "Self resolution" subsection enumerating: (1) inherent method path lowering, (2) precedence vs. trait methods, (3) failure mode when ambiguous, with a `SIFR-RUST-RESOLVE-*` code. Show the lowered Rust call site for the existing `KafkaConsumer.poll` example.

### B7 — Generated bridge type naming convention is implicit (Blocker)
**File:** `internal_docs/rust_interop_architecture.md` §User Model, lines 71–75 and §Bridge Type Contract row "generated bridge enum / struct".
Sifr `HashError` becomes Rust `HashErrorBridge` in the example, but the suffix rule and the place where the type is defined (generated module path) are never stated. Bridge authors writing `src/bridges/*.rs` need to know what to import.
**Suggested change:** specify the generated type path and naming rule (e.g. "every Sifr record/enum reachable across an `@rust` boundary materializes as `crate::__sifr_bridge::<sifr_module>::<Name>` with `repr(C)` for records and `repr(u32)` discriminants for closed enums"). Document this once and reuse.

### B8 — `closed enum` bridge representation is unspecified (Blocker)
**File:** `internal_docs/rust_interop_architecture.md` §Bridge Type Contract, table row "closed enum".
"Generated bridge enum" appears in all three columns but the discriminant type, exhaustiveness, repr attribute, and behavior on unknown discriminant from Rust are not defined. This is needed for milestone_39_5 type checking and the panic-vs-`Result` path on bad discriminants.
**Suggested change:** specify `repr(<width>)`, fixed discriminant assignment rule, and that round-trip from Rust must validate the discriminant before crossing into Sifr — invalid values map to a `SIFR-RUST-TYPE-*` runtime error rather than UB.

### B9 — Cache key omits Cargo panic strategy and profile (Blocker)
**File:** `internal_docs/rust_interop_architecture.md` §Cargo and Build Cache, lines 437–453.
The architecture rejects `panic = "abort"` builds for recoverable bridges, but the cache key only lists "selected Sifr runtime metadata" — too vague to guarantee invalidation when a user flips the profile from `unwind` to `abort` (or changes any other profile setting that influences ABI/codegen, e.g. `lto`, `codegen-units`, `incremental`, `opt-level` when it changes panic behavior under feature gates).
**Suggested change:** add explicit cache-key entries for: selected Cargo profile name, the resolved `panic` strategy, `lto`, `codegen-units`, `incremental`, target features, and the rustc target-spec hash. Tie these to the `SIFR-RUST-PANIC-*` and `SIFR-RUST-CARGO-*` diagnostic families.

### B10 — Phase 39 → Phase 40 dependency ordering is asserted but not justified (Blocker for sequencing)
**File:** `plans/phases/39_rust_interop.md` "Depends on" and `plans/phases/40_*.md` "Depends on Phase 39".
The chain now requires Rust interop to land before Stable GA. That is a substantive change — it puts the entire Rust-interop ecosystem on the GA hot path, including ecosystem certification (milestone_39_12). Either this is intentional (Rust interop is part of the stable promise) or it's a numbering artifact. If intentional, the rationale should be documented because Phase 40's exit gate now indirectly demands every milestone_39_* fixture to be green.
**Suggested change:** either (a) add a one-paragraph rationale at the top of `39_rust_interop.md` stating "Rust interop must land before stable GA because stable promotes the Rust-backed package contract as part of the supported surface," or (b) decouple Phase 40 from Phase 39 by listing the dependency as "if Phase 39 has landed, Phase 40 inherits its quality gates" and noting that stable GA can proceed without Rust interop being complete.

---

## Optional refinements (non-blocking)

- **R1.** `internal_docs/rust_interop_architecture.md` §Package Layout: the `backend/Cargo.toml` example doesn't state the required crate type. Static linking is the only supported model (non-goals reject dynamic ABI loading), so document `crate-type = ["lib"]` and explicitly reject `cdylib`/`dylib` for the backend crate.
- **R2.** §Bridge Type Contract: add a row for callbacks (`Callback[...]`, `ThreadsafeCallback[...]`). They have rich semantics in §Callbacks but no entry in the contract table, which makes the table look complete when it isn't.
- **R3.** §Bridge Type Contract: clarify nested borrowing for `Option[str]`, `Option[bytes]`, and `list[str]`. The flat row says `Option<T>`/`Vec<T>`, but `Option<&str>` vs `Option<String>` is a different ABI.
- **R4.** §Cargo and Build Cache: state how Cargo workspace `Cargo.lock` is shared vs per-package, and how the `bridge-version` field in `sifr.toml` is versioned (it currently appears as `1` with no schema definition).
- **R5.** §Zero-Copy and Views: name the shared bridge crate for DLPack/tensors (only `sifr_arrow_bridge` is named today). Without the name, milestone_39_9's "DLPack-style tensor handoff through shared bridge crates" is under-specified.
- **R6.** §User Model `HashError(Error)`: the `Error` superclass is referenced as if it exists; if Sifr's stable error supertype is not yet named in another doc, link to it.
- **R7.** `plans/phases/39_rust_interop.md` milestone_39_12: "production-grade review rounds and close every blocker" — phrase the closing criterion concretely (e.g. "no remaining `SIFR-RUST-*` diagnostic family has open spec gaps; every fixture has both positive and negative paths checked into the area"). The current wording is the only spot in the phase doc with vague language.
- **R8.** §Verification Area block is duplicated verbatim between `internal_docs/rust_interop_architecture.md` (lines 484–520) and `plans/phases/39_rust_interop.md` (lines 217–252). The phase doc already declares the architecture normative — drop the duplicate fixture list from the phase doc and link to the architecture instead, so the two cannot drift.
- **R9.** §Diagnostics: every `SIFR-RUST-*` family is listed, but no individual codes (e.g. `SIFR-RUST-RESOLVE-0001`) are reserved yet. Reserving the first code per family in milestone_39_0 makes downstream PRs easier to land.
- **R10.** `internal_docs/integer_model.md` line 494 changes the example to `@rust(bridge.net.set_flags)` but the surrounding paragraph still talks about FFI generally. Consider replacing the remaining "FFI" terminology in the §Rust Interop subsection (lines 493–509) with "Rust interop" for consistency with the rest of the rename.

---

## Open questions blocking implementation readiness

Only the items above marked **B1–B10** block implementation. Of those, **B1, B2, B3, B5, B6, B7, B8** are pure spec gaps that can be closed by editing `internal_docs/rust_interop_architecture.md` without changing the phase milestones. **B4 and B9** require small architecture additions (panic poisoning rule; cache key extension). **B10** is a sequencing decision that needs an explicit answer from the roadmap owner.

No hidden fallback path, no backwards-compatibility shim, and no vague MVP phrasing was detected. Stale references to the prior "Phase 39 = Stable GA" numbering have all been updated in active phase files; archive issues retain the old numbers, which is appropriate.
