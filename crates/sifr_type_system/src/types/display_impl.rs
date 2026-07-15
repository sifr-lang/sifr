/// Capitalize the first letter of a string.
use super::Type;

pub(super) fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FixedIntType, FunctionType, IterationCapability, OwnershipKind, ParamConvention};

    #[test]
    fn test_ownership_primitives_are_copy() {
        assert_eq!(Type::Int.ownership(), OwnershipKind::Copy);
        assert_eq!(
            Type::FixedInt(FixedIntType::U8).ownership(),
            OwnershipKind::Copy
        );
        assert_eq!(Type::Float.ownership(), OwnershipKind::Copy);
        assert_eq!(Type::Bool.ownership(), OwnershipKind::Copy);
        assert_eq!(Type::None.ownership(), OwnershipKind::Copy);
    }

    #[test]
    fn test_ownership_str_is_move() {
        assert_eq!(Type::Str.ownership(), OwnershipKind::Move);
    }

    #[test]
    fn test_rust_type_mapping() {
        assert_eq!(Type::Int.rust_type(), "i64");
        assert_eq!(Type::FixedInt(FixedIntType::I8).rust_type(), "i8");
        assert_eq!(Type::FixedInt(FixedIntType::I16).rust_type(), "i16");
        assert_eq!(Type::FixedInt(FixedIntType::I32).rust_type(), "i32");
        assert_eq!(Type::FixedInt(FixedIntType::I64).rust_type(), "i64");
        assert_eq!(Type::FixedInt(FixedIntType::U8).rust_type(), "u8");
        assert_eq!(Type::FixedInt(FixedIntType::U16).rust_type(), "u16");
        assert_eq!(Type::FixedInt(FixedIntType::U32).rust_type(), "u32");
        assert_eq!(Type::FixedInt(FixedIntType::U64).rust_type(), "u64");
        assert_eq!(Type::FixedInt(FixedIntType::ISize).rust_type(), "isize");
        assert_eq!(Type::FixedInt(FixedIntType::USize).rust_type(), "usize");
        assert_eq!(Type::Float.rust_type(), "f64");
        assert_eq!(Type::Bool.rust_type(), "bool");
        assert_eq!(Type::Str.rust_type(), "String");
        assert_eq!(Type::None.rust_type(), "()");
    }

    #[test]
    fn test_fixed_width_type_names_and_union_variants() {
        let fixed = Type::FixedInt(FixedIntType::U32);
        assert_eq!(fixed.display_name(), "uint32");
        assert_eq!(fixed.union_variant_name(), "Uint32");
    }

    #[test]
    fn test_fixed_width_current_int_builtin_widening_policy() {
        for fixed in [
            FixedIntType::I8,
            FixedIntType::I16,
            FixedIntType::I32,
            FixedIntType::U8,
            FixedIntType::U16,
            FixedIntType::U32,
        ] {
            assert!(fixed.supports_current_int_builtin_widening());
        }

        for fixed in [
            FixedIntType::I64,
            FixedIntType::U64,
            FixedIntType::ISize,
            FixedIntType::USize,
        ] {
            assert!(!fixed.supports_current_int_builtin_widening());
        }
    }

    #[test]
    fn test_fixed_width_current_scalar_promotion_policy() {
        for fixed in [
            FixedIntType::I8,
            FixedIntType::I16,
            FixedIntType::I32,
            FixedIntType::I64,
            FixedIntType::U8,
            FixedIntType::U16,
            FixedIntType::U32,
            FixedIntType::ISize,
        ] {
            assert!(fixed.supports_current_scalar_promotion_to_int());
        }

        for fixed in [FixedIntType::U64, FixedIntType::USize] {
            assert!(!fixed.supports_current_scalar_promotion_to_int());
        }
    }

    #[test]
    fn test_assignability() {
        assert!(Type::Int.is_assignable_to(&Type::Int));
        assert!(!Type::Int.is_assignable_to(&Type::Str));
        assert!(Type::Int.is_assignable_to(&Type::Float));
        assert!(Type::LiteralInt(42).is_assignable_to(&Type::Float));
        assert!(Type::Any.is_assignable_to(&Type::Int));
        assert!(Type::Int.is_assignable_to(&Type::Any));
        assert!(Type::Never.is_assignable_to(&Type::Int));
    }

    #[test]
    fn test_decimal_assignability_rules() {
        assert!(Type::Decimal.is_assignable_to(&Type::Decimal));
        assert!(Type::BigDecimal.is_assignable_to(&Type::BigDecimal));
        assert!(!Type::Decimal.is_assignable_to(&Type::BigDecimal));
        assert!(!Type::BigDecimal.is_assignable_to(&Type::Decimal));
    }

    #[test]
    fn test_typevar_assignability_is_strict() {
        assert!(Type::TypeVar("T".to_string()).is_assignable_to(&Type::TypeVar("T".to_string())));
        assert!(!Type::TypeVar("T".to_string()).is_assignable_to(&Type::TypeVar("U".to_string())));
        assert!(!Type::TypeVar("T".to_string()).is_assignable_to(&Type::Int));
        assert!(!Type::Int.is_assignable_to(&Type::TypeVar("T".to_string())));
    }

    #[test]
    fn test_list_type() {
        let list_int = Type::List(Box::new(Type::Int));
        assert_eq!(list_int.ownership(), OwnershipKind::Move);
        assert_eq!(list_int.display_name(), "list[int]");
        assert_eq!(list_int.rust_type(), "Vec<i64>");
        assert_eq!(list_int.iterable_element_type(), Some(Type::Int));
    }

    #[test]
    fn test_iterator_and_iterable_type_rules() {
        let iter_int = Type::Iterator(Box::new(Type::Int));
        let iterable_int = Type::Iterable(Box::new(Type::Int));
        let list_int = Type::List(Box::new(Type::Int));

        assert_eq!(iter_int.display_name(), "Iterator[int]");
        assert_eq!(iterable_int.display_name(), "Iterable[int]");
        assert_eq!(iter_int.iterable_element_type(), Some(Type::Int));
        assert_eq!(iterable_int.iterable_element_type(), Some(Type::Int));
        assert!(iter_int.is_assignable_to(&iterable_int));
        assert!(list_int.is_assignable_to(&iterable_int));
    }

    #[test]
    fn test_bytes_iterable_and_index_rules_uses_uint8() {
        let uint8 = Type::FixedInt(FixedIntType::U8);

        assert_eq!(Type::Bytes.iterable_element_type(), Some(uint8.clone()));
        assert_eq!(
            Type::Bytes
                .iteration_metadata()
                .map(|metadata| metadata.element_type),
            Some(uint8.clone())
        );
        assert_eq!(
            Type::Bytes.index_result_type(&Type::Int),
            Some(Type::Union(vec![uint8.clone(), Type::None]))
        );
        assert!(Type::Bytes.is_assignable_to(&Type::Iterable(Box::new(uint8))));
        assert!(!Type::Bytes.is_assignable_to(&Type::Iterable(Box::new(Type::Int))));
    }

    #[test]
    fn test_reversible_alias_rules() {
        let reversible_int = Type::reversible(Type::Int);
        let iterable_int = Type::Iterable(Box::new(Type::Int));
        let list_int = Type::List(Box::new(Type::Int));
        let set_int = Type::Set(Box::new(Type::Int));
        let iterator_int = Type::Iterator(Box::new(Type::Int));

        assert!(list_int.is_assignable_to(&reversible_int));
        assert!(!set_int.is_assignable_to(&reversible_int));
        assert!(!iterator_int.is_assignable_to(&reversible_int));
        assert!(reversible_int.is_assignable_to(&iterable_int));
        assert!(reversible_int.supports_iteration_capability(IterationCapability::DoubleEnded));
    }

    #[test]
    fn test_tuple_iterability_requires_homogeneous_elements() {
        let homogeneous = Type::Tuple(vec![Type::Int, Type::Int, Type::Int]);
        let heterogeneous = Type::Tuple(vec![Type::Int, Type::Str]);

        assert_eq!(homogeneous.iterable_element_type(), Some(Type::Int));
        assert_eq!(heterogeneous.iterable_element_type(), None);
        assert!(homogeneous.is_assignable_to(&Type::Iterable(Box::new(Type::Int))));
        assert!(!heterogeneous.is_assignable_to(&Type::Iterable(Box::new(Type::Any))));
    }

    #[test]
    fn test_class_with_iter_method_is_iterable() {
        let iterable_class = Type::Class {
            name: "Counter".to_string(),
            fields: vec![],
            methods: vec![(
                "__iter__".to_string(),
                FunctionType::new(vec![], Type::Iterator(Box::new(Type::Int))),
            )],
            parent_class: None,
        };

        assert_eq!(iterable_class.iterable_element_type(), Some(Type::Int));
        assert!(iterable_class.is_assignable_to(&Type::Iterable(Box::new(Type::Int))));
    }

    #[test]
    fn test_class_with_next_method_is_iterator_protocol() {
        let self_iter_type = Type::Class {
            name: "CounterIter".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        let iterator_class = Type::Class {
            name: "CounterIter".to_string(),
            fields: vec![],
            methods: vec![
                (
                    "__iter__".to_string(),
                    FunctionType::new(vec![], self_iter_type),
                ),
                (
                    "__next__".to_string(),
                    FunctionType::new(vec![], Type::Union(vec![Type::Int, Type::None])),
                ),
            ],
            parent_class: None,
        };

        assert_eq!(iterator_class.iterator_element_type(), Some(Type::Int));
        assert!(iterator_class.is_assignable_to(&Type::Iterator(Box::new(Type::Int))));
        assert!(iterator_class.is_assignable_to(&Type::Iterable(Box::new(Type::Int))));
    }

    #[test]
    fn test_class_with_reversed_method_is_reversible_iterable() {
        let reversible_class = Type::Class {
            name: "Deck".to_string(),
            fields: vec![],
            methods: vec![
                (
                    "__iter__".to_string(),
                    FunctionType::new(vec![], Type::Iterator(Box::new(Type::Int))),
                ),
                (
                    "__reversed__".to_string(),
                    FunctionType::new(vec![], Type::Iterator(Box::new(Type::Int))),
                ),
            ],
            parent_class: None,
        };

        assert!(reversible_class.is_reversible_iterable());
        assert!(reversible_class.is_assignable_to(&Type::reversible(Type::Int)));
    }

    #[test]
    fn test_dict_type() {
        let dict_str_int = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
        assert_eq!(dict_str_int.ownership(), OwnershipKind::Move);
        assert_eq!(dict_str_int.display_name(), "dict[str, int]");
        assert_eq!(dict_str_int.rust_type(), "HashMap<String, i64>");
    }

    #[test]
    fn test_tuple_type() {
        let tuple = Type::Tuple(vec![Type::Int, Type::Str]);
        assert_eq!(tuple.ownership(), OwnershipKind::Move);
        assert_eq!(tuple.display_name(), "tuple[int, str]");
        assert_eq!(tuple.rust_type(), "(i64, String)");
    }

    #[test]
    fn test_tuple_ownership_all_copy_is_copy() {
        let tuple = Type::Tuple(vec![Type::Int, Type::Float]);
        assert_eq!(tuple.ownership(), OwnershipKind::Copy);
    }

    #[test]
    fn test_tuple_ownership_with_move_is_move() {
        let tuple = Type::Tuple(vec![Type::Int, Type::Str]);
        assert_eq!(tuple.ownership(), OwnershipKind::Move);
    }

    #[test]
    fn test_python_buffer_capabilities_propagate_through_aggregates() {
        let buffer = Type::PythonBuffer(Box::new(Type::FixedInt(FixedIntType::U8)));
        let nested = Type::Class {
            name: "NestedBuffer".to_string(),
            fields: vec![(
                "views".to_string(),
                Type::List(Box::new(Type::Union(vec![Type::None, buffer]))),
            )],
            methods: vec![],
            parent_class: None,
        };

        assert!(nested.contains_affine_resource());
        assert!(!nested.supports_derived_clone());
        assert!(!nested.supports_structural_equality());
    }

    #[test]
    fn test_affine_capability_query_terminates_on_recursive_class_shape() {
        let recursive = Type::Class {
            name: "Node".to_string(),
            fields: vec![(
                "next".to_string(),
                Type::Class {
                    name: "Node".to_string(),
                    fields: vec![],
                    methods: vec![],
                    parent_class: None,
                },
            )],
            methods: vec![],
            parent_class: None,
        };

        assert!(!recursive.contains_affine_resource());
        assert!(recursive.supports_derived_clone());
        assert!(recursive.supports_structural_equality());
    }

    #[test]
    fn test_rust_trait_object_capabilities_are_not_overstated() {
        let callable = Type::Callable(
            vec![Type::Int],
            vec![ParamConvention::own()],
            Box::new(Type::Int),
        );
        let holder = Type::Class {
            name: "Holder".to_string(),
            fields: vec![("callback".to_string(), callable)],
            methods: vec![],
            parent_class: None,
        };

        for ty in [Type::Any, Type::Unknown, holder] {
            assert!(!ty.supports_derived_clone(), "{ty:?}");
            assert!(!ty.supports_structural_equality(), "{ty:?}");
        }
    }

    #[test]
    fn test_hash_and_format_capabilities_match_generated_rust_traits() {
        let callable_class = Type::Class {
            name: "CallbackHolder".to_string(),
            fields: vec![(
                "callback".to_string(),
                Type::Callable(
                    vec![Type::Int],
                    vec![ParamConvention::own()],
                    Box::new(Type::Int),
                ),
            )],
            methods: vec![],
            parent_class: None,
        };
        let comparable_union = Type::Union(vec![Type::Int, Type::Str]);
        let task_result = Type::TaskResult(Box::new(Type::Int), Box::new(Type::Never));
        let join_item_id = Type::Class {
            name: "JoinItemId".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };

        assert!(comparable_union.supports_structural_equality());
        assert!(comparable_union.supports_hash_key());
        assert!(comparable_union.supports_debug_formatting());
        assert!(comparable_union.supports_display_formatting());
        assert!(task_result.supports_debug_formatting());
        assert!(Type::List(Box::new(task_result)).supports_debug_formatting());
        assert!(join_item_id.supports_debug_formatting());
        assert!(join_item_id.supports_display_formatting());
        assert!(!Type::Float.supports_hash_key());
        assert!(!Type::List(Box::new(Type::Int)).supports_hash_key());
        assert!(!Type::Set(Box::new(Type::Float)).supports_structural_equality());
        assert!(!Type::Dict(
            Box::new(Type::List(Box::new(Type::Int))),
            Box::new(Type::Int),
        )
        .supports_structural_equality());
        assert!(!callable_class.supports_debug_formatting());
        assert!(!callable_class.supports_display_formatting());
    }

    #[test]
    fn test_transitive_non_send_ancestry_disables_generated_rust_traits() {
        let local_child = Type::Class {
            name: "LocalChild".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: Some("LocalParent|NonSend".to_string()),
        };

        assert!(!local_child.supports_derived_clone());
        assert!(!local_child.supports_structural_equality());
        assert!(!local_child.supports_hash_key());
        assert!(!local_child.supports_debug_formatting());
    }

    #[test]
    fn test_collection_assignability() {
        let list_int = Type::List(Box::new(Type::Int));
        let list_int2 = Type::List(Box::new(Type::Int));
        let list_str = Type::List(Box::new(Type::Str));
        assert!(list_int.is_assignable_to(&list_int2));
        assert!(!list_int.is_assignable_to(&list_str));

        // Mutable collections are invariant.
        let list_int_or_str = Type::List(Box::new(Type::Union(vec![Type::Int, Type::Str])));
        assert!(!list_int.is_assignable_to(&list_int_or_str));

        let dict_int_int = Type::Dict(Box::new(Type::Int), Box::new(Type::Int));
        let dict_int_union = Type::Dict(
            Box::new(Type::Int),
            Box::new(Type::Union(vec![Type::Int, Type::Str])),
        );
        assert!(!dict_int_int.is_assignable_to(&dict_int_union));

        let object_a = Type::Class {
            name: "Object".to_string(),
            fields: vec![("_handle".to_string(), Type::Int)],
            methods: vec![],
            parent_class: None,
        };
        let object_b = Type::Class {
            name: "Object".to_string(),
            fields: vec![("_token".to_string(), Type::Int)],
            methods: vec![],
            parent_class: None,
        };
        assert!(Type::List(Box::new(object_a.clone()))
            .is_assignable_to(&Type::List(Box::new(object_b))));

        let child = Type::Class {
            name: "ChildObject".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: Some("Object".to_string()),
        };
        assert!(!Type::List(Box::new(child)).is_assignable_to(&Type::List(Box::new(object_a))));
    }

    #[test]
    fn test_class_assignability_supports_transitive_inheritance_chain() {
        let base = Type::Class {
            name: "Base".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        let mid = Type::Class {
            name: "Mid".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: Some("Base".to_string()),
        };
        let leaf = Type::Class {
            name: "Leaf".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: Some("Mid|Base".to_string()),
        };

        assert!(leaf.is_assignable_to(&mid));
        assert!(leaf.is_assignable_to(&base));
    }

    #[test]
    fn test_error_assignability_requires_actual_error_ancestry() {
        let error = Type::Class {
            name: "Error".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        let non_error_child = Type::Class {
            name: "Widget".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: Some("BaseThing".to_string()),
        };
        let real_error_child = Type::Class {
            name: "ValueError".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };

        assert!(!non_error_child.is_assignable_to(&error));
        assert!(real_error_child.is_assignable_to(&error));
    }

    #[test]
    fn test_index_result_type() {
        let list_int = Type::List(Box::new(Type::Int));
        // Safe indexing returns Option[T] = T | None
        assert_eq!(
            list_int.index_result_type(&Type::Int),
            Some(Type::Union(vec![Type::Int, Type::None]))
        );
        assert_eq!(list_int.index_result_type(&Type::Str), None);

        let dict_any_int = Type::Dict(Box::new(Type::Any), Box::new(Type::Int));
        assert_eq!(
            dict_any_int.index_result_type(&Type::Str),
            Some(Type::Union(vec![Type::Int, Type::None]))
        );
    }

    #[test]
    fn test_contains_element_type_range_and_sifr_defaultdict() {
        assert_eq!(Type::Range.contains_element_type(), Some(Type::Int));

        let sifr_defaultdict = Type::Alias {
            name: "__sifr_defaultdict_list".to_string(),
            type_args: Vec::new(),
            body: Box::new(Type::Dict(
                Box::new(Type::Str),
                Box::new(Type::List(Box::new(Type::Int))),
            )),
        };
        assert_eq!(sifr_defaultdict.contains_element_type(), Some(Type::Str));
    }

    // --- Union type tests ---

    #[test]
    fn test_union_display_name() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert_eq!(u.display_name(), "int | str");
    }

    #[test]
    fn test_literal_display_name() {
        assert_eq!(Type::LiteralInt(42).display_name(), "42");
        assert_eq!(
            Type::LiteralStr("GET".to_string()).display_name(),
            "\"GET\""
        );
        assert_eq!(Type::LiteralBool(true).display_name(), "True");
        assert_eq!(Type::LiteralBool(false).display_name(), "False");
    }

    #[test]
    fn test_unknown_display_name() {
        assert_eq!(Type::Unknown.display_name(), "Unknown");
    }

    #[test]
    fn test_literal_assignable_to_base() {
        assert!(Type::LiteralInt(42).is_assignable_to(&Type::Int));
        assert!(Type::LiteralStr("GET".to_string()).is_assignable_to(&Type::Str));
        assert!(Type::LiteralBool(true).is_assignable_to(&Type::Bool));
    }

    #[test]
    fn test_literal_not_assignable_to_wrong_base() {
        assert!(!Type::LiteralInt(42).is_assignable_to(&Type::Str));
        assert!(!Type::LiteralStr("GET".to_string()).is_assignable_to(&Type::Int));
    }

    #[test]
    fn test_assignable_to_union() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert!(Type::Int.is_assignable_to(&u));
        assert!(Type::Str.is_assignable_to(&u));
        assert!(!Type::Bool.is_assignable_to(&u));
    }

    #[test]
    fn test_union_assignable_to_target() {
        // Union is assignable to target only if ALL members are assignable
        let u = Type::Union(vec![Type::Int, Type::Int]);
        assert!(u.is_assignable_to(&Type::Int));

        let u2 = Type::Union(vec![Type::Int, Type::Str]);
        assert!(!u2.is_assignable_to(&Type::Int));
    }

    #[test]
    fn test_anything_assignable_to_unknown() {
        assert!(Type::Int.is_assignable_to(&Type::Unknown));
        assert!(Type::Str.is_assignable_to(&Type::Unknown));
        assert!(Type::Bool.is_assignable_to(&Type::Unknown));
    }

    #[test]
    fn test_union_rust_type_option() {
        let optional_str = Type::Union(vec![Type::None, Type::Str]);
        assert_eq!(optional_str.rust_type(), "Option<String>");
    }

    #[test]
    fn test_union_rust_type_enum() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert_eq!(u.rust_type(), "IntOrStr");
    }

    #[test]
    fn test_union_ownership() {
        // Union with Move member -> Move
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert_eq!(u.ownership(), OwnershipKind::Move);
        // Union with only Copy members -> Copy
        let u2 = Type::Union(vec![Type::Int, Type::Bool]);
        assert_eq!(u2.ownership(), OwnershipKind::Copy);
    }

    #[test]
    fn test_alias_resolves() {
        let alias = Type::Alias {
            name: "UserId".to_string(),
            type_args: Vec::new(),
            body: Box::new(Type::Int),
        };
        assert_eq!(alias.display_name(), "UserId");
        assert_eq!(alias.rust_type(), "i64");
        assert!(alias.is_assignable_to(&Type::Int));
    }

    #[test]
    fn test_literal_is_numeric() {
        assert!(Type::LiteralInt(42).is_numeric());
        assert!(!Type::LiteralStr("x".to_string()).is_numeric());
    }

    #[test]
    fn test_is_union() {
        assert!(Type::Union(vec![Type::Int, Type::Str]).is_union());
        assert!(!Type::Int.is_union());
    }

    #[test]
    fn test_is_literal() {
        assert!(Type::LiteralInt(42).is_literal());
        assert!(Type::LiteralStr("x".to_string()).is_literal());
        assert!(Type::LiteralBool(true).is_literal());
        assert!(!Type::Int.is_literal());
    }

    #[test]
    fn test_never_assignable_to_union() {
        let u = Type::Union(vec![Type::Int, Type::Str]);
        assert!(Type::Never.is_assignable_to(&u));
    }

    #[test]
    fn test_async_callable_is_distinct_from_sync_callable() {
        let conventions = vec![crate::ParamConvention::own()];
        let async_callable =
            Type::AsyncCallable(vec![Type::Int], conventions.clone(), Box::new(Type::Str));
        let sync_callable =
            Type::Callable(vec![Type::Int], conventions.clone(), Box::new(Type::Str));
        let async_function = Type::AsyncFunction(FunctionType::new(
            vec![("value".to_string(), Type::Int)],
            Type::Str,
        ));

        assert_eq!(async_callable.display_name(), "AsyncCallable[[int], str]");
        assert!(async_function.is_assignable_to(&async_callable));
        assert!(!sync_callable.is_assignable_to(&async_callable));
        assert!(!async_callable.is_assignable_to(&sync_callable));
        assert!(async_callable
            .rust_type()
            .contains("AsyncFn(i64) -> String"));
    }

    #[test]
    fn test_async_callable_requires_matching_owned_parameter_conventions() {
        let target = Type::AsyncCallable(
            vec![Type::Str],
            vec![crate::ParamConvention::own()],
            Box::new(Type::Str),
        );
        let borrowed = Type::AsyncFunction(FunctionType::new(
            vec![("value".to_string(), Type::Str)],
            Type::Str,
        ));
        let owned = Type::AsyncFunction(FunctionType {
            params: vec![(
                "value".to_string(),
                Type::Str,
                crate::ParamConvention::own(),
            )],
            return_type: Box::new(Type::Str),
        });

        assert!(!borrowed.is_assignable_to(&target));
        assert!(owned.is_assignable_to(&target));
    }
}
