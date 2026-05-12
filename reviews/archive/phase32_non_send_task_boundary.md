

Based on my comprehensive review of the Phase 32 non-send task-boundary slice, here is my assessment:

---

## Review: Phase 32 Non-Send Task Boundary Slice

### Documents Reviewed
- `internal_docs/phases/32_async_ecosystem.md` — milestone_async_4 status and definition of done
- `internal_docs/async_concurrency_model.md` — canonical async/concurrency model (lines 569-571)
- `docs/errors/SIFR-OWN-0010.md` — auto-generated diagnostic doc

### Changed Files Reviewed
| File | What it does | Status |
|---|---|---|
| `typing_and_functions.rs` | Registers `NonSend` as zero-runtime marker base class | ✓ |
| `task_scope_calls.rs` | Structural sendability check via `non_send_reason_inner` with cycle guard | ✓ |
| `ownership_diagnostics.rs` | Emits `SIFR-OWN-0010` diagnostic | ✓ |
| `classes.rs` | Excludes `NonSend` from parent-field emission in `CLASS_MISSING_INITIALIZER` check | ✓ |
| `class_emitter.rs` | Excludes `NonSend` parent from struct fields and default constructor | ✓ |
| `codes.rs` | Registers `OWN_NON_SEND_TASK_CAPTURE` constant | ✓ |
| `diagnostic-codes.md`, `internal_docs/diagnostic_codes.md` | Regenerated/updated | ✓ |
| `spawn_non_send_field_rejected.sifr` | E2E fixture: direct NonSend field | ✓ (untracked) |
| `spawn_self_with_non_send_field_rejected.sifr` | E2E fixture: self-with-NonSend field | ✓ (untracked) |

### Correctness Analysis

**1. `NonSend` marker registration** (`typing_and_functions.rs:53-64`)
- Registered as empty-field class with no parent — zero runtime footprint
- Correctly available as compiler-recognized built-in without import

**2. Structural sendability check** (`task_scope_calls.rs:161-214`)
- `non_send_reason_inner` recursively traverses:
  - Class inheritance chain (`class_has_non_send_marker`)
  - Class fields
  - Container component types (`List`, `Set`, `Dict`, `Result`, `Task`, `Coroutine`, etc.)
  - Tuple/Union/Intersection elements
  - Alias/Newtype/Callable parameterization
- **Cycle guard**: `visiting` HashSet prevents infinite recursion on self-referential types
- Returns `None` (sendable) for all other types — conservative, safe

**3. `class_has_non_send_marker`** (`task_scope_calls.rs:216-219`)
- Checks direct name `== "NonSend"`
- Splits parent chain by `|` and checks for `NonSend`
- Correctly handles any single-parent or multi-parent class

**4. Diagnostic emission** (`ownership_diagnostics.rs:132-146`)
- Message: `scope.spawn() cannot move \`{value}\` of type \`{ty}\` across a task boundary because {reason}`
- Structured args: `value`, `type_name` (message+json), `reason` (json-only)
- Dedupe args: `value`, `type_name`, `reason`

**5. Codegen marker treatment** (`class_emitter.rs:76-85, 355-362`)
- `NonSend` parent excluded from struct fields: `if parent != "NonSend"`
- Default constructor skipped when parent is `NonSend` or class has no parent

### Coverage Analysis

| Milestone criterion | Fixture | Implementation | Status |
|---|---|---|---|
| User class inheriting `NonSend` not sendable | `spawn_non_send_field_rejected.sifr` | `non_send_reason_inner` → `class_has_non_send_marker` | ✓ |
| Class containing NonSend fields not sendable | Unit test `test_scope_spawn_rejects_non_send_field_argument` | Recursive field traversal | ✓ |
| Self-with-NonSend-field rejection | `spawn_self_with_non_send_field_rejected.sifr` + unit test | Same recursive check | ✓ |
| Container component types checked | Implicit via `non_send_reason_inner` | Recursively checks `Type::List`, `Result`, etc. | ✓ |
| Cycles guarded conservatively | Implicit via `visiting` HashSet | Returns `None` on cycle | ✓ |
| `SIFR-OWN-0010` emitted (not Rust error) | Fixture expects `SIFR-OWN-0010` | `ownership_diagnostics::non_send_task_capture` | ✓ |

### Doc Accuracy
- `docs/errors/SIFR-OWN-0010.md` — correctly generated from `gen-error-docs`
- `internal_docs/phases/32_async_ecosystem.md` — updated milestone progress
- `internal_docs/async_concurrency_model.md` — `NonSend` marker treatment documented

### Validation Results
```
cargo test -p sifr_hir -- test_scope_spawn_rejects_non_send_field_argument ... ok
cargo test -p sifr_hir -- test_scope_spawn_rejects_self_with_non_send_field ... ok
cargo run -q -p sifr -- check spawn_non_send_field_rejected.sifr → SIFR-OWN-0010 ✓
cargo run -q -p sifr -- check spawn_self_with_non_send_field_rejected.sifr → SIFR-OWN-0010 ✓
cargo test -p sifr -- test_e2e_fail ... ok
```

### Issue Found: One Fixture Untracked

`spawn_self_with_non_send_field_rejected.sifr` is in `git status` as untracked (`??`) despite being listed in the changed files. The fixture is correct and the test passes, but it should be `git add`ed to be part of the commit.

---

## SATISFIED

The non-send task-boundary slice correctly implements:
- `NonSend` as a zero-runtime marker base class
- Structural sendability checking with recursive field traversal
- Cycle-safe traversal via `visiting` HashSet
- `SIFR-OWN-0010` as a stable Sifr-native diagnostic
- Codegen that treats `NonSend` as a marker (not a runtime parent field)

**One action required**: `git add` the untracked fixture `spawn_self_with_non_send_field_rejected.sifr` before committing.
