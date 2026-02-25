use crate::{
    sifr_type_to_rust_type, RustEmitter, RustEnumVariant, RustItem, RustParam, RustStmt, RustType,
    Visibility,
};
use sifr_hir::{HirModule, HirStmt};
use sifr_type_system::ParamConvention;
use sifr_type_system::Type;
use std::fmt::Write as _;

impl RustEmitter {
    /// Collect all union types from the module that need enum definitions,
    /// and build a map of function signatures for call-site wrapping.
    pub(super) fn collect_union_types(&mut self, module: &HirModule) {
        for func in &module.functions {
            // Record function signature with conventions
            let param_info: Vec<(Type, ParamConvention)> = func
                .params
                .iter()
                .map(|p| (p.ty.clone(), p.convention))
                .collect();
            self.func_signatures
                .insert(func.name.clone(), (param_info, func.return_type.clone()));

            // Track generator functions (contain yield statements)
            if crate::body_contains_yield(&func.body) {
                self.generator_functions.insert(func.name.clone());
            }

            // Check params
            for param in &func.params {
                self.register_union_type(&param.ty);
            }
            // Check return type
            self.register_union_type(&func.return_type);
            // Check body statements
            self.collect_union_types_in_stmts(&func.body);
        }
        // Also scan class method bodies and register their signatures
        for class in &module.classes {
            for method in &class.methods {
                // Register method signature under ClassName::method_name
                let param_info: Vec<(Type, ParamConvention)> = method
                    .params
                    .iter()
                    .map(|p| (p.ty.clone(), p.convention))
                    .collect();
                self.func_signatures.insert(
                    format!("{}::{}", class.name, method.name),
                    (param_info, method.return_type.clone()),
                );

                for param in &method.params {
                    self.register_union_type(&param.ty);
                }
                self.register_union_type(&method.return_type);
                self.collect_union_types_in_stmts(&method.body);
            }
        }
    }

    fn collect_union_types_in_stmts(&mut self, stmts: &[HirStmt]) {
        for stmt in stmts {
            match stmt {
                HirStmt::Let { ty, .. } => self.register_union_type(ty),
                HirStmt::If {
                    then_body,
                    elif_clauses,
                    else_body,
                    ..
                } => {
                    self.collect_union_types_in_stmts(then_body);
                    for (_, body) in elif_clauses {
                        self.collect_union_types_in_stmts(body);
                    }
                    if let Some(body) = else_body {
                        self.collect_union_types_in_stmts(body);
                    }
                }
                HirStmt::While {
                    body, else_body, ..
                } => {
                    self.collect_union_types_in_stmts(body);
                    if let Some(eb) = else_body {
                        self.collect_union_types_in_stmts(eb);
                    }
                }
                HirStmt::For {
                    body, else_body, ..
                } => {
                    self.collect_union_types_in_stmts(body);
                    if let Some(eb) = else_body {
                        self.collect_union_types_in_stmts(eb);
                    }
                }
                _ => {}
            }
        }
    }

    fn register_union_type(&mut self, ty: &Type) {
        if let Type::Union(members) = ty {
            // Skip Option<T> pattern (T | None with exactly 2 members)
            let non_none: Vec<&Type> = members
                .iter()
                .filter(|m| !matches!(m, Type::None))
                .collect();
            let has_none = members.iter().any(|m| matches!(m, Type::None));
            if has_none && non_none.len() == 1 {
                return; // This maps to Option<T>, no enum needed
            }
            // Register the enum name and its member types
            let enum_name = ty.union_enum_name();
            self.union_enums
                .entry(enum_name)
                .or_insert_with(|| members.clone());
        }
    }

    /// Generate Rust enum definitions for all collected union types.
    pub(super) fn generate_enum_definitions(&mut self) {
        // Sort enum names for deterministic output
        let mut enums: Vec<(String, Vec<Type>)> = self.union_enums.clone().into_iter().collect();
        enums.sort_by(|a, b| a.0.cmp(&b.0));

        self.enum_items.clear();
        for (enum_name, members) in enums {
            let variants = members
                .iter()
                .map(|member| RustEnumVariant {
                    name: member.union_variant_name(),
                    tuple_fields: vec![sifr_type_to_rust_type(member)],
                    fields: Vec::new(),
                    value: None,
                })
                .collect();
            self.enum_items.push(RustItem::Enum {
                name: enum_name.clone(),
                visibility: Visibility::Private,
                derives: vec!["Debug".to_string(), "Clone".to_string()],
                repr: None,
                variants,
            });

            let mut match_lines = String::from("match self {\n");
            for member in &members {
                let variant = member.union_variant_name();
                let fmt_spec = if matches!(member, Type::Class { .. }) {
                    "{:?}"
                } else {
                    "{}"
                };
                let _ = writeln!(
                    match_lines,
                    "    {enum_name}::{variant}(v) => write!(f, \"{fmt_spec}\", v),"
                );
            }
            match_lines.push('}');

            self.enum_items.push(RustItem::Impl {
                target: enum_name,
                type_params: Vec::new(),
                trait_: Some("std::fmt::Display".to_string()),
                items: vec![RustItem::Fn {
                    name: "fmt".to_string(),
                    visibility: Visibility::Private,
                    type_params: Vec::new(),
                    params: vec![
                        RustParam::SelfParam { mutable: false },
                        RustParam::Named {
                            name: "f".to_string(),
                            ty: RustType::RawCode("&mut std::fmt::Formatter<'_>".to_string()),
                        },
                    ],
                    ret: Some(RustType::RawCode("std::fmt::Result".to_string())),
                    body: vec![RustStmt::RawCode(match_lines)],
                    is_async: false,
                }],
            });
        }
    }
}
