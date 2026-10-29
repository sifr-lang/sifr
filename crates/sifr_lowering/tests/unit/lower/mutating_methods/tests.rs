use super::*;

#[test]
fn non_class_receiver_registry_covers_iterators_and_task_handles() {
    let async_generator = Type::AsyncGenerator(Box::new(Type::Int), Box::new(Type::Never));
    assert_eq!(
        receiver_convention_for_non_class_method(&async_generator, "aclose"),
        ReceiverConvention::MutableBorrow
    );

    let task = Type::Task(Box::new(Type::Int), Box::new(Type::Never));
    assert_eq!(
        receiver_convention_for_non_class_method(&task, "cancel"),
        ReceiverConvention::SharedBorrow
    );
    assert_eq!(
        receiver_convention_for_non_class_method(&task, "join"),
        ReceiverConvention::Owned
    );

    let join_set = Type::JoinSet(Box::new(Type::Int), Box::new(Type::Never));
    assert_eq!(
        receiver_convention_for_non_class_method(&join_set, "add"),
        ReceiverConvention::MutableBorrow
    );
    assert_eq!(
        receiver_convention_for_non_class_method(&join_set, "__sifr_join_all"),
        ReceiverConvention::Owned
    );
}

#[test]
fn receiver_effect_registry_distinguishes_collection_mutations() {
    let list = Type::List(Box::new(Type::Int));
    let dict = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
    let set = Type::Set(Box::new(Type::Int));
    let effect = |ty: &Type, method: &str, convention| {
        receiver_mutation_summary(ty, method, convention).effect
    };

    for method in ["append", "appendleft", "extend", "insert"] {
        assert_eq!(
            effect(&list, method, ReceiverConvention::MutableBorrow),
            ReceiverMutationEffect::Growth
        );
    }
    for method in ["clear", "pop", "popleft", "remove"] {
        assert_eq!(
            effect(&list, method, ReceiverConvention::MutableBorrow),
            ReceiverMutationEffect::Removal
        );
    }
    for method in ["reverse", "sort"] {
        assert_eq!(
            effect(&list, method, ReceiverConvention::MutableBorrow),
            ReceiverMutationEffect::Reorder
        );
    }
    for method in ["update", "setdefault"] {
        assert_eq!(
            effect(&dict, method, ReceiverConvention::MutableBorrow),
            ReceiverMutationEffect::ValueMutation
        );
    }
    for method in ["clear", "pop"] {
        assert_eq!(
            effect(&dict, method, ReceiverConvention::MutableBorrow),
            ReceiverMutationEffect::Removal
        );
    }
    for method in ["add", "update"] {
        assert_eq!(
            effect(&set, method, ReceiverConvention::MutableBorrow),
            ReceiverMutationEffect::Growth
        );
    }
    for method in [
        "pop",
        "remove",
        "discard",
        "clear",
        "intersection_update",
        "difference_update",
        "symmetric_difference_update",
    ] {
        assert_eq!(
            effect(&set, method, ReceiverConvention::MutableBorrow),
            ReceiverMutationEffect::Removal
        );
    }
    assert_eq!(
        effect(&list, "append", ReceiverConvention::SharedBorrow),
        ReceiverMutationEffect::None
    );
    assert_eq!(
        effect(&list, "future_mutator", ReceiverConvention::MutableBorrow),
        ReceiverMutationEffect::Removal
    );
}

#[test]
fn set_pop_uses_mutable_receiver_authority_for_direct_and_alias_types() {
    let set = Type::Set(Box::new(Type::Int));
    let alias = Type::alias("SetAlias", set.clone());
    for ty in [set, alias] {
        assert!(is_collection_mutating_method(&ty, "pop"));
        assert_eq!(
            receiver_convention_for_non_class_method(&ty, "pop"),
            ReceiverConvention::MutableBorrow
        );
        let summary = receiver_mutation_summary(&ty, "pop", ReceiverConvention::MutableBorrow);
        assert_eq!(summary.effect, ReceiverMutationEffect::Removal);
        assert_eq!(summary.fact_invalidation, ReceiverFactInvalidation::All);
    }
}

#[test]
fn receiver_summary_invalidates_only_falsified_sequence_facts() {
    let optional_int = Type::Union(vec![Type::Int, Type::None]);
    let list = Type::List(Box::new(optional_int.clone()));
    let dict = Type::Dict(Box::new(Type::Str), Box::new(optional_int));
    let summary = |ty: &Type, method: &str| {
        receiver_mutation_summary(ty, method, ReceiverConvention::MutableBorrow)
    };

    assert_eq!(
        summary(&list, "append").fact_invalidation,
        ReceiverFactInvalidation::GrowthSensitiveSubscriptPresence
    );
    for method in ["insert", "appendleft", "reverse", "sort"] {
        assert_eq!(
            summary(&list, method).fact_invalidation,
            ReceiverFactInvalidation::SubscriptPresence
        );
    }
    assert_eq!(
        summary(&dict, "setdefault").fact_invalidation,
        ReceiverFactInvalidation::None
    );
    assert_eq!(
        summary(&dict, "update").fact_invalidation,
        ReceiverFactInvalidation::SubscriptPresence
    );
    assert_eq!(
        summary(&list, "pop").fact_invalidation,
        ReceiverFactInvalidation::All
    );

    let buffer = Type::PythonBuffer(Box::new(Type::FixedInt(sifr_type_system::FixedIntType::U8)));
    assert_eq!(
        summary(&buffer, "write").fact_domain,
        ReceiverFactDomain::NoRelevantSequenceFacts
    );
    assert_eq!(
        summary(&buffer, "write").fact_invalidation,
        ReceiverFactInvalidation::None
    );
    let join_set = Type::JoinSet(Box::new(Type::Int), Box::new(Type::Never));
    for method in [
        "add",
        "__sifr_add_task",
        "__sifr_add_blocking_task",
        "__sifr_spawn_blocking",
        "__sifr_spawn_cpu",
    ] {
        let summary = summary(&join_set, method);
        assert_eq!(summary.effect, ReceiverMutationEffect::Growth);
        assert_eq!(
            summary.fact_domain,
            ReceiverFactDomain::NoRelevantSequenceFacts
        );
        assert_eq!(summary.fact_invalidation, ReceiverFactInvalidation::None);
    }
    let future = summary(&join_set, "future_mutator");
    assert_eq!(future.effect, ReceiverMutationEffect::Removal);
    assert_eq!(
        future.fact_domain,
        ReceiverFactDomain::RelevantSequenceFacts
    );
    assert_eq!(future.fact_invalidation, ReceiverFactInvalidation::All);
    assert_eq!(
        summary(&list, "append").fact_domain,
        ReceiverFactDomain::RelevantSequenceFacts
    );
}
