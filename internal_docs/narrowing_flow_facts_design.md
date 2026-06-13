# Narrowing Flow Facts And Invalidation

Status: accepted for WS1 D0
Phase: `plans/issues/archive/ad-hoc-leetcode-divergence-closure-2026-04-24.md`

## Goal

Optional, index, and dictionary narrowing must remain proof-gated and local. A successful proof may remove `None` from an expression only while every dependency that made the proof true is unchanged.

The base type of safe collection access remains optional:

- `list[T][i] -> T | None`
- `str[i] -> str | None`
- `dict[K, V][k] -> V | None`
- `dict.get(k) -> V | None`

Narrowing is a local fact layered on top of those base types. It must not alter the language-level return type of subscript or lookup operations.

## Fact Kinds

The checker may track these facts:

- Binding non-null fact: a local binding is known not to be `None` after `x is not None`, false-exit `x is None`, or equivalent dominated control flow.
- Sequence length fact: a sequence path has a proven minimum length from truthiness, `len(seq) > n`, false-exit `len(seq) == n`, or equivalent bounds.
- Index in-range fact: an index binding is proven in range for a sequence path, such as `i < len(values)` or `for i in range(len(values))`.
- Subscript-present fact: `seq[i] is not None` proves that the same stable subscript expression can be used as non-optional while dependencies remain unchanged.
- Dict-key fact: `key in d`, `key not in d` false-exit, or `d[key] = value` proves that the same stable key expression is present in that dict while dependencies remain unchanged.

Facts are path-sensitive, not global. They may be saved/restored around branches and loops, but they must never cross a function boundary or escape into nested mutable closure state.

## Dependency Model

Every fact has dependencies:

- Binding non-null facts depend on the binding itself.
- Sequence length and index facts depend on the sequence path and any index binding in the proof.
- Subscript-present facts depend on the sequence path and the index expression.
- Dict-key facts depend on the dict path and the key expression.
- Attribute-path facts such as `self.items` depend on both the full path and any rebinding of a prefix path such as `self`.

Only stable key/index expressions are eligible for subscript or dict facts. Names, literals, tuples of stable tokens, and simple arithmetic over stable tokens are allowed. Calls, comprehensions, and object constructions are not stable proof tokens.

## Invalidation Rules

The checker must clear dependent facts when any dependency changes:

- Rebinding `x` clears `x`'s non-null fact and all facts whose sequence/dict/index/key dependency mentions `x`.
- Rebinding `seq` clears facts about `seq` and nested paths such as `seq.items`.
- Rebinding an index/key binding clears facts that mention that binding in an index/key expression.
- Mutations that may shrink or remove collection contents clear facts for that collection path. Examples: `list.clear`, `list.pop`, `list.remove`, `dict.clear`, `dict.pop`, and destructive set update methods.
- Field assignment to a collection path clears facts for that field path. Rebinding a prefix object clears facts for nested field paths.
- Calls through unknown or user-defined functions must not create new collection facts. Future interprocedural alias analysis may preserve facts only after proving no aliasing mutation.
- Facts do not cross nested function boundaries. Mutable `nonlocal` capture remains unsupported.

Monotonic collection mutations such as `append` may preserve existing index bounds only when the implementation can prove they do not invalidate the specific fact. Until then, invalidation may be conservative.

## Diagnostics Contract

When a proof cannot be maintained, the resulting type error should explain the missing proof instead of implying implicit unwrapping:

- Optional binding access: "value may be None; narrow with `is not None` after the last assignment."
- Sequence index access: "index access returns `T | None`; prove the index is in bounds after the last mutation/rebinding."
- Dict lookup access: "dict lookup returns `V | None`; prove the key exists after the last dict mutation/rebinding."

If the checker has tracked an invalidating operation, diagnostics should include the invalidator category and target, for example "previous index proof for `values[i]` was invalidated by rebinding `values`". The first implementation may use existing type mismatch diagnostics while preserving safety; richer invalidator spans are a follow-up requirement for WS1 diagnostics work.

## Initial Guardrail

WS1 D0 introduces the shared invalidation primitive for existing facts:

- optional binding narrowing is cleared on rebinding,
- sequence/dict guards are cleared when a dependent binding is rebound,
- facts for a collection path are cleared after collection methods that may remove entries.

This closes known unsound D0 holes before adding broader N1/I1/I2/N2-N4 narrowing rules.

