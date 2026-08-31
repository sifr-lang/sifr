use crate::checked_place::CheckedPlaceFailureKind;
use crate::{HirExpr, HirStmt, RustEmitter, RustExpr, RustStmt, Type};
use sifr_ir::HirCollectionMutation;

#[derive(Clone, Copy)]
struct ProjectionFailure<'a> {
    ty: Option<&'a Type>,
    kind: CheckedPlaceFailureKind,
}

pub(crate) struct CheckedNestedMutationPlan<'a> {
    pub(crate) root: RustExpr,
    pub(crate) root_ty: &'a Type,
    pub(crate) outer_index: &'a HirExpr,
    pub(crate) inner_index: &'a HirExpr,
    pub(crate) value: &'a HirExpr,
    pub(crate) operation: &'a HirCollectionMutation,
    pub(crate) outer_failure: Option<&'a Type>,
    pub(crate) inner_failure: Option<&'a Type>,
}

impl RustEmitter {
    pub(crate) fn lower_checked_place_mutation_stmt_for_ir(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let lowered = match stmt {
            HirStmt::SubscriptAssign {
                object,
                index,
                value,
                object_ty,
                failure,
            } => self.lower_subscript_assign_stmt_for_ir(
                object,
                index,
                value,
                object_ty,
                failure.as_ref(),
            )?,
            HirStmt::NestedSubscriptAssign {
                object,
                outer_index,
                inner_index,
                value,
                object_ty,
                outer_failure,
                inner_failure,
                operation,
            } => self.lower_checked_nested_mutation_for_ir(CheckedNestedMutationPlan {
                root: RustExpr::Ident(object.clone()),
                root_ty: object_ty,
                outer_index,
                inner_index,
                value,
                operation,
                outer_failure: outer_failure.as_ref(),
                inner_failure: inner_failure.as_ref(),
            })?,
            HirStmt::AttributeNestedSubscriptAssign {
                object,
                field,
                outer_index,
                inner_index,
                value,
                field_ty,
                outer_failure,
                inner_failure,
                operation,
            } => self.lower_checked_nested_mutation_for_ir(CheckedNestedMutationPlan {
                root: RustExpr::Field {
                    expr: Box::new(Self::object_name_expr_for_ir(object)),
                    field: field.clone(),
                },
                root_ty: field_ty,
                outer_index,
                inner_index,
                value,
                operation,
                outer_failure: outer_failure.as_ref(),
                inner_failure: inner_failure.as_ref(),
            })?,
            HirStmt::SubscriptAugAssign {
                object,
                index,
                op,
                value,
                object_ty,
                failure,
            } => self.lower_subscript_augassign_stmt_for_ir(
                object,
                index,
                op,
                value,
                object_ty,
                failure.as_ref(),
            )?,
            HirStmt::AttributeSubscriptAssign {
                object,
                field,
                index,
                value,
                field_ty,
                failure,
                operation,
            } => self.lower_checked_single_mutation_for_ir(
                RustExpr::Field {
                    expr: Box::new(Self::object_name_expr_for_ir(object)),
                    field: field.clone(),
                },
                field_ty,
                index,
                value,
                operation,
                failure.as_ref(),
            )?,
            HirStmt::Delete {
                object,
                index,
                failure,
            } => self.lower_delete_stmt_for_ir(object, index, failure.as_ref())?,
            _ => return Ok(None),
        };
        Ok(lowered.map(|stmt| vec![stmt]))
    }

    pub(crate) fn lower_subscript_assign_stmt_for_ir(
        &mut self,
        object: &str,
        index: &HirExpr,
        value: &HirExpr,
        object_ty: &Type,
        failure: Option<&Type>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        self.lower_checked_single_mutation_for_ir(
            RustExpr::Ident(object.to_string()),
            object_ty,
            index,
            value,
            &HirCollectionMutation::Assign,
            failure,
        )
    }

    pub(crate) fn lower_checked_single_mutation_for_ir(
        &mut self,
        receiver: RustExpr,
        container_ty: &Type,
        index: &HirExpr,
        value: &HirExpr,
        operation: &HirCollectionMutation,
        failure: Option<&Type>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if crate::helpers::is_option_type(index.ty()) {
            return Ok(None);
        }
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let container_ty = crate::resolve_alias_type_for_plain_call(container_ty);
        let target_ty = match container_ty {
            Type::List(target) | Type::Dict(_, target) => target.as_ref(),
            _ => return Ok(None),
        };
        let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
        let (action, lowered_value) = self.checked_place_action(
            target_ty,
            value,
            lowered_value,
            operation,
            "__assign_value",
        )?;
        let failure_kind = match container_ty {
            Type::List(_) => CheckedPlaceFailureKind::Index,
            Type::Dict(_, _) => CheckedPlaceFailureKind::Key,
            _ => return Ok(None),
        };
        let projection_failure = ProjectionFailure {
            ty: failure,
            kind: failure_kind,
        };
        let mutation = match (container_ty, operation) {
            (Type::List(_), _) => self.wrap_sequence_projection(
                receiver,
                Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
                "__index",
                "__elem",
                vec![action],
                projection_failure,
            ),
            (Type::Dict(_, _), HirCollectionMutation::Assign) => RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__assign_key".to_string(),
                    ty: None,
                    value: Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
                },
                RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method: "insert".to_string(),
                    args: vec![
                        RustExpr::Ident("__assign_key".to_string()),
                        RustExpr::Ident("__assign_value".to_string()),
                    ],
                }),
            ]),
            (Type::Dict(_, _), HirCollectionMutation::AugAssign(_)) => {
                let lowered_key = Self::clone_non_copy_name_expr_for_ir(index, lowered_index);
                self.wrap_mapping_projection(
                    receiver,
                    lowered_key,
                    "__assign_key",
                    "__elem",
                    vec![action],
                    projection_failure,
                )
            }
            _ => return Ok(None),
        };
        Ok(Some(RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: "__assign_value".to_string(),
                ty: None,
                value: lowered_value,
            },
            mutation,
        ])))
    }

    pub(crate) fn lower_checked_nested_mutation_for_ir(
        &mut self,
        plan: CheckedNestedMutationPlan<'_>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let CheckedNestedMutationPlan {
            root,
            root_ty,
            outer_index,
            inner_index,
            value,
            operation,
            outer_failure,
            inner_failure,
        } = plan;
        if crate::helpers::is_option_type(outer_index.ty())
            || crate::helpers::is_option_type(inner_index.ty())
        {
            return Ok(None);
        }
        let Some(lowered_outer_index) = self.lower_stmt_expr_for_ir(outer_index)? else {
            return Ok(None);
        };
        let Some(lowered_inner_index) = self.lower_stmt_expr_for_ir(inner_index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };

        let root_ty = crate::resolve_alias_type_for_plain_call(root_ty);
        let inner_container_ty = match root_ty {
            Type::List(inner) | Type::Dict(_, inner) => {
                crate::resolve_alias_type_for_plain_call(inner)
            }
            _ => return Ok(None),
        };
        let target_ty = match inner_container_ty {
            Type::List(target) | Type::Dict(_, target) => target.as_ref(),
            _ => return Ok(None),
        };

        let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
        let (action, lowered_value) = self.checked_place_action(
            target_ty,
            value,
            lowered_value,
            operation,
            "__nested_assign_value",
        )?;

        let inner_failure = ProjectionFailure {
            ty: inner_failure,
            kind: match inner_container_ty {
                Type::List(_) => CheckedPlaceFailureKind::Index,
                Type::Dict(_, _) => CheckedPlaceFailureKind::Key,
                _ => return Ok(None),
            },
        };
        let inner = match (inner_container_ty, operation) {
            (Type::List(_), _) => self.wrap_sequence_projection(
                RustExpr::Ident("__row".to_string()),
                Self::clone_non_copy_name_expr_for_ir(inner_index, lowered_inner_index),
                "__inner",
                "__elem",
                vec![action],
                inner_failure,
            ),
            (Type::Dict(_, _), HirCollectionMutation::Assign) => RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__inner_key".to_string(),
                    ty: None,
                    value: Self::clone_non_copy_name_expr_for_ir(inner_index, lowered_inner_index),
                },
                RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__row".to_string())),
                    method: "insert".to_string(),
                    args: vec![
                        RustExpr::Ident("__inner_key".to_string()),
                        RustExpr::Ident("__nested_assign_value".to_string()),
                    ],
                }),
            ]),
            (Type::Dict(_, _), HirCollectionMutation::AugAssign(_)) => {
                let lowered_key =
                    Self::clone_non_copy_name_expr_for_ir(inner_index, lowered_inner_index);
                self.wrap_mapping_projection(
                    RustExpr::Ident("__row".to_string()),
                    lowered_key,
                    "__inner_key",
                    "__elem",
                    vec![action],
                    inner_failure,
                )
            }
            _ => return Ok(None),
        };

        let outer_failure = ProjectionFailure {
            ty: outer_failure,
            kind: match root_ty {
                Type::List(_) => CheckedPlaceFailureKind::Index,
                Type::Dict(_, _) => CheckedPlaceFailureKind::Key,
                _ => return Ok(None),
            },
        };
        let outer = match root_ty {
            Type::List(_) => self.wrap_sequence_projection(
                root,
                Self::clone_non_copy_name_expr_for_ir(outer_index, lowered_outer_index),
                "__outer",
                "__row",
                vec![inner],
                outer_failure,
            ),
            Type::Dict(_, _) => {
                let lowered_key =
                    Self::clone_non_copy_name_expr_for_ir(outer_index, lowered_outer_index);
                self.wrap_mapping_projection(
                    root,
                    lowered_key,
                    "__outer_key",
                    "__row",
                    vec![inner],
                    outer_failure,
                )
            }
            _ => return Ok(None),
        };
        Ok(Some(RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: "__nested_assign_value".to_string(),
                ty: None,
                value: lowered_value,
            },
            outer,
        ])))
    }

    fn checked_place_action(
        &mut self,
        target_ty: &Type,
        value: &HirExpr,
        lowered_value: RustExpr,
        operation: &HirCollectionMutation,
        value_binding: &str,
    ) -> Result<(RustStmt, RustExpr), crate::CodegenError> {
        let result = match operation {
            HirCollectionMutation::Assign => {
                let lowered_value = crate::helpers::flatten_option_value_for_target(
                    target_ty,
                    value.ty(),
                    lowered_value,
                );
                let action = RustStmt::Assign {
                    target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                    value: RustExpr::Ident(value_binding.to_string()),
                };
                (action, lowered_value)
            }
            HirCollectionMutation::AugAssign(op) => {
                if op == "+="
                    && matches!(
                        crate::resolve_alias_type_for_plain_call(target_ty),
                        Type::Str | Type::LiteralStr(_)
                    )
                {
                    return Ok((
                        RustStmt::Expr(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__elem".to_string())),
                            method: "push_str".to_string(),
                            args: vec![self.string_view_expr(
                                value,
                                RustExpr::Ident(value_binding.to_string()),
                            )],
                        }),
                        lowered_value,
                    ));
                }
                let exact_integer = matches!(
                    crate::resolve_alias_type_for_plain_call(target_ty),
                    Type::Int | Type::LiteralInt(_)
                );
                let lowered_value = if exact_integer {
                    self.coerce_typed_expr_to_sifr_int_value(lowered_value, value.ty())
                } else {
                    lowered_value
                };
                let Some(action) = Self::build_subscript_augassign_elem_stmt_for_ir(
                    op,
                    value,
                    RustExpr::Ident(value_binding.to_string()),
                    exact_integer,
                ) else {
                    return Err(crate::CodegenError::new(format!(
                        "unsupported checked-place augmented assignment operator `{op}`"
                    )));
                };
                (action, lowered_value)
            }
        };
        Ok(result)
    }

    fn wrap_sequence_projection(
        &mut self,
        receiver: RustExpr,
        index: RustExpr,
        prefix: &str,
        binding: &str,
        body: Vec<RustStmt>,
        failure: ProjectionFailure<'_>,
    ) -> RustStmt {
        let raw = format!("{prefix}_raw");
        let normalized = format!("{prefix}_normalized");
        RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: raw.clone(),
                ty: None,
                value: index,
            },
            RustStmt::Let {
                mutable: false,
                name: normalized.clone(),
                ty: None,
                value: crate::build_normalized_index_expr(
                    &raw,
                    RustExpr::MethodCall {
                        receiver: Box::new(receiver.clone()),
                        method: "len".to_string(),
                        args: Vec::new(),
                    },
                ),
            },
            RustStmt::IfLet {
                pattern: format!("Some({binding})"),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ident(normalized)],
                },
                then_body: body,
                else_body: self.projection_failure_body(failure),
            },
        ])
    }

    fn wrap_mapping_projection(
        &mut self,
        receiver: RustExpr,
        key: RustExpr,
        key_name: &str,
        binding: &str,
        body: Vec<RustStmt>,
        failure: ProjectionFailure<'_>,
    ) -> RustStmt {
        RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: key_name.to_string(),
                ty: None,
                value: key,
            },
            RustStmt::IfLet {
                pattern: format!("Some({binding})"),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident(key_name.to_string())),
                    }],
                },
                then_body: body,
                else_body: self.projection_failure_body(failure),
            },
        ])
    }

    fn projection_failure_body(&mut self, failure: ProjectionFailure<'_>) -> Option<Vec<RustStmt>> {
        failure
            .ty
            .map(|ty| vec![self.checked_place_failure_return(ty, failure.kind)])
    }
}
