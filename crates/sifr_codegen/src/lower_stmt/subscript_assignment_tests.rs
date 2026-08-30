use super::*;
#[test]
fn lowers_simple_attribute_list_subscript_assign_stmt() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "items".to_string(),
        index: HirExpr::Name {
            name: "i".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        value: HirExpr::Name {
            name: "v".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        field_ty: Type::List(Box::new(Type::Int)),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("attribute list subscript assign lowered");
    assert_eq!(lowered.len(), 1);
    let rendered = crate::render_stmts(&lowered);
    assert!(rendered.contains("let __idx_raw = i.clone();"));
    assert!(
        rendered.contains("let __idx_norm = __idx_raw.normalize_index_or_len(self.items.len());")
    );
    assert!(rendered.contains("if let Some(__elem) = self.items.get_mut(__idx_norm)"));
    assert!(rendered.contains("*__elem = v.clone();"));
    assert!(!rendered.contains("__idx_norm >= 0"));
    assert!(!rendered.contains("to_usize_proven"));
}

#[test]
fn lowers_simple_alias_attribute_list_subscript_assign_stmt() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        value: HirExpr::IntLiteral(1),
        field_ty: Type::alias("IntList", Type::List(Box::new(Type::Int))),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias attribute-list subscript assign lowered");
    assert!(matches!(lowered[0], RustStmt::Block(_)));
}

#[test]
fn lowers_simple_attribute_dict_subscript_assign_stmt() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "mapping".to_string(),
        index: HirExpr::Name {
            name: "key".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        value: HirExpr::Name {
            name: "val".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        field_ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("attribute dict subscript assign lowered");
    assert!(matches!(
        lowered[0],
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        }) if method == "insert"
            && matches!(
                recv.as_ref(),
                RustExpr::Field { expr, field }
                    if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
                    && field == "mapping"
            )
            && matches!(args.first(), Some(RustExpr::Clone(inner))
                if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "key"))
            && matches!(args.get(1), Some(RustExpr::Clone(inner))
                if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "val"))
    ));
}

#[test]
fn lowers_simple_alias_attribute_dict_subscript_assign_stmt() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "mapping".to_string(),
        index: HirExpr::IntLiteral(1),
        value: HirExpr::IntLiteral(2),
        field_ty: Type::alias(
            "IntMap",
            Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
        ),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias attribute-dict subscript assign lowered");
    assert!(matches!(lowered[0], RustStmt::Expr(_)));
}

#[test]
fn does_not_lower_attribute_dict_subscript_assign_with_string_name_key() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "mapping".to_string(),
        index: HirExpr::Name {
            name: "k".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
        value: HirExpr::IntLiteral(1),
        field_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn does_not_lower_attribute_subscript_assign_with_non_leaf_index() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "items".to_string(),
        index: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "next_idx".to_string(),
            args: vec![],
            ty: Type::Int,
        },
        value: HirExpr::IntLiteral(1),
        field_ty: Type::List(Box::new(Type::Int)),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn lowers_simple_list_subscript_assign_stmt() {
    let stmt = HirStmt::SubscriptAssign {
        object: "items".to_string(),
        index: HirExpr::Name {
            name: "i".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        value: HirExpr::Name {
            name: "v".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::Int)),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("list subscript assign lowered");
    assert_eq!(lowered.len(), 1);
    let RustStmt::Block(stmts) = &lowered[0] else {
        panic!("expected block-lowered list subscript assignment");
    };
    assert_eq!(stmts.len(), 3);
    assert!(matches!(
        stmts[0],
        RustStmt::Let {
            mutable: false,
            ref name,
            value: RustExpr::Clone(ref inner),
            ..
        } if name == "__idx_raw" && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "i")
    ));
    assert!(matches!(
        stmts[2],
        RustStmt::IfLet {
            pattern: ref outer_pattern,
            expr: RustExpr::MethodCall {
                receiver: ref recv,
                method: ref outer_method,
                args: ref outer_args,
            },
            then_body: ref body,
            else_body: None,
        } if outer_pattern == "Some(__elem)"
            && outer_method == "get_mut"
            && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "items")
            && matches!(outer_args.first(), Some(RustExpr::Ident(name)) if name == "__idx_norm")
            && matches!(
                body.first(),
                Some(RustStmt::Assign {
                    target: RustExpr::Deref(target),
                    value: RustExpr::Clone(rhs),
                }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                    && matches!(rhs.as_ref(), RustExpr::Ident(name) if name == "v")
            )
    ));
}

#[test]
fn lowers_simple_alias_list_subscript_assign_stmt() {
    let stmt = HirStmt::SubscriptAssign {
        object: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        value: HirExpr::IntLiteral(1),
        object_ty: Type::alias("IntList", Type::List(Box::new(Type::Int))),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias-list subscript assign lowered");
    assert!(matches!(lowered[0], RustStmt::Block(_)));
}

#[test]
fn lowers_simple_dict_subscript_assign_stmt() {
    let stmt = HirStmt::SubscriptAssign {
        object: "mapping".to_string(),
        index: HirExpr::Name {
            name: "key".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
        value: HirExpr::Name {
            name: "val".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("dict subscript assign lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        }) if method == "insert"
            && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "mapping")
            && matches!(args.first(), Some(RustExpr::Clone(inner))
                if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "key"))
            && matches!(args.get(1), Some(RustExpr::Clone(inner))
                if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "val"))
    ));
}

#[test]
fn lowers_simple_dict_subscript_assign_clones_non_copy_name_value() {
    let stmt = HirStmt::SubscriptAssign {
        object: "mapping".to_string(),
        index: HirExpr::Name {
            name: "key".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
        value: HirExpr::Name {
            name: "val".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
        object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Str)),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("dict subscript assign lowered");
    assert!(matches!(
        lowered[0],
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        }) if method == "insert"
            && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "mapping")
            && matches!(args.first(), Some(RustExpr::Clone(inner))
                if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "key"))
            && matches!(args.get(1), Some(RustExpr::Clone(inner))
                if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "val"))
    ));
}

#[test]
fn does_not_lower_subscript_assign_with_non_leaf_index() {
    let stmt = HirStmt::SubscriptAssign {
        object: "items".to_string(),
        index: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "next_idx".to_string(),
            args: vec![],
            ty: Type::Int,
        },
        value: HirExpr::IntLiteral(1),
        object_ty: Type::List(Box::new(Type::Int)),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn lowers_simple_list_delete_stmt() {
    let stmt = HirStmt::Delete {
        object: HirExpr::Name {
            name: "items".to_string(),
            binding_id: None,
            ty: Type::List(Box::new(Type::Int)),
        },
        index: HirExpr::Name {
            name: "i".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("list delete lowered");
    assert_eq!(lowered.len(), 1);
    let rendered = crate::render_stmts(&lowered);
    assert!(rendered.contains("let __idx_norm = __idx_raw.normalize_index_or_len(items.len());"));
    assert!(rendered.contains("if __idx_norm < items.len()"));
    assert!(rendered.contains("let _ = items.remove(__idx_norm);"));
    assert!(!rendered.contains("__idx_norm >= 0"));
    assert!(!rendered.contains("to_usize_proven"));
}

#[test]
fn lowers_simple_dict_delete_with_string_literal_key_stmt() {
    let stmt = HirStmt::Delete {
        object: HirExpr::Name {
            name: "mapping".to_string(),
            binding_id: None,
            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        },
        index: HirExpr::StringLiteral("key".to_string()),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("dict delete lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            ref name,
            value: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            ..
        } if name == "_"
            && method == "remove"
            && matches!(recv.as_ref(), RustExpr::Ident(obj) if obj == "mapping")
            && matches!(
                args.first(),
                Some(RustExpr::Ref {
                    mutable: false,
                    expr: inner,
                }) if matches!(inner.as_ref(), RustExpr::Literal(RustLiteral::Str(key)) if key == "key")
            )
    ));
}

#[test]
fn does_not_lower_dict_delete_with_name_key() {
    let stmt = HirStmt::Delete {
        object: HirExpr::Name {
            name: "mapping".to_string(),
            binding_id: None,
            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        },
        index: HirExpr::Name {
            name: "k".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn lowers_simple_nested_subscript_assign_stmt() {
    let stmt = HirStmt::NestedSubscriptAssign {
        object: "matrix".to_string(),
        outer_index: HirExpr::Name {
            name: "i".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        inner_index: HirExpr::Name {
            name: "j".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        value: HirExpr::Name {
            name: "v".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::List(Box::new(Type::Int)))),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("nested subscript assign lowered");
    assert_eq!(lowered.len(), 1);
    let rendered = crate::render_stmts(&lowered);
    assert!(rendered.contains("let __oi_raw = i.clone();"));
    assert!(rendered.contains("matrix.get_mut(__oi_norm)"));
    assert!(rendered.contains("let __ii_raw = j.clone();"));
    assert!(rendered.contains("__row.get_mut(__ii_norm)"));
    assert!(rendered.contains("*__elem = v.clone();"));
    assert!(!rendered.contains(">= 0"));
    assert!(!rendered.contains("to_usize_proven"));
}

#[test]
fn lowers_simple_nested_subscript_assign_stmt_with_optional_indices() {
    let stmt = HirStmt::NestedSubscriptAssign {
        object: "matrix".to_string(),
        outer_index: HirExpr::Name {
            name: "oi".to_string(),
            binding_id: None,
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        inner_index: HirExpr::Name {
            name: "ii".to_string(),
            binding_id: None,
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        value: HirExpr::Name {
            name: "v".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::List(Box::new(Type::Int)))),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("nested subscript assign lowered");
    let rendered = crate::render_stmts(&lowered);
    assert!(rendered.contains("let __oi_raw_opt = oi.clone();"));
    assert!(rendered.contains("if let Some(__oi_raw) = __oi_raw_opt"));
    assert!(rendered.contains("matrix.get_mut(__oi_norm)"));
    assert!(rendered.contains("let __ii_raw_opt = ii.clone();"));
    assert!(rendered.contains("if let Some(__ii_raw) = __ii_raw_opt"));
    assert!(rendered.contains("__row.get_mut(__ii_norm)"));
    assert!(rendered.contains("*__elem = v.clone();"));
}

#[test]
fn does_not_lower_nested_subscript_assign_with_non_leaf_inner_index() {
    let stmt = HirStmt::NestedSubscriptAssign {
        object: "matrix".to_string(),
        outer_index: HirExpr::IntLiteral(0),
        inner_index: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "inner_idx".to_string(),
            args: vec![],
            ty: Type::Int,
        },
        value: HirExpr::IntLiteral(1),
        object_ty: Type::List(Box::new(Type::List(Box::new(Type::Int)))),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}
