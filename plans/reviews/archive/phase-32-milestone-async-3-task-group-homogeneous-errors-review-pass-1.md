

Based on my review, here is my assessment:

## Review: TaskGroup homogeneous-error slice

### Scope & Tracking

The tracking is scoped correctly: only `TaskGroup` bindings (not `TaskScope`) via `group.spawn(...)` calls. The `is_task_group_type` guard ensures `scope.spawn` is unaffected. The `task_group_error_types` map is per-module lowering context, which is appropriate for the current single-function analysis scope.

### Never Handling

The `Never` coercion is correct:
```rust
if matches!(task_err_ty.resolve_alias(), Type::Never) {
    return Some(());
}
```
`Never` children bypass tracking entirely, making them compatible with any group error type. The bidirectional assignability check (`is_assignable_to` both ways) in the existing-type comparison further ensures type-compatible children are accepted.

### Diagnostic

```sifr
task.TaskGroup() children must share one error type in v1; expected 'ValueError', got 'IOError'
```
Points at the second spawn arg, identifies the mismatch, and is appropriately scoped to v1. This is acceptable for the milestone slice.

### Negative Fixture Verification

```
$ cargo run -q -p sifr -- check .../task_group_heterogeneous_error_rejected.sifr
type error: [main] task.TaskGroup() children must share one error type in v1; expected 'ValueError', got 'IOError'
```
The fixture fires correctly and produces the expected diagnostic.

### Potential False Positive Concern (non-blocking)

Branch-separated spawns are treated as sequential:

```sifr
async with task.TaskGroup() as group:
    if cond:
        group.spawn(value_error_child())  # records ValueError
    else:
        group.spawn(io_error_child())     # second record, rejects
```

This is conservative and correct per the phase doc ("rejects later `group.spawn(...)` children with a different ordinary error type"). However, the branches are mutually exclusive — the group will only ever observe one error type at runtime. The current implementation lacks flow/branch tracking to allow this pattern.

**This is a non-blocking note.** Fixing it requires analysis beyond this slice's scope. The conservative behavior is defensible for v1 and can be addressed as a follow-up when branch-tracking infrastructure exists.

---

**Verdict: SATISFIED, with non-blocking notes**

The slice is correct, tests pass, and the false-positive concern does not block merge. The implementation correctly enforces homogeneous error types for `TaskGroup` children in v1, handles `Never` children properly, and produces an acceptable diagnostic.
