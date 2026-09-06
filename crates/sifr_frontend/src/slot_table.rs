use crate::{ShapeNode, StructuralShape, describe_type_with_externals};
use sifr_lowering::{
    AdapterHandlerPlan, CallableIdentity, ExternalDefs, LoweringResult, StaticMethodParam,
    StaticMethodSlot, StaticMethodSlotContext, StaticMethodSlotInputRole,
};
use sifr_type_system::{ReceiverConvention, Type};
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MethodSlotErrorKind {
    List,
    Method,
    Signature,
    Context,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MethodSlotError {
    kind: MethodSlotErrorKind,
    reason: String,
    range: Option<ruff_text_size::TextRange>,
}

impl MethodSlotError {
    fn new(kind: MethodSlotErrorKind, reason: impl Into<String>) -> Self {
        Self {
            kind,
            reason: reason.into(),
            range: None,
        }
    }

    fn at(mut self, range: Option<ruff_text_size::TextRange>) -> Self {
        self.range = range.or(self.range);
        self
    }

    pub(crate) const fn kind(&self) -> MethodSlotErrorKind {
        self.kind
    }

    pub(crate) fn into_reason(self) -> String {
        self.reason
    }

    pub(crate) const fn range(&self) -> Option<ruff_text_size::TextRange> {
        self.range
    }
}

pub(crate) fn resolve_method_slots(
    value: &crate::ConstValue,
    described_shape: &StructuralShape,
    target_type: &Type,
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
) -> Result<(Vec<StaticMethodSlot>, Option<StaticMethodSlotContext>), MethodSlotError> {
    let Some(references) = method_slot_references(value)? else {
        return Ok((Vec::new(), None));
    };
    if references.is_empty() {
        return Ok((Vec::new(), None));
    }
    let mut owner_types = HashMap::new();
    collect_nominal_types(
        target_type,
        module_name,
        &mut owner_types,
        &mut BTreeSet::new(),
    );
    let available = shape_method_references(&described_shape.root);
    let mut slots = Vec::with_capacity(references.len());
    let handler_plans = handler_plans_for_target(target_type, result);
    for reference in references {
        let (owner_identity, method_name, handler) = match reference {
            MethodSlotReference::Legacy(reference) => {
                if !available.contains(&reference) {
                    return Err(MethodSlotError::new(
                        MethodSlotErrorKind::Method,
                        format!(
                            "method slot `{reference}` does not name an annotated method in the concrete structural shape"
                        ),
                    ));
                }
                let (owner, method) = reference.rsplit_once("::").ok_or_else(|| {
                    MethodSlotError::new(
                        MethodSlotErrorKind::List,
                        format!(
                            "method slot `{reference}` must use the exact `module.Type::method` identity"
                        ),
                    )
                })?;
                (owner.to_string(), method.to_string(), None)
            }
            MethodSlotReference::Handler(callable) => {
                let handler = handler_plans
                    .iter()
                    .find(|handler| handler.callable == callable)
                    .ok_or_else(|| {
                        MethodSlotError::new(
                            MethodSlotErrorKind::Method,
                            format!(
                                "selected handler '{}::{}' was not produced by a method descriptor on the adapted class",
                                callable.owner.as_deref().unwrap_or(&callable.module),
                                callable.symbol
                            ),
                        )
                    })?;
                let owner = callable.owner.clone().ok_or_else(|| {
                    MethodSlotError::new(
                        MethodSlotErrorKind::Method,
                        "selected handler must name a user-authored method",
                    )
                })?;
                (owner, callable.symbol.clone(), Some(handler))
            }
        };
        let owner_type = owner_types
            .get(&owner_identity)
            .cloned()
            .or_else(|| {
                owner_type_for_identity(
                    &owner_identity,
                    target_type,
                    module_name,
                    result,
                    external_defs,
                )
            })
            .ok_or_else(|| {
                MethodSlotError::new(
                    MethodSlotErrorKind::Method,
                    format!(
                        "method slot owner `{owner_identity}` is not reachable from the concrete type"
                    ),
                )
            })?;
        let expected = handler.map(|handler| &handler.callable);
        let mut slot = resolve_method_slot(
            &owner_identity,
            &method_name,
            &owner_type,
            module_name,
            result,
            external_defs,
            expected,
        )
        .map_err(|problem| problem.at(handler.map(|handler| handler.descriptor_range)))?;
        if let Some(handler) = handler {
            slot.descriptor_type = Some(handler.descriptor_type.clone());
            slot.descriptor_value = Some(handler.descriptor_value.clone());
            slot.descriptor_origin = Some(handler.descriptor_origin);
            slot.descriptor_range = Some(handler.descriptor_range);
            slot.declaration_order = Some(handler.declaration_order);
        }
        slots.push(slot);
    }
    let mut context_contract: Option<(&Type, bool)> = None;
    for slot in &slots {
        let Some(context) = slot.context_type.as_ref() else {
            continue;
        };
        match context_contract {
            None => context_contract = Some((context, slot.context_mutable)),
            Some((expected, mutable)) if expected == context && mutable == slot.context_mutable => {
            }
            Some(_) => {
                return Err(MethodSlotError::new(
                    MethodSlotErrorKind::Context,
                    "all method slots in one static program must use one context type and borrow mode",
                )
                .at(slot.descriptor_range));
            }
        }
    }
    let context = match context_contract {
        None => StaticMethodSlotContext::None,
        Some((context, mutable)) => {
            if !structural_slot_type_supported(context, module_name, result, external_defs) {
                return Err(MethodSlotError::new(
                    MethodSlotErrorKind::Context,
                    "method slot context must be a structural type",
                ));
            }
            if mutable {
                StaticMethodSlotContext::Mutable(context.clone())
            } else {
                StaticMethodSlotContext::Shared(context.clone())
            }
        }
    };
    Ok((slots, Some(context)))
}

enum MethodSlotReference {
    Legacy(String),
    Handler(CallableIdentity),
}

fn method_slot_references(
    value: &crate::ConstValue,
) -> Result<Option<Vec<MethodSlotReference>>, MethodSlotError> {
    let crate::ConstValue::Record(fields) = value else {
        return Ok(None);
    };
    let Some(value) = fields.get("sifr_method_slots") else {
        return Ok(None);
    };
    let crate::ConstValue::List(values) = value else {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::List,
            "reserved `sifr_method_slots` must be a list of exact method identities",
        ));
    };
    let mut seen = BTreeSet::new();
    let mut references = Vec::with_capacity(values.len());
    for value in values {
        let reference = match value {
            crate::ConstValue::String(reference) => MethodSlotReference::Legacy(reference.clone()),
            crate::ConstValue::CallableIdentity(callable) => {
                MethodSlotReference::Handler(callable.clone())
            }
            _ => {
                return Err(MethodSlotError::new(
                    MethodSlotErrorKind::List,
                    "reserved `sifr_method_slots` must contain only exact method identities",
                ));
            }
        };
        let key = match &reference {
            MethodSlotReference::Legacy(reference) => reference.clone(),
            MethodSlotReference::Handler(callable) => format!(
                "{}::{}:{}",
                callable.owner.as_deref().unwrap_or(&callable.module),
                callable.symbol,
                callable.signature
            ),
        };
        if !seen.insert(key.clone()) {
            return Err(MethodSlotError::new(
                MethodSlotErrorKind::List,
                format!("method slot `{key}` is duplicated"),
            ));
        }
        references.push(reference);
    }
    Ok(Some(references))
}

fn shape_method_references(root: &ShapeNode) -> BTreeSet<String> {
    fn collect(
        node: &ShapeNode,
        references: &mut BTreeSet<String>,
        visiting: &mut BTreeSet<String>,
    ) {
        match node {
            ShapeNode::Nominal {
                identity,
                type_arguments,
                fields,
                methods,
                ..
            } => {
                if !visiting.insert(identity.clone()) {
                    return;
                }
                references.extend(
                    methods
                        .iter()
                        .map(|method| format!("{identity}::{}", method.name)),
                );
                for argument in type_arguments {
                    collect(argument, references, visiting);
                }
                for field in fields {
                    collect(&field.declared_type, references, visiting);
                }
                visiting.remove(identity);
            }
            ShapeNode::List(value)
            | ShapeNode::Set(value)
            | ShapeNode::Optional(value)
            | ShapeNode::Newtype { inner: value, .. } => collect(value, references, visiting),
            ShapeNode::Dictionary(key, value) => {
                collect(key, references, visiting);
                collect(value, references, visiting);
            }
            ShapeNode::Tuple(values) | ShapeNode::Union(values) => {
                for value in values {
                    collect(value, references, visiting);
                }
            }
            ShapeNode::Primitive(_)
            | ShapeNode::FixedInteger(_)
            | ShapeNode::Enum { .. }
            | ShapeNode::RecursiveReference(_)
            | ShapeNode::TypeParameter(_)
            | ShapeNode::Other(_) => {}
        }
    }
    let mut references = BTreeSet::new();
    collect(root, &mut references, &mut BTreeSet::new());
    references
}

fn collect_nominal_types(
    ty: &Type,
    module_name: &str,
    owners: &mut HashMap<String, Type>,
    visiting: &mut BTreeSet<String>,
) {
    match ty.resolve_alias() {
        Type::Class {
            identity,
            name,
            fields,
            type_args,
            ..
        } => {
            let identity = identity
                .clone()
                .unwrap_or_else(|| format!("{module_name}.{name}"));
            if !visiting.insert(identity.clone()) {
                return;
            }
            owners.entry(identity.clone()).or_insert_with(|| ty.clone());
            for argument in type_args {
                collect_nominal_types(argument, module_name, owners, visiting);
            }
            for (_, field) in fields {
                collect_nominal_types(field, module_name, owners, visiting);
            }
            visiting.remove(&identity);
        }
        Type::List(value)
        | Type::Set(value)
        | Type::Iterable(value)
        | Type::Iterator(value)
        | Type::Newtype { inner: value, .. } => {
            collect_nominal_types(value, module_name, owners, visiting);
        }
        Type::Dict(key, value) | Type::Result(key, value) => {
            collect_nominal_types(key, module_name, owners, visiting);
            collect_nominal_types(value, module_name, owners, visiting);
        }
        Type::Tuple(values) | Type::Union(values) | Type::Intersection(values) => {
            for value in values {
                collect_nominal_types(value, module_name, owners, visiting);
            }
        }
        _ => {}
    }
}

fn handler_plans_for_target<'a>(
    target_type: &Type,
    result: &'a LoweringResult,
) -> &'a [AdapterHandlerPlan] {
    let Type::Class { name, .. } = target_type.resolve_alias() else {
        return &[];
    };
    result
        .class_adapter_selections
        .iter()
        .find(|selection| selection.owner == *name)
        .map(|selection| selection.handler_plans.as_slice())
        .unwrap_or_default()
}

fn owner_type_for_identity(
    owner_identity: &str,
    target_type: &Type,
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
) -> Option<Type> {
    if matches!(
        target_type.resolve_alias(),
        Type::Class { identity, name, .. }
            if identity.as_deref().unwrap_or(name) == owner_identity
    ) {
        return Some(target_type.clone());
    }
    for class in &result.module.classes {
        if let Some(parent) = &class.parent_type {
            if matches!(
                parent.resolve_alias(),
                Type::Class { identity, name, .. }
                    if identity.as_deref().unwrap_or(name) == owner_identity
            ) {
                return Some(parent.clone());
            }
        }
        let identity = class
            .identity
            .clone()
            .unwrap_or_else(|| format!("{module_name}.{}", class.name));
        if identity == owner_identity {
            return Some(Type::Class {
                identity: Some(identity),
                type_args: class
                    .type_params
                    .iter()
                    .cloned()
                    .map(Type::TypeVar)
                    .collect(),
                name: class.name.clone(),
                fields: class.fields.clone(),
                methods: Vec::new(),
                parent_class: class.semantic_parent_chain(),
            });
        }
    }
    let (source_module, class_name) = owner_identity.rsplit_once('.')?;
    external_defs
        .classes
        .get(source_module)?
        .get(class_name)
        .cloned()
}

fn resolve_method_slot(
    owner_identity: &str,
    method_name: &str,
    owner_type: &Type,
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    expected_identity: Option<&CallableIdentity>,
) -> Result<StaticMethodSlot, MethodSlotError> {
    let (source_module, class_name) = owner_identity.rsplit_once('.').ok_or_else(|| {
        MethodSlotError::new(
            MethodSlotErrorKind::List,
            format!("method slot owner `{owner_identity}` is not module-qualified"),
        )
    })?;
    let hir_name = if method_name == "__init__" {
        "new"
    } else {
        method_name
    };
    if source_module == module_name {
        let class = result
            .module
            .classes
            .iter()
            .find(|class| {
                class.identity.as_deref() == Some(owner_identity)
                    || (class.identity.is_none() && class.name == class_name)
            })
            .ok_or_else(|| {
                MethodSlotError::new(
                    MethodSlotErrorKind::Method,
                    format!("method slot owner `{owner_identity}` is unavailable"),
                )
            })?;
        let method = class
            .methods
            .iter()
            .chain(class.operator_impls.iter().map(|(_, method)| method))
            .find(|method| method.name == hir_name)
            .ok_or_else(|| {
                MethodSlotError::new(
                    MethodSlotErrorKind::Method,
                    format!("method slot `{owner_identity}::{method_name}` is unavailable"),
                )
            })?;
        let mut slot = StaticMethodSlot {
            owner_identity: owner_identity.to_string(),
            owner_type: owner_type.clone(),
            name: method_name.to_string(),
            hir_name: hir_name.to_string(),
            method_kind: method.method_kind,
            receiver: method.receiver,
            params: method.params.iter().map(static_method_param).collect(),
            return_type: method.return_type.clone(),
            is_async: method.is_async,
            input_role: if method.receiver.is_some() {
                StaticMethodSlotInputRole::Receiver
            } else {
                StaticMethodSlotInputRole::Value
            },
            input_type: Type::Unknown,
            output_type: Type::Unknown,
            context_type: None,
            context_mutable: false,
            descriptor_type: None,
            descriptor_value: None,
            descriptor_origin: None,
            descriptor_range: None,
            declaration_order: None,
            is_fallible: false,
        };
        validate_expected_identity(&slot, expected_identity)?;
        specialize_slot_types(&mut slot, &class.type_params, owner_type);
        return finish_method_slot(slot, module_name, result, external_defs);
    }
    let method = external_defs
        .structural_methods_for(source_module)
        .and_then(|classes| classes.get(class_name))
        .and_then(|methods| methods.iter().find(|method| method.name == method_name))
        .ok_or_else(|| {
            MethodSlotError::new(
                MethodSlotErrorKind::Method,
                format!("imported method slot `{owner_identity}::{method_name}` is unavailable"),
            )
        })?;
    let mut slot = StaticMethodSlot {
        owner_identity: owner_identity.to_string(),
        owner_type: owner_type.clone(),
        name: method_name.to_string(),
        hir_name: hir_name.to_string(),
        method_kind: method.method_kind,
        receiver: method.receiver,
        params: method.params.iter().map(static_method_param).collect(),
        return_type: method.return_type.clone(),
        is_async: method.is_async,
        input_role: if method.receiver.is_some() {
            StaticMethodSlotInputRole::Receiver
        } else {
            StaticMethodSlotInputRole::Value
        },
        input_type: Type::Unknown,
        output_type: Type::Unknown,
        context_type: None,
        context_mutable: false,
        descriptor_type: None,
        descriptor_value: None,
        descriptor_origin: None,
        descriptor_range: None,
        declaration_order: None,
        is_fallible: false,
    };
    let type_params = external_defs
        .class_type_params
        .get(source_module)
        .and_then(|classes| classes.get(class_name))
        .map(Vec::as_slice)
        .unwrap_or_default();
    validate_expected_identity(&slot, expected_identity)?;
    specialize_slot_types(&mut slot, type_params, owner_type);
    finish_method_slot(slot, module_name, result, external_defs)
}

fn specialize_slot_types(slot: &mut StaticMethodSlot, type_params: &[String], owner_type: &Type) {
    let Type::Class { type_args, .. } = owner_type.resolve_alias() else {
        return;
    };
    let bindings = type_params
        .iter()
        .cloned()
        .zip(type_args.iter().cloned())
        .collect::<HashMap<_, _>>();
    for parameter in &mut slot.params {
        parameter.ty = sifr_lowering::substitute_type_vars(&parameter.ty, &bindings);
    }
    slot.return_type = sifr_lowering::substitute_type_vars(&slot.return_type, &bindings);
}

fn validate_expected_identity(
    slot: &StaticMethodSlot,
    expected: Option<&CallableIdentity>,
) -> Result<(), MethodSlotError> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let signature = crate::canonical_types::function_identity(&sifr_type_system::FunctionType {
        receiver: slot.receiver,
        params: slot
            .params
            .iter()
            .map(|parameter| {
                (
                    parameter.name.clone(),
                    parameter.ty.clone(),
                    parameter.convention,
                )
            })
            .collect(),
        return_type: Box::new(slot.return_type.clone()),
    });
    if expected.owner.as_deref() != Some(slot.owner_identity.as_str())
        || expected.symbol != slot.name
        || expected.signature != signature
    {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::Signature,
            format!(
                "selected handler `{}::{}` no longer matches its checked method signature",
                slot.owner_identity, slot.name
            ),
        ));
    }
    Ok(())
}

fn finish_method_slot(
    mut slot: StaticMethodSlot,
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
) -> Result<StaticMethodSlot, MethodSlotError> {
    if slot.name == "__init__" {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::Method,
            format!(
                "method slot `{}::{}` cannot name a constructor",
                slot.owner_identity, slot.name
            ),
        ));
    }
    if slot.is_async {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::Method,
            format!(
                "method slot `{}::{}` must be synchronous",
                slot.owner_identity, slot.name
            ),
        ));
    }
    match slot.return_type.resolve_alias() {
        Type::Result(output, _) => {
            slot.output_type = output.as_ref().clone();
            slot.is_fallible = true;
        }
        output => {
            slot.output_type = output.clone();
            slot.is_fallible = false;
        }
    }
    let receiver_input = slot.receiver.is_some();
    let maximum_params = 2;
    let minimum_params = usize::from(!receiver_input);
    if slot.params.len() < minimum_params || slot.params.len() > maximum_params {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::Signature,
            format!(
                "method slot `{}::{}` must have exactly one value input and at most one context parameter",
                slot.owner_identity, slot.name
            ),
        ));
    }
    let context = if receiver_input {
        match slot.params.as_slice() {
            [] => {
                slot.input_type = slot.owner_type.clone();
                None
            }
            [parameter] if parameter.convention.is_borrowed() => {
                slot.input_type = slot.owner_type.clone();
                Some(parameter)
            }
            [value] => {
                debug_assert!(value.convention.is_owned());
                slot.input_role = StaticMethodSlotInputRole::ReceiverAndValue;
                slot.input_type = Type::Tuple(vec![slot.owner_type.clone(), value.ty.clone()]);
                None
            }
            [value, context] => {
                if !value.convention.is_owned() {
                    return Err(MethodSlotError::new(
                        MethodSlotErrorKind::Signature,
                        format!(
                            "method slot `{}::{}` receiver value input must be owned",
                            slot.owner_identity, slot.name
                        ),
                    ));
                }
                slot.input_role = StaticMethodSlotInputRole::ReceiverAndValue;
                slot.input_type = Type::Tuple(vec![slot.owner_type.clone(), value.ty.clone()]);
                Some(context)
            }
            _ => unreachable!("receiver slot parameter count was checked"),
        }
    } else {
        let input = slot.params.first().ok_or_else(|| {
            MethodSlotError::new(
                MethodSlotErrorKind::Signature,
                format!(
                    "method slot `{}::{}` is missing its value input",
                    slot.owner_identity, slot.name
                ),
            )
        })?;
        slot.input_type = input.ty.clone();
        slot.params.get(1)
    };
    if let Some(context) = context {
        if !context.convention.is_borrowed() {
            return Err(MethodSlotError::new(
                MethodSlotErrorKind::Context,
                format!(
                    "method slot `{}::{}` context must be an immutable or mutable borrow",
                    slot.owner_identity, slot.name
                ),
            ));
        }
        slot.context_type = Some(context.ty.clone());
        slot.context_mutable = context.convention.is_mutable();
    }
    if slot.receiver.is_some_and(ReceiverConvention::is_owned)
        && !same_nominal_specialization(&slot.output_type, &slot.owner_type)
    {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::Signature,
            format!(
                "owned handler `{}::{}` must return exactly Self or Result[Self, E]",
                slot.owner_identity, slot.name
            ),
        ));
    }
    if !structural_slot_type_supported(&slot.input_type, module_name, result, external_defs)
        || !structural_slot_type_supported(&slot.output_type, module_name, result, external_defs)
    {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::Signature,
            format!(
                "method slot `{}::{}` input and output must be structural types",
                slot.owner_identity, slot.name
            ),
        ));
    }
    Ok(slot)
}

fn same_nominal_specialization(left: &Type, right: &Type) -> bool {
    let (
        Type::Class {
            identity: left_identity,
            name: left_name,
            type_args: left_args,
            ..
        },
        Type::Class {
            identity: right_identity,
            name: right_name,
            type_args: right_args,
            ..
        },
    ) = (left.resolve_alias(), right.resolve_alias())
    else {
        return false;
    };
    left_name == right_name
        && left_args == right_args
        && match (left_identity, right_identity) {
            (Some(left), Some(right)) => left == right,
            _ => true,
        }
}

fn structural_slot_type_supported(
    ty: &Type,
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
) -> bool {
    fn supported(node: &ShapeNode) -> bool {
        match node {
            ShapeNode::Primitive(_)
            | ShapeNode::FixedInteger(_)
            | ShapeNode::Enum { .. }
            | ShapeNode::RecursiveReference(_) => true,
            ShapeNode::Nominal {
                type_arguments,
                fields,
                ..
            } => {
                type_arguments.iter().all(supported)
                    && fields.iter().all(|field| supported(&field.declared_type))
            }
            ShapeNode::List(value)
            | ShapeNode::Set(value)
            | ShapeNode::Optional(value)
            | ShapeNode::Newtype { inner: value, .. } => supported(value),
            ShapeNode::Dictionary(key, value) => supported(key) && supported(value),
            ShapeNode::Tuple(values) | ShapeNode::Union(values) => values.iter().all(supported),
            ShapeNode::TypeParameter(_) | ShapeNode::Other(_) => false,
        }
    }
    supported(&describe_type_with_externals(module_name, ty, result, external_defs).root)
}

fn static_method_param(param: &sifr_lowering::HirParam) -> StaticMethodParam {
    StaticMethodParam {
        name: param.name.clone(),
        ty: param.ty.clone(),
        keyword_only: param.keyword_only,
        convention: param.convention,
    }
}
