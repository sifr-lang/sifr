use sifr_ir::HirExpr;

use crate::{
    RustEmitter, RustExpr, RustLiteral, RustParam, RustStmt, RustType, Type,
    resolve_alias_type_for_plain_call,
};

enum StringCompareValue {
    Char(RustExpr),
    Str {
        direct: RustExpr,
        view: RustExpr,
        comparison_state: Option<RustExpr>,
    },
    OptionalStr(RustExpr),
}

impl RustEmitter {
    pub(crate) fn try_lower_string_equality_for_compare(
        &mut self,
        left: &HirExpr,
        right: &HirExpr,
        op: &str,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let Some(left) = self.lower_string_compare_value(left)? else {
            return Ok(None);
        };
        let Some(right) = self.lower_string_compare_value(right)? else {
            return Ok(None);
        };
        let (left, right) = match (left, right) {
            (StringCompareValue::Char(left), StringCompareValue::Char(right)) => (left, right),
            (
                StringCompareValue::Str { direct: left, .. },
                StringCompareValue::Str { direct: right, .. },
            ) => (left, right),
            (StringCompareValue::OptionalStr(left), StringCompareValue::OptionalStr(right)) => {
                (left, right)
            }
            (
                StringCompareValue::Char(left),
                StringCompareValue::Str {
                    view: right,
                    comparison_state,
                    ..
                },
            ) => (
                char_option_to_comparison_state(left),
                comparison_state
                    .unwrap_or_else(|| string_option_to_comparison_state(some_expr(right))),
            ),
            (
                StringCompareValue::Str {
                    view: left,
                    comparison_state,
                    ..
                },
                StringCompareValue::Char(right),
            ) => (
                comparison_state
                    .unwrap_or_else(|| string_option_to_comparison_state(some_expr(left))),
                char_option_to_comparison_state(right),
            ),
            (StringCompareValue::Char(left), StringCompareValue::OptionalStr(right)) => (
                char_option_to_comparison_state(left),
                string_option_to_comparison_state(right),
            ),
            (StringCompareValue::OptionalStr(left), StringCompareValue::Char(right)) => (
                string_option_to_comparison_state(left),
                char_option_to_comparison_state(right),
            ),
            (
                StringCompareValue::Str { view: left, .. },
                StringCompareValue::OptionalStr(right),
            ) => (some_expr(left), right),
            (
                StringCompareValue::OptionalStr(left),
                StringCompareValue::Str { view: right, .. },
            ) => (left, some_expr(right)),
        };
        Ok(Some(RustExpr::BinOp {
            left: Box::new(left),
            op: op.to_string(),
            right: Box::new(right),
        }))
    }

    fn lower_string_compare_value(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<StringCompareValue>, crate::CodegenError> {
        if let HirExpr::Index { object, index, .. } = expr
            && matches!(
                resolve_alias_type_for_plain_call(object.ty()),
                Type::Str | Type::LiteralStr(_)
            )
        {
            return self
                .lower_borrowed_string_index_char_for_compare(object, index)
                .map(|value| value.map(StringCompareValue::Char));
        }
        if let Some(value) = self.try_lower_borrowed_string_lookup_for_compare(expr)? {
            return Ok(Some(StringCompareValue::OptionalStr(value)));
        }
        if let Some((direct, view)) = self.lower_borrowed_string_name_for_compare(expr) {
            return Ok(Some(StringCompareValue::Str {
                direct,
                view,
                comparison_state: None,
            }));
        }
        let HirExpr::StringLiteral(value) = expr else {
            return Ok(None);
        };
        let literal = RustExpr::Literal(RustLiteral::StaticStr(value.clone()));
        Ok(Some(StringCompareValue::Str {
            direct: literal.clone(),
            view: literal,
            comparison_state: Some(literal_string_comparison_state(value)),
        }))
    }

    pub(crate) fn lower_borrowed_string_name_for_compare(
        &self,
        expr: &HirExpr,
    ) -> Option<(RustExpr, RustExpr)> {
        let HirExpr::Name { name, ty, .. } = expr else {
            return None;
        };
        if !matches!(
            resolve_alias_type_for_plain_call(ty),
            Type::Str | Type::LiteralStr(_)
        ) {
            return None;
        }
        let direct = RustExpr::Ident(name.clone());
        let view = self.string_view_expr(expr, direct.clone());
        Some((direct, view))
    }

    pub(crate) fn try_lower_borrowed_string_lookup_for_compare(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let HirExpr::Index { object, index, .. } = expr else {
            return Ok(None);
        };
        match resolve_alias_type_for_plain_call(object.ty()) {
            Type::Str | Type::LiteralStr(_) => {
                self.lower_borrowed_string_index_char_for_compare(object, index)
            }
            Type::List(element_ty) if matches!(element_ty.as_ref().resolve_alias(), Type::Str) => {
                self.lower_borrowed_list_string_index_for_compare(object, index)
            }
            Type::Dict(_, value_ty) if matches!(value_ty.as_ref().resolve_alias(), Type::Str) => {
                self.lower_borrowed_dict_string_index_for_compare(object, index)
            }
            _ => Ok(None),
        }
    }

    fn lower_borrowed_string_index_char_for_compare(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let lowered_object = if self.string_char_cache_for_expr(object).is_some() {
            RustExpr::Literal(RustLiteral::Str(String::new()))
        } else {
            let Some(lowered) = self.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            lowered
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let lowered_index = Self::clone_non_copy_name_expr_for_ir(index, lowered_index);
        Ok(Some(self.lower_string_index_char_option_with_cache(
            object,
            lowered_object,
            lowered_index,
        )))
    }

    fn lower_borrowed_list_string_index_for_compare(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        if let HirExpr::Index {
            object: outer_object,
            index: outer_index,
            ..
        } = object
        {
            if matches!(
                crate::resolve_alias_type_for_plain_call(outer_object.ty()),
                crate::Type::List(_)
            ) {
                let Some(lowered_outer_object) = self.lower_stmt_expr_for_ir(outer_object)? else {
                    return Ok(None);
                };
                let Some(lowered_outer_index) = self.lower_stmt_expr_for_ir(outer_index)? else {
                    return Ok(None);
                };
                let lowered_outer_index =
                    Self::clone_non_copy_name_expr_for_ir(outer_index, lowered_outer_index);
                let Some(lowered_inner_index) = self.lower_stmt_expr_for_ir(index)? else {
                    return Ok(None);
                };
                let lowered_inner_index =
                    Self::clone_non_copy_name_expr_for_ir(index, lowered_inner_index);
                return Ok(Some(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__sifr_cmp_outer_list".to_string(),
                            ty: None,
                            value: RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(lowered_outer_object),
                            },
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__sifr_cmp_outer_i".to_string(),
                            ty: None,
                            value: lowered_outer_index,
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__sifr_cmp_outer_norm".to_string(),
                            ty: None,
                            value: crate::build_normalized_list_index_i64_expr(
                                RustExpr::Ident("__sifr_cmp_outer_list".to_string()),
                                "__sifr_cmp_outer_i",
                            ),
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__sifr_cmp_i".to_string(),
                            ty: None,
                            value: lowered_inner_index,
                        },
                    ],
                    expr: Some(Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident(
                                "__sifr_cmp_outer_list".to_string(),
                            )),
                            method: "get".to_string(),
                            args: vec![RustExpr::Ident("__sifr_cmp_outer_norm".to_string())],
                        }),
                        method: "and_then".to_string(),
                        args: vec![RustExpr::ClosureBlock {
                            params: vec![RustParam::Named {
                                name: "__sifr_cmp_list".to_string(),
                                ty: RustType::Named("_".to_string()),
                            }],
                            body: vec![
                                RustStmt::Let {
                                    mutable: false,
                                    name: "__sifr_cmp_norm".to_string(),
                                    ty: None,
                                    value: crate::build_normalized_list_index_i64_expr(
                                        RustExpr::Ident("__sifr_cmp_list".to_string()),
                                        "__sifr_cmp_i",
                                    ),
                                },
                                RustStmt::Return(Some(Self::as_str_option(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident(
                                        "__sifr_cmp_list".to_string(),
                                    )),
                                    method: "get".to_string(),
                                    args: vec![RustExpr::Ident("__sifr_cmp_norm".to_string())],
                                }))),
                            ],
                            is_move: false,
                            is_async: false,
                        }],
                    })),
                }));
            }
        }
        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
            return Ok(None);
        };
        let mut stmts = vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_cmp_list".to_string(),
            ty: None,
            value: RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_object),
            },
        }];
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let lowered_index = Self::clone_non_copy_name_expr_for_ir(index, lowered_index);
        stmts.extend([
            RustStmt::Let {
                mutable: false,
                name: "__sifr_cmp_i".to_string(),
                ty: None,
                value: lowered_index,
            },
            RustStmt::Let {
                mutable: false,
                name: "__sifr_cmp_norm".to_string(),
                ty: None,
                value: crate::build_normalized_list_index_i64_expr(
                    RustExpr::Ident("__sifr_cmp_list".to_string()),
                    "__sifr_cmp_i",
                ),
            },
        ]);
        Ok(Some(RustExpr::Block {
            stmts,
            expr: Some(Box::new(Self::as_str_option(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_cmp_list".to_string())),
                method: "get".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(RustExpr::Ident("__sifr_cmp_norm".to_string())),
                    ty: RustType::Named("usize".to_string()),
                }],
            }))),
        }))
    }

    fn lower_borrowed_dict_string_index_for_compare(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let key = if matches!(
            resolve_alias_type_for_plain_call(index.ty()),
            Type::Str | Type::LiteralStr(_)
        ) {
            self.string_view_expr(index, lowered_index)
        } else {
            RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Paren(Box::new(lowered_index))),
            }
        };
        Ok(Some(Self::as_str_option(RustExpr::MethodCall {
            receiver: Box::new(lowered_object),
            method: "get".to_string(),
            args: vec![key],
        })))
    }

    fn as_str_option(option_expr: RustExpr) -> RustExpr {
        RustExpr::MethodCall {
            receiver: Box::new(option_expr),
            method: "map".to_string(),
            args: vec![RustExpr::Path(vec![
                "std".to_string(),
                "string".to_string(),
                "String".to_string(),
                "as_str".to_string(),
            ])],
        }
    }
}

fn some_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
        args: vec![value],
    }
}

fn char_option_to_comparison_state(option: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(option),
        method: "map".to_string(),
        args: vec![RustExpr::Path(vec!["Some".to_string()])],
    }
}

fn literal_string_comparison_state(value: &str) -> RustExpr {
    let mut chars = value.chars();
    let first = chars.next();
    let single_char = if chars.next().is_none() { first } else { None };
    let char_state = single_char.map_or_else(
        || RustExpr::Path(vec!["None".to_string()]),
        |character| some_expr(RustExpr::Literal(RustLiteral::Char(character))),
    );
    some_expr(char_state)
}

fn string_option_to_comparison_state(option: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(option),
        method: "map".to_string(),
        args: vec![RustExpr::ClosureBlock {
            params: vec![RustParam::Named {
                name: "__sifr_cmp_s".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: vec![
                RustStmt::Let {
                    mutable: true,
                    name: "__sifr_cmp_chars".to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__sifr_cmp_s".to_string())),
                        method: "chars".to_string(),
                        args: Vec::new(),
                    },
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_cmp_first".to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__sifr_cmp_chars".to_string())),
                        method: "next".to_string(),
                        args: Vec::new(),
                    },
                },
                RustStmt::Return(Some(RustExpr::If {
                    cond: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__sifr_cmp_chars".to_string())),
                            method: "next".to_string(),
                            args: Vec::new(),
                        }),
                        method: "is_some".to_string(),
                        args: Vec::new(),
                    }),
                    then_expr: Box::new(RustExpr::Path(vec!["None".to_string()])),
                    else_expr: Some(Box::new(RustExpr::Ident("__sifr_cmp_first".to_string()))),
                })),
            ],
            is_move: false,
            is_async: false,
        }],
    }
}
