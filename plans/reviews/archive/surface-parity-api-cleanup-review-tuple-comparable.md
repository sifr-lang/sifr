- Lexicographic tuple ordering is standard semantics in both Python and Rust — no semantic surprises.
- The current narrow gating is an implementation gap, not a principled design choice.
- heapq fixtures explicitly depend on `tuple[int, int]` being orderable, making this a real-world requirement.
- Closing the gap aligns Sifr's `Comparable` contract with what the runtime actually guarantees.
- Treating it as a compiler feature keeps parity explicit and prevents silent divergence as the language evolves.

DECISION: compiler_feature

[exit_code]=0
