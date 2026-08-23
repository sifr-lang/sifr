use super::{HirFunction, HirStmt, RustEmitter, RustTypeParam, Type};
use std::collections::HashSet;

impl RustEmitter {
    pub(crate) fn lower_function_type_params(&self, func: &HirFunction) -> Vec<RustTypeParam> {
        if func.type_params.is_empty() {
            return Vec::new();
        }
        let needs_hash_eq = Self::func_needs_hash_eq(func);
        let context_type_params = self.context_type_params.get(&func.name);
        let context_type_param = func.type_params.iter().find(|type_param| {
            context_type_params.is_some_and(|params| params.contains(*type_param))
        });
        let context_is_shared = context_type_param.is_some_and(|context| {
            func.params.iter().any(|param| {
                matches!(param.ty.resolve_alias(), Type::TypeVar(name) if name == context)
                    && param.convention.is_shared_borrow()
            })
        });
        func.type_params
            .iter()
            .map(|type_param| {
                let string_structural = self
                    .string_structural_type_params
                    .get(&func.name)
                    .is_some_and(|params| params.contains(type_param));
                let structural = self
                    .structural_type_params
                    .get(&func.name)
                    .is_some_and(|params| params.contains(type_param))
                    || func.rust_interop.iter().any(|declaration| {
                        declaration.kind == sifr_ir::RustInteropDecoratorKind::Structural
                    });
                let static_program = self
                    .static_program_type_params
                    .get(&func.name)
                    .is_some_and(|params| params.contains(type_param));
                let attached_api = func.decorators.iter().any(|item| item == "attached_api");
                let attached_storage = attached_api || Self::is_nullcontext_value_forwarder(func);
                let method_slots = self
                    .method_slot_type_params
                    .get(&func.name)
                    .is_some_and(|params| params.contains(type_param));
                let context = self
                    .context_type_params
                    .get(&func.name)
                    .is_some_and(|params| params.contains(type_param));
                let mut base = if context {
                    "sifr_runtime::interop::structural::StructuralType".to_string()
                } else if structural && static_program {
                    "sifr_runtime::interop::structural::StructuralConstruct + sifr_runtime::interop::structural::StructuralProject + sifr_runtime::interop::structural::StaticProgramType"
                        .to_string()
                } else if static_program {
                    "sifr_runtime::interop::structural::StaticProgramType + Clone".to_string()
                } else if string_structural {
                    "sifr_runtime::interop::structural::StructuralConstruct + sifr_runtime::interop::structural::StructuralProject + Clone + 'static".to_string()
                } else if structural {
                    "sifr_runtime::interop::structural::StructuralConstruct + sifr_runtime::interop::structural::StructuralProject".to_string()
                } else if attached_storage {
                    "Clone + 'static".to_string()
                } else if needs_hash_eq {
                    "Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq + 'static".to_string()
                } else {
                    "Clone + std::fmt::Display + PartialOrd + 'static".to_string()
                };
                if Self::returns_borrowed_type_param(func, type_param) {
                    Self::append_bound(&mut base, "Clone");
                }
                if attached_storage {
                    Self::append_bound(&mut base, "Clone");
                    Self::append_bound(&mut base, "'static");
                }
                let base = if method_slots {
                    match context_type_param {
                        Some(context) if context_is_shared => format!(
                            "{base} + for<'__sifr_context> sifr_runtime::interop::structural::MethodSlotTable<sifr_runtime::interop::structural::SharedContext<'__sifr_context, {context}>>"
                        ),
                        Some(context) => format!(
                            "{base} + sifr_runtime::interop::structural::MethodSlotTable<{context}>"
                        ),
                        None => format!(
                            "{base} + sifr_runtime::interop::structural::MethodSlotTable<sifr_runtime::interop::structural::NoContext>"
                        ),
                    }
                } else {
                    base
                };
                let extra = Self::extra_bound_items_for_type_param(type_param, &func.body)
                    .into_iter()
                    .filter(|bound| !base.split(" + ").any(|existing| existing == bound))
                    .fold(String::new(), |mut rendered, bound| {
                        use std::fmt::Write as _;
                        let _ = write!(rendered, " + {bound}");
                        rendered
                    });
                RustTypeParam {
                    name: type_param.clone(),
                    bounds: vec![format!("{base}{extra}")],
                }
            })
            .collect()
    }

    fn is_nullcontext_value_forwarder(func: &HirFunction) -> bool {
        if func.name != "nullcontext" || func.type_params.is_empty() {
            return false;
        }
        if !matches!(&func.return_type, Type::Class { name, .. } if name == "NullContext") {
            return false;
        }
        let [
            HirStmt::Return {
                value:
                    Some(crate::HirExpr::ConstructorCall {
                        class_name, args, ..
                    }),
            },
        ] = func.body.as_slice()
        else {
            return false;
        };
        class_name == "NullContext"
            && args.iter().all(|arg| {
                matches!(arg, crate::HirExpr::Name { name, .. } if func.params.iter().any(|param| param.name == *name))
            })
    }

    fn returns_borrowed_type_param(func: &HirFunction, type_param: &str) -> bool {
        let borrowed = func
            .params
            .iter()
            .filter(|param| {
                param.convention.is_borrowed()
                    && Self::type_mentions_type_param(&param.ty, type_param)
            })
            .map(|param| param.name.as_str())
            .collect::<HashSet<_>>();
        if borrowed.is_empty() {
            return false;
        }
        let mut returned = false;
        let mut on_stmt = |stmt: &HirStmt| {
            if let HirStmt::Return {
                value: Some(crate::HirExpr::Name { name, .. }),
            } = stmt
            {
                returned |= borrowed.contains(name.as_str());
            }
        };
        let mut on_expr = |_expr: &crate::HirExpr| {};
        crate::hir_analysis::traversal::walk_stmts(
            &func.body,
            crate::hir_analysis::traversal::TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        returned
    }

    fn append_bound(bounds: &mut String, bound: &str) {
        if bounds.split(" + ").any(|existing| existing == bound) {
            return;
        }
        bounds.push_str(" + ");
        bounds.push_str(bound);
    }
}
