use super::{RustEmitter, Type};

impl RustEmitter {
    pub(crate) fn adapt_consuming_call_argument_for_ir(
        &self,
        target_ty: &Type,
        source_ty: &Type,
        lowered: crate::RustExpr,
        borrowed_source: bool,
    ) -> (crate::RustExpr, bool) {
        let probe = crate::RustExpr::Ident("__sifr_consuming_upcast_probe".to_string());
        if self.consuming_value_conversion_for_ir(target_ty, source_ty, probe.clone()) == probe {
            return (lowered, false);
        }
        let lowered = if borrowed_source && !crate::helpers::is_copy_type_for_codegen(source_ty) {
            crate::ownership_plan::materialize_owned_value(source_ty, lowered)
        } else {
            lowered
        };
        (
            self.consuming_value_conversion_for_ir(target_ty, source_ty, lowered),
            true,
        )
    }

    /// Convert between the source and target runtime representations exactly once.
    ///
    /// Optional-wrapper adaptation and recursive union upcasting overlap for an
    /// `Option<Union>` source. Keep their source/target representation pair under
    /// one authority so a caller cannot apply both transitions to the same value.
    pub(crate) fn consuming_value_conversion_for_ir(
        &self,
        target_ty: &Type,
        source_ty: &Type,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        let wrapper_adapted =
            crate::helpers::flatten_option_value_for_target(target_ty, source_ty, lowered.clone());
        if wrapper_adapted != lowered {
            return wrapper_adapted;
        }
        self.consuming_value_upcast_for_ir(target_ty, source_ty, lowered)
    }

    fn consuming_value_upcast_for_ir(
        &self,
        target_ty: &Type,
        source_ty: &Type,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        let target = crate::resolve_alias_type_for_plain_call(target_ty);
        let source = crate::resolve_alias_type_for_plain_call(source_ty);
        if target == source {
            return lowered;
        }

        if let (Type::Union(target_members), Some(source_inner)) =
            (target, Self::option_inner_type_for_ir(source_ty))
        {
            if !crate::helpers::is_option_type(target_ty)
                && source_ty.is_assignable_to(target_ty)
                && target_members
                    .iter()
                    .any(|member| matches!(member.resolve_alias(), Type::None))
            {
                let binding = "__sifr_option_value";
                let present = self.consuming_value_upcast_for_ir(
                    target_ty,
                    &source_inner,
                    crate::RustExpr::Ident(binding.to_string()),
                );
                let mapped = map_value(lowered, "map", binding, present);
                return crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(mapped))),
                    method: "unwrap_or".to_string(),
                    args: vec![crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            target.union_enum_name(),
                            Type::None.union_variant_name(),
                        ])),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                    }],
                };
            }
        }

        if let Some(target_inner) = Self::option_inner_type_for_ir(target_ty) {
            if let Some(source_inner) = Self::option_inner_type_for_ir(source_ty) {
                let value = crate::RustExpr::Ident("__sifr_option_value".to_string());
                let converted =
                    self.consuming_value_upcast_for_ir(&target_inner, &source_inner, value.clone());
                if converted == value {
                    return lowered;
                }
                return map_value(lowered, "map", "__sifr_option_value", converted);
            }
            // The ordinary option adapter wraps the converted payload in Some.
            return self.consuming_value_upcast_for_ir(&target_inner, source_ty, lowered);
        }

        if let (Type::Result(source_ok, source_error), Type::Result(target_ok, target_error)) =
            (source, target)
        {
            let ok_value = crate::RustExpr::Ident("__sifr_ok_value".to_string());
            let converted_ok =
                self.consuming_value_upcast_for_ir(target_ok, source_ok, ok_value.clone());
            let mut converted = if converted_ok == ok_value {
                lowered
            } else {
                map_value(lowered, "map", "__sifr_ok_value", converted_ok)
            };

            let error_value = crate::RustExpr::Ident("__sifr_error_value".to_string());
            let converted_error =
                self.consuming_value_upcast_for_ir(target_error, source_error, error_value.clone());
            if converted_error != error_value {
                converted = map_value(converted, "map_err", "__sifr_error_value", converted_error);
            }
            return converted;
        }

        if let (Type::Union(source_members), Type::Union(target_members)) = (source, target) {
            let mut arms = Vec::with_capacity(source_members.len());
            for source_member in source_members {
                let Some(target_member) =
                    crate::helpers::find_union_member(target_members, source_member)
                else {
                    return lowered;
                };
                let source_is_none = matches!(source_member.resolve_alias(), Type::None);
                let binding = if source_is_none {
                    "_"
                } else {
                    "__sifr_union_value"
                };
                let converted_member = if source_is_none {
                    crate::RustExpr::Literal(crate::RustLiteral::Unit)
                } else {
                    self.consuming_value_upcast_for_ir(
                        target_member,
                        source_member,
                        crate::RustExpr::Ident(binding.to_string()),
                    )
                };
                let wrapped = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        target.union_enum_name(),
                        target_member.union_variant_name(),
                    ])),
                    args: vec![converted_member],
                };
                arms.push(crate::RustMatchArm {
                    pattern: format!(
                        "{}::{}({binding})",
                        source.union_enum_name(),
                        source_member.union_variant_name()
                    ),
                    bindings: Vec::new(),
                    guard: None,
                    body: vec![crate::RustStmt::TailExpr(wrapped)],
                });
            }
            return crate::RustExpr::Match {
                expr: Box::new(lowered),
                arms,
            };
        }

        if let Type::Union(target_members) = target {
            if let Some(target_member) =
                crate::helpers::find_union_member(target_members, source_ty)
            {
                let converted =
                    self.consuming_value_upcast_for_ir(target_member, source_ty, lowered);
                let converted = if matches!(target_member.resolve_alias(), Type::None) {
                    crate::RustExpr::Literal(crate::RustLiteral::Unit)
                } else {
                    converted
                };
                return crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        target.union_enum_name(),
                        target_member.union_variant_name(),
                    ])),
                    args: vec![converted],
                };
            }
        }

        self.consuming_class_upcast_for_ir(target_ty, source_ty, lowered)
    }

    pub(crate) fn consuming_class_upcast_for_ir(
        &self,
        target_ty: &Type,
        source_ty: &Type,
        mut lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        let target_inner = Self::option_inner_type_for_ir(target_ty);
        let target_ty = target_inner.as_ref().unwrap_or(target_ty);
        let (
            Type::Class {
                identity: source_identity,
                name: source_name,
                parent_class: Some(parent_chain),
                ..
            },
            Type::Class {
                identity: target_identity,
                name: target_name,
                ..
            },
        ) = (
            crate::resolve_alias_type_for_plain_call(source_ty),
            crate::resolve_alias_type_for_plain_call(target_ty),
        )
        else {
            return lowered;
        };
        let source_identity = source_identity.as_ref().unwrap_or(source_name);
        let target_identity = target_identity.as_ref().unwrap_or(target_name);
        if source_identity == target_identity {
            return lowered;
        }
        let ancestors = parent_chain.split('|').collect::<Vec<_>>();
        let Some(target_index) = ancestors
            .iter()
            .position(|ancestor| *ancestor == target_identity)
        else {
            return lowered;
        };
        // Root Error has a direct consuming conversion from each concrete error.
        // A data ancestor need not itself be an error (an explicit Error marker
        // may be introduced by the child), so never discard the child's message
        // by first converting it to that data ancestor.
        if target_ty.is_builtin_error_base() {
            return crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "::std::convert::Into::<Error>::into".to_string(),
                ])),
                args: vec![lowered],
            };
        }
        for (index, ancestor) in ancestors.iter().take(target_index + 1).enumerate() {
            let rendered_target = if index == target_index {
                self.render_rust_type_with_generics(target_ty)
            } else {
                render_ancestor_rust_type(self.current_module_name.as_deref(), ancestor)
            };
            lowered = crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Verbatim(format!(
                    "::std::convert::Into::<{rendered_target}>::into"
                ))),
                args: vec![lowered],
            };
        }
        lowered
    }
}

#[cfg(test)]
mod representation_tests {
    use super::*;

    fn render_conversion(target: &Type, source: &Type) -> String {
        let emitter = RustEmitter::new();
        crate::render_expr(&emitter.consuming_value_conversion_for_ir(
            target,
            source,
            crate::RustExpr::Ident("value".to_string()),
        ))
    }

    #[test]
    fn representation_conversion_flattens_nested_option_once() {
        let target = sifr_type_system::make_union(vec![Type::Str, Type::None]);
        let source = Type::Union(vec![target.clone(), Type::None]);
        let rendered = render_conversion(&target, &source);

        assert_eq!(rendered.matches(".flatten(").count(), 1, "{rendered}");
        assert_eq!(rendered.matches(".map(").count(), 0, "{rendered}");
        assert_eq!(rendered.matches(".unwrap_or(").count(), 0, "{rendered}");
    }

    #[test]
    fn representation_conversion_maps_option_union_once() {
        let payload = sifr_type_system::make_union(vec![Type::Int, Type::Str]);
        let source = sifr_type_system::safe_optional_result(payload);
        let target =
            sifr_type_system::make_union(vec![Type::Bool, Type::Int, Type::Str, Type::None]);
        let rendered = render_conversion(&target, &source);

        assert_eq!(rendered.matches(".map(").count(), 1, "{rendered}");
        assert_eq!(rendered.matches(".unwrap_or(").count(), 1, "{rendered}");
    }

    #[test]
    fn representation_conversion_materializes_nullable_union_once() {
        let target = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
        let source = sifr_type_system::safe_optional_result(target.clone());
        let rendered = render_conversion(&target, &source);

        assert_eq!(rendered.matches(".map(").count(), 0, "{rendered}");
        assert_eq!(rendered.matches(".unwrap_or(").count(), 1, "{rendered}");
    }
}

fn render_ancestor_rust_type(current_module: Option<&str>, ancestor: &str) -> String {
    let Some((module, name)) = ancestor.rsplit_once('.') else {
        return sifr_type_system::source_class_rust_name(ancestor);
    };
    if current_module == Some(module) {
        return sifr_type_system::source_class_rust_name(name);
    }
    if module.starts_with("sifr.") || module.starts_with("_sifr.") {
        return sifr_type_system::stdlib_class_rust_name(module, name);
    }
    format!(
        "crate::{}::{}",
        module.replace('.', "::"),
        sifr_type_system::source_class_rust_name(name)
    )
}

fn map_value(
    receiver: crate::RustExpr,
    method: &str,
    binding: &str,
    body: crate::RustExpr,
) -> crate::RustExpr {
    crate::RustExpr::MethodCall {
        receiver: Box::new(crate::RustExpr::Paren(Box::new(receiver))),
        method: method.to_string(),
        args: vec![crate::RustExpr::Closure {
            params: vec![crate::RustParam::Named {
                name: binding.to_string(),
                ty: crate::RustType::Named("_".to_string()),
            }],
            body: Box::new(body),
            is_move: false,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::render_ancestor_rust_type;
    use crate::{RustEmitter, RustExpr};
    use sifr_type_system::Type;

    fn class(identity: &str, name: &str, parent_class: Option<&str>) -> Type {
        Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: parent_class.map(str::to_string),
        }
    }

    #[test]
    fn basename_only_ancestor_match_does_not_emit_an_upcast() {
        let source = class("pkg.Child", "Child", Some("pkg.Root"));
        let unrelated_root = class("other.Root", "Root", None);
        let value = RustExpr::Ident("value".to_string());

        assert_eq!(
            RustEmitter::new().consuming_value_conversion_for_ir(
                &unrelated_root,
                &source,
                value.clone(),
            ),
            value
        );
    }

    #[test]
    fn ancestor_paths_are_local_stdlib_or_crate_rooted_by_identity() {
        assert_eq!(
            render_ancestor_rust_type(Some("models"), "models.Mid"),
            "Mid"
        );
        assert_eq!(
            render_ancestor_rust_type(Some("adapter"), "models.Mid"),
            "crate::models::Mid"
        );
        assert_eq!(
            render_ancestor_rust_type(Some("adapter"), "sifr.resource.NullContext"),
            sifr_type_system::stdlib_class_rust_name("sifr.resource", "NullContext")
        );
    }
}
