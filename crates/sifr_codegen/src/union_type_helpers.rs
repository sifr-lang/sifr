use crate::RustEmitter;
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

        for (enum_name, members) in &enums {
            // Generate the enum definition
            self.enum_defs.push_str("#[derive(Debug, Clone)]\n");
            let _ = writeln!(self.enum_defs, "enum {enum_name} {{");
            for member in members {
                let variant = member.union_variant_name();
                let rust_ty = member.rust_type();
                let _ = writeln!(self.enum_defs, "    {variant}({rust_ty}),");
            }
            self.enum_defs.push_str("}\n\n");

            // Generate Display impl so println!("{}", x) works
            let _ = writeln!(self.enum_defs, "impl std::fmt::Display for {enum_name} {{");
            self.enum_defs.push_str(
                "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n",
            );
            self.enum_defs.push_str("        match self {\n");
            for member in members {
                let variant = member.union_variant_name();
                // Use {:?} for class types (they derive Debug, not Display)
                let fmt_spec = if matches!(member, Type::Class { .. }) {
                    "{:?}"
                } else {
                    "{}"
                };
                let _ = writeln!(
                    self.enum_defs,
                    "            {enum_name}::{variant}(v) => write!(f, \"{fmt_spec}\", v),"
                );
            }
            self.enum_defs.push_str("        }\n");
            self.enum_defs.push_str("    }\n");
            self.enum_defs.push_str("}\n\n");
        }
    }
}
