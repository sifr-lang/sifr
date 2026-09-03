use super::{
    HirFunction, HirParam, ParamConvention, RustEmitter, RustType, Type, is_result_int_type,
    result_int_return_type_to_sifr_int,
};

impl RustEmitter {
    pub(crate) fn returns_result_none(ty: &Type) -> bool {
        match crate::resolve_alias_type_for_plain_call(ty) {
            Type::Result(ok_ty, _) => matches!(
                crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                Type::None
            ),
            _ => false,
        }
    }

    pub(crate) fn lower_function_param_type(
        &self,
        ty: &Type,
        convention: ParamConvention,
    ) -> RustType {
        if convention.is_shared_borrow()
            && let Type::StructuralRecord(record) = ty.resolve_alias()
        {
            return RustType::Ref {
                mutable: false,
                inner: Box::new(RustType::ImplTrait {
                    trait_: crate::RustTrait::Named {
                        name: crate::structural_identity_codegen::structural_record_view_trait_name(
                            record,
                        ),
                        params: record
                            .fields()
                            .iter()
                            .map(|field| crate::sifr_type_to_rust_type(field.ty()))
                            .collect(),
                        associated_types: Vec::new(),
                    },
                    auto_traits: Vec::new(),
                }),
            };
        }
        let base = self.rust_ir_type_with_generics(ty);
        if convention.is_shared_borrow()
            && let Some(view) = self.recursive_option_borrowed_type(ty)
        {
            return view;
        }
        if convention.is_shared_borrow()
            && (!crate::helpers::is_copy_type_for_codegen(ty)
                || matches!(
                    ty.resolve_alias(),
                    Type::Callable(..) | Type::AsyncCallable(..)
                ))
        {
            crate::ownership_plan::shared_borrowed_param_type(ty, base)
        } else if convention.is_mut_borrow()
            && (!crate::helpers::is_copy_type_for_codegen(ty)
                || matches!(ty.resolve_alias(), Type::TypeVar(_) | Type::Any))
        {
            RustType::Ref {
                mutable: true,
                inner: Box::new(base),
            }
        } else {
            base
        }
    }

    pub(crate) fn lower_python_callback_param_type(
        &self,
        ty: &Type,
        convention: ParamConvention,
        require_static: bool,
    ) -> RustType {
        let resolved = ty.resolve_alias();
        if !matches!(resolved, Type::Callable(..) | Type::AsyncCallable(..)) {
            return self.lower_function_param_type(ty, convention);
        }
        let mut bounded = self.rust_ir_type_with_generics(ty);
        if let RustType::ImplTrait { auto_traits, .. } | RustType::DynTrait { auto_traits, .. } =
            &mut bounded
        {
            if !matches!(resolved, Type::AsyncCallable(..)) {
                auto_traits.extend(["Send".to_string(), "Sync".to_string()]);
            }
            if require_static {
                auto_traits.push("'static".to_string());
            }
        }
        if !crate::helpers::is_copy_type_for_codegen(ty) && convention.is_borrowed() {
            RustType::Ref {
                mutable: convention.is_mut_borrow(),
                inner: Box::new(bounded),
            }
        } else {
            bounded
        }
    }

    pub(crate) fn lower_module_function_param_type(
        &self,
        func_name: &str,
        param_idx: usize,
        param: &HirParam,
    ) -> RustType {
        if matches!(func_name, "py_local_callback" | "py_threadsafe_callback")
            && matches!(param.ty.resolve_alias(), Type::Callable(..))
        {
            return self.lower_python_callback_param_type(&param.ty, param.convention, true);
        }
        if self.function_param_lowers_to_sifr_int(func_name, param_idx)
            && matches!(
                crate::resolve_alias_type_for_plain_call(&param.ty),
                Type::Int
            )
        {
            return self.lower_function_param_type(&param.ty, param.convention);
        }
        if self.function_param_lowers_to_sifr_int_result(func_name, param_idx)
            && is_result_int_type(&param.ty)
        {
            return result_int_return_type_to_sifr_int(&param.ty);
        }
        self.lower_function_param_type(&param.ty, param.convention)
    }

    pub(crate) fn lower_function_return_type(
        &self,
        func: &HirFunction,
        is_generator: bool,
    ) -> Option<RustType> {
        if is_generator {
            return Some(self.rust_ir_type_with_generics(&func.return_type));
        }

        if func.return_type == Type::None {
            return None;
        }
        if self.function_returns_sifr_int(&func.name) {
            return Some(RustType::Named("SifrInt".to_string()));
        }
        if self
            .sifr_int_result_function_returns
            .borrow()
            .contains(&func.name)
        {
            return Some(result_int_return_type_to_sifr_int(&func.return_type));
        }
        Some(self.rust_ir_type_with_generics(&func.return_type))
    }
}
