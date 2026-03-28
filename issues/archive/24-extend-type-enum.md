## Extend Type Enum with Union, Literal, Unknown, and Alias Variants

#### **Current Situation**

- The `Type` enum in `crates/sifr_type_system/src/types.rs` supports only primitives (Int, Float, Bool, Str, None), collections (List, Dict, Tuple, Range), Function, Any, and Never.
- There is no way to express "a value can be one of several types" (union), specific values as types (literals), safe dynamic data (Unknown), or named type aliases.
- `is_assignable_to()` only handles exact type matching and basic Any/Never rules.
- `resolve_type_annotation()` in `infer.rs` only resolves primitive type names.

#### **Desired Situation**

- The `Type` enum supports Union, Intersection, LiteralInt, LiteralStr, LiteralBool, Optional, Alias, and Unknown variants.
- Union types are automatically flattened (no nested unions) and deduplicated.
- Literal types can widen to their base type at mutable assignment.
- `is_assignable_to()` handles union subtyping (T assignable to union if assignable to any member).
- `resolve_type_annotation()` handles `X | Y` union syntax and type alias references.
- Unknown type is accepted everywhere but operations on it are rejected without narrowing.

#### **Suggested Solution**

**New files:**
- `crates/sifr_type_system/src/union.rs`: Union construction (`make_union`), normalization (flatten, dedup, sort), simplification (single-element union -> unwrap), union subtyping, and union display.
- `crates/sifr_type_system/src/literal.rs`: Literal type creation, widening (`LiteralInt(42)` -> `Int`), literal-to-base-type mapping, literal equality.

**Modified files:**
- `crates/sifr_type_system/src/types.rs`: Add new variants to `Type` enum. Update `rust_type()`, `display_name()`, `ownership()`, `is_assignable_to()`, `index_result_type()`, `iterable_element_type()`.
- `crates/sifr_type_system/src/infer.rs`: Update `resolve_type_annotation()` to handle union syntax (`int | str`), literal type annotations, `Unknown`, and type alias lookup.
- `crates/sifr_type_system/src/check.rs`: Update type checking to handle union operands in binary ops, comparisons, etc. Add error for operations on Unknown without narrowing.
- `crates/sifr_type_system/src/lib.rs`: Export new modules.

**Unit tests:**
- Union normalization: `Union(vec![Int, Union(vec![Str, Bool])])` -> `Union(vec![Bool, Int, Str])`
- Single-element union: `Union(vec![Int])` -> `Int`
- Literal widening: `LiteralInt(42).widen()` -> `Int`
- Subtyping: `Int.is_assignable_to(Union(vec![Int, Str]))` -> true
- Unknown: operations on Unknown produce type error
