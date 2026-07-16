# M10 Wave 2 Full-Diff Review — Pass 4

Reviewer: Codex CLI `gpt-5.6-sol`, high reasoning, fast service tier
Frozen scope: `main...d1afe5eb7` (PR #2988)

## Blockers

1. **High — writable admission can be bypassed through distinct views of one backing exporter.** Admission is keyed only by `Py_buffer.obj` (`raw.rs:122`, `buffer_ops.rs:249`). CPython reports each immediate `memoryview` as `obj`: two `memoryview(bytearray)` instances can have different identities while sharing the same data pointer. The permanent test reacquires only through one object, so writer/writer or reader/writer aliases over shared backing memory remain possible.

2. **High — generator, iterator, and borrowed-parameter clone paths still admit affine buffers.** `iter`, `next`, `zip`, `map`, `filter`, `reversed`, and `enumerate` lack the affine iteration guard. Preserved affine collections select clone-based iterator lowering. `yield view` neither consumes the binding nor rejects reuse. Generator bodies and reassigned mutable borrowed parameters clone borrowed non-copy parameters, but `PythonBuffer` intentionally has no `Clone` implementation.

3. **High — generic `TypeVar`/`Any`/`Unknown` collection capability handling remains unsound.** Hashability validation admits unresolved and dynamic types, affine collection checks see only unresolved `TypeVar`, and generic call specialization does not revalidate generated Rust clone/hash/equality requirements. Generic collection functions instantiated with `Buffer` and dynamic set/dict shapes can therefore reach Rust trait failures.

4. **High — conditional expressions evade affine move tracking.** A bare affine name in either branch records no move, while assignment consumes only a direct name. The checker accepts assigning `left if flag else right` and then reusing `left`, although code generation moves the selected branch.

5. **Medium — permanent coverage does not cover the remaining acceptance surfaces.** Missing cases include iterator builtins, yield statements, conditional expressions, borrowed-parameter clone sites, generic/dynamic specialization, and distinct wrappers over shared buffer storage.

6. **Medium — activation documentation remains contradictory.** The diagnostic registry and capability ledger activate `PYZC`, but `internal_docs/python_interop_architecture.md` still says it remains reserved.

## Cleared Areas

No separate blocker was found in exact-once release after admission, access/release linearization, count cleanup, producer retention, declaration validation, or generated import-root/bridge/`Self` acquisition. Diagnostic registration and touched-file responsibility limits were otherwise sound. The create-PR timing overrun was assessed as environmental.

## Verdict

VERDICT: CHANGES REQUIRED
