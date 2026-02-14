# Sifr Architecture Review: Missing Language Semantics

Date: 2026-02-14
Scope reviewed: `.cursor/plans/sifr_compiler_architecture_fa3c10ee.plan.md` and current compiler implementation in `crates/`.

## Executive Verdict

Most recommendations are directionally correct, but several should be reframed to preserve Sifr's explicit ownership model and avoid hidden runtime behavior.

- Adopt now: 6, 7, 12 (partially), 13
- Adopt with redesign: 8, 9, 10
- Already partially covered (not missing entirely): 11

## Item-by-Item Review

### 6) Slices and Views

Verdict: **Makes sense, high priority**

Why:
- The architecture plan lists slicing syntax, but does not define copy-vs-view semantics.
- Without an explicit contract, behavior and performance expectations are ambiguous.

Recommendation:
- For MVP, `list[a:b]` should produce a new list (copy semantics).
- Add an explicit view API later (for example `items.view(a, b)` or dedicated view type) if needed for performance-critical paths.

---

### 7) String Semantics (UTF-8)

Verdict: **Makes sense, needs a stronger contract**

Why:
- `str` maps to Rust `String`, but indexing/length semantics are not fully specified in the plan.
- Current codegen lowers generic indexing to `x[i as usize]`, which is not valid for Rust `String` and does not encode safe UTF-8 behavior.

Recommendation:
- Define `s[i]` as returning a character string (or `Option[str]`, preferred for safety consistency).
- Define `len()` as code point count (Python-like) and explicitly document complexity (`O(n)` for Unicode).
- Document out-of-bounds/invalid access behavior consistently with the Option/Result model.

---

### 8) Interior Mutability Pattern

Verdict: **Concept valid, current recommendation too aggressive**

Why:
- Shared mutable graphs/callback systems eventually need an interior mutability story.
- Automatic insertion of `RefCell`/`Mutex` everywhere would hide borrow rules and introduce runtime borrow failures unexpectedly.

Recommendation:
- Do not auto-wrap all shared mutable state.
- Introduce explicit opt-in shared mutable abstractions later (language-level wrapper that can map to `RefCell`/`Arc<Mutex<_>>`).
- Keep default ownership/borrow behavior explicit and predictable.

---

### 9) Send/Sync and Thread Safety (M11)

Verdict: **Needed, but recommendation mostly not**

Why:
- The plan already enforces `Send + 'static` for async captures crossing `.await`, but lacks a broader concurrency contract.
- "All types Send + Sync by default" is too strong and may force unnecessary synchronization overhead.
- Auto-upgrading `Rc` to `Arc` across boundaries is surprising and can mask cost/behavior changes.

Recommendation:
- Add a dedicated Concurrency Safety contract section.
- Keep boundary checks explicit (spawn/thread/task boundaries require sendable types).
- Require explicit shared-state primitives for cross-task mutable sharing; avoid silent upgrades.

---

### 10) Drop / Destructor Semantics

Verdict: **Partially makes sense, needs careful design**

Why:
- Cleanup behavior is not clearly defined in the plan, but deterministic destruction is relevant in a Rust-backed language.
- Directly exposing Python-style `__del__` mapped 1:1 to `Drop` can cause subtle semantics and ordering expectations.

Recommendation:
- First define destruction guarantees (scope end, move semantics, panic behavior).
- Only then decide whether to expose user-defined destructors, and with what restrictions.
- Prefer deterministic cleanup primitives and explicit resource wrappers for MVP.

---

### 11) Enum Variants with Data (ADTs)

Verdict: **Not missing entirely (already partially covered)**

Why:
- M5 already includes class unions with tag-based discriminated narrowing, which provides ADT-like modeling.
- The plan already compiles unions to Rust enums.

Recommendation:
- Treat explicit enum syntax as an ergonomic enhancement, not a conceptual gap.
- Keep M5 class unions as the primary initial ADT path; evaluate adding `enum` syntax after M5 stabilization.

---

### 12) Derive / Auto-implemented Traits

Verdict: **Makes sense, with scope constraints**

Why:
- Current codegen for union enums already derives `Debug, Clone`.
- Plan mentions `@dataclass` and derive-like behavior later (M14), but debug/equality usability is needed earlier.

Recommendation:
- Document baseline auto-derives earlier as a language contract for generated types.
- Safe baseline: `Debug`, `Clone`, `PartialEq` where valid.
- Do not promise universal `Hash`; make hashability conditional on field/member hashability and language-level hashability rules.

---

### 13) Newtype Pattern for Validation

Verdict: **Makes sense, medium priority**

Why:
- Domain invariants (port numbers, IDs, bounded values) benefit from type-level wrappers.
- This aligns naturally with Rust newtype representation and improves API correctness.

Recommendation:
- Add after M5 foundations (classes/protocols) as either:
  - explicit newtype syntax, or
  - constrained single-field wrapper class contract.
- Ensure zero-cost runtime representation where possible.

## Priority Actions to Add to the Architecture Plan

1. **M2 semantic contracts**
   - Slice copy semantics now; view semantics deferred.
   - UTF-8 string indexing and length semantics, including complexity notes.

2. **Cross-cutting contract additions**
   - Concurrency safety beyond async closure capture (`Send`/sharing rules at boundaries).
   - Destruction/cleanup semantics (determinism and guarantees).

3. **M5+ type modeling**
   - Clarify that class unions already serve ADT needs; explicit enums are optional ergonomic sugar.
   - Add a newtype/invariant pattern roadmap item.

4. **Trait derivation contract (early)**
   - Promote derive expectations from "implementation detail" to documented language behavior.
   - Define conditional hashability rules.

## Final Assessment

The recommendation list is strong overall. The main corrections are:
- avoid hidden auto-magic (`RefCell`/`Mutex`, `Rc -> Arc` upgrades),
- specify string/slice semantics now,
- and frame ADTs as mostly present already via class unions, with explicit enums as syntax-level ergonomics later.
