# LeetCode 0706 HashMap Storage Design

Status: accepted for WS4 0706 prerequisite

## Decision

Use separate chaining with explicit buckets.

The rewrite should represent storage as:

- `buckets: list[list[tuple[int, int]]]`
- a fixed prime bucket count for the fixture, for example `769`

Each bucket stores `(key, value)` pairs. The fixture must not delegate to Sifr's built-in `dict`.

## Operations

`hash_index(key)`:

- compute `key % len(buckets)`
- normalize negative results by adding `len(buckets)`

`put(key, value)`:

- find the target bucket
- replace the tuple when the key already exists
- append `(key, value)` when absent
- assign the updated bucket back to `buckets[index]`

`get(key)`:

- scan only the target bucket
- return the stored value when found
- return `-1` when absent

`remove(key)`:

- scan only the target bucket
- rebuild that bucket without the removed key
- assign the rebuilt bucket back to `buckets[index]`

## Rationale

Separate chaining keeps the implementation explicit and reviewable in current Sifr without relying on unsafe aliases, interior mutability, or built-in dictionary delegation. Rebuilding a single bucket on `remove` avoids needing `take_at` for this fixture while still deleting entries rather than writing sentinel values.

The design handles real stored value `-1`: absence is determined by key lookup, not by a value sentinel stored in the bucket.

## Rewrite Constraints

- Do not use built-in `dict` for storage.
- Do not implement `remove` by writing `-1`.
- Do not scan all buckets for `get`, `put`, or `remove`.
- Keep missing index and bucket access explicit through `None` checks.
- The final fixture should include the LeetCode sample and a real stored `-1` value case.
