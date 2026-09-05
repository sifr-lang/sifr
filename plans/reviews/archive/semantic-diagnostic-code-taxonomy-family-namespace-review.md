# Review: Per-Family Namespace Amendment to Diagnostic Code Taxonomy

Reviewer: agent
Date: 2026-04-29
Source: `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`
Repo state at review: branch `main`, commit `c891177b` (file is uncommitted-modified — diff reviewed)
Prior reviews: `reviews/semantic-diagnostic-code-taxonomy-proposal-review.md`, `-2.md`, `-3.md`
Lens: principal-engineer / compiler architecture

Severity: 🔴 blocker, 🟠 must-fix, 🟡 should-fix, 🟢 polish.

---

## Verdict: NEARLY READY — one 🟠 contradiction to fix, two 🟡

The per-family namespace design is **elegant, implementable, and correctly scoped**. The amendment is internally clean for the substantive sections (Proposed Diagnostic Families, numbering convention, ownership rules, Decimal Code Migration table, Initial Code Examples). However, the amendment did **not propagate** to two locations that still encode old global-range numeric assumptions, and one new bullet introduces a small example-vs-policy mismatch.

| # | Severity | Location | Issue |
|---|---|---|---|
| F1 | 🟠 | `milestone_diag_6` scope, line 578 | Says `SIFR-DECIMAL-25xx`; new codes are `SIFR-DECIMAL-0001..0008` |
| F2 | 🟡 | Internal code allocation, line 824 | `SIFR-INTERNAL-9001` is a leftover global-range number, inconsistent with per-family "first active is usually `0001`" |
| F3 | 🟡 | Sub-range example, line 140 vs line 148 | Example `SIFR-STDLIB-0100..0199` is 100 codes; policy says "preferably 50 codes at a time" |
| F4 | 🟢 | `milestone_diag_2a` scope/DoD, lines 478, 496 | Word "ranges" is now ambiguous (per-family local span vs. stdlib sub-range vs. global allocation) |
| F5 | 🟢 | Pre-existing typo, line 373 | Stray `>` in `impl From<TypeError> for SifrDiagnostic>` (not introduced by this amendment, but easy to fix while editing) |

No 🔴 findings. The design is sound; only stale numeric references and one minor example need cleanup.

---

## 1. Design assessment: per-family namespace

The shift from one shared `0000..9999` global space (with sub-allocations like `SIFR-NAME-1000..1499`) to a per-family local `0000..9999` space is the right call.

### Why it is elegant

- **Identity is the full string.** `SIFR-NAME-0001` and `SIFR-CALL-0001` are unambiguously different codes because the family prefix is part of the namespace. The doc states this explicitly (line 131).
- **Family addition is unbounded.** New semantic domains create a new prefix; there is no "find unused space" puzzle, no fragmentation pressure, no eventual exhaustion (the previous design squeezed 17 families into 10,000 numeric slots, leaving a few hundred per family and no real headroom for future families like `SIFR-CONST-*`, `SIFR-MACRO-*`, `SIFR-LIFETIME-*`, etc.).
- **Human readability stays intact.** Local numbers can be small and meaningful (`SIFR-NAME-0001` = first name-resolution diagnostic). Reading `SIFR-NAME-1001` always required mentally subtracting the base (`1000`) to recover ordinal position; that mental tax is now gone.
- **Existing workspace codes survive.** `SIFR-WORKSPACE-0101` no longer needs a forced renumber to `SIFR-WORKSPACE-6101`; the prior round-3 review's Y16 "Exact workspace renumbering" table is correctly deleted, eliminating churn that had no semantic justification.

### Why it is implementable

- **Typed model is straightforward.** `DiagnosticCode { family: Family, suffix: u16 }` (or an enum with named variants per code, with `family()` and `suffix()` accessors) maps cleanly. The proposal already requires `DiagnosticCode` to be a typed enum/strict newtype (line 248) — the per-family variant is, if anything, simpler than a single 4-digit space because the variant naturally partitions by family.
- **Registry semantics work.** A registry keyed by `(family, suffix)` (or by full string) and indexed by family is the natural shape. Each family's table can live in its own module under `crates/sifr_diagnostics/src/codes/<family>.rs`, with a top-level `codes.rs` aggregator.
- **Doc URLs derive cleanly.** `https://sifr.sh/docs/errors/<CODE>` (line 172) remains globally unique because the full code is globally unique. The URL policy needs no changes.
- **Sort/display order is deterministic.** Lexicographic sort on the full code string yields stable diagnostic ordering across renderers.
- **Fixture grammar (`SIFR-<FAMILY>-dddd`, line 561) is well-defined.** Because the FAMILY token is unbounded by design, the e2e harness must validate against the registry, not against a hard-coded family allowlist. Lines 695 ("every emitted code must exist in the registry") and 815 ("do not allow an `expect-error` fixture annotation to use a code absent from the registry") already require this — registry-driven validation is the correct enforcement mechanism, no schema change needed.

### What does *not* need changing

The bulk of the document — `Design Principle`, `Diagnostic Identity Policy`, `Non-Goals`, `Family ownership rules for overlaps`, `Generic examples`, `Documentation URL Policy`, `Target Architecture`, `Existing Surface Inventory`, `Dependency Ownership`, `Type System Integration`, `Diagnostic Builder API`, `Span Policy`, `Source Mapping Architecture`, `Stability Policy`, `Hard Rules`, `Non-Error Diagnostics`, `Phase Definition of Done`, `Risk Register` — is unaffected by the namespace switch and remains correctly aligned. The `Initial Code Examples` table (lines 736–758) is fully renumbered to local `000x` values and is internally consistent with the `Decimal Code Migration` table.

---

## 2. Material findings

### F1. 🟠 `milestone_diag_6` scope still references `SIFR-DECIMAL-25xx`

**Location:** line 578, `milestone_diag_6` Scope.

> Convert existing decimal pseudo-codes to real top-level `SIFR-DECIMAL-25xx` codes.

This contradicts the `Decimal Code Migration` table at lines 184–190, which maps the new codes to `SIFR-DECIMAL-0001` through `SIFR-DECIMAL-0008`. The `25xx` form is a direct holdover from the old global-range allocation where DECIMAL was `2500..2599` — under the per-family scheme, the DECIMAL family starts at `0001` like every other family, and `25xx` no longer corresponds to anything.

This is the only place a stale numeric range survived. It is a small typo with material impact: an implementer reading milestone_diag_6 in isolation could legitimately assign codes in the `2500..2599` range and produce a registry that disagrees with the migration table.

**Fix:** replace with one of:
- `Convert existing decimal pseudo-codes to real top-level SIFR-DECIMAL-* codes (see the Decimal Code Migration table).`
- `Convert existing decimal pseudo-codes to real top-level SIFR-DECIMAL-000x codes per the Decimal Code Migration table.`

The neighboring "No decimal diagnostic message embeds `[E25xx]`" (line 586) is **correct as-is** — `[E25xx]` there refers to the *retired message-embedded pseudo-codes*, not to new code numbering, and the amendment correctly preserves that wording.

### F2. 🟡 `SIFR-INTERNAL-9001` is inconsistent with the per-family numbering convention

**Location:** line 824.

> `SIFR-INTERNAL-9001` is the stable catch-all for unclassified compiler panics after a panic boundary.

Under the previous global allocation, `SIFR-INTERNAL-*` was reserved `9000..9999`, so `9001` = base + 1, i.e. the first active code in the family. Under the new per-family scheme, the INTERNAL family is local `0000..9999` like every other family, and the numbering convention (line 138) says:

> The first active code in a family is usually `0001`, for example `SIFR-NAME-0001`.

`SIFR-INTERNAL-9001` is now a numerically arbitrary choice — there is no functional reason it sits at `9001` rather than `0001`. The "usually" hedge in the convention does formally permit this, but readers will reasonably expect the catch-all to be `SIFR-INTERNAL-0001` and may treat `9001` as either a typo or evidence that there is some hidden allocation scheme they are missing.

**Fix options (one of):**
1. Change to `SIFR-INTERNAL-0001` for consistency with the per-family convention. Cleanest.
2. Keep `9001` and add a sentence: "Internal-failure codes are intentionally allocated in the upper part of the local range (`9000+`) to leave `0001..0999` available for narrowly-scoped recurring internal categories." This makes the convention conscious rather than incidental.

Option 1 is recommended unless there is a compatibility reason — and there is not, since the proposal is explicitly pre-1.0 and forbids compatibility aliases.

### F3. 🟡 Stdlib sub-range example contradicts sub-range size policy

**Location:** lines 140 and 148.

Line 140 (newly added by this amendment):
> A family can reserve semantic sub-ranges locally, for example `SIFR-STDLIB-0100..0199` for one stdlib module. These local sub-ranges have no meaning outside that family.

Line 148 (existing policy, unchanged):
> Each stdlib module should receive a reserved contiguous local sub-range, preferably 50 codes at a time, tracked in the diagnostic registry.

The example reserves 100 codes (`0100..0199`); the policy prefers 50 codes. The example was added in this amendment, so it is a new inconsistency introduced alongside the namespace change.

**Fix:** make the example match the policy:
- `for example SIFR-STDLIB-0100..0149 for one stdlib module.`

Or, if the intent is to allow 100-code blocks for larger stdlib modules, relax the policy on line 148 to `preferably 50 codes per module (100 for larger modules)`. Either is fine; pick one.

### F4. 🟢 "Ranges" wording in milestone_diag_2a is now ambiguous

**Location:** lines 478 and 496.

- Line 478 (Scope): `Define code family ranges and initial reserved codes.`
- Line 496 (DoD): `The registry skeleton exists with families, ranges, state machine, and reserved base codes.`

Pre-amendment, "ranges" unambiguously meant the global-allocation slices like `SIFR-NAME-1000..1499`. Post-amendment, the only "ranges" left are (a) the uniform local `0000..9999` per family — which is the same for every family and barely worth calling a "range" — and (b) the optional stdlib sub-ranges (line 140/148).

This is not a contradiction; it is just stale terminology that will read oddly. Suggested rewordings:

- Line 478: `Define code family namespaces, the per-family local 0000..9999 convention, and initial reserved codes (including each family's reserved 0000 base).`
- Line 496: `The registry skeleton exists with families, the per-family numbering convention, state machine, and reserved family bases (0000 per family).`

Polish only — not a blocker.

### F5. 🟢 Pre-existing typo: stray `>` in type-system integration

**Location:** line 373.

> Do not add `impl From<TypeError> for SifrDiagnostic>` as the long-term design.

The trailing `>` is unbalanced. Should read `impl From<TypeError> for SifrDiagnostic`. This typo predates this amendment (visible in the same shape across earlier rounds), but flagging it since it is one character and it is in an editable section of the same document.

---

## 3. Things that look fine — explicit no-action

For traceability, here is what I checked and found correctly aligned with the per-family namespace model. No action needed.

- **Decimal Code Migration table** (lines 184–190): all `SIFR-DECIMAL-0001..0008` — consistent.
- **Initial Code Examples table** (lines 736–758): all renumbered to local `000x` codes — consistent.
- **Existing code renumbering table** (lines 158–165): correctly retires `SIFR-TYPE-0001` "and never reused", clarifies workspace codes "may remain", and changes `SIFR-CODEGEN-7xxx` / `SIFR-BUILD-8xxx` mentions to family-relative `xxxx` — all consistent.
- **Numbering convention** (lines 135–140): correctly states base-`0000` reserved per family, first active usually `0001`, retired codes remain as registry gaps.
- **Family ownership rules** (lines 142–148): unchanged from prior round, still semantically correct under per-family namespacing.
- **Hard Rules** (lines 802–820): no rule references global numeric ranges; "do not preserve `SIFR-TYPE-0001` compatibility" remains correct as the retired catch-all.
- **Documentation URL Policy** (line 172): URL form `https://sifr.sh/docs/errors/<CODE>` works because the full code is globally unique.
- **Workspace renumbering table:** correctly **removed** entirely. The prior round-3 Y16 finding required exact mappings to `6001..6004` / `6101..6103`; under per-family namespacing those mappings have no purpose, and the removal is the right call.
- **Mermaid sequencing diagram** (lines 715–728): unaffected by the namespace switch.
- **JSON envelope, schema, source-span, accumulator model, severity enum, ChildSeverity, message_template, args:** all unaffected by the namespace switch and remain correctly specified per round-3 verification.
- **`SIFR-PARSE-0001` retention** (line 160): prior round-3 noted this required PARSE family base reserved with `0001` as base+1; under the new uniform per-family `0000` base, this still works — `SIFR-PARSE-0001` is the first active code in the PARSE family, fully consistent with line 138.
- **Fixture grammar** `SIFR-<FAMILY>-dddd` (line 561): correctly registry-driven via lines 695 and 815.

---

## 4. Implementation notes (informational, not findings)

A few observations for whoever picks up `milestone_diag_1` / `diag_2a`:

- **Code identity in Rust.** Recommend a typed enum with one variant per active code, and a derived `(family, suffix)` view, rather than a flat `(Family, u16)` newtype. The enum form gives exhaustive matching at every emission site (which is what makes "no `format!`-constructed codes" — Hard Rule, line 816 — actually enforceable at the type level rather than by lint). The `(family, suffix)` accessors are then used only at the registry/serialization boundary.
- **Registry layout.** Putting each family in its own file (`crates/sifr_diagnostics/src/codes/name.rs`, `codes/call.rs`, …) keeps file size bounded as families grow toward `9999`, matches the workspace convention against monolithic files (per `AGENTS.md`), and makes per-family ownership reviews (e.g. "all SIFR-OWN-* codes assigned by ownership-system owner") straightforward.
- **Reserved family bases.** Seventeen families × one reserved `0000` base each = 17 reserved bases. The registry should record these as `state: Reserved` rows from day one (in `milestone_diag_2a`'s skeleton), so the convention is enforced by the registry rather than only by documentation.
- **Stdlib sub-ranges as registry metadata.** Sub-ranges like `SIFR-STDLIB-0100..0149` should appear in the registry as `module: "math"` annotations on each contained code, plus a top-level `stdlib_subranges` table mapping module → range. The docs generator can then render a per-module index page without hand-writing it.

---

## 5. Recommendation

Apply F1 (🟠) and F3 (🟡) before starting `milestone_diag_1`; both are one-line edits that close the only places where the amendment did not propagate. F2 (🟡) is also a one-line edit and is recommended in the same pass. F4 and F5 are optional polish and can ride along with whatever PR closes F1.

Once those edits land, the per-family namespace amendment is fully internally consistent and the proposal is implementable as written. The design itself is the right call and represents a strict improvement over the global-range allocation in every dimension I evaluated.
