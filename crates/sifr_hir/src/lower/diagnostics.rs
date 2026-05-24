use crate::hir_nodes::HirStmt;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, Stmt, StmtClassDef, StmtFunctionDef};
use sifr_type_system::Type;

use super::LowerCtx;

#[allow(dead_code)]
pub(in crate::lower) fn is_error_class_with_ctx(
    class_def: &StmtClassDef,
    error_types: &std::collections::HashSet<String>,
) -> bool {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            let base_name = n.id.as_str();
            if base_name == "Error" || error_types.contains(base_name) {
                return true;
            }
        }
    }
    false
}

/// Check if a class definition has `(Error)` as its base class.
pub(in crate::lower) fn is_error_class(class_def: &StmtClassDef) -> bool {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            if n.id.as_str() == "Error" {
                return true;
            }
        }
    }
    false
}

/// Check if a type is a valid error type (a class registered in `error_types`).
pub(in crate::lower) fn is_valid_error_type(ty: &Type, ctx: &LowerCtx) -> bool {
    match ty {
        Type::Class { name, .. } => ctx.error_types.contains(name),
        Type::TimeoutResult(inner) => is_valid_error_type(inner, ctx),
        _ => false,
    }
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
    errors: &mut std::collections::HashSet<String>,
) {
    for stmt in stmts {
        match stmt {
            HirStmt::Raise { value } => {
                if let Type::Class { name, .. } = value.ty() {
                    errors.insert(name.clone());
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
            HirStmt::While { body, .. }
            | HirStmt::For { body, .. }
            | HirStmt::AsyncFor { body, .. } => {
                collect_raise_error_types(body, errors);
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

/// Get the parent class name for single inheritance.
/// Returns None for Error, Protocol, and primitive base classes.
pub(in crate::lower) fn get_parent_class(class_def: &StmtClassDef) -> Option<String> {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            let name = n.id.as_str();
            // Skip special base classes
            if matches!(
                name,
                "Error" | "Protocol" | "int" | "float" | "str" | "bool" | "Enum"
            ) {
                return None;
            }
            return Some(name.to_string());
        }
    }
    None
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
                        auto_value = v + 1;
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
                    auto_value = v + 1;
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
