# LeetCode Trie Helper Decision

Status: accepted for LeetCode fixture scope

## Decision

Keep the explicit trie helper in `audits/leetcode/src/helpers/trie.sifr` instead of adding a public
`sifr.trie` stdlib module.

The helper is backed by owned node indices:

- `list[list[tuple[str, int]]]` stores outgoing edges by character.
- `list[bool]` stores terminal-word markers.
- Helper methods expose whole-word operations and proof-friendly node traversal helpers.

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

The trie is needed to make the LeetCode trie fixtures canonical, but it is not a general Sifr
stdlib commitment. Shipping it in `lib/sifr` would prematurely expand the public stdlib surface
for a corpus-specific helper and would create a false Python parity claim.

Workspace source-root resolution now supports helper imports from non-`main.sifr` LeetCode root
fixtures. Trie-dependent Sifr fixtures import the shared helper with:

```sifr
from helpers.trie import Trie
```

The implementation keeps mutation explicit: `insert` is the only operation that creates nodes.
Reads return optional values for missing edges or invalid node indices, so wildcard DFS and
board-search rewrites remain proof-gated without implicit unwraps or user-triggerable panics.

## Fixture Implications

- `0208_implement_trie_prefix_tree` imports the helper and preserves LeetCode-compatible method names.
- `0211_design_add_and_search_words_data_structure` imports the helper and uses `child`,
  `children`, and `is_terminal` for wildcard DFS without per-word linear scans.
- `0212_word_search_ii` imports the helper and uses node indices to prune board DFS by prefix.
- `1397_find_all_good_strings` remains better served by KMP/lps state than by a trie for this phase.
