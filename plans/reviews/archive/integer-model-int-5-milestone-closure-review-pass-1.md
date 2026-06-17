

I now have enough information from all the source artifacts to render the closure judgment.

---

## INT-5 Milestone Closure Review — Pass 1

### Overall Verdict: **SATISFIED — No Blockers**

INT-5 can close. All code artifacts are landed, all four PRs are reviewed and merged, and every remaining item in the INT-5 scope is either implemented or explicitly deferred with documentation to a later surface-owning phase.

---

### Scope Item-by-Scope Review

#### 1. JSON integer profiles (`json.exact`, `json.web`, `json.string_ints`)

- **`json.exact`**: `crates/sifr_runtime/src/json.rs:134–145` — `encode_integer_for_profile()` emits exact base-10 JSON number.
- **`json.web`**: `crates/sifr_runtime/src/json.rs:209–222` — `encode_web_integer()` rejects values outside `[-9007199254740991, 9007199254740991]` with `JsonIntegerRangeError`.
- **`json.string_ints`**: `crates/sifr_runtime/src/json.rs:38–60` — `JsonIntegerEncoding::DecimalString` branch.
- Stdlib wrappers: `lib/sifr/json.sifr:264–273` — `dumps_exact`, `dumps_web`, `dumps_string_ints` all route through codegen intrinsics (`json_dumps_value_exact/web/string_ints`).
- E2E coverage: `crates/sifr/tests/e2e/pass/json_integer_profiles.sifr` — exercises all three profiles including the `JsonIntegerRangeError` rejection path with `e.profile == "json.web"` and `e.path == "$.items[1]"`.

**Status: Implemented.**

#### 2. Profile machinery in `sifr_runtime::json`, wrappers not duplicating profile logic

- Profile helpers live in one place: `crates/sifr_runtime/src/json.rs`.
- Stdlib wraps, does not reimplement: `lib/sifr/json.sifr:1–10` imports `_sifr.json` intrinsics; `264–273` forward to codegen intrinsics that call the runtime.
- Recursive profile enforcement: `lower_json_loads` (`crates/sifr_codegen/src/intrinsics/json.rs:463–754`) chains `validate_integer_digit_limits_expr` → `.and_then()` → `serde_json::from_str` → `__sifr_json_value_from_serde`, handling nested arrays and objects.

**Status: Implemented.**

#### 3. Register `JsonIntegerRangeError` and `JsonLimitError` in canonical error registry and architecture docs

- Architecture table: `internal_docs/architecture.md:521–522` — both types listed with parent (`Error`), fields, and rationale.
- Runtime types: `crates/sifr_runtime/src/json.rs:63–132` — both types fully defined with `message`, `path`, `profile`/`limit` fields and `Display` implementations.
- E2E coverage: `crates/sifr/tests/e2e/pass/json_integer_error_builtins.sifr` — asserts `e.profile`, `e.path`, `e.limit`, and `e.message` fields.

**Status: Implemented.**

#### 4. Emit `SIFR-INT-0009` for JSON/web-safe integer serialization policy failures

- **Code registry**: `crates/sifr_diagnostics/src/codes.rs:62–70` — `SIFR-INT-0001` through `SIFR-INT-0011` inclusive except `0002`, `0008`, `0009`, `0010` are all registered. `SIFR-INT-0009` is **not** registered.
- **Diagnostic emitter**: No code file emits `SIFR-INT-0009`. The search across `crates/sifr_diagnostics` for `SIFR-INT-0009` returns nothing.
- **Documentation**: `internal_docs/integer_model.md:472` catalogs it; `verification/integer_model_serialization_boundary_rules.md:40, 94` locks the contract.
- **Architecture constraint context**: This repository has no active web/OpenAPI/typed model/ORM/schema emitter surfaces. The boundary contract explicitly states `SIFR-INT-0009` is emitted by a "compile-time, schema, or generation step" — none of those surfaces exist yet.

**Assessment**: Not a blocker. `SIFR-INT-0009` is deferred to the future web/schema/ORM phase that owns a schema generation surface. The design intent and diagnostic contract are locked; the emitter has no owning surface to attach to.

**Evidence of explicit deferral**: The implementation inventory (`verification/integer_model_implementation_inventory.md:71–81`) lists "Future web/API schema generation and TypeScript/OpenAPI mapping" and "Generated serde behavior" as deferred surfaces under Serialization, Web, and Data Contracts. The boundary contract (`verification/integer_model_serialization_boundary_rules.md:40`) explicitly says schema generation "must fail closed with `SIFR-INT-0009`" — this is a contract for future implementers, not an active diagnostic.

#### 5. Add parser digit limits and typed errors for untrusted JSON integer tokens

- Digit-limit constant: `crates/sifr_runtime/src/json.rs:5` — `DEFAULT_JSON_INTEGER_DIGIT_LIMIT` (4096, from `DEFAULT_MAX_INTEGER_DIGITS`).
- Token-level validation: `crates/sifr_runtime/src/json.rs:147–161` — `validate_integer_token_digit_limit()`.
- Document-level scan: `crates/sifr_runtime/src/json.rs:163–207` — `validate_json_integer_digit_limits()` skips strings, fractional numbers, exponent notation.
- Pre-`serde_json` enforcement in `json_loads`: `crates/sifr_codegen/src/intrinsics/json.rs:676–683` — digit scan is the first `.and_then()` step, before `serde_json::from_str`.
- Explicit boundary API: `lib/sifr/json.sifr:230–231` — `validate_integer_digit_limits(s: str) -> Result[None, JsonLimitError]`.
- E2E coverage: `crates/sifr/tests/e2e/pass/stdlib_json_consolidated.sifr:32–39` — oversized integer test asserts `e.limit == 4096`; `crates/sifr/tests/e2e/pass/stdlib_json_consolidated.sifr:41–45` — `loads` with oversized integer returns `JSONDecodeError` (converted from `JsonLimitError` via `__sifr_json_limit_error_as_decode_error`).

**Status: Implemented.**

#### 6. Map OpenAPI/JSON Schema integer fields according to static range and selected profile

**Assessment**: Deferred. OpenAPI/JSON Schema mapping requires a schema generation surface that does not exist in this repository. The boundary contract (`verification/integer_model_serialization_boundary_rules.md:25–42`) locks the mapping table. The design doc (`internal_docs/integer_model.md:370–374`) defines the requirements. No active surface owns this emitter.

**Evidence**: Implementation inventory (`verification/integer_model_implementation_inventory.md:71`) explicitly defers "Future web/API schema generation and TypeScript/OpenAPI mapping."

#### 7. Define TypeScript client mappings

**Assessment**: Deferred. TypeScript client generation requires a code generation surface that does not exist. The boundary contract (`verification/integer_model_serialization_boundary_rules.md:44–57`) locks the type mappings. No active surface owns this emitter.

#### 8. Define generated `serde::Serialize`/`Deserialize` derive behavior

**Assessment**: Deferred. Serde derive generation for Sifr classes/structs requires a model/schema emitter. The boundary contract (`verification/integer_model_serialization_boundary_rules.md:59–75`) locks the required behavior. The implementation inventory explicitly defers this.

#### 9. Enforce SQL/storage range checks and explicit dtype/schema choices

**Assessment**: Deferred. ORM/storage mapping requires data-layer surfaces. The boundary contract (`verification/integer_model_serialization_boundary_rules.md:77–88`) locks the storage contract. No active surface owns this.

#### 10. Emit diagnostics with field paths and policy suggestions for serialization failures

**Assessment**: Deferred. The diagnostic payload contract for `SIFR-INT-0009` is locked (boundary kind, field/path, selected profile, static range, policy alternatives). The emitter is deferred to the schema/generation phase.

---

### Validation Confirmation

- Each of the four INT-5 PRs has a satisfied review artifact in `reviews/`.
- `#1893` ran `scripts/run_all_tests.sh --profile quick` successfully after the last code change.
- `JsonIntegerRangeError` and `JsonLimitError` are registered in architecture docs with correct parent, fields, and rationale.
- The boundary contract (`verification/integer_model_serialization_boundary_rules.md`) is locked and explicitly defers schema/client/generated-serde/storage emission to later phases.

---

### No Blockers

Every item in the INT-5 scope is either implemented with functional code, active e2e coverage, and architectural documentation, or explicitly deferred with documentation to a later phase that owns the relevant surface. The remaining deferred items have no owning surface in the current codebase — attempting to implement `SIFR-INT-0009`, OpenAPI mapping, TypeScript generation, generated serde, or SQL mapping would require inventing a fake diagnostic path or schema emitter with no caller.

**INT-5 closure review is satisfied.**
