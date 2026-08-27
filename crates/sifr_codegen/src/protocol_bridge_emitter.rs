use crate::{RustEmitter, RustExpr, RustItem, RustParam, RustStmt, Visibility};
use sifr_ir::{HirClass, HirModule};
use sifr_type_system::{Type, source_class_rust_name};
use std::collections::{HashMap, HashSet};

type OperatorBounds = HashMap<String, HashSet<String>>;

impl RustEmitter {
    pub(crate) fn emit_protocol_impls(&mut self, class: &HirClass, module: &HirModule) {
        for proto_name in &class.implements_protocols {
            let Some(proto_class) = module
                .classes
                .iter()
                .find(|candidate| candidate.name == *proto_name && candidate.is_protocol())
            else {
                continue;
            };

            let mut impl_items = Vec::new();
            for method in &class.methods {
                let Some(protocol_method) = proto_class
                    .methods
                    .iter()
                    .find(|candidate| candidate.name == method.name)
                else {
                    continue;
                };
                let mut params = Vec::with_capacity(method.params.len() + 1);
                params.push(Self::rust_receiver_param(protocol_method));
                let mut call_args = Vec::with_capacity(method.params.len() + 1);
                call_args.push(RustExpr::Ident("self".to_string()));
                for param in &method.params {
                    params.push(RustParam::Named {
                        name: param.name.clone(),
                        ty: crate::sifr_type_to_rust_type(&param.ty),
                    });
                    call_args.push(RustExpr::Ident(param.name.clone()));
                }
                let delegated_call = RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        source_class_rust_name(&class.name),
                        crate::user_callable_rust_name(&method.name),
                    ])),
                    args: call_args,
                };
                impl_items.push(RustItem::Fn {
                    name: crate::user_callable_rust_name(&method.name),
                    visibility: Visibility::Private,
                    type_params: Vec::new(),
                    params,
                    ret: if method.return_type == Type::None {
                        None
                    } else {
                        Some(crate::sifr_type_to_rust_type(&method.return_type))
                    },
                    body: if method.return_type == Type::None {
                        vec![RustStmt::Expr(delegated_call)]
                    } else {
                        vec![RustStmt::Return(Some(delegated_call))]
                    },
                    is_async: false,
                });
            }

            if !impl_items.is_empty() {
                self.body_items.push(RustItem::Impl {
                    target: source_class_rust_name(&class.name),
                    type_params: Vec::new(),
                    trait_: Some(source_class_rust_name(proto_name)),
                    items: impl_items,
                });
            }
        }
    }
}
