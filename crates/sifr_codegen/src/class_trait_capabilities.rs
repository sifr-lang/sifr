use crate::RustEmitter;
use sifr_ir::{HirClass, HirModule};
use sifr_type_system::Type;

fn parent_chain_contains(parent: Option<&str>, ancestor: &str) -> bool {
    parent.is_some_and(|chain| chain.split('|').any(|name| name == ancestor))
}

fn supports_generic_debug(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::TypeVar(_) => true,
        Type::List(value)
        | Type::Set(value)
        | Type::Iterable(value)
        | Type::Newtype { inner: value, .. }
        | Type::Failure(value)
        | Type::TimeoutResult(value) => supports_generic_debug(value),
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::TaskResult(left, right)
        | Type::Select2(left, right) => {
            supports_generic_debug(left) && supports_generic_debug(right)
        }
        Type::Tuple(values) | Type::Union(values) => values.iter().all(supports_generic_debug),
        Type::Class {
            fields,
            parent_class,
            ..
        } => {
            !parent_chain_contains(parent_class.as_deref(), "NonSend")
                && fields
                    .iter()
                    .all(|(_, field)| supports_generic_debug(field))
        }
        _ => ty.supports_debug_formatting(),
    }
}

fn supports_generic_clone(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::TypeVar(_) => true,
        Type::List(value)
        | Type::Set(value)
        | Type::Iterable(value)
        | Type::Newtype { inner: value, .. } => supports_generic_clone(value),
        Type::Dict(left, right) | Type::Result(left, right) => {
            supports_generic_clone(left) && supports_generic_clone(right)
        }
        Type::Tuple(values) | Type::Union(values) => values.iter().all(supports_generic_clone),
        Type::Class {
            fields,
            parent_class,
            ..
        } => {
            !parent_chain_contains(parent_class.as_deref(), "NonSend")
                && fields
                    .iter()
                    .all(|(_, field)| supports_generic_clone(field))
        }
        _ => ty.supports_derived_clone(),
    }
}

fn supports_generic_hash(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::TypeVar(_) => true,
        Type::Tuple(values) | Type::Union(values) => values.iter().all(supports_generic_hash),
        Type::Result(left, right) => supports_generic_hash(left) && supports_generic_hash(right),
        Type::Newtype { inner, .. } => supports_generic_hash(inner),
        Type::Class {
            fields,
            methods,
            parent_class,
            ..
        } => {
            !parent_chain_contains(parent_class.as_deref(), "NonSend")
                && !methods.iter().any(|(name, _)| name == "__eq__")
                && fields.iter().all(|(_, field)| supports_generic_hash(field))
        }
        _ => ty.supports_hash_key(),
    }
}

fn supports_generic_equality(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::TypeVar(_) => true,
        Type::List(value) | Type::Iterable(value) => supports_generic_equality(value),
        Type::Set(value) => supports_generic_hash(value),
        Type::Dict(key, value) => supports_generic_hash(key) && supports_generic_equality(value),
        Type::Result(left, right) => {
            supports_generic_equality(left) && supports_generic_equality(right)
        }
        Type::Tuple(values) | Type::Union(values) => values.iter().all(supports_generic_equality),
        Type::Newtype { inner, .. } => supports_generic_equality(inner),
        Type::Class {
            fields,
            methods,
            parent_class,
            ..
        } => {
            methods.iter().any(|(name, _)| name == "__eq__")
                || (!parent_chain_contains(parent_class.as_deref(), "NonSend")
                    && fields
                        .iter()
                        .all(|(_, field)| supports_generic_equality(field)))
        }
        _ => ty.supports_structural_equality(),
    }
}

pub(crate) fn supports_declaration_display(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::TypeVar(_)) || ty.supports_display_formatting()
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ClassTraitCapabilities {
    pub(crate) debug: bool,
    pub(crate) clone: bool,
    pub(crate) partial_eq: bool,
    pub(crate) hash: bool,
}

impl ClassTraitCapabilities {
    const fn all() -> Self {
        Self {
            debug: true,
            clone: true,
            partial_eq: true,
            hash: true,
        }
    }
}

impl RustEmitter {
    pub(crate) fn is_current_process_resource_class(&self, class_name: &str) -> bool {
        self.current_module_name.as_deref() == Some("sifr.process")
            && matches!(
                class_name,
                "Child"
                    | "AsyncChild"
                    | "ProcessHandle"
                    | "PipeReader"
                    | "PipeWriter"
                    | "AsyncPipeReader"
                    | "AsyncPipeWriter"
            )
    }

    pub(crate) fn class_trait_capabilities(
        &self,
        class: &HirClass,
        module: &HirModule,
        visiting: &mut std::collections::HashSet<String>,
    ) -> ClassTraitCapabilities {
        if !visiting.insert(class.name.clone()) {
            return ClassTraitCapabilities::default();
        }
        if class.python_opaque_declaration().is_some()
            || self.is_current_process_resource_class(&class.name)
        {
            visiting.remove(&class.name);
            return ClassTraitCapabilities {
                debug: true,
                ..ClassTraitCapabilities::default()
            };
        }

        let has_callable_field = class
            .fields
            .iter()
            .any(|(_, ty)| matches!(ty, Type::Callable(..) | Type::AsyncCallable(..)));
        let has_non_send_class_field = class.fields.iter().any(|(_, ty)| {
            matches!(
                ty.resolve_alias(),
                Type::Class { parent_class, .. }
                    if parent_class.as_deref() == Some("NonSend")
            )
        });
        if has_callable_field
            || has_non_send_class_field
            || class.parent_class.as_deref() == Some("NonSend")
        {
            visiting.remove(&class.name);
            return ClassTraitCapabilities::default();
        }

        let parent = match class.parent_class.as_deref() {
            None => ClassTraitCapabilities::all(),
            Some(parent_name) => module
                .classes
                .iter()
                .find(|candidate| candidate.name == parent_name)
                .map_or_else(ClassTraitCapabilities::default, |parent| {
                    self.class_trait_capabilities(parent, module, visiting)
                }),
        };
        let has_custom_eq = class
            .operator_impls
            .iter()
            .any(|(name, _)| name == "__eq__");
        let has_affine_field = class
            .fields
            .iter()
            .any(|(_, ty)| ty.contains_affine_resource());
        let capabilities = ClassTraitCapabilities {
            debug: parent.debug
                && class
                    .fields
                    .iter()
                    .all(|(_, ty)| supports_generic_debug(ty)),
            clone: parent.clone
                && class
                    .fields
                    .iter()
                    .all(|(_, ty)| supports_generic_clone(ty)),
            partial_eq: parent.partial_eq
                && (has_custom_eq
                    || class
                        .fields
                        .iter()
                        .all(|(_, ty)| supports_generic_equality(ty))),
            hash: parent.hash
                && !has_custom_eq
                && class.is_hashable
                && !has_affine_field
                && class.fields.iter().all(|(_, ty)| supports_generic_hash(ty)),
        };
        visiting.remove(&class.name);
        capabilities
    }
}
