use crate::{RustEmitter, RustExpr, RustStmt};
use sifr_ir::HirExpr;
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn is_sql_query_expr(expr: &HirExpr) -> bool {
        let HirExpr::ConstructorCall { class_name, ty, .. } = expr else {
            return false;
        };
        match class_name.as_str() {
            "__sifr_sql_bound" => matches!(ty.resolve_alias(), Type::Class {
                identity: Some(identity), type_args, ..
            } if identity == "sifr.sql.BoundQuery" && type_args.len() == 5),
            "__sifr_sql_connect"
            | "__sifr_sql_fetch_one"
            | "__sifr_sql_fetch_optional"
            | "__sifr_sql_fetch_all" => matches!(ty.resolve_alias(), Type::Awaitable(result)
                    if matches!(result.resolve_alias(), Type::Result(_, error)
                        if matches!(error.resolve_alias(), Type::Class { identity: Some(identity), .. }
                            if identity == "sifr.sql.SqlError"))),
            _ => false,
        }
    }

    pub(crate) fn try_lower_sql_query_expr(&mut self, expr: &HirExpr) -> Option<RustExpr> {
        let HirExpr::ConstructorCall {
            class_name,
            args,
            ty,
        } = expr
        else {
            return None;
        };
        if !Self::is_sql_query_expr(expr) {
            return None;
        }
        match class_name.as_str() {
            "__sifr_sql_bound" => {
                let [descriptor, HirExpr::TemplateString(template)] = args.as_slice() else {
                    return None;
                };
                let Type::Class { type_args, .. } = ty else {
                    return None;
                };
                let Type::StructuralRecord(row) = &type_args[1] else {
                    return None;
                };
                let row_name =
                    crate::structural_identity_codegen::structural_record_layout_rust_name(row);
                let HirExpr::StringLiteral(descriptor_json) = descriptor else {
                    return None;
                };
                let descriptor_value: serde_json::Value =
                    serde_json::from_str(descriptor_json).ok()?;
                let columns = descriptor_value.get("result_types")?.as_array()?;
                let fields = row.fields().iter().map(|field| {
                    let index = columns.iter().position(|column| column.get("name").and_then(serde_json::Value::as_str) == Some(field.name()))?;
                    let field_type = crate::render_type(&crate::sifr_type_to_rust_type(field.ty()));
                    let field_name = crate::Renderer::render_identifier(field.name());
                    Some(format!("{field_name}: ::sifr_sql_sqlite_runtime::application::decode::<{field_type}>(values, {index})?"))
                }).collect::<Option<Vec<_>>>()?.join(", ");
                let decoder = RustExpr::Verbatim(format!("|values| Ok({row_name} {{ {fields} }})"));
                let mut stmts = Vec::new();
                let mut values = Vec::new();
                for (index, interpolation) in template.interpolations.iter().enumerate() {
                    let name = format!("__sifr_sql_capture_{index}");
                    let value = self.try_lower_registry_expr_strict(&interpolation.value)?;
                    stmts.push(RustStmt::Let {
                        mutable: false,
                        name: name.clone(),
                        ty: None,
                        value: if matches!(interpolation.value_type.resolve_alias(), Type::Str) {
                            // Sifr string operands can render as borrowed str or String.
                            // SQL parameters own their text across async execution.
                            RustExpr::MethodCall {
                                receiver: Box::new(value),
                                method: "to_string".into(),
                                args: Vec::new(),
                            }
                        } else if interpolation.clone_from_borrow {
                            RustExpr::Clone(Box::new(value))
                        } else {
                            value
                        },
                    });
                    values.push(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "::sifr_sql_sqlite_runtime::application::encode".into(),
                        ])),
                        args: vec![RustExpr::Ident(name)],
                    });
                }
                let call = RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "::sifr_sql_sqlite_runtime::application::BoundQuery::new".into(),
                    ])),
                    args: vec![
                        self.try_lower_registry_expr_strict(descriptor)?,
                        RustExpr::Vec(values),
                        decoder,
                    ],
                };
                Some(RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(call)),
                })
            }
            "__sifr_sql_connect" => {
                let lowered = args
                    .iter()
                    .map(|arg| {
                        self.try_lower_registry_expr_strict(arg)
                            .map(|value| RustExpr::Clone(Box::new(value)))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "::sifr_sql_sqlite_runtime::application::connect".into(),
                    ])),
                    args: lowered,
                })
            }
            "__sifr_sql_fetch_one" | "__sifr_sql_fetch_optional" | "__sifr_sql_fetch_all" => {
                let [pool, query, rest @ ..] = args.as_slice() else {
                    return None;
                };
                let pool = self.try_lower_registry_expr_strict(pool)?;
                let query = self.try_lower_registry_expr_strict(query)?;
                let method = class_name.strip_prefix("__sifr_sql_")?;
                let mut call_args = vec![RustExpr::Clone(Box::new(query))];
                if let Some(bound) = rest.first() {
                    // The frontend has already validated a positive literal u64 bound.
                    let HirExpr::IntLiteral(value) = bound else {
                        return None;
                    };
                    call_args.push(RustExpr::Verbatim(format!("{value}u64")));
                }
                Some(RustExpr::MethodCall {
                    receiver: Box::new(pool),
                    method: method.into(),
                    args: call_args,
                })
            }
            _ => None,
        }
    }
}
