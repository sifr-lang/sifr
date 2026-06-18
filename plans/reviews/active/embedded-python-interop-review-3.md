1. **No remaining blockers.** All polish items confirmed in place:
   - `call_attr(obj, "name", args, kwargs)` signature is consistent at lines 248, 262, 287, 369, 372.
   - `PyValue` declared in the runtime API sketch (line 184) and referenced by `call`/`getitem` (lines 196-197).
   - `zero_copy_as`/`copy_as` rows present in the lowering table (lines 291-292).
   - Arrow Rust↔Sifr mapping noted at line 190 ("Generic wrappers specialize into Sifr-level ArrowArray, ArrowStream, and ArrowSchema handles").
   - `LocalCallback` explicitly non-`Send` (line 502).
   - `BufferView` acquisition named at line 431.
   - Coroutine cleanup split into two clear steps (lines 369-370).

2. **One low-signal nit, not worth blocking:** the conceptual Rust block uses `pub enum PyValue;` (line 184), which isn't literal valid Rust — but the surrounding `pub struct PyObjectHandle;` lines use the same illustrative shorthand, and the section is titled "Conceptual Runtime API". Consistent within the block; leave as-is.

3. **Ready to commit.**
