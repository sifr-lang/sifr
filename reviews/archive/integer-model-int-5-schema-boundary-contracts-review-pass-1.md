

Based on my thorough review of the three working-tree artifacts, here is my assessment:

---

## INT-5 Schema Boundary Contract Review

**Verdict: No blockers. Reviewer is satisfied for this INT-5 schema boundary contract slice.**

---

### Review Details

**1. OpenAPI/JSON Schema integer mappings — adequate coverage**

The contract correctly maps:
- Fixed-width types (`int8`-`uint32`) → `json.web` with explicit `minimum`/`maximum` bounds
- `int64`/`uint64` → `json.web` decimal string default with `x-sifr-format: integer-decimal-string`
- Exact `int` → `json.web` decimal string default or range-error policy
- `json.string_ints` → decimal string with `x-sifr-format` marker
- `json.exact` → `type: integer` with `x-sifr-integer-profile: exact` and client warning

The fail-closed rule with `SIFR-INT-0009` (line 40) and field-path-plus-suggested-policies payload is appropriately actionable for future schema emitters. One intentional asymmetry exists (`int` defaults to decimal string *or* typed error policy, while `int64`/`uint64` default to decimal string) — this is deliberate given exact `int` may have provably-bounded static ranges, while `int64`/`uint64` are inherently 64-bit. Not a blocker.

**2. TypeScript client mappings — precision loss prevention is explicit**

The most critical clause is on lines 54-57: "*int64, uint64, and exact int response fields under json.web default to decimal strings. A generated TypeScript number for those fields is valid only when the schema also carries a static safe range.*"

This directly addresses the silent precision loss risk. The branded `SifrDecimalIntString` type (lines 50-51) prevents accidental misuse. `bigint` mapping is gated behind *explicit* runtime + JSON parser strategy configuration, not a silent fallback. No blockers.

**3. Generated serde — profile selection bypass prevention is locked**

Lines 60-74 correctly prohibit:
- Direct Rust primitive or `SifrInt` serde behavior that bypasses profile selection
- Internal derives without explicit profile declaration
- Recursive/nested field inheritance without explicit override

The distinction between *public/browser-facing* (default `json.web`) and *internal* (must declare explicitly) is the right split. Serialization failures return `JsonIntegerRangeError` with model path; digit-limit violations return `JsonLimitError`. This is consistent with the runtime profile machinery reviewed in PRs #1890 and #1891.

**4. SQL/storage — explicit representation requirements are unambiguous**

The table on lines 78-87 makes ORM/storage behavior explicit:
- Fixed-width SQL columns require `uint*`/`int*` field or fallible narrowing
- Unsigned dialect columns require explicit `uint*`
- `NUMERIC`/`DECIMAL` with integer scale: requires checked exact `int` mapping
- Text/binary formats require explicit policy

The critical anti-inference rule ("ORM and storage layers must not infer `int64` or `BIGINT` from a plain Sifr `int` annotation") prevents the exact silent-widening failure this contract is designed to stop.

**5. SIFR-INT-0009 diagnostic contract — runtime vs. compile-time distinction is clear**

The contract cleanly separates:
- **Compile-time/generation-time**: `SIFR-INT-0009` is emitted when schema or generation steps would create unsafe/ambiguous integer boundaries (lines 94-96)
- **Runtime**: `JsonIntegerRangeError` for selected-profile violations (lines 97-99)
- **Decoder layer**: `JsonLimitError` for untrusted integer token budget violations

Diagnostic payload requirements (lines 101-107) cover boundary kind, field/path, selected/missing profile, static range, and suggested alternatives — sufficient for future compiler/schema tooling to produce actionable diagnostics.

**6. Blockers before opening contract-lock PR**

None identified.

---

### Minor Observations (non-blocking)

1. **Exact-client "exact-client support" terminology** (line 52): The contract says `bigint` is valid "only when the target runtime and JSON parser strategy are explicitly configured." Future TypeScript/ORM phases will need to define *how* "explicitly configured" is declared in schema metadata. This is appropriately deferred to the owning phases — the contract correctly notes it must be explicit.

2. **`int` vs. `int64` asymmetry** (lines 34-35): `int64`/`uint64` map to decimal string by default; plain `int` maps to decimal string *or* typed error policy. This asymmetry is intentional (exact `int` may have provably-bounded ranges; `int64` is inherently 64-bit), but implementors should document why the asymmetry exists when they ship the owning phases.

3. **"fallible narrowing" in SQL table** (line 83): The contract says "fallible narrowing from `int` with range checks." Future ORM phases will need to define the fallible API shape. This is appropriately deferred.

---

### Contract Quality Assessment

The artifact is well-structured as an *intent and constraint document* rather than an implementation spec — it correctly establishes hard rules that future owning phases (web, ORM, schema generators, TypeScript clients) cannot violate while appropriately deferring implementation mechanics (safe-range constraint syntax, exact-client configuration declaration, ORM narrowing API shape) to those owning phases.

The cross-references from `internal_docs/integer_model.md` (lines 344-346) and `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` (line 565) correctly tie the contract into the canonical design and phase tracker.

**The reviewer is satisfied. This INT-5 schema boundary contract slice is ready to open as a contract-lock PR.**
