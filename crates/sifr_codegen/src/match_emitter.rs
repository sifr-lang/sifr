use crate::{helpers::is_option_type, RustEmitter};
use sifr_hir::{HirExpr, HirMatchArm, HirPattern, HirStmt};
use sifr_type_system::Type;

impl RustEmitter {
    fn match_pattern_literal_code(value: &HirExpr) -> Option<String> {
        match value {
            HirExpr::IntLiteral(v) => Some(v.to_string()),
            HirExpr::FloatLiteral(v) => Some(format!("{v:?}")),
            HirExpr::BoolLiteral(v) => Some(v.to_string()),
            HirExpr::StringLiteral(s) => Some(format!("{s:?}")),
            HirExpr::EnumVariant {
                enum_name, variant, ..
            } => Some(format!("{enum_name}::{variant}")),
            HirExpr::NoneLiteral => Some("None".to_string()),
            _ => None,
        }
    }

    fn render_match_pattern_value(&mut self, value: &HirExpr) -> String {
        if let Some(code) = Self::match_pattern_literal_code(value) {
            return code;
        }
        self.render_expr_with_lowered_path(value)
    }

    fn render_match_guard_expr(
        &mut self,
        guard_expr: &HirExpr,
        pattern: &HirPattern,
        is_non_option_union: bool,
    ) -> String {
        let guard_code = self.render_expr_with_lowered_path(guard_expr);
        Self::substitute_class_captures_in_guard(&guard_code, pattern, is_non_option_union)
    }

    pub(super) fn emit_match(
        &mut self,
        subject: &HirExpr,
        subject_ty: &Type,
        arms: &[HirMatchArm],
    ) {
        // Determine how to emit the match based on subject type
        let is_option = is_option_type(subject_ty);
        let is_non_option_union = matches!(subject_ty, Type::Union(_)) && !is_option;

        self.write_indent();
        // For both union and non-union subjects, emit a direct Rust match.
        self.write("match ");
        self.emit_expr(subject);
        self.write(" {\n");

        self.indent += 1;

        let mut has_wildcard = false;
        for arm in arms {
            if matches!(arm.pattern, HirPattern::Wildcard) {
                has_wildcard = true;
            }
            self.emit_match_arm(
                &arm.pattern,
                subject_ty,
                arm.guard.as_ref(),
                &arm.body,
                is_option,
                is_non_option_union,
            );
        }

        // If no wildcard and not a union type, add a wildcard arm to make it exhaustive
        if !has_wildcard && !is_option && !is_non_option_union {
            // Already handled by the arms themselves
        }

        self.indent -= 1;
        self.writeln("}");
    }

    fn emit_match_arm(
        &mut self,
        pattern: &HirPattern,
        subject_ty: &Type,
        guard: Option<&HirExpr>,
        body: &[HirStmt],
        is_option: bool,
        is_non_option_union: bool,
    ) {
        self.write_indent();

        // Build the pattern part (without =>)
        let has_str_guard = matches!(
            pattern,
            HirPattern::Literal {
                value: HirExpr::StringLiteral(_)
            }
        ) || matches!(pattern, HirPattern::Or { patterns } if patterns.iter().any(|p| matches!(p, HirPattern::Literal { value: HirExpr::StringLiteral(_) })));

        match pattern {
            HirPattern::Wildcard => {
                self.write("_");
            }
            HirPattern::None => {
                if is_option {
                    self.write("None");
                } else {
                    self.write("_");
                }
            }
            HirPattern::Capture { name, ty } => {
                if is_option {
                    let _ = ty;
                    self.write(&format!("Some({name})"));
                } else {
                    self.write(name);
                }
            }
            HirPattern::Literal { value } => {
                if let HirExpr::StringLiteral(_) = value {
                    // String matching needs a guard since Rust can't match String directly
                    self.write("__s");
                } else {
                    let lit_code = self.render_match_pattern_value(value);
                    self.write(&lit_code);
                }
            }
            HirPattern::Or { patterns } => {
                let has_str = patterns.iter().any(|p| {
                    matches!(
                        p,
                        HirPattern::Literal {
                            value: HirExpr::StringLiteral(_)
                        }
                    )
                });
                if has_str {
                    self.write("__s");
                } else {
                    let mut parts = Vec::new();
                    for p in patterns {
                        match p {
                            HirPattern::Literal { value } => {
                                let lit_code = self.render_match_pattern_value(value);
                                parts.push(lit_code);
                            }
                            HirPattern::None => parts.push("None".to_string()),
                            HirPattern::Wildcard => parts.push("_".to_string()),
                            HirPattern::Value { path } => {
                                parts.push(path.join("::"));
                            }
                            _ => parts.push("_".to_string()),
                        }
                    }
                    self.write(&parts.join(" | "));
                }
            }
            HirPattern::Class { class_name, fields } => {
                if is_non_option_union {
                    let enum_name = subject_ty.union_enum_name();
                    let variant_name = if let Type::Union(members) = subject_ty {
                        let target_ty = match class_name.as_str() {
                            "int" => Some(Type::Int),
                            "str" => Some(Type::Str),
                            "float" => Some(Type::Float),
                            "bool" => Some(Type::Bool),
                            other => members
                                .iter()
                                .find(|m| matches!(m, Type::Class { name, .. } if name == other))
                                .cloned(),
                        };
                        if let Some(ty) = target_ty {
                            ty.union_variant_name()
                        } else {
                            class_name.clone()
                        }
                    } else {
                        class_name.clone()
                    };
                    if fields.is_empty() {
                        self.write(&format!("{enum_name}::{variant_name}(_)"));
                    } else {
                        self.write(&format!("{enum_name}::{variant_name}(__inner)"));
                    }
                } else {
                    // For direct struct patterns, use __matched with field guards
                    self.write("__matched");
                }
            }
            HirPattern::Value { path } => {
                let rust_path = path.join("::");
                self.write(&rust_path);
            }
            HirPattern::Tuple { elements } => {
                self.write("(");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    match elem {
                        HirPattern::Capture { name, .. } => self.write(name),
                        HirPattern::Wildcard => self.write("_"),
                        HirPattern::Literal { value } => {
                            let lit_code = self.render_match_pattern_value(value);
                            self.write(&lit_code);
                        }
                        _ => self.write("_"),
                    }
                }
                self.write(")");
            }
        }

        // Build field guards for class patterns with literal field values
        let class_field_guards: Vec<String> = if let HirPattern::Class { fields, .. } = pattern {
            if is_non_option_union {
                Vec::new()
            } else {
                fields
                    .iter()
                    .filter_map(|(fname, fpat)| match fpat {
                        HirPattern::Literal { value } => {
                            let lit_code = self.render_match_pattern_value(value);
                            Some(format!("__matched.{fname} == {lit_code}"))
                        }
                        HirPattern::None => Some(format!("__matched.{fname}.is_none()")),
                        _ => None,
                    })
                    .collect()
            }
        } else {
            Vec::new()
        };

        // Add guard
        if has_str_guard {
            // Build string guard condition
            let str_guard = match pattern {
                HirPattern::Literal {
                    value: HirExpr::StringLiteral(s),
                } => {
                    format!("__s == {s:?}")
                }
                HirPattern::Or { patterns } => {
                    let conditions: Vec<String> = patterns
                        .iter()
                        .map(|p| match p {
                            HirPattern::Literal {
                                value: HirExpr::StringLiteral(s),
                            } => {
                                format!("__s == {s:?}")
                            }
                            _ => "__s == _".to_string(),
                        })
                        .collect();
                    conditions.join(" || ")
                }
                _ => String::new(),
            };
            if let Some(guard_expr) = guard {
                let guard_code =
                    self.render_match_guard_expr(guard_expr, pattern, is_non_option_union);
                self.write(&format!(" if ({str_guard}) && ({guard_code})"));
            } else {
                self.write(&format!(" if {str_guard}"));
            }
        } else if !class_field_guards.is_empty() {
            let mut all_guards = class_field_guards;
            if let Some(guard_expr) = guard {
                let guard_code =
                    self.render_match_guard_expr(guard_expr, pattern, is_non_option_union);
                all_guards.push(guard_code);
            }
            self.write(&format!(" if {}", all_guards.join(" && ")));
        } else if let Some(guard_expr) = guard {
            let guard_code = self.render_match_guard_expr(guard_expr, pattern, is_non_option_union);
            self.write(&format!(" if {guard_code}"));
        }

        self.write(" => {\n");
        self.indent += 1;

        // For class patterns with fields on union types, destructure
        if let HirPattern::Class { class_name, fields } = pattern {
            if is_non_option_union && !fields.is_empty() {
                for (fname, fpat) in fields {
                    if let HirPattern::Capture { name, .. } = fpat {
                        self.write_indent();
                        self.write(&format!("let {name} = __inner.{fname};\n"));
                    }
                }
            } else if !is_non_option_union {
                for (fname, fpat) in fields {
                    if let HirPattern::Capture { name, .. } = fpat {
                        self.write_indent();
                        self.write(&format!("let {name} = __matched.{fname};\n"));
                    }
                }
            }
            let _ = class_name;
        }

        for s in body {
            self.emit_stmt(s);
        }

        self.indent -= 1;
        self.writeln("}");
    }
}
