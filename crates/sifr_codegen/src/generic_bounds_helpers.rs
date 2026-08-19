use crate::{RustEmitter, RustItem};
use sifr_ir::{HirClass, HirFunction, HirStmt};
use sifr_type_system::{FunctionType, Type};
use std::collections::{HashMap, HashSet};

impl RustEmitter {
    pub(crate) fn class_needs_phantom_marker(class: &HirClass) -> bool {
        class.type_params.iter().any(|type_param| {
            !class
                .fields
                .iter()
                .any(|(_, field)| Self::type_mentions_type_param(field, type_param))
                && !class
                    .parent_type
                    .as_ref()
                    .is_some_and(|parent| Self::type_mentions_type_param(parent, type_param))
        })
    }

    pub(crate) fn type_mentions_type_param(ty: &Type, type_param: &str) -> bool {
        match ty.resolve_alias() {
            Type::TypeVar(name) => name == type_param,
            Type::List(value)
            | Type::Set(value)
            | Type::Iterable(value)
            | Type::Iterator(value)
            | Type::Awaitable(value)
            | Type::Failure(value)
            | Type::TimeoutResult(value)
            | Type::Newtype { inner: value, .. } => {
                Self::type_mentions_type_param(value, type_param)
            }
            Type::Dict(left, right)
            | Type::Result(left, right)
            | Type::Task(left, right)
            | Type::TaskResult(left, right)
            | Type::Coroutine(left, right)
            | Type::Select2(left, right)
            | Type::BlockingTask(left, right)
            | Type::JoinSet(left, right)
            | Type::AsyncIterator(left, right)
            | Type::AsyncGenerator(left, right) => {
                Self::type_mentions_type_param(left, type_param)
                    || Self::type_mentions_type_param(right, type_param)
            }
            Type::Tuple(values) | Type::Union(values) | Type::Intersection(values) => values
                .iter()
                .any(|value| Self::type_mentions_type_param(value, type_param)),
            Type::Callable(params, _, return_type)
            | Type::AsyncCallable(params, _, return_type) => {
                params
                    .iter()
                    .any(|value| Self::type_mentions_type_param(value, type_param))
                    || Self::type_mentions_type_param(return_type, type_param)
            }
            Type::Function(function) | Type::AsyncFunction(function) => {
                function
                    .params
                    .iter()
                    .any(|(_, value, _)| Self::type_mentions_type_param(value, type_param))
                    || Self::type_mentions_type_param(&function.return_type, type_param)
            }
            Type::Class {
                fields, methods, ..
            } => {
                fields
                    .iter()
                    .any(|(_, value)| Self::type_mentions_type_param(value, type_param))
                    || methods.iter().any(|(_, function)| {
                        function
                            .params
                            .iter()
                            .any(|(_, value, _)| Self::type_mentions_type_param(value, type_param))
                            || Self::type_mentions_type_param(&function.return_type, type_param)
                    })
            }
            _ => false,
        }
    }

    /// Whether this type parameter occurs in a stored hash-key position.
    pub(crate) fn class_type_param_needs_hash_eq(class: &HirClass, type_param: &str) -> bool {
        fn type_has_hash_key_param(ty: &Type, type_param: &str) -> bool {
            match ty.resolve_alias() {
                Type::Dict(key, value) => {
                    RustEmitter::type_mentions_type_param(key, type_param)
                        || type_has_hash_key_param(value, type_param)
                }
                Type::Set(value) => RustEmitter::type_mentions_type_param(value, type_param),
                Type::List(inner)
                | Type::Iterable(inner)
                | Type::Iterator(inner)
                | Type::Newtype { inner, .. } => type_has_hash_key_param(inner, type_param),
                Type::Tuple(members) | Type::Union(members) | Type::Intersection(members) => {
                    members
                        .iter()
                        .any(|member| type_has_hash_key_param(member, type_param))
                }
                _ => false,
            }
        }
        class
            .fields
            .iter()
            .any(|(_, ty)| type_has_hash_key_param(ty, type_param))
    }

    /// Check if a generic function needs Hash + Eq bounds (uses `TypeVar` as dict key
    /// or returns a generic class that needs Hash + Eq).
    pub(crate) fn func_needs_hash_eq(func: &HirFunction) -> bool {
        fn type_has_typevar_dict_key(ty: &Type) -> bool {
            match ty {
                Type::Dict(key, _) => matches!(key.as_ref(), Type::TypeVar(_)),
                Type::List(inner) => type_has_typevar_dict_key(inner),
                Type::Union(members) => members.iter().any(type_has_typevar_dict_key),
                Type::Class { fields, .. } => {
                    fields.iter().any(|(_, t)| type_has_typevar_dict_key(t))
                }
                _ => false,
            }
        }
        // Check params
        if func.params.iter().any(|p| type_has_typevar_dict_key(&p.ty)) {
            return true;
        }
        // Check return type
        if type_has_typevar_dict_key(&func.return_type) {
            return true;
        }
        false
    }

    /// Convert a Sifr type to structured Rust IR, filling inferred generic
    /// arguments for classes whose HIR type does not carry explicit arguments.
    pub(crate) fn rust_ir_type_with_generics(&self, ty: &Type) -> crate::RustType {
        if let Type::Class {
            name, type_args, ..
        } = ty
        {
            let base = crate::sifr_type_to_rust_type(ty);
            if !self.generic_classes.contains(name) || !type_args.is_empty() {
                return base;
            }
            let params = self
                .infer_generic_class_type_args(name, ty)
                .map(|args| {
                    args.iter()
                        .map(|arg| self.rust_ir_type_with_generics(arg))
                        .collect::<Vec<_>>()
                })
                .or_else(|| {
                    self.generic_class_params.get(name).map(|params| {
                        params
                            .iter()
                            .cloned()
                            .map(crate::RustType::Named)
                            .collect::<Vec<_>>()
                    })
                });
            if let (crate::RustType::Named(name), Some(params)) = (&base, params) {
                return crate::RustType::Generic {
                    base: name.clone(),
                    params,
                };
            }
            return base;
        }
        crate::sifr_type_to_rust_type(ty)
    }

    pub(crate) fn rust_ir_struct_field_type_with_generics(&self, ty: &Type) -> crate::RustType {
        match ty {
            Type::Callable(..) | Type::AsyncCallable(..) => crate::sifr_type_to_rust_field_type(ty),
            _ => self.rust_ir_type_with_generics(ty),
        }
    }

    pub(crate) fn rust_ir_type_with_static_bound(&self, ty: &Type) -> crate::RustType {
        let mut rust_ty = self.rust_ir_type_with_generics(ty);
        if let crate::RustType::DynTrait { auto_traits, .. }
        | crate::RustType::ImplTrait { auto_traits, .. } = &mut rust_ty
        {
            auto_traits.push("'static".to_string());
        }
        rust_ty
    }

    pub(crate) fn render_rust_type_with_generics(&self, ty: &Type) -> String {
        crate::render_type(&self.rust_ir_type_with_generics(ty))
    }

    pub(crate) fn rust_generator_error_type_with_generics(&self, ty: &Type) -> crate::RustType {
        if matches!(ty.resolve_alias(), Type::Never) {
            crate::RustType::Named("std::convert::Infallible".to_string())
        } else {
            self.rust_ir_type_with_generics(ty)
        }
    }

    pub(crate) fn type_contains_generic_class(&self, ty: &Type) -> bool {
        match ty {
            Type::Class { name, .. } => self.generic_classes.contains(name),
            Type::List(inner) | Type::Set(inner) | Type::Alias { body: inner, .. } => {
                self.type_contains_generic_class(inner)
            }
            Type::Dict(key, value) | Type::Result(key, value) => {
                self.type_contains_generic_class(key) || self.type_contains_generic_class(value)
            }
            Type::Tuple(items) | Type::Union(items) | Type::Intersection(items) => items
                .iter()
                .any(|item| self.type_contains_generic_class(item)),
            Type::Callable(params, _, ret) | Type::AsyncCallable(params, _, ret) => {
                params
                    .iter()
                    .any(|param| self.type_contains_generic_class(param))
                    || self.type_contains_generic_class(ret)
            }
            Type::Function(ft) => {
                ft.params
                    .iter()
                    .any(|(_, param_ty, _)| self.type_contains_generic_class(param_ty))
                    || self.type_contains_generic_class(ft.return_type.as_ref())
            }
            _ => false,
        }
    }

    pub(crate) fn render_generic_class_type(&self, name: &str, ty: &Type) -> String {
        if !self.generic_classes.contains(name) {
            return name.to_string();
        }

        if let Some(type_args) = self.infer_generic_class_type_args(name, ty) {
            return format!(
                "{name}<{}>",
                type_args
                    .iter()
                    .map(|arg| self.render_rust_type_with_generics(arg))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if let Some(params) = self.generic_class_params.get(name) {
            return format!("{name}<{}>", params.join(", "));
        }

        name.to_string()
    }

    pub(crate) fn infer_generic_class_type_args(&self, name: &str, ty: &Type) -> Option<Vec<Type>> {
        let template = self.generic_class_templates.get(name)?;
        let Type::Class {
            type_args,
            fields,
            methods,
            ..
        } = ty
        else {
            return None;
        };
        if template.type_params.is_empty() {
            return None;
        }
        if type_args.len() == template.type_params.len() {
            return Some(type_args.clone());
        }

        let mut bindings = HashMap::new();
        let mut visiting = HashSet::new();
        self.collect_typevar_bindings_from_class(
            template,
            fields,
            methods,
            &mut bindings,
            &mut visiting,
        );

        template
            .type_params
            .iter()
            .map(|type_param| bindings.get(type_param).cloned())
            .collect()
    }

    pub(crate) fn collect_typevar_bindings_from_class(
        &self,
        template: &HirClass,
        concrete_fields: &[(String, Type)],
        concrete_methods: &[(String, FunctionType)],
        bindings: &mut HashMap<String, Type>,
        visiting: &mut HashSet<String>,
    ) {
        if !visiting.insert(template.name.clone()) {
            return;
        }

        for (field_name, template_ty) in &template.fields {
            if let Some((_, concrete_ty)) = concrete_fields
                .iter()
                .find(|(candidate_name, _)| candidate_name == field_name)
            {
                self.collect_typevar_bindings(template_ty, concrete_ty, bindings, visiting);
            }
        }

        for template_method in &template.methods {
            if let Some((_, concrete_method)) = concrete_methods
                .iter()
                .find(|(candidate_name, _)| candidate_name == &template_method.name)
            {
                self.collect_typevar_bindings_from_function_type(
                    template_method,
                    concrete_method,
                    bindings,
                    visiting,
                );
            }
        }

        visiting.remove(&template.name);
    }

    pub(crate) fn collect_typevar_bindings_from_function_type(
        &self,
        template: &HirFunction,
        concrete: &FunctionType,
        bindings: &mut HashMap<String, Type>,
        visiting: &mut HashSet<String>,
    ) {
        for (template_param, concrete_param) in template.params.iter().zip(concrete.params.iter()) {
            self.collect_typevar_bindings(
                &template_param.ty,
                &concrete_param.1,
                bindings,
                visiting,
            );
        }
        self.collect_typevar_bindings(
            &template.return_type,
            concrete.return_type.as_ref(),
            bindings,
            visiting,
        );
    }

    pub(crate) fn collect_typevar_bindings(
        &self,
        template: &Type,
        concrete: &Type,
        bindings: &mut HashMap<String, Type>,
        visiting: &mut HashSet<String>,
    ) {
        match (template, concrete) {
            (Type::TypeVar(name), ty) => {
                bindings.entry(name.clone()).or_insert_with(|| ty.clone());
            }
            (Type::List(template_inner), Type::List(concrete_inner))
            | (Type::Set(template_inner), Type::Set(concrete_inner)) => {
                self.collect_typevar_bindings(template_inner, concrete_inner, bindings, visiting);
            }
            (
                Type::Dict(template_key, template_value),
                Type::Dict(concrete_key, concrete_value),
            )
            | (
                Type::Result(template_key, template_value),
                Type::Result(concrete_key, concrete_value),
            ) => {
                self.collect_typevar_bindings(template_key, concrete_key, bindings, visiting);
                self.collect_typevar_bindings(template_value, concrete_value, bindings, visiting);
            }
            (Type::Tuple(template_items), Type::Tuple(concrete_items))
            | (Type::Union(template_items), Type::Union(concrete_items)) => {
                for (template_item, concrete_item) in
                    template_items.iter().zip(concrete_items.iter())
                {
                    self.collect_typevar_bindings(template_item, concrete_item, bindings, visiting);
                }
            }
            (
                Type::Alias {
                    type_args: template_args,
                    body: template_body,
                    ..
                },
                Type::Alias {
                    type_args: concrete_args,
                    body: concrete_body,
                    ..
                },
            ) => {
                for (template_arg, concrete_arg) in template_args.iter().zip(concrete_args.iter()) {
                    self.collect_typevar_bindings(template_arg, concrete_arg, bindings, visiting);
                }
                self.collect_typevar_bindings(template_body, concrete_body, bindings, visiting);
            }
            (
                Type::Callable(template_params, _, template_ret),
                Type::Callable(concrete_params, _, concrete_ret),
            ) => {
                for (template_param, concrete_param) in
                    template_params.iter().zip(concrete_params.iter())
                {
                    self.collect_typevar_bindings(
                        template_param,
                        concrete_param,
                        bindings,
                        visiting,
                    );
                }
                self.collect_typevar_bindings(template_ret, concrete_ret, bindings, visiting);
            }
            (
                Type::AsyncCallable(template_params, _, template_ret),
                Type::AsyncCallable(concrete_params, _, concrete_ret),
            ) => {
                for (template_param, concrete_param) in
                    template_params.iter().zip(concrete_params.iter())
                {
                    self.collect_typevar_bindings(
                        template_param,
                        concrete_param,
                        bindings,
                        visiting,
                    );
                }
                self.collect_typevar_bindings(template_ret, concrete_ret, bindings, visiting);
            }
            (Type::Function(template_ft), Type::Function(concrete_ft)) => {
                for (template_param, concrete_param) in
                    template_ft.params.iter().zip(concrete_ft.params.iter())
                {
                    self.collect_typevar_bindings(
                        &template_param.1,
                        &concrete_param.1,
                        bindings,
                        visiting,
                    );
                }
                self.collect_typevar_bindings(
                    template_ft.return_type.as_ref(),
                    concrete_ft.return_type.as_ref(),
                    bindings,
                    visiting,
                );
            }
            (
                Type::Class {
                    name: template_name,
                    ..
                },
                Type::Class {
                    name: concrete_name,
                    fields: concrete_fields,
                    methods: concrete_methods,
                    ..
                },
            ) if template_name == concrete_name => {
                if let Some(template_class) = self.generic_class_templates.get(template_name) {
                    self.collect_typevar_bindings_from_class(
                        template_class,
                        concrete_fields,
                        concrete_methods,
                        bindings,
                        visiting,
                    );
                }
            }
            _ => {}
        }
    }

    pub(crate) fn extra_bound_items_for_type_param(tp: &str, body: &[HirStmt]) -> Vec<String> {
        let requirements =
            crate::hir_analysis::queries::collect_typevar_operator_requirements(body, tp);
        let mut extra = Vec::new();
        if requirements.needs_add {
            extra.push(format!("std::ops::Add<Output = {tp}>"));
        }
        if requirements.needs_sub {
            extra.push(format!("std::ops::Sub<Output = {tp}>"));
        }
        if requirements.needs_mul {
            extra.push(format!("std::ops::Mul<Output = {tp}>"));
        }
        if requirements.needs_div {
            extra.push(format!("std::ops::Div<Output = {tp}>"));
        }
        if requirements.needs_rem {
            extra.push(format!("std::ops::Rem<Output = {tp}>"));
        }
        if requirements.needs_neg {
            extra.push(format!("std::ops::Neg<Output = {tp}>"));
        }
        if requirements.needs_partial_eq {
            extra.push("PartialEq".to_string());
        }
        if requirements.needs_partial_ord {
            extra.push("PartialOrd".to_string());
        }
        extra
    }

    /// Rust trait requirements are transitive across calls on any instance of
    /// the current class, including both `self.method()` and `other.method()`.
    pub(crate) fn class_method_type_param_bounds(
        class: &HirClass,
        method_items: &[(&HirFunction, RustItem)],
    ) -> HashMap<String, HashMap<String, HashSet<String>>> {
        let mut requirements = method_items
            .iter()
            .map(|(method, item)| {
                let emitted_clone = Self::emitted_items_require_clone(std::slice::from_ref(item));
                let params = class
                    .type_params
                    .iter()
                    .map(|param| {
                        let mut bounds =
                            Self::extra_bound_items_for_type_param(param, &method.body)
                                .into_iter()
                                .collect::<HashSet<_>>();
                        if emitted_clone && Self::body_mentions_type_param(&method.body, param) {
                            bounds.insert("Clone".to_string());
                        }
                        (param.clone(), bounds)
                    })
                    .collect::<HashMap<_, _>>();
                (method.name.clone(), params)
            })
            .collect::<HashMap<_, _>>();

        loop {
            let mut changed = false;
            for (method, _) in method_items {
                let inherited = Self::collect_direct_class_method_calls(&method.body, &class.name)
                    .into_iter()
                    .filter_map(|called| requirements.get(&called))
                    .cloned()
                    .collect::<Vec<_>>();
                let entry = requirements.entry(method.name.clone()).or_default();
                for inherited_by_param in inherited {
                    for (param, bounds) in inherited_by_param {
                        let target = entry.entry(param).or_default();
                        let before = target.len();
                        target.extend(bounds);
                        changed |= target.len() != before;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        requirements
    }

    /// Whether a method body actually consumes a value whose semantic type
    /// contains this class type parameter. This is paired with inspection of
    /// the emitted item so a clone in one consumer never leaks a `Clone` bound
    /// onto unrelated parameters of a multi-parameter class.
    pub(crate) fn body_mentions_type_param(body: &[HirStmt], type_param: &str) -> bool {
        let mut mentioned = false;
        let mut on_stmt = |_stmt: &HirStmt| {};
        let mut on_expr = |expr: &sifr_ir::HirExpr| {
            if matches!(expr, sifr_ir::HirExpr::Name { name, .. } if name == "self") {
                return;
            }
            if Self::type_mentions_type_param(expr.ty(), type_param) {
                mentioned = true;
            }
        };
        crate::hir_analysis::traversal::walk_stmts(
            body,
            crate::hir_analysis::traversal::TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        mentioned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_type_rendering_preserves_compiler_owned_class_names() {
        let emitter = RustEmitter::new();
        for (identity, expected) in [
            ("sifr.io.FileHandle", "__SifrIoFileHandle"),
            ("sifr.io.TextFileHandle", "__SifrIoTextFileHandle"),
        ] {
            let ty = Type::Class {
                identity: Some(identity.to_string()),
                type_args: Vec::new(),
                name: identity.rsplit('.').next().expect("class name").to_string(),
                fields: Vec::new(),
                methods: Vec::new(),
                parent_class: None,
            };
            assert_eq!(emitter.render_rust_type_with_generics(&ty), expected);
        }
    }

    #[test]
    fn canonical_generic_class_rendering_preserves_concrete_arguments() {
        let mut emitter = RustEmitter::new();
        emitter.generic_classes.insert("NullContext".to_string());
        let ty = Type::Class {
            identity: Some("sifr.resource.NullContext".to_string()),
            type_args: vec![Type::Int],
            name: "NullContext".to_string(),
            fields: vec![("value".to_string(), Type::Int)],
            methods: Vec::new(),
            parent_class: None,
        };
        let canonical = sifr_type_system::stdlib_class_rust_name("sifr.resource", "NullContext");

        assert_eq!(
            emitter.render_rust_type_with_generics(&ty),
            format!("{canonical}<i64>")
        );
    }
}
