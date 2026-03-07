use super::*;

pub(super) fn lower_expr(expr: &Expr, ctx: &mut LowerCtx) -> Option<HirExpr> {
    match expr {
        Expr::NumberLiteral(num) => lower_number_literal(num),
        Expr::StringLiteral(s) => {
            let value = s.value.to_str().to_string();
            Some(HirExpr::StringLiteral(value))
        }
        Expr::BooleanLiteral(b) => Some(HirExpr::BoolLiteral(b.value)),
        Expr::NoneLiteral(_) => Some(HirExpr::NoneLiteral),
        Expr::Name(name) => lower_name(name, ctx),
        Expr::BinOp(binop) => lower_binop(binop, ctx),
        Expr::UnaryOp(unary) => lower_unaryop(unary, ctx),
        Expr::Compare(cmp) => lower_compare(cmp, ctx),
        Expr::BoolOp(boolop) => lower_boolop(boolop, ctx),
        Expr::Call(call) => lower_call(call, ctx),
        Expr::If(if_expr) => lower_if_expr(if_expr, ctx),
        Expr::List(list) => lower_list_literal(list, ctx),
        Expr::Set(set) => lower_set_literal(set, ctx),
        Expr::Dict(dict) => lower_dict_literal(dict, ctx),
        Expr::Tuple(tuple) => lower_tuple_literal(tuple, ctx),
        Expr::Subscript(sub) => lower_subscript(sub, ctx),
        Expr::Attribute(attr) => lower_attribute(attr, ctx),
        Expr::FString(fstring) => lower_fstring(fstring, ctx),
        Expr::Named(named) => lower_named_expr(named, ctx),
        Expr::Lambda(lambda) => lower_lambda(lambda, ctx),
        Expr::ListComp(comp) => lower_list_comp(comp, ctx),
        Expr::SetComp(comp) => lower_set_comp(comp, ctx),
        Expr::DictComp(comp) => lower_dict_comp(comp, ctx),
        Expr::Generator(gen) => lower_generator_expr(gen, ctx),
        _ => {
            ctx.error("unsupported expression type".to_string());
            None
        }
    }
}

pub(super) fn lower_number_literal(num: &ExprNumberLiteral) -> Option<HirExpr> {
    match &num.value {
        Number::Int(i) => {
            let val = i.as_i64()?;
            Some(HirExpr::IntLiteral(val))
        }
        Number::Float(f) => Some(HirExpr::FloatLiteral(*f)),
        Number::Complex { .. } => None, // Not supported in M1
    }
}

pub(super) fn lower_name(name: &ExprName, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let var_name = name.id.clone();

    // Check if it's a known variable
    if let Some(info) = ctx.scope.lookup(&var_name) {
        let is_moved = info.is_moved;
        // Use effective type (narrowed if available)
        let ty = info.effective_type().clone();
        if is_moved {
            ctx.error(format!("use of moved value: '{var_name}'"));
        }
        return Some(HirExpr::Name { name: var_name, ty });
    }

    // Check if it's a known function
    if let Some(ft) = ctx.functions.get(&var_name) {
        let ft = ft.clone();
        return Some(HirExpr::Name {
            name: var_name,
            ty: Type::Function(ft),
        });
    }

    // Check built-in constants
    match var_name.as_str() {
        "True" => return Some(HirExpr::BoolLiteral(true)),
        "False" => return Some(HirExpr::BoolLiteral(false)),
        _ => {}
    }

    ctx.error(format!("undefined variable: '{var_name}'"));
    None
}

/// Map a binary operator to its corresponding dunder method name.
pub(super) fn op_to_dunder(op: &str) -> Option<&'static str> {
    match op {
        "+" => Some("__add__"),
        "-" => Some("__sub__"),
        "*" => Some("__mul__"),
        "/" => Some("__truediv__"),
        "//" => Some("__floordiv__"),
        "%" => Some("__mod__"),
        "**" => Some("__pow__"),
        _ => None,
    }
}

/// Shape compatibility used when generic inference leaves unresolved TypeVars.
/// TypeVars are treated as wildcards, but container/class structure must still match.
fn is_compatible_with_unresolved_typevars(source: &Type, target: &Type) -> bool {
    match target {
        Type::TypeVar(_) => true,
        Type::List(target_elem) => match source {
            Type::List(source_elem) => {
                is_compatible_with_unresolved_typevars(source_elem, target_elem)
            }
            _ => false,
        },
        Type::Set(target_elem) => match source {
            Type::Set(source_elem) => {
                is_compatible_with_unresolved_typevars(source_elem, target_elem)
            }
            _ => false,
        },
        Type::Dict(target_key, target_val) => match source {
            Type::Dict(source_key, source_val) => {
                is_compatible_with_unresolved_typevars(source_key, target_key)
                    && is_compatible_with_unresolved_typevars(source_val, target_val)
            }
            _ => false,
        },
        Type::Tuple(target_elems) => match source {
            Type::Tuple(source_elems) => {
                source_elems.len() == target_elems.len()
                    && source_elems
                        .iter()
                        .zip(target_elems.iter())
                        .all(|(src, dst)| is_compatible_with_unresolved_typevars(src, dst))
            }
            _ => false,
        },
        Type::Result(target_ok, target_err) => match source {
            Type::Result(source_ok, source_err) => {
                is_compatible_with_unresolved_typevars(source_ok, target_ok)
                    && is_compatible_with_unresolved_typevars(source_err, target_err)
            }
            _ => false,
        },
        Type::Class {
            name: target_name, ..
        } => match source {
            Type::Class {
                name: source_name, ..
            } => source_name == target_name,
            _ => false,
        },
        Type::Union(target_members) => target_members
            .iter()
            .any(|member| is_compatible_with_unresolved_typevars(source, member)),
        _ => source.is_assignable_to(target),
    }
}

pub(super) fn lower_binop(binop: &ExprBinOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let left = lower_expr(&binop.left, ctx)?;
    let right = lower_expr(&binop.right, ctx)?;

    let op_str = match binop.op {
        Operator::Add => "+",
        Operator::Sub => "-",
        Operator::Mult => "*",
        Operator::Div => "/",
        Operator::FloorDiv => "//",
        Operator::Mod => "%",
        Operator::Pow => "**",
        Operator::BitAnd => "&",
        Operator::BitOr => "|",
        Operator::BitXor => "^",
        Operator::LShift => "<<",
        Operator::RShift => ">>",
        Operator::MatMult => {
            ctx.error("matrix multiplication operator (@) is not supported".to_string());
            return None;
        }
    };

    match type_check_binary_op(left.ty(), op_str, right.ty()) {
        Ok(result_ty) => {
            if result_ty == Type::Int {
                check_int_overflow_risk(op_str, &left, &right, ctx);
            }
            Some(HirExpr::BinOp {
                left: Box::new(left),
                op: op_str.to_string(),
                right: Box::new(right),
                ty: result_ty,
            })
        }
        Err(e) => {
            // Check for operator overloading on class types
            if let Type::Class { methods, .. } = left.ty() {
                if let Some(dunder) = op_to_dunder(op_str) {
                    if let Some((_, ft)) = methods.iter().find(|(n, _)| n == dunder) {
                        let result_ty = *ft.return_type.clone();
                        return Some(HirExpr::BinOp {
                            left: Box::new(left),
                            op: op_str.to_string(),
                            right: Box::new(right),
                            ty: result_ty,
                        });
                    }
                }
            }
            ctx.error(e.message);
            None
        }
    }
}

pub(super) fn check_int_overflow_risk(
    op: &str,
    left: &HirExpr,
    right: &HirExpr,
    ctx: &mut LowerCtx,
) {
    let is_left_const = matches!(left, HirExpr::IntLiteral(_));
    let is_right_const = matches!(right, HirExpr::IntLiteral(_));

    match op {
        "**" => {
            if let HirExpr::IntLiteral(exp) = right {
                if *exp > 40 {
                    ctx.warn(format!(
                        "warning: int exponentiation with large exponent ({exp}) may overflow i64; consider using bigint"
                    ));
                }
            } else {
                ctx.warn(
                    "warning: int exponentiation (**) with non-constant exponent may overflow i64 at runtime; consider using bigint".to_string()
                );
            }
        }
        "*" => {
            if !is_left_const && !is_right_const {
                ctx.warn(
                    "warning: int multiplication with non-constant operands may overflow i64 at runtime; consider using bigint for large values".to_string()
                );
            }
        }
        "<<" => {
            if !is_right_const {
                ctx.warn(
                    "warning: int left shift (<<) with non-constant shift amount may overflow i64 at runtime; consider using bigint".to_string()
                );
            } else if let HirExpr::IntLiteral(shift) = right {
                if *shift >= 63 {
                    ctx.warn(format!(
                        "warning: int left shift by {shift} exceeds i64 range; consider using bigint"
                    ));
                }
            }
        }
        _ => {}
    }
}

pub(super) fn lower_unaryop(unary: &ExprUnaryOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let operand = lower_expr(&unary.operand, ctx)?;

    let op_str = match unary.op {
        UnaryOp::USub => "-",
        UnaryOp::UAdd => "+",
        UnaryOp::Not => "not",
        UnaryOp::Invert => "~",
    };

    match type_check_unary_op(op_str, operand.ty()) {
        Ok(result_ty) => Some(HirExpr::UnaryOp {
            op: op_str.to_string(),
            operand: Box::new(operand),
            ty: result_ty,
        }),
        Err(e) => {
            ctx.error(e.message);
            None
        }
    }
}

pub(super) fn lower_compare(cmp: &ExprCompare, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let left = lower_expr(&cmp.left, ctx)?;

    // Handle `in` and `not in` operators specially
    if cmp.ops.len() == 1 {
        match &cmp.ops[0] {
            CmpOp::In => {
                let collection = lower_expr(&cmp.comparators[0], ctx)?;
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if !left.ty().is_assignable_to(&elem_ty) {
                        ctx.error(format!(
                            "'in' operator: element type '{}' is not compatible with collection element type '{}'",
                            left.ty().display_name(),
                            elem_ty.display_name()
                        ));
                    }
                } else {
                    ctx.error(format!(
                        "'in' operator not supported for type '{}'",
                        collection_ty.display_name()
                    ));
                }
                return Some(HirExpr::ContainsOp {
                    element: Box::new(left),
                    collection: Box::new(collection),
                    ty: Type::Bool,
                });
            }
            CmpOp::NotIn => {
                let collection = lower_expr(&cmp.comparators[0], ctx)?;
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if !left.ty().is_assignable_to(&elem_ty) {
                        ctx.error(format!(
                            "'not in' operator: element type '{}' is not compatible with collection element type '{}'",
                            left.ty().display_name(),
                            elem_ty.display_name()
                        ));
                    }
                } else {
                    ctx.error(format!(
                        "'not in' operator not supported for type '{}'",
                        collection_ty.display_name()
                    ));
                }
                // Wrap in a UnaryOp not
                let contains = HirExpr::ContainsOp {
                    element: Box::new(left),
                    collection: Box::new(collection),
                    ty: Type::Bool,
                };
                return Some(HirExpr::UnaryOp {
                    op: "not".to_string(),
                    operand: Box::new(contains),
                    ty: Type::Bool,
                });
            }
            _ => {}
        }
    }

    let mut ops = Vec::new();
    let mut comparators = Vec::new();

    for (op, comparator) in cmp.ops.iter().zip(cmp.comparators.iter()) {
        let op_str = match op {
            CmpOp::Eq => "==",
            CmpOp::NotEq => "!=",
            CmpOp::Lt => "<",
            CmpOp::Gt => ">",
            CmpOp::LtE => "<=",
            CmpOp::GtE => ">=",
            CmpOp::Is => "is",
            CmpOp::IsNot => "is not",
            _ => {
                ctx.error("unsupported comparison operator".to_string());
                return None;
            }
        };

        let right = lower_expr(comparator, ctx)?;

        // `is` and `is not` are identity checks (used for None comparison)
        // They don't need type_check_comparison
        if op_str != "is" && op_str != "is not" {
            if let Err(e) = type_check_comparison(left.ty(), op_str, right.ty()) {
                // Check for operator overloading on class types
                let has_overload = match left.ty() {
                    Type::Class { methods, .. } => {
                        let dunder = match op_str {
                            "==" | "!=" => "__eq__",
                            "<" | ">" | "<=" | ">=" => "__lt__",
                            _ => "",
                        };
                        !dunder.is_empty() && methods.iter().any(|(n, _)| n == dunder)
                    }
                    _ => false,
                };
                if !has_overload {
                    ctx.error(e.message);
                    return None;
                }
            }
        }

        ops.push(op_str.to_string());
        comparators.push(right);
    }

    Some(HirExpr::Compare {
        left: Box::new(left),
        ops,
        comparators,
        ty: Type::Bool,
    })
}

pub(super) fn lower_boolop(boolop: &ExprBoolOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let op_str = match boolop.op {
        BoolOp::And => "and",
        BoolOp::Or => "or",
    };

    let mut values = Vec::new();
    for val in &boolop.values {
        let expr = lower_expr(val, ctx)?;
        values.push(expr);
    }

    // Check all values are Bool
    for val in &values {
        if let Err(e) = type_check_bool_op(val.ty(), op_str, &Type::Bool) {
            ctx.error(e.message);
            return None;
        }
    }

    Some(HirExpr::BoolOp {
        op: op_str.to_string(),
        values,
        ty: Type::Bool,
    })
}

pub(super) fn lower_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    // Handle method calls: obj.method(args)
    if let Expr::Attribute(attr) = call.func.as_ref() {
        return lower_method_call(attr, call, ctx);
    }

    let func_name = if let Expr::Name(n) = call.func.as_ref() {
        n.id.clone()
    } else {
        ctx.error("only simple function calls are supported".to_string());
        return None;
    };

    // Handle `cls(...)` in @classmethod as constructor call for the current class
    if func_name == "cls" {
        if let Some(ref class_name) = ctx.current_class {
            let class_name = class_name.clone();
            if let Some(class_ty) = ctx.class_types.get(&class_name).cloned() {
                // Lower arguments
                let mut args = Vec::new();
                for arg in &call.arguments.args {
                    let expr = lower_expr(arg, ctx)?;
                    args.push(expr);
                }
                return Some(HirExpr::ConstructorCall {
                    class_name,
                    args,
                    ty: class_ty,
                });
            }
        }
    }

    // Special handling for range() built-in
    if func_name == "range" {
        return lower_range_call(call, ctx);
    }

    // Special handling for len() built-in
    if func_name == "len" {
        return lower_len_call(call, ctx);
    }

    // Special handling for isinstance() built-in
    if func_name == "isinstance" {
        return lower_isinstance_call(call, ctx);
    }

    // Special handling for reveal_type() built-in
    if func_name == "reveal_type" {
        return lower_reveal_type_call(call, ctx);
    }

    // Special handling for str() conversion
    if func_name == "str" {
        if call.arguments.args.len() == 1 {
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            return Some(HirExpr::Call {
                func: "str".to_string(),
                args: vec![arg],
                ty: Type::Str,
            });
        }
    }

    // pow(base, exp) -> base ** exp
    if func_name == "pow" {
        if call.arguments.args.len() != 2 {
            ctx.error("pow() takes exactly 2 arguments".to_string());
            return None;
        }
        let base = lower_expr(&call.arguments.args[0], ctx)?;
        let exp = lower_expr(&call.arguments.args[1], ctx)?;
        let result_ty = if base.ty() == &Type::Int && exp.ty() == &Type::Int {
            Type::Int
        } else {
            Type::Float
        };
        return Some(HirExpr::Call {
            func: "pow".to_string(),
            args: vec![base, exp],
            ty: result_ty,
        });
    }

    // Special handling for abs() built-in
    if func_name == "abs" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!(
                "abs() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let ty = arg.ty().clone();
        if !ty.is_numeric() {
            ctx.error(format!(
                "abs() argument must be numeric, got '{}'",
                ty.display_name()
            ));
            return None;
        }
        return Some(HirExpr::Call {
            func: "abs".to_string(),
            args: vec![arg],
            ty,
        });
    }

    // Special handling for hash() built-in
    if func_name == "hash" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!(
                "hash() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let ty = arg.ty().clone();
        // Check if the type is hashable
        if !is_hashable_type(&ty) {
            ctx.error(format!(
                "hash() argument must be hashable, got '{}'",
                ty.display_name()
            ));
            return None;
        }
        return Some(HirExpr::Call {
            func: "hash".to_string(),
            args: vec![arg],
            ty: Type::Int,
        });
    }

    // Special handling for round() built-in
    if func_name == "round" {
        if call.arguments.args.is_empty() || call.arguments.args.len() > 2 {
            ctx.error(format!(
                "round() takes 1 or 2 arguments, got {}",
                call.arguments.args.len()
            ));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        if !arg.ty().is_numeric() {
            ctx.error(format!(
                "round() argument must be numeric, got '{}'",
                arg.ty().display_name()
            ));
            return None;
        }
        if call.arguments.args.len() == 2 {
            let ndigits = lower_expr(&call.arguments.args[1], ctx)?;
            return Some(HirExpr::Call {
                func: "round".to_string(),
                args: vec![arg, ndigits],
                ty: Type::Float,
            });
        }
        return Some(HirExpr::Call {
            func: "round".to_string(),
            args: vec![arg],
            ty: Type::Int,
        });
    }

    // Special handling for repr() built-in
    if func_name == "repr" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!(
                "repr() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(HirExpr::Call {
            func: "repr".to_string(),
            args: vec![arg],
            ty: Type::Str,
        });
    }

    // Decimal("...") / Decimal(int|bigint|bigdecimal)
    if func_name == "Decimal" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!(
                "[E2505] Decimal() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        let decimal_conversion_error_ty = ctx
            .class_types
            .get("DecimalConversionError")
            .cloned()
            .unwrap_or(Type::Class {
                name: "DecimalConversionError".to_string(),
                fields: vec![("message".to_string(), Type::Str)],
                methods: vec![],
                parent_class: None,
            });
        let result_ty = match arg_ty {
            Type::Str => {
                if !matches!(call.arguments.args[0], Expr::StringLiteral(_)) {
                    ctx.error(
                        "[E2501] Decimal() string construction requires a string literal"
                            .to_string(),
                    );
                    return None;
                }
                Type::Decimal
            }
            Type::Int | Type::LiteralInt(_) | Type::Decimal => Type::Decimal,
            Type::BigInt | Type::BigDecimal => Type::Result(
                Box::new(Type::Decimal),
                Box::new(decimal_conversion_error_ty),
            ),
            Type::Float => {
                ctx.error(
                    "[E2505] Decimal(float_value) is not allowed; use Decimal(\"...\") for exact construction"
                        .to_string(),
                );
                return None;
            }
            _ => {
                ctx.error(format!(
                    "[E2505] Decimal() requires str, int, bigint, decimal, or bigdecimal argument, got '{}'",
                    arg_ty.display_name()
                ));
                return None;
            }
        };
        return Some(HirExpr::Call {
            func: "Decimal".to_string(),
            args: vec![arg],
            ty: result_ty,
        });
    }

    // BigDecimal("...") / BigDecimal(int|bigint|decimal)
    if func_name == "BigDecimal" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!(
                "[E2506] BigDecimal() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        match arg_ty {
            Type::Str => {
                if !matches!(call.arguments.args[0], Expr::StringLiteral(_)) {
                    ctx.error(
                        "[E2502] BigDecimal() string construction requires a string literal"
                            .to_string(),
                    );
                    return None;
                }
            }
            Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::Decimal | Type::BigDecimal => {}
            Type::Float => {
                ctx.error(
                    "[E2506] BigDecimal(float_value) is not allowed; use BigDecimal(\"...\") for exact construction"
                        .to_string(),
                );
                return None;
            }
            _ => {
                ctx.error(format!(
                    "[E2506] BigDecimal() requires str, int, bigint, decimal, or bigdecimal argument, got '{}'",
                    arg_ty.display_name()
                ));
                return None;
            }
        }
        return Some(HirExpr::Call {
            func: "BigDecimal".to_string(),
            args: vec![arg],
            ty: Type::BigDecimal,
        });
    }

    // Special handling for int() conversion
    if func_name == "int" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!(
                "int() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        // int(str) -> Result[int, ParseError] (fallible)
        // int(float) -> int (infallible truncation)
        // int(int) -> int (identity)
        // int(bool) -> int (True=1, False=0)
        // int(bigint) -> Result[int, OverflowError] (may overflow i64)
        let result_ty = if arg_ty == Type::Str {
            let parse_error_ty =
                ctx.class_types
                    .get("ParseError")
                    .cloned()
                    .unwrap_or(Type::Class {
                        name: "ParseError".to_string(),
                        fields: vec![("message".to_string(), Type::Str)],
                        methods: vec![],
                        parent_class: None,
                    });
            Type::Result(Box::new(Type::Int), Box::new(parse_error_ty))
        } else if arg_ty == Type::BigInt {
            let overflow_error_ty =
                ctx.class_types
                    .get("OverflowError")
                    .cloned()
                    .unwrap_or(Type::Class {
                        name: "OverflowError".to_string(),
                        fields: vec![("message".to_string(), Type::Str)],
                        methods: vec![],
                        parent_class: None,
                    });
            Type::Result(Box::new(Type::Int), Box::new(overflow_error_ty))
        } else {
            Type::Int
        };
        return Some(HirExpr::Call {
            func: "int".to_string(),
            args: vec![arg],
            ty: result_ty,
        });
    }

    // bigint(n) — convert int to bigint (always succeeds)
    if func_name == "bigint" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!(
                "bigint() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        if arg_ty != Type::Int && arg_ty != Type::BigInt {
            ctx.error(format!(
                "bigint() requires an int argument, got '{}'",
                arg_ty.display_name()
            ));
            return None;
        }
        return Some(HirExpr::Call {
            func: "bigint".to_string(),
            args: vec![arg],
            ty: Type::BigInt,
        });
    }

    // Special handling for float() conversion
    if func_name == "float" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!(
                "float() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        // float(str) -> Result[float, ParseError] (fallible)
        // float(int) -> float (infallible widening)
        // float(float) -> float (identity)
        let result_ty = if arg_ty == Type::Str {
            let parse_error_ty =
                ctx.class_types
                    .get("ParseError")
                    .cloned()
                    .unwrap_or(Type::Class {
                        name: "ParseError".to_string(),
                        fields: vec![("message".to_string(), Type::Str)],
                        methods: vec![],
                        parent_class: None,
                    });
            Type::Result(Box::new(Type::Float), Box::new(parse_error_ty))
        } else {
            Type::Float
        };
        return Some(HirExpr::Call {
            func: "float".to_string(),
            args: vec![arg],
            ty: result_ty,
        });
    }

    // Special handling for bool() conversion
    if func_name == "bool" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!(
                "bool() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(HirExpr::Call {
            func: "bool".to_string(),
            args: vec![arg],
            ty: Type::Bool,
        });
    }

    // --- Built-in generic functions ---

    // min(iterable) or min(a, b) -> element type
    if func_name == "min" {
        if call.arguments.args.len() == 2 {
            // min(a, b) -> std::cmp::min(a, b)
            let a = lower_expr(&call.arguments.args[0], ctx)?;
            let b = lower_expr(&call.arguments.args[1], ctx)?;
            let result_ty = a.ty().clone();
            return Some(HirExpr::Call {
                func: "min".to_string(),
                args: vec![a, b],
                ty: result_ty,
            });
        } else if call.arguments.args.len() == 1 {
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let elem_ty = if let Type::List(elem) = arg.ty() {
                *elem.clone()
            } else {
                ctx.error(format!(
                    "min() argument must be a list, got '{}'",
                    arg.ty().display_name()
                ));
                return None;
            };
            // Returns Option[T] = T | None (safe: None on empty list)
            return Some(HirExpr::Call {
                func: "min".to_string(),
                args: vec![arg],
                ty: Type::Union(vec![elem_ty, Type::None]),
            });
        }
        ctx.error("min() takes 1 or 2 arguments".to_string());
        return None;
    }

    // max(iterable) or max(a, b) -> element type
    if func_name == "max" {
        if call.arguments.args.len() == 2 {
            // max(a, b) -> std::cmp::max(a, b)
            let a = lower_expr(&call.arguments.args[0], ctx)?;
            let b = lower_expr(&call.arguments.args[1], ctx)?;
            let result_ty = a.ty().clone();
            return Some(HirExpr::Call {
                func: "max".to_string(),
                args: vec![a, b],
                ty: result_ty,
            });
        } else if call.arguments.args.len() == 1 {
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let elem_ty = if let Type::List(elem) = arg.ty() {
                *elem.clone()
            } else {
                ctx.error(format!(
                    "max() argument must be a list, got '{}'",
                    arg.ty().display_name()
                ));
                return None;
            };
            // Returns Option[T] = T | None (safe: None on empty list)
            return Some(HirExpr::Call {
                func: "max".to_string(),
                args: vec![arg],
                ty: Type::Union(vec![elem_ty, Type::None]),
            });
        }
        ctx.error("max() takes 1 or 2 arguments".to_string());
        return None;
    }

    // sum(iterable) -> element type (int or float)
    if func_name == "sum" {
        if call.arguments.args.len() != 1 {
            ctx.error("sum() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let elem_ty = if let Type::List(elem) = arg.ty() {
            *elem.clone()
        } else {
            ctx.error(format!(
                "sum() argument must be a list, got '{}'",
                arg.ty().display_name()
            ));
            return None;
        };
        return Some(HirExpr::Call {
            func: "sum".to_string(),
            args: vec![arg],
            ty: elem_ty,
        });
    }

    // sorted(iterable) -> list of element type
    if func_name == "sorted" {
        if call.arguments.args.len() != 1 {
            ctx.error("sorted() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let list_ty = if let Type::List(_) = arg.ty() {
            arg.ty().clone()
        } else {
            ctx.error(format!(
                "sorted() argument must be a list, got '{}'",
                arg.ty().display_name()
            ));
            return None;
        };
        return Some(HirExpr::Call {
            func: "sorted".to_string(),
            args: vec![arg],
            ty: list_ty,
        });
    }

    // reversed(iterable) -> list of element type
    if func_name == "reversed" {
        if call.arguments.args.len() != 1 {
            ctx.error("reversed() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let list_ty = if let Type::List(_) = arg.ty() {
            arg.ty().clone()
        } else {
            ctx.error(format!(
                "reversed() argument must be a list, got '{}'",
                arg.ty().display_name()
            ));
            return None;
        };
        return Some(HirExpr::Call {
            func: "reversed".to_string(),
            args: vec![arg],
            ty: list_ty,
        });
    }

    // enumerate(iterable) -> list of (int, element) tuples
    if func_name == "enumerate" {
        if call.arguments.args.len() != 1 {
            ctx.error("enumerate() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let elem_ty = if let Type::List(elem) = arg.ty() {
            *elem.clone()
        } else {
            ctx.error(format!(
                "enumerate() argument must be a list, got '{}'",
                arg.ty().display_name()
            ));
            return None;
        };
        let tuple_ty = Type::Tuple(vec![Type::Int, elem_ty]);
        let result_ty = Type::List(Box::new(tuple_ty));
        return Some(HirExpr::Call {
            func: "enumerate".to_string(),
            args: vec![arg],
            ty: result_ty,
        });
    }

    // zip(iter1, iter2) -> list of (elem1, elem2) tuples
    if func_name == "zip" {
        if call.arguments.args.len() != 2 {
            ctx.error("zip() takes exactly 2 arguments".to_string());
            return None;
        }
        let arg1 = lower_expr(&call.arguments.args[0], ctx)?;
        let arg2 = lower_expr(&call.arguments.args[1], ctx)?;
        let elem1 = if let Type::List(elem) = arg1.ty() {
            *elem.clone()
        } else {
            ctx.error(format!(
                "zip() argument 1 must be a list, got '{}'",
                arg1.ty().display_name()
            ));
            return None;
        };
        let elem2 = if let Type::List(elem) = arg2.ty() {
            *elem.clone()
        } else {
            ctx.error(format!(
                "zip() argument 2 must be a list, got '{}'",
                arg2.ty().display_name()
            ));
            return None;
        };
        let tuple_ty = Type::Tuple(vec![elem1, elem2]);
        let result_ty = Type::List(Box::new(tuple_ty));
        return Some(HirExpr::Call {
            func: "zip".to_string(),
            args: vec![arg1, arg2],
            ty: result_ty,
        });
    }

    // any(iterable) -> bool
    if func_name == "any" {
        if call.arguments.args.len() != 1 {
            ctx.error("any() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(HirExpr::Call {
            func: "any".to_string(),
            args: vec![arg],
            ty: Type::Bool,
        });
    }

    // all(iterable) -> bool
    if func_name == "all" {
        if call.arguments.args.len() != 1 {
            ctx.error("all() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(HirExpr::Call {
            func: "all".to_string(),
            args: vec![arg],
            ty: Type::Bool,
        });
    }

    // map(func, iterable) -> list
    if func_name == "map" {
        if call.arguments.args.len() != 2 {
            ctx.error("map() takes exactly 2 arguments (function, iterable)".to_string());
            return None;
        }
        // Lower iterable first to get element type for contextual lambda typing
        let iter_arg = lower_expr(&call.arguments.args[1], ctx)?;
        let elem_ty = match iter_arg.ty() {
            Type::List(elem) => *elem.clone(),
            _ => Type::Any,
        };
        // Lower lambda with contextual typing
        let func_arg = lower_lambda_with_context(&call.arguments.args[0], &[elem_ty], ctx)?;
        // Determine result element type from the function's return type
        let result_elem_ty = match func_arg.ty() {
            Type::Function(ft) => *ft.return_type.clone(),
            _ => Type::Any,
        };
        let result_ty = Type::List(Box::new(result_elem_ty));
        return Some(HirExpr::Call {
            func: "map".to_string(),
            args: vec![func_arg, iter_arg],
            ty: result_ty,
        });
    }

    // filter(func, iterable) -> list (same element type)
    if func_name == "filter" {
        if call.arguments.args.len() != 2 {
            ctx.error("filter() takes exactly 2 arguments (function, iterable)".to_string());
            return None;
        }
        // Lower iterable first to get element type for contextual lambda typing
        let iter_arg = lower_expr(&call.arguments.args[1], ctx)?;
        let elem_ty = match iter_arg.ty() {
            Type::List(elem) => *elem.clone(),
            _ => Type::Any,
        };
        // Lower lambda with contextual typing
        let func_arg = lower_lambda_with_context(&call.arguments.args[0], &[elem_ty], ctx)?;
        let result_ty = iter_arg.ty().clone();
        return Some(HirExpr::Call {
            func: "filter".to_string(),
            args: vec![func_arg, iter_arg],
            ty: result_ty,
        });
    }

    // open(path, mode="r") -> FileHandle  — built-in file open (raises IOError on failure)
    // Matches Python's open() behavior: raises on error, returns FileHandle directly.
    if func_name == "open" {
        let n_args = call.arguments.args.len();
        let _n_kwargs = call.arguments.keywords.len();
        let path_arg = if n_args >= 1 {
            lower_expr(&call.arguments.args[0], ctx)?
        } else {
            ctx.error(
                "open() requires at least 1 argument: open(path) or open(path, mode)".to_string(),
            );
            return None;
        };
        let mode_arg = if n_args >= 2 {
            lower_expr(&call.arguments.args[1], ctx)?
        } else if let Some(kw) = call
            .arguments
            .keywords
            .iter()
            .find(|k| k.arg.as_deref() == Some("mode"))
        {
            lower_expr(&kw.value, ctx)?
        } else {
            HirExpr::StringLiteral("r".to_string())
        };
        // Return type: FileHandle (raises IOError on failure — used in try/except blocks)
        // FileHandle methods are defined in io.sifr; register them here for type checking.
        let io_err_ty = Type::Class {
            name: "IOError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: None,
        };
        let file_handle_ty = Type::Class {
            name: "FileHandle".to_string(),
            fields: vec![
                ("_handle".to_string(), Type::Int),
                ("_mode".to_string(), Type::Str),
            ],
            methods: vec![
                (
                    "read".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(Box::new(Type::Str), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "write".to_string(),
                    FunctionType::all_borrow(
                        vec![("data".to_string(), Type::Str)],
                        Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "readline".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(
                            Box::new(Type::Union(vec![Type::Str, Type::None])),
                            Box::new(io_err_ty.clone()),
                        ),
                    ),
                ),
                (
                    "readlines".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(
                            Box::new(Type::List(Box::new(Type::Str))),
                            Box::new(io_err_ty.clone()),
                        ),
                    ),
                ),
                (
                    "close".to_string(),
                    FunctionType::all_borrow(vec![], Type::None),
                ),
                (
                    "read_bytes".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(
                            Box::new(Type::List(Box::new(Type::Int))),
                            Box::new(io_err_ty.clone()),
                        ),
                    ),
                ),
                (
                    "write_bytes".to_string(),
                    FunctionType::all_borrow(
                        vec![("data".to_string(), Type::List(Box::new(Type::Int)))],
                        Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "__enter__".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Class {
                            name: "FileHandle".to_string(),
                            fields: vec![
                                ("_handle".to_string(), Type::Int),
                                ("_mode".to_string(), Type::Str),
                            ],
                            methods: vec![],
                            parent_class: None,
                        },
                    ),
                ),
                (
                    "__exit__".to_string(),
                    FunctionType::all_borrow(vec![], Type::None),
                ),
            ],
            parent_class: None,
        };
        // Register FileHandle in the class types so method calls work
        ctx.class_types
            .insert("FileHandle".to_string(), file_handle_ty.clone());
        // Register IOError as a possible exception from this call
        ctx.try_block_error_types.insert("IOError".to_string());
        return Some(HirExpr::Call {
            func: "builtin_open".to_string(),
            args: vec![path_arg, mode_arg],
            ty: file_handle_ty,
        });
    }

    // Check if this is a Callable-typed variable being called
    let callable_info = ctx.scope.lookup(&func_name).and_then(|info| {
        if let Type::Callable(ref param_types, ref conventions, ref ret_type) = info.ty {
            Some((param_types.clone(), conventions.clone(), *ret_type.clone()))
        } else {
            None
        }
    });
    if let Some((param_types, conventions, ret_type)) = callable_info {
        // Lower arguments
        let mut args = Vec::new();
        for arg in &call.arguments.args {
            let expr = lower_expr(arg, ctx)?;
            args.push(expr);
        }
        if args.len() != param_types.len() {
            ctx.error(format!(
                "callable '{}' expects {} argument(s), got {}",
                func_name,
                param_types.len(),
                args.len()
            ));
            return None;
        }
        // Type check arguments and apply convention-aware move tracking
        for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() {
            if !arg.ty().is_assignable_to(param_ty) {
                ctx.error(format!(
                    "argument {} of callable '{}': expected '{}', got '{}'",
                    i + 1,
                    func_name,
                    param_ty.display_name(),
                    arg.ty().display_name()
                ));
            }
            // Apply move tracking based on convention
            let convention = conventions
                .get(i)
                .copied()
                .unwrap_or(ParamConvention::Borrow);
            if convention == ParamConvention::Own {
                // Own convention: transfer ownership, mark variable as moved
                if let HirExpr::Name { name, ty } = arg {
                    if ty.ownership() == OwnershipKind::Move {
                        ctx.scope.mark_moved(name);
                    }
                }
            }
            // Borrow/MutBorrow: no move, variable remains usable
        }
        return Some(HirExpr::Call {
            func: func_name,
            args,
            ty: ret_type,
        });
    }

    let ft = ctx.functions.get(&func_name).cloned().or_else(|| {
        ctx.error(format!("undefined function: '{func_name}'"));
        None
    })?;

    // Lower positional arguments
    let mut positional_args = Vec::new();
    for arg in &call.arguments.args {
        let expr = lower_expr(arg, ctx)?;
        positional_args.push(expr);
    }

    // Lower keyword arguments
    let mut keyword_args: Vec<(String, HirExpr)> = Vec::new();
    for kw in &call.arguments.keywords {
        if let Some(ref arg_name) = kw.arg {
            let expr = lower_expr(&kw.value, ctx)?;
            keyword_args.push((arg_name.to_string(), expr));
        }
    }

    // Resolve keyword arguments to positional order
    let args = if func_name == "print" {
        // print() is special - just pass positional args
        positional_args
    } else if keyword_args.is_empty() {
        // No keyword args - check count and use positional directly
        // Allow fewer args if there are defaults
        let is_vararg = ctx.vararg_functions.contains(&func_name);
        if is_vararg && positional_args.len() >= ft.params.len() - 1 {
            // Vararg function: collect extra args into a list for the last param
            let regular_count = ft.params.len() - 1; // all params except the vararg
            let mut args = Vec::new();
            for arg in positional_args.iter().take(regular_count) {
                args.push(arg.clone());
            }
            // Collect remaining args into a list literal
            let vararg_elements: Vec<HirExpr> = positional_args[regular_count..].to_vec();
            let elem_ty = if let Type::List(ref elem) = ft.params[regular_count].1 {
                *elem.clone()
            } else {
                Type::Any
            };
            args.push(HirExpr::ListLiteral {
                elements: vararg_elements,
                ty: Type::List(Box::new(elem_ty)),
            });
            // Skip the normal argument handling below
            let is_constructor = ctx.class_types.contains_key(&func_name);
            if is_constructor {
                let ty = ctx.class_types.get(&func_name).unwrap().clone();
                return Some(HirExpr::ConstructorCall {
                    class_name: func_name,
                    args,
                    ty,
                });
            }
            return Some(HirExpr::Call {
                func: func_name,
                args,
                ty: *ft.return_type.clone(),
            });
        } else if positional_args.len() > ft.params.len() {
            ctx.error(format!(
                "function '{}' expects at most {} argument(s), got {}",
                func_name,
                ft.params.len(),
                positional_args.len()
            ));
            return None;
        }
        // Fill in defaults for missing arguments
        if positional_args.len() < ft.params.len() {
            let defaults = ctx.function_defaults.get(&func_name).cloned();
            let mut filled = positional_args;
            for i in filled.len()..ft.params.len() {
                if let Some(ref defs) = defaults {
                    if let Some((_, default_expr)) = defs.iter().find(|(idx, _)| *idx == i) {
                        filled.push(default_expr.clone());
                    } else {
                        ctx.error(format!(
                            "function '{}': missing argument '{}' with no default value",
                            func_name, ft.params[i].0
                        ));
                        return None;
                    }
                } else {
                    ctx.error(format!(
                        "function '{}': missing argument '{}' with no default value",
                        func_name, ft.params[i].0
                    ));
                    return None;
                }
            }
            filled
        } else {
            positional_args
        }
    } else {
        // Resolve keyword arguments into positional order
        let mut resolved = Vec::new();
        let mut used_kwargs: std::collections::HashSet<String> = std::collections::HashSet::new();
        let defaults = ctx.function_defaults.get(&func_name).cloned();

        // Check: no positional args after keyword args (already enforced by parser)
        for (i, (param_name, _param_ty, _)) in ft.params.iter().enumerate() {
            if i < positional_args.len() {
                // Check no duplicate keyword for this position
                if keyword_args.iter().any(|(k, _)| k == param_name) {
                    ctx.error(format!(
                        "function '{func_name}': argument '{param_name}' given both positionally and as keyword"
                    ));
                    return None;
                }
                resolved.push(positional_args[i].clone());
            } else if let Some(pos) = keyword_args.iter().position(|(k, _)| k == param_name) {
                resolved.push(keyword_args[pos].1.clone());
                used_kwargs.insert(param_name.clone());
            } else {
                // Try to fill from default values
                if let Some(ref defs) = defaults {
                    if let Some((_, default_expr)) = defs.iter().find(|(idx, _)| *idx == i) {
                        resolved.push(default_expr.clone());
                    } else {
                        ctx.error(format!(
                            "function '{func_name}': missing argument '{param_name}' with no default value"
                        ));
                        return None;
                    }
                } else {
                    ctx.error(format!(
                        "function '{func_name}': missing argument '{param_name}' with no default value"
                    ));
                    return None;
                }
            }
        }

        // Check for unknown keyword arguments
        for (kw_name, _) in &keyword_args {
            if !ft.params.iter().any(|(p, _, _)| p == kw_name) {
                ctx.error(format!(
                    "function '{func_name}': unexpected keyword argument '{kw_name}'"
                ));
                return None;
            }
        }

        resolved
    };

    // Check argument types (skip for print)
    if func_name != "print" {
        let is_generic_function = ctx.generic_functions.contains_key(&func_name);
        for (i, (arg, (param_name, param_ty, _))) in args.iter().zip(ft.params.iter()).enumerate() {
            if is_generic_function {
                let mut type_vars = Vec::new();
                collect_type_vars(param_ty, &mut type_vars);
                if !type_vars.is_empty() {
                    // Generic params are validated after binding/substitution.
                    continue;
                }
            }
            if !arg.ty().is_assignable_to(param_ty) {
                ctx.error(format!(
                    "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                    i + 1,
                    param_name,
                    func_name,
                    param_ty.display_name(),
                    arg.ty().display_name()
                ));
            }
        }
    }

    // Exclusivity check: enforce that the same variable is not passed as mut twice,
    // or as both mut and immutable borrow in the same call.
    {
        let mut mut_borrowed: Vec<String> = Vec::new();
        let mut immut_borrowed: Vec<String> = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if let HirExpr::Name { name, ty } = arg {
                if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                    let convention = ft
                        .params
                        .get(i)
                        .map(|(_, _, c)| *c)
                        .unwrap_or(ParamConvention::Borrow);
                    match convention {
                        ParamConvention::MutBorrow => {
                            if mut_borrowed.contains(name) {
                                ctx.error(format!(
                                    "cannot borrow '{name}' as mutable more than once in the same call to '{func_name}'"
                                ));
                            } else if immut_borrowed.contains(name) {
                                ctx.error(format!(
                                    "cannot borrow '{name}' as mutable because it is already borrowed as immutable in the same call to '{func_name}'"
                                ));
                            }
                            mut_borrowed.push(name.clone());
                        }
                        ParamConvention::Borrow => {
                            if mut_borrowed.contains(name) {
                                ctx.error(format!(
                                    "cannot borrow '{name}' as immutable because it is already borrowed as mutable in the same call to '{func_name}'"
                                ));
                            }
                            immut_borrowed.push(name.clone());
                        }
                        ParamConvention::Own => {} // ownership transfer, no borrow conflict
                    }
                }
            }
        }
    }

    // Track ownership: only mark arguments as moved when the parameter convention is Own
    // and the argument type is Move. Borrow and MutBorrow do not consume the value.
    for (i, arg) in args.iter().enumerate() {
        if let HirExpr::Name { name, ty } = arg {
            if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                let convention = ft
                    .params
                    .get(i)
                    .map(|(_, _, c)| *c)
                    .unwrap_or(ParamConvention::Borrow);
                if convention == ParamConvention::Own {
                    ctx.scope.mark_moved(name);
                }
            }
        }
    }

    // If this is a generic function, infer type variable bindings and substitute
    let return_type = if ctx.generic_functions.contains_key(&func_name) {
        let mut bindings = HashMap::new();
        for (arg, (_, param_ty, _)) in args.iter().zip(ft.params.iter()) {
            infer_type_var_bindings(param_ty, arg.ty(), &mut bindings);
        }
        // Re-check argument types after TypeVar substitution so repeated type
        // parameters (e.g. assert_eq[T](a: T, b: T)) enforce consistent types.
        if func_name != "print" {
            for (i, (arg, (param_name, param_ty, _))) in
                args.iter().zip(ft.params.iter()).enumerate()
            {
                let concrete_param_ty = substitute_type_vars(param_ty, &bindings);
                let mut unresolved_type_vars = Vec::new();
                collect_type_vars(&concrete_param_ty, &mut unresolved_type_vars);
                if !unresolved_type_vars.is_empty() {
                    if !is_compatible_with_unresolved_typevars(arg.ty(), &concrete_param_ty) {
                        ctx.error(format!(
                            "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                            i + 1,
                            param_name,
                            func_name,
                            concrete_param_ty.display_name(),
                            arg.ty().display_name()
                        ));
                    }
                    continue;
                }
                if !arg.ty().is_assignable_to(&concrete_param_ty) {
                    ctx.error(format!(
                        "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                        i + 1,
                        param_name,
                        func_name,
                        concrete_param_ty.display_name(),
                        arg.ty().display_name()
                    ));
                }
            }
        }
        // Check protocol bounds on type parameters (scoped to this function)
        let mut bound_errors: Vec<String> = Vec::new();
        if let Some(owner_bounds) = ctx.type_param_bounds.get(&func_name) {
            for (tv_name, concrete_ty) in &bindings {
                if let Some(specs) = owner_bounds.get(tv_name) {
                    let mut required_bounds = Vec::new();
                    let mut constraints = Vec::new();
                    for spec in specs {
                        if let Some(constraint_name) = decode_typevar_constraint(spec) {
                            constraints.push(constraint_name.to_string());
                        } else {
                            required_bounds.push(spec.clone());
                        }
                    }

                    for bound in required_bounds {
                        if !type_satisfies_bound(concrete_ty, &bound, ctx) {
                            bound_errors.push(format!(
                                "type '{}' does not implement protocol '{}' required by type parameter '{}'",
                                concrete_ty.display_name(),
                                bound,
                                tv_name
                            ));
                        }
                    }

                    if !constraints.is_empty()
                        && !constraints.iter().any(|constraint| {
                            type_satisfies_constraint(concrete_ty, constraint, ctx)
                        })
                    {
                        bound_errors.push(format!(
                            "type '{}' does not satisfy constraints ({}) required by type parameter '{}'",
                            concrete_ty.display_name(),
                            constraints.join(", "),
                            tv_name
                        ));
                    }
                }
            }
        }
        for err in bound_errors {
            ctx.error(err);
        }
        if bindings.is_empty() {
            *ft.return_type
        } else {
            substitute_type_vars(&ft.return_type, &bindings)
        }
    } else {
        *ft.return_type
    };

    // If this is a class constructor call, emit ConstructorCall
    if ctx.class_types.contains_key(&func_name) {
        Some(HirExpr::ConstructorCall {
            class_name: func_name,
            args,
            ty: return_type,
        })
    } else {
        Some(HirExpr::Call {
            func: func_name,
            args,
            ty: return_type,
        })
    }
}

pub(super) fn lower_fstring(fstring: &ExprFString, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut parts = Vec::new();

    for part in &fstring.value {
        match part {
            sifr_python_ast::FStringPart::Literal(s) => {
                parts.push(HirFStringPart::Literal(s.to_string()));
            }
            sifr_python_ast::FStringPart::FString(fs) => {
                for element in &fs.elements {
                    match element {
                        FStringElement::Literal(lit) => {
                            parts.push(HirFStringPart::Literal(lit.value.to_string()));
                        }
                        FStringElement::Expression(expr_elem) => {
                            let expr = lower_expr(&expr_elem.expression, ctx)?;
                            parts.push(HirFStringPart::Expr(expr));
                        }
                    }
                }
            }
        }
    }

    Some(HirExpr::FString {
        parts,
        ty: Type::Str,
    })
}

pub(super) fn lower_tuple_unpack_assign(
    tuple: &ExprTuple,
    value: &Expr,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    // Extract target names
    let mut target_names = Vec::new();
    for elt in &tuple.elts {
        if let Expr::Name(n) = elt {
            target_names.push(n.id.clone());
        } else {
            ctx.error("tuple unpacking target must be a simple name".to_string());
            return None;
        }
    }

    // Lower the value expression
    let value_expr = lower_expr(value, ctx)?;
    let value_ty = value_expr.ty().clone();

    // Check that the value is a tuple with matching length
    let elem_types = if let Type::Tuple(elems) = &value_ty {
        if elems.len() != target_names.len() {
            ctx.error(format!(
                "tuple unpacking: expected {} values, got {}",
                target_names.len(),
                elems.len()
            ));
            return None;
        }
        elems.clone()
    } else {
        ctx.error(format!(
            "cannot unpack non-tuple type '{}'",
            value_ty.display_name()
        ));
        return None;
    };

    // Define variables in scope
    let mut targets = Vec::new();
    for (name, ty) in target_names.into_iter().zip(elem_types.into_iter()) {
        ctx.scope.define(name.clone(), ty.clone());
        targets.push((name, ty));
    }

    Some(HirStmt::TupleUnpack {
        targets,
        value: value_expr,
    })
}

pub(super) fn lower_star_unpack_assign(
    tuple: &ExprTuple,
    value: &Expr,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let value_expr = lower_expr(value, ctx)?;
    let value_ty = value_expr.ty().clone();

    // Get the element type from the list
    let elem_ty = if let Type::List(elem) = &value_ty {
        *elem.clone()
    } else {
        ctx.error("star unpacking requires a list type".to_string());
        return None;
    };

    let mut before = Vec::new();
    let mut star: Option<(String, Type)> = None;
    let mut after = Vec::new();

    for elt in &tuple.elts {
        match elt {
            Expr::Starred(starred) => {
                if star.is_some() {
                    ctx.error("multiple starred expressions in assignment".to_string());
                    return None;
                }
                if let Expr::Name(n) = starred.value.as_ref() {
                    let name = n.id.clone();
                    let star_ty = Type::List(Box::new(elem_ty.clone()));
                    ctx.scope.define(name.clone(), star_ty.clone());
                    star = Some((name, star_ty));
                } else {
                    ctx.error("starred target must be a simple name".to_string());
                    return None;
                }
            }
            Expr::Name(n) => {
                let name = n.id.clone();
                ctx.scope.define(name.clone(), elem_ty.clone());
                if star.is_none() {
                    before.push((name, elem_ty.clone()));
                } else {
                    after.push((name, elem_ty.clone()));
                }
            }
            _ => {
                ctx.error("star unpacking target must be a simple name".to_string());
                return None;
            }
        }
    }

    let star = star.unwrap_or_else(|| {
        ctx.error("star unpacking requires a starred expression".to_string());
        ("_".to_string(), Type::List(Box::new(elem_ty.clone())))
    });

    Some(HirStmt::StarUnpack {
        before,
        star,
        after,
        value: value_expr,
    })
}

pub(super) fn lower_list_literal(list: &ExprList, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_ty: Option<Type> = None;

    for elt in &list.elts {
        let expr = lower_expr(elt, ctx)?;
        let ty = expr.ty().clone();
        if let Some(ref expected) = elem_ty {
            if !ty.is_assignable_to(expected) {
                ctx.error(format!(
                    "list element type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    ty.display_name()
                ));
            }
        } else {
            elem_ty = Some(ty);
        }
        elements.push(expr);
    }

    let final_elem_ty = elem_ty.unwrap_or(Type::Any);
    let list_ty = Type::List(Box::new(final_elem_ty));

    Some(HirExpr::ListLiteral {
        elements,
        ty: list_ty,
    })
}

pub(super) fn lower_set_literal(set: &ExprSet, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_ty: Option<Type> = None;

    for elt in &set.elts {
        let expr = lower_expr(elt, ctx)?;
        let ty = expr.ty().clone();
        if let Some(ref expected) = elem_ty {
            if !ty.is_assignable_to(expected) {
                ctx.error(format!(
                    "set element type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    ty.display_name()
                ));
            }
        } else {
            elem_ty = Some(ty);
        }
        elements.push(expr);
    }

    let final_elem_ty = elem_ty.unwrap_or(Type::Any);
    let set_ty = Type::Set(Box::new(final_elem_ty));

    Some(HirExpr::SetLiteral {
        elements,
        ty: set_ty,
    })
}

pub(super) fn lower_dict_literal(dict: &ExprDict, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut key_ty: Option<Type> = None;
    let mut val_ty: Option<Type> = None;

    for item in &dict.items {
        if let Some(ref key_expr) = item.key {
            let key = lower_expr(key_expr, ctx)?;
            let kt = key.ty().clone();
            if let Some(ref expected) = key_ty {
                if !kt.is_assignable_to(expected) {
                    ctx.error(format!(
                        "dict key type mismatch: expected '{}', got '{}'",
                        expected.display_name(),
                        kt.display_name()
                    ));
                }
            } else {
                key_ty = Some(kt);
            }
            keys.push(key);
        } else {
            ctx.error("dict unpacking (**) not supported".to_string());
            return None;
        }

        let val = lower_expr(&item.value, ctx)?;
        let vt = val.ty().clone();
        if let Some(ref expected) = val_ty {
            if !vt.is_assignable_to(expected) {
                ctx.error(format!(
                    "dict value type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    vt.display_name()
                ));
            }
        } else {
            val_ty = Some(vt);
        }
        values.push(val);
    }

    let final_key_ty = key_ty.unwrap_or(Type::Any);
    let final_val_ty = val_ty.unwrap_or(Type::Any);
    let dict_ty = Type::Dict(Box::new(final_key_ty), Box::new(final_val_ty));

    Some(HirExpr::DictLiteral {
        keys,
        values,
        ty: dict_ty,
    })
}

pub(super) fn lower_tuple_literal(tuple: &ExprTuple, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_types = Vec::new();

    for elt in &tuple.elts {
        let expr = lower_expr(elt, ctx)?;
        elem_types.push(expr.ty().clone());
        elements.push(expr);
    }

    let tuple_ty = Type::Tuple(elem_types);

    Some(HirExpr::TupleLiteral {
        elements,
        ty: tuple_ty,
    })
}

pub(super) fn lower_subscript(sub: &ExprSubscript, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let object = lower_expr(&sub.value, ctx)?;
    let object_ty = object.ty().clone();

    // Check if the slice is a Slice expression (x[start:stop] or x[start:stop:step])
    if let Expr::Slice(slice_expr) = sub.slice.as_ref() {
        let start = if let Some(ref s) = slice_expr.lower {
            Some(Box::new(lower_expr(s, ctx)?))
        } else {
            None
        };
        let stop = if let Some(ref s) = slice_expr.upper {
            Some(Box::new(lower_expr(s, ctx)?))
        } else {
            None
        };
        let step = if let Some(ref s) = slice_expr.step {
            Some(Box::new(lower_expr(s, ctx)?))
        } else {
            None
        };

        // Determine result type for slicing
        let result_ty = match &object_ty {
            Type::List(elem_ty) => Type::List(elem_ty.clone()),
            Type::Str => Type::Str,
            Type::Tuple(elems) => {
                // Compile-time tuple slicing: indices must be integer literals
                if let (Some(start_expr), Some(stop_expr)) = (&start, &stop) {
                    if let (HirExpr::IntLiteral(s), HirExpr::IntLiteral(e)) =
                        (start_expr.as_ref(), stop_expr.as_ref())
                    {
                        let Ok(len_i64) = i64::try_from(elems.len()) else {
                            ctx.error("tuple too large for slicing index computation".to_string());
                            return Some(HirExpr::Slice {
                                object: Box::new(object),
                                start,
                                stop,
                                step,
                                ty: Type::Any,
                            });
                        };
                        let normalize = |idx: i64| if idx < 0 { len_i64 + idx } else { idx };
                        let s = normalize(*s);
                        let e = normalize(*e);
                        if s <= e {
                            if let (Ok(s_usize), Ok(e_usize)) =
                                (usize::try_from(s), usize::try_from(e))
                            {
                                if e_usize <= elems.len() {
                                    Type::Tuple(elems[s_usize..e_usize].to_vec())
                                } else {
                                    ctx.error("tuple slice indices out of range".to_string());
                                    Type::Any
                                }
                            } else {
                                ctx.error("tuple slice indices out of range".to_string());
                                Type::Any
                            }
                        } else {
                            ctx.error("tuple slice indices out of range".to_string());
                            Type::Any
                        }
                    } else {
                        ctx.error(
                            "tuple slicing requires compile-time constant indices".to_string(),
                        );
                        Type::Any
                    }
                } else {
                    // Partial slice on tuple
                    let s = start
                        .as_ref()
                        .and_then(|e| match e.as_ref() {
                            HirExpr::IntLiteral(v) => usize::try_from(*v).ok(),
                            _ => None,
                        })
                        .unwrap_or(0);
                    let e = stop
                        .as_ref()
                        .and_then(|e| match e.as_ref() {
                            HirExpr::IntLiteral(v) => usize::try_from(*v).ok(),
                            _ => None,
                        })
                        .unwrap_or(elems.len());
                    if s <= e && e <= elems.len() {
                        Type::Tuple(elems[s..e].to_vec())
                    } else {
                        Type::Tuple(elems.clone())
                    }
                }
            }
            _ => {
                ctx.error(format!("cannot slice type '{}'", object_ty.display_name()));
                Type::Any
            }
        };

        return Some(HirExpr::Slice {
            object: Box::new(object),
            start,
            stop,
            step,
            ty: result_ty,
        });
    }

    let index = lower_expr(&sub.slice, ctx)?;
    let index_ty = index.ty().clone();

    let result_ty = object_ty.index_result_type(&index_ty).unwrap_or_else(|| {
        ctx.error(format!(
            "cannot index type '{}' with '{}'",
            object_ty.display_name(),
            index_ty.display_name()
        ));
        Type::Any
    });

    Some(HirExpr::Index {
        object: Box::new(object),
        index: Box::new(index),
        ty: result_ty,
    })
}

pub(super) fn lower_attribute(attr: &ExprAttribute, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let field_name = attr.attr.to_string();

    // Check for enum variant access: Color.RED
    if let Expr::Name(name) = attr.value.as_ref() {
        let class_name = name.id.clone();
        if let Some(ty) = ctx.class_types.get(&class_name).cloned() {
            if let Type::Enum { ref variants, .. } = ty {
                if variants.iter().any(|(v, _)| v == &field_name) {
                    return Some(HirExpr::EnumVariant {
                        enum_name: class_name,
                        variant: field_name,
                        ty,
                    });
                }
            }
        }
    }

    let object = lower_expr(&attr.value, ctx)?;
    let object_ty = object.ty().clone();

    // Check if the object is a class instance with this field
    if let Type::Class {
        name: _, fields, ..
    } = &object_ty
    {
        if let Some((_, field_ty)) = fields.iter().find(|(n, _)| n == &field_name) {
            return Some(HirExpr::FieldAccess {
                object: Box::new(object),
                field: field_name,
                ty: field_ty.clone(),
            });
        }
        ctx.error(format!(
            "type '{}' has no field '{}'",
            object_ty.display_name(),
            field_name
        ));
        return None;
    }

    // Check if the object is an enum instance - access .name or .value
    if let Type::Enum {
        name: enum_name, ..
    } = &object_ty
    {
        match field_name.as_str() {
            "name" => {
                return Some(HirExpr::FieldAccess {
                    object: Box::new(object),
                    field: "name".to_string(),
                    ty: Type::Str,
                });
            }
            "value" => {
                return Some(HirExpr::FieldAccess {
                    object: Box::new(object),
                    field: "value".to_string(),
                    ty: Type::Int,
                });
            }
            _ => {
                ctx.error(format!(
                    "enum '{enum_name}' has no attribute '{field_name}'"
                ));
                return None;
            }
        }
    }

    // Not a class field access -- report unsupported
    ctx.error(format!(
        "attribute access '.{field_name}' is not supported as an expression; use as a method call"
    ));
    None
}

pub(super) fn lower_method_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    // Handle super().__init__() and super().method() calls
    if let Expr::Call(super_call) = attr.value.as_ref() {
        if let Expr::Name(name) = super_call.func.as_ref() {
            if name.id.as_str() == "super" {
                let method_name = attr.attr.to_string();
                if let Some(parent_name) = ctx.current_parent_class.clone() {
                    // Lower arguments
                    let mut args = Vec::new();
                    for arg in &call.arguments.args {
                        let expr = lower_expr(arg, ctx)?;
                        args.push(expr);
                    }

                    return Some(HirExpr::SuperCall {
                        parent_class: parent_name,
                        method: if method_name == "__init__" {
                            "new".to_string()
                        } else {
                            method_name
                        },
                        args,
                        ty: Type::None,
                    });
                }
                ctx.error("super() used outside of a class with a parent".to_string());
                return None;
            }
        }
    }

    // Handle ClassName.method() calls (classmethod/staticmethod)
    if let Expr::Name(name) = attr.value.as_ref() {
        let class_name = name.id.clone();
        if ctx.class_types.contains_key(&class_name) {
            let method_name = attr.attr.to_string();
            // Lower arguments
            let mut args = Vec::new();
            for arg in &call.arguments.args {
                let expr = lower_expr(arg, ctx)?;
                args.push(expr);
            }
            // Look up the method's return type from the class type
            if let Some(Type::Class { methods, .. }) = ctx.class_types.get(&class_name) {
                if let Some((_, ft)) = methods.iter().find(|(n, _)| n == &method_name) {
                    let return_ty = *ft.return_type.clone();
                    return Some(HirExpr::Call {
                        func: format!("{class_name}::{method_name}"),
                        args,
                        ty: return_ty,
                    });
                }
            }
            ctx.error(format!(
                "type '{class_name}' has no class/static method '{method_name}'"
            ));
            return None;
        }
    }

    let object = lower_expr(&attr.value, ctx)?;
    let object_ty = object.ty().clone();
    let method_name = attr.attr.to_string();

    // Lower arguments
    let mut args = Vec::new();
    for arg in &call.arguments.args {
        let expr = lower_expr(arg, ctx)?;
        args.push(expr);
    }

    // Resolve method return type based on object type and method name
    let return_ty = resolve_method_type(&object_ty, &method_name, &args, ctx)?;

    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method_name,
        args,
        ty: return_ty,
    })
}

/// Resolve the return type of a method call on a given type.
pub(super) fn resolve_method_type(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    ctx: &mut LowerCtx,
) -> Option<Type> {
    match object_ty {
        Type::List(elem_ty) => match method {
            "append" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.append() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                if !args[0].ty().is_assignable_to(elem_ty) {
                    ctx.error(format!(
                        "list.append() argument type '{}' is not compatible with list element type '{}'",
                        args[0].ty().display_name(),
                        elem_ty.display_name()
                    ));
                }
                Some(Type::None)
            }
            "extend" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.extend() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "insert" => {
                if args.len() != 2 {
                    ctx.error(format!(
                        "list.insert() takes exactly 2 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "clear" => {
                if !args.is_empty() {
                    ctx.error("list.clear() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    ctx.error("list.copy() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(elem_ty.clone()))
            }
            "reverse" => {
                if !args.is_empty() {
                    ctx.error("list.reverse() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "sort" => {
                if !args.is_empty() {
                    ctx.error("list.sort() takes no arguments in this milestone".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "count" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.count() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Int)
            }
            "contains" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.contains() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            "len" => {
                if !args.is_empty() {
                    ctx.error("list.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "pop" => {
                if !args.is_empty() {
                    ctx.error("list.pop() takes no arguments".to_string());
                    return None;
                }
                // pop() returns Option[T] = T | None
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            "popleft" => {
                if !args.is_empty() {
                    ctx.error("list.popleft() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            "appendleft" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.appendleft() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "remove" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.remove() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "index" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.index() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                // Returns Option[int] = int | None (safe: no panic if not found)
                Some(Type::Union(vec![Type::Int, Type::None]))
            }
            _ => {
                ctx.error(format!("list has no method '{method}'"));
                None
            }
        },
        Type::Dict(key_ty, val_ty) => match method {
            "len" => {
                if !args.is_empty() {
                    ctx.error("dict.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "keys" => {
                if !args.is_empty() {
                    ctx.error("dict.keys() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(key_ty.clone()))
            }
            "values" => {
                if !args.is_empty() {
                    ctx.error("dict.values() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(val_ty.clone()))
            }
            "items" => {
                if !args.is_empty() {
                    ctx.error("dict.items() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(Box::new(Type::Tuple(vec![
                    *key_ty.clone(),
                    *val_ty.clone(),
                ]))))
            }
            "update" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "dict.update() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "clear" => {
                if !args.is_empty() {
                    ctx.error("dict.clear() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    ctx.error("dict.copy() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Dict(key_ty.clone(), val_ty.clone()))
            }
            "contains" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "dict.contains() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            "get" => {
                if args.is_empty() || args.len() > 2 {
                    ctx.error(format!(
                        "dict.get() takes 1 or 2 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                if args.len() == 2 {
                    // dict.get(key, default) -> V (returns default if key not found)
                    Some(*val_ty.clone())
                } else {
                    // dict.get(key) -> V | None
                    Some(Type::Union(vec![*val_ty.clone(), Type::None]))
                }
            }
            "pop" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "dict.pop() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                // pop() returns Option[V] = V | None
                Some(Type::Union(vec![*val_ty.clone(), Type::None]))
            }
            _ => {
                ctx.error(format!("dict has no method '{method}'"));
                None
            }
        },
        Type::Set(elem_ty) => match method {
            "len" => {
                if !args.is_empty() {
                    ctx.error("set.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "add" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.add() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "remove" | "discard" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "contains" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.contains() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            "clear" => {
                if !args.is_empty() {
                    ctx.error("set.clear() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    ctx.error("set.copy() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Set(elem_ty.clone()))
            }
            "union" | "intersection" | "difference" | "symmetric_difference" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Set(elem_ty.clone()))
            }
            "issubset" | "issuperset" | "isdisjoint" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            "pop" => {
                if !args.is_empty() {
                    ctx.error("set.pop() takes no arguments".to_string());
                    return None;
                }
                // Returns Option[T] = T | None (safe: no panic on empty set)
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            _ => {
                ctx.error(format!("set has no method '{method}'"));
                None
            }
        },
        Type::Str => match method {
            "len" => Some(Type::Int),
            "upper" | "lower" | "strip" | "lstrip" | "rstrip" | "title" | "capitalize"
            | "swapcase" => Some(Type::Str),
            "startswith" | "endswith" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "str.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            "isdigit" | "isalpha" | "isalnum" | "isspace" | "isupper" | "islower" => {
                if !args.is_empty() {
                    ctx.error(format!("str.{method}() takes no arguments"));
                    return None;
                }
                Some(Type::Bool)
            }
            "split" => {
                if args.len() > 1 {
                    ctx.error(format!(
                        "str.split() takes 0 or 1 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::List(Box::new(Type::Str)))
            }
            "replace" => {
                if args.len() != 2 {
                    ctx.error(format!(
                        "str.replace() takes exactly 2 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Str)
            }
            "join" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "str.join() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Str)
            }
            "count" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "str.count() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Int)
            }
            "center" | "ljust" | "rjust" | "zfill" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "str.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Str)
            }
            "find" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "str.find() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                // find() returns Option[int] = int | None
                Some(Type::Union(vec![Type::Int, Type::None]))
            }
            _ => {
                ctx.error(format!("str has no method '{method}'"));
                None
            }
        },
        Type::Tuple(_) => match method {
            "len" => Some(Type::Int),
            "count" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "tuple.count() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Int)
            }
            _ => {
                ctx.error(format!("tuple has no method '{method}'"));
                None
            }
        },
        Type::Class {
            name,
            fields,
            methods,
            ..
        } => {
            if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) {
                // Check argument count
                if args.len() != ft.params.len() {
                    ctx.error(format!(
                        "{}.{}() takes {} argument(s), got {}",
                        name,
                        method,
                        ft.params.len(),
                        args.len()
                    ));
                    return None;
                }
                // Check argument types
                for (i, (arg, (param_name, param_ty, _))) in
                    args.iter().zip(ft.params.iter()).enumerate()
                {
                    if !arg.ty().is_assignable_to(param_ty) {
                        ctx.error(format!(
                            "argument {} ('{}') of {}.{}(): expected '{}', got '{}'",
                            i + 1,
                            param_name,
                            name,
                            method,
                            param_ty.display_name(),
                            arg.ty().display_name()
                        ));
                    }
                }
                Some(*ft.return_type.clone())
            } else if let Some((_, field_ty)) = fields.iter().find(|(n, _)| n == method) {
                // Check if the field is a Callable type — allow calling it like a method
                if let Type::Callable(param_types, _, ret_type) = field_ty {
                    if args.len() != param_types.len() {
                        ctx.error(format!(
                            "{}.{}() (callable field) takes {} argument(s), got {}",
                            name,
                            method,
                            param_types.len(),
                            args.len()
                        ));
                        return None;
                    }
                    for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() {
                        if !arg.ty().is_assignable_to(param_ty) {
                            ctx.error(format!(
                                "argument {} of {}.{}(): expected '{}', got '{}'",
                                i + 1,
                                name,
                                method,
                                param_ty.display_name(),
                                arg.ty().display_name()
                            ));
                        }
                    }
                    Some(*ret_type.clone())
                } else {
                    ctx.error(format!(
                        "field '{}' of class '{}' is not callable (type: '{}')",
                        method,
                        name,
                        field_ty.display_name()
                    ));
                    None
                }
            } else {
                ctx.error(format!("class '{name}' has no method '{method}'"));
                None
            }
        }
        Type::Protocol { name, methods, .. } => {
            if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) {
                if args.len() != ft.params.len() {
                    ctx.error(format!(
                        "{}.{}() takes {} argument(s), got {}",
                        name,
                        method,
                        ft.params.len(),
                        args.len()
                    ));
                }
                Some(*ft.return_type.clone())
            } else {
                ctx.error(format!("protocol '{name}' has no method '{method}'"));
                None
            }
        }
        Type::Newtype { name, inner } => {
            // Newtype has a built-in `value()` method that returns the inner type
            if method == "value" {
                if !args.is_empty() {
                    ctx.error(format!("{name}.value() takes no arguments"));
                    return None;
                }
                Some(*inner.clone())
            } else {
                // Delegate to the inner type's methods
                resolve_method_type(inner, method, args, ctx)
            }
        }
        Type::Enum { name, .. } => {
            match method {
                "name" => {
                    if !args.is_empty() {
                        ctx.error(format!("{name}.name() takes no arguments"));
                        return None;
                    }
                    Some(Type::Str)
                }
                "value" => {
                    if !args.is_empty() {
                        ctx.error(format!("{name}.value() takes no arguments"));
                        return None;
                    }
                    Some(Type::Int)
                }
                _ => {
                    // Check user-defined methods registered in functions
                    let method_key = format!("{name}.{method}");
                    if let Some(ft) = ctx.functions.get(&method_key).cloned() {
                        return Some(*ft.return_type.clone());
                    }
                    ctx.error(format!("enum '{name}' has no method '{method}'"));
                    None
                }
            }
        }
        Type::BigInt => {
            if method == "clone" {
                if !args.is_empty() {
                    ctx.error("bigint.clone() takes no arguments".to_string());
                    return None;
                }
                Some(Type::BigInt)
            } else {
                ctx.error(format!("type 'bigint' has no method '{method}'"));
                None
            }
        }
        Type::Decimal | Type::BigDecimal => {
            resolve_decimal_method_type(object_ty, method, args, ctx)
        }
        _ => {
            ctx.error(format!(
                "type '{}' has no method '{}'",
                object_ty.display_name(),
                method
            ));
            None
        }
    }
}

pub(super) fn lower_len_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        ctx.error(format!(
            "len() takes exactly 1 argument, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let arg_ty = arg.ty().clone();

    // len() works on str, list, dict, tuple, set
    // Also works on T|None where T is a valid len() argument (auto-unwrap)
    let effective_ty = if let Type::Union(members) = &arg_ty {
        let non_none: Vec<&Type> = members
            .iter()
            .filter(|m| !matches!(m, Type::None))
            .collect();
        if non_none.len() == 1 {
            non_none[0].clone()
        } else {
            arg_ty.clone()
        }
    } else {
        arg_ty.clone()
    };
    match &effective_ty {
        Type::Str | Type::List(_) | Type::Dict(_, _) | Type::Tuple(_) | Type::Set(_) => {
            Some(HirExpr::MethodCall {
                object: Box::new(arg),
                method: "len".to_string(),
                args: vec![],
                ty: Type::Int,
            })
        }
        _ => {
            ctx.error(format!(
                "len() argument must be a string, list, dict, or tuple, got '{}'",
                arg_ty.display_name()
            ));
            None
        }
    }
}

pub(super) fn lower_isinstance_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 2 {
        ctx.error(format!(
            "isinstance() takes exactly 2 arguments, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    // Extract the type name as a string literal so codegen can use it for match arms
    let type_name = match &call.arguments.args[1] {
        Expr::Name(n) => n.id.clone(),
        _ => "unknown".to_string(),
    };
    // isinstance() always returns bool -- the narrowing happens at the if-statement level
    // We pass both the variable and the type name string to codegen
    Some(HirExpr::Call {
        func: "isinstance".to_string(),
        args: vec![arg, HirExpr::StringLiteral(type_name)],
        ty: Type::Bool,
    })
}

pub(super) fn lower_reveal_type_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        ctx.error(format!(
            "reveal_type() takes exactly 1 argument, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let ty = arg.ty().clone();
    // Store the reveal_type diagnostic (not an error, just informational)
    ctx.reveal_types
        .push(format!("reveal_type: {}", ty.display_name()));
    // reveal_type returns the value unchanged, so we emit a print of the type at runtime
    // For now, just return the argument expression
    Some(arg)
}

pub(super) fn lower_range_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let args: Vec<_> = call.arguments.args.iter().collect();

    match args.len() {
        1 => {
            // range(end) -> 0..end
            let end = lower_expr(args[0], ctx)?;
            if end.ty() != &Type::Int {
                ctx.error(format!(
                    "range() argument must be 'int', got '{}'",
                    end.ty().display_name()
                ));
                return None;
            }
            Some(HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(end),
                step: None,
                ty: Type::Range,
            })
        }
        2 => {
            // range(start, end) -> start..end
            let start = lower_expr(args[0], ctx)?;
            let end = lower_expr(args[1], ctx)?;
            if start.ty() != &Type::Int {
                ctx.error(format!(
                    "range() start argument must be 'int', got '{}'",
                    start.ty().display_name()
                ));
                return None;
            }
            if end.ty() != &Type::Int {
                ctx.error(format!(
                    "range() end argument must be 'int', got '{}'",
                    end.ty().display_name()
                ));
                return None;
            }
            Some(HirExpr::RangeLiteral {
                start: Box::new(start),
                end: Box::new(end),
                step: None,
                ty: Type::Range,
            })
        }
        3 => {
            // range(start, end, step) -> (start..end).step_by(step)
            let start = lower_expr(args[0], ctx)?;
            let end = lower_expr(args[1], ctx)?;
            let step = lower_expr(args[2], ctx)?;
            Some(HirExpr::RangeLiteral {
                start: Box::new(start),
                end: Box::new(end),
                step: Some(Box::new(step)),
                ty: Type::Range,
            })
        }
        _ => {
            ctx.error(format!(
                "range() takes 1, 2, or 3 arguments, got {}",
                args.len()
            ));
            None
        }
    }
}

pub(super) fn lower_if_expr(if_expr: &ExprIf, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let condition = lower_expr(&if_expr.test, ctx)?;
    let then_expr = lower_expr(&if_expr.body, ctx)?;
    let else_expr = lower_expr(&if_expr.orelse, ctx)?;

    let then_ty = then_expr.ty().clone();
    let else_ty = else_expr.ty().clone();

    if !then_ty.is_assignable_to(&else_ty) && !else_ty.is_assignable_to(&then_ty) {
        ctx.error(format!(
            "if expression branches have incompatible types: '{}' and '{}'",
            then_ty.display_name(),
            else_ty.display_name()
        ));
        return None;
    }

    Some(HirExpr::IfExpr {
        condition: Box::new(condition),
        then_expr: Box::new(then_expr),
        else_expr: Box::new(else_expr),
        ty: then_ty,
    })
}

/// Lower a lambda or regular expression with contextual type information for parameters.
/// If the expression is a lambda, use `context_types` for untyped parameters.
/// If it's not a lambda, just lower it normally.
pub(super) fn lower_lambda_with_context(
    expr: &Expr,
    context_types: &[Type],
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if let Expr::Lambda(lambda) = expr {
        ctx.scope.push();

        let mut params = Vec::new();
        if let Some(ref parameters) = lambda.parameters {
            for (i, param) in parameters.args.iter().enumerate() {
                let param_name = param.parameter.name.to_string();
                let param_ty = if let Some(ref ann) = param.parameter.annotation {
                    resolve_annotation_expr(ann, ctx)
                } else if i < context_types.len() {
                    // Use contextual type
                    context_types[i].clone()
                } else {
                    Type::Any
                };
                ctx.scope.define(param_name.clone(), param_ty.clone());
                params.push(HirParam {
                    name: param_name,
                    ty: param_ty,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::default(),
                });
            }
        }

        let body = lower_expr(&lambda.body, ctx)?;
        let body_ty = body.ty().clone();

        ctx.scope.pop();

        let param_types: Vec<(String, Type)> = params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let fn_ty = Type::Function(FunctionType::new(param_types, body_ty));

        Some(HirExpr::Lambda {
            params,
            body: Box::new(body),
            ty: fn_ty,
        })
    } else {
        // Not a lambda, lower normally
        lower_expr(expr, ctx)
    }
}

pub(super) fn lower_lambda(lambda: &ExprLambda, ctx: &mut LowerCtx) -> Option<HirExpr> {
    ctx.scope.push();

    let mut params = Vec::new();
    if let Some(ref parameters) = lambda.parameters {
        for param in &parameters.args {
            let param_name = param.parameter.name.to_string();
            let param_ty = if let Some(ref ann) = param.parameter.annotation {
                resolve_annotation_expr(ann, ctx)
            } else {
                // Lambda params without annotations: infer as Any for now
                // Contextual typing will refine this at call sites
                Type::Any
            };
            ctx.scope.define(param_name.clone(), param_ty.clone());
            params.push(HirParam {
                name: param_name,
                ty: param_ty,
                default: None,
                keyword_only: false,
                convention: ParamConvention::default(),
            });
        }
    }

    let body = lower_expr(&lambda.body, ctx)?;
    let body_ty = body.ty().clone();

    ctx.scope.pop();

    // Build the function type for the lambda
    let param_types: Vec<(String, Type)> = params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let fn_ty = Type::Function(FunctionType::new(param_types, body_ty));

    Some(HirExpr::Lambda {
        params,
        body: Box::new(body),
        ty: fn_ty,
    })
}

pub(super) fn lower_list_comp(comp: &ExprListComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if comp.generators.is_empty() {
        ctx.error("list comprehension must have at least one generator".to_string());
        return None;
    }

    let mut generators = Vec::new();
    let num_gens = comp.generators.len();

    // Process each generator: push scope, define var, lower iter
    for gen in &comp.generators {
        let var_name = match &gen.target {
            Expr::Name(n) => n.id.clone(),
            Expr::Tuple(tup) => {
                let names: Vec<String> = tup
                    .elts
                    .iter()
                    .filter_map(|e| {
                        if let Expr::Name(n) = e {
                            Some(n.id.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                if names.len() != tup.elts.len() {
                    ctx.error(
                        "comprehension tuple target must contain only simple names".to_string(),
                    );
                    return None;
                }
                names.join(",")
            }
            _ => {
                ctx.error("comprehension target must be a simple name or tuple".to_string());
                return None;
            }
        };

        let iter_expr = lower_expr(&gen.iter, ctx)?;
        let iter_ty = iter_expr.ty().clone();
        let elem_ty = match &iter_ty {
            Type::List(elem) => *elem.clone(),
            Type::Set(elem) => *elem.clone(),
            Type::Str => Type::Str,
            Type::Range => Type::Int,
            Type::Dict(key, _) => *key.clone(),
            Type::Tuple(elems) if !elems.is_empty() => elems[0].clone(),
            _ => {
                ctx.error(format!(
                    "cannot iterate over type '{}'",
                    iter_ty.display_name()
                ));
                return None;
            }
        };

        ctx.scope.push();
        if var_name.contains(',') {
            let names: Vec<&str> = var_name.split(',').collect();
            if let Type::Tuple(elem_types) = &elem_ty {
                for (i, name) in names.iter().enumerate() {
                    let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                    ctx.scope.define((*name).to_string(), ty);
                }
            } else {
                for name in &names {
                    ctx.scope.define((*name).to_string(), Type::Any);
                }
            }
        } else {
            ctx.scope.define(var_name.clone(), elem_ty.clone());
        }

        let filter = if gen.ifs.is_empty() {
            None
        } else {
            let first = lower_expr(&gen.ifs[0], ctx)?;
            if gen.ifs.len() == 1 {
                Some(first)
            } else {
                let mut combined = first;
                for cond in &gen.ifs[1..] {
                    let next = lower_expr(cond, ctx)?;
                    combined = HirExpr::BoolOp {
                        op: "and".to_string(),
                        values: vec![combined, next],
                        ty: Type::Bool,
                    };
                }
                Some(combined)
            }
        };

        generators.push((var_name, iter_expr, filter));
    }

    // Lower the expression (all generator vars are in scope)
    let expr = lower_expr(&comp.elt, ctx)?;
    let expr_ty = expr.ty().clone();

    // Pop all scopes
    for _ in 0..num_gens {
        ctx.scope.pop();
    }

    let result_ty = Type::List(Box::new(expr_ty));

    Some(HirExpr::ListComp {
        expr: Box::new(expr),
        generators,
        ty: result_ty,
    })
}

pub(super) fn lower_set_comp(comp: &ExprSetComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut generators = Vec::new();
    let num_gens = comp.generators.len();
    for gen in &comp.generators {
        let var_name = if let Expr::Name(n) = &gen.target {
            n.id.clone()
        } else {
            ctx.error("set comprehension target must be a simple name".to_string());
            return None;
        };
        let iter_expr = lower_expr(&gen.iter, ctx)?;
        let iter_ty = iter_expr.ty().clone();
        let elem_ty = match &iter_ty {
            Type::List(elem) => *elem.clone(),
            Type::Set(elem) => *elem.clone(),
            Type::Range => Type::Int,
            _ => {
                ctx.error(format!(
                    "cannot iterate over type '{}'",
                    iter_ty.display_name()
                ));
                return None;
            }
        };
        ctx.scope.push();
        ctx.scope.define(var_name.clone(), elem_ty);
        let filter = if gen.ifs.is_empty() {
            None
        } else {
            Some(lower_expr(&gen.ifs[0], ctx)?)
        };
        generators.push((var_name, iter_expr, filter));
    }
    let expr = lower_expr(&comp.elt, ctx)?;
    let expr_ty = expr.ty().clone();
    for _ in 0..num_gens {
        ctx.scope.pop();
    }
    let result_ty = Type::Set(Box::new(expr_ty));
    Some(HirExpr::SetComp {
        expr: Box::new(expr),
        generators,
        ty: result_ty,
    })
}

pub(super) fn lower_dict_comp(comp: &ExprDictComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut generators = Vec::new();
    let num_gens = comp.generators.len();
    for gen in &comp.generators {
        let var_name = match &gen.target {
            Expr::Name(n) => n.id.clone(),
            Expr::Tuple(tup) => {
                let names: Vec<String> = tup
                    .elts
                    .iter()
                    .filter_map(|e| {
                        if let Expr::Name(n) = e {
                            Some(n.id.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                names.join(",")
            }
            _ => {
                ctx.error("dict comprehension target must be a simple name or tuple".to_string());
                return None;
            }
        };
        let iter_expr = lower_expr(&gen.iter, ctx)?;
        let iter_ty = iter_expr.ty().clone();
        let elem_ty = match &iter_ty {
            Type::List(elem) => *elem.clone(),
            Type::Set(elem) => *elem.clone(),
            Type::Range => Type::Int,
            Type::Dict(key, _) => *key.clone(),
            _ => {
                ctx.error(format!(
                    "cannot iterate over type '{}'",
                    iter_ty.display_name()
                ));
                return None;
            }
        };
        ctx.scope.push();
        if var_name.contains(',') {
            let names: Vec<&str> = var_name.split(',').collect();
            if let Type::Tuple(elem_types) = &elem_ty {
                for (i, name) in names.iter().enumerate() {
                    let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                    ctx.scope.define((*name).to_string(), ty);
                }
            } else {
                for name in &names {
                    ctx.scope.define((*name).to_string(), Type::Any);
                }
            }
        } else {
            ctx.scope.define(var_name.clone(), elem_ty);
        }
        let filter = if gen.ifs.is_empty() {
            None
        } else {
            Some(lower_expr(&gen.ifs[0], ctx)?)
        };
        generators.push((var_name, iter_expr, filter));
    }
    let key_expr = lower_expr(&comp.key, ctx)?;
    let val_expr = lower_expr(&comp.value, ctx)?;
    let key_ty = key_expr.ty().clone();
    let val_ty = val_expr.ty().clone();
    for _ in 0..num_gens {
        ctx.scope.pop();
    }
    let result_ty = Type::Dict(Box::new(key_ty), Box::new(val_ty));
    Some(HirExpr::DictComp {
        key_expr: Box::new(key_expr),
        val_expr: Box::new(val_expr),
        generators,
        ty: result_ty,
    })
}

pub(super) fn lower_generator_expr(gen: &ExprGenerator, ctx: &mut LowerCtx) -> Option<HirExpr> {
    // Only support single generator: (expr for var in iter) or (expr for var in iter if cond)
    if gen.generators.len() != 1 {
        ctx.error("only single-generator generator expressions are supported".to_string());
        return None;
    }

    let comp = &gen.generators[0];

    // Get the variable name
    let var_name = if let Expr::Name(n) = &comp.target {
        n.id.clone()
    } else {
        ctx.error("generator target must be a simple name".to_string());
        return None;
    };

    // Lower the iterable
    let iter_expr = lower_expr(&comp.iter, ctx)?;
    let iter_ty = iter_expr.ty().clone();

    // Determine element type from the iterable
    let elem_ty = match &iter_ty {
        Type::List(elem) => *elem.clone(),
        Type::Str => Type::Str,
        _ => {
            ctx.error(format!(
                "cannot iterate over type '{}'",
                iter_ty.display_name()
            ));
            return None;
        }
    };

    // Push scope and define the loop variable
    ctx.scope.push();
    ctx.scope.define(var_name.clone(), elem_ty.clone());

    // Lower the expression
    let expr = lower_expr(&gen.elt, ctx)?;
    let expr_ty = expr.ty().clone();

    // Lower the filter condition if present
    let filter = if comp.ifs.is_empty() {
        None
    } else {
        let first = lower_expr(&comp.ifs[0], ctx)?;
        if comp.ifs.len() == 1 {
            Some(Box::new(first))
        } else {
            let mut combined = first;
            for cond in &comp.ifs[1..] {
                let next = lower_expr(cond, ctx)?;
                combined = HirExpr::BoolOp {
                    op: "and".to_string(),
                    values: vec![combined, next],
                    ty: Type::Bool,
                };
            }
            Some(Box::new(combined))
        }
    };

    ctx.scope.pop();

    let result_ty = Type::List(Box::new(expr_ty));

    Some(HirExpr::GeneratorExpr {
        expr: Box::new(expr),
        var: var_name,
        iter: Box::new(iter_expr),
        filter,
        ty: result_ty,
    })
}

pub(super) fn lower_named_expr(named: &ExprNamed, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let name = if let Expr::Name(n) = named.target.as_ref() {
        n.id.clone()
    } else {
        ctx.error("walrus operator target must be a simple name".to_string());
        return None;
    };

    let value = lower_expr(&named.value, ctx)?;
    let ty = value.ty().clone();

    // Define the variable in the current scope
    ctx.scope.define(name.clone(), ty.clone());

    Some(HirExpr::WalrusExpr {
        name,
        value: Box::new(value),
        ty,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_python_parser::parse_module;

    fn lower_source(source: &str) -> Result<HirModule, Vec<LoweringError>> {
        let parsed = parse_module(source).expect("parse failed");
        lower_module(parsed.suite()).map(|r| r.module)
    }

    #[test]
    fn test_simple_function() {
        let module = lower_source("def add(a: int, b: int) -> int:\n    return a + b\n").unwrap();
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "add");
        assert_eq!(module.functions[0].return_type, Type::Int);
    }

    #[test]
    fn test_type_mismatch_error() {
        let result = lower_source("def main():\n    x: int = \"hello\"\n");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("type mismatch")));
    }

    #[test]
    fn test_undefined_variable() {
        let result = lower_source("def main():\n    print(x)\n");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("undefined variable")));
    }

    #[test]
    fn test_use_after_move() {
        // Under borrow-by-default, consume() needs `own` to move the argument.
        // Without `own`, the argument is borrowed and no move error occurs.
        let result = lower_source(
            "def consume(own s: str) -> str:\n    return s\ndef main():\n    s: str = \"hello\"\n    x: str = consume(s)\n    print(s)\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("moved value")));
    }

    #[test]
    fn test_borrow_by_default_no_move() {
        // Under borrow-by-default, passing to a function that borrows does NOT move.
        let result = lower_source(
            "def process(s: str) -> int:\n    return len(s)\ndef main():\n    s: str = \"hello\"\n    x: int = process(s)\n    print(s)\n"
        );
        assert!(
            result.is_ok(),
            "borrow-by-default should not cause use-after-move"
        );
    }

    #[test]
    fn test_copy_type_no_move() {
        let module =
            lower_source("def main():\n    x: int = 42\n    print(x)\n    print(x)\n").unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_while_loop() {
        let module =
            lower_source("def main():\n    i: int = 0\n    while i < 10:\n        i = i + 1\n")
                .unwrap();
        assert_eq!(module.functions.len(), 1);
        // Body should contain a Let and a While
        assert!(module.functions[0].body.len() >= 2);
        assert!(matches!(module.functions[0].body[1], HirStmt::While { .. }));
    }

    #[test]
    fn test_for_range() {
        let module =
            lower_source("def main():\n    for i in range(10):\n        print(i)\n").unwrap();
        assert_eq!(module.functions.len(), 1);
        assert!(matches!(module.functions[0].body[0], HirStmt::For { .. }));
    }

    #[test]
    fn test_for_range_start_end() {
        let module =
            lower_source("def main():\n    for i in range(1, 5):\n        print(i)\n").unwrap();
        assert_eq!(module.functions.len(), 1);
        assert!(matches!(module.functions[0].body[0], HirStmt::For { .. }));
    }

    #[test]
    fn test_break_outside_loop() {
        let result = lower_source("def main():\n    break\n");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("'break' outside of loop")));
    }

    #[test]
    fn test_continue_outside_loop() {
        let result = lower_source("def main():\n    continue\n");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("'continue' outside of loop")));
    }

    #[test]
    fn test_break_inside_loop() {
        let module = lower_source("def main():\n    while True:\n        break\n").unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_nested_loops() {
        let module = lower_source(
            "def main():\n    for i in range(3):\n        for j in range(2):\n            print(i)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_fstring_basic() {
        let module = lower_source(
            "def main():\n    name: str = \"Alice\"\n    msg: str = f\"Hello, {name}!\"\n    print(msg)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        // Should have 3 statements: let name, let msg, print
        assert_eq!(module.functions[0].body.len(), 3);
    }

    #[test]
    fn test_fstring_with_expression() {
        let module = lower_source(
            "def main():\n    a: int = 2\n    b: int = 3\n    print(f\"{a} + {b} = {a + b}\")\n",
        )
        .unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_tuple_unpack() {
        let module = lower_source(
            "def main():\n    pair: tuple[int, str] = (1, \"hello\")\n    x, y = pair\n    print(x)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        // Should have: let pair, tuple_unpack, print
        assert!(module.functions[0].body.len() >= 3);
        assert!(matches!(
            module.functions[0].body[1],
            HirStmt::TupleUnpack { .. }
        ));
    }

    #[test]
    fn test_tuple_unpack_wrong_count() {
        let result = lower_source(
            "def main():\n    pair: tuple[int, str] = (1, \"hello\")\n    x, y, z = pair\n",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("expected 3 values, got 2")));
    }

    #[test]
    fn test_tuple_unpack_non_tuple() {
        let result = lower_source("def main():\n    x: int = 42\n    a, b = x\n");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("cannot unpack non-tuple")));
    }

    #[test]
    fn test_for_tuple_target_requires_tuple_elements() {
        let result = lower_source(
            "def main():\n    nums: list[int] = [1, 2, 3]\n    for a, b in nums:\n        print(a)\n",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| {
            e.message
                .contains("for loop tuple target expects iterable elements of tuple type")
        }));
    }

    #[test]
    fn test_generic_class_subscript_requires_declared_type_params() {
        let result = lower_source(
            "T = TypeVar(\"T\")\nclass LegacyBox:\n    value: T\ndef f(x: LegacyBox[int]) -> int:\n    return 1\n",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("does not declare type parameters")));
    }

    #[test]
    fn test_generic_class_subscript_arity_mismatch_errors() {
        let result = lower_source(
            "class Pair[T]:\n    left: T\n    right: T\ndef f(x: Pair[int, str]) -> int:\n    return 1\n",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("expects 1 type argument(s), got 2")));
    }

    #[test]
    fn test_match_tuple_pattern_requires_tuple_subject() {
        let result = lower_source(
            "def main():\n    x: int = 1\n    match x:\n        case (a, b):\n            print(a)\n",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e
            .message
            .contains("tuple pattern requires subject of tuple type")));
    }

    #[test]
    fn test_match_tuple_pattern_arity_mismatch_errors() {
        let result = lower_source(
            "def main():\n    x: tuple[int, int] = (1, 2)\n    match x:\n        case (a, b, c):\n            print(a)\n",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e
            .message
            .contains("tuple pattern expects 3 element(s), subject has 2")));
    }

    #[test]
    fn test_protocol_bound_forwarding_accepts_conforming_typevar() {
        let result = lower_source(
            "class Runner(Protocol):\n    def run(self) -> int:\n        pass\n\nclass Job:\n    def run(self) -> int:\n        return 1\n\ndef use_runner[T: Runner](x: T) -> T:\n    return x\n\ndef relay_runner[U: Runner](x: U) -> U:\n    return use_runner(x)\n\ndef main():\n    j: Job = relay_runner(Job())\n    print(j.run())\n",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_protocol_bound_forwarding_rejects_unknown_bound() {
        let result = lower_source(
            "def take_missing[T: MissingBound](x: T) -> T:\n    return x\n\ndef relay_missing[U: MissingBound](x: U) -> U:\n    return take_missing(x)\n\ndef main():\n    print(1)\n",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| {
            e.message
                .contains("does not implement protocol 'MissingBound'")
        }));
    }

    #[test]
    fn test_protocol_bound_forwarding_rejects_non_conforming_typevar() {
        let result = lower_source(
            "class Readable(Protocol):\n    def read(self) -> str:\n        pass\n\nclass Closable(Protocol):\n    def close(self) -> None:\n        pass\n\ndef take_readable[T: Readable](x: T) -> T:\n    return x\n\ndef relay_bad[U: Closable](x: U) -> U:\n    return take_readable(x)\n\ndef main():\n    print(1)\n",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("does not implement protocol 'Readable'")));
    }
}
