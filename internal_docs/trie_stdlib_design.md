# Trie Stdlib Decision

Status: accepted for WS2 S6

## Decision

Add an explicit `sifr.trie.Trie` type instead of nested-dict auto-insert helpers.

The trie is backed by owned node indices:

- `list[list[tuple[str, int]]]` stores outgoing edges by character.
- `list[bool]` stores terminal-word markers.
- Public methods expose whole-word operations and proof-friendly node traversal helpers.

## API

- `insert(word: str) -> None`
- `contains(word: str) -> bool`
- `search(word: str) -> bool`
- `starts_with(prefix: str) -> bool`
- `startsWith(prefix: str) -> bool`
- `find_node(text: str) -> int | None`
- `child(node: int, ch: str) -> int | None`
- `children(node: int) -> list[int]`
- `is_terminal(node: int) -> bool`
- `node_count() -> int`

## Rationale

Recursive trie node objects would require self-referential class fields and shared mutable child aliases that are not part of Sifr's current ownership model. Nested-dict helpers would make the canonical LeetCode rewrites possible, but they invite Python-style auto-insert-on-read semantics unless every call site is carefully constrained.

The chosen API keeps mutation explicit: `insert` is the only operation that creates nodes. Reads return `Option` values for missing edges or invalid node indices, so wildcard DFS and board-search rewrites can remain proof-gated without implicit unwraps or user-triggerable panics.

The implementation uses owned edge lists instead of nested dictionaries because current string-key dictionary lookup lowering is not yet a safe dependency for stdlib codegen. This preserves trie traversal semantics and avoids auto-insert-on-read behavior. A later internal representation swap to maps can be made behind the same API when dictionary lowering is ready.

## Fixture Implications

- `0208_implement_trie_prefix_tree` can import `sifr.trie.Trie` directly with LeetCode-compatible method names.
- `0211_design_add_and_search_words_data_structure` can use `child`, `children`, and `is_terminal` for wildcard DFS without per-word linear scans.
- `0212_word_search_ii` can use node indices to prune board DFS by prefix.
- `1397_find_all_good_strings` remains better served by KMP/lps state than by a trie for this phase.
