## Review: sifr-lang/leetcode PR #40 (`a20d9d5` → `9f3cf6e`)

**Diff scope:** one file, `src/0146_lru_cache.sifr`, +4/−2. Two call sites rewritten from `self.insertAfter(node, self.head)` to `head = self.head` / `self.insertAfter(node, head)` (`moveToFront`, line 48–49; `put`, line 95–96). Nothing else in the corpus changed.

### 1. LRU semantics preserved

`self.head` is written exactly once, at `__init__` line 18 (`self.head = 0`), and never reassigned anywhere in the class — the only other reads are line 56 (`node == self.head`) and the two migrated sites. `head: int` is a scalar value type, so `head = self.head` is a value copy taken at the identical program point as the previous inline read; no intervening statement exists between the snapshot and the call in either method. The two forms are value-identical unconditionally, not just for the fixture's trace.

I also traced `main()` against the post-change list structure (sentinels 0/1, cap 2): `put(1,1)`/`put(2,2)` build `0↔3↔2↔1`; `get(1)==1` promotes node 2; `put(3,3)` evicts key 2; `get(2)==-1`; `put(4,4)` evicts key 1; `get(1)==-1`, `get(3)==3`, `get(4)==4`. The `cap=1` block behaves likewise. Doubly-linked-list invariants and eviction order are unchanged.

### 2. Sufficient for all affected call sites

I scanned all 411 `.sifr` fixtures in `src/` (31 of which define `self` methods) for any call passing a `self.<field>` as an argument, both same-line and as a continuation-line argument:

- Zero remaining method calls of the form `<recv>.<method>(… self.<field> …)` anywhere in the corpus after this change.
- The only surviving `self.`-valued arguments are to free functions with no `mut` parameter: `len(self.…)` (0155, 0146:87), and `_int_at` / `_child` / `_bool_at` / `_terminal` in 0208/0212, whose signatures take plain `list[…]`/`dict[…]` params. These are shared reads with no mutable place involved, so the unified prefix-overlap rule in plan §4 does not implicate them.
- Pre-existing self-reads that are *not* affected and correctly left alone: `self.prev[self.tail]` (line 52) and `len(self.key_to_node) >= self.cap` (line 87) — neither is a `MutableBorrow` receiver call, so neither is a receiver/argument overlap.

The remaining `self.method(self.…)` hits in the repo (`0146_lru_cache.py:24,25,30,32`, `0721_accounts_merge.rs:20`, `2709_greatest_common_divisor_traversal.rs:20`) are paired Python/Rust reference implementations, not compiled by Sifr. Correctly untouched.

### 3. No ownership or type regression

`head` is a fresh owned `int` local, type inferred from the `head: int` field declaration — no annotation needed and none of the corpus's other locals annotate scalars either. Because it is a value copy, no shared borrow of `self` is live across the `&mut self` `insertAfter` call, which is precisely what the new exclusivity rule requires. The local is never reassigned, so it does not need `mut` and cannot trip the IR mutability pass. In `put`, `head` is introduced after `node` at line 90 with no shadowing of any binding or field accessor. No new heap allocation, clone, or move out of `self` is created.

### 4. Appropriate to merge

This matches the parent milestone's deliberate design decision rather than working around a defect: plan §4 states an overlapping shared read of a `MutableBorrow` receiver place is rejected with `SIFR-OWN-0002`, and explicitly "intentionally rejects `self.helper.read(self.helper)` rather than depending on argument auto-cloning or Rust two-phase-borrow behavior." `self.head` is a prefix-overlap of the receiver place `self`, so the pre-change source is rejected by design. The issue's own status text (lines 17–19) names this exact migration as required work, and the Item 2 evidence block (line 786) records that the migrated fixture compiles, runs, and passes the 12-case representative suite. The change is the minimal accommodation — a snapshot local, not a semantic or algorithmic rewrite.

### Non-blocking observations

- **Unexplained temporaries.** The fixture already carries a header comment (lines 3–4) explaining its *other* ownership accommodation (integer node ids instead of a cyclic `Node` graph). The two new `head = self.head` lines have no such note, and read as removable redundancy to anyone unaware of same-call exclusivity. A one-line comment at each site, or an extension of the header comment, would protect them from a future "cleanup." Purely a documentation nit; it does not affect correctness or merge readiness.
- **Conservatism of the rule, not of this PR.** For a `Copy` scalar field like `head`, rejecting the inline read is stricter than aliasing safety strictly requires. That is a property of the approved design (readiness pass 7 returned `SATISFIED` on it), not a defect in this corpus change, and re-litigating it belongs in the parent issue rather than here.

No correctness, ownership, typing, sufficiency, or scope problems found.

SATISFIED
