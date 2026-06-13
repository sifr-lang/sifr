I'll review the staged changes for milestone 3 of the ad hoc async diagnostics: "Restrict Blocking Offload Targets."

## Review Summary

### Design Requirements Check

| Requirement | Status |
|-------------|--------|
| Reject `task.spawn_blocking` on unannotated local sync functions | ✅ |
| Reject `ThreadPoolExecutor.submit` on unannotated local sync functions | ✅ |
| Allow annotated `@blocking_io` targets | ✅ |
| Allow annotated `@cpu_heavy` targets | ✅ |
| Preserve existing sendability, arity, and result/error constraints | ✅ |
| Add validation fixtures | ✅ |
| Minimal known-stdlib classification (`uuid4_obj`) | ✅ |

---

### Correctness Analysis

**1. Diagnostic Code Registration** (`codes.rs`)
- New code `SIFR-ASYNC-0005` properly added to `DIAGNOSTIC_REGISTRY`, `ACTIVE_DIAGNOSTIC_CODES`
- Owner correctly set to `sifr_hir::lower::workload_annotations`
- Message template and args are consistent

**2. Implementation Logic** (`workload_annotations.rs`)
- `reject_unclassified_offload_target` correctly checks:
  - Non-name expressions → error (can't validate)
  - Functions with workload annotation → allowed
  - Known stdlib targets (`uuid4_obj`) → allowed  
  - Everything else → rejected
- Error messages are clear and actionable

**3. Integration Points**
- `task_calls.rs`: Correctly calls `reject_unclassified_offload_target` before result type handling
- `blocking_executor_calls.rs`: Same pattern for `ThreadPoolExecutor.submit`
- Both return early on rejection, preserving existing error flow

---

### Fixture Coverage

| Fixture | Purpose | Coverage |
|---------|---------|----------|
| `spawn_blocking_unannotated_rejected.sifr` | Fail: unannotated rejected | ✅ |
| `thread_pool_submit_unannotated_rejected.sifr` | Fail: unannotated rejected | ✅ |
| `spawn_blocking_blocking_io_allowed.sifr` | Pass: `@blocking_io` allowed | ✅ |
| `spawn_blocking_cpu_heavy_allowed.sifif` | Pass: `@cpu_heavy` allowed | ✅ |
| `spawn_blocking_known_stdlib_blocking_allowed.sifr` | Pass: stdlib `uuid4_obj` | ✅ |

**Existing fixture updates**: All existing pass/fail fixtures correctly annotated to remain valid under new constraints.

---

### Registry & Documentation Consistency

- `docs/errors/SIFR-ASYNC-0005.md` generated correctly
- `docs/errors/diagnostic-codes.md` updated
- `internal_docs/diagnostic_codes.md` updated
- `internal_docs/phases/32_async_ecosystem.md` status updated
- `issues/ad-hoc-async-effect-and-offload-diagnostics-execution.md` checkboxes marked complete
- `quick_e2e_manifest.json` updated with new pass fixtures

---

### Regression Analysis

**Preserved from previous milestones:**
- Non-`Send` rejection (`spawn_blocking_non_send_rejected.sifr`) still rejects
- Arity constraints preserved (check happens before classification check in error case)
- Result type constraints preserved (result type extraction unchanged)
- Direct sync calls without `spawn_blocking` remain allowed

**Unit test updates:**
- `test_spawn_blocking_lowers_to_blocking_task_handle`: Updated source to include `@cpu_heavy`
- `test_thread_pool_executor_submit_lowers_to_blocking_task_handle`: Same
- `test_thread_pool_executor_submit_rejects_non_send_return`: Updated source
- New tests added for rejection cases with correct error code assertions

---

### Minor Observations (Non-blocking)

1. The `None` error message says "must be a named sync function classified as..." but the actual rejection is "not classified". This is slightly inconsistent with the message wording, but the actual behavior is correct.

2. The `group-skew advisory` from the quick validation is documented as advisory-only, not blocking.

---

### Validation Results Confirmed

- ✅ `cargo test -p sifr_hir test_spawn_blocking_rejects_unclassified_target`
- ✅ `cargo test -p sifr_hir test_thread_pool_executor_submit_rejects_unclassified_target`
- ✅ `cargo test -p sifr_diagnostics codes::tests::registry_skeleton_is_internally_consistent`
- ✅ `cargo test -p sifr -- test_e2e_fail`
- ✅ `cargo run -q -p sifr -- check` on all offload pass fixtures
- ✅ `cargo run -q -p sifr -- run` on `spawn_blocking_known_stdlib_blocking_allowed.sifr`

---

## Approval

**The milestone is approved.** All design requirements are met, implementation is correct, fixture coverage is complete, and existing constraints are preserved.
