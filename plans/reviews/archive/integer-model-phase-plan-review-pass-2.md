# Integer Model — Phase-Plan Review Pass 2

Scope: principal-engineer follow-up review confirming whether the pass-1
blockers (B1–B6) and significant gaps (G1–G8) raised in
[reviews/integer-model-phase-plan-review-pass-1.md](integer-model-phase-plan-review-pass-1.md)
have been resolved in the current uncommitted edits to
`internal_docs/integer_model.md`,
`issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`,
`internal_docs/architecture.md`, and the new
`verification/integer_model_implementation_inventory.md`.

Files inspected:

- [internal_docs/integer_model.md](../internal_docs/integer_model.md)
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
- [internal_docs/architecture.md](../internal_docs/architecture.md)
- [verification/integer_model_implementation_inventory.md](../verification/integer_model_implementation_inventory.md)
- [reviews/integer-model-phase-plan-review-pass-1.md](integer-model-phase-plan-review-pass-1.md)

## Verdict

**Implementation-ready.** All six pass-1 blockers (B1–B6) are resolved, and
all eight significant gaps (G1–G8) either have explicit owners and acceptance
criteria or are deferred with a stated rationale. The semantic doc, phase
plan, and architecture amendments now agree, the contract surface is
discoverable from a single canonical entry point, and the milestone shape
supports PR-sliceable execution.

INT-0 can close on this review and INT-1 is ready to start. There are no new
blockers. A small set of polish items remain (mostly per-milestone
diagnostic-code attribution and a still-soft INT-8 perf threshold); they are
worth landing but should not gate the start of implementation.

The rest of the document walks the pass-1 findings one by one, then lists the
remaining polish items separately.

---

## Pass-1 blockers — status

### B1. INT-0 contract lock and architecture anchors — Resolved

All four anchor points called out in pass 1 are fixed in
`internal_docs/architecture.md`:

1. [architecture.md:817-836](../internal_docs/architecture.md#L817) — the
   `Type` enum now places `Int` outside the `// Primitives (Copy)` block with
   the comment `// Exact integer (value-semantic at source, not Rust Copy)`.
   The eight fixed-width variants (`Int8..Int64`, `UInt8..UInt64`) plus
   `ISize`/`USize` are added in a dedicated `// Fixed-width integer
   primitives (Copy)` block, matching
   [integer_model.md:60-63](../internal_docs/integer_model.md#L60).
2. The standalone `BigInt` variant is gone from the `Type` enum. `LiteralInt`
   now carries `SifrIntLiteral`
   ([architecture.md:851](../internal_docs/architecture.md#L851)), aligning
   with the design doc's literal-representation rule.
3. [architecture.md:933-936](../internal_docs/architecture.md#L933) — the
   Ownership Model section now explicitly states "Fixed-width integer types,
   `float`, and `bool` are Rust `Copy` values… Source-level `int` remains
   scalar and value-semantic, but it lowers to `SifrInt` and is not Rust
   `Copy`." Cross-cutting consistency is preserved at
   [architecture.md:323](../internal_docs/architecture.md#L323) (Borrow and
   Lifetime Strategy), [architecture.md:711](../internal_docs/architecture.md#L711)
   (Auto-Derived Traits), and
   [architecture.md:788](../internal_docs/architecture.md#L788) (Standard
   Protocol Primitives) — all four sites describe `int` as non-`Copy` and
   value-semantic, with codegen owning the borrow/clone discipline.
4. [architecture.md:517](../internal_docs/architecture.md#L517) — the
   `OverflowError` row now reads "Fixed-width narrowing and
   representation-preserving fixed-width arithmetic overflow" instead of the
   stale `bigint`-to-`int` description.

The new INT-0 validation pattern in
[issues:78](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L78)
(adding `Type::Int([^A-Za-z0-9_]|$)` and `wraps in release`) is correctly
broader than the original. The expectation that this pattern still surfaces
implementation matches under `crates/sifr_codegen/...` is clarified in
[verification/integer_model_implementation_inventory.md:103](../verification/integer_model_implementation_inventory.md#L103):
"INT-0 uses it to classify remaining matches, not to require a zero-result
tree." That is the right framing — those matches are tracked migration
targets owned by INT-1/INT-2B/INT-3.

Architecture also gains a top-of-file source-of-truth pointer at
[architecture.md:46-50](../internal_docs/architecture.md#L46) and
amendment-aware notes throughout the borrow/codegen/protocol contracts
(e.g., `&Vec<SifrInt>` in
[architecture.md:335-338](../internal_docs/architecture.md#L335),
`enum IntOrStr { Int(SifrInt), Str(String) }` in
[architecture.md:919](../internal_docs/architecture.md#L919)).

### B2. Error class registration — Resolved

The five new error variants are now registered in the canonical Error
Hierarchy table at
[architecture.md:517-522](../internal_docs/architecture.md#L517) with
parent classes and fields:

- `OverflowError` (parent: `Error`)
- `ArithmeticLimitError` (parent: `OverflowError`, fields: `message`, `limit: int`)
- `FloatOverflowError` (parent: `OverflowError`)
- `FloatPrecisionLossError` (parent: `OverflowError`)
- `JsonIntegerRangeError` (parent: `Error`, fields: `message`, `path: str`, `profile: str`)
- `JsonLimitError` (parent: `Error`, fields: `message`, `limit: int`)

INT-0 acceptance now includes the documentation requirement
([issues:71](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L71)),
and INT-3 / INT-5 own the runtime registration in the canonical built-in
error registry
([issues:188](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L188),
[issues:252](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L252)).
Exhaustiveness checking (cross-cutting contract #3) now has a concrete
target.

One minor inconsistency to keep an eye on: the design doc lists
`ArithmeticLimitError` in arithmetic return types
([integer_model.md:132-138](../internal_docs/integer_model.md#L132)) and
also in JSON parsing paths via `JsonLimitError`
([integer_model.md:340](../internal_docs/integer_model.md#L340)). The
architecture table now has both, and the parent-class assignment of
`ArithmeticLimitError` under `OverflowError` is consistent with the design
doc's framing ("Exceeding that budget returns `ArithmeticLimitError`",
[integer_model.md:138](../internal_docs/integer_model.md#L138)). No change
needed.

### B3. Crate ownership for `SifrInt` and JSON profiles — Resolved

The design doc and phase plan both commit to `crates/sifr_runtime`:

- [integer_model.md:461](../internal_docs/integer_model.md#L461): "The
  target runtime placement is a new workspace crate, `crates/sifr_runtime`,
  linked by generated projects through the codegen-emitted Cargo manifest.
  `SifrInt`, integer parsing/formatting helpers, arithmetic budget helpers,
  normalized integer hashing, and JSON integer profile helpers live there
  rather than being re-emitted into every generated Rust file."
- [issues:39-40](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L39)
  and [issues:98-99](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L98)
  in the dependencies section and INT-1 scope: "Create `crates/sifr_runtime`
  as the shared runtime crate… Teach codegen/build materialization to emit
  the generated Cargo dependency on `sifr_runtime` and `num-bigint` through
  that crate rather than vendoring integer helpers into every generated
  file."
- INT-5 explicitly places JSON profile machinery in `sifr_runtime::json`
  ([issues:251](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L251)):
  "Implement profile machinery in `sifr_runtime::json` and expose wrappers
  from future stdlib/web layers instead of duplicating profile logic."

The inventory ([verification:24-26](../verification/integer_model_implementation_inventory.md#L24))
confirms the crate is the home for `SifrInt`, normalized integer hashing,
parsing/formatting, arithmetic budget helpers, and JSON integer profiles.
INT-1 acceptance ("Generated Cargo manifests link the shared runtime crate;
generated files do not carry duplicate hand-written `SifrInt` modules",
[issues:110](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L110))
guarantees codegen does not silently regress to vendor-everywhere.

This unblocks the workspace `Cargo.toml` change, the codegen Cargo manifest
emitter update, and the audit replacement of `RuntimeNeed::BigInt` /
`needs_bigint`
([verification:28-30](../verification/integer_model_implementation_inventory.md#L28)).
INT-6 dtype runtime placement is intentionally deferred to the data-science
phase, which is consistent with B6.

### B4. INT-2 AST/HIR split — Resolved

INT-2 is now split cleanly:

- **INT-2A (Parser Boundary and Literal Capture,
  [issues:122-145](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L122))**
  keeps the Ruff submodule untouched by default and introduces a
  parser-driver / AST-to-HIR shim that captures the original lexeme when
  the parser-side numeric value would be lossy. Acceptance specifically
  guards both paths: "The constructed-AST path and parsed-source path
  produce equivalent HIR literal representations"
  ([issues:137](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L137)).
  Reserved-name diagnostic owner is also explicit (`SIFR-INT-003`,
  [issues:132](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L132)).
- **INT-2B (HIR, Type System, and Const Fitting,
  [issues:147-177](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L147))**
  owns the HIR/type-system literal representation, fixed-width type
  variants, const-evaluable evaluator (with the 4096-decimal-digit budget),
  and HIR maintainability guardrail updates.

The inventory backstops the split: "Parser boundary work should avoid broad
Ruff submodule churn unless necessary. The intended first step is a
parser-driver or AST-to-HIR shim that preserves integer literal lexemes
when parser-side numeric storage would be lossy"
([verification:20](../verification/integer_model_implementation_inventory.md#L20)).
This means INT-2A acceptance (`x: int = 10 ** 100` reaches HIR without
truncation) cannot be satisfied by a constructed-AST hack alone — both
paths are tested.

### B5. Dict/set hash coherence ownership — Resolved

The hash-coherence contract now has two explicit owners:

- INT-1 scope
  ([issues:103](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L103))
  delivers the runtime primitive: "Implement normalized integer hashing
  helpers for exact and fixed-width dict/set keys."
- INT-4 scope
  ([issues:222](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L222))
  delivers the language-surface integration: "Implement integer dict/set
  key hashing through normalized integer hashing so equal mathematical
  values hash consistently across `int` and fixed-width families where
  equality is allowed", with INT-4 acceptance
  ([issues:233](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L233))
  requiring "`dict[int, V]` lookups using equal fixed-width integer keys
  behave coherently according to the integer equality/hash contract".

This is the right split: the helper lives in the runtime crate so codegen
can route both `SifrInt` and the eight fixed-width Rust primitives through
the same hash, and INT-4 owns the dict/set codegen change that adopts it.
The exact mechanism (an `IntegerKey` newtype adapter vs eager coercion to
`SifrInt` at storage time) is left to implementation, which is appropriate
at this level of phase planning.

### B6. INT-6 contract-lock vs deferred runtime — Resolved

INT-6 is now split cleanly:

- **INT-6A (Dtype Contract Lock,
  [issues:276-300](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L276))**
  must close in this phase. Acceptance specifically requires that "a future
  PR implementing `array[int32] + array[int32] -> array[int32]` without a
  fallible or explicit overflow policy fails an existing contract test or
  pending-fixture gate"
  ([issues:292](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L292)).
  The contract lives under `verification/validation_contracts/` (or
  equivalent test-owned location).
- **INT-6B (Deferred Dtype Runtime Integration,
  [issues:302-323](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L302))**
  explicitly defers the kernel implementation to the owning data-science
  phase but binds it to the INT-6A contract.

This converts the original "blocked behind a tracked runtime issue" hedge
into a concrete commitment that future runtime work cannot bypass. The
design doc ([integer_model.md:263](../internal_docs/integer_model.md#L263))
is also explicit that `array` is "a future dtype-bearing surface in this
design context", removing the implication that array runtimes exist today.

---

## Pass-1 significant gaps — status

### G1. `bytearray` element rules — Resolved

[integer_model.md:261](../internal_docs/integer_model.md#L261) now mirrors
the `bytes` rule for the read path and explicitly covers the write path:
"`bytearray` follows the same element type rule on reads and iteration:
elements are `uint8`. Writes require a fitting literal or a `uint8` value.
Assigning an arbitrary `int` to a bytearray element requires explicit
fallible narrowing through `uint8(value)` so mutation cannot silently
truncate." INT-4 scope and acceptance both encode this:
[issues:221](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L221),
[issues:232](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L232).
Diagnostic family `SIFR-INT-010` covers the boundary.

### G2. Compile-time evaluator scope and budget — Resolved

[integer_model.md:99](../internal_docs/integer_model.md#L99) commits the
specific limits and the cross-module rule: "The first compile-time
evaluator budget is 4096 decimal digits for any evaluated integer result,
plus an implementation-defined operation-count guard… Imported immutable
module constants may carry const-evaluable status across module boundaries
only when the frontend query layer can prove the imported initializer and
its dependency graph are acyclic and within budget." INT-2B scope and
acceptance pick this up
([issues:155-166](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L155))
with `SIFR-INT-004` as the over-budget diagnostic, and the validation
section requires negative tests for budget exhaustion plus cross-module
const fitting tests
([issues:172-174](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L172)).

### G3. `int128` / `uint128` reservation diagnostic — Resolved

`SIFR-INT-003` is reserved for this case
([integer_model.md:65](../internal_docs/integer_model.md#L65),
[integer_model.md:450](../internal_docs/integer_model.md#L450)) and INT-2A
owns emitting it
([issues:132](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L132),
[issues:138](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L138)).

### G4. Generic numeric bounds vs `Addable` — Resolved

[integer_model.md:204-206](../internal_docs/integer_model.md#L204) commits
to refining `Addable`: "The existing `Addable` protocol must be refined to
carry an associated output type or be limited to `Self`-preserving
addition. A generic function that wants mathematical integer addition
across `int` and fixed-width families should use a future integer protocol
with an explicit accumulator/output type, not assume Rust's `Add<Output =
Self>` shape. This refinement belongs to the scalar arithmetic milestone
because it changes operator typing and generic monomorphization."

The cross-cutting protocol contract is updated in
[architecture.md:788](../internal_docs/architecture.md#L788): "Under the
integer-model amendment, `Addable` must model the operator output type;
fixed-width scalar `+` returns exact `int`, so fixed-width types do not
satisfy a generic `T + T -> T` contract through ordinary arithmetic."

INT-3 scope owns the refinement
([issues:191](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L191))
and the validation section adds protocol/generic tests
([issues:208](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L208)).

### G5. `int` × `float` equality and ordering — Resolved

[integer_model.md:193](../internal_docs/integer_model.md#L193) is now
explicit: "Integer and float comparisons are exact rather than cast-based.
`int(2 ** 53 + 1) == float(2 ** 53 + 1)` compares the exact integer to the
exact rational value represented by the float and returns `False`; it must
not cast the integer to `float` first. Ordering follows the same rule by
comparing the integer against the exact decomposed float
mantissa/exponent. NaN remains unordered according to the float comparison
contract."

INT-3 acceptance encodes the test
([issues:199](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L199):
"`int(2 ** 53 + 1) == float(2 ** 53 + 1)` is evaluated exactly and does
not pass through a lossy integer-to-float cast"), and the Validation Matrix
adds "Integer/float comparisons" as a row
([integer_model.md:519](../internal_docs/integer_model.md#L519)).

### G6. Review history gate — Acceptable as-is

INT-0 acceptance now reads "Review history names the most recent agent
review artifact and a human/codex acknowledgement that blocking findings
were addressed"
([issues:73](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L73)),
which is stronger than the previous bare checkbox. The Review History
section
([issues:379-387](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L379))
lists every prior pass and the pending pass-2 entry. This is sufficient.

### G7. Performance acceptance — Mostly resolved, soft threshold remains

INT-8 now names a concrete benchmark fixture
("`verification/perf/sifr_int_loop.sifr` or the phase-35 equivalent
benchmark fixture for a small-int accumulation loop",
[issues:357](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L357))
and ties acceptance to the phase-35 performance tooling
([issues:365-366](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L365)).
The first criterion is measurable: "Common small-`int` loops do not
allocate in the loop body for proven-small values, as measured by the
repository's phase-35 performance tooling or an explicitly documented
allocator probe."

The throughput criterion is still soft: "Small-int accumulation throughput
is within an implementation-approved threshold recorded in the benchmark
artifact." That defers the actual ratio (e.g., "within 2x of `i64`-only
Rust") to whoever closes INT-8. It is acceptable because the fail mode is
now visible — there is a benchmark artifact to argue against — but tighter
acceptance ("ratio recorded and ratified by phase-35 budget tooling, with
regression > X% failing the milestone") would close the loophole. Marked
as polish, not blocking.

### G8. Diagnostic code allocation — Mostly resolved, per-milestone attribution partial

The `SIFR-INT-001..010` family is allocated and table-mapped at
[integer_model.md:444-457](../internal_docs/integer_model.md#L444). INT-2A
and INT-2B explicitly own `SIFR-INT-003` and `SIFR-INT-004`. INT-7
formally reserves the family
([issues:332](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L332)).

What is still missing: per-milestone code attribution for INT-3, INT-4,
INT-5. The pre-allocation makes intent recoverable from the table (INT-3
should emit `SIFR-INT-005`, `SIFR-INT-006`, `SIFR-INT-007`; INT-4 should
emit `SIFR-INT-010`; INT-5 should emit `SIFR-INT-009`), but the milestone
text does not name those codes the way INT-2A/2B do. This is polish, not a
blocker — INT-7 acts as the catch-all closure for "stable diagnostic
codes" — but pinning each milestone to its code(s) would prevent silent
slippage.

---

## Smaller pass-1 findings — status

| Pass-1 item | Status |
| --- | --- |
| `SifrInt` ABI commitment | Captured implicitly via `crates/sifr_runtime` placement; the `Small(i64)` / `Big(Box<num_bigint::BigInt>)` shape is recorded in [integer_model.md:36-41](../internal_docs/integer_model.md#L36) and INT-1 scope authorizes equivalents. ABI-stability note (`#[doc(hidden)]` or generated-code ABI stamp) is not yet written down — optional. |
| `num-bigint` vs alternatives | Not addressed; remains optional, suitable to revisit at INT-8. |
| `bytes` vs `array[uint8]` | Resolved at [integer_model.md:259-263](../internal_docs/integer_model.md#L259) including the explicit "future dtype-bearing surface" caveat. |
| `isize`/`usize` low-level module carve-out | Partially resolved at [integer_model.md:247](../internal_docs/integer_model.md#L247) ("a future package/module-level low-level interop opt-in"). The exact mechanism (module attribute, package opt-in, directory convention) is deferred. Acceptable for this phase; future low-level-interop phase owns the lock. |
| Pickle / serde derive default policy | Resolved at [integer_model.md:352](../internal_docs/integer_model.md#L352): "Generated `serde::Serialize` or `serde::Deserialize` support for Sifr structs/classes must use an explicit integer profile rather than Rust's default primitive serialization for `SifrInt`. Framework-level default derives use `json.web`…" INT-5 acceptance enforces it ([issues:265](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L265)). |
| REPL/debug formatting | Not addressed; deferred. Acceptable. |
| HIR maintainability guardrails update | Captured in INT-2B scope ([issues:158](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L158)) and validated via `python3 scripts/check_hir_maintainability_guardrails.py` ([issues:176](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L176)). |
| Migration scope of existing demos/fixtures | Partially addressed via [verification/integer_model_implementation_inventory.md:83-94](../verification/integer_model_implementation_inventory.md#L83) which enumerates known legacy references to retire/quarantine. INT-1 itself does not include the explicit "expect bulk fixture/snapshot churn; reset baselines via `cargo insta review`" note suggested in pass 1, but the inventory effectively makes the blast radius visible. Optional polish. |

---

## Remaining polish items (non-blocking)

These are worth addressing before or during the relevant milestone but
should not delay INT-0 closure or INT-1 start.

1. **Per-milestone diagnostic code attribution.** Pin `SIFR-INT-005..007` to
   INT-3 scope, `SIFR-INT-010` to INT-4 scope, `SIFR-INT-009` to INT-5
   scope, and `SIFR-INT-008` to INT-6A scope so each milestone has a
   concrete code to test against rather than relying on INT-7 to retrofit.
   Pass-1 G8 remediation only landed for INT-2A/2B.
2. **INT-8 throughput threshold.** Replace "implementation-approved
   threshold" with either a hard ratio against an `i64`-only Rust
   reference, or an explicit binding to phase-35 budget tooling output
   ("regressions caught by phase-35 budget gates fail this milestone"). The
   benchmark fixture is named; the gate is not.
3. **`SifrInt` representation ABI note.** A short paragraph in
   [integer_model.md:461](../internal_docs/integer_model.md#L461) noting
   that the `Small(i64)` / `Big(...)` variant tags are part of the
   generated-code surface (or are deliberately `#[doc(hidden)]` /
   compile-time-internal) would prevent later "swap to `Small(i128)`" PRs
   from accidentally breaking generated projects. INT-1 is the natural
   place to add it.
4. **Frontend query crate ownership for cross-module const propagation.**
   INT-2B mentions "the frontend query layer" but does not name the crate.
   `sifr_frontend` is the canonical frontend API
   ([architecture.md:230](../internal_docs/architecture.md#L230),
   [architecture.md:754](../internal_docs/architecture.md#L754)) — pinning
   it explicitly in INT-2B prevents the const-fitting cross-module path
   from accidentally landing in `sifr_hir` or `sifr_type_system`.
5. **Fixture/snapshot churn warning in INT-1.** Add a one-line note in
   INT-1 scope stating that integer-bearing pass fixtures and codegen
   snapshots will see bulk churn during this milestone and that baselines
   should be reset via `cargo insta review`. The inventory already
   enumerates the affected directories, but a reader of INT-1 alone may be
   surprised.
6. **`bigint` transition diagnostic shape.** INT-2B says "`bigint` is gone
   from public docs/tests or emits intentional transition diagnostics
   only" ([issues:167](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md#L167))
   but does not specify the diagnostic code or message family. If a
   transition alias survives implementation staging, allocate (e.g.,
   `SIFR-INT-002` already covers narrowing, but `bigint`-specific
   deprecation may warrant a dedicated note family). Optional; can be
   resolved when the actual transition decision is made.

---

## PR-slicing readiness

The eleven-milestone breakdown supports clean PR slicing:

- INT-0 → 1 PR (architecture amendment + design doc + inventory; already
  effectively prepared in this branch).
- INT-1 → 1 PR for the new `crates/sifr_runtime` crate plus a follow-up
  PR for codegen Cargo manifest emission. Reasonable to land as 1–2 PRs.
- INT-2A → 1 PR (parser shim + `SIFR-INT-003`).
- INT-2B → 1 PR for HIR/type-system literal representation + 1 PR for
  const-fitting evaluator. Reasonable to land as 1–2 PRs.
- INT-3 → 1 PR per arithmetic family (scalar promotion, division/modulo,
  exponentiation/shift budgets, `Addable` refinement, int×float
  comparison) — naturally 3–5 PRs.
- INT-4 → 1 PR per surface (indexes/`len`/`range`, bytes/bytearray,
  dict/set hashing integration, `sum`/`min`/`max`/`abs`, pattern
  matching) — naturally 3–5 PRs.
- INT-5 → 1 PR per profile (`json.exact`, `json.web`, `json.string_ints`,
  serde derive integration, OpenAPI/TS mapping, SQL guardrails) —
  naturally 3–5 PRs.
- INT-6A → 1 PR (contract artifact + pending fixtures).
- INT-6B → owned by data-science phase; not in this phase's PR count.
- INT-7 → 1 PR for diagnostic code finalization + 1 PR for documentation
  cleanup + 1 PR for legacy fixture retirement.
- INT-8 → 1 PR for benchmark fixture + 1 PR for fuzz/property tests + 1
  PR for closure review.

Each PR has clear validation (`scripts/run_all_tests.sh --profile quick`
plus the per-milestone validation listed in the issue), making review
overhead tractable.

---

## Summary

Pass-1 blockers B1–B6: **all resolved.**
Pass-1 significant gaps G1–G8: **all resolved or with explicit deferrals.**
Smaller findings: **resolved, deferred, or marked optional.**

INT-0 may close on this review. INT-1 is ready to start. The polish items
above can be addressed in their owning milestones without rework.
