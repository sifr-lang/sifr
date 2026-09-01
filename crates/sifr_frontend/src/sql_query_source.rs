use crate::{SqlEditorDocumentView, SqlQueryDeclaration};
use sifr_ir::{
    HirExpr, HirModule, HirStmt, visit_hir_function_exprs_mut, visit_hir_stmts_exprs_mut,
};
use sifr_sql_contract::{IntegerSign, IntegerWidth, SifrType};
use sifr_type_system::{FixedIntType, Type};
use std::collections::{BTreeMap, BTreeSet};

pub fn sql_query_declarations(
    module: &HirModule,
    profile_locals: &BTreeMap<String, String>,
) -> Result<Vec<SqlQueryDeclaration>, String> {
    let mut declarations = Vec::new();
    for function in &module.functions {
        let mut sql_decorators = function
            .decorators
            .iter()
            .filter_map(|decorator| decorator.strip_suffix(".query"))
            .filter(|name| profile_locals.contains_key(*name));
        let Some(local_profile_name) = sql_decorators.next() else {
            continue;
        };
        if sql_decorators.next().is_some() {
            return Err(format!(
                "SQL query function '{}' must use exactly one imported profile query decorator",
                function.name
            ));
        }
        let profile_name = &profile_locals[local_profile_name];
        let mut function = function.clone();
        let mut templates = Vec::new();
        visit_hir_function_exprs_mut(&mut function, &mut |expression| {
            if let HirExpr::TemplateString(template) = expression {
                templates.push(template.clone());
            }
        });
        if templates.len() != 1 {
            return Err(format!(
                "@{local_profile_name}.query function '{}' must contain exactly one typed template",
                function.name
            ));
        }
        let template = templates.remove(0);
        let parameter_types = template
            .interpolations
            .iter()
            .map(|interpolation| sql_contract_type(&interpolation.value_type))
            .collect::<Result<Vec<_>, _>>()?;
        declarations.push(SqlQueryDeclaration {
            symbol: function.name.clone(),
            profile_name: profile_name.clone(),
            exported: !function.name.starts_with('_'),
            document: SqlEditorDocumentView::from_hir(&template).with_profile(profile_name.clone()),
            parameter_types,
        });
    }
    Ok(declarations)
}

/// Erase compiler-owned SQL namespace operations after their contracts have
/// been compiled. A profile `sql(...)` call has no runtime dispatch: its typed
/// template value is the ordinary runtime representation retained by the
/// decorated query function.
pub fn erase_compiler_sql_surfaces(module: &mut HirModule, profiles: &BTreeSet<String>) {
    let mut erase = |expression: &mut HirExpr| {
        if let HirExpr::FieldAccess { object, field, ty } = expression
            && field == "schema"
            && matches!(object.as_ref(), HirExpr::Name { name, .. } if profiles.contains(name))
        {
            *expression = HirExpr::ConstructorCall {
                class_name: "SqlSchema".to_string(),
                args: Vec::new(),
                ty: ty.clone(),
            };
            return;
        }
        let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expression
        else {
            return;
        };
        let HirExpr::Name { name, .. } = object.as_ref() else {
            return;
        };
        if profiles.contains(name) && method == "sql" && args.len() == 1 {
            *expression = args.remove(0);
        }
    };
    for function in &mut module.functions {
        visit_hir_function_exprs_mut(function, &mut erase);
    }
    for class in &mut module.classes {
        for method in &mut class.methods {
            visit_hir_function_exprs_mut(method, &mut erase);
        }
        for (_, method) in &mut class.operator_impls {
            visit_hir_function_exprs_mut(method, &mut erase);
        }
    }
    for (_, _, value) in &mut module.constants {
        let mut statement = HirStmt::Expr {
            expr: value.clone(),
        };
        visit_hir_stmts_exprs_mut(std::slice::from_mut(&mut statement), &mut erase);
        if let HirStmt::Expr { expr } = statement {
            *value = expr;
        }
    }
    module.imports.retain(|import| {
        import.module != "sifr.sql.schemas"
            && import.module != "sifr.sql.requirements"
            && import.module != "sifr.sql"
    });
}

pub(crate) fn sql_contract_type(ty: &Type) -> Result<SifrType, String> {
    Ok(match ty.resolve_alias() {
        Type::Bool | Type::LiteralBool(_) => SifrType::Bool,
        Type::FixedInt(fixed) => {
            let (sign, width) = match fixed {
                FixedIntType::I8 => (IntegerSign::Signed, IntegerWidth::Bits8),
                FixedIntType::I16 => (IntegerSign::Signed, IntegerWidth::Bits16),
                FixedIntType::I32 => (IntegerSign::Signed, IntegerWidth::Bits32),
                FixedIntType::I64 => (IntegerSign::Signed, IntegerWidth::Bits64),
                FixedIntType::U8 => (IntegerSign::Unsigned, IntegerWidth::Bits8),
                FixedIntType::U16 => (IntegerSign::Unsigned, IntegerWidth::Bits16),
                FixedIntType::U32 => (IntegerSign::Unsigned, IntegerWidth::Bits32),
                FixedIntType::U64 => (IntegerSign::Unsigned, IntegerWidth::Bits64),
                FixedIntType::ISize | FixedIntType::USize => {
                    return Err("SQL contracts do not support target-sized integers".to_string());
                }
            };
            SifrType::FixedInteger { sign, width }
        }
        Type::Int | Type::LiteralInt(_) => SifrType::ExactInteger,
        Type::Float => SifrType::Float,
        Type::Decimal => SifrType::Decimal,
        Type::BigDecimal => SifrType::BigDecimal,
        Type::Str | Type::LiteralStr(_) => SifrType::Str,
        Type::Bytes => SifrType::Bytes,
        Type::None => SifrType::None,
        Type::List(element) => SifrType::List {
            element: Box::new(sql_contract_type(element)?),
        },
        Type::Union(members) => SifrType::Union {
            members: members
                .iter()
                .map(sql_contract_type)
                .collect::<Result<BTreeSet<_>, _>>()?,
        },
        Type::Class {
            identity: Some(identity),
            ..
        }
        | Type::Newtype {
            identity: Some(identity),
            ..
        }
        | Type::Enum {
            identity: Some(identity),
            ..
        } => SifrType::Custom {
            identity: identity.clone(),
        },
        unsupported => {
            return Err(format!(
                "SQL interpolation type '{unsupported}' has no closed codec contract"
            ));
        }
    })
}
