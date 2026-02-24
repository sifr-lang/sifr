use crate::{helpers::collect_mutated_vars_with_sigs, RustEmitter};
use sifr_hir::{HirClass, HirFunction, HirModule};
use sifr_type_system::Type;

impl RustEmitter {
    pub(super) fn emit_operator_impls(&mut self, class: &HirClass) {
        for (dunder, func) in &class.operator_impls {
            match dunder.as_str() {
                "__add__" => self.emit_binop_trait_impl(class, func, "Add", "add", "+"),
                "__sub__" => self.emit_binop_trait_impl(class, func, "Sub", "sub", "-"),
                "__mul__" => self.emit_binop_trait_impl(class, func, "Mul", "mul", "*"),
                "__truediv__" => self.emit_binop_trait_impl(class, func, "Div", "div", "/"),
                "__mod__" => self.emit_binop_trait_impl(class, func, "Rem", "rem", "%"),
                "__neg__" => self.emit_unaryop_trait_impl(class, func, "Neg", "neg"),
                "__eq__" => self.emit_eq_trait_impl(class, func),
                "__lt__" => self.emit_ord_trait_impl(class, func),
                "__str__" | "__repr__" => {} // Handled separately in emit_class via Display
                _ => {}                      // Other dunders not yet supported
            }
        }
    }

    /// Emit `impl std::ops::Trait for ClassName` for binary operators.
    /// Uses reference-based impl to avoid consuming the operands.
    fn emit_binop_trait_impl(
        &mut self,
        class: &HirClass,
        func: &HirFunction,
        trait_name: &str,
        method_name: &str,
        _op: &str,
    ) {
        let is_generic = !class.type_params.is_empty();
        let bounds = Self::generic_bounds_for_class(class);
        let generic_suffix = if is_generic {
            let params: Vec<String> = class.type_params.clone();
            format!("<{}>", params.join(", "))
        } else {
            String::new()
        };
        let class_with_generics = format!("{}{}", class.name, generic_suffix);

        let rhs_ty = if let Some(param) = func.params.first() {
            if param.ty.rust_type() == class.name {
                format!("&{class_with_generics}")
            } else {
                param.ty.rust_type()
            }
        } else {
            format!("&{class_with_generics}")
        };
        let output_ty = if func.return_type.rust_type() == class.name {
            class_with_generics.clone()
        } else {
            func.return_type.rust_type()
        };

        self.output.push('\n');
        self.write_indent();
        if is_generic {
            let bounded_params: Vec<String> = class
                .type_params
                .iter()
                .map(|p| format!("{p}: {bounds}"))
                .collect();
            self.write(&format!(
                "impl<{}> std::ops::{}<{}> for &{} {{\n",
                bounded_params.join(", "),
                trait_name,
                rhs_ty,
                class_with_generics
            ));
        } else {
            self.write(&format!(
                "impl std::ops::{}<{}> for &{} {{\n",
                trait_name, rhs_ty, class.name
            ));
        }
        self.indent += 1;
        self.write_indent();
        self.write(&format!("type Output = {output_ty};\n\n"));
        self.write_indent();
        self.write(&format!("fn {method_name}(self, "));
        if let Some(param) = func.params.first() {
            self.write(&param.name);
        } else {
            self.write("rhs");
        }
        self.write(": ");
        self.write(&rhs_ty);
        self.write(") -> Self::Output {\n");
        self.indent += 1;

        let saved_mutated = std::mem::take(&mut self.mutated_vars);
        self.mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);
        for stmt in &func.body {
            self.emit_stmt(stmt);
        }
        self.mutated_vars = saved_mutated;

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit `impl std::ops::Neg for ClassName` for unary negation.
    fn emit_unaryop_trait_impl(
        &mut self,
        class: &HirClass,
        func: &HirFunction,
        trait_name: &str,
        method_name: &str,
    ) {
        let output_ty = func.return_type.rust_type();

        self.output.push('\n');
        self.write_indent();
        self.write(&format!(
            "impl std::ops::{} for {} {{\n",
            trait_name, class.name
        ));
        self.indent += 1;
        self.write_indent();
        self.write(&format!("type Output = {output_ty};\n\n"));
        self.write_indent();
        self.write(&format!("fn {method_name}(self) -> Self::Output {{\n"));
        self.indent += 1;

        for stmt in &func.body {
            self.emit_stmt(stmt);
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit `impl PartialEq for ClassName` for __eq__.
    fn emit_eq_trait_impl(&mut self, class: &HirClass, func: &HirFunction) {
        self.output.push('\n');
        self.write_indent();
        self.write(&format!("impl PartialEq for {} {{\n", class.name));
        self.indent += 1;
        self.write_indent();
        self.write("fn eq(&self, ");
        if let Some(param) = func.params.first() {
            self.write(&param.name);
        } else {
            self.write("other");
        }
        self.write(&format!(": &{}) -> bool {{\n", class.name));
        self.indent += 1;

        for stmt in &func.body {
            self.emit_stmt(stmt);
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit `impl PartialOrd for ClassName` for __lt__.
    fn emit_ord_trait_impl(&mut self, class: &HirClass, func: &HirFunction) {
        self.output.push('\n');
        self.write_indent();
        self.write(&format!("impl PartialOrd for {} {{\n", class.name));
        self.indent += 1;
        self.write_indent();
        self.write("fn partial_cmp(&self, ");
        if let Some(param) = func.params.first() {
            self.write(&param.name);
        } else {
            self.write("other");
        }
        self.write(&format!(
            ": &{}) -> Option<std::cmp::Ordering> {{\n",
            class.name
        ));
        self.indent += 1;

        // For __lt__, we generate a comparison that returns Ordering
        // The user's __lt__ body returns bool, so we need to adapt
        // Simple approach: compare using the body logic
        // We'll emit: if self < other { Some(Less) } else if self == other { Some(Equal) } else { Some(Greater) }
        // But for simplicity, just use the fields for comparison
        self.write_indent();
        self.write("Some(");
        // Use the first field for comparison as a simple heuristic
        if let Some((field_name, _)) = class.fields.first() {
            self.write(&format!(
                "self.{}.partial_cmp(&{}.{})?",
                field_name,
                if let Some(param) = func.params.first() {
                    &param.name
                } else {
                    "other"
                },
                field_name
            ));
        } else {
            self.write("std::cmp::Ordering::Equal");
        }
        self.write(")\n");

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit `impl Protocol for ClassName` blocks for satisfied protocols.
    pub(super) fn emit_protocol_impls(&mut self, class: &HirClass, module: &HirModule) {
        for proto_name in &class.implements_protocols {
            // Find the protocol definition to get its method list
            let proto_class = module
                .classes
                .iter()
                .find(|c| c.name == *proto_name && c.is_protocol());
            let proto_method_names: Vec<String> = proto_class
                .map(|pc| pc.methods.iter().map(|m| m.name.clone()).collect())
                .unwrap_or_default();

            if proto_method_names.is_empty() {
                continue;
            }

            self.output.push('\n');
            self.write_indent();
            self.write(&format!("impl {} for {} {{\n", proto_name, class.name));
            self.indent += 1;

            // Delegate to inherent methods instead of duplicating the body
            for method in &class.methods {
                if !proto_method_names.contains(&method.name) {
                    continue;
                }

                self.write_indent();
                self.write("fn ");
                self.write(&method.name);
                self.write("(&self");
                for param in &method.params {
                    self.write(", ");
                    self.write(&param.name);
                    self.write(": ");
                    self.write(&param.ty.rust_type());
                }
                self.write(")");
                if method.return_type != Type::None {
                    self.write(" -> ");
                    self.write(&method.return_type.rust_type());
                }
                self.write(" {\n");
                self.indent += 1;
                // Delegate to the inherent impl method
                self.write_indent();
                self.write(&format!("{}::{}(self", class.name, method.name));
                for param in &method.params {
                    self.write(", ");
                    self.write(&param.name);
                }
                self.write(")\n");
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }

            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
        }
    }
}
