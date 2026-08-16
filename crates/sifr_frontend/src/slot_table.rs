use crate::{describe_type_with_externals, ShapeNode, StructuralShape};
use sifr_lowering::{
    ExternalDefs, LoweringResult, StaticMethodParam, StaticMethodSlot, StaticMethodSlotContext,
};
use sifr_type_system::Type;
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
}

impl MethodSlotError {
    fn new(kind: MethodSlotErrorKind, reason: impl Into<String>) -> Self {
        Self {
            kind,
            reason: reason.into(),
        }
    }

    pub(crate) const fn kind(&self) -> MethodSlotErrorKind {
        self.kind
    }

    pub(crate) fn into_reason(self) -> String {
        self.reason
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
    let mut owner_types = HashMap::new();
    collect_nominal_types(
        target_type,
        module_name,
        &mut owner_types,
        &mut BTreeSet::new(),
    );
    let available = shape_method_references(&described_shape.root);
    let mut slots = Vec::with_capacity(references.len());
    for reference in references {
        if !available.contains(&reference) {
            return Err(MethodSlotError::new(
                MethodSlotErrorKind::Method,
                format!(
                    "method slot `{reference}` does not name an annotated method in the concrete structural shape"
                ),
            ));
        }
        let (owner_identity, method_name) = reference.rsplit_once("::").ok_or_else(|| {
            MethodSlotError::new(
                MethodSlotErrorKind::List,
                format!(
                    "method slot `{reference}` must use the exact `module.Type::method` identity"
                ),
            )
        })?;
        let owner_type = owner_types.get(owner_identity).cloned().ok_or_else(|| {
            MethodSlotError::new(
                MethodSlotErrorKind::Method,
                format!(
                    "method slot owner `{owner_identity}` is not reachable from the concrete type"
                ),
            )
        })?;
        slots.push(resolve_method_slot(
            owner_identity,
            method_name,
            owner_type,
            module_name,
            result,
            external_defs,
        )?);
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
                ));
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

fn method_slot_references(
    value: &crate::ConstValue,
) -> Result<Option<Vec<String>>, MethodSlotError> {
    let crate::ConstValue::Record(fields) = value else {
        return Ok(None);
    };
    let Some(value) = fields.get("sifr_method_slots") else {
        return Ok(None);
    };
    let crate::ConstValue::List(values) = value else {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::List,
            "reserved `sifr_method_slots` must be a nonempty list of strings",
        ));
    };
    if values.is_empty() {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::List,
            "reserved `sifr_method_slots` must not be empty",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut references = Vec::with_capacity(values.len());
    for value in values {
        let crate::ConstValue::String(reference) = value else {
            return Err(MethodSlotError::new(
                MethodSlotErrorKind::List,
                "reserved `sifr_method_slots` must contain only strings",
            ));
        };
        if !seen.insert(reference.clone()) {
            return Err(MethodSlotError::new(
                MethodSlotErrorKind::List,
                format!("method slot `{reference}` is duplicated"),
            ));
        }
        references.push(reference.clone());
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

fn resolve_method_slot(
    owner_identity: &str,
    method_name: &str,
    owner_type: Type,
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
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
        return finish_method_slot(
            StaticMethodSlot {
                owner_identity: owner_identity.to_string(),
                owner_type,
                name: method_name.to_string(),
                hir_name: hir_name.to_string(),
                method_kind: method.method_kind,
                receiver: method.receiver,
                params: method.params.iter().map(static_method_param).collect(),
                return_type: method.return_type.clone(),
                is_async: method.is_async,
                input_type: Type::Unknown,
                output_type: Type::Unknown,
                context_type: None,
                context_mutable: false,
            },
            module_name,
            result,
            external_defs,
        );
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
    finish_method_slot(
        StaticMethodSlot {
            owner_identity: owner_identity.to_string(),
            owner_type,
            name: method_name.to_string(),
            hir_name: hir_name.to_string(),
            method_kind: method.method_kind,
            receiver: method.receiver,
            params: method.params.iter().map(static_method_param).collect(),
            return_type: method.return_type.clone(),
            is_async: method.is_async,
            input_type: Type::Unknown,
            output_type: Type::Unknown,
            context_type: None,
            context_mutable: false,
        },
        module_name,
        result,
        external_defs,
    )
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
    if slot.method_kind == sifr_lowering::MethodKind::ClassMethod {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::Method,
            format!(
                "method slot `{}::{}` cannot be a class method; use a static method or instance receiver",
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
    let Type::Result(output, _) = slot.return_type.resolve_alias() else {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::Signature,
            format!(
                "method slot `{}::{}` must return Result",
                slot.owner_identity, slot.name
            ),
        ));
    };
    slot.output_type = output.as_ref().clone();
    let receiver_input = slot.receiver.is_some();
    let maximum_params = if receiver_input { 1 } else { 2 };
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
        slot.input_type = slot.owner_type.clone();
        slot.params.first()
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
    if !structural_slot_type_supported(&slot.input_type, module_name, result, external_defs)
        || !structural_slot_type_supported(&slot.output_type, module_name, result, external_defs)
    {
        return Err(MethodSlotError::new(
            MethodSlotErrorKind::Signature,
            format!(
                "method slot `{}::{}` input and successful Result output must be structural types",
                slot.owner_identity, slot.name
            ),
        ));
    }
    Ok(slot)
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
