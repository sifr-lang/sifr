# Integer Model — Phase-Plan Review Pass 1

Scope: principal-engineer implementation-readiness review of the split between
`internal_docs/integer_model.md` (semantic source of truth) and
`issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` (ad-hoc
implementation phase), against the cross-cutting commitments in
`internal_docs/architecture.md`. Goal: identify blocking gaps before INT-0
closes and the team starts cutting code in INT-1.

Files inspected:

- [internal_docs/integer_model.md](../internal_docs/integer_model.md)
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
- [internal_docs/architecture.md](../internal_docs/architecture.md)

## Verdict

**Not yet implementation-ready.** The semantic doc is in good shape — most of
the under-specified surfaces from review passes 1–5 have been folded in — and
the milestone shape is roughly correct. But the contract-lock milestone (INT-0)
is **incomplete in practice**: `architecture.md` still contains four
load-bearing statements that contradict the new model, the validation `rg`
pattern in INT-0 is too narrow to catch them, and several boundary contracts
named in the design doc have no corresponding milestone owner. There are also
two milestones (INT-2, INT-6) whose scope is large enough that they should be
broken down before INT-1 starts so the next reviewer can see what "done" means.

The blocking issues are listed first; everything else is concrete recommendations
or optional follow-ups.

---

## Blocking issues

### B1. INT-0 has not actually locked the contract

INT-0's acceptance criteria say "No canonical docs still describe `int` as Rust
`i64`" and "Architecture points at the internal design doc and this
implementation issue", and the validation step is

```
rg "int = i64|Type::Int.*i64|bigint.*default|wraps in release" internal_docs docs issues demos crates verification
```

That `rg` pattern misses the four places in `internal_docs/architecture.md`
where the legacy model is still the architectural commitment, not just a
historical aside:

1. [architecture.md:817-824](../internal_docs/architecture.md#L817) — the `Type`
   enum definition still puts `Int` under `// Primitives (Copy)`. This is the
   single source of truth referenced by [architecture.md:319](../internal_docs/architecture.md#L319)
   ("Every `Type` variant must have exactly one Rust representation. The
   `rust_type()` method on `Type` is the single source of truth for this
   mapping."). Leaving `Int` in the `Copy` block directly contradicts
   [integer_model.md:436](../internal_docs/integer_model.md#L436) and
   [architecture.md:323](../internal_docs/architecture.md#L323) which both say
   `int` is **not** Rust `Copy`. INT-0 cannot legitimately close until this
   block is rewritten with `Int` outside the `Copy` tier and the eight
   fixed-width variants plus `ISize`/`USize` added (or explicitly deferred to
   INT-2 with a placeholder comment).
2. [architecture.md:859-860](../internal_docs/architecture.md#L859) — the
   `Type` enum still carries `// Arbitrary-precision integer
   (milestone_integer_safety)` / `BigInt,`. The design doc says public `bigint`
   is gone, with at most a transition alias.
3. [architecture.md:920](../internal_docs/architecture.md#L920) — the Ownership
   Model section still asserts: "Primitive types (`int`, `float`, `bool`) are
   `Copy` -- assignment copies". This is the cross-cutting contract that
   downstream milestones (borrow-default, codegen, generics) read; it cannot
   silently disagree with the integer-model amendment.
4. [architecture.md:517](../internal_docs/architecture.md#L517) — the Error
   Hierarchy table row reads: ``OverflowError | Error | message: str |
   `bigint`-to-`int` conversion via `int(b)` ``. Public `bigint` no longer
   exists, so the documented use case for `OverflowError` is wrong; it should
   describe fixed-width narrowing instead.

**Recommendation:** Treat INT-0 as not done. Either land an architecture-doc
patch as part of INT-0 that fixes all four sites, or break out a small INT-0a
sub-milestone "Architecture amendment landing" with explicit acceptance
criteria pointing at those four anchors. Tighten the validation grep:

```
rg -n 'Primitives \(Copy\)|`int`.*are `Copy`|BigInt,?\s*$|bigint.*int\(b\)|int\s*=\s*i64|Type::Int[^A-Za-z]' \
   internal_docs docs issues demos crates verification
```

### B2. New error classes are not registered anywhere

[integer_model.md:130-136, 330, 337](../internal_docs/integer_model.md#L130)
introduces five new error variants used in the typed return signatures of
arithmetic and JSON I/O:

- `ArithmeticLimitError`
- `FloatOverflowError`
- `FloatPrecisionLossError`
- `JsonIntegerRangeError`
- `JsonLimitError`

None of these appear in
[architecture.md:498-517](../internal_docs/architecture.md#L498) (the Error
Hierarchy table that the rest of the compiler reads to plan codegen and
exhaustiveness). The phase plan does not mention adding them. Without explicit
registration the exhaustiveness checker (cross-cutting contract #3,
[architecture.md:367](../internal_docs/architecture.md#L367)) cannot enforce the
arithmetic contract, and `try`/`except` exhaustiveness will silently regress.

**Recommendation:** Add an explicit deliverable to INT-3 (and a parallel one
in INT-5 for the JSON variants) of the form:

> Register `ArithmeticLimitError`, `FloatOverflowError`,
> `FloatPrecisionLossError` (INT-3) and `JsonIntegerRangeError`, `JsonLimitError`
> (INT-5) in the Error Hierarchy table in `internal_docs/architecture.md` and
> in the canonical built-in error registry, with parent class, fields, and
> `Display` rules specified.

Without this, INT-3 and INT-5 acceptance criteria ("typed errors instead of
panicking", "typed serialization errors") cannot be verified — any class with
a matching name will satisfy a textual test.

### B3. Crate ownership is undefined for SifrInt and JSON profiles

The phase plan identifies *what* needs to change but not *where*. There is no
`sifr_runtime` crate in the workspace
([architecture.md:226-249](../internal_docs/architecture.md#L226)), and the
phase plan does not say where `SifrInt`, the small/big representation, parsing
helpers, formatting helpers, and the explosive-op budget controls live. The
two reasonable options have very different blast radii:

- Add a new `sifr_runtime` crate that emitted Rust code links to (cleanest
  separation, but adds a workspace member, dependency on `num-bigint`, and a
  Cargo manifest emission change in `sifr_codegen`).
- Vendor `SifrInt` inside `sifr_codegen` as a generated module that is
  re-emitted into every project (no new crate, but every generated project
  carries the same hand-written code and `num-bigint` becomes a transitive
  generated dep that codegen has to register).

INT-1 does not commit to either. The same ambiguity applies to INT-5 (JSON
profiles — `sifr_std`? a new `sifr_serde` adapter? inline in `sifr_codegen`?)
and INT-6 (dtype kernels).

**Recommendation:** Make INT-1 explicitly own the crate-placement decision
with one of the two options above, recorded in the milestone scope. Make INT-5
explicitly own the placement of JSON profile machinery (most likely
`sifr_std`, but the phase needs to commit). This unblocks the workspace
`Cargo.toml` change, `cargo deny`/audit policy, and the codegen Cargo manifest
emitter.

### B4. INT-2 conflates two boundaries: AST literal vs HIR/type-system literal

INT-2 says "Change `LiteralInt` and related AST/HIR internals away from `i64`."
The Sifr AST is the Ruff fork submodule
([architecture.md:222-249](../internal_docs/architecture.md#L222),
`third_party/ruff/crates/ruff_python_ast` aliased as `sifr_python_ast`), which
is on a maintenance branch (`sifr/0.15.12-maintenance`). Changing
`ruff_python_ast::Int` to a non-`i64` representation is a much larger change
than changing `sifr_hir::LiteralInt` and forces a rebase strategy on the
upstream submodule; doing it in `sifr_hir` only requires a lossless translation
at the AST → HIR boundary.

The phase plan does not name which boundary changes, and
[architecture.md:832](../internal_docs/architecture.md#L832) already shows
`LiteralInt(SifrIntLiteral)` in the `Type` enum — implying HIR/type-system
ownership — but says nothing about the parser-side `Int` token.

**Recommendation:** Split INT-2 into two explicit deliverables:

- INT-2a (AST boundary): keep Ruff's `Int` token unchanged at parse time;
  introduce a parser-driver shim that captures the original lexeme as a
  decimal/hex/oct/bin string when the token's `i64` representation would be
  lossy, and fail fast with a typed parse error if the token text is malformed.
  This avoids touching the submodule.
- INT-2b (HIR / type-system boundary): replace `sifr_hir::LiteralInt` and
  `sifr_type_system::LiteralInt` with an arbitrary-precision representation
  (normalized base-10 string + cached `num_bigint::BigInt`, or a
  `SifrIntLiteral` enum mirroring `SifrInt`).

Without that split, INT-2 acceptance ("`x: int = 10 ** 100` type-checks") may
appear to pass for the constructed-AST path while still failing the parsed
path because the lexer truncated at `i64::MAX` upstream.

### B5. Dict-key and set-key hash coherence has no implementation owner

[integer_model.md:191-198](../internal_docs/integer_model.md#L191) commits to:

> If two hashable exact/fixed integer values compare equal, their hashes must
> agree. `assert hash(int(1)) == hash(int8(1))`.

This is a non-trivial codegen contract because in Rust, `i32`, `u8`, and a
custom `SifrInt` newtype each have their own `Hash` impl, and `HashMap<K, V>`
hashes the *key type*, not a normalized integer value. To honor the
cross-family hash rule, every fixed-width integer used as a dict/set key must
either:

- be hashed via a Sifr-defined `IntegerKey` adapter that normalizes to
  `SifrInt::Small(i128)`/`SifrInt::Big` before hashing, or
- be coerced to `SifrInt` at the point of dict/set storage (which gives up the
  fixed-width memory benefit at the dict layer).

The phase plan does not call out either approach. The closest mention is INT-3
"cross-type hash coherence" inside a bullet on arithmetic, which is the wrong
milestone for a container codegen change. The Validation Matrix in the design
doc references "Cross-type dict/set lookup" but the phase plan has no scope
item that delivers it.

**Recommendation:** Add an explicit scope bullet in INT-4 (or a new INT-4.5):

> Implement integer dict/set key hashing such that `dict[int, V]` and
> `dict[int32, V]` produce the same hash for the same mathematical value, and
> `dict[int, V]` accepts a fixed-width key by widening at the boundary.
> Validate via cross-family `dict.get`/`set.contains` fixtures.

Without it, the hash-coherence claim is unimplementable from the current plan
and silently breaks `dict[int | int32, V]`-shaped union containers.

### B6. INT-6 dtype contract lock vs deferred runtime work is conflated

INT-6 is gated by data-science/AI runtime surfaces that do not exist yet, and
the milestone hedges with "or blocked behind a tracked runtime issue if arrays
are not implemented yet" ([phase plan:232](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L232)).
That hedge mixes two different things:

- the *contract* (type names, dtype names, scalar↔dtype promotion rules,
  default overflow policy on dtype arithmetic, schema-driven loaders) — which
  must be locked by this phase to prevent later milestones from defaulting to
  unsafe behavior, and
- the *implementation* (actual array/tensor/dataframe kernels, Arrow/Parquet
  loader integration) — which legitimately blocks on the data-science phase.

If both stay inside INT-6, the phase will close on the strength of "no array
runtime exists yet, so nothing to validate" without locking the dtype contract,
and a future data-science milestone is free to ship `array[int32] +
array[int32] -> array[int32]` with silent wrap because INT-6 left no test
gate behind.

**Recommendation:** Split INT-6 into:

- INT-6a (contract lock, must close in this phase): land the dtype rules as
  type-system stubs / type-checker rules / blocked-fixture comments / a
  `verification/contracts/` document that future runtime work is tested
  against. Acceptance criterion: a future `array[int32] + array[int32] ->
  array[int32]` PR fails type-check today.
- INT-6b (deferred to data-science phase): actual kernels. Acceptance is owned
  by that phase, not by this one.

This turns the hedge into a discrete commitment.

---

## Significant gaps (non-blocking but worth fixing before INT-1)

### G1. `bytearray` is missing entirely from the design doc

[architecture.md:97](../internal_docs/architecture.md#L97) lists `bytearray` as
a first-class type with the same CPython parity contract as `bytes`. The
integer-model doc's "Bytes, Arrays, DataFrames, and Tensors" section
[integer_model.md:251-253](../internal_docs/integer_model.md#L251) only
discusses `bytes`. If `bytearray[i] = 256` is allowed, and `bytearray` is
otherwise structurally similar to `bytes`, the same `uint8` element rule and
the same fitting/narrowing rule must apply, and assignment-into-element needs
to be specified (fitting literal vs fallible narrowing vs explicit `uint8(...)`
constructor). Add a paragraph mirroring the `bytes` rules and explicitly
covering the *write* path.

### G2. Compile-time evaluator scope and budget are under-specified

[integer_model.md:87-104](../internal_docs/integer_model.md#L87) defines the
v1 const-evaluable subset (literals, unary `+`/`-`, `+`/`-`/`*`, shifts with
constant counts, non-negative `**` within budget, parens, immutable module
constants whose initializer is const-evaluable). Two real questions are not
answered:

- Cross-module const evaluation: if `LIMIT: int = 200` lives in module A and
  is imported into module B, is the import edge guaranteed to carry the
  const-evaluable status? The HIR query layer
  ([architecture.md:230](../internal_docs/architecture.md#L230)) needs to
  expose this; today no milestone scopes it. Add a bullet in INT-2 covering
  cross-module const propagation.
- The "compile-time budget" is referenced but never quantified. The runtime
  arithmetic-limit budget is "configurable maximum output bit length"
  ([integer_model.md:136](../internal_docs/integer_model.md#L136)), but the
  compile-time fitting evaluator needs its own ceiling so a malicious
  `2 ** 10**8` literal does not hang the type checker. Specify a default
  (e.g., 4096 bits matching the parser digit limit) and a configuration
  surface, and add a negative compile-time test in INT-2.

### G3. `int128` / `uint128` reservation has no diagnostic plan

[integer_model.md:64](../internal_docs/integer_model.md#L64) says `int128` and
`uint128` are reserved as future names. The phase plan repeats this in INT-2
scope but adds no acceptance criterion or diagnostic for what happens when a
user writes `x: int128 = 0` today. If the parser silently treats `int128` as
an unknown identifier the user gets a confusing "name not in scope" error; the
intent is "reserved name, not yet supported". Add a stable `SIFR-*` diagnostic
code under INT-2 specifically for these reserved-but-unimplemented names, and
a fixture that asserts the diagnostic shape.

### G4. Generic numeric bounds are under-specified relative to existing protocols

[integer_model.md:200-209](../internal_docs/integer_model.md#L200) states that
`T + T -> T` is invalid for fixed-width types because the operator output
widens to `int`. But Sifr already has the `Addable` protocol
([architecture.md:770](../internal_docs/architecture.md#L770)) with `Add (+
Sum for sum())` mapping. Two open questions:

- Does `int32: Addable` continue to hold under the new model? If the operator
  returns `int`, the `Add` Rust mapping (`Add<Output = Self>`) no longer
  matches, and any existing generic `def f[T: Addable](...) -> T` applied to
  `list[int32]` would suddenly fail to monomorphize.
- What protocol covers the "exact integer" surface in generic bounds?
  `T: Integer`? `T: ExactInteger`? Today no protocol expresses "any of `int`,
  `int8..int64`, `uint8..uint64`" with the right output-type semantics.

The doc gestures at this with `def sum_int32(values: list[int32]) -> int`
([integer_model.md:202-208](../internal_docs/integer_model.md#L202)) but does
not commit to a generic surface. Add a paragraph naming whether `Addable` is
narrowed/redefined under this model, whether a new `Numeric`/`Integer`
protocol is needed, and which milestone delivers it (INT-3 is the natural
home).

### G5. `int` × `float` equality and ordering semantics are ambiguous

[integer_model.md:189](../internal_docs/integer_model.md#L189) says: "Equality
and ordering compare mathematical values, not bit patterns." Combined with
[integer_model.md:187-188](../internal_docs/integer_model.md#L187) saying
`int + float` is fallible unless the integer is exactly representable, this
implies `int(2**53 + 1) == float(2**53 + 1)` evaluates the comparison exactly
(returning `False` in this case, because the float is `2**53`). This needs to
be explicit because Rust `i64.partial_cmp(&f64)` does not exist out of the box
and the obvious `as f64` cast loses precision. Specify whether equality and
`<`/`>` against `float` are exact (and document the algorithm) or fallible
(returning `Result[bool, FloatPrecisionLossError]`). Add a Validation Matrix
row covering this.

### G6. INT-0 acceptance criterion "agent review artifacts present" is weak

INT-0 has a checkbox for "agent review artifacts for design and phase plan
are present under `reviews/`". This is true today (passes 1–5 plus this pass)
but is not a gate that prevents INT-0 from regressing. A cleaner gate is "the
two specific artifact filenames are listed in the Review History block of the
phase issue, and the most recent one is human-acknowledged". The current
Review History does name the prior passes but there is no rule that the next
phase milestone cannot proceed without a specific human ack — the checkbox
"Phase-plan review pass 1 completed after splitting design and milestones"
will be ticked by whoever closes this review, with no second signature.
Optional, but worth tightening.

### G7. Performance acceptance is qualitative

INT-8's acceptance criterion "Common small-`int` loops do not allocate on
every iteration" is not measurable as written. It needs:

- A concrete benchmark fixture (e.g., `verification/perf/sifr_int_loop.sifr`
  doing `for i in range(0, 10_000_000): total = total + i`).
- A concrete acceptance threshold (e.g., "zero heap allocations in the loop
  body as measured by `dhat-rs`/`heaptrack`/`cargo flamegraph`", or "throughput
  within 2x of equivalent `i64`-only Rust loop").
- A fail mode (regression > X% fails the milestone).

Without this, INT-8 closes whenever someone declares it good enough. Tie it to
Phase 35's performance-budget infrastructure
([phases/35_performance_benchmarking_and_budgets.md](../internal_docs/phases/35_performance_benchmarking_and_budgets.md))
if it is online.

### G8. Diagnostic codes are promised but not allocated

[integer_model.md:432](../internal_docs/integer_model.md#L432) ends with
"These diagnostics should use stable `SIFR-*` codes when implemented" but
neither the design doc nor the phase plan allocates a code range. INT-7 says
"Add stable diagnostic codes for integer range, narrowing, unsafe division,
float precision, bool comparison, JSON policy, and dtype overflow-policy
errors" without naming them. The diagnostic registry (`sifr_diagnostics`,
[architecture.md:231](../internal_docs/architecture.md#L231)) is the canonical
home for these — pre-allocate the seven families a code range now (e.g.,
`SIFR-INT-001..099`) so each milestone (INT-2 emits `SIFR-INT-010` for
fitting, INT-3 emits `SIFR-INT-020` for division, etc.) has a concrete code
to test against. This also lets the cross-references between
`integer_model.md` and the issue tracker be hyperlinked rather than vague.

---

## Smaller findings

- **`SifrInt` enum naming.** The doc ([integer_model.md:36-41](../internal_docs/integer_model.md#L36))
  commits to `Small(i64) | Big(Box<num_bigint::BigInt>)`. Worth noting in
  INT-1 scope that the variant tags are part of the generated-code ABI and
  any later change (e.g., `Small(i128)` for fewer big-spills) is a breaking
  change to every generated project. Either commit and document it, or stage
  it behind a non-public `#[doc(hidden)]` representation. Optional.
- **`num-bigint` vs alternatives.** The doc names `num_bigint::BigInt`
  specifically. Alternatives (`malachite::Integer`, `dashu::Integer`) have
  different perf and dependency footprints. Locking `num-bigint` pre-perf
  evaluation is fine for INT-1 but should be revisited at INT-8 with a swap
  spike or an explicit "we are committing" note. Optional.
- **`bytes` vs `array[uint8]`.** [integer_model.md:253](../internal_docs/integer_model.md#L253)
  correctly says `bytes` is not an alias for `array[uint8]`. Worth adding a
  one-line note that `array` itself is an as-yet-unimplemented dtype-bearing
  builtin (see INT-6 risk above) so a reader doesn't infer that it exists
  today.
- **`isize`/`usize` source visibility.** [integer_model.md:62](../internal_docs/integer_model.md#L62)
  says `isize`/`usize` are "FFI/low-level interop types only" and
  [integer_model.md:241](../internal_docs/integer_model.md#L241) says
  exposing `usize` is "limited to Rust FFI signatures or explicit low-level
  modules". The "explicit low-level modules" carve-out is undefined — add a
  sentence specifying the gating mechanism (a module attribute, a
  per-package opt-in, or a directory convention).
- **Pickle / serde derive.** The model commits to JSON, SQL, Arrow/Parquet,
  and Rust FFI as boundary types but does not say what happens when a Sifr
  user writes `class Foo: x: int` and the codegen needs to derive
  `serde::Serialize`. Implicit answer is "uses the active JSON profile", but
  the doc never says it. Add a paragraph in "Serialization and External
  Boundaries" naming the default-derive policy. Optional but worth resolving
  before INT-5.
- **REPL/debug formatting.** Not mentioned. If sifr ships a REPL or debugger
  surface, large `int` values need a display rule. Defer.
- **HIR maintainability guardrails script.** `scripts/check_hir_maintainability_guardrails.py`
  may need updating when the HIR gains eight new fixed-width type variants.
  Worth flagging in INT-2 as a known follow-up so the validation step does
  not surprise the implementer. Optional.
- **Migration scope of existing demos/fixtures.** [architecture.md:1079](../internal_docs/architecture.md#L1079)
  shows `x: int = "hello"` as an existing test. Every demo/fixture that
  passes an `int` value through ownership-sensitive code paths becomes a
  fixture under the new non-`Copy` semantics. INT-1 acceptance includes an
  e2e fixture for repeated use of an `int` after calls but does not call out
  the regression risk to *every* existing fixture. Add a scope note: "expect
  bulk fixture and codegen-snapshot churn during INT-1; reset baselines via
  `cargo insta review`."

---

## Recommendations summary

Land before INT-0 closes:

1. Patch `internal_docs/architecture.md` at the four anchor points called out
   in **B1**.
2. Tighten the INT-0 validation `rg` pattern (**B1**).
3. Register the five new error classes and assign owners (**B2**).
4. Commit on crate placement for `SifrInt` and JSON profiles (**B3**).
5. Split INT-2 into AST-boundary vs HIR-boundary deliverables (**B4**).
6. Add a hash-coherence implementation owner in INT-4 (**B5**).
7. Split INT-6 into contract-lock and deferred-implementation halves (**B6**).

Land before INT-1 starts:

8. Specify `bytearray` element rules (**G1**).
9. Quantify the compile-time evaluator budget and cover cross-module const
   propagation (**G2**).
10. Allocate diagnostic codes for `int128`/`uint128` reservations and the
    seven diagnostic families (**G3**, **G8**).
11. Resolve generic numeric bounds against the existing `Addable` protocol
    (**G4**).
12. Specify `int` × `float` equality semantics (**G5**).
13. Replace INT-8's qualitative perf criterion with a concrete benchmark and
    threshold (**G7**).

Optional / opportunistic:

14. Tighten the review-history gate (**G6**).
15. Resolve `bytes` vs `array[uint8]`, `isize`/`usize` low-level module
    carve-out, serde-derive default policy, `num-bigint` swap spike, REPL
    formatting, and HIR guardrail script updates as called out in the
    "Smaller findings" list.

Once **B1–B6** are addressed and **G1–G8** have explicit owners, the phase
plan is ready to execute.
