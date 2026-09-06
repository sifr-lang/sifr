use crate::hir_nodes::HirStmt;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, Stmt, StmtClassDef, StmtFunctionDef};
use sifr_type_system::Type;

use super::LowerCtx;

pub(in crate::lower) fn is_error_class_with_ctx(class_def: &StmtClassDef, ctx: &LowerCtx) -> bool {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            let base_name = n.id.as_str();
            // The declaration-base pass distinguishes the builtin Error marker
            // from an imported data parent whose local spelling is Error. Local
            // declarations are provisional here, including class Error(Error).
            if base_name == "Error"
                && super::descriptor_declarations::data_parent_name(class_def.name.as_str(), ctx)
                    .as_deref()
                    != Some("Error")
            {
                return true;
            }
            let is_error = ctx.class_types.get(base_name).map_or_else(
                || ctx.error_types.contains(base_name),
                |ty| ty.is_builtin_error_base() || matches!(ty.resolve_alias(), Type::Class {
                    parent_class: Some(chain), ..
                } if chain.split('|').any(|parent| matches!(parent, "Error" | "sifr.builtin.Error"))),
            );
            if is_error {
                return true;
            }
        }
    }
    false
}

/// Check if a type is a valid error type (a class registered in `error_types`).
pub(in crate::lower) fn is_valid_error_type(ty: &Type, ctx: &LowerCtx) -> bool {
    match ty.resolve_alias() {
        Type::Class { name, .. } => ctx.error_types.contains(name),
        Type::Union(members) => {
            !members.is_empty()
                && members
                    .iter()
                    .all(|member| is_valid_error_type(member, ctx))
        }
        Type::TimeoutResult(inner) => is_valid_error_type(inner, ctx),
        Type::TypeVar(name) => typevar_is_error_bounded(name, ctx),
        _ => false,
    }
}

fn typevar_is_error_bounded(name: &str, ctx: &LowerCtx) -> bool {
    let Some(specs) = ctx.declared_type_var_bounds.get(name) else {
        return false;
    };
    let mut constraints = Vec::new();
    for spec in specs {
        if let Some(constraint_name) = super::decode_typevar_constraint(spec) {
            constraints.push(constraint_name);
            continue;
        }
        if spec == "Error" {
            return true;
        }
        if ctx
            .class_types
            .get(spec)
            .is_some_and(|ty| is_valid_error_type(ty, ctx))
        {
            return true;
        }
    }
    !constraints.is_empty()
        && constraints.iter().all(|constraint_name| {
            ctx.class_types
                .get(*constraint_name)
                .is_some_and(|ty| is_valid_error_type(ty, ctx))
        })
}

/// Format a type name for use in error messages.
pub(in crate::lower) fn format_type_name(ty: &Type) -> String {
    match ty {
        Type::Int => "int".to_string(),
        Type::FixedInt(fixed) => fixed.source_name().to_string(),
        Type::Float => "float".to_string(),
        Type::Str => "str".to_string(),
        Type::Bool => "bool".to_string(),
        Type::None => "None".to_string(),
        Type::Class { name, .. } => name.clone(),
        Type::Alias { name, .. } => name.clone(),
        Type::Union(members) => {
            let duplicate_class_names = members
                .iter()
                .filter_map(|member| match member.resolve_alias() {
                    Type::Class { name, .. } => Some(name),
                    _ => None,
                })
                .filter(|name| {
                    members
                        .iter()
                        .filter(|member| {
                            matches!(member.resolve_alias(), Type::Class { name: other, .. } if other == *name)
                        })
                        .count()
                        > 1
                })
                .collect::<std::collections::HashSet<_>>();
            members
                .iter()
                .map(|member| match member.resolve_alias() {
                    Type::Class {
                        identity: Some(identity),
                        name,
                        ..
                    } if duplicate_class_names.contains(name) => identity.clone(),
                    _ => format_type_name(member),
                })
                .collect::<Vec<_>>()
                .join(" | ")
        }
        Type::List(inner) => format!("list[{}]", format_type_name(inner)),
        Type::Dict(k, v) => format!("dict[{}, {}]", format_type_name(k), format_type_name(v)),
        Type::Failure(inner) => format!("Failure[{}]", format_type_name(inner)),
        Type::TimeoutResult(inner) => format!("TimeoutResult[{}]", format_type_name(inner)),
        _ => format!("{ty:?}"),
    }
}

pub(in crate::lower) fn list_append_argument_type_mismatch(
    ctx: &mut LowerCtx,
    actual: &Type,
    expected: &Type,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_MISMATCH,
        format!(
            "list.append() argument type '{}' is not compatible with list element type '{}'",
            actual.display_name(),
            expected.display_name()
        ),
        range,
    );
}

/// Collect error types from raise statements in a list of HIR statements.
pub(in crate::lower) fn collect_raise_error_types(
    stmts: &[HirStmt],
    errors: &mut std::collections::HashSet<Type>,
) {
    for stmt in stmts {
        match stmt {
            HirStmt::Raise { value } => {
                if matches!(value.ty().resolve_alias(), Type::Class { .. }) {
                    errors.insert(value.ty().resolve_alias().clone());
                }
            }
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                collect_raise_error_types(then_body, errors);
                for (_, body) in elif_clauses {
                    collect_raise_error_types(body, errors);
                }
                if let Some(eb) = else_body {
                    collect_raise_error_types(eb, errors);
                }
            }
            HirStmt::While {
                body, else_body, ..
            }
            | HirStmt::For {
                body, else_body, ..
            }
            | HirStmt::AsyncFor {
                body, else_body, ..
            } => {
                collect_raise_error_types(body, errors);
                if let Some(else_body) = else_body {
                    collect_raise_error_types(else_body, errors);
                }
            }
            HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
                collect_raise_error_types(body, errors);
            }
            HirStmt::Match { arms, .. } => {
                for arm in arms {
                    collect_raise_error_types(&arm.body, errors);
                }
            }
            HirStmt::TryExcept { handlers, .. } => {
                for handler in handlers {
                    collect_raise_error_types(&handler.body, errors);
                }
            }
            HirStmt::TryFinally { body, finalbody } => {
                collect_raise_error_types(body, errors);
                collect_raise_error_types(finalbody, errors);
            }
            _ => {}
        }
    }
}

/// Check if a class definition has `(Protocol)` as its base class.
pub(in crate::lower) fn is_protocol_class(class_def: &StmtClassDef) -> bool {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            if n.id.as_str() == "Protocol" {
                return true;
            }
        }
    }
    false
}

/// Check if a class definition is a newtype wrapper around a primitive.
/// Returns the wrapped primitive type if so.
pub(in crate::lower) fn get_newtype_inner(class_def: &StmtClassDef) -> Option<Type> {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            match n.id.as_str() {
                "int" => return Some(Type::Int),
                "float" => return Some(Type::Float),
                "str" => return Some(Type::Str),
                "bool" => return Some(Type::Bool),
                _ => {}
            }
        }
    }
    None
}

/// Dunder method names that map to Rust operator trait impls.
const OPERATOR_DUNDERS: &[&str] = &[
    "__add__",
    "__sub__",
    "__mul__",
    "__truediv__",
    "__floordiv__",
    "__mod__",
    "__eq__",
    "__ne__",
    "__lt__",
    "__le__",
    "__gt__",
    "__ge__",
    "__str__",
    "__repr__",
    "__neg__",
    "__pos__",
    "__contains__",
];

/// Check if a method name is an operator dunder.
pub(in crate::lower) fn is_operator_dunder(name: &str) -> bool {
    OPERATOR_DUNDERS.contains(&name)
}

/// Check if a class is an enum (inherits from Enum)
pub(in crate::lower) fn is_enum_class(class_def: &StmtClassDef) -> bool {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            if n.id.as_str() == "Enum" {
                return true;
            }
        }
    }
    false
}

pub(in crate::lower) struct EnumVariantInfo {
    pub(in crate::lower) name: String,
    pub(in crate::lower) value: Option<i64>,
    pub(in crate::lower) name_range: TextRange,
}

/// Collect enum variants from a class body.
pub(in crate::lower) fn collect_enum_variants(class_def: &StmtClassDef) -> Vec<EnumVariantInfo> {
    let mut variants = Vec::new();
    let mut auto_value = 1i64;
    for stmt in &class_def.body {
        match stmt {
            Stmt::Assign(assign) => {
                if assign.targets.len() == 1 {
                    if let Expr::Name(name) = &assign.targets[0] {
                        let variant_name = name.id.to_string();
                        // Check if it has an integer value
                        let value = if let Expr::NumberLiteral(num) = assign.value.as_ref() {
                            if let sifr_python_ast::Number::Int(i) = &num.value {
                                i.as_i64()
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        let v = value.unwrap_or(auto_value);
                        auto_value = v.checked_add(1).unwrap_or(v);
                        variants.push(EnumVariantInfo {
                            name: variant_name,
                            value,
                            name_range: name.range(),
                        });
                    }
                }
            }
            Stmt::AnnAssign(ann) => {
                // `RED: int = 1` style
                if let Expr::Name(name) = ann.target.as_ref() {
                    let variant_name = name.id.to_string();
                    let value = if let Some(val_expr) = &ann.value {
                        if let Expr::NumberLiteral(num) = val_expr.as_ref() {
                            if let sifr_python_ast::Number::Int(i) = &num.value {
                                i.as_i64()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let v = value.unwrap_or(auto_value);
                    auto_value = v.checked_add(1).unwrap_or(v);
                    variants.push(EnumVariantInfo {
                        name: variant_name,
                        value,
                        name_range: name.range(),
                    });
                }
            }
            _ => {}
        }
    }
    variants
}

/// Check if a function definition has a specific decorator.
pub(in crate::lower) fn has_decorator(func: &StmtFunctionDef, decorator_name: &str) -> bool {
    for decorator in &func.decorator_list {
        if let Expr::Name(n) = &decorator.expression {
            if n.id.as_str() == decorator_name {
                return true;
            }
        }
    }
    false
}
