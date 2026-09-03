use crate::{ReceiverConvention, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverMutationEffect {
    None,
    Growth,
    Removal,
    Reorder,
    ValueMutation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverFactInvalidation {
    None,
    GrowthSensitiveSubscriptPresence,
    SubscriptPresence,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverFactDomain {
    RelevantSequenceFacts,
    NoRelevantSequenceFacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReceiverMutationSummary {
    pub effect: ReceiverMutationEffect,
    pub fact_domain: ReceiverFactDomain,
    pub fact_invalidation: ReceiverFactInvalidation,
}

pub fn receiver_mutation_summary(
    object_ty: &Type,
    method: &str,
    convention: ReceiverConvention,
) -> ReceiverMutationSummary {
    if convention != ReceiverConvention::MutableBorrow {
        return ReceiverMutationSummary {
            effect: ReceiverMutationEffect::None,
            fact_domain: receiver_fact_domain(object_ty, method, ReceiverMutationEffect::None),
            fact_invalidation: ReceiverFactInvalidation::None,
        };
    }

    let effect = match object_ty.resolve_alias() {
        Type::List(_) => match method {
            "append" | "appendleft" | "extend" | "insert" => ReceiverMutationEffect::Growth,
            "clear" | "pop" | "popleft" | "remove" => ReceiverMutationEffect::Removal,
            "reverse" | "sort" => ReceiverMutationEffect::Reorder,
            _ => ReceiverMutationEffect::Removal,
        },
        Type::Dict(_, _) => match method {
            "update" | "setdefault" => ReceiverMutationEffect::ValueMutation,
            "clear" | "pop" => ReceiverMutationEffect::Removal,
            _ => ReceiverMutationEffect::Removal,
        },
        Type::Set(_) => match method {
            "add" | "update" => ReceiverMutationEffect::Growth,
            "remove"
            | "discard"
            | "clear"
            | "intersection_update"
            | "difference_update"
            | "symmetric_difference_update" => ReceiverMutationEffect::Removal,
            _ => ReceiverMutationEffect::Removal,
        },
        Type::PythonBuffer(_) => ReceiverMutationEffect::ValueMutation,
        Type::JoinSet(_, _)
            if matches!(method, "add" | "__sifr_spawn_blocking" | "__sifr_spawn_cpu") =>
        {
            ReceiverMutationEffect::Growth
        }
        Type::Class { .. } | Type::Protocol { .. } => ReceiverMutationEffect::Removal,
        _ => ReceiverMutationEffect::Removal,
    };
    let fact_domain = receiver_fact_domain(object_ty, method, effect);
    let fact_invalidation = match (fact_domain, effect) {
        (ReceiverFactDomain::NoRelevantSequenceFacts, _) => ReceiverFactInvalidation::None,
        (_, ReceiverMutationEffect::None) => ReceiverFactInvalidation::None,
        (_, ReceiverMutationEffect::Removal) => ReceiverFactInvalidation::All,
        (_, ReceiverMutationEffect::Reorder) => ReceiverFactInvalidation::SubscriptPresence,
        (_, ReceiverMutationEffect::Growth)
            if matches!(object_ty.resolve_alias(), Type::List(_))
                && matches!(method, "insert" | "appendleft") =>
        {
            ReceiverFactInvalidation::SubscriptPresence
        }
        (_, ReceiverMutationEffect::Growth)
            if matches!(object_ty.resolve_alias(), Type::List(_))
                && matches!(method, "append" | "extend") =>
        {
            ReceiverFactInvalidation::GrowthSensitiveSubscriptPresence
        }
        (_, ReceiverMutationEffect::ValueMutation)
            if matches!(object_ty.resolve_alias(), Type::Dict(_, _)) && method == "update" =>
        {
            ReceiverFactInvalidation::SubscriptPresence
        }
        (_, ReceiverMutationEffect::Growth | ReceiverMutationEffect::ValueMutation) => {
            ReceiverFactInvalidation::None
        }
    };
    ReceiverMutationSummary {
        effect,
        fact_domain,
        fact_invalidation,
    }
}

fn receiver_fact_domain(
    object_ty: &Type,
    method: &str,
    effect: ReceiverMutationEffect,
) -> ReceiverFactDomain {
    match (object_ty.resolve_alias(), method, effect) {
        // These exact operations mutate runtime resources that cannot be the
        // owner of a list/dict/set sequence fact. Keep the operation list
        // explicit: a future shrinking operation must be classified instead
        // of inheriting preservation from its receiver type.
        (Type::PythonBuffer(_), "write", ReceiverMutationEffect::ValueMutation)
        | (
            Type::JoinSet(_, _),
            "add" | "__sifr_spawn_blocking" | "__sifr_spawn_cpu",
            ReceiverMutationEffect::Growth,
        ) => ReceiverFactDomain::NoRelevantSequenceFacts,
        _ => ReceiverFactDomain::RelevantSequenceFacts,
    }
}
