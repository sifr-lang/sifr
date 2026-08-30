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
    normalize_members(&mut members);

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
    if matches!(to_remove.resolve_alias(), Type::None) {
        if let Some(payload) = union.optional_member_type() {
            return payload;
        }
    }
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
            Type::Alias { body, .. } if matches!(body.as_ref(), Type::Union(_)) => {
                flatten_into(out, vec![*body]);
            }
            other => out.push(other),
        }
    }
}

fn normalize_members(types: &mut Vec<Type>) {
    if types.len() < 2 {
        return;
    }
    // Remove common exact duplicates before allocating canonical identity keys.
    let mut exact = Vec::with_capacity(types.len());
    for ty in types.drain(..) {
        if !exact.contains(&ty) {
            exact.push(ty);
        }
    }
    *types = exact;
    if types.len() < 2 {
        return;
    }
    let mut keyed = types
        .drain(..)
        .map(|ty| (type_source_sort_key(&ty), ty))
        .collect::<Vec<_>>();
    keyed.sort_unstable_by(|left, right| left.0.cmp(&right.0));

    let mut normalized = Vec::with_capacity(keyed.len());
    let mut keyed = keyed.into_iter().peekable();
    while let Some((primary_key, ty)) = keyed.next() {
        if keyed
            .peek()
            .is_none_or(|(next_key, _)| next_key != &primary_key)
        {
            normalized.push(ty);
            continue;
        }
        let mut group = vec![ty];
        while keyed
            .peek()
            .is_some_and(|(next_key, _)| next_key == &primary_key)
        {
            if let Some((_, member)) = keyed.next() {
                group.push(member);
            }
        }
        append_normalized_group(&mut normalized, group);
    }
    *types = normalized;
}

fn append_normalized_group(normalized: &mut Vec<Type>, group: Vec<Type>) {
    if group.len() == 1 {
        normalized.extend(group);
        return;
    }

    let mut unique = Vec::with_capacity(group.len());
    for ty in group {
        if !unique.contains(&ty) {
            unique.push(ty);
        }
    }
    if unique.len() == 1 {
        normalized.extend(unique);
        return;
    }

    let mut keyed = unique
        .into_iter()
        .map(|ty| (ty.union_identity_key(), ty))
        .collect::<Vec<_>>();
    keyed.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    let mut deduplicated: Vec<(String, Type)> = Vec::with_capacity(keyed.len());
    for (identity, ty) in keyed {
        if let Some((current_identity, current)) = deduplicated.last_mut() {
            if current_identity == &identity {
                if representative_key(&ty) > representative_key(current) {
                    *current = ty;
                }
                continue;
            }
        }
        deduplicated.push((identity, ty));
    }
    normalized.extend(deduplicated.into_iter().map(|(_, ty)| ty));
}

fn representative_key(ty: &Type) -> (u8, usize, usize, String, String, String) {
    let representation = format!("{ty:?}");
    let (kind, completeness, details, local_name) = match ty {
        Type::Class {
            name,
            fields,
            methods,
            parent_class,
            ..
        } => (
            2,
            (fields.len() + methods.len(), fields.len()),
            nominal_member_details(fields, methods, parent_class.as_deref()),
            name.clone(),
        ),
        Type::Protocol { name, methods, .. } => (
            2,
            (methods.len(), 0),
            nominal_member_details(&[], methods, None),
            name.clone(),
        ),
        Type::Enum { name, variants, .. } => (
            2,
            (variants.len(), 0),
            enum_member_details(variants),
            name.clone(),
        ),
        Type::Newtype { name, .. } => (1, (0, 0), ty.union_identity_key(), name.clone()),
        Type::Alias { name, .. } => (
            1,
            (type_information_score(ty), 0),
            ty.union_identity_key(),
            name.clone(),
        ),
        _ => (
            1,
            (type_information_score(ty), 0),
            ty.union_identity_key(),
            String::new(),
        ),
    };
    (
        kind,
        completeness.0,
        completeness.1,
        details,
        local_name,
        representation,
    )
}

fn type_information_score(ty: &Type) -> usize {
    match ty {
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::Awaitable(inner)
        | Type::PythonBuffer(inner)
        | Type::PythonDlpackTensor(inner)
        | Type::Alias { body: inner, .. }
        | Type::Newtype { inner, .. } => type_information_score(inner),
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::Coroutine(left, right)
        | Type::Task(left, right)
        | Type::TaskResult(left, right)
        | Type::Select2(left, right)
        | Type::BlockingTask(left, right)
        | Type::JoinSet(left, right)
        | Type::AsyncIterator(left, right)
        | Type::AsyncGenerator(left, right) => {
            type_information_score(left) + type_information_score(right)
        }
        Type::Tuple(items)
        | Type::Template(items)
        | Type::Union(items)
        | Type::Intersection(items) => items.iter().map(type_information_score).sum(),
        Type::Function(function) | Type::AsyncFunction(function) => {
            function_information_score(function)
        }
        Type::Class {
            type_args,
            fields,
            methods,
            parent_class,
            ..
        } => {
            fields.len()
                + methods.len()
                + usize::from(parent_class.is_some())
                + type_args.iter().map(type_information_score).sum::<usize>()
                + fields
                    .iter()
                    .map(|(_, field)| type_information_score(field))
                    .sum::<usize>()
                + methods
                    .iter()
                    .map(|(_, method)| function_information_score(method))
                    .sum::<usize>()
        }
        Type::Protocol { methods, .. } => {
            methods.len()
                + methods
                    .iter()
                    .map(|(_, method)| function_information_score(method))
                    .sum::<usize>()
        }
        Type::Callable(parameters, _, result) | Type::AsyncCallable(parameters, _, result) => {
            parameters.iter().map(type_information_score).sum::<usize>()
                + type_information_score(result)
        }
        Type::Enum { variants, .. } => variants.len(),
        Type::None
        | Type::Bool
        | Type::Int
        | Type::FixedInt(_)
        | Type::Float
        | Type::Str
        | Type::Bytes
        | Type::Range
        | Type::Any
        | Type::Never
        | Type::LiteralInt(_)
        | Type::LiteralStr(_)
        | Type::LiteralBool(_)
        | Type::Unknown
        | Type::TypeVar(_)
        | Type::PythonArrow(_)
        | Type::PythonDlpackStream
        | Type::Decimal
        | Type::BigDecimal => 0,
    }
}

fn function_information_score(function: &crate::types::FunctionType) -> usize {
    function
        .params
        .iter()
        .map(|(_, parameter, _)| type_information_score(parameter))
        .sum::<usize>()
        + type_information_score(&function.return_type)
}

fn nominal_member_details(
    fields: &[(String, Type)],
    methods: &[(String, crate::types::FunctionType)],
    parent_class: Option<&str>,
) -> String {
    let mut details = String::new();
    for (name, ty) in fields {
        append_representative_component(&mut details, "field");
        append_representative_component(&mut details, name);
        append_representative_component(&mut details, &ty.union_identity_key());
    }
    for (name, function) in methods {
        append_representative_component(&mut details, "method");
        append_representative_component(&mut details, name);
        append_representative_component(
            &mut details,
            &Type::Function(function.clone()).union_identity_key(),
        );
    }
    if let Some(parent) = parent_class {
        append_representative_component(&mut details, "parent");
        append_representative_component(&mut details, parent);
    }
    details
}

fn enum_member_details(variants: &[(String, Option<i64>)]) -> String {
    let mut details = String::new();
    for (name, value) in variants {
        append_representative_component(&mut details, name);
        append_representative_component(
            &mut details,
            &value.map_or_else(|| "implicit".to_string(), |value| value.to_string()),
        );
    }
    details
}

fn append_representative_component(target: &mut String, value: &str) {
    target.push_str(&value.len().to_string());
    target.push(':');
    target.push_str(value);
}

fn type_source_sort_key(ty: &Type) -> (u8, String) {
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
        Type::Template(_) => (41, String::new()),
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
        Type::PythonBuffer(_) => (29, "buffer".to_string()),
        Type::PythonArrow(kind) => (29, format!("arrow:{}", kind.source_name())),
        Type::PythonDlpackTensor(_) => (29, "dlpack_tensor".to_string()),
        Type::PythonDlpackStream => (29, "dlpack_stream".to_string()),
        Type::Unknown => (29, String::new()),
        Type::Any => (29, String::new()),
        Type::Never => (29, String::new()),
        Type::Union(_) => (30, String::new()),
        Type::Intersection(_) => (31, String::new()),
        Type::Alias { body, .. } => type_source_sort_key(body),
        Type::Class { identity, name, .. } => (31, identity.as_ref().unwrap_or(name).clone()),
        Type::Result(_, _) => (32, String::new()),
        Type::Protocol { identity, name, .. } => (33, identity.as_ref().unwrap_or(name).clone()),
        Type::Newtype { identity, name, .. } => (34, identity.as_ref().unwrap_or(name).clone()),
        Type::TypeVar(name) => (35, name.clone()),
        Type::Callable(..) => (36, String::new()),
        Type::AsyncCallable(..) => (37, String::new()),
        Type::Enum { identity, name, .. } => (38, identity.as_ref().unwrap_or(name).clone()),
        Type::Decimal => (39, String::new()),
        Type::BigDecimal => (40, String::new()),
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
    fn test_python_resource_union_order_is_canonical() {
        let buffer = Type::PythonBuffer(Box::new(Type::Int));
        let dlpack = Type::PythonDlpackTensor(Box::new(Type::Int));

        let forward = make_union(vec![buffer.clone(), dlpack.clone()]);
        let reversed = make_union(vec![dlpack, buffer]);

        assert_eq!(forward, reversed);
        assert!(matches!(forward, Type::Union(members) if members.len() == 2));
    }

    #[test]
    fn python_resource_union_dedup_uses_alias_transparent_element_identity() {
        let byte_alias = Type::Alias {
            name: "Byte".to_string(),
            type_args: Vec::new(),
            body: Box::new(Type::FixedInt(crate::types::FixedIntType::U8)),
        };
        let uint8 = Type::FixedInt(crate::types::FixedIntType::U8);

        for (left, right) in [
            (
                Type::PythonBuffer(Box::new(byte_alias.clone())),
                Type::PythonBuffer(Box::new(uint8.clone())),
            ),
            (
                Type::PythonDlpackTensor(Box::new(byte_alias)),
                Type::PythonDlpackTensor(Box::new(uint8)),
            ),
        ] {
            let forward = make_union(vec![left.clone(), right.clone()]);
            let reverse = make_union(vec![right, left]);
            assert!(!matches!(forward, Type::Union(_)));
            assert_eq!(forward, reverse);
        }
    }

    #[test]
    fn nested_nominal_snapshot_dedup_keeps_the_complete_member_in_each_order() {
        let incomplete = Type::List(Box::new(Type::Class {
            identity: Some("pkg.Item".to_string()),
            type_args: Vec::new(),
            name: "Item".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        }));
        let complete = Type::List(Box::new(Type::Class {
            identity: Some("pkg.Item".to_string()),
            type_args: Vec::new(),
            name: "Item".to_string(),
            fields: vec![("value".to_string(), Type::Int)],
            methods: Vec::new(),
            parent_class: None,
        }));

        assert_eq!(
            make_union(vec![incomplete.clone(), complete.clone()]),
            complete
        );
        assert_eq!(make_union(vec![complete.clone(), incomplete]), complete);
    }

    #[test]
    fn narrowing_preserves_non_union_alias_semantics() {
        let mapping = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
        let alias = Type::Alias {
            name: "__sifr_defaultdict_str_int".to_string(),
            type_args: Vec::new(),
            body: Box::new(mapping.clone()),
        };
        let optional = make_union(vec![alias.clone(), Type::None]);

        assert_eq!(make_union(vec![mapping, alias.clone()]), alias);
        assert_eq!(remove_none_from_union(&optional), alias);
    }

    #[test]
    fn nominal_union_order_uses_declaration_identity_for_equal_source_names() {
        let class = |identity: &str| Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: "Record".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let enumeration = |identity: &str| Type::Enum {
            identity: Some(identity.to_string()),
            name: "Status".to_string(),
            variants: vec![("READY".to_string(), Some(1))],
        };
        let newtype = |identity: &str| Type::Newtype {
            identity: Some(identity.to_string()),
            name: "Token".to_string(),
            inner: Box::new(Type::Int),
        };

        for (left, right) in [
            (class("alpha.Record"), class("zoo.Record")),
            (enumeration("alpha.Status"), enumeration("zoo.Status")),
            (newtype("alpha.Token"), newtype("zoo.Token")),
        ] {
            let expected = make_union(vec![left.clone(), right.clone()]);
            assert_eq!(make_union(vec![right, left]), expected);
        }
    }

    #[test]
    fn nominal_union_order_ignores_local_import_spellings() {
        let class = |name: &str, identity: &str| Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let member_names = |union: Type| {
            let Type::Union(members) = union else {
                panic!("expected union");
            };
            members
                .iter()
                .map(Type::union_variant_name)
                .collect::<Vec<_>>()
        };
        let original = make_union(vec![class("Alpha", "pkg.Alpha"), class("Beta", "pkg.Beta")]);
        let aliased = make_union(vec![class("Zeta", "pkg.Alpha"), class("Beta", "pkg.Beta")]);

        assert_eq!(member_names(original), member_names(aliased));
    }

    #[test]
    fn nominal_snapshot_dedup_keeps_the_same_complete_member_in_each_order() {
        let complete = Type::Class {
            identity: Some("pkg.Record".to_string()),
            type_args: Vec::new(),
            name: "Record".to_string(),
            fields: vec![("value".to_string(), Type::Int)],
            methods: Vec::new(),
            parent_class: None,
        };
        let incomplete = Type::Class {
            identity: Some("pkg.Record".to_string()),
            type_args: Vec::new(),
            name: "Record".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let forward = make_union(vec![incomplete.clone(), complete.clone()]);
        let reverse = make_union(vec![complete.clone(), incomplete]);

        assert_eq!(forward, reverse);
        assert_eq!(forward, complete);
    }

    #[test]
    fn nominal_snapshot_dedup_is_independent_of_local_spelling_order() {
        let class = |name: &str| Type::Class {
            identity: Some("pkg.Record".to_string()),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let protocol = |name: &str| Type::Protocol {
            identity: Some("pkg.Readable".to_string()),
            name: name.to_string(),
            methods: Vec::new(),
        };
        let newtype = |name: &str| Type::Newtype {
            identity: Some("pkg.Token".to_string()),
            name: name.to_string(),
            inner: Box::new(Type::Int),
        };
        let enumeration = |name: &str| Type::Enum {
            identity: Some("pkg.Status".to_string()),
            name: name.to_string(),
            variants: Vec::new(),
        };

        for (left, right) in [
            (class("LocalRecord"), class("Record")),
            (protocol("LocalReadable"), protocol("Readable")),
            (newtype("LocalToken"), newtype("Token")),
            (enumeration("LocalStatus"), enumeration("Status")),
        ] {
            assert_eq!(
                make_union(vec![left.clone(), right.clone()]),
                make_union(vec![right, left])
            );
        }
    }

    #[test]
    fn nominal_snapshot_completeness_counts_fields_and_methods_together() {
        let method = crate::types::FunctionType::new(Vec::new(), Type::None);
        let method_snapshot = Type::Class {
            identity: Some("pkg.Record".to_string()),
            type_args: Vec::new(),
            name: "Record".to_string(),
            fields: vec![("first".to_string(), Type::Int)],
            methods: vec![("render".to_string(), method)],
            parent_class: None,
        };
        let field_snapshot = Type::Class {
            identity: Some("pkg.Record".to_string()),
            type_args: Vec::new(),
            name: "Record".to_string(),
            fields: vec![
                ("first".to_string(), Type::Int),
                ("second".to_string(), Type::Str),
            ],
            methods: Vec::new(),
            parent_class: None,
        };

        assert_eq!(
            make_union(vec![method_snapshot, field_snapshot.clone()]),
            field_snapshot
        );
    }

    #[test]
    fn union_order_is_permutation_independent_for_equal_primary_keys() {
        let error = |identity: &str| Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: "Failure".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let protocol = |identity: &str| Type::Protocol {
            identity: Some(identity.to_string()),
            name: "Readable".to_string(),
            methods: Vec::new(),
        };
        let alias = |body| Type::Alias {
            name: "Value".to_string(),
            type_args: Vec::new(),
            body: Box::new(body),
        };

        for (left, right) in [
            (
                Type::List(Box::new(Type::Int)),
                Type::List(Box::new(Type::Str)),
            ),
            (
                Type::Dict(Box::new(Type::Int), Box::new(Type::Str)),
                Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
            ),
            (
                Type::Set(Box::new(Type::Int)),
                Type::Set(Box::new(Type::Str)),
            ),
            (Type::Tuple(vec![Type::Int]), Type::Range),
            (
                Type::Result(Box::new(Type::Int), Box::new(error("alpha.Failure"))),
                Type::Result(Box::new(Type::Int), Box::new(error("zoo.Failure"))),
            ),
            (protocol("alpha.Readable"), protocol("zoo.Readable")),
            (alias(Type::Int), alias(Type::Str)),
            (
                Type::Callable(
                    vec![Type::Int],
                    vec![crate::ParamConvention::own()],
                    Box::new(Type::Bool),
                ),
                Type::Callable(
                    vec![Type::Str],
                    vec![crate::ParamConvention::own()],
                    Box::new(Type::Bool),
                ),
            ),
        ] {
            assert_eq!(
                make_union(vec![left.clone(), right.clone()]),
                make_union(vec![right, left])
            );
        }
    }
}
