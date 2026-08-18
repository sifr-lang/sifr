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
            let has_collision = class_methods(&owner_type).is_some_and(|methods| {
                methods
                    .iter()
                    .any(|(name, _)| name == &declaration.public_name)
            });
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
            let canonical_qualified = class_name(&owner_type)
                .filter(|name| *name != owner)
                .map(|name| format!("{name}.{}", declaration.public_name));
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
            .filter(|declaration| declaration.set == *set)
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

fn class_methods(owner_type: &Type) -> Option<&[(String, FunctionType)]> {
    match owner_type.resolve_alias() {
        Type::Class { methods, .. } => Some(methods),
        _ => None,
    }
}

fn class_name(owner_type: &Type) -> Option<&str> {
    match owner_type.resolve_alias() {
        Type::Class { name, .. } => Some(name),
        _ => None,
    }
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
