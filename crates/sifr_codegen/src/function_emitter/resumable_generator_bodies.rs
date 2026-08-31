use super::{HirFunction, RustEmitter, RustExpr, RustParam, RustStmt, RustType, Type};

impl RustEmitter {
    fn prepare_resumable_generator_parameters(
        func: &HirFunction,
        mutable_param_shadows: &[(String, RustExpr)],
    ) -> (Vec<RustStmt>, Vec<String>) {
        let mut body = Self::emit_mutable_param_shadow_stmts(mutable_param_shadows);
        let cloned_borrowed_params = func
            .params
            .iter()
            .filter(|param| {
                !mutable_param_shadows
                    .iter()
                    .any(|(name, _)| name == &param.name)
                    && param.convention.is_borrowed()
                    && !crate::helpers::is_copy_type_for_codegen(&param.ty)
            })
            .map(|param| param.name.clone())
            .collect::<Vec<_>>();
        for name in &cloned_borrowed_params {
            body.push(RustStmt::Let {
                mutable: false,
                name: name.clone(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(name.clone()))),
            });
        }
        (body, cloned_borrowed_params)
    }

    fn lower_resumable_generator_statements(
        &mut self,
        func: &HirFunction,
        cloned_borrowed_params: &[String],
        closure_return_type: Type,
    ) -> Vec<RustStmt> {
        let saved_borrowed_params = self.borrowed_params.clone();
        let saved_mut_borrowed_params = self.mut_borrowed_params.clone();
        let saved_return_type = self.current_return_type.replace(closure_return_type);
        let saved_generator_closure = self.emission_ctx.in_generator_closure;
        self.emission_ctx.in_generator_closure = true;
        for name in cloned_borrowed_params {
            self.borrowed_params.remove(name);
            self.mut_borrowed_params.remove(name);
        }

        let mut lowered = Vec::new();
        for (stmt_index, stmt) in func.body.iter().enumerate() {
            lowered.extend(self.lower_stmt_strict_for_function_with_following(
                stmt,
                Some(&func.body[stmt_index + 1..]),
                "resumable generator statement lowering",
            ));
        }

        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        self.current_return_type = saved_return_type;
        self.emission_ctx.in_generator_closure = saved_generator_closure;
        lowered
    }

    pub(crate) fn lower_resumable_generator_function_body(
        &mut self,
        func: &HirFunction,
        mutable_param_shadows: &[(String, RustExpr)],
    ) -> Vec<RustStmt> {
        let Type::Iterator(yield_type) = func.return_type.resolve_alias() else {
            panic!(
                "sync generator must have Iterator return type: {}",
                func.name
            );
        };
        let yield_type = self.rust_ir_type_with_generics(yield_type);
        let (mut body, cloned_borrowed_params) =
            Self::prepare_resumable_generator_parameters(func, mutable_param_shadows);
        let producer_body =
            self.lower_resumable_generator_statements(func, &cloned_borrowed_params, Type::None);
        let factory = RustExpr::ClosureBlock {
            params: vec![RustParam::Named {
                name: "__sifr_yielder".to_string(),
                ty: RustType::Generic {
                    base: "__SifrYielder".to_string(),
                    params: vec![yield_type],
                },
            }],
            body: producer_body,
            is_move: true,
            is_async: true,
        };
        let generator = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "__SifrGenerator".to_string(),
                "new".to_string(),
            ])),
            args: vec![factory],
        };
        body.push(RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
            args: vec![generator],
        })));
        body
    }

    pub(crate) fn lower_resumable_async_generator_function_body(
        &mut self,
        func: &HirFunction,
        mutable_param_shadows: &[(String, RustExpr)],
    ) -> Vec<RustStmt> {
        let Type::AsyncGenerator(yield_type, error_type) = func.return_type.resolve_alias() else {
            panic!(
                "async generator must have AsyncGenerator return type: {}",
                func.name
            );
        };
        let rust_yield_type = self.rust_ir_type_with_generics(yield_type);
        let closure_return_type = Type::Result(Box::new(Type::None), error_type.clone());
        let (mut body, cloned_borrowed_params) =
            Self::prepare_resumable_generator_parameters(func, mutable_param_shadows);
        let mut producer_body = self.lower_resumable_generator_statements(
            func,
            &cloned_borrowed_params,
            closure_return_type,
        );
        if !crate::hir_analysis::queries::block_control_flow_effect(&func.body).always_exits() {
            producer_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
            })));
        }
        let factory = RustExpr::ClosureBlock {
            params: vec![RustParam::Named {
                name: "__sifr_yielder".to_string(),
                ty: RustType::Generic {
                    base: "__SifrYielder".to_string(),
                    params: vec![rust_yield_type],
                },
            }],
            body: producer_body,
            is_move: true,
            is_async: true,
        };
        body.push(RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "AsyncGenerator".to_string(),
                "new_lazy".to_string(),
            ])),
            args: vec![factory],
        })));
        body
    }
}
