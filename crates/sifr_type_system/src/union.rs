//! Union type construction, normalization, and operations.
//!
//! Union types represent "one of several types" (e.g., `int | str`).
//! They are normalized: flattened (no nested unions), deduplicated,
//! and sorted for consistent comparison.

use crate::types::Type;

/// Construct a union type from a list of member types.
///
/// Applies normalization:
/// 1. Flatten nested unions
/// 2. Deduplicate
/// 3. Sort for consistent ordering
/// 4. If only one type remains, return it directly (no single-element union)
/// 5. If empty, return Never
pub fn make_union(types: Vec<Type>) -> Type {
    let mut members = Vec::new();
    flatten_into(&mut members, types);
    deduplicate(&mut members);
    sort_members(&mut members);

    match members.len() {
        0 => Type::Never,
        1 => {
            let mut iter = members.into_iter();
            if let Some(member) = iter.next() {
                member
            } else {
                unreachable!("single-element union arm must contain one member")
            }
        }
        _ => Type::Union(members),
    }
}

/// Check if `ty` is a member of the union (or equal to a non-union type).
pub fn union_contains(union: &Type, ty: &Type) -> bool {
    match union {
        Type::Union(members) => members.iter().any(|m| m == ty || member_contains(m, ty)),
        other => other == ty,
    }
}

/// Remove a type from a union, returning the remaining type.
///
/// Used for narrowing: if `x: int | str` and we know `x` is `int`,
/// the else branch has `x: str`.
pub fn subtract_from_union(union: &Type, to_remove: &Type) -> Type {
    match union {
        // Unknown minus a specific type is still Unknown (we can't enumerate what's left)
        Type::Unknown => Type::Unknown,
        Type::Union(members) => {
            let remaining: Vec<Type> = members
                .iter()
                .filter(|m| !should_subtract(m, to_remove))
                .cloned()
                .collect();
            make_union(remaining)
        }
        other => {
            if should_subtract(other, to_remove) {
                Type::Never
            } else {
                other.clone()
            }
        }
    }
}

/// Determine if type `a` should be removed when subtracting `to_remove`.
/// Unlike `types_overlap`, this does NOT remove a base type when subtracting a literal.
/// e.g., subtracting `LiteralStr`("+") from str should NOT remove str,
/// because str is broader than any single literal.
fn should_subtract(a: &Type, to_remove: &Type) -> bool {
    if a == to_remove {
        return true;
    }
    match (a, to_remove) {
        // Subtracting a literal from its base type: DON'T subtract
        // str - LiteralStr("x") = str (str has infinitely many values)
        // int - LiteralInt(42) = int
        // bool - LiteralBool(true) = bool (well, bool is finite, but keep it simple)
        (Type::Str, Type::LiteralStr(_)) => false,
        (Type::Int, Type::LiteralInt(_)) => false,
        (Type::Bool, Type::LiteralBool(_)) => false,
        // Subtracting a base type from a literal: DO subtract (literal is a subset)
        (Type::LiteralStr(_), Type::Str) => true,
        (Type::LiteralInt(_), Type::Int) => true,
        (Type::LiteralBool(_), Type::Bool) => true,
        // Any overlaps with everything
        (Type::Any, _) | (_, Type::Any) => true,
        // Union: overlap if any member overlaps
        (Type::Union(members), other) | (other, Type::Union(members)) => {
            members.iter().any(|m| should_subtract(m, other))
        }
        _ => false,
    }
}

/// Filter a union to only include types that overlap with `target`.
///
/// Used for narrowing: if `x: int | str | bool` and `isinstance(x, int)`,
/// the then-branch has `x: int`.
pub fn intersect_with_union(union: &Type, target: &Type) -> Type {
    match union {
        // Unknown can be narrowed to anything via isinstance
        Type::Unknown => target.clone(),
        Type::Union(members) => {
            let matching: Vec<Type> = members
                .iter()
                .filter(|m| types_overlap(m, target))
                .cloned()
                .collect();
            make_union(matching)
        }
        other => {
            if types_overlap(other, target) {
                other.clone()
            } else {
                Type::Never
            }
        }
    }
}

/// Check if a type is assignable to a union (i.e., assignable to at least one member).
pub fn is_assignable_to_union(ty: &Type, union_members: &[Type]) -> bool {
    union_members.iter().any(|m| ty.is_assignable_to(m))
}

/// Check if a union type is assignable to a target (all members must be assignable).
pub fn union_is_assignable_to(union_members: &[Type], target: &Type) -> bool {
    union_members.iter().all(|m| m.is_assignable_to(target))
}

/// Remove `None` from a union, returning the remaining type.
/// Used for `is not None` narrowing.
pub fn remove_none_from_union(union: &Type) -> Type {
    subtract_from_union(union, &Type::None)
}

/// Check if a union contains None.
pub fn union_contains_none(ty: &Type) -> bool {
    union_contains(ty, &Type::None)
}

// --- Internal helpers ---

/// Flatten nested unions into a flat list.
fn flatten_into(out: &mut Vec<Type>, types: Vec<Type>) {
    for ty in types {
        match ty {
            Type::Union(inner) => flatten_into(out, inner),
            other => out.push(other),
        }
    }
}

/// Remove duplicate types.
fn deduplicate(types: &mut Vec<Type>) {
    let mut seen = Vec::new();
    types.retain(|ty| {
        if seen.contains(ty) {
            false
        } else {
            seen.push(ty.clone());
            true
        }
    });
}

/// Sort types for consistent ordering.
/// Order: None, Bool, Int, fixed-width ints, Float, Str, `LiteralBool`, `LiteralInt`, `LiteralStr`,
///        List, Dict, Tuple, Range, Iterable, Iterator, Function, Unknown, Any, Never, Union, Intersection, Alias
fn sort_members(types: &mut [Type]) {
    types.sort_by_key(type_sort_key);
}

fn type_sort_key(ty: &Type) -> (u8, String) {
    match ty {
        Type::None => (0, String::new()),
        Type::Bool => (1, String::new()),
        Type::Int => (2, String::new()),
        Type::FixedInt(fixed) => (3, fixed.source_name().to_string()),
        Type::Float => (4, String::new()),
        Type::Str => (5, String::new()),
        Type::Bytes => (6, String::new()),
        Type::LiteralBool(v) => (7, format!("{v}")),
        Type::LiteralInt(v) => (8, format!("{v}")),
        Type::LiteralStr(v) => (9, v.clone()),
        Type::List(_) => (10, String::new()),
        Type::Dict(_, _) => (11, String::new()),
        Type::Set(_) => (12, String::new()),
        Type::Tuple(_) => (13, String::new()),
        Type::Range => (13, String::new()),
        Type::Iterable(_) => (14, String::new()),
        Type::Iterator(_) => (15, String::new()),
        Type::Function(_) => (16, String::new()),
        Type::AsyncFunction(_) => (17, String::new()),
        Type::Coroutine(_, _) => (18, String::new()),
        Type::Task(_, _) => (19, String::new()),
        Type::TaskResult(_, _) => (20, String::new()),
        Type::Failure(_) => (21, String::new()),
        Type::TimeoutResult(_) => (22, String::new()),
        Type::Select2(_, _) => (23, String::new()),
        Type::BlockingTask(_, _) => (24, String::new()),
        Type::JoinSet(_, _) => (25, String::new()),
        Type::Awaitable(_) => (26, String::new()),
        Type::AsyncIterator(_, _) => (27, String::new()),
        Type::AsyncGenerator(_, _) => (28, String::new()),
        Type::Unknown => (29, String::new()),
        Type::Any => (29, String::new()),
        Type::Never => (29, String::new()),
        Type::Union(_) => (30, String::new()),
        Type::Intersection(_) => (31, String::new()),
        Type::Alias {
            name, type_args, ..
        } => (
            31,
            if type_args.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}[{}]",
                    name,
                    type_args
                        .iter()
                        .map(Type::display_name)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
        ),
        Type::Class { name, .. } => (31, name.clone()),
        Type::Result(_, _) => (32, String::new()),
        Type::Protocol { name, .. } => (33, name.clone()),
        Type::Newtype { name, .. } => (34, name.clone()),
        Type::TypeVar(name) => (35, name.clone()),
        Type::Callable(..) => (36, String::new()),
        Type::AsyncCallable(..) => (37, String::new()),
        Type::Enum { name, .. } => (38, name.clone()),
        Type::BigInt => (39, String::new()),
        Type::Decimal => (40, String::new()),
        Type::BigDecimal => (41, String::new()),
    }
}

/// Check if a member type contains a target type (for literal-to-base matching).
fn member_contains(member: &Type, target: &Type) -> bool {
    matches!(
        (member, target),
        (Type::Int, Type::LiteralInt(_))
            | (Type::Str, Type::LiteralStr(_))
            | (Type::Bool, Type::LiteralBool(_))
    )
}

/// Check if two types overlap (share possible values).
fn types_overlap(a: &Type, b: &Type) -> bool {
    if a == b {
        return true;
    }
    match (a, b) {
        // Literal types overlap with their base types
        (Type::LiteralInt(_), Type::Int) | (Type::Int, Type::LiteralInt(_)) => true,
        (Type::LiteralStr(_), Type::Str) | (Type::Str, Type::LiteralStr(_)) => true,
        (Type::LiteralBool(_), Type::Bool) | (Type::Bool, Type::LiteralBool(_)) => true,
        // Any overlaps with everything
        (Type::Any, _) | (_, Type::Any) => true,
        // Union: overlap if any member overlaps
        (Type::Union(members), other) | (other, Type::Union(members)) => {
            members.iter().any(|m| types_overlap(m, other))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_union_basic() {
        let u = make_union(vec![Type::Int, Type::Str]);
        assert_eq!(u, Type::Union(vec![Type::Int, Type::Str]));
    }

    #[test]
    fn test_make_union_single_element() {
        let u = make_union(vec![Type::Int]);
        assert_eq!(u, Type::Int);
    }

    #[test]
    fn test_make_union_empty() {
        let u = make_union(vec![]);
        assert_eq!(u, Type::Never);
    }

    #[test]
    fn test_make_union_dedup() {
        let u = make_union(vec![Type::Int, Type::Str, Type::Int]);
        assert_eq!(u, Type::Union(vec![Type::Int, Type::Str]));
    }

    #[test]
    fn test_make_union_flatten() {
        let inner = Type::Union(vec![Type::Str, Type::Bool]);
        let u = make_union(vec![Type::Int, inner]);
        assert_eq!(u, Type::Union(vec![Type::Bool, Type::Int, Type::Str]));
    }

    #[test]
    fn test_make_union_sorted() {
        let u = make_union(vec![Type::Str, Type::Int, Type::None]);
        assert_eq!(u, Type::Union(vec![Type::None, Type::Int, Type::Str]));
    }

    #[test]
    fn test_subtract_from_union() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        let result = subtract_from_union(&u, &Type::Int);
        assert_eq!(result, Type::Str);
    }

    #[test]
    fn test_subtract_none_from_optional() {
        let u = Type::Union(vec![Type::None, Type::Str]);
        let result = remove_none_from_union(&u);
        assert_eq!(result, Type::Str);
    }

    #[test]
    fn test_intersect_with_union() {
        let u = Type::Union(vec![Type::Bool, Type::Int, Type::Str]);
        let result = intersect_with_union(&u, &Type::Int);
        assert_eq!(result, Type::Int);
    }

    #[test]
    fn test_union_contains() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert!(union_contains(&u, &Type::Int));
        assert!(union_contains(&u, &Type::Str));
        assert!(!union_contains(&u, &Type::Bool));
    }

    #[test]
    fn test_union_contains_none() {
        let u = Type::Union(vec![Type::None, Type::Str]);
        assert!(union_contains_none(&u));
        let u2 = Type::Union(vec![Type::Int, Type::Str]);
        assert!(!union_contains_none(&u2));
    }

    #[test]
    fn test_is_assignable_to_union() {
        let members = vec![Type::Int, Type::Str];
        assert!(is_assignable_to_union(&Type::Int, &members));
        assert!(is_assignable_to_union(&Type::Str, &members));
        assert!(!is_assignable_to_union(&Type::Bool, &members));
    }

    #[test]
    fn test_literal_overlap_with_base() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        let result = intersect_with_union(&u, &Type::LiteralInt(42));
        assert_eq!(result, Type::Int);
    }
}
