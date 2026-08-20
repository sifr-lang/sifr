use super::{HirImport, LowerCtx, Type};
use sifr_ir::{AttachedApiDeclaration, AttachedApiReceiver, AttachedApiSetIdentity};
use sifr_type_system::{FunctionType, ReceiverConvention};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug)]
pub(super) struct AttachedMethodBinding {
    pub declaration: AttachedApiDeclaration,
    pub emitted_function: String,
    pub provisional: bool,
}

pub(super) fn apply(ctx: &mut LowerCtx) {
    let module = ctx.current_module_name.clone().unwrap_or_default();
    let selections = ctx
        .adapted_class_bindings
        .iter()
        .map(|(owner, selection)| (owner.clone(), selection.clone()))
        .collect::<Vec<_>>();
    let mut imports: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for (owner, selection) in selections {
        let owner_is_local = ctx
            .class_adapter_selections
            .iter()
            .any(|candidate| candidate.owner == owner);
        let selection_is_finalized = ctx.attached_api_selections_finalized || !owner_is_local;
        let (declarations, provisional) = match selection.attached_api_set.as_ref() {
            Some(set) => (declarations_for_set(ctx, set, &module), false),
            None if selection_is_finalized => continue,
            None => (
                provisional_declarations(ctx, &selection.provider_module),
                true,
            ),
        };
        if declarations.is_empty() {
            if !provisional {
                super::descriptor_declarations::malformed(
                    ctx,
                    "attached_api_set",
                    "selected attached-API set has no visible package functions",
                    selection.range,
                );
            }
            continue;
        }
        let Some(owner_type) = ctx.class_types.get(&owner).cloned() else {
            super::descriptor_declarations::malformed(
                ctx,
                "attached_api_owner",
                "selected attached-API owner type is unavailable",
                selection.range,
            );
            continue;
        };
        for declaration in declarations {
            let qualified = format!("{owner}.{}", declaration.public_name);
            let has_collision = class_has_member(&owner_type, &declaration.public_name);
            if has_collision {
                if provisional {
                    continue;
                }
                let native_range = ctx
                    .method_source_ranges
                    .get(&qualified)
                    .copied()
                    .unwrap_or(selection.range);
                super::descriptor_declarations::malformed(
                    ctx,
                    "attached_api_collision",
                    "attached API public name collides with a native class member",
                    native_range,
                );
                super::descriptor_declarations::malformed(
                    ctx,
                    "attached_api_collision",
                    "selected attached API collides with this class declaration",
                    selection.range,
                );
                continue;
            }

            let function_type = specialize_owner(&declaration, &owner_type);
            let surface = surface_function_type(&declaration, &function_type);
            if let Some(Type::Class { methods, .. }) = ctx.class_types.get_mut(&owner) {
                methods.push((declaration.public_name.clone(), surface));
            }
            let canonical_qualified = canonical_owner_binding_key(&selection.owner, &owner_type)
                .filter(|identity| *identity != owner)
                .map(|identity| format!("{identity}.{}", declaration.public_name));
            if declaration.receiver != AttachedApiReceiver::Type {
                ctx.class_instance_methods.insert(qualified.clone());
                if let Some(canonical) = canonical_qualified.as_ref() {
                    ctx.class_instance_methods.insert(canonical.clone());
                }
            }
            let emitted_function = if declaration.module == module {
                declaration.function.clone()
            } else {
                let alias = hidden_alias(&declaration.module, &declaration.function);
                imports
                    .entry(declaration.module.clone())
                    .or_default()
                    .insert(declaration.function.clone(), alias.clone());
                alias
            };
            let binding = AttachedMethodBinding {
                declaration,
                emitted_function,
                provisional,
            };
            ctx.attached_method_bindings
                .insert(qualified, binding.clone());
            if let Some(canonical) = canonical_qualified {
                ctx.attached_method_bindings.insert(canonical, binding);
            }
        }
    }

    ctx.synthetic_attached_imports = imports
        .into_iter()
        .map(|(module, entries)| HirImport {
            module,
            names: entries.keys().cloned().collect(),
            aliases: entries.into_iter().collect(),
        })
        .collect();
}

fn provisional_declarations(ctx: &LowerCtx, provider_module: &str) -> Vec<AttachedApiDeclaration> {
    let mut declarations = ctx
        .attached_apis
        .iter()
        .filter(|declaration| !declaration.function.starts_with('_'))
        .cloned()
        .chain(
            ctx.externals
                .attached_apis
                .values()
                .flat_map(HashMap::values)
                .filter(|declaration| !declaration.function.starts_with('_'))
                .cloned(),
        )
        .collect::<Vec<_>>();
    declarations.sort_by(|left, right| {
        (left.module != provider_module)
            .cmp(&(right.module != provider_module))
            .then_with(|| left.public_name.cmp(&right.public_name))
            .then_with(|| left.module.cmp(&right.module))
            .then_with(|| left.set.symbol.cmp(&right.set.symbol))
            .then_with(|| left.function.cmp(&right.function))
    });
    declarations.dedup_by(|left, right| left.public_name == right.public_name);
    declarations
}

fn declarations_for_set(
    ctx: &LowerCtx,
    set: &AttachedApiSetIdentity,
    current_module: &str,
) -> Vec<AttachedApiDeclaration> {
    let mut declarations = if set.module == current_module {
        ctx.attached_apis
            .iter()
            .filter(|declaration| declaration.set == *set && !declaration.function.starts_with('_'))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        ctx.externals
            .attached_apis
            .get(&set.module)
            .into_iter()
            .flat_map(HashMap::values)
            .filter(|declaration| declaration.set == *set && !declaration.function.starts_with('_'))
            .cloned()
            .collect::<Vec<_>>()
    };
    declarations.sort_by(|left, right| {
        left.public_name
            .cmp(&right.public_name)
            .then_with(|| left.function.cmp(&right.function))
    });
    declarations
}

pub(super) fn specialize_owner(
    declaration: &AttachedApiDeclaration,
    owner_type: &Type,
) -> FunctionType {
    let bindings = HashMap::from([(declaration.owner_type_param.clone(), owner_type.clone())]);
    substitute_function_type(&declaration.function_type, &bindings)
}

fn surface_function_type(
    declaration: &AttachedApiDeclaration,
    function_type: &FunctionType,
) -> FunctionType {
    let (receiver, params) = match declaration.receiver {
        AttachedApiReceiver::Type => (None, function_type.params.clone()),
        AttachedApiReceiver::Immutable => (
            Some(ReceiverConvention::SharedBorrow),
            function_type.params.iter().skip(1).cloned().collect(),
        ),
        AttachedApiReceiver::Mutable => (
            Some(ReceiverConvention::MutableBorrow),
            function_type.params.iter().skip(1).cloned().collect(),
        ),
        AttachedApiReceiver::Owned => (
            Some(ReceiverConvention::Owned),
            function_type.params.iter().skip(1).cloned().collect(),
        ),
    };
    FunctionType {
        receiver,
        params,
        return_type: function_type.return_type.clone(),
    }
}

fn substitute_function_type(
    function_type: &FunctionType,
    bindings: &HashMap<String, Type>,
) -> FunctionType {
    FunctionType {
        receiver: function_type.receiver,
        params: function_type
            .params
            .iter()
            .map(|(name, ty, convention)| {
                (
                    name.clone(),
                    super::substitute_type_vars(ty, bindings),
                    *convention,
                )
            })
            .collect(),
        return_type: Box::new(super::substitute_type_vars(
            &function_type.return_type,
            bindings,
        )),
    }
}

fn class_has_member(owner_type: &Type, member: &str) -> bool {
    match owner_type.resolve_alias() {
        Type::Class {
            methods, fields, ..
        } => {
            methods.iter().any(|(name, _)| name == member)
                || fields.iter().any(|(name, _)| name == member)
        }
        _ => false,
    }
}

fn canonical_owner_binding_key<'a>(selected_owner: &str, owner_type: &'a Type) -> Option<&'a str> {
    match owner_type.resolve_alias() {
        Type::Class {
            identity: Some(identity),
            ..
        } if identity.rsplit('.').next() == Some(selected_owner) => Some(identity),
        _ => None,
    }
}

pub(super) fn binding_for_owner<'a>(
    ctx: &'a LowerCtx,
    surface_name: &str,
    owner_type: &Type,
    member: &str,
) -> Option<&'a AttachedMethodBinding> {
    let (canonical_identity, emitted_name) = match owner_type.resolve_alias() {
        Type::Class { identity, name, .. } => (identity.as_deref(), Some(name.as_str())),
        _ => (None, None),
    };
    [Some(surface_name), canonical_identity, emitted_name]
        .into_iter()
        .flatten()
        .find_map(|owner| {
            ctx.attached_method_bindings
                .get(&format!("{owner}.{member}"))
        })
}

fn hidden_alias(module: &str, function: &str) -> String {
    let sanitized = module
        .chars()
        .chain(std::iter::once('_'))
        .chain(function.chars())
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("__sifr_attached_api_{sanitized}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class(identity: Option<&str>) -> Type {
        Type::Class {
            identity: identity.map(str::to_string),
            type_args: Vec::new(),
            name: "Selected".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        }
    }

    #[test]
    fn canonical_binding_key_requires_the_selected_owner_symbol() {
        let selected = class(Some("fixture.Selected"));
        let mismatched = class(Some("fixture.Other"));

        assert_eq!(
            canonical_owner_binding_key("Selected", &selected),
            Some("fixture.Selected")
        );
        assert_eq!(canonical_owner_binding_key("Selected", &mismatched), None);
    }

    #[test]
    fn module_less_owner_does_not_gain_a_synthetic_binding_key() {
        assert_eq!(canonical_owner_binding_key("Selected", &class(None)), None);
    }
}
