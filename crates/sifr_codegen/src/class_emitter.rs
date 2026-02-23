use crate::{
    helpers::{collect_mutated_vars_with_sigs, is_auto_display_type, recursive_field_rust_type},
    RustEmitter,
};
use sifr_hir::{HirClass, HirModule, HirStmt};
use sifr_type_system::Type;

impl RustEmitter {
    pub(super) fn emit_class(&mut self, class: &HirClass, module: &HirModule, module_public: bool) {
        // --- Protocol: emit trait definition ---
        if class.is_protocol {
            self.emit_protocol_trait(class, module_public);
            return;
        }

        // --- Enum: emit Rust enum with repr(i64) ---
        if class.is_enum {
            self.emit_enum_class(class, module_public);
            return;
        }

        // --- Newtype: emit tuple struct ---
        if let Some(ref inner) = class.newtype_inner {
            self.emit_newtype(class, inner, module_public);
            return;
        }

        // Check if class defines __eq__ (don't auto-derive PartialEq)
        let has_custom_eq = class.operator_impls.iter().any(|(n, _)| n == "__eq__");
        let has_custom_str = class.operator_impls.iter().any(|(n, _)| n == "__str__");

        // Check if any field is a Callable type (Box<dyn Fn> doesn't implement Debug/Clone/PartialEq)
        let has_callable_field = class.fields.iter().any(|(_, t)| matches!(t, Type::Callable(..)));

        // Derive attributes
        self.write_indent();
        if has_callable_field {
            // Callable fields (Box<dyn Fn>) don't implement Debug, Clone, or PartialEq
            self.write("#[derive()]\n");
        } else if has_custom_eq {
            // Don't derive PartialEq if custom __eq__ is defined
            self.write("#[derive(Debug, Clone)]\n");
        } else if class.is_hashable {
            self.write("#[derive(Debug, Clone, PartialEq, Eq, Hash)]\n");
        } else {
            self.write("#[derive(Debug, Clone, PartialEq)]\n");
        }

        // Struct definition
        self.write_indent();
        if module_public {
            self.write("pub struct ");
        } else {
            self.write("struct ");
        }
        self.write(&class.name);
        let class_bounds = Self::generic_bounds_for_class(class);
        if !class.type_params.is_empty() {
            self.write("<");
            for (i, tp) in class.type_params.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(&format!("{tp}: {class_bounds}"));
            }
            self.write(">");
        }
        self.write(" {\n");
        self.indent += 1;

        // If this class has a parent, embed the parent struct as a field
        if let Some(ref parent) = class.parent_class {
            self.write_indent();
            if module_public {
                self.write("pub ");
            }
            let parent_field = parent.to_lowercase();
            self.write(&parent_field);
            self.write(": ");
            self.write(parent);
            self.write(",\n");
        }

        // Emit own fields (skip fields that come from the parent)
        for (field_name, field_ty) in &class.fields {
            // Skip parent-inherited fields (they're accessed via the embedded parent struct)
            if class.parent_class.is_some() {
                // We'll emit all fields listed in class.fields since the lowering
                // should only put the child's own fields here
            }
            self.write_indent();
            if module_public {
                self.write("pub ");
            }
            self.write(field_name);
            self.write(": ");
            let is_recursive = self.recursive_fields.contains(&(class.name.clone(), field_name.clone()));
            if is_recursive {
                self.write(&recursive_field_rust_type(field_ty, &class.name));
            } else if class.name == "deque" && field_name == "_data" {
                if let Type::List(elem) = field_ty {
                    self.collection_needs.needs_vecdeque = true;
                    self.write(&format!("VecDeque<{}>", elem.rust_type()));
                } else {
                    self.write(&field_ty.rust_type_for_struct_field());
                }
            } else {
                self.write(&field_ty.rust_type_for_struct_field());
            }
            self.write(",\n");
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");

        // Impl block
        self.write_indent();
        self.write("impl");
        if !class.type_params.is_empty() {
            self.write("<");
            for (i, tp) in class.type_params.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(&format!("{tp}: {class_bounds}"));
            }
            self.write(">");
        }
        self.write(" ");
        self.write(&class.name);
        if !class.type_params.is_empty() {
            self.write("<");
            for (i, tp) in class.type_params.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(tp);
            }
            self.write(">");
        }
        self.write(" {\n");
        self.indent += 1;

        // If no explicit constructor (no "new" method), generate a default one from fields
        let has_constructor = class.methods.iter().any(|m| m.name == "new");
        if !has_constructor && !class.fields.is_empty() {
            self.write_indent();
            if module_public {
                self.write("pub fn new(");
            } else {
                self.write("fn new(");
            }
            for (i, (field_name, field_ty)) in class.fields.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.write(field_name);
                self.write(": ");
                let is_recursive = self.recursive_fields.contains(&(class.name.clone(), field_name.clone()));
                if is_recursive {
                    self.write(&recursive_field_rust_type(field_ty, &class.name));
                } else {
                    self.write(&field_ty.rust_type());
                }
            }
            self.write(") -> Self {\n");
            self.indent += 1;
            self.write_indent();
            self.write("Self {\n");
            self.indent += 1;
            for (field_name, _) in &class.fields {
                self.write_indent();
                self.write(field_name);
                self.write(",\n");
            }
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
            self.indent -= 1;
            self.write_indent();
            self.write("}\n\n");
        }

        self.current_class_name = Some(class.name.clone());
        for method in &class.methods {
            self.emit_class_method(method, class, module_public);
            self.output.push('\n');
        }
        self.current_class_name = None;

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");

        // --- Emit operator trait impls ---
        self.emit_operator_impls(class);

        // For error types, implement Display and Error traits
        if class.is_error_type {
            self.output.push('\n');
            self.write_indent();
            self.write("impl std::fmt::Display for ");
            self.write(&class.name);
            self.write(" {\n");
            self.indent += 1;
            self.write_indent();
            self.write("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
            self.indent += 1;
            // If there's a 'message' field, use it for Display
            if class.fields.iter().any(|(name, _)| name == "message") {
                self.write_indent();
                self.write("write!(f, \"{}\", self.message)\n");
            } else {
                // Use Debug format as fallback
                self.write_indent();
                self.write("write!(f, \"{:?}\", self)\n");
            }
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
            self.indent -= 1;
            self.write_indent();
            self.write("}\n\n");

            self.write_indent();
            self.write("impl std::error::Error for ");
            self.write(&class.name);
            self.write(" {}\n");
        } else if has_custom_str && !class.is_error_type {
            // __str__ maps to Display (only if not error type, which already has Display)
            // The __str__ body is emitted inside the Display impl
            if let Some((_, str_func)) = class.operator_impls.iter().find(|(n, _)| n == "__str__") {
                self.output.push('\n');
                self.write_indent();
                self.write("impl std::fmt::Display for ");
                self.write(&class.name);
                self.write(" {\n");
                self.indent += 1;
                self.write_indent();
                self.write("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
                self.indent += 1;
                // Emit the body of __str__ but wrap the return value in write!(f, "{}", ...)
                // For simplicity, if the body is a single Return, emit write!(f, "{}", return_expr)
                if str_func.body.len() == 1 {
                    if let Some(HirStmt::Return { value: Some(ref ret_expr) }) = str_func.body.first() {
                        self.write_indent();
                        self.write("write!(f, \"{}\", ");
                        self.emit_expr(ret_expr);
                        self.write(")\n");
                    } else {
                        // Single non-return statement
                        let saved = self.emission_ctx.in_display_impl;
                        self.emission_ctx.in_display_impl = true;
                        let saved_mutated = std::mem::take(&mut self.mutated_vars);
                        self.mutated_vars = collect_mutated_vars_with_sigs(&str_func.body, &self.func_signatures);
                        for stmt in &str_func.body {
                            self.emit_stmt(stmt);
                        }
                        self.mutated_vars = saved_mutated;
                        self.emission_ctx.in_display_impl = saved;
                        self.write_indent();
                        self.write("Ok(())\n");
                    }
                } else {
                    // Multi-statement body: emit with in_display_impl flag
                    let saved = self.emission_ctx.in_display_impl;
                    self.emission_ctx.in_display_impl = true;
                    // Pre-scan for mutated variables so let mut is emitted correctly
                    let saved_mutated = std::mem::take(&mut self.mutated_vars);
                    self.mutated_vars = collect_mutated_vars_with_sigs(&str_func.body, &self.func_signatures);
                    for stmt in &str_func.body {
                        self.emit_stmt(stmt);
                    }
                    self.mutated_vars = saved_mutated;
                    self.emission_ctx.in_display_impl = saved;
                    self.write_indent();
                    self.write("Ok(())\n");
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
        } else if !has_custom_str && !class.is_error_type && !class.fields.is_empty()
            && class.fields.iter().all(|(_, t)| is_auto_display_type(t))
        {
            // Auto-generate Display impl: ClassName(field1=value1, field2=value2)
            self.output.push('\n');
            self.write_indent();
            self.write("impl std::fmt::Display for ");
            self.write(&class.name);
            if !class.type_params.is_empty() {
                self.write("<");
                for (i, tp) in class.type_params.iter().enumerate() {
                    if i > 0 { self.write(", "); }
                    self.write(tp);
                }
                self.write(">");
            }
            self.write(" {\n");
            self.indent += 1;
            self.write_indent();
            self.write("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
            self.indent += 1;
            self.write_indent();
            self.write("write!(f, \"");
            self.write(&class.name);
            self.write("(");
            for (i, (field_name, _)) in class.fields.iter().enumerate() {
                if i > 0 { self.write(", "); }
                self.write(field_name);
                self.write("={}");
            }
            self.write(")\"");
            for (field_name, _) in &class.fields {
                self.write(", self.");
                self.write(field_name);
            }
            self.write(")\n");
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
        }

        // Emit protocol trait impls
        self.emit_protocol_impls(class, module);
    }
}
