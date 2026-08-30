use crate::{RustEmitter, RustExpr, RustLiteral, RustStmt, RustTrait, RustType};
use sifr_ir::{HirTemplateFormatSpec, HirTemplateFormatSpecPart, HirTemplateString};

impl RustEmitter {
    pub(crate) fn try_lower_template_string_expr_for_ir(
        &mut self,
        template: &HirTemplateString,
    ) -> Option<RustExpr> {
        let mut stmts = Vec::new();
        let mut lowered_interpolations = Vec::new();

        for (index, interpolation) in template.interpolations.iter().enumerate() {
            let value_name = format!("__sifr_template_value_{index}");
            let lowered_value = self.try_lower_registry_expr_strict(&interpolation.value)?;
            stmts.push(RustStmt::Let {
                mutable: false,
                name: value_name.clone(),
                ty: None,
                value: if interpolation.clone_from_borrow {
                    RustExpr::Clone(Box::new(lowered_value))
                } else {
                    lowered_value
                },
            });

            let format_spec_name = format!("__sifr_template_format_spec_{index}");
            stmts.push(RustStmt::Let {
                mutable: false,
                name: format_spec_name.clone(),
                ty: None,
                value: match interpolation.format_spec.as_ref() {
                    Some(spec) => self.lower_template_format_spec(spec, &format!("t{index}"))?,
                    None => RustExpr::Literal(RustLiteral::Str(String::new())),
                },
            });

            lowered_interpolations.push(RustExpr::StructInit {
                name: "__SifrTemplateInterpolation".to_string(),
                fields: vec![
                    (
                        "value".to_string(),
                        RustExpr::Cast {
                            expr: Box::new(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "Box".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![RustExpr::Ident(value_name)],
                            }),
                            ty: boxed_any_type(),
                        },
                    ),
                    (
                        "value_type".to_string(),
                        RustExpr::Literal(RustLiteral::Str(
                            interpolation.value_type.display_name(),
                        )),
                    ),
                    (
                        "expression".to_string(),
                        RustExpr::Literal(RustLiteral::Str(
                            interpolation.expression_source.clone(),
                        )),
                    ),
                    (
                        "conversion".to_string(),
                        option_char(interpolation.conversion),
                    ),
                    ("format_spec".to_string(), RustExpr::Ident(format_spec_name)),
                    range_field("source_start", interpolation.source_range.start()),
                    range_field("source_end", interpolation.source_range.end()),
                    range_field("expression_start", interpolation.expression_range.start()),
                    range_field("expression_end", interpolation.expression_range.end()),
                    range_field("virtual_start", interpolation.virtual_range.start()),
                    range_field("virtual_end", interpolation.virtual_range.end()),
                ],
            });
        }

        let strings = template
            .segments
            .iter()
            .map(|segment| RustExpr::Literal(RustLiteral::Str(segment.value.clone())))
            .collect();
        Some(RustExpr::Block {
            stmts,
            expr: Some(Box::new(RustExpr::StructInit {
                name: "__SifrTemplate".to_string(),
                fields: vec![
                    ("strings".to_string(), RustExpr::Vec(strings)),
                    (
                        "interpolations".to_string(),
                        RustExpr::Vec(lowered_interpolations),
                    ),
                ],
            })),
        })
    }

    fn lower_template_format_spec(
        &mut self,
        spec: &HirTemplateFormatSpec,
        prefix: &str,
    ) -> Option<RustExpr> {
        let mut stmts = Vec::new();
        let mut format_string = String::new();
        let mut arguments = Vec::new();

        for (index, part) in spec.parts.iter().enumerate() {
            match part {
                HirTemplateFormatSpecPart::Literal(value) => {
                    push_escaped_format_literal(&mut format_string, value);
                }
                HirTemplateFormatSpecPart::Interpolation {
                    value,
                    clone_from_borrow,
                    conversion,
                    format_spec,
                    ..
                } => {
                    let value_name = format!("__sifr_template_spec_{prefix}_{index}");
                    let lowered_value = self.try_lower_registry_expr_strict(value)?;
                    stmts.push(RustStmt::Let {
                        mutable: false,
                        name: value_name.clone(),
                        ty: None,
                        value: if *clone_from_borrow {
                            RustExpr::Clone(Box::new(lowered_value))
                        } else {
                            lowered_value
                        },
                    });
                    if let Some(nested) = format_spec {
                        stmts.push(RustStmt::Let {
                            mutable: false,
                            name: format!("__sifr_template_nested_spec_{prefix}_{index}"),
                            ty: None,
                            value: self
                                .lower_template_format_spec(nested, &format!("{prefix}_{index}"))?,
                        });
                    }
                    format_string.push_str(if matches!(conversion, Some('r' | 'a')) {
                        "{:?}"
                    } else {
                        "{}"
                    });
                    arguments.push(RustExpr::Ident(value_name));
                }
            }
        }

        Some(RustExpr::Block {
            stmts,
            expr: Some(Box::new(RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str: format_string,
                args: arguments,
            })),
        })
    }
}

fn push_escaped_format_literal(target: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '{' => target.push_str("{{"),
            '}' => target.push_str("}}"),
            _ => target.push(character),
        }
    }
}

fn boxed_any_type() -> RustType {
    RustType::Boxed(Box::new(RustType::DynTrait {
        trait_: RustTrait::Named {
            name: "std::any::Any".to_string(),
            params: Vec::new(),
            associated_types: Vec::new(),
        },
        auto_traits: Vec::new(),
    }))
}

fn option_char(value: Option<char>) -> RustExpr {
    value.map_or(RustExpr::Literal(RustLiteral::None), |value| {
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![RustExpr::Literal(RustLiteral::Char(value))],
        }
    })
}

fn range_field(name: &str, value: ruff_text_size::TextSize) -> (String, RustExpr) {
    (
        name.to_string(),
        RustExpr::Literal(RustLiteral::Int(i64::from(value.to_u32()))),
    )
}
