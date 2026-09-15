use crate::{RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};
use sifr_type_system::Type;

mod comprehension;
pub(crate) use comprehension::materialize_comprehension_value;

impl crate::RustEmitter {
    /// Produces a `&str` view without calling the unstable `str::as_str` on a
    /// shared string parameter, whose Rust ABI is already `&str`.
    pub(crate) fn string_view_expr(
        &self,
        source: &sifr_ir::HirExpr,
        lowered: RustExpr,
    ) -> RustExpr {
        if matches!(
            source,
            sifr_ir::HirExpr::Name { name, ty, .. }
                if self.borrowed_params.contains(name)
                    && matches!(ty.resolve_alias(), Type::Str | Type::LiteralStr(_))
        ) {
            return lowered;
        }
        let lowered = match lowered {
            RustExpr::Ref { .. } | RustExpr::Deref(_) => RustExpr::Paren(Box::new(lowered)),
            other => other,
        };
        method_call(lowered, "as_str")
    }

    pub(crate) fn recursive_option_borrowed_type(&self, ty: &Type) -> Option<RustType> {
        let inner = ty.optional_member_type()?;
        let Type::Class { name, .. } = inner.resolve_alias() else {
            return None;
        };
        if !self
            .recursive_fields
            .iter()
            .any(|(class_name, _)| class_name == name)
        {
            return None;
        }
        Some(RustType::Option(Box::new(RustType::Ref {
            mutable: false,
            inner: Box::new(self.rust_ir_type_with_generics(&inner)),
        })))
    }

    pub(crate) fn expr_is_recursive_option_borrowed_view(&self, expr: &sifr_ir::HirExpr) -> bool {
        match expr {
            sifr_ir::HirExpr::Name { name, ty, .. } => {
                self.recursive_option_borrowed_views.contains(name)
                    || (self.recursive_option_borrowed_type(ty).is_some()
                        && (self.borrowed_params.contains(name)
                            || self.mut_borrowed_params.contains(name)))
            }
            sifr_ir::HirExpr::FieldAccess { object, field, ty } => {
                if self.recursive_option_borrowed_type(ty).is_none() {
                    return false;
                }
                let object_is_borrowed = matches!(
                    object.as_ref(),
                    sifr_ir::HirExpr::Name { name, .. }
                        if self.borrowed_params.contains(name)
                            || self.mut_borrowed_params.contains(name)
                            || self.recursive_option_borrowed_views.contains(name)
                );
                let class_name = object
                    .ty()
                    .optional_member_type()
                    .unwrap_or_else(|| object.ty().clone());
                matches!(class_name.resolve_alias(), Type::Class { name, .. }
                    if object_is_borrowed
                        && self.recursive_fields.contains(&(name.clone(), field.clone())))
            }
            _ => false,
        }
    }

    pub(crate) fn adapt_recursive_option_borrowed_argument(
        &self,
        param_ty: &Type,
        convention: sifr_type_system::ParamConvention,
        arg: &sifr_ir::HirExpr,
        effective_arg_ty: &Type,
        lowered: RustExpr,
    ) -> Option<RustExpr> {
        if !convention.is_shared_borrow() || self.recursive_option_borrowed_type(param_ty).is_none()
        {
            return None;
        }
        if matches!(arg, sifr_ir::HirExpr::NoneLiteral) {
            return Some(lowered);
        }
        if crate::helpers::is_option_type(effective_arg_ty) {
            return Some(if self.expr_is_recursive_option_borrowed_view(arg) {
                lowered
            } else {
                method_call(lowered, "as_ref")
            });
        }
        let already_borrowed = matches!(
            arg,
            sifr_ir::HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name)
                    || self.recursive_option_borrowed_views.contains(name)
        );
        let inner = if already_borrowed {
            lowered
        } else {
            RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered),
            }
        };
        Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![inner],
        })
    }
}

/// The single codegen decision for turning a source value into owned Rust storage.
///
/// Borrowed `str` and sequence parameters are unsized views. Calling `clone()` on
/// either view only copies the reference, so ownership boundaries must use this
/// plan instead of constructing clone expressions ad hoc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnedMaterialization {
    ToOwned,
    ToVec,
    Clone,
}

pub(crate) fn shared_borrowed_param_type(ty: &Type, owned: RustType) -> RustType {
    let inner = match ty.resolve_alias() {
        Type::Str | Type::LiteralStr(_) => RustType::Str,
        Type::Bytes => RustType::Slice(Box::new(RustType::Named("u8".to_string()))),
        Type::List(_) | Type::Iterable(_) => match owned {
            RustType::Vec(element) => RustType::Slice(element),
            other => other,
        },
        _ => owned,
    };
    RustType::Ref {
        mutable: false,
        inner: Box::new(inner),
    }
}

pub(crate) fn owned_materialization(ty: &Type) -> OwnedMaterialization {
    match ty.resolve_alias() {
        Type::Str | Type::LiteralStr(_) => OwnedMaterialization::ToOwned,
        Type::Bytes | Type::List(_) | Type::Iterable(_) => OwnedMaterialization::ToVec,
        _ => OwnedMaterialization::Clone,
    }
}

pub(crate) fn materialize_owned_value(ty: &Type, value: RustExpr) -> RustExpr {
    let borrowed_view = match value {
        RustExpr::Clone(inner) => *inner,
        RustExpr::MethodCall {
            receiver,
            method,
            args,
        } if method == "clone" && args.is_empty() => *receiver,
        other => other,
    };
    match owned_materialization(ty) {
        OwnedMaterialization::ToOwned => method_call(borrowed_view, "to_owned"),
        OwnedMaterialization::ToVec => method_call(borrowed_view, "to_vec"),
        OwnedMaterialization::Clone => clone_once(borrowed_view),
    }
}

pub(crate) fn materialize_borrowed_option_value(value: RustExpr) -> RustExpr {
    let borrowed_option = match value {
        RustExpr::Clone(inner) => *inner,
        RustExpr::MethodCall {
            receiver,
            method,
            args,
        } if method == "clone" && args.is_empty() => *receiver,
        other => other,
    };
    method_call(borrowed_option, "cloned")
}

pub(crate) fn clone_once(value: RustExpr) -> RustExpr {
    match value {
        RustExpr::Clone(_) => value,
        RustExpr::MethodCall {
            ref method,
            ref args,
            ..
        } if method == "clone" && args.is_empty() => value,
        other => RustExpr::Clone(Box::new(other)),
    }
}

fn method_call(value: RustExpr, method: &str) -> RustExpr {
    if matches!(
        &value,
        RustExpr::MethodCall {
            method: existing,
            args,
            ..
        } if existing == method && args.is_empty()
    ) {
        return value;
    }
    RustExpr::MethodCall {
        receiver: Box::new(value),
        method: method.to_string(),
        args: Vec::new(),
    }
}

pub(crate) fn addable_support_items() -> Vec<RustItem> {
    let trait_item = RustItem::Trait {
        name: "__SifrAdd".to_string(),
        visibility: Visibility::Pub,
        supertraits: vec!["Sized".to_string()],
        methods: vec![RustItem::TraitMethodSig {
            name: "__sifr_add".to_string(),
            params: vec![
                RustParam::SelfValue,
                RustParam::Named {
                    name: "rhs".to_string(),
                    ty: RustType::Named("Self".to_string()),
                },
            ],
            ret: Some(RustType::Named("Self".to_string())),
            is_async: false,
        }],
    };
    let arithmetic_impl = |target: &str| RustItem::Impl {
        target: target.to_string(),
        type_params: Vec::new(),
        trait_: Some("__SifrAdd".to_string()),
        items: vec![RustItem::Fn {
            name: "__sifr_add".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: vec![
                RustParam::SelfValue,
                RustParam::Named {
                    name: "rhs".to_string(),
                    ty: RustType::Named("Self".to_string()),
                },
            ],
            ret: Some(RustType::Named("Self".to_string())),
            body: vec![RustStmt::Return(Some(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("self".to_string())),
                op: "+".to_string(),
                right: Box::new(RustExpr::Ident("rhs".to_string())),
            }))],
            is_async: false,
        }],
    };
    let string_impl = RustItem::Impl {
        target: "String".to_string(),
        type_params: Vec::new(),
        trait_: Some("__SifrAdd".to_string()),
        items: vec![RustItem::Fn {
            name: "__sifr_add".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: vec![
                RustParam::MutableSelfValue,
                RustParam::Named {
                    name: "rhs".to_string(),
                    ty: RustType::Named("Self".to_string()),
                },
            ],
            ret: Some(RustType::Named("Self".to_string())),
            body: vec![
                RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("self".to_string())),
                    method: "push_str".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("rhs".to_string())),
                    }],
                }),
                RustStmt::Return(Some(RustExpr::Ident("self".to_string()))),
            ],
            is_async: false,
        }],
    };
    vec![
        trait_item,
        arithmetic_impl("sifr_runtime::SifrInt"),
        arithmetic_impl("f64"),
        string_impl,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_views_and_owned_materialization_are_structural() {
        assert_eq!(
            shared_borrowed_param_type(&Type::Str, RustType::String_),
            RustType::Ref {
                mutable: false,
                inner: Box::new(RustType::Str),
            }
        );
        assert_eq!(
            shared_borrowed_param_type(
                &Type::List(Box::new(Type::Bool)),
                RustType::Vec(Box::new(RustType::Bool)),
            ),
            RustType::Ref {
                mutable: false,
                inner: Box::new(RustType::Slice(Box::new(RustType::Bool))),
            }
        );

        let string = materialize_owned_value(&Type::Str, RustExpr::Ident("value".to_string()));
        assert!(matches!(
            string,
            RustExpr::MethodCall { method, .. } if method == "to_owned"
        ));
        let list = materialize_owned_value(
            &Type::List(Box::new(Type::Bool)),
            RustExpr::Ident("values".to_string()),
        );
        assert!(matches!(
            list,
            RustExpr::MethodCall { method, .. } if method == "to_vec"
        ));

        let once = materialize_owned_value(&Type::Str, RustExpr::Ident("value".to_string()));
        assert_eq!(materialize_owned_value(&Type::Str, once.clone()), once);
        assert_eq!(
            materialize_owned_value(
                &Type::Str,
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("value".to_string())),
                    method: "clone".to_string(),
                    args: Vec::new(),
                },
            ),
            once,
        );
    }

    #[test]
    fn clone_plan_never_compounds_an_existing_clone() {
        let once = RustExpr::Clone(Box::new(RustExpr::Ident("value".to_string())));
        assert_eq!(clone_once(once.clone()), once);
    }
}
