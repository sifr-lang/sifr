I have enough material. Compiling Round 4.

# Phase 39 Rust Interop Design — Review (Round 4)

## Verdict

**One blocker remains plus several polish nits.** Round 3's N1–N5 are substantively addressed (panic-abort named, enum validation scope clarified, `Error` linked, opaque decorator keys enumerated). The one remaining blocker is the same shape as Round 1's B1: a bridge-table type is named but its concrete Rust definition is missing — this time `Handle<T>` instead of `SifrIntBridge`.

---

## Findings (severity-ordered)

### B11 — `Handle<T>` is undefined (Blocker)

**File:** `internal_docs/rust_interop_architecture.md` §Bridge Type Contract, row "opaque class" (L318); also `Handle<T>::inner` accessed in the lowered example at L185.

The bridge table says opaque classes lower to `&Handle<T>` / `&mut Handle<T>` / `Handle<T>`, and §Self Resolution lowers `Self.poll` to `crate::bridges::kafka::Consumer::poll(&mut handle.inner)`. But no section names the crate path, struct definition, fields, poison flag layout, or trait bounds of `Handle<T>`. This is the same gap shape as Round 1's `SifrIntBridge`: the contract is testable only against a concrete type.

Concrete missing items:
- Where `Handle<T>` lives (`sifr_runtime::interop::Handle<T>` analogous to `SifrIntBridge`?).
- Whether `inner` is a public field, an accessor, or pinned.
- How the poison/closed bits are represented and where the generated wrapper toggles them.
- Whether `Handle<T>: Send/Sync` is derived from `T` or is opt-in through `@rust.opaque(send=, sync=)`.
- Whether bridge authors writing free functions taking owned `Handle<T>` import it from runtime or generated bridge namespace.

**Suggested change:** add an "Opaque Handle Representation" subsection adjacent to §Generated Bridge Types stating:
1. `Handle<T>` lives at `sifr_runtime::interop::Handle<T>` (or `crate::__sifr_bridge::Handle<T>` — pick one);
2. layout: `T` storage + closed flag + poisoned flag, with explicit accessor names;
3. `Send`/`Sync` derivation rule from `@rust.opaque(send=, sync=)` plus the inner type;
4. how `inner` is referenced by generated `Self.method` lowering vs. user bridge functions taking owned/borrowed handles;
5. close-state transition rules (`mark_closed`, `mark_poisoned`) and which only the generated wrapper may invoke. Then update the L185 lowering example to reference the named accessors instead of `handle.inner`.

Without this, milestone_39_6 cannot generate handle wrappers and milestone_39_4 cannot validate the opaque row of the bridge table.

---

## Optional refinements (non-blocking)

### O8 — `panic=map_error` adapter shape is named but never illustrated

**File:** `internal_docs/rust_interop_architecture.md` §Panic Surface Policy (L375).

`panic=map_error` is one of three accepted ways to inject `RustPanicError` into the declared error channel, but the surface — whether `panic=map_error=adapter_fn`, `panic=(map_error, adapter_fn)`, or a separate `@rust.panic(map=...)` decorator — is not shown. Without an example, milestone_39_1's parser cannot decide between these forms, and milestone_39_8's adapter validation has nothing to bind to.

**Suggested change:** add a one-line example next to the policy bullet showing the exact decorator surface, e.g. `@rust(bridge.x.y, panic=map_error(map_panic_to_x))`, and state whether the adapter function is resolved through the same dotted-path rules as `@rust` targets.

### O9 — `IndexMap` shorthand vs full path is now inconsistent within one section

**File:** `internal_docs/rust_interop_architecture.md` bridge-table row L313 vs paragraph L324.

The table writes `&interop::IndexMap<String, T>` (relative shorthand) while the adjacent paragraph writes `sifr_runtime::interop::IndexMap` (full path). The `int` row writes `&SifrIntBridge` (no prefix). Round 3 N1 flagged this; the latest pass closed body text but the table row still uses the `interop::` shorthand. Pick one convention (recommend full `sifr_runtime::interop::…` everywhere a bridge author would type the path).

### O10 — Ownership-model vocabulary drift between §Opaque Types and §Opaque Handles

**File:** `internal_docs/rust_interop_architecture.md` L406 vs L145.

§Opaque Handles enumerates ownership models as "owned, borrowed, shared, or exclusive" (four values). §Opaque Types defines `borrow=` accepting "shared / exclusive / owned" (three values, no "borrowed"). It's recoverable by reading both sections, but a reader will ask: is "borrowed" a real value of `borrow=`? Either drop "borrowed" from L406 or document what it means as a `borrow=` value.

### O11 — `@rust.opaque(type=...)` resolves a Rust type, not a callable, but the resolution table only covers callables

**File:** `internal_docs/rust_interop_architecture.md` §Path Resolution table L154–159 and §Opaque Types L121.

`@rust.opaque(type=bridge.kafka.Consumer)` passes a Rust *type* path through the same machinery. The resolution table reads as if every root resolves a function or method. Add one sentence stating the four roots also resolve Rust type paths for `@rust.opaque(type=...)` and that probe code constructs a type-existence assertion (e.g. `const _: fn() -> Option<bridge::kafka::Consumer> = || None;`).

### O12 — Same-workspace crate declaration is asserted but not exemplified

**File:** `internal_docs/rust_interop_architecture.md` §Package Layout L210–212 and §Path Resolution L169.

The layout shows a sibling `backend/` directory and the doc says "Same-workspace crates are not special. They must still be declared as ordinary Cargo dependencies." A reader has to infer the resolution: add `tokenizer_backend = { path = "backend" }` in `Cargo.toml`, then `@rust(tokenizer_backend.encode)`. Show the two-line example. Milestone_39_3 explicitly covers "same-workspace dependency behavior", so the example pays for itself.

### O13 — "Sifr's `Send + 'static` equivalent" is undefined

**File:** `internal_docs/rust_interop_architecture.md` §Callbacks L532.

Sifr does not surface `Send`/`'static` as user-visible traits, but the doc requires "captured values to satisfy Sifr's `Send + 'static` equivalent." Either name the Sifr-side rule (likely an effect from `@cpu_heavy`/`@blocking_io`/task-spawn analysis) or link to where that rule is defined. As written, milestone_39_11 has to invent the rule.

### O14 — One remaining "FFI" reference in `integer_model.md` validation matrix

**File:** `internal_docs/integer_model.md` Validation Matrix row "Interop" (around L538).

Body text and the §Rust Interop subsection are now consistent ("Low-level interop structs", "Rust interop"). The validation matrix row still uses "FFI": `"passing exact int to FFI without explicit narrowing"` and `"usize/isize in FFI signatures"`. Replace `FFI` with `Rust interop` (or `low-level interop`) for consistency with the rest of the rename. Trivial.

### O15 — `bridge-version = 1` schema content is not enumerated

**File:** `internal_docs/rust_interop_architecture.md` L246.

The doc says a bridge-version mismatch fails package validation but does not list what `bridge-version = 1` formally covers (e.g. "bridge type naming `crate::__sifr_bridge::<module>::<Name>Bridge`, `repr(u32)` closed-enum discriminants, `sifr_runtime::interop::{SifrIntBridge, IndexMap}` versions, `Handle<T>` layout, `__sifr_bridge` namespace reservation"). Tabulating these once means future schema bumps are unambiguous.

### O16 — `Handle<T>` poisoning vs `mark_closed` ordering is implicit

**File:** `internal_docs/rust_interop_architecture.md` L148 and L396.

Two adjacent contracts: `own self` close paths "mark the Sifr handle closed before returning success, and mark it poisoned if the Rust call panics," and the panic wrapper "marks the opaque receiver plus any mutable or owned opaque handles passed to the Rust call as poisoned automatically." When a panic occurs *during* close, both "closed" and "poisoned" fire — does the wrapper resolve to "poisoned wins" (re-entry returns the panic error, not a closed-handle error)? Tied to B11's representation: once `Handle<T>` is named, document the state-transition table.

### O17 — `bridge-version` archive metadata duplication

**File:** `internal_docs/rust_interop_architecture.md` L242 vs L246.

`bridge-version` appears twice: as required archive metadata at L242 and as the schema-version anchor at L246. They are the same value, but reading the archive bullet first makes it sound like a separate file. Reword L242 as "...declared `src/bridges/*.rs` files, Sifr-managed projection files, and the `[rust].bridge-version` value declared in `sifr.toml`."

---

## Cross-file consistency

- `integer_model.md` ↔ `rust_interop_architecture.md`: `SifrIntBridge` definition lines up (path, trait surface, Copy/repr stance). Only the validation matrix "FFI" residue diverges (O14).
- `39_rust_interop.md` ↔ `rust_interop_architecture.md`: milestone scopes mirror the architecture for opaque keys, panic surface vocab, profile/panic cache inputs, enum discriminant validation scope, and `[trust].rust-panic-abort` opt-in. milestone_39_6 references "Send/Sync policy" without naming `Handle<T>` — the gap surfaces here too.
- `roadmap.md` and `plans/phases/index.md` both carry Phase 39 = Rust Interop, Phase 40 = Stable GA. Consistent with the round-1 B10 rationale (now at `39_rust_interop.md` L10).
- No `extern rust`, `dlopen`, fallback, or MVP residue. Reserved-root rule (`bridge`/`Self`) is internally consistent and the diagnostic family is named.

---

## Implementation readiness

Language surface, declaration syntax, panic boundary contract, async runtime semantics, zero-copy/view composition, trust pre/post split, cache-key inputs, generated bridge naming, closed-enum repr, and verification fixture inventory are implementation-ready.

The **`Handle<T>` representation gap (B11)** is the only thing that blocks milestone_39_4 + milestone_39_6 from starting cleanly. It is a small spec addition (a subsection with type path + fields + state-flag semantics), comparable in size to Round 1's `SifrIntBridge` fix. Everything else is polish that can ride along with the same milestone_39_0 documentation pass.
