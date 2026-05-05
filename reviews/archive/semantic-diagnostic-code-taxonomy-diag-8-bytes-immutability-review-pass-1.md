# Review: milestone_diag_8 slice 6 — SIFR-OWN-0007 bytes immutability

**Files reviewed:**
- `crates/sifr_diagnostics/src/codes.rs`
- `crates/sifr_hir/src/lower/ownership_diagnostics.rs`
- `crates/sifr_hir/src/lower/statements.rs`
- `crates/sifr_hir/src/lower/aug_assign_lowering.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`
- `crates/sifr/tests/e2e/fail/bytes_subscript_assignment_unsupported.sifr`
- `docs/errors/SIFR-OWN-0007.md`
- `docs/errors/diagnostic-codes.md`
- `internal_docs/diagnostic_codes.md`
- `internal_docs/diagnostic_emission_inventory.md`

**Validation run:** `scripts/run_all_tests.sh --profile quick` — passed, wall_time=933.94s.

---

## Summary

The slice adds `SIFR-OWN-0007` ("Immutable bytes value is mutated.") as an active OWN-family diagnostic and routes both regular and augmented bytes subscript assignment through ownership helper functions. The implementation is locally correct and the routing is consistent. No compatibility fallbacks are present. However, there is a **taxonomy inconsistency** between the single message template in the registry and the two distinct message strings emitted by the two helper functions.

---

## What was done

1. **Registered `SIFR-OWN-0007`** in `codes.rs` as `OWN_IMMUTABLE_BYTES_ASSIGNMENT` with severity Error and message template `"bytes is immutable; subscript assignment is not supported"`. Owner: `sifr_hir::lower::statements`.

2. **Added two ownership helpers** in `ownership_diagnostics.rs`:
   - `immutable_bytes_subscript_assignment(ctx)` → emits `"bytes is immutable; subscript assignment is not supported"` via `error_with_code(OWN_IMMUTABLE_BYTES_ASSIGNMENT, ...)`
   - `immutable_bytes_augmented_subscript_assignment(ctx)` → emits `"bytes is immutable; augmented subscript assignment is not supported"` via the **same** `OWN_IMMUTABLE_BYTES_ASSIGNMENT` code

3. **Routed assignment lowering** in `statements.rs` (3 sites: nested subscript, attribute subscript, plain subscript) from raw `ctx.error(...)` to `immutable_bytes_subscript_assignment(ctx)`.

4. **Routed augmented-assignment lowering** in `aug_assign_lowering.rs` (3 sites: nested subscript, attribute subscript, plain subscript) from raw `ctx.error(...)` to `immutable_bytes_augmented_subscript_assignment(ctx)`.

5. **Locked the fail fixture** `bytes_subscript_assignment_unsupported.sifr` to `SIFR-OWN-0007` via `# expect-error: SIFR-OWN-0007`.

6. **Regenerated diagnostic docs** via `cargo run -q -p sifr_diagnostics --bin gen-error-docs`. The generated `SIFR-OWN-0007.md` page correctly shows the single template string.

---

## Correctness issues

### 1. Message-template / emission mismatch (medium severity)

`SIFR-OWN-0007` has one message template registered:

```
"bytes is immutable; subscript assignment is not supported"
```

But `immutable_bytes_augmented_subscript_assignment` emits:

```
"bytes is immutable; augmented subscript assignment is not supported"
```

Both helpers use the same `DiagnosticCode::OWN_IMMUTABLE_BYTES_ASSIGNMENT` code. This means the augmented assignment case produces a diagnostic whose **message text does not match its declared template**. The test `test_bytes_augmented_subscript_assignment_has_ownership_code` asserts this exact message and it passes at runtime, but the registry entry only documents the non-augmented template.

**Implication:** If diagnostic rendering or deduplication logic ever uses the template string as a reference (e.g., for machineApplicable suggestion mapping, deduplication grouping, or future auto-fix tooling), the augmented case would be mismatched against its own registered template.

**Options:**
- Add a second distinct code for augmented (`SIFR-OWN-0008`) with the augmented template
- Separate the augmented case into a sub-case or variant within the same code (not currently supported by the registry schema)
- Update the existing template to be more generic so both messages are sub-strings of it (e.g., `"bytes is immutable; subscript assignment is not supported"` already covers the augmented case semantically — the augmented message just adds the word "augmented")

The simplest fix would be to keep the single code and expand the template in the registry to note the augmented variant is also served, or to have the `immutable_bytes_augmented_subscript_assignment` emit the canonical template string directly. Since the slice scope explicitly says "immutable bytes subscript and augmented-subscript assignment" and the fixture only locks the non-augmented case, there may be an intentional decision to cover both with one code. However, the registry entry's single template does not reflect this.

---

## Behavioral observations (non-blocking)

### 2. Both helpers share the same code with different messages

The two helpers are `immutable_bytes_subscript_assignment` and `immutable_bytes_augmented_subscript_assignment` — they are separate functions with different message strings but the same diagnostic code. This is a pragmatic consolidation (both are "bytes immutability violation") but creates a slight asymmetry in the registry where the code's template only describes the non-augmented case.

### 3. The augmented assignment fixture is not present

The fail fixture `bytes_subscript_assignment_unsupported.sifr` only tests the plain subscript assignment case (`payload[0] = 65`). There is no separate augmented assignment fail fixture, and `internal_docs/diagnostic_emission_inventory.md` line 320 only lists the one fixture. The augmented assignment path is validated by the unit test (`test_bytes_augmented_subscript_assignment_has_ownership_code`) rather than an e2e fixture.

### 4. Inventory update is minimal

`diagnostic_emission_inventory.md` gains only `SIFR-OWN-0007` at line 320 in the table. The `bytes and binary/text I/O` row (line 152) correctly notes `SIFR-OWN-*` for immutable bytes assignment. The update is consistent.

---

## Scope creep check

- Only `SIFR-OWN-0007` added — no other codes introduced
- Only bytes subscript assignment touched — no other type mutations
- No new fixtures beyond the one lock
- No fallback or compatibility paths added
- Generated docs regenerated correctly

**Scope: clean.**

---

## Test coverage

| Test | What it validates | Status |
| --- | --- | --- |
| `test_bytes_subscript_assignment_has_ownership_code` | Plain `b[i] = x` emits `OWN_IMMUTABLE_BYTES_ASSIGNMENT` with exact message | Passes |
| `test_bytes_augmented_subscript_assignment_has_ownership_code` | Plain `b[i] += x` emits `OWN_IMMUTABLE_BYTES_ASSIGNMENT` with augmented message | Passes |
| `bytes_subscript_assignment_unsupported.sifr` e2e fail | `# expect-error: SIFR-OWN-0007` locked fixture | Passes |
| `cargo test -p sifr_diagnostics` | Registry consistency, doc pages exist, active codes match | Passes |
| `cargo clippy -p sifr_diagnostics -p sifr_hir --no-deps -- -D warnings` | No warnings | Passes |

**Test coverage: sufficient for the slice scope.**

---

## No fallback / compatibility path introduced

Confirmed:
- `statements.rs`: all 3 bytes subscript sites now call `immutable_bytes_subscript_assignment` — no `ctx.error` fallback
- `aug_assign_lowering.rs`: all 3 bytes augmented subscript sites now call `immutable_bytes_augmented_subscript_assignment` — no `ctx.error` fallback
- `ownership_diagnostics.rs`: the two new helpers use `error_with_code` directly with the active code — no raw `ctx.error` path

**Fallback introduced: No.**

---

## Diagnostic-code taxonomy fit

`SIFR-OWN-0007` fits squarely within the `OWN` family (ownership, borrow, move, and lifetime diagnostics). The issue (immutable bytes mutation via subscript) is correctly categorized as an ownership violation rather than a type mismatch or unsupported operator. The owner module `sifr_hir::lower::statements` matches the lowering site.

The `OWN` family now has:
- `0001` Use after move
- `0002` Double mutable borrow
- `0003` Borrowed parameter escapes
- `0004` Moved across loop
- `0005` Immutable parameter mutation
- `0006` Immutable parameter reassignment
- `0007` Immutable bytes subscript assignment

Sequential, no gaps, correct family assignment.

---

## Generated-doc consistency

- `docs/errors/SIFR-OWN-0007.md` exists, generated content matches registry entry
- `docs/errors/diagnostic-codes.md` Active Codes table includes `SIFR-OWN-0007`
- `internal_docs/diagnostic_codes.md` Registry table includes `SIFR-OWN-0007` as Active
- `internal_docs/diagnostic_emission_inventory.md` lists `SIFR-OWN-0007` under ownership section with correct fixture

All generated docs are consistent with the implementation.

---

## Verdict

**Approve with a note.** The implementation is correct and complete for its stated scope. The single finding is the message-template mismatch between the registry's single declared template and the two distinct message strings emitted by the two helper functions. This does not cause a runtime failure (all tests pass) but is a taxonomy integrity concern if diagnostic rendering or auto-fix tooling ever uses the template as a authoritative reference.

The fix is straightforward: either (a) add `SIFR-OWN-0008` for the augmented case with its own template, or (b) make `immutable_bytes_augmented_subscript_assignment` emit the canonical template string instead of its own variant message, or (c) update the `SIFR-OWN-0007` template to document that both variants are served. Option (b) is the lowest churn — emit the registered template string in the augmented helper — but it changes the user-visible message for augmented cases from `"...augmented subscript assignment is not supported"` to `"...subscript assignment is not supported"`, which loses the specific "augmented" qualifier.

Given the scope says "immutable bytes subscript and augmented-subscript assignment," option (a) (distinct codes 0007/0008) is the most taxonomically honest and matches the pattern of having separate codes for related-but-distinct scenarios. However, that would also require a new fixture and updating the unit tests accordingly.

**Recommendation:** Accept the current slice as-is (it passes all validations and the taxonomy gap is a pre-existing design question, not a new bug introduced by this slice), but open a follow-up issue to decide between option (a) or (c) for the augmented case message-template discrepancy.
