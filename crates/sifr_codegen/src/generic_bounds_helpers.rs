use crate::RustEmitter;
use sifr_ir::{HirClass, HirFunction, HirStmt};
use sifr_type_system::{FunctionType, OwnershipKind, Type};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

impl RustEmitter {
    /// Check if a generic class needs Hash + Eq bounds on its type parameters.
    /// This is true when a type parameter is used as a `HashMap` key (dict field with `TypeVar` key).
    pub(crate) fn class_needs_hash_eq(class: &HirClass) -> bool {
        fn type_has_typevar_dict_key(ty: &Type) -> bool {
            match ty {
                Type::Dict(key, _) => matches!(key.as_ref(), Type::TypeVar(_)),
                Type::List(inner) => type_has_typevar_dict_key(inner),
                Type::Union(members) => members.iter().any(type_has_typevar_dict_key),
                _ => false,
            }
        }
        class
            .fields
            .iter()
            .any(|(_, ty)| type_has_typevar_dict_key(ty))
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

    pub(crate) fn generic_bounds_for_class(class: &HirClass) -> String {
        if class.name == "deque" {
            return "Clone + PartialEq".to_string();
        }
        if Self::class_needs_hash_eq(class) {
            "Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq".to_string()
        } else {
            "Clone + std::fmt::Display + PartialOrd".to_string()
        }
    }

    /// Convert a Type to its Rust representation, appending generic type params
    /// for classes that are known to be generic (e.g., Counter -> Counter<T>).
    pub(crate) fn rust_type_with_generics(&self, ty: &Type) -> String {
        match ty {
            Type::Int | Type::LiteralInt(_) => "i64".to_string(),
            Type::Float => "f64".to_string(),
            Type::Bool | Type::LiteralBool(_) => "bool".to_string(),
            Type::Str | Type::LiteralStr(_) => "String".to_string(),
            Type::None => "()".to_string(),
            Type::List(inner) => format!("Vec<{}>", self.rust_type_with_generics(inner)),
            Type::Dict(key, value) => format!(
                "HashMap<{}, {}>",
                self.rust_type_with_generics(key),
                self.rust_type_with_generics(value)
            ),
            Type::Set(inner) => format!("HashSet<{}>", self.rust_type_with_generics(inner)),
            Type::Tuple(items) => {
                if let Some((elem, len)) = crate::homogeneous_large_tuple_backing_array(ty) {
                    format!("[{}; {}]", self.rust_type_with_generics(elem), len)
                } else {
                    format!(
                        "({})",
                        items
                            .iter()
                            .map(|item| self.rust_type_with_generics(item))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            Type::Result(ok, err) => format!(
                "Result<{}, {}>",
                self.rust_type_with_generics(ok),
                self.rust_type_with_generics(err)
            ),
            Type::Task(ok, err) => format!(
                "__SifrTask<{}, {}>",
                self.rust_type_with_generics(ok),
                self.rust_generator_error_type_with_generics(err)
            ),
            Type::BlockingTask(ok, err) => format!(
                "__SifrBlockingTask<{}, {}>",
                self.rust_type_with_generics(ok),
                self.rust_generator_error_type_with_generics(err)
            ),
            Type::JoinSet(ok, err) => format!(
                "__SifrJoinSet<{}, {}>",
                self.rust_type_with_generics(ok),
                self.rust_generator_error_type_with_generics(err)
            ),
            Type::TaskResult(ok, err) => format!(
                "__SifrTaskResult<{}, {}>",
                self.rust_type_with_generics(ok),
                self.rust_generator_error_type_with_generics(err)
            ),
            Type::Union(members) => {
                let non_none: Vec<&Type> = members
                    .iter()
                    .filter(|member| !matches!(member, Type::None))
                    .collect();
                let has_none = members.iter().any(|member| matches!(member, Type::None));
                if has_none && non_none.len() == 1 {
                    format!("Option<{}>", self.rust_type_with_generics(non_none[0]))
                } else {
                    ty.rust_type()
                }
            }
            Type::Alias { body, .. } => self.rust_type_with_generics(body),
            Type::Class { name, .. } => self.render_generic_class_type(name, ty),
            Type::Failure(err) => {
                format!("__SifrFailure<{}>", self.rust_type_with_generics(err))
            }
            Type::TimeoutResult(err) => {
                format!("__SifrTimeoutResult<{}>", self.rust_type_with_generics(err))
            }
            Type::Select2(first, second) => format!(
                "__SifrSelect2<{}, {}>",
                self.rust_type_with_generics(first),
                self.rust_type_with_generics(second)
            ),
            Type::AsyncGenerator(item, err) => format!(
                "AsyncGenerator<{}, {}>",
                self.rust_type_with_generics(item),
                self.rust_generator_error_type_with_generics(err)
            ),
            Type::Never => "std::convert::Infallible".to_string(),
            Type::TypeVar(name) => name.clone(),
            Type::Callable(params, conventions, ret) => {
                let param_types = params
                    .iter()
                    .zip(conventions.iter())
                    .map(|(param_ty, convention)| {
                        let rendered = self.rust_type_with_generics(param_ty);
                        if param_ty.ownership() == OwnershipKind::Move && convention.is_borrowed() {
                            if convention.is_mut_borrow() {
                                format!("&mut {rendered}")
                            } else {
                                format!("&{rendered}")
                            }
                        } else {
                            rendered
                        }
                    })
                    .collect::<Vec<_>>();
                let ret_type = self.rust_type_with_generics(ret);
                if ret_type == "()" {
                    format!("impl Fn({})", param_types.join(", "))
                } else {
                    format!("impl Fn({}) -> {}", param_types.join(", "), ret_type)
                }
            }
            _ => ty.rust_type(),
        }
    }

    pub(crate) fn rust_struct_field_type_with_generics(&self, ty: &Type) -> String {
        match ty {
            Type::Callable(params, conventions, ret) => {
                let param_types = params
                    .iter()
                    .zip(conventions.iter())
                    .map(|(param_ty, convention)| {
                        let rendered = self.rust_type_with_generics(param_ty);
                        if param_ty.ownership() == OwnershipKind::Move && convention.is_borrowed() {
                            if convention.is_mut_borrow() {
                                format!("&mut {rendered}")
                            } else {
                                format!("&{rendered}")
                            }
                        } else {
                            rendered
                        }
                    })
                    .collect::<Vec<_>>();
                let ret_type = self.rust_type_with_generics(ret);
                if ret_type == "()" {
                    format!("Box<dyn Fn({})>", param_types.join(", "))
                } else {
                    format!("Box<dyn Fn({}) -> {}>", param_types.join(", "), ret_type)
                }
            }
            _ => self.rust_type_with_generics(ty),
        }
    }

    pub(crate) fn rust_ir_type_with_generics(&self, ty: &Type) -> crate::RustType {
        crate::RustType::Named(self.rust_type_with_generics(ty))
    }

    pub(crate) fn rust_generator_error_type_with_generics(&self, ty: &Type) -> String {
        if matches!(ty.resolve_alias(), Type::Never) {
            "std::convert::Infallible".to_string()
        } else {
            self.rust_type_with_generics(ty)
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
            Type::Callable(params, _, ret) => {
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
                    .map(|arg| self.rust_type_with_generics(arg))
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
            fields, methods, ..
        } = ty
        else {
            return None;
        };
        if template.type_params.is_empty() {
            return None;
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

    pub(crate) fn extra_bounds_for_type_param(tp: &str, body: &[HirStmt]) -> String {
        let requirements =
            crate::hir_analysis::queries::collect_typevar_operator_requirements(body, tp);
        let mut extra = String::new();
        if requirements.needs_add {
            let _ = write!(extra, " + std::ops::Add<Output = {tp}>");
        }
        if requirements.needs_sub {
            let _ = write!(extra, " + std::ops::Sub<Output = {tp}>");
        }
        extra
    }
}
