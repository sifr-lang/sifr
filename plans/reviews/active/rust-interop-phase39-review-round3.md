# Phase 39 Rust Interop Design — Review (Round 3)

## Verdict

**No blocking gaps remain.** The current diff cleanly folds in Round 2's O1–O6 (and the verification-tree dedup from O7/R8). The R10 FFI-terminology pass in `integer_model.md` §Rust Interop is also done (`FFI structs` → `Low-level interop structs`). Phase doc and architecture mirror each other on bridge naming, ordered dicts, `SifrIntBridge`, `@rust.async`, profile/panic cache inputs, poisoned-handle behavior, current-thread non-`Send` futures, and milestone_39_12 closure criteria. The design is implementation-ready for milestone_39_0.

A few polish nits below; none block.

---

## Round 2 follow-through (sanity check)

| # | Status | Where |
|---|---|---|
| **O1** `IndexMap` path | ✅ | `rust_interop_architecture.md` L291: `sifr_runtime::interop::IndexMap`, "runtime re-export of the pinned `indexmap::IndexMap` version." |
| **O2** `SifrIntBridge` traits | ✅ | L289: `Eq`, `Ord`, `Hash`, `Send`, `Sync`, no `Copy`, no `repr(C)`. |
| **O3** KafkaConsumer example consistent with `Self` rule | ✅ | L138–141: `aclose` now uses bridge shim + `async def`. |
| **O4** `@rust.zero_copy` vs `@rust.view` composition | ✅ | L428: "They compose…" sentence added. |
| **O5** Poisoning actor named | ✅ | L345: generated wrapper marks poisoned automatically; bridge authors do not. |
| **O6** `set[T]`/`tuple[...]` exclusion | ✅ | L295: explicitly produce `SIFR-RUST-TYPE-*`. |
| **O7** Verification tree dedup | ✅ | `39_rust_interop.md` L217–219 now links to the architecture's §Verification Area instead of duplicating the tree. |

No phase drift; cache-key, async-rule, poisoning, and naming language line up across both files.

---

## Optional nits (non-blocking)

- **N1 — Mixed `interop::` shorthand vs full path.** Bridge table row L280 writes `&interop::IndexMap<String, T>`, while the adjacent `int` row writes `&SifrIntBridge` (no prefix). Body paragraphs always use the full `sifr_runtime::interop::…` form. Pick one shorthand convention so bridge authors don't have to guess.
- **N2 — `@rust.opaque` parameter names vs §Opaque Handles vocabulary.** The example at L130–132 uses `borrow=exclusive`, but §Opaque Handles L355 enumerates these values under "ownership model: owned, borrowed, shared, or exclusive." It isn't explicit whether the manifest key is `borrow=` or `ownership=`, nor how the parameter governs Rust `&self`/`&mut self`/`self` lowering (only inferable from the L178 `Self.poll` example using `&mut handle.inner`). Worth one sentence enumerating opaque-decorator keys to remove the implicit mapping.
- **N3 — `panic = "abort"` opt-in mechanism unnamed.** L343 and milestone_39_8 both require explicit opt-in for abort-profile bridge builds, but neither names the `sifr.toml` field, decorator, or CLI flag. Naming it (e.g. a `[trust] allow-panic-abort = ["…"]` key) would close the last ambiguity for milestone_39_8.
- **N4 — Closed-enum discriminant validation scope.** L323 says invalid discriminants returned "through integer or wire adapters" produce runtime conversion errors. Restating that direct-typed `EnumBridge` returns do *not* require validation (because Rust's type system already enforces it) would prevent over-implementation in milestone_39_5.
- **N5 — `Error` superclass link.** Round 2 O7/R6 carryover. The architecture now says "`Error` is Sifr's canonical error base type" but still doesn't cite where `Error` is defined. Trivial.

Nothing here blocks committing the design.
