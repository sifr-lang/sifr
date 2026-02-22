use crate::helpers::{is_option_type, MUTATING_METHODS};
use crate::RustEmitter;
use sifr_hir::HirExpr;
use sifr_type_system::{ParamConvention, Type};

impl RustEmitter {
    /// Check if an expression is a call to a generator function
    pub(crate) fn is_generator_call(&self, expr: &HirExpr) -> bool {
        if let HirExpr::Call { func, .. } = expr {
            self.generator_functions.contains(func)
        } else {
            false
        }
    }

    pub(crate) fn emit_method_call(&mut self, object: &HirExpr, method: &str, args: &[HirExpr]) {
        // For mutating methods on self.field, suppress .clone() so mutations are applied
        // to the actual field, not a temporary clone.
        let is_self_field = matches!(object, HirExpr::FieldAccess { object: inner, .. }
            if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"));
        if is_self_field && MUTATING_METHODS.contains(&method) {
            self.pending_self_field_clone_suppression += 1;
        }
        let obj_ty = object.ty();
        if self.try_emit_method_via_registry(obj_ty, object, method, args) {
            return;
        }
        match (obj_ty, method) {
            (Type::Set(_), "len") => {
                self.write("(");
                self.emit_expr(object);
                self.write(".len() as i64)");
            }
            // Tuple count()
            (Type::Tuple(_), "count") => {
                // For tuples, count is tricky - we need to check each element
                // For now, emit a simple comparison chain
                self.write("0_i64 /* tuple.count() not fully supported */");
            }
            // Tuple len() - compile-time constant
            (Type::Tuple(elems), "len") => {
                self.write(&format!("{}_i64", elems.len()));
            }
            // String len() - character count
            (Type::Str, "len") => {
                self.write("(");
                self.emit_expr(object);
                self.write(".chars().count() as i64)");
            }
            // len() on Option types (T|None) - unwrap first
            (ty, "len") if is_option_type(ty) => {
                self.write("(");
                self.emit_expr(object);
                self.write(".as_ref().unwrap().len() as i64)");
            }
            // Generic len() for all types
            (_, "len") => {
                self.write("(");
                self.emit_expr(object);
                self.write(".len() as i64)");
            }
            (
                Type::Class {
                    name: ref class_name,
                    fields,
                    methods,
                    ..
                },
                _,
            ) => {
                // Check if this is a callable field invocation (not a real method)
                let is_callable_field = !methods.iter().any(|(n, _)| n == method)
                    && fields
                        .iter()
                        .any(|(n, t)| n == method && matches!(t, Type::Callable(..)));

                if is_callable_field {
                    // Callable field: emit (obj.field)(args) instead of obj.method(args)
                    self.write("(");
                    self.emit_expr(object);
                    self.write(&format!(".{method})("));
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.emit_expr(arg);
                    }
                    self.write(")");
                } else {
                    // Regular class instance method call -- use convention-aware argument emission
                    self.emit_expr(object);
                    self.write(&format!(".{method}("));
                    // Look up method conventions from func_signatures
                    let method_key = format!("{class_name}::{method}");
                    let method_info = self.func_signatures.get(&method_key).cloned();
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        if let Some((ref params, _)) = method_info {
                            // Method params skip self, so param index i corresponds to params[i]
                            // (self is not in func_signatures params)
                            if let Some((param_ty, convention)) = params.get(i) {
                                // For borrowed generic params (&T), wrapping expressions
                                // avoids Rust precedence pitfalls like `&(x) as i64`.
                                // This includes literals which otherwise produce invalid code like `&3_i64`.
                                if *convention == ParamConvention::Borrow
                                    && matches!(param_ty, Type::TypeVar(_))
                                {
                                    self.write("&(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                self.emit_borrow_prefix(*convention, arg.ty(), Some(param_ty));
                                self.emit_expr(arg);
                                continue;
                            }
                        }
                        // Fallback: emit as-is
                        self.emit_expr(arg);
                    }
                    self.write(")");
                }
            }
            _ => {
                // Fallback: emit as-is
                self.emit_expr(object);
                self.write(&format!(".{method}("));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
        }
    }

    /// Emit `&` or `&mut` prefix for a function argument based on parameter convention.
    /// Copy types never get a borrow prefix (they're passed by value),
    /// unless the parameter type is a `TypeVar` (generic), in which case we always borrow.
    pub(crate) fn emit_borrow_prefix(
        &mut self,
        convention: ParamConvention,
        arg_ty: &Type,
        param_ty: Option<&Type>,
    ) {
        self.emit_borrow_prefix_for_name(convention, arg_ty, param_ty, None);
    }

    pub(crate) fn emit_borrow_prefix_for_name(
        &mut self,
        convention: ParamConvention,
        arg_ty: &Type,
        param_ty: Option<&Type>,
        arg_name: Option<&str>,
    ) {
        // Own convention: pass by value (move), no prefix needed
        if convention == ParamConvention::Own {
            return;
        }
        // If the parameter type is a TypeVar, always emit the borrow prefix
        // because the generated Rust signature uses &T for borrowed TypeVar params
        let is_generic_param = param_ty.is_some_and(|t| matches!(t, Type::TypeVar(_)));
        // Copy types are always passed by value regardless of convention,
        // unless the parameter is generic (TypeVar)
        if !is_generic_param && arg_ty.ownership() == sifr_type_system::OwnershipKind::Copy {
            return;
        }
        // If the argument is already a borrowed parameter (&T), don't add another borrow.
        // This handles the case where a Callable call passes a borrowed param:
        //   fn apply(f: Callable[[list[int]], int], items: &Vec<i64>) { f(items) }
        // Here items is already &Vec<i64>, so we pass it as-is (no extra &).
        //
        // Similarly, if the argument is already a mutably borrowed parameter (&mut T),
        // don't add another &mut. E.g.:
        //   fn heapify(data: &mut Vec<i64>) { _sift_down(data, 0, n); }
        // Here data is already &mut Vec<i64>; passing &mut data would be &&mut Vec<i64> error.
        if let Some(name) = arg_name {
            if self.borrowed_params.contains(name) && convention == ParamConvention::Borrow {
                return; // already &T, no additional borrow needed
            }
            if self.mut_borrowed_params.contains(name) {
                if convention == ParamConvention::MutBorrow {
                    return; // already &mut T, no additional &mut needed
                }
                if convention == ParamConvention::Borrow {
                    return; // &mut T -> &T is implicit reborrow in Rust; no extra & needed
                }
            }
        }
        match convention {
            ParamConvention::Borrow => self.write("&"),
            ParamConvention::MutBorrow => self.write("&mut "),
            ParamConvention::Own => {} // no prefix -- pass by value (move)
        }
    }
}
