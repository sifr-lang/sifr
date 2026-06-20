# Phase 39 Rust Interop Design — Review (Round 2)

## Verdict

**No blocking gaps remain.** Every B1–B10 from round 1 is fully resolved in the unstaged follow-up. The design is implementation-ready across language, compiler, Cargo, runtime, zero-copy/tensor, diagnostics, and verification. The remaining findings below are minor polish — none block milestone_39_0 or downstream milestones.

---

## B1–B10 Resolution Check

| # | Status | Where |
|---|---|---|
| **B1** Exact-int bridge | ✅ | `rust_interop_architecture.md` §Bridge Type Contract table + paragraph at L287 names `sifr_runtime::interop::SifrIntBridge`, ownership/borrow forms, `Copy`/`repr(C)` status; `integer_model.md` L486 references the same path. |
| **B2** Dict ordering | ✅ | Table row + L289 paragraph maps `dict[str,T]` to `IndexMap<String,T>`; non-`str` keys explicitly deferred. |
| **B3** `@rust.view` | ✅ | New §Zero-Copy and Views block (L424–439) defines parameters (`owner`, `lifetime`, `mutability`, `send`, `sync`), enumerates `call`/`owner`/`static`, distinguishes from `@rust.zero_copy`. |
| **B4** `AssertUnwindSafe`/poisoning | ✅ | §Error Semantics L329 + L341 require `AssertUnwindSafe`, document poisoning contract; `SIFR-RUST-PANIC-0001` reserves "poisoned handle". |
| **B5** Non-`Send` futures | ✅ | §Async L399–400 splits the rule; current-thread `thread_affinity=tokio_current_thread` permits !Send; function-level `@rust.async(...)` form added L407. Milestone_39_1 now also parses `@rust.async(...)`. |
| **B6** `Self` resolution | ✅ | New §`Self` Resolution L165–187 specifies inherent-only, shows lowered call site, sends trait methods through bridge shims, names `SIFR-RUST-RESOLVE-*`. |
| **B7** Bridge type naming | ✅ | §User Model L81 + new §Generated Bridge Types L309–319 specify `crate::__sifr_bridge::<sifr_module_path>::<Name>Bridge`, reserve the suffix. |
| **B8** Closed enum repr | ✅ | §Generated Bridge Types L319 specifies `repr(u32)`, declaration-order discriminants, validation-on-cross with `SIFR-RUST-TYPE-*`. |
| **B9** Cache key | ✅ | §Cargo and Build Cache L528–529 add profile name, panic strategy, `lto`, `codegen-units`, `incremental`, target features, target-spec hash, bridge-version schema. Phase doc milestone_39_2 mirrors. |
| **B10** Phase 39→40 ordering | ✅ | `39_rust_interop.md` L10 adds the rationale paragraph. |

---

## Optional refinements (non-blocking)

### O1 — `IndexMap` import path is unspecified
`rust_interop_architecture.md` §Bridge Type Contract uses `IndexMap<String, T>` in bridge author–facing signatures but never says where bridge authors import it from. The handle-vs-runtime indirection used for `SifrIntBridge` (`sifr_runtime::interop::SifrIntBridge`) sets the precedent.
**Suggested change:** add one line near L289 stating either "`IndexMap` is re-exported as `sifr_runtime::interop::IndexMap`" or "bridge crates must declare `indexmap` as a Cargo dependency at the version pinned by `sifr_runtime`."

### O2 — `SifrIntBridge` trait surface incomplete
L287 says "owned, immutable, cloneable" but omits `Send`/`Sync`/`Eq`/`Hash`/`Ord` — all of which `SifrInt` in `integer_model.md` L484 declares. Bridge authors using `SifrIntBridge` as a `IndexMap` key or across threads need this stated.
**Suggested change:** at L287 add "`SifrIntBridge` is `Clone`, `Eq`, `Ord`, `Hash`, `Send`, and `Sync`" (or whichever subset is intended) and link `SifrInt` ↔ `SifrIntBridge` semantically.

### O3 — `KafkaConsumer` User Model example now contradicts new `Self` rule
The User Model example (L121–139) still has `@rust(Self.aclose)` paired with `close=async_close` and a `def aclose` (not `async def`). After the new `Self Resolution` rule (inherent-only) and the bridge-shim recipe at L182, this is inconsistent in two ways: (a) if Rust's `Consumer::aclose` is a trait method, the example should use the bridge shim instead; (b) `async_close` should be modeled by `async def`, not `def`.
**Suggested change:** in `rust_interop_architecture.md` L121–139, either annotate "`aclose` is an inherent method on `bridge.kafka.Consumer`" and switch to `async def aclose(...)`, or replace the second method with the `@rust(bridge.kafka.consumer_aclose)` shim form already shown at L182.

### O4 — Composition of `@rust.zero_copy` and `@rust.view` not stated
Two adjacent examples (L417 `digest_view` with only `@rust.zero_copy`, L427 `tokens_view` with only `@rust.view`) leave the relationship ambiguous: orthogonal, exclusive, or one-implies-the-other? Milestone_39_9 enforces both but the architecture never says which combinations are legal.
**Suggested change:** add one sentence above L424 stating which decorator is required vs optional for borrowed-view bridges (e.g. "`@rust.zero_copy` is required whenever the API must not copy; `@rust.view` is required whenever the return is a borrowed view. They compose and may both apply to a single declaration.").

### O5 — Poisoning actor unspecified
§Error Semantics L341 says bridge code must "leave state observably consistent after a caught panic or mark the handle as poisoned." But the code that panicked cannot itself run poisoning logic; the wrapper has to do it. Without naming the actor, milestone_39_8 cannot decide whether poisoning is generated-glue automatic or bridge-author opt-in.
**Suggested change:** at L341 add "The generated wrapper marks the opaque handle as poisoned automatically when `catch_unwind` returns `Err`; bridge authors do not implement poisoning manually and must not depend on additional bridge code running after a panic."

### O6 — `set[T]`, `tuple[...]`, and other container types absent from bridge table
The §Bridge Type Contract table is presented as the canonical list, but Sifr supports `set` and tuple types that don't appear. Implementation-wise this means `def foo(s: set[str])` decorated `@rust(...)` has no defined contract. Round-1 phrasing "intentionally small and explicit" suggests this is deliberate, but it isn't documented.
**Suggested change:** add a sentence after the bridge table (around L286): "Sifr container types not listed above (`set[T]`, `tuple[...]`, etc.) are not bridge-compatible in Phase 39 and produce `SIFR-RUST-TYPE-*` diagnostics. Future phases may extend the contract; no implicit conversion is allowed."

### O7 — Round-1 R6 and R8 carry over
- **R6** (User Model L60 `class HashError(Error)`): the `Error` superclass is still referenced without a link to its definition; minor.
- **R8** (verification area block): still duplicated verbatim between `rust_interop_architecture.md` L582–629 and `39_rust_interop.md` L217–262. Drift risk. Suggest deleting one and linking the other (architecture is normative — leave that copy, link from the phase doc).

---

## Cross-file consistency

- `integer_model.md` ↔ `rust_interop_architecture.md`: both now agree on `sifr_runtime::interop::SifrIntBridge`. No drift.
- `39_rust_interop.md` ↔ `rust_interop_architecture.md`: milestone_39_1 (`@rust.async`), milestone_39_2 (profile/panic cache inputs), milestone_39_5 (generated bridge naming + ordered dicts + SifrIntBridge), milestone_39_7 (current-thread non-`Send`), milestone_39_8 (poisoned handle), and milestone_39_12 (concrete closure criterion) all mirror the architecture changes.
- No `extern rust`, `dlopen`, fallback, or MVP residue.

## Implementation readiness

Language surface, compiler lowering (`InteropBuildPlan`), Cargo cache key, runtime panic boundary, async runtime semantics, zero-copy/tensor contracts, diagnostic family + first-codes, and verification fixture matrix are all defined to a level where milestone_39_0 can land without further architecture edits. O1–O7 above are polish that can be folded into milestone_39_0's documentation pass.
