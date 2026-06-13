# Category 2b Review — Collection / Index / Stdlib Ergonomics

Date: 2026-04-21
Scope: independent review of Category 2b in `verification/leetcode/leetcode_divergence_decision_analysis_20260409.md`.
Cross-checks: `verification/leetcode/leetcode_pair_diff_scan_20260409.json`, paired fixtures under `audits/leetcode/`.

## Summary

All 22 fixtures currently listed in Category 2b are correctly classified. No fixture should move out, and no unclassified fixture with `changed_total_lines >= 80` in the raw scan looks like a misplaced Category 2b case. The stated improvement list is directionally right but imprecise in two ways: it treats "string-to-int parsing" as a monolithic bullet when the pain is actually character-digit classification plus decimal parsing; and it understates how much leverage comes from a real `heap` primitive relative to the other stdlib bullets.

## Classification dividing lines used in this review

- **Category 1 (rewrite debt)**: public model changed, asymptotic regression, or the canonical algorithm is absent.
- **Category 2a (recursive/cursor ergonomics)**: linked-list or tree cursor rewiring, or Optional narrowing lost across recursive field projections.
- **Category 2b (collection/index/stdlib)**: dead Optional guards on proven list/dict index access, missing stdlib primitive (heap / deque / DSU / parsing / trie), or owned-collection cloning noise.
- **Category 3 (okay as-is)**: Sifr side is fine, diff inflated by noisy Python side.
- **Category 4 (architecture boundary)**: mutable `nonlocal` closure or shared-ownership aliasing that Sifr intentionally rejects.

## Per-Fixture Verdicts

- **0130_surrounded_regions** — Grid DFS with proven-bounds indexing but dead Optional guard boilerplate on `board[r][c]`. Keep in 2b. Gap: Optional-flow preservation on list-of-list access.
- **0150_evaluate_reverse_polish_notation** — Manual `digitValue()` / `parseSignedInt()` boilerplate replaces `int(token)`. Stack pops carry dead Optional fallbacks inside proven-non-empty branches. Keep in 2b. Gap: string→int parsing plus stack pop flow.
- **0261_graph_valid_tree** — Hand-rolled DSU over `dict[int, int]` with dead Optional guards on dict reads inside keys that were just inserted. Keep in 2b. Gap: DSU helper + map-read narrowing after contains-check.
- **0269_alien_dictionary** — Kahn's with manual adjacency and in-degree maps, plus zip-by-index ceremony. Dead guards on entries that were just initialized. Keep in 2b. Gap: defaultdict-style map ergonomics and adjacency containers.
- **0286_walls_and_gates** — BFS over list-as-queue with `head` index cursor and dead Optional guards on `rooms[r][c]` under proven-bounds `(r, c)`. Keep in 2b. Gap: `deque.popleft` + Optional-flow preservation.
- **0297_serialize_and_deserialize_binary_tree** — Manual `parseIntToken` replaces `int(token)`; otherwise tree recursion is fine. Keep in 2b. Gap: string→int parsing; the recursive tree walk itself is not a 2a cursor rewire.
- **0355_design_twitter** — Manual heap via sort-on-push or linear-scan-on-pop; Python uses `heapq`. No public model change. Keep in 2b. Gap: `heap` primitive.
- **0394_decode_string** — Manual `isDigit` table and digit accumulator replace `str.isdigit()` + `int()`. Stack-of-contexts pattern otherwise matches Python. Keep in 2b. Gap: character-class helpers + string→int parsing.
- **0417_pacific_atlantic_water_flow** — DFS with proven-bounds grid access; dead Optional guards on `heights[r][c]`. Keep in 2b. Gap: Optional-flow preservation.
- **0513_find_bottom_left_tree_value** — BFS with list-as-queue; `head` cursor and dead `None` guards on tree fields. Borderline with 2a on the field-read side, but the dominant pattern is the queue ergonomics, not recursive cursor rewire. Keep in 2b. Gap: `deque`, plus modest non-optional narrowing on tree children (already covered by 2a).
- **0567_permutation_in_string** — Sliding window with fixed-size count arrays; dead Optional guards on `count[ord(ch) - ord('a')]` inside proven-bounds slots. Keep in 2b. Gap: Optional-flow preservation for fixed-size integer arrays.
- **0721_accounts_merge** — DSU by email string, with map `.get()` + None guard chains on keys that were just inserted. Keep in 2b. Gap: DSU helper + dict narrowing after insert.
- **0743_network_delay_time** — Dijkstra with a manual priority-queue that encodes `(weight, node)` as a single int via modular packing. The algorithm is the canonical one; only the heap is missing. Keep in 2b. Gap: `heap`.
- **0752_open_the_lock** — BFS over lock states with list-as-queue; dead guards on string-indexing of already-bounded wheel positions. Keep in 2b. Gap: `deque` + small-string/char ergonomics.
- **0778_swim_in_rising_water** — Dijkstra-variant with manual heap; algorithm shape matches Python. Keep in 2b. Gap: `heap`.
- **1203_sort_items_by_groups_respecting_dependencies** — Layered topological sort with nested dict-of-list adjacency; defensive `.get()` + None chains on entries the code itself initialized. Keep in 2b. Gap: adjacency-map ergonomics + Optional-flow preservation.
- **1397_find_all_good_strings** — KMP + DP with trie-like offset table and manual `int(ch) - int('0')` digit indexing. Keep in 2b. Gap: trie-friendly dict ergonomics + character→index parity.
- **1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree** — Kruskal driven by a hand-rolled DSU, run twice per edge. Keep in 2b. Gap: DSU helper.
- **1584_min_cost_to_connect_all_points** — Prim with manual heap; algorithm matches Python, only the heap primitive is absent. Keep in 2b. Gap: `heap`.
- **1631_path_with_minimum_effort** — Dijkstra with manual heap; direct analogue of 0743 / 0778 / 1584. Keep in 2b. Gap: `heap`.
- **2092_find_all_people_with_secret** — Sort-by-time batch DSU with union-then-reset logic. Keep in 2b. Gap: DSU helper with optional reset/detach.
- **2709_greatest_common_divisor_traversal** — DSU over prime-factor groups. Keep in 2b. Gap: DSU helper.

## Cross-Check Against The Raw Scan

Using `leetcode_pair_diff_scan_20260409.json`, every paired fixture with `changed_total_lines >= 80` is already placed in Category 1, 2a, 2b, 3, or 4. A few to call out explicitly so the audit trail is clear:

- `0146_lru_cache`, `0208_implement_trie_prefix_tree`, `0211_design_add_and_search_words_data_structure`, `0745_prefix_and_suffix_search`, `0332_reconstruct_itinerary`, `1091_shortest_path_in_binary_matrix`, `0049_group_anagrams`, `1514_path_with_maximum_probability`, `0103_binary_tree_zigzag_level_order_traversal`, `2101_detonate_the_maximum_bombs`, `0102_binary_tree_level_order_traversal`, `1834_single_threaded_cpu`, `1220_count_vowels_permutation`, `1345_jump_game_iv`, `0909_snakes_and_ladders`, `0127_word_ladder` — these appear in the top of the scan but are **not** in any of Categories 1–4 in the analysis document. Several of them (e.g. `0146_lru_cache`, `0208_implement_trie_prefix_tree`, `0211_design_add_and_search_words_data_structure`, `0745_prefix_and_suffix_search`, `0332_reconstruct_itinerary`, `1091_shortest_path_in_binary_matrix`, `1834_single_threaded_cpu`, `0127_word_ladder`, `0909_snakes_and_ladders`, `1345_jump_game_iv`, `0103_binary_tree_zigzag_level_order_traversal`, `0102_binary_tree_level_order_traversal`, `1514_path_with_maximum_probability`, `1220_count_vowels_permutation`) look on inspection like collection / deque / heap / trie ergonomics cases that would belong in 2b if they were included.

This is outside the "review only Category 2b" scope, but it is the one audit finding worth flagging: the scope rule at the top of the analysis (`changed_total_lines >= 80` plus manual exceptions) is not actually exhaustively applied — several large-diff fixtures are simply unlisted. That is a coverage issue for the analysis as a whole, not a misclassification inside 2b. No action inside 2b required; flagged here for the owner of the analysis file.

## Accuracy Of The Stated Improvement List

The current bullet list for 2b is:

1. preserve proven non-Optional collection/index values across normal statement flow
2. safer owned collection helpers with minimal cloning and predictable ownership
3. stdlib parity: `heap`, `deque`, DSU / union-find, string-to-int parsing, trie-friendly dictionary ergonomics

This is faithful to Sifr principles (no truthiness, no implicit nullable access, no ownership-weakening aliasing) and matches what shows up in the fixtures. Two sharpening edits are worth making:

- **Split "string-to-int parsing"** into two concrete sub-items: (a) character-class predicates such as `ch.isdigit()` / `ch.isalpha()`, and (b) full-token `int(s)` parsing returning a `Result`. Fixtures 0394 and 1397 are dominated by (a); 0150 and 0297 are dominated by (b). Folding them together hides the fact that (a) is the cheaper, higher-frequency win.
- **Name the map-after-insert narrowing case explicitly** under the Optional-flow bullet. Multiple DSU fixtures (0261, 0721, 2092, 2709) waste lines re-guarding a dict key that the code itself just inserted. This is distinct from "index access on a proven-bounds list" and deserves its own sub-point because the fix is different (parent/rep-dict invariant preservation, or a DSU helper that hides the dict behind a typed API).

The "owned collection helpers with minimal cloning" bullet is correctly present but is not materially exercised by any of the 22 fixtures here — none of them are dominated by clone-heaviness. It belongs in 2b but should be marked as low current leverage relative to the stdlib items, so roadmap work is not prioritized against an empty pressure source.

## Suggested Ordering Of The 2b Work

Fixture-count evidence (pain × frequency) supports this order:

1. **`heap`** — unblocks 0355, 0743, 0778, 1584, 1631 (five fixtures, all with manual priority-queue boilerplate that materially obscures the canonical algorithm).
2. **DSU helpers** — unblock 0261, 0721, 2092, 2709, 1489 (five fixtures).
3. **`deque`** — unblocks 0286, 0513, 0752 (three fixtures) and cleans the manual `head` cursor pattern repeated in every BFS.
4. **Optional-flow preservation on list/dict index access** — scattered across 0130, 0286, 0417, 0567, 0752, 1203; high total surface, but each individual fixture gains less than a single stdlib primitive does.
5. **`int(s)` parsing + `str.isdigit()`** — unblock 0150, 0297, 0394, 1397 (four fixtures); small per-fixture win that compounds with stdlib-level changes.
6. **Trie-friendly dict ergonomics** — only 1397 exercises this in 2b; low standalone priority but pairs naturally with the trie-design fixtures sitting outside 2b (0208, 0211, 0745).

This is a work-sequencing recommendation, not a severity ranking, consistent with the analysis document's own framing.

## Concrete Edits To The Analysis File

No fixture moves in or out of Category 2b. Two text-level edits are recommended:

1. In the "What should improve" list under section 2b, split the `string-to-int parsing` bullet into `character-class predicates (isdigit/isalpha)` and `whole-token int parsing returning Result`, and add a sub-bullet under Optional-flow preservation for `dict index narrowing after insert / contains-check`.
2. In the "Practical Priority Order" section, promote `heap` above DSU in the stdlib unblock order. The current order already lists `heap` first among stdlib primitives, so this is a no-op for the document — but confirming it here closes the loop and avoids a future reshuffle when the owner cross-references fixture counts.

No other edits to Category 2b are required.
