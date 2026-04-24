# LeetCode 0146 LRU Recency Design

Status: accepted for WS4 0146 prerequisite

## Decision

Use integer node handles plus maps for the recency list.

The rewrite should represent cache entries with:

- `key_to_node: dict[int, int]`
- `node_key: dict[int, int]`
- `node_value: dict[int, int]`
- `prev: dict[int, int]`
- `next: dict[int, int]`
- fixed sentinel node ids `head = 0` and `tail = 1`
- monotonically increasing `next_id` for allocated entry nodes

## Operations

`get(key)`:

- look up `key_to_node[key]`
- return `-1` only when the key is absent
- move the found node to the front of the recency list
- return `node_value[node]`

`put(key, value)`:

- no-op when capacity is less than or equal to zero
- if the key exists, update `node_value[node]` and move the node to the front
- otherwise, evict `prev[tail]` when the cache is full
- allocate a new node id, write all maps, and link it immediately after `head`

Internal helpers:

- `detach(node)` rewires `prev[node]` and `next[node]`
- `insert_after(node, left)` links a detached node after `left`
- `move_to_front(node)` composes detach plus insert-after-head
- `evict_lru()` removes the node before `tail` from every map

## Rationale

This keeps `get`, `put`, update, and eviction at expected O(1) dictionary/list-rewire cost without shared mutable node aliases, object identity, or interior mutability. Integer node handles avoid cycles in user objects and avoid sentinel key collisions.

The design also handles real cached value `-1`: the cache distinguishes absence through `key_to_node`, not through a sentinel value stored in `node_value`.

## Rewrite Constraints

- Do not delegate to built-in ordered dictionaries or list scans.
- Do not represent recency with shifting arrays.
- Do not store cyclic object references.
- Keep every missing-map lookup explicit through `int | None` checks.
- The final fixture should include the LeetCode sample and a real `-1` cached value case.
