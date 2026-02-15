//! Sifr Code Generation
//!
//! Translates the typed HIR into Rust source code.

use sifr_hir::*;
use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

/// Generate Rust source code from a HIR module.
pub fn generate_rust(module: &HirModule) -> String {
    let mut emitter = RustEmitter::new();

    // First pass: collect all union types used in the module
    emitter.collect_union_types(module);

    // Generate enum definitions for non-Option union types
    emitter.generate_enum_definitions();

    // Second pass: emit the actual code
    emitter.emit_module(module);

    let mut result = String::new();
    if emitter.needs_hashmap {
        result.push_str("use std::collections::HashMap;\n\n");
    }
    if !emitter.enum_defs.is_empty() {
        result.push_str(&emitter.enum_defs);
        result.push('\n');
    }
    result.push_str(&emitter.output);
    result
}

/// Generate Rust source code for a multi-module project.
/// Returns a map of filename -> Rust source code.
pub fn generate_rust_multi(modules: &[(&str, &HirModule)]) -> HashMap<String, String> {
    let mut files = HashMap::new();

    for (module_name, module) in modules {
        let mut emitter = RustEmitter::new();
        // For non-main modules, enable pub mode
        if *module_name != "main" {
            emitter.pub_mode = true;
        }
        emitter.collect_union_types(module);
        emitter.generate_enum_definitions();
        emitter.emit_module(module);

        let mut result = String::new();

        // For non-main modules, add imports as `use` statements
        for import in &module.imports {
            for name in &import.names {
                result.push_str(&format!("use crate::{}::{};\n", import.module, name));
            }
        }

        if emitter.needs_hashmap {
            result.push_str("use std::collections::HashMap;\n");
        }
        if !result.is_empty() {
            result.push('\n');
        }
        if !emitter.enum_defs.is_empty() {
            result.push_str(&emitter.enum_defs);
            result.push('\n');
        }

        result.push_str(&emitter.output);

        files.insert(module_name.to_string(), result);
    }

    files
}

/// Generate a complete Rust project (Cargo.toml + main.rs content).
pub fn generate_project(module: &HirModule, project_name: &str) -> (String, String) {
    let cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"
"#
    );

    let main_rs = generate_rust(module);
    (cargo_toml, main_rs)
}

struct RustEmitter {
    output: String,
    indent: usize,
    needs_hashmap: bool,
    /// Track union enum types that need to be defined (name -> member types)
    union_enums: HashMap<String, Vec<Type>>,
    /// Accumulated enum definitions to prepend
    enum_defs: String,
    /// The return type of the function currently being emitted
    current_return_type: Option<Type>,
    /// Set of variable names currently narrowed via `if let Some(...)` unwrap
    option_unwrapped_vars: HashSet<String>,
    /// Function signatures: name -> (param_types, return_type)
    func_signatures: HashMap<String, (Vec<Type>, Type)>,
    /// Whether we're inside a loop that has an else clause
    in_loop_with_else: bool,
    /// Whether to emit `pub` on all top-level items (for module exports)
    pub_mode: bool,
    /// Set of variable names that are mutated in the current function body
    mutated_vars: HashSet<String>,
    /// Set of class names that have Display impl (via __str__ or error type)
    display_classes: HashSet<String>,
    /// Map from child class name -> (parent class name, set of parent field names)
    parent_fields: HashMap<String, (String, HashSet<String>)>,
    /// The class currently being emitted (for field access resolution)
    current_class_name: Option<String>,
}

impl RustEmitter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            needs_hashmap: false,
            union_enums: HashMap::new(),
            enum_defs: String::new(),
            current_return_type: None,
            option_unwrapped_vars: HashSet::new(),
            func_signatures: HashMap::new(),
            in_loop_with_else: false,
            pub_mode: false,
            mutated_vars: HashSet::new(),
            display_classes: HashSet::new(),
            parent_fields: HashMap::new(),
            current_class_name: None,
        }
    }

    /// Collect all union types from the module that need enum definitions,
    /// and build a map of function signatures for call-site wrapping.
    fn collect_union_types(&mut self, module: &HirModule) {
        for func in &module.functions {
            // Record function signature
            let param_types: Vec<Type> = func.params.iter().map(|p| p.ty.clone()).collect();
            self.func_signatures.insert(func.name.clone(), (param_types, func.return_type.clone()));

            // Check params
            for param in &func.params {
                self.register_union_type(&param.ty);
            }
            // Check return type
            self.register_union_type(&func.return_type);
            // Check body statements
            self.collect_union_types_in_stmts(&func.body);
        }
        // Also scan class method bodies
        for class in &module.classes {
            for method in &class.methods {
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
                HirStmt::If { then_body, elif_clauses, else_body, .. } => {
                    self.collect_union_types_in_stmts(then_body);
                    for (_, body) in elif_clauses {
                        self.collect_union_types_in_stmts(body);
                    }
                    if let Some(body) = else_body {
                        self.collect_union_types_in_stmts(body);
                    }
                }
                HirStmt::While { body, else_body, .. } => {
                    self.collect_union_types_in_stmts(body);
                    if let Some(eb) = else_body {
                        self.collect_union_types_in_stmts(eb);
                    }
                }
                HirStmt::For { body, else_body, .. } => {
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
            let non_none: Vec<&Type> = members.iter().filter(|m| !matches!(m, Type::None)).collect();
            let has_none = members.iter().any(|m| matches!(m, Type::None));
            if has_none && non_none.len() == 1 {
                return; // This maps to Option<T>, no enum needed
            }
            // Register the enum name and its member types
            let enum_name = ty.union_enum_name();
            self.union_enums.entry(enum_name).or_insert_with(|| members.clone());
        }
    }

    /// Generate Rust enum definitions for all collected union types.
    fn generate_enum_definitions(&mut self) {
        // Sort enum names for deterministic output
        let mut enums: Vec<(String, Vec<Type>)> = self.union_enums.clone().into_iter().collect();
        enums.sort_by(|a, b| a.0.cmp(&b.0));

        for (enum_name, members) in &enums {
            // Generate the enum definition
            self.enum_defs.push_str(&format!("#[derive(Debug, Clone)]\n"));
            self.enum_defs.push_str(&format!("enum {} {{\n", enum_name));
            for member in members {
                let variant = member.union_variant_name();
                let rust_ty = member.rust_type();
                self.enum_defs.push_str(&format!("    {}({}),\n", variant, rust_ty));
            }
            self.enum_defs.push_str("}\n\n");

            // Generate Display impl so println!("{}", x) works
            self.enum_defs.push_str(&format!("impl std::fmt::Display for {} {{\n", enum_name));
            self.enum_defs.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
            self.enum_defs.push_str("        match self {\n");
            for member in members {
                let variant = member.union_variant_name();
                // Use {:?} for class types (they derive Debug, not Display)
                let fmt_spec = if matches!(member, Type::Class { .. }) { "{:?}" } else { "{}" };
                self.enum_defs.push_str(&format!(
                    "            {}::{}(v) => write!(f, \"{}\", v),\n",
                    enum_name, variant, fmt_spec
                ));
            }
            self.enum_defs.push_str("        }\n");
            self.enum_defs.push_str("    }\n");
            self.enum_defs.push_str("}\n\n");
        }
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn emit_module(&mut self, module: &HirModule) {
        // Pre-scan: collect classes that have Display impls
        for class in &module.classes {
            if class.is_error_type
                || class.newtype_inner.is_some()
                || class.operator_impls.iter().any(|(n, _)| n == "__str__" || n == "__repr__")
            {
                self.display_classes.insert(class.name.clone());
            }
        }

        // Pre-scan: collect parent field info for inheritance
        for class in &module.classes {
            if let Some(ref parent_name) = class.parent_class {
                // Find the parent class and collect its field names
                if let Some(parent_class) = module.classes.iter().find(|c| c.name == *parent_name) {
                    let parent_field_names: HashSet<String> = parent_class.fields.iter()
                        .map(|(name, _)| name.clone())
                        .collect();
                    self.parent_fields.insert(
                        class.name.clone(),
                        (parent_name.clone(), parent_field_names),
                    );
                }
            }
        }

        // Emit class definitions first (structs + impls)
        for class in &module.classes {
            self.emit_class(class, module);
            self.output.push('\n');
        }

        for (i, func) in module.functions.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            }
            self.emit_function(func);
        }
    }

    fn emit_class(&mut self, class: &HirClass, module: &HirModule) {
        // --- Protocol: emit trait definition ---
        if class.is_protocol {
            self.emit_protocol_trait(class);
            return;
        }

        // --- Newtype: emit tuple struct ---
        if let Some(ref inner) = class.newtype_inner {
            self.emit_newtype(class, inner);
            return;
        }

        // Check if class defines __eq__ (don't auto-derive PartialEq)
        let has_custom_eq = class.operator_impls.iter().any(|(n, _)| n == "__eq__");
        let has_custom_str = class.operator_impls.iter().any(|(n, _)| n == "__str__");

        // Derive attributes
        self.write_indent();
        if has_custom_eq {
            // Don't derive PartialEq if custom __eq__ is defined
            self.write("#[derive(Debug, Clone)]\n");
        } else if class.is_hashable {
            self.write("#[derive(Debug, Clone, PartialEq, Eq, Hash)]\n");
        } else {
            self.write("#[derive(Debug, Clone, PartialEq)]\n");
        }

        // Struct definition
        self.write_indent();
        if self.pub_mode {
            self.write("pub struct ");
        } else {
            self.write("struct ");
        }
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;

        // If this class has a parent, embed the parent struct as a field
        if let Some(ref parent) = class.parent_class {
            self.write_indent();
            if self.pub_mode {
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
            if self.pub_mode {
                self.write("pub ");
            }
            self.write(field_name);
            self.write(": ");
            self.write(&field_ty.rust_type());
            self.write(",\n");
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");

        // Impl block
        self.write_indent();
        self.write("impl ");
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;

        // If no explicit constructor (no "new" method), generate a default one from fields
        let has_constructor = class.methods.iter().any(|m| m.name == "new");
        if !has_constructor && !class.fields.is_empty() {
            self.write_indent();
            if self.pub_mode {
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
                self.write(&field_ty.rust_type());
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
            self.emit_class_method(method, class);
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
                if let Some(HirStmt::Return { value: Some(ref ret_expr) }) = str_func.body.first() {
                    self.write_indent();
                    self.write("write!(f, \"{}\", ");
                    self.emit_expr(ret_expr);
                    self.write(")\n");
                } else {
                    // Fallback: emit body statements
                    for stmt in &str_func.body {
                        self.emit_stmt(stmt);
                    }
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
        }

        // Emit protocol trait impls
        self.emit_protocol_impls(class, module);
    }

    /// Emit a Rust `trait` definition for a Protocol class.
    fn emit_protocol_trait(&mut self, class: &HirClass) {
        self.write_indent();
        if self.pub_mode {
            self.write("pub trait ");
        } else {
            self.write("trait ");
        }
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;

        for method in &class.methods {
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
            self.write(";\n");
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit a newtype tuple struct.
    fn emit_newtype(&mut self, class: &HirClass, inner: &Type) {
        // Derive attributes
        self.write_indent();
        if is_hashable_type_codegen(inner) {
            self.write("#[derive(Debug, Clone, PartialEq, Eq, Hash)]\n");
        } else {
            self.write("#[derive(Debug, Clone, PartialEq)]\n");
        }

        self.write_indent();
        if self.pub_mode {
            self.write(&format!("pub struct {}({});\n\n", class.name, inner.rust_type()));
        } else {
            self.write(&format!("struct {}({});\n\n", class.name, inner.rust_type()));
        }

        // Impl block with constructor and value() accessor
        self.write_indent();
        self.write("impl ");
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;

        // Constructor: fn new(value: InnerType) -> Self
        self.write_indent();
        let pub_prefix = if self.pub_mode { "pub " } else { "" };
        self.write(&format!("{}fn new(value: {}) -> Self {{\n", pub_prefix, inner.rust_type()));
        self.indent += 1;
        self.write_indent();
        self.write("Self(value)\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");

        // Accessor: fn value(&self) -> InnerType
        self.write_indent();
        self.write(&format!("{}fn value(&self) -> {} {{\n", pub_prefix, inner.rust_type()));
        self.indent += 1;
        self.write_indent();
        if inner.ownership() == sifr_type_system::OwnershipKind::Copy {
            self.write("self.0\n");
        } else {
            self.write("self.0.clone()\n");
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");

        // Emit any custom methods
        for method in &class.methods {
            self.output.push('\n');
            self.emit_class_method(method, class);
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");

        // Display impl for newtypes
        self.output.push('\n');
        self.write_indent();
        self.write("impl std::fmt::Display for ");
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;
        self.write_indent();
        self.write("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
        self.indent += 1;
        self.write_indent();
        self.write("write!(f, \"{}\", self.0)\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
    }

    /// Emit Rust operator trait impls for operator overloading dunders.
    fn emit_operator_impls(&mut self, class: &HirClass) {
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
                _ => {} // Other dunders not yet supported
            }
        }
    }

    /// Emit `impl std::ops::Trait for ClassName` for binary operators.
    /// Uses reference-based impl to avoid consuming the operands.
    fn emit_binop_trait_impl(&mut self, class: &HirClass, func: &HirFunction, trait_name: &str, method_name: &str, _op: &str) {
        let rhs_ty = if let Some(param) = func.params.first() {
            // If the param type is the same class, use &ClassName
            if param.ty.rust_type() == class.name {
                format!("&{}", class.name)
            } else {
                param.ty.rust_type()
            }
        } else {
            format!("&{}", class.name)
        };
        let output_ty = func.return_type.rust_type();

        self.output.push('\n');
        self.write_indent();
        self.write(&format!("impl std::ops::{}<{}> for &{} {{\n", trait_name, rhs_ty, class.name));
        self.indent += 1;
        self.write_indent();
        self.write(&format!("type Output = {};\n\n", output_ty));
        self.write_indent();
        self.write(&format!("fn {}(self, ", method_name));
        if let Some(param) = func.params.first() {
            self.write(&param.name);
        } else {
            self.write("rhs");
        }
        self.write(": ");
        self.write(&rhs_ty);
        self.write(") -> Self::Output {\n");
        self.indent += 1;

        // Emit the body
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

    /// Emit `impl std::ops::Neg for ClassName` for unary negation.
    fn emit_unaryop_trait_impl(&mut self, class: &HirClass, func: &HirFunction, trait_name: &str, method_name: &str) {
        let output_ty = func.return_type.rust_type();

        self.output.push('\n');
        self.write_indent();
        self.write(&format!("impl std::ops::{} for {} {{\n", trait_name, class.name));
        self.indent += 1;
        self.write_indent();
        self.write(&format!("type Output = {};\n\n", output_ty));
        self.write_indent();
        self.write(&format!("fn {}(self) -> Self::Output {{\n", method_name));
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
        self.write(&format!(": &{}) -> Option<std::cmp::Ordering> {{\n", class.name));
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
            self.write(&format!("self.{}.partial_cmp(&{}.{})?", field_name,
                if let Some(param) = func.params.first() { &param.name } else { "other" },
                field_name));
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
    fn emit_protocol_impls(&mut self, class: &HirClass, module: &HirModule) {
        for proto_name in &class.implements_protocols {
            // Find the protocol definition to get its method list
            let proto_class = module.classes.iter().find(|c| c.name == *proto_name && c.is_protocol);
            let proto_method_names: Vec<String> = proto_class
                .map(|pc| pc.methods.iter().map(|m| m.name.clone()).collect())
                .unwrap_or_default();

            if proto_method_names.is_empty() { continue; }

            self.output.push('\n');
            self.write_indent();
            self.write(&format!("impl {} for {} {{\n", proto_name, class.name));
            self.indent += 1;

            // Only emit methods that are part of the protocol
            for method in &class.methods {
                if !proto_method_names.contains(&method.name) { continue; }

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
                for stmt in &method.body {
                    self.emit_stmt(stmt);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }

            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
        }
    }

    fn emit_class_method(&mut self, method: &HirFunction, class: &HirClass) {
        self.current_return_type = Some(method.return_type.clone());

        // Pre-scan: collect mutated variables so we know which need `mut`
        self.mutated_vars = collect_mutated_vars(&method.body);

        self.write_indent();
        let pub_prefix = if self.pub_mode { "pub " } else { "" };

        match method.method_kind {
            MethodKind::ClassMethod => {
                // @classmethod -> associated function (no self)
                self.write(&format!("{}fn {}(", pub_prefix, method.name));
                for (i, param) in method.params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
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
                for stmt in &method.body {
                    self.emit_stmt(stmt);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
            MethodKind::StaticMethod => {
                // @staticmethod -> associated function (no self)
                self.write(&format!("{}fn {}(", pub_prefix, method.name));
                for (i, param) in method.params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
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
                for stmt in &method.body {
                    self.emit_stmt(stmt);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
            MethodKind::Regular => {
                if method.name == "new" {
                    // Constructor: fn new(params) -> Self
                    self.write(&format!("{}fn new(", pub_prefix));
                    for (i, param) in method.params.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.write(&param.name);
                        self.write(": ");
                        self.write(&param.ty.rust_type());
                    }
                    self.write(") -> Self {\n");
                    self.indent += 1;

                    // Check if there's a super() call in the body
                    let has_super = method.body.iter().any(|stmt| {
                        if let HirStmt::Expr { expr } = stmt {
                            matches!(expr, HirExpr::SuperCall { .. })
                        } else {
                            false
                        }
                    });

                    if has_super && class.parent_class.is_some() {
                        // Inheritance constructor: emit super call, then Self { parent: ..., own fields }
                        let parent_name = class.parent_class.as_ref().unwrap();
                        let mut super_args: Option<&Vec<HirExpr>> = None;
                        let mut field_inits: Vec<(&str, &HirExpr)> = Vec::new();
                        let mut other_stmts: Vec<&HirStmt> = Vec::new();

                        for stmt in &method.body {
                            if let HirStmt::Expr { expr: HirExpr::SuperCall { args, .. } } = stmt {
                                super_args = Some(args);
                            } else if let HirStmt::FieldAssign { field, value, .. } = stmt {
                                field_inits.push((field, value));
                            } else {
                                other_stmts.push(stmt);
                            }
                        }

                        // Emit non-field, non-super statements first
                        for stmt in &other_stmts {
                            self.emit_stmt(stmt);
                        }

                        // Build Self { parent: ParentType::new(...), own_field: value, ... }
                        self.write_indent();
                        self.write("Self {\n");
                        self.indent += 1;

                        // Emit parent field
                        self.write_indent();
                        let parent_field = parent_name.to_lowercase();
                        self.write(&parent_field);
                        self.write(": ");
                        self.write(parent_name);
                        self.write("::new(");
                        if let Some(args) = super_args {
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    self.write(", ");
                                }
                                self.emit_expr(arg);
                            }
                        }
                        self.write("),\n");

                        // Emit own field inits
                        for (field_name, value) in &field_inits {
                            self.write_indent();
                            self.write(field_name);
                            self.write(": ");
                            self.emit_expr(value);
                            self.write(",\n");
                        }

                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                    } else {
                        // Regular constructor
                        let mut field_inits: Vec<(&str, &HirExpr)> = Vec::new();
                        let mut other_stmts: Vec<&HirStmt> = Vec::new();
                        for stmt in &method.body {
                            if let HirStmt::FieldAssign { field, value, .. } = stmt {
                                field_inits.push((field, value));
                            } else {
                                other_stmts.push(stmt);
                            }
                        }

                        // Emit non-field statements first
                        for stmt in &other_stmts {
                            self.emit_stmt(stmt);
                        }

                        // Emit Self { field: value, ... }
                        self.write_indent();
                        self.write("Self {\n");
                        self.indent += 1;
                        for (field_name, value) in &field_inits {
                            self.write_indent();
                            self.write(field_name);
                            self.write(": ");
                            self.emit_expr(value);
                            self.write(",\n");
                        }
                        // For any fields not explicitly assigned, check if param name matches
                        for (field_name, _) in &class.fields {
                            if !field_inits.iter().any(|(f, _)| f == field_name) {
                                if method.params.iter().any(|p| &p.name == field_name) {
                                    self.write_indent();
                                    self.write(field_name);
                                    self.write(",\n");
                                }
                            }
                        }
                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                    }

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                } else {
                    // Regular method: determine &self vs &mut self
                    let is_mutating = body_contains_field_assign_codegen(&method.body);
                    if is_mutating {
                        self.write(&format!("{}fn ", pub_prefix));
                        self.write(&method.name);
                        self.write("(&mut self");
                    } else {
                        self.write(&format!("{}fn ", pub_prefix));
                        self.write(&method.name);
                        self.write("(&self");
                    }
                    for param in &method.params {
                        self.write(", ");
                        self.write(&param.name);
                        self.write(": ");
                        // Pass class types by reference
                        if matches!(param.ty, Type::Class { .. }) {
                            self.write("&");
                        }
                        self.write(&param.ty.rust_type());
                    }
                    self.write(")");

                    if method.return_type != Type::None {
                        self.write(" -> ");
                        self.write(&method.return_type.rust_type());
                    }

                    self.write(" {\n");
                    self.indent += 1;

                    for stmt in &method.body {
                        self.emit_stmt(stmt);
                    }

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                }
            }
        }

        self.current_return_type = None;
        self.mutated_vars.clear();
    }

    fn emit_function(&mut self, func: &HirFunction) {
        // Track the current function's return type for Option wrapping
        self.current_return_type = Some(func.return_type.clone());

        // Pre-scan: collect mutated variables so we know which need `mut`
        self.mutated_vars = collect_mutated_vars(&func.body);

        // Emit decorator comments before the function
        for decorator in &func.decorators {
            self.write_indent();
            self.write(&format!("// @{}\n", decorator));
        }

        // Function signature -- only emit params without defaults, or all params
        // Since Rust doesn't have default params, we emit all params and handle
        // defaults at call site
        self.write_indent();
        if self.pub_mode && func.name != "main" {
            self.write("pub fn ");
        } else {
            self.write("fn ");
        }
        self.write(&func.name);
        self.write("(");

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            // Emit `mut` for parameters that are mutated in the body
            if self.mutated_vars.contains(&param.name) {
                self.write("mut ");
            }
            self.write(&param.name);
            self.write(": ");
            self.write(&param.ty.rust_type());
        }

        self.write(")");

        // Return type (omit for main and for None return)
        if func.return_type != Type::None || func.name != "main" {
            if func.return_type != Type::None {
                self.write(" -> ");
                self.write(&func.return_type.rust_type());
            }
        }

        self.write(" {\n");
        self.indent += 1;

        // Detect if this is a generator function (contains yield statements)
        let is_generator = body_contains_yield(&func.body);
        if is_generator {
            // Emit the yields accumulator at the start
            let yield_ty = if let Type::List(ref elem) = func.return_type {
                elem.rust_type()
            } else {
                "i64".to_string() // fallback
            };
            self.write_indent();
            self.write(&format!("let mut _yields: Vec<{}> = Vec::new();\n", yield_ty));
        }

        // Body
        for stmt in &func.body {
            self.emit_stmt(stmt);
        }

        // If generator, return the accumulated yields
        if is_generator {
            self.write_indent();
            self.write("_yields\n");
        }

        self.indent -= 1;
        self.writeln("}");

        self.current_return_type = None;
        self.mutated_vars.clear();
    }

    fn emit_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let { name, ty, value, is_mutable: _ } => {
                self.write_indent();
                // Only emit `mut` if the variable is actually mutated later
                if self.mutated_vars.contains(name) {
                    self.write("let mut ");
                } else {
                    self.write("let ");
                }
                self.write(name);
                self.write(": ");
                self.write(&ty.rust_type());
                self.write(" = ");
                if is_option_type(ty) && matches!(value, HirExpr::NoneLiteral) {
                    // `x: str | None = None` -> `let x: Option<String> = None`
                    self.write("None");
                } else if is_option_type(ty) && !is_option_type(value.ty()) && !matches!(value.ty(), Type::None) {
                    // RHS is a plain value (not already Option) -> wrap in Some()
                    // But if RHS is a function call returning Option, don't double-wrap
                    self.write("Some(");
                    self.emit_expr(value);
                    self.write(")");
                } else {
                    // RHS already returns the right type (e.g., function returning Option<T>)
                    self.emit_expr(value);
                }
                self.write(";\n");
            }
            HirStmt::Assign { name, value } => {
                self.write_indent();
                self.write(name);
                self.write(" = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::AugAssign { name, op, value } => {
                self.write_indent();
                let var_ty = value.ty();
                match op.as_str() {
                    "+=" => {
                        // Special cases for string and list
                        match var_ty {
                            Type::Str => {
                                self.write(name);
                                self.write(".push_str(");
                                self.emit_str_ref_expr(value);
                                self.write(");\n");
                                return;
                            }
                            _ => {
                                // Check if target is a list (we need to look at the value context)
                                // For list += list, use extend
                                if let Type::List(_) = var_ty {
                                    self.write(name);
                                    self.write(".extend(");
                                    self.emit_expr(value);
                                    self.write(");\n");
                                    return;
                                }
                            }
                        }
                        self.write(name);
                        self.write(" += ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "-=" | "*=" | "%=" => {
                        self.write(name);
                        self.write(&format!(" {} ", op));
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "/=" => {
                        self.write(name);
                        self.write(" /= ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "//=" => {
                        self.write(name);
                        self.write(" /= ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "**=" => {
                        self.write(name);
                        self.write(" = ");
                        self.write(&format!("({} as f64).powf(", name));
                        self.emit_expr(value);
                        self.write(" as f64);\n");
                    }
                    _ => {
                        self.write(name);
                        self.write(&format!(" {} ", op));
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                }
            }
            HirStmt::Return { value } => {
                let ret_is_option = self.current_return_type.as_ref().map_or(false, |t| is_option_type(t));
                self.write_indent();
                if let Some(val) = value {
                    self.write("return ");
                    if ret_is_option && matches!(val, HirExpr::NoneLiteral) {
                        // `return None` in Python -> `return None` in Rust Option
                        self.write("None");
                    } else if ret_is_option && !is_option_type(val.ty()) {
                        // Returning a non-Option value from an Option function -> wrap in Some()
                        self.write("Some(");
                        self.emit_expr(val);
                        self.write(")");
                    } else {
                        self.emit_expr(val);
                    }
                    self.write(";\n");
                } else {
                    if ret_is_option {
                        self.write("return None;\n");
                    } else {
                        self.write("return;\n");
                    }
                }
            }
            HirStmt::Expr { expr } => {
                self.write_indent();
                self.emit_expr(expr);
                self.write(";\n");
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                // Detect isinstance narrowing for union enums:
                // `if isinstance(x, int):` -> `match x { IntOrStr::Int(x) => { ... }, IntOrStr::Str(x) => { ... } }`
                if let Some((var_name, variant_name, enum_name, other_variants)) = detect_isinstance_union(condition) {
                    self.write_indent();
                    self.write(&format!("match {} {{\n", var_name));
                    self.indent += 1;

                    // Then branch: the matched variant
                    self.write_indent();
                    self.write(&format!("{}::{}({}) => {{\n", enum_name, variant_name, var_name));
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");

                    // Else branch: the other variant(s)
                    if let Some(else_stmts) = else_body {
                        if other_variants.len() == 1 {
                            let (other_variant, _) = &other_variants[0];
                            self.write_indent();
                            self.write(&format!("{}::{}({}) => {{\n", enum_name, other_variant, var_name));
                        } else {
                            self.write_indent();
                            self.write("_ => {\n");
                        }
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                        self.writeln("}");
                    }

                    self.indent -= 1;
                    self.writeln("}");
                }
                // Detect truthiness on Option: `if x:` where x is Option -> `if let Some(x) = x {`
                else if let Some(var_name) = detect_option_truthiness(condition) {
                    self.write_indent();
                    self.write(&format!("if let Some({}) = {} {{\n", var_name, var_name));
                    self.indent += 1;
                    self.option_unwrapped_vars.insert(var_name.clone());
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.option_unwrapped_vars.remove(&var_name);
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("} else {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }
                    self.writeln("}");
                }
                // Detect Option narrowing: `if x is not None:` -> `if let Some(x) = x {`
                else if let Some(var_name) = detect_is_not_none_var(condition) {
                    self.write_indent();
                    // Use `if let Some(var) = var` to unwrap and shadow the variable
                    self.write(&format!("if let Some({}) = {} {{\n", var_name, var_name));
                    self.indent += 1;
                    self.option_unwrapped_vars.insert(var_name.clone());
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.option_unwrapped_vars.remove(&var_name);
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("} else {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }
                    self.writeln("}");
                } else if let Some(var_name) = detect_is_none_var(condition) {
                    self.write_indent();
                    self.write(&format!("if {}.is_none() {{\n", var_name));
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        // In the else branch of `if x is None`, x is not None
                        self.write_indent();
                        self.write(&format!("}} else if let Some({}) = {} {{\n", var_name, var_name));
                        self.indent += 1;
                        self.option_unwrapped_vars.insert(var_name.clone());
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.option_unwrapped_vars.remove(&var_name);
                        self.indent -= 1;
                    }
                    self.writeln("}");
                } else {
                    // Normal if/elif/else
                    // Hoist any walrus expressions before the if
                    self.emit_walrus_hoists(condition);
                    self.write_indent();
                    self.write("if ");
                    self.emit_expr(condition);
                    self.write(" {\n");
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;

                    for (cond, body) in elif_clauses {
                        self.write_indent();
                        self.write("} else if ");
                        self.emit_expr(cond);
                        self.write(" {\n");
                        self.indent += 1;
                        for s in body {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }

                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("} else {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }

                    self.writeln("}");
                }
            }
            HirStmt::While { condition, body, else_body } => {
                let has_else = else_body.is_some();
                if has_else {
                    self.writeln("let mut _broke = false;");
                }
                let prev_loop_else = self.in_loop_with_else;
                self.in_loop_with_else = has_else;
                // Hoist any walrus expressions
                self.emit_walrus_hoists(condition);
                self.write_indent();
                self.write("while ");
                self.emit_expr(condition);
                self.write(" {\n");
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.writeln("}");
                self.in_loop_with_else = prev_loop_else;
                if let Some(else_stmts) = else_body {
                    self.writeln("if !_broke {");
                    self.indent += 1;
                    for s in else_stmts {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");
                }
            }
            HirStmt::For { target, iter, body, else_body, .. } => {
                let has_else = else_body.is_some();
                if has_else {
                    self.writeln("let mut _broke = false;");
                }
                let prev_loop_else = self.in_loop_with_else;
                self.in_loop_with_else = has_else;
                self.write_indent();
                self.write("for ");
                self.write(target);
                self.write(" in ");
                // For lists, iterate with .iter() to borrow and clone elements
                // But not for generator expressions which are already iterators
                let is_generator = matches!(iter, HirExpr::GeneratorExpr { .. });
                let is_list = matches!(iter.ty(), Type::List(_));
                let is_dict = matches!(iter.ty(), Type::Dict(_, _));
                self.emit_expr(iter);
                if is_generator {
                    // Generator expressions are already iterators, no .iter() needed
                } else if is_list {
                    self.write(".iter().cloned()");
                } else if is_dict {
                    self.write(".keys().cloned()");
                }
                self.write(" {\n");
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.writeln("}");
                self.in_loop_with_else = prev_loop_else;
                if let Some(else_stmts) = else_body {
                    self.writeln("if !_broke {");
                    self.indent += 1;
                    for s in else_stmts {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");
                }
            }
            HirStmt::Break => {
                if self.in_loop_with_else {
                    self.writeln("_broke = true;");
                }
                self.writeln("break;");
            }
            HirStmt::Continue => {
                self.writeln("continue;");
            }
            HirStmt::Pass => {
                // No-op in Rust
            }
            HirStmt::TupleUnpack { targets, value } => {
                self.write_indent();
                self.write("let (");
                for (i, (name, _ty)) in targets.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(name);
                }
                self.write(") = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::StarUnpack { before, star, after, value } => {
                // Emit: let _tmp = value.clone() to avoid moving;
                self.write_indent();
                self.write("let _star_tmp = ");
                self.emit_expr(value);
                self.write(".clone();\n");
                // Emit before vars
                for (i, (name, _ty)) in before.iter().enumerate() {
                    self.write_indent();
                    self.write(&format!("let {} = _star_tmp[{}].clone();\n", name, i));
                }
                // Emit star var
                let (star_name, _star_ty) = star;
                if after.is_empty() {
                    self.write_indent();
                    self.write(&format!("let {} = _star_tmp[{}..].to_vec();\n", star_name, before.len()));
                } else {
                    self.write_indent();
                    self.write(&format!("let {} = _star_tmp[{}.._star_tmp.len() - {}].to_vec();\n", star_name, before.len(), after.len()));
                }
                // Emit after vars
                for (i, (name, _ty)) in after.iter().enumerate() {
                    self.write_indent();
                    self.write(&format!("let {} = _star_tmp[_star_tmp.len() - {}].clone();\n", name, after.len() - i));
                }
            }
            HirStmt::TryExcept { body, handlers } => {
                // Determine the error type from the first handler
                let error_rust_type = handlers.first()
                    .and_then(|h| h.error_resolved_type.as_ref())
                    .map(|t| t.rust_type())
                    .unwrap_or_else(|| "String".to_string());

                // Emit try body as a closure that returns Result, then match on it
                self.write_indent();
                self.write(&format!("match (|| -> Result<(), {}> {{\n", error_rust_type));
                self.indent += 1;
                for stmt in body {
                    self.emit_stmt(stmt);
                }
                self.write_indent();
                self.write("Ok(())\n");
                self.indent -= 1;
                self.write_indent();
                self.write("})() {\n");
                self.indent += 1;
                self.write_indent();
                self.write("Ok(()) => {}\n");
                for handler in handlers {
                    self.write_indent();
                    if let Some(ref name) = handler.name {
                        self.write(&format!("Err({}) => {{\n", name));
                    } else {
                        self.write("Err(_e) => {\n");
                    }
                    self.indent += 1;
                    for stmt in &handler.body {
                        self.emit_stmt(stmt);
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
            HirStmt::Raise { value } => {
                self.write_indent();
                self.write("return Err(");
                self.emit_expr(value);
                self.write(");\n");
            }
            HirStmt::Assert { test, msg } => {
                self.write_indent();
                if let Some(msg_expr) = msg {
                    self.write("assert!(");
                    self.emit_expr(test);
                    self.write(", \"{}\", ");
                    self.emit_display_expr(msg_expr);
                    self.write(");\n");
                } else {
                    self.write("assert!(");
                    self.emit_expr(test);
                    self.write(");\n");
                }
            }
            HirStmt::FieldAssign { object, field, value } => {
                self.write_indent();
                // Check if this is assigning to a parent field via inheritance
                if let Some(ref class_name) = self.current_class_name.clone() {
                    if let Some((parent_name, parent_field_names)) = self.parent_fields.get(class_name).cloned() {
                        if parent_field_names.contains(field.as_str()) {
                            self.write(object);
                            self.write(".");
                            self.write(&parent_name.to_lowercase());
                            self.write(".");
                            self.write(field);
                            self.write(" = ");
                            self.emit_expr(value);
                            self.write(";\n");
                            return;
                        }
                    }
                }
                self.write(object);
                self.write(".");
                self.write(field);
                self.write(" = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::Delete { object, index } => {
                let obj_ty = object.ty();
                self.write_indent();
                match obj_ty {
                    Type::Dict(_, _) => {
                        // del d[key] -> let _ = d.remove(&key);
                        self.write("let _ = ");
                        self.emit_expr(object);
                        self.write(".remove(");
                        self.emit_key_ref_expr(index);
                        self.write(");\n");
                    }
                    Type::List(_) => {
                        // del a[i] -> let _ = a.remove(i as usize);
                        self.write("let _ = ");
                        self.emit_expr(object);
                        self.write(".remove(");
                        self.emit_expr(index);
                        self.write(" as usize);\n");
                    }
                    _ => {
                        self.write("/* unsupported del */\n");
                    }
                }
            }
            HirStmt::Yield { value } => {
                self.write_indent();
                self.write("_yields.push(");
                self.emit_expr(value);
                self.write(");\n");
            }
            HirStmt::With { var, value, body } => {
                self.write_indent();
                self.write("{\n");
                self.indent += 1;
                self.write_indent();
                self.write("let ");
                self.write(var);
                self.write(" = ");
                self.emit_expr(value);
                self.write(";\n");
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
        }
    }

    /// Emit any walrus (named expression) assignments that need to be hoisted before a condition.
    fn emit_walrus_hoists(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::WalrusExpr { name, value, ty } => {
                self.write_indent();
                self.write("let ");
                self.write(name);
                self.write(": ");
                self.write(&ty.rust_type());
                self.write(" = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            HirExpr::Compare { left, comparators, .. } => {
                self.emit_walrus_hoists(left);
                for c in comparators {
                    self.emit_walrus_hoists(c);
                }
            }
            HirExpr::BoolOp { values, .. } => {
                for v in values {
                    self.emit_walrus_hoists(v);
                }
            }
            HirExpr::BinOp { left, right, .. } => {
                self.emit_walrus_hoists(left);
                self.emit_walrus_hoists(right);
            }
            _ => {}
        }
    }

    fn emit_list_slice(&mut self, object: &HirExpr, start: &Option<Box<HirExpr>>, stop: &Option<Box<HirExpr>>, step: &Option<Box<HirExpr>>) {
        if let Some(step_expr) = step {
            // Step slicing
            self.write("{ let _v = &");
            self.emit_expr(object);
            self.write("; let _len = _v.len() as i64; let _step = ");
            self.emit_expr(step_expr);
            self.write("; ");

            // Resolve start
            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _s = ");
                self.emit_expr(s);
                self.write("; if _s < 0 { ((_len + _s).max(0)) as usize } else { (_s.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { 0 } else { (_len - 1) as usize }");
            }
            self.write("; ");

            // Resolve stop
            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _e = ");
                self.emit_expr(e);
                self.write("; if _e < 0 { ((_len + _e).max(0)) as usize } else { (_e.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { _len as usize } else { 0_usize.wrapping_sub(1) }");
            }
            self.write("; ");

            // Build result
            self.write("let mut _result = Vec::new(); ");
            self.write("if _step > 0 { let mut _i = _start; while _i < _stop { _result.push(_v[_i].clone()); _i += _step as usize; } }");
            self.write(" else { let mut _i = _start as i64; let _stop_i = _stop as i64; while _i > _stop_i { _result.push(_v[_i as usize].clone()); _i += _step; } }");
            self.write("; _result }");
        } else {
            // Simple slice without step
            self.write("{ let _v = &");
            self.emit_expr(object);
            self.write("; let _len = _v.len() as i64; ");

            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _s = ");
                self.emit_expr(s);
                self.write("; if _s < 0 { ((_len + _s).max(0)) as usize } else { (_s.min(_len)) as usize } }");
            } else {
                self.write("0_usize");
            }
            self.write("; ");

            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _e = ");
                self.emit_expr(e);
                self.write("; if _e < 0 { ((_len + _e).max(0)) as usize } else { (_e.min(_len)) as usize } }");
            } else {
                self.write("_len as usize");
            }
            self.write("; ");

            self.write("_v[_start.._stop].to_vec() }");
        }
    }

    fn emit_string_slice(&mut self, object: &HirExpr, start: &Option<Box<HirExpr>>, stop: &Option<Box<HirExpr>>, step: &Option<Box<HirExpr>>) {
        if let Some(step_expr) = step {
            self.write("{ let _s: Vec<char> = ");
            self.emit_expr(object);
            self.write(".chars().collect(); let _len = _s.len() as i64; let _step = ");
            self.emit_expr(step_expr);
            self.write("; ");

            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _sv = ");
                self.emit_expr(s);
                self.write("; if _sv < 0 { ((_len + _sv).max(0)) as usize } else { (_sv.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { 0 } else { (_len - 1) as usize }");
            }
            self.write("; ");

            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _ev = ");
                self.emit_expr(e);
                self.write("; if _ev < 0 { ((_len + _ev).max(0)) as usize } else { (_ev.min(_len)) as usize } }");
            } else {
                self.write("if _step > 0 { _len as usize } else { 0_usize.wrapping_sub(1) }");
            }
            self.write("; ");

            self.write("let mut _result = String::new(); ");
            self.write("if _step > 0 { let mut _i = _start; while _i < _stop { _result.push(_s[_i]); _i += _step as usize; } }");
            self.write(" else { let mut _i = _start as i64; let _stop_i = _stop as i64; while _i > _stop_i { _result.push(_s[_i as usize]); _i += _step; } }");
            self.write("; _result }");
        } else {
            self.write("{ let _s = &");
            self.emit_expr(object);
            self.write("; let _len = _s.chars().count() as i64; ");

            self.write("let _start = ");
            if let Some(s) = start {
                self.write("{ let _sv = ");
                self.emit_expr(s);
                self.write("; if _sv < 0 { ((_len + _sv).max(0)) as usize } else { (_sv.min(_len)) as usize } }");
            } else {
                self.write("0_usize");
            }
            self.write("; ");

            self.write("let _stop = ");
            if let Some(e) = stop {
                self.write("{ let _ev = ");
                self.emit_expr(e);
                self.write("; if _ev < 0 { ((_len + _ev).max(0)) as usize } else { (_ev.min(_len)) as usize } }");
            } else {
                self.write("_len as usize");
            }
            self.write("; ");

            self.write("_s.chars().skip(_start).take(_stop - _start).collect::<String>() }");
        }
    }

    fn emit_method_call(&mut self, object: &HirExpr, method: &str, args: &[HirExpr]) {
        let obj_ty = object.ty();
        match (obj_ty, method) {
            // String methods
            (Type::Str, "upper") => {
                self.emit_expr(object);
                self.write(".to_uppercase()");
            }
            (Type::Str, "lower") => {
                self.emit_expr(object);
                self.write(".to_lowercase()");
            }
            (Type::Str, "strip") => {
                self.emit_expr(object);
                self.write(".trim().to_string()");
            }
            (Type::Str, "lstrip") => {
                self.emit_expr(object);
                self.write(".trim_start().to_string()");
            }
            (Type::Str, "rstrip") => {
                self.emit_expr(object);
                self.write(".trim_end().to_string()");
            }
            (Type::Str, "startswith") => {
                self.emit_expr(object);
                self.write(".starts_with(");
                if !args.is_empty() {
                    self.emit_str_ref_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Str, "endswith") => {
                self.emit_expr(object);
                self.write(".ends_with(");
                if !args.is_empty() {
                    self.emit_str_ref_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Str, "split") => {
                self.emit_expr(object);
                if args.is_empty() {
                    self.write(".split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>()");
                } else {
                    self.write(".split(");
                    self.emit_str_ref_expr(&args[0]);
                    self.write(").map(|s| s.to_string()).collect::<Vec<String>>()");
                }
            }
            (Type::Str, "replace") => {
                self.emit_expr(object);
                self.write(".replace(");
                if args.len() >= 2 {
                    self.emit_str_ref_expr(&args[0]);
                    self.write(", ");
                    self.emit_str_ref_expr(&args[1]);
                }
                self.write(")");
            }
            (Type::Str, "find") => {
                // Returns Option<i64> = int | None
                self.emit_expr(object);
                self.write(".find(");
                if !args.is_empty() {
                    self.emit_str_ref_expr(&args[0]);
                }
                self.write(").map(|i| i as i64)");
            }
            // String methods - extended
            (Type::Str, "title") => {
                // Title case: capitalize first letter of each word
                self.emit_expr(object);
                self.write(".split_whitespace().map(|w| { let mut c = w.chars(); match c.next() { None => String::new(), Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase() } }).collect::<Vec<_>>().join(\" \")");
            }
            (Type::Str, "capitalize") => {
                self.write("{ let _s = ");
                self.emit_expr(object);
                self.write("; let mut _c = _s.chars(); match _c.next() { None => String::new(), Some(f) => f.to_uppercase().to_string() + &_c.as_str().to_lowercase() } }");
            }
            (Type::Str, "swapcase") => {
                self.emit_expr(object);
                self.write(".chars().map(|c| if c.is_uppercase() { c.to_lowercase().to_string() } else { c.to_uppercase().to_string() }).collect::<String>()");
            }
            (Type::Str, "isdigit") => {
                self.write("!");
                self.emit_expr(object);
                self.write(".is_empty() && ");
                self.emit_expr(object);
                self.write(".chars().all(|c| c.is_ascii_digit())");
            }
            (Type::Str, "isalpha") => {
                self.write("!");
                self.emit_expr(object);
                self.write(".is_empty() && ");
                self.emit_expr(object);
                self.write(".chars().all(|c| c.is_alphabetic())");
            }
            (Type::Str, "isalnum") => {
                self.write("!");
                self.emit_expr(object);
                self.write(".is_empty() && ");
                self.emit_expr(object);
                self.write(".chars().all(|c| c.is_alphanumeric())");
            }
            (Type::Str, "isspace") => {
                self.write("!");
                self.emit_expr(object);
                self.write(".is_empty() && ");
                self.emit_expr(object);
                self.write(".chars().all(|c| c.is_whitespace())");
            }
            (Type::Str, "isupper") => {
                self.emit_expr(object);
                self.write(".chars().any(|c| c.is_alphabetic()) && ");
                self.emit_expr(object);
                self.write(".chars().filter(|c| c.is_alphabetic()).all(|c| c.is_uppercase())");
            }
            (Type::Str, "islower") => {
                self.emit_expr(object);
                self.write(".chars().any(|c| c.is_alphabetic()) && ");
                self.emit_expr(object);
                self.write(".chars().filter(|c| c.is_alphabetic()).all(|c| c.is_lowercase())");
            }
            (Type::Str, "join") => {
                // Python: "sep".join(items) -> Rust: items.join("sep")
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                    self.write(".join(");
                    self.emit_str_ref_expr(object);
                    self.write(")");
                }
            }
            (Type::Str, "count") => {
                self.emit_expr(object);
                self.write(".matches(");
                if !args.is_empty() {
                    self.emit_str_ref_expr(&args[0]);
                }
                self.write(").count() as i64");
            }
            (Type::Str, "center") => {
                self.write("{ let _s = ");
                self.emit_expr(object);
                self.write("; let _w = ");
                if !args.is_empty() { self.emit_expr(&args[0]); }
                self.write(" as usize; let _len = _s.chars().count(); if _len >= _w { _s } else { let _pad = _w - _len; let _left = _pad / 2; let _right = _pad - _left; format!(\"{}{}{}\", \" \".repeat(_left), _s, \" \".repeat(_right)) } }");
            }
            (Type::Str, "ljust") => {
                self.write("format!(\"{:<width$}\", ");
                self.emit_expr(object);
                self.write(", width = ");
                if !args.is_empty() { self.emit_expr(&args[0]); }
                self.write(" as usize)");
            }
            (Type::Str, "rjust") => {
                self.write("format!(\"{:>width$}\", ");
                self.emit_expr(object);
                self.write(", width = ");
                if !args.is_empty() { self.emit_expr(&args[0]); }
                self.write(" as usize)");
            }
            (Type::Str, "zfill") => {
                self.write("format!(\"{:0>width$}\", ");
                self.emit_expr(object);
                self.write(", width = ");
                if !args.is_empty() { self.emit_expr(&args[0]); }
                self.write(" as usize)");
            }
            // List methods
            (Type::List(_), "append") => {
                self.emit_expr(object);
                self.write(".push(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::List(_), "extend") => {
                self.emit_expr(object);
                self.write(".extend(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::List(_), "insert") => {
                self.emit_expr(object);
                self.write(".insert(");
                if args.len() >= 2 {
                    self.emit_expr(&args[0]);
                    self.write(" as usize, ");
                    self.emit_expr(&args[1]);
                }
                self.write(")");
            }
            (Type::List(_), "clear") => {
                self.emit_expr(object);
                self.write(".clear()");
            }
            (Type::List(_), "copy") => {
                self.emit_expr(object);
                self.write(".clone()");
            }
            (Type::List(_), "reverse") => {
                self.emit_expr(object);
                self.write(".reverse()");
            }
            (Type::List(_), "sort") => {
                self.emit_expr(object);
                self.write(".sort()");
            }
            (Type::List(_), "count") => {
                self.emit_expr(object);
                self.write(".iter().filter(|x| **x == ");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(").count() as i64");
            }
            (Type::List(_), "contains") => {
                self.emit_expr(object);
                self.write(".contains(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::List(_), "pop") => {
                // Returns Option<T> = T | None
                self.emit_expr(object);
                self.write(".pop()");
            }
            // Dict methods
            (Type::Dict(_, _), "keys") => {
                self.emit_expr(object);
                self.write(".keys().cloned().collect::<Vec<_>>()");
            }
            (Type::Dict(_, _), "values") => {
                self.emit_expr(object);
                self.write(".values().cloned().collect::<Vec<_>>()");
            }
            (Type::Dict(_, _), "items") => {
                self.emit_expr(object);
                self.write(".iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()");
            }
            (Type::Dict(_, _), "update") => {
                self.emit_expr(object);
                self.write(".extend(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Dict(_, _), "clear") => {
                self.emit_expr(object);
                self.write(".clear()");
            }
            (Type::Dict(_, _), "copy") => {
                self.emit_expr(object);
                self.write(".clone()");
            }
            (Type::Dict(_, _), "contains") => {
                self.emit_expr(object);
                self.write(".contains_key(");
                if !args.is_empty() {
                    self.emit_key_ref_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::Dict(_, _), "get") => {
                // Returns Option<V> = V | None
                self.emit_expr(object);
                self.write(".get(");
                if !args.is_empty() {
                    self.emit_key_ref_expr(&args[0]);
                }
                self.write(").cloned()");
            }
            (Type::Dict(_, _), "pop") => {
                // Returns Option<V> = V | None
                self.emit_expr(object);
                self.write(".remove(");
                if !args.is_empty() {
                    self.emit_key_ref_expr(&args[0]);
                }
                self.write(")");
            }
            // Tuple count()
            (Type::Tuple(_), "count") => {
                // For tuples, count is tricky - we need to check each element
                // For now, emit a simple comparison chain
                self.write("0_i64 /* tuple.count() not fully supported */");
            }
            // Tuple len() - compile-time constant
            (Type::Tuple(elems), "len") => {
                self.write(&format!("{}_i64", elems.len()));
            }
            // String len() - character count
            (Type::Str, "len") => {
                self.emit_expr(object);
                self.write(".chars().count() as i64");
            }
            // Generic len() for all types
            (_, "len") => {
                self.emit_expr(object);
                self.write(".len() as i64");
            }
            (Type::Class { .. }, _) => {
                // Class instance method call
                self.emit_expr(object);
                self.write(&format!(".{}(", method));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    // Pass class instances by reference
                    if matches!(arg.ty(), Type::Class { .. }) {
                        self.write("&");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
            _ => {
                // Fallback: emit as-is
                self.emit_expr(object);
                self.write(&format!(".{}(", method));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
        }
    }

    fn emit_expr(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::IntLiteral(val) => {
                self.write(&val.to_string());
                self.write("_i64");
            }
            HirExpr::FloatLiteral(val) => {
                let s = val.to_string();
                self.write(&s);
                if !s.contains('.') {
                    self.write(".0");
                }
                self.write("_f64");
            }
            HirExpr::StringLiteral(val) => {
                self.write(&format!("{:?}.to_string()", val));
            }
            HirExpr::BoolLiteral(val) => {
                self.write(if *val { "true" } else { "false" });
            }
            HirExpr::NoneLiteral => {
                // None in sifr maps to Rust's None (for Option contexts)
                // The parent (Let/Return) handles the wrapping context
                self.write("None");
            }
            HirExpr::Name { name, .. } => {
                self.write(name);
            }
            HirExpr::BinOp { left, op, right, ty } => {
                // Special handling for string concatenation
                if op == "+" && *ty == Type::Str {
                    // Flatten chained string concatenation into a single format! call
                    let mut parts: Vec<&HirExpr> = Vec::new();
                    collect_string_concat_parts(left, &mut parts);
                    collect_string_concat_parts(right, &mut parts);
                    let placeholders: String = parts.iter().map(|_| "{}").collect::<Vec<_>>().join("");
                    self.write(&format!("format!(\"{}\"", placeholders));
                    for part in &parts {
                        self.write(", ");
                        self.emit_expr(part);
                    }
                    self.write(")");
                } else if op == "//" {
                    // Floor division
                    self.emit_expr(left);
                    self.write(" / ");
                    self.emit_expr(right);
                } else if op == "**" {
                    // Power: int ** int -> i64::pow, otherwise float
                    if left.ty() == &Type::Int && right.ty() == &Type::Int {
                        self.emit_expr(left);
                        self.write(".pow(");
                        self.emit_expr(right);
                        self.write(" as u32)");
                    } else if left.ty() == &Type::Float && right.ty() == &Type::Int {
                        self.emit_expr(left);
                        self.write(".powi(");
                        self.emit_expr(right);
                        self.write(" as i32)");
                    } else {
                        self.write("(");
                        self.emit_expr(left);
                        self.write(" as f64).powf(");
                        self.emit_expr(right);
                        self.write(" as f64)");
                    }
                } else if op == "*" && left.ty() == &Type::Str && right.ty() == &Type::Int {
                    // String multiplication: "abc" * 3 -> "abc".repeat(3)
                    self.emit_expr(left);
                    self.write(".repeat(");
                    self.emit_expr(right);
                    self.write(" as usize)");
                } else if op == "*" && left.ty() == &Type::Int && right.ty() == &Type::Str {
                    // Reverse string multiplication: 3 * "abc"
                    self.emit_expr(right);
                    self.write(".repeat(");
                    self.emit_expr(left);
                    self.write(" as usize)");
                } else if matches!(left.ty(), Type::Class { .. }) {
                    // Class type with operator overloading: use reference-based ops
                    self.write("&");
                    self.emit_expr(left);
                    self.write(&format!(" {} ", op));
                    self.write("&");
                    self.emit_expr(right);
                } else {
                    // Wrap sub-expressions in parens if they are BinOps to preserve precedence
                    let needs_left_parens = matches!(left.as_ref(), HirExpr::BinOp { .. });
                    let needs_right_parens = matches!(right.as_ref(), HirExpr::BinOp { .. });
                    if needs_left_parens { self.write("("); }
                    self.emit_expr(left);
                    if needs_left_parens { self.write(")"); }
                    self.write(&format!(" {} ", op));
                    if needs_right_parens { self.write("("); }
                    self.emit_expr(right);
                    if needs_right_parens { self.write(")"); }
                }
            }
            HirExpr::UnaryOp { op, operand, .. } => {
                if op == "not" {
                    self.write("!");
                } else {
                    self.write(op);
                }
                self.emit_expr(operand);
            }
            HirExpr::Compare { left, ops, comparators, .. } => {
                // For single comparison
                if ops.len() == 1 {
                    let op = &ops[0];
                    // Handle `is None` / `is not None` for Option types
                    if (op == "is" || op == "is not") && matches!(comparators[0], HirExpr::NoneLiteral) {
                        self.emit_expr(left);
                        if op == "is" {
                            self.write(".is_none()");
                        } else {
                            self.write(".is_some()");
                        }
                    } else if op == "is" {
                        self.emit_expr(left);
                        self.write(" == ");
                        self.emit_expr(&comparators[0]);
                    } else if op == "is not" {
                        self.emit_expr(left);
                        self.write(" != ");
                        self.emit_expr(&comparators[0]);
                    } else {
                        self.emit_expr(left);
                        self.write(&format!(" {} ", op));
                        self.emit_expr(&comparators[0]);
                    }
                } else {
                    // Chained comparisons: a < b < c -> a < b && b < c
                    self.write("(");
                    self.emit_expr(left);
                    self.write(&format!(" {} ", ops[0]));
                    self.emit_expr(&comparators[0]);
                    for i in 1..ops.len() {
                        self.write(" && ");
                        self.emit_expr(&comparators[i - 1]);
                        self.write(&format!(" {} ", ops[i]));
                        self.emit_expr(&comparators[i]);
                    }
                    self.write(")");
                }
            }
            HirExpr::BoolOp { op, values, .. } => {
                let rust_op = if op == "and" { "&&" } else { "||" };
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        self.write(&format!(" {} ", rust_op));
                    }
                    self.emit_expr(val);
                }
            }
            HirExpr::Call { func, args, .. } => {
                if func == "print" {
                    // Map print() to println!
                    if args.is_empty() {
                        self.write("println!()");
                    } else if let HirExpr::FString { parts, .. } = &args[0] {
                        // Inline f-string directly into println! to avoid double-format
                        self.emit_fstring_macro("println!", parts);
                    } else if matches!(args[0].ty(), Type::Class { .. } | Type::Newtype { .. }) {
                        // Check if class has Display impl
                        let class_name = match args[0].ty() {
                            Type::Class { name, .. } | Type::Newtype { name, .. } => name.clone(),
                            _ => String::new(),
                        };
                        if self.display_classes.contains(&class_name) {
                            self.write("println!(\"{}\", ");
                        } else {
                            self.write("println!(\"{:?}\", ");
                        }
                        self.emit_expr(&args[0]);
                        self.write(")");
                    } else if matches!(args[0].ty(), Type::List(_) | Type::Dict(_, _) | Type::Tuple(_)) {
                        // Collections use Debug format
                        self.write("println!(\"{:?}\", ");
                        self.emit_expr(&args[0]);
                        self.write(")");
                    } else {
                        // Use emit_display_expr for all other cases:
                        // - Option<T> gets map_or wrapping
                        // - String literals omit .to_string()
                        // - Everything else emits normally
                        self.write("println!(\"{}\", ");
                        self.emit_display_expr(&args[0]);
                        self.write(")");
                    };
                } else if func == "isinstance" {
                    // isinstance() is handled by narrowing at the HIR level.
                    // At codegen time, we emit `true` since the narrowing has
                    // already validated the types. In practice, isinstance checks
                    // appear in if-conditions and the narrowing determines which
                    // branch to take.
                    self.write("true");
                } else if func == "str" {
                    // str() conversion -> format!("{}", arg)
                    if !args.is_empty() {
                        self.write("format!(\"{}\", ");
                        self.emit_display_expr(&args[0]);
                        self.write(")");
                    } else {
                        self.write("String::new()");
                    }
                } else if func == "abs" {
                    if !args.is_empty() {
                        self.write("(");
                        self.emit_expr(&args[0]);
                        self.write(").abs()");
                    }
                } else if func == "hash" {
                    // hash(x) -> { use std::hash::{Hash, Hasher}; let mut h = std::collections::hash_map::DefaultHasher::new(); x.hash(&mut h); h.finish() as i64 }
                    if !args.is_empty() {
                        self.write("{ use std::hash::{Hash, Hasher}; let mut _h = std::collections::hash_map::DefaultHasher::new(); ");
                        self.emit_expr(&args[0]);
                        self.write(".hash(&mut _h); _h.finish() as i64 }");
                    }
                } else if func == "round" {
                    if args.len() == 1 {
                        self.emit_expr(&args[0]);
                        self.write(".round() as i64");
                    } else if args.len() == 2 {
                        // round(x, n) -> (x * 10^n).round() / 10^n
                        self.write("((");
                        self.emit_expr(&args[0]);
                        self.write(" as f64 * 10.0_f64.powi(");
                        self.emit_expr(&args[1]);
                        self.write(" as i32)).round() / 10.0_f64.powi(");
                        self.emit_expr(&args[1]);
                        self.write(" as i32))");
                    }
                } else if func == "repr" {
                    if !args.is_empty() {
                        self.write("format!(\"{:?}\", ");
                        self.emit_expr(&args[0]);
                        self.write(")");
                    }
                } else if func == "int" {
                    if !args.is_empty() {
                        match args[0].ty() {
                            Type::Float => {
                                self.emit_expr(&args[0]);
                                self.write(" as i64");
                            }
                            Type::Str => {
                                // int(str) -> Result<i64, String>
                                self.emit_expr(&args[0]);
                                self.write(".parse::<i64>().map_err(|e| e.to_string())");
                            }
                            Type::Bool => {
                                self.write("if ");
                                self.emit_expr(&args[0]);
                                self.write(" { 1_i64 } else { 0_i64 }");
                            }
                            _ => {
                                self.emit_expr(&args[0]);
                            }
                        }
                    }
                } else if func == "float" {
                    if !args.is_empty() {
                        match args[0].ty() {
                            Type::Int => {
                                self.emit_expr(&args[0]);
                                self.write(" as f64");
                            }
                            Type::Str => {
                                // float(str) -> Result<f64, String>
                                self.emit_expr(&args[0]);
                                self.write(".parse::<f64>().map_err(|e| e.to_string())");
                            }
                            _ => {
                                self.emit_expr(&args[0]);
                            }
                        }
                    }
                } else if func == "bool" {
                    if !args.is_empty() {
                        match args[0].ty() {
                            Type::Int => {
                                self.emit_expr(&args[0]);
                                self.write(" != 0");
                            }
                            Type::Str => {
                                self.write("!");
                                self.emit_expr(&args[0]);
                                self.write(".is_empty()");
                            }
                            _ => {
                                self.emit_expr(&args[0]);
                            }
                        }
                    }
                } else if func == "min" {
                    // min(list) -> *list.iter().min().unwrap()
                    self.emit_expr(&args[0]);
                    if matches!(args[0].ty(), Type::List(ref e) if matches!(e.as_ref(), Type::Float)) {
                        self.write(".iter().cloned().reduce(f64::min).unwrap()");
                    } else {
                        self.write(".iter().min().unwrap().clone()");
                    }
                } else if func == "max" {
                    // max(list) -> *list.iter().max().unwrap()
                    self.emit_expr(&args[0]);
                    if matches!(args[0].ty(), Type::List(ref e) if matches!(e.as_ref(), Type::Float)) {
                        self.write(".iter().cloned().reduce(f64::max).unwrap()");
                    } else {
                        self.write(".iter().max().unwrap().clone()");
                    }
                } else if func == "sum" {
                    // sum(list) -> list.iter().sum()
                    self.emit_expr(&args[0]);
                    self.write(".iter().sum::<");
                    if let Type::List(ref elem) = args[0].ty() {
                        self.write(&elem.rust_type());
                    } else {
                        self.write("_");
                    }
                    self.write(">()");
                } else if func == "sorted" {
                    // sorted(list) -> { let mut v = list.clone(); v.sort(); v }
                    self.write("{ let mut _sorted = ");
                    self.emit_expr(&args[0]);
                    self.write(".clone(); _sorted.sort(); _sorted }");
                } else if func == "reversed" {
                    // reversed(list) -> { let mut v = list.clone(); v.reverse(); v }
                    self.write("{ let mut _rev = ");
                    self.emit_expr(&args[0]);
                    self.write(".clone(); _rev.reverse(); _rev }");
                } else if func == "enumerate" {
                    // enumerate(list) -> list.iter().enumerate().map(|(i, v)| (i as i64, v.clone())).collect()
                    self.emit_expr(&args[0]);
                    self.write(".iter().enumerate().map(|(i, v)| (i as i64, v.clone())).collect::<Vec<_>>()");
                } else if func == "zip" {
                    // zip(a, b) -> a.iter().zip(b.iter()).map(|(a, b)| (a.clone(), b.clone())).collect()
                    self.emit_expr(&args[0]);
                    self.write(".iter().zip(");
                    self.emit_expr(&args[1]);
                    self.write(".iter()).map(|(a, b)| (a.clone(), b.clone())).collect::<Vec<_>>()");
                } else if func == "any" {
                    // any(list) -> list.iter().any(|x| *x)
                    self.emit_expr(&args[0]);
                    self.write(".iter().any(|x| *x)");
                } else if func == "all" {
                    // all(list) -> list.iter().all(|x| *x)
                    self.emit_expr(&args[0]);
                    self.write(".iter().all(|x| *x)");
                } else if func == "map" {
                    // map(func, list) -> list.clone().into_iter().map(func).collect()
                    self.emit_expr(&args[1]);
                    self.write(".clone().into_iter().map(");
                    self.emit_lambda_untyped(&args[0]);
                    self.write(").collect::<Vec<_>>()");
                } else if func == "filter" {
                    // filter(func, list) -> list.clone().into_iter().filter(func).collect()
                    self.emit_expr(&args[1]);
                    self.write(".clone().into_iter().filter(|x| { let x = *x; (");
                    self.emit_lambda_untyped(&args[0]);
                    self.write(")(x) }).collect::<Vec<_>>()");
                } else {
                    self.write(func);
                    self.write("(");
                    // Look up param types to wrap union enum arguments
                    let param_types: Option<Vec<Type>> = self.func_signatures.get(func).map(|(pts, _)| pts.clone());
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        // Wrap arguments to match parameter types
                        if let Some(ref pts) = param_types {
                            if i < pts.len() {
                                // Option param with non-Option arg -> wrap in Some()
                                if is_option_type(&pts[i]) && !is_option_type(arg.ty()) && !matches!(arg, HirExpr::NoneLiteral) {
                                    self.write("Some(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                // Non-Option union param -> wrap in enum variant
                                if let Type::Union(members) = &pts[i] {
                                    if !is_option_type(&pts[i]) {
                                        let arg_ty = arg.ty();
                                        if let Some(variant) = find_union_variant(members, arg_ty) {
                                            let enum_name = pts[i].union_enum_name();
                                            self.write(&format!("{}::{}(", enum_name, variant));
                                            self.emit_expr(arg);
                                            self.write(")");
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        self.emit_expr(arg);
                    }
                    self.write(")");
                }
            }
            HirExpr::IfExpr {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.write("if ");
                self.emit_expr(condition);
                self.write(" { ");
                self.emit_expr(then_expr);
                self.write(" } else { ");
                self.emit_expr(else_expr);
                self.write(" }");
            }
            HirExpr::RangeLiteral { start, end, .. } => {
                self.emit_expr(start);
                self.write("..");
                self.emit_expr(end);
            }
            HirExpr::ListLiteral { elements, .. } => {
                self.write("vec![");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                }
                self.write("]");
            }
            HirExpr::DictLiteral { keys, values, .. } => {
                self.needs_hashmap = true;
                self.write("HashMap::from([");
                for (i, (key, val)) in keys.iter().zip(values.iter()).enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write("(");
                    self.emit_expr(key);
                    self.write(", ");
                    self.emit_expr(val);
                    self.write(")");
                }
                self.write("])");
            }
            HirExpr::TupleLiteral { elements, .. } => {
                self.write("(");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                }
                if elements.len() == 1 {
                    self.write(","); // Single-element tuple needs trailing comma in Rust
                }
                self.write(")");
            }
            HirExpr::Index { object, index, .. } => {
                let obj_ty = object.ty();
                match obj_ty {
                    Type::Dict(_, _) => {
                        // Safe dict indexing: d[key] -> d.get(&key).cloned()
                        // Returns Option<V> which maps to our V | None union
                        self.emit_expr(object);
                        self.write(".get(");
                        self.emit_key_ref_expr(index);
                        self.write(").cloned()");
                    }
                    Type::Tuple(_) => {
                        // Tuple indexing: t.0, t.1, etc. (handle negative)
                        // Tuples are fixed-size, so indexing is always safe at compile time
                        if let HirExpr::IntLiteral(val) = index.as_ref() {
                            if *val < 0 {
                                if let Type::Tuple(elems) = obj_ty {
                                    let resolved = (elems.len() as i64 + val) as usize;
                                    self.emit_expr(object);
                                    self.write(&format!(".{}", resolved));
                                }
                            } else {
                                self.emit_expr(object);
                                self.write(".");
                                self.emit_expr(index);
                            }
                        } else {
                            self.emit_expr(object);
                            self.write(".");
                            self.emit_expr(index);
                        }
                    }
                    Type::Str => {
                        // Safe string indexing: returns Option<String>
                        // Handle negative indices
                        self.write("{ let _s = &");
                        self.emit_expr(object);
                        self.write("; let _i = ");
                        self.emit_expr(index);
                        self.write("; let _idx = if _i < 0 { (_s.chars().count() as i64 + _i) as usize } else { _i as usize }; _s.chars().nth(_idx).map(|c| c.to_string()) }");
                    }
                    _ => {
                        // Safe list indexing: returns Option<T>
                        // Handle negative indices
                        self.write("{ let _v = &");
                        self.emit_expr(object);
                        self.write("; let _i = ");
                        self.emit_expr(index);
                        self.write("; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }");
                    }
                }
            }
            HirExpr::MethodCall { object, method, args, .. } => {
                self.emit_method_call(object, method, args);
            }
            HirExpr::ContainsOp { element, collection, .. } => {
                let coll_ty = collection.ty();
                match coll_ty {
                    Type::Dict(_, _) => {
                        self.emit_expr(collection);
                        self.write(".contains_key(");
                        self.emit_key_ref_expr(element);
                        self.write(")");
                    }
                    Type::Str => {
                        self.emit_expr(collection);
                        self.write(".contains(");
                        self.emit_str_ref_expr(element);
                        self.write(")");
                    }
                    _ => {
                        // List: collection.contains(&element)
                        self.emit_expr(collection);
                        self.write(".contains(&");
                        self.emit_expr(element);
                        self.write(")");
                    }
                }
            }
            HirExpr::Slice { object, start, stop, step, ty } => {
                let obj_ty = object.ty();
                match obj_ty {
                    Type::Str => {
                        self.emit_string_slice(object, start, stop, step);
                    }
                    Type::Tuple(_) => {
                        // Compile-time tuple slicing: direct field access
                        if let Type::Tuple(result_elems) = ty {
                            let start_idx = start.as_ref().and_then(|e| if let HirExpr::IntLiteral(v) = e.as_ref() { Some(*v as usize) } else { None }).unwrap_or(0);
                            self.write("(");
                            for (i, _) in result_elems.iter().enumerate() {
                                if i > 0 {
                                    self.write(", ");
                                }
                                self.emit_expr(object);
                                self.write(&format!(".{}", start_idx + i));
                            }
                            if result_elems.len() == 1 {
                                self.write(",");
                            }
                            self.write(")");
                        }
                    }
                    _ => {
                        // List slicing
                        self.emit_list_slice(object, start, stop, step);
                    }
                }
            }
            HirExpr::WalrusExpr { name, value: _, .. } => {
                // Walrus operator: the variable is already hoisted by emit_walrus_hoists
                // Just emit the variable name (the assignment was already emitted)
                self.write(name);
            }
            HirExpr::FieldAccess { object, field, ty } => {
                // Determine if we need .clone() (non-Copy field accessed on &self)
                let is_self_access = matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self");
                let needs_clone = is_self_access && needs_clone_for_type(ty);

                // Determine the class name for parent field resolution
                // Either from current_class_name (inside a method) or from the object's type
                let class_name_for_parent = if let Some(ref cn) = self.current_class_name {
                    if is_self_access { Some(cn.clone()) } else { None }
                } else {
                    None
                }.or_else(|| {
                    // For external access like obj.field, check the object's type
                    if let Type::Class { name, .. } = object.ty() {
                        Some(name.clone())
                    } else {
                        None
                    }
                });

                // Check if this is accessing a parent field via inheritance
                if let Some(ref class_name) = class_name_for_parent {
                    if let Some((parent_name, parent_field_names)) = self.parent_fields.get(class_name).cloned() {
                        if parent_field_names.contains(field.as_str()) {
                            // Access via embedded parent: obj.parent.field
                            self.emit_expr(object);
                            self.write(".");
                            self.write(&parent_name.to_lowercase());
                            self.write(".");
                            self.write(field);
                            if needs_clone {
                                self.write(".clone()");
                            }
                            return;
                        }
                    }
                }
                self.emit_expr(object);
                self.write(".");
                self.write(field);
                if needs_clone {
                    self.write(".clone()");
                }
            }
            HirExpr::ConstructorCall { class_name, args, .. } => {
                self.write(class_name);
                self.write("::new(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
            HirExpr::QuestionMark { expr, .. } => {
                self.emit_expr(expr);
                self.write("?");
            }
            HirExpr::OkWrap { value, .. } => {
                self.write("Ok(");
                self.emit_expr(value);
                self.write(")");
            }
            HirExpr::ErrWrap { value, .. } => {
                self.write("Err(");
                self.emit_expr(value);
                self.write(")");
            }
            HirExpr::FString { parts, .. } => {
                self.emit_fstring_macro("format!", parts);
            }
            HirExpr::SuperCall { parent_class, method, args, .. } => {
                // super().__init__(args) -> ParentType::new(args)
                self.write(parent_class);
                self.write("::");
                self.write(method);
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
            HirExpr::Lambda { params, body, .. } => {
                self.write("|");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&param.name);
                    // Only emit type annotation if it's not Any
                    if param.ty != Type::Any {
                        self.write(": ");
                        // For reference types, use &Type
                        if matches!(param.ty, Type::Str | Type::Class { .. }) {
                            self.write("&");
                        }
                        self.write(&param.ty.rust_type());
                    }
                }
                self.write("| ");
                self.emit_expr(body);
            }
            HirExpr::ListComp { expr, var, iter, filter, ty } => {
                // [expr for var in iter] -> iter.iter().map(|&var| expr).collect::<Vec<_>>()
                // [expr for var in iter if cond] -> iter.iter().filter(|&&var| cond).map(|&var| expr).collect::<Vec<_>>()
                // Use .into_iter() for owned values to avoid reference issues
                self.emit_expr(iter);
                if filter.is_some() {
                    // With filter: use .iter().filter().map().cloned().collect()
                    // to avoid ownership issues
                    self.write(".iter()");
                    if let Some(ref cond) = filter {
                        self.write(".filter(|");
                        self.write(var);
                        self.write("| { let ");
                        self.write(var);
                        self.write(" = **");
                        self.write(var);
                        self.write("; ");
                        self.emit_expr(cond);
                        self.write(" })");
                    }
                    self.write(".map(|");
                    self.write(var);
                    self.write("| { let ");
                    self.write(var);
                    self.write(" = *");
                    self.write(var);
                    self.write("; ");
                    self.emit_expr(expr);
                    self.write(" })");
                } else {
                    // Without filter: use .clone().into_iter().map()
                    self.write(".clone().into_iter()");
                    self.write(".map(|");
                    self.write(var);
                    self.write("| ");
                    self.emit_expr(expr);
                    self.write(")");
                }
                // Determine the collect type
                if let Type::List(ref elem) = ty {
                    self.write(&format!(".collect::<Vec<{}>>()", elem.rust_type()));
                } else {
                    self.write(".collect::<Vec<_>>()");
                }
            }
            HirExpr::GeneratorExpr { expr, var, iter, filter, .. } => {
                // (expr for var in iter) -> iter.clone().into_iter().map(|var| expr)
                // Lazy iterator - no .collect()
                self.emit_expr(iter);
                if filter.is_some() {
                    self.write(".iter()");
                    if let Some(ref cond) = filter {
                        self.write(".filter(|");
                        self.write(var);
                        self.write("| { let ");
                        self.write(var);
                        self.write(" = **");
                        self.write(var);
                        self.write("; ");
                        self.emit_expr(cond);
                        self.write(" })");
                    }
                    self.write(".map(|");
                    self.write(var);
                    self.write("| { let ");
                    self.write(var);
                    self.write(" = *");
                    self.write(var);
                    self.write("; ");
                    self.emit_expr(expr);
                    self.write(" })");
                } else {
                    self.write(".clone().into_iter()");
                    self.write(".map(|");
                    self.write(var);
                    self.write("| ");
                    self.emit_expr(expr);
                    self.write(")");
                }
                // No .collect() - lazy iterator
            }
        }
    }

    /// Emit an f-string as a Rust format macro call (format!, println!, etc.).
    /// This avoids the double-format pattern `println!("{}", format!(...))`.
    /// Emit a lambda expression without type annotations on parameters.
    /// Used when the lambda is passed to .map()/.filter() where Rust can infer types.
    fn emit_lambda_untyped(&mut self, expr: &HirExpr) {
        if let HirExpr::Lambda { params, body, .. } = expr {
            self.write("|");
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.write(&param.name);
            }
            self.write("| ");
            self.emit_expr(body);
        } else {
            // Not a lambda, emit as-is
            self.emit_expr(expr);
        }
    }

    fn emit_fstring_macro(&mut self, macro_name: &str, parts: &[HirFStringPart]) {
        let mut format_str = String::new();
        let mut exprs: Vec<&HirExpr> = Vec::new();
        for part in parts {
            match part {
                HirFStringPart::Literal(s) => {
                    // Escape braces in the literal for Rust's format!
                    for ch in s.chars() {
                        match ch {
                            '{' => format_str.push_str("{{"),
                            '}' => format_str.push_str("}}"),
                            _ => format_str.push(ch),
                        }
                    }
                }
                HirFStringPart::Expr(expr) => {
                    format_str.push_str("{}");
                    exprs.push(expr);
                }
            }
        }
        self.write(macro_name);
        self.write("(\"");
        self.write(&format_str);
        self.write("\"");
        for expr in &exprs {
            self.write(", ");
            self.emit_display_expr(expr);
        }
        self.write(")");
    }

    /// Emit an expression as a HashMap key reference.
    /// String literals are emitted directly (e.g., `"key"`) since HashMap::get accepts &str via Borrow.
    /// Other expressions are emitted with `&` prefix (e.g., `&var`).
    fn emit_key_ref_expr(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write(&format!("{:?}", val));
        } else {
            self.write("&");
            self.emit_expr(expr);
        }
    }

    /// Emit an expression as a `&str` reference.
    /// String literals are emitted directly (e.g., `"hello"`).
    /// Other string expressions are emitted with `.as_str()` (e.g., `s.as_str()`).
    fn emit_str_ref_expr(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write(&format!("{:?}", val));
        } else {
            self.emit_expr(expr);
            self.write(".as_str()");
        }
    }

    /// Emit an expression suitable for use inside format!/println! contexts.
    /// Wraps Option<T> expressions so they display as the inner value or "None".
    /// Omits `.to_string()` on string literals since format macros accept &str.
    fn emit_display_expr(&mut self, expr: &HirExpr) {
        if is_option_type(expr.ty()) {
            // Wrap: expr.map_or("None".to_string(), |_v| format!("{}", _v))
            self.write("(");
            self.emit_expr(expr);
            self.write(").map_or(\"None\".to_string(), |_v| format!(\"{}\", _v))");
        } else if let HirExpr::StringLiteral(val) = expr {
            // In display contexts, string literals don't need .to_string()
            self.write(&format!("{:?}", val));
        } else {
            self.emit_expr(expr);
        }
    }
}

/// Check if a type is an Option type (T | None with exactly 2 members).
fn is_option_type(ty: &Type) -> bool {
    if let Type::Union(members) = ty {
        let non_none: Vec<&Type> = members.iter().filter(|m| !matches!(m, Type::None)).collect();
        let has_none = members.iter().any(|m| matches!(m, Type::None));
        has_none && non_none.len() == 1
    } else {
        false
    }
}

/// Detect truthiness check on an Option variable: `if x:` where x has type T | None.
fn detect_option_truthiness(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Name { name, ty } = expr {
        if is_option_type(ty) {
            return Some(name.clone());
        }
    }
    None
}

/// Detect `x is not None` pattern in a Compare expression. Returns the variable name.
fn detect_is_not_none_var(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Compare { left, ops, comparators, .. } = expr {
        if ops.len() == 1 && ops[0] == "is not" && matches!(comparators[0], HirExpr::NoneLiteral) {
            if let HirExpr::Name { name, .. } = left.as_ref() {
                return Some(name.clone());
            }
        }
    }
    None
}

/// Detect `isinstance(x, type)` where x is a non-Option union type.
/// Returns (var_name, variant_name, enum_name, other_variants: Vec<(variant_name, type)>).
fn detect_isinstance_union(expr: &HirExpr) -> Option<(String, String, String, Vec<(String, Type)>)> {
    if let HirExpr::Call { func, args, .. } = expr {
        if func == "isinstance" && args.len() == 2 {
            if let HirExpr::Name { name, ty } = &args[0] {
                if let Type::Union(members) = ty {
                    if !is_option_type(ty) {
                        // The second arg is a StringLiteral with the type name
                        if let HirExpr::StringLiteral(type_name) = &args[1] {
                            let target_ty = match type_name.as_str() {
                                "int" => Type::Int,
                                "str" => Type::Str,
                                "float" => Type::Float,
                                "bool" => Type::Bool,
                                other => {
                                    // Check if it's a class type in the union members
                                    if let Some(class_ty) = members.iter().find(|m| {
                                        matches!(m, Type::Class { name, .. } if name == other)
                                    }) {
                                        class_ty.clone()
                                    } else {
                                        return None;
                                    }
                                }
                            };
                            // Check that this type is a member of the union
                            if members.contains(&target_ty) {
                                let variant = target_ty.union_variant_name();
                                let enum_name = ty.union_enum_name();
                                // Collect other variants for else branch destructuring
                                let other_variants: Vec<(String, Type)> = members.iter()
                                    .filter(|m| *m != &target_ty)
                                    .map(|m| (m.union_variant_name(), m.clone()))
                                    .collect();
                                return Some((name.clone(), variant, enum_name, other_variants));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Find the matching union variant name for an argument type.
fn find_union_variant(members: &[Type], arg_ty: &Type) -> Option<String> {
    for member in members {
        if arg_ty.is_assignable_to(member) {
            return Some(member.union_variant_name());
        }
    }
    None
}

/// Detect `x is None` pattern in a Compare expression. Returns the variable name.
fn detect_is_none_var(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Compare { left, ops, comparators, .. } = expr {
        if ops.len() == 1 && ops[0] == "is" && matches!(comparators[0], HirExpr::NoneLiteral) {
            if let HirExpr::Name { name, .. } = left.as_ref() {
                return Some(name.clone());
            }
        }
    }
    None
}

/// Check if a type is hashable (for codegen derive decisions).
fn is_hashable_type_codegen(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Bool | Type::Str | Type::None => true,
        Type::Float => false,
        _ => false,
    }
}

/// Collect all parts of a chained string concatenation (`a + b + c`).
/// Recursively flattens nested BinOp::Add on strings into a flat list of expressions.
fn collect_string_concat_parts<'a>(expr: &'a HirExpr, parts: &mut Vec<&'a HirExpr>) {
    if let HirExpr::BinOp { left, op, right, ty } = expr {
        if op == "+" && *ty == Type::Str {
            collect_string_concat_parts(left, parts);
            collect_string_concat_parts(right, parts);
            return;
        }
    }
    parts.push(expr);
}

/// Check if a method body contains any field assignments (self.field = ...).
fn body_contains_field_assign_codegen(stmts: &[HirStmt]) -> bool {
    stmts.iter().any(|s| matches!(s, HirStmt::FieldAssign { .. }))
}

/// Check if a function body contains any yield statements (making it a generator).
fn body_contains_yield(stmts: &[HirStmt]) -> bool {
    for stmt in stmts {
        match stmt {
            HirStmt::Yield { .. } => return true,
            HirStmt::If { then_body, elif_clauses, else_body, .. } => {
                if body_contains_yield(then_body) { return true; }
                for (_, body) in elif_clauses {
                    if body_contains_yield(body) { return true; }
                }
                if let Some(eb) = else_body {
                    if body_contains_yield(eb) { return true; }
                }
            }
            HirStmt::While { body, else_body, .. } => {
                if body_contains_yield(body) { return true; }
                if let Some(eb) = else_body {
                    if body_contains_yield(eb) { return true; }
                }
            }
            HirStmt::For { body, else_body, .. } => {
                if body_contains_yield(body) { return true; }
                if let Some(eb) = else_body {
                    if body_contains_yield(eb) { return true; }
                }
            }
            HirStmt::With { body, .. } => {
                if body_contains_yield(body) { return true; }
            }
            _ => {}
        }
    }
    false
}

/// Check if a type needs .clone() when accessed from &self (non-Copy types).
fn needs_clone_for_type(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Float | Type::Bool | Type::None => false,
        Type::LiteralInt(_) | Type::LiteralBool(_) => false,
        Type::Str | Type::LiteralStr(_) => true, // String is not Copy
        Type::List(_) | Type::Dict(_, _) => true,
        Type::Tuple(_) => true, // tuples of non-Copy are non-Copy
        Type::Class { .. } => true,
        Type::Newtype { .. } => true,
        _ => false,
    }
}

/// Mutating methods that require the receiver variable to be `mut`.
const MUTATING_METHODS: &[&str] = &[
    "append", "extend", "insert", "clear", "reverse", "sort", "pop", "remove",
    "push_str", "update",
];

/// Collect the set of variable names that are mutated in a function body.
/// A variable is mutated if it appears in:
/// - `HirStmt::Assign` (reassignment)
/// - `HirStmt::AugAssign` (augmented assignment like +=)
/// - `HirStmt::Expr` containing a `MethodCall` on the variable with a mutating method
/// - `HirStmt::Delete` on the variable
fn collect_mutated_vars(stmts: &[HirStmt]) -> HashSet<String> {
    let mut mutated = HashSet::new();
    collect_mutated_vars_inner(stmts, &mut mutated);
    mutated
}

fn collect_mutated_vars_inner(stmts: &[HirStmt], mutated: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            HirStmt::Assign { name, .. } => {
                mutated.insert(name.clone());
            }
            HirStmt::AugAssign { name, .. } => {
                mutated.insert(name.clone());
            }
            HirStmt::Expr { expr } => {
                collect_mutated_vars_in_expr(expr, mutated);
            }
            HirStmt::Let { value, .. } => {
                // Scan the value expression for mutating method calls
                collect_mutated_vars_in_expr(value, mutated);
            }
            HirStmt::Return { value: Some(expr) } => {
                collect_mutated_vars_in_expr(expr, mutated);
            }
            HirStmt::If { condition, then_body, elif_clauses, else_body } => {
                collect_mutated_vars_in_expr(condition, mutated);
                collect_mutated_vars_inner(then_body, mutated);
                for (cond, body) in elif_clauses {
                    collect_mutated_vars_in_expr(cond, mutated);
                    collect_mutated_vars_inner(body, mutated);
                }
                if let Some(body) = else_body {
                    collect_mutated_vars_inner(body, mutated);
                }
            }
            HirStmt::While { condition, body, else_body } => {
                collect_mutated_vars_in_expr(condition, mutated);
                collect_mutated_vars_inner(body, mutated);
                if let Some(eb) = else_body {
                    collect_mutated_vars_inner(eb, mutated);
                }
            }
            HirStmt::For { body, else_body, .. } => {
                collect_mutated_vars_inner(body, mutated);
                if let Some(eb) = else_body {
                    collect_mutated_vars_inner(eb, mutated);
                }
            }
            HirStmt::TryExcept { body, handlers } => {
                collect_mutated_vars_inner(body, mutated);
                for handler in handlers {
                    collect_mutated_vars_inner(&handler.body, mutated);
                }
            }
            HirStmt::Delete { object, .. } => {
                if let HirExpr::Name { name, .. } = object {
                    mutated.insert(name.clone());
                }
            }
            HirStmt::Yield { value } => {
                collect_mutated_vars_in_expr(value, mutated);
            }
            HirStmt::With { body, value, .. } => {
                collect_mutated_vars_in_expr(value, mutated);
                collect_mutated_vars_inner(body, mutated);
            }
            _ => {}
        }
    }
}

fn collect_mutated_vars_in_expr(expr: &HirExpr, mutated: &mut HashSet<String>) {
    match expr {
        HirExpr::MethodCall { object, method, args, .. } => {
            if MUTATING_METHODS.contains(&method.as_str()) {
                if let HirExpr::Name { name, .. } = object.as_ref() {
                    mutated.insert(name.clone());
                }
            }
            // Recurse into sub-expressions
            collect_mutated_vars_in_expr(object, mutated);
            for arg in args {
                collect_mutated_vars_in_expr(arg, mutated);
            }
        }
        HirExpr::Call { args, .. } => {
            for arg in args {
                collect_mutated_vars_in_expr(arg, mutated);
            }
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_mutated_vars_in_expr(left, mutated);
            collect_mutated_vars_in_expr(right, mutated);
        }
        HirExpr::UnaryOp { operand, .. } => {
            collect_mutated_vars_in_expr(operand, mutated);
        }
        HirExpr::Compare { left, comparators, .. } => {
            collect_mutated_vars_in_expr(left, mutated);
            for c in comparators {
                collect_mutated_vars_in_expr(c, mutated);
            }
        }
        HirExpr::BoolOp { values, .. } => {
            for v in values {
                collect_mutated_vars_in_expr(v, mutated);
            }
        }
        HirExpr::IfExpr { condition, then_expr, else_expr, .. } => {
            collect_mutated_vars_in_expr(condition, mutated);
            collect_mutated_vars_in_expr(then_expr, mutated);
            collect_mutated_vars_in_expr(else_expr, mutated);
        }
        HirExpr::Index { object, index, .. } => {
            collect_mutated_vars_in_expr(object, mutated);
            collect_mutated_vars_in_expr(index, mutated);
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let HirFStringPart::Expr(e) = part {
                    collect_mutated_vars_in_expr(e, mutated);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_hir::*;
    use sifr_type_system::Type;

    #[test]
    fn test_simple_function_codegen() {
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::StringLiteral("Hello, World!".to_string())],
                        ty: Type::None,
                    },
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("fn main()"));
        assert!(rust_code.contains("println!"));
        assert!(rust_code.contains("Hello, World!"));
    }

    #[test]
    fn test_arithmetic_codegen() {
        let module = HirModule {
            functions: vec![HirFunction {
                name: "add".to_string(),
                params: vec![
                    HirParam { name: "a".to_string(), ty: Type::Int, default: None, keyword_only: false },
                    HirParam { name: "b".to_string(), ty: Type::Int, default: None, keyword_only: false },
                ],
                return_type: Type::Int,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::BinOp {
                        left: Box::new(HirExpr::Name { name: "a".to_string(), ty: Type::Int }),
                        op: "+".to_string(),
                        right: Box::new(HirExpr::Name { name: "b".to_string(), ty: Type::Int }),
                        ty: Type::Int,
                    }),
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("fn add(a: i64, b: i64) -> i64"));
        assert!(rust_code.contains("return a + b;"));
    }

    // --- Codegen Quality Tests ---

    #[test]
    fn test_no_unnecessary_mut() {
        // Variable that is never reassigned should NOT have `mut`
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Let {
                        name: "x".to_string(),
                        ty: Type::Int,
                        value: HirExpr::IntLiteral(42),
                        is_mutable: true, // HIR says mutable, but codegen should ignore
                    },
                    HirStmt::Expr {
                        expr: HirExpr::Call {
                            func: "print".to_string(),
                            args: vec![HirExpr::Name { name: "x".to_string(), ty: Type::Int }],
                            ty: Type::None,
                        },
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("let x: i64"), "should emit `let x` without mut");
        assert!(!rust_code.contains("let mut x"), "should NOT emit `let mut x`");
    }

    #[test]
    fn test_mut_on_reassigned_variable() {
        // Variable that IS reassigned should have `mut`
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Let {
                        name: "x".to_string(),
                        ty: Type::Int,
                        value: HirExpr::IntLiteral(0),
                        is_mutable: true,
                    },
                    HirStmt::Assign {
                        name: "x".to_string(),
                        value: HirExpr::IntLiteral(1),
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("let mut x: i64"), "should emit `let mut x` for reassigned var");
    }

    #[test]
    fn test_println_fstring_inlined() {
        // print(f"hello {name}") should emit println!("hello {}", name) not println!("{}", format!(...))
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Let {
                        name: "name".to_string(),
                        ty: Type::Str,
                        value: HirExpr::StringLiteral("World".to_string()),
                        is_mutable: false,
                    },
                    HirStmt::Expr {
                        expr: HirExpr::Call {
                            func: "print".to_string(),
                            args: vec![HirExpr::FString {
                                parts: vec![
                                    HirFStringPart::Literal("Hello, ".to_string()),
                                    HirFStringPart::Expr(HirExpr::Name { name: "name".to_string(), ty: Type::Str }),
                                    HirFStringPart::Literal("!".to_string()),
                                ],
                                ty: Type::Str,
                            }],
                            ty: Type::None,
                        },
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("println!(\"Hello, {}!\", name)"), "should inline f-string into println!");
        assert!(!rust_code.contains("format!(\"Hello, {}!\""), "should NOT have standalone format! inside println!");
    }

    #[test]
    fn test_no_tostring_in_println() {
        // print("hello") should emit println!("{}", "hello") not println!("{}", "hello".to_string())
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::StringLiteral("hello".to_string())],
                        ty: Type::None,
                    },
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("println!(\"{}\", \"hello\")"), "should emit string literal without .to_string()");
        assert!(!rust_code.contains("\"hello\".to_string()"), "should NOT have .to_string() in println context");
    }

    #[test]
    fn test_hashmap_short_name() {
        // Dict literal should use HashMap::from not std::collections::HashMap::from
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Let {
                    name: "d".to_string(),
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                    value: HirExpr::DictLiteral {
                        keys: vec![HirExpr::StringLiteral("a".to_string())],
                        values: vec![HirExpr::IntLiteral(1)],
                        ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                    },
                    is_mutable: false,
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("use std::collections::HashMap;"), "should have HashMap import");
        assert!(rust_code.contains("HashMap::from("), "should use short HashMap::from");
        assert!(!rust_code.contains("std::collections::HashMap::from("), "should NOT use fully qualified HashMap::from");
        assert!(rust_code.contains("HashMap<String, i64>"), "type annotation should use short HashMap");
    }

    #[test]
    fn test_dict_get_string_literal_key() {
        // d["key"] should emit d.get("key") not d.get(&"key".to_string())
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Let {
                        name: "d".to_string(),
                        ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                        value: HirExpr::DictLiteral {
                            keys: vec![HirExpr::StringLiteral("key".to_string())],
                            values: vec![HirExpr::IntLiteral(1)],
                            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                        },
                        is_mutable: false,
                    },
                    HirStmt::Let {
                        name: "v".to_string(),
                        ty: Type::Union(vec![Type::Int, Type::None]),
                        value: HirExpr::Index {
                            object: Box::new(HirExpr::Name {
                                name: "d".to_string(),
                                ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                            }),
                            index: Box::new(HirExpr::StringLiteral("key".to_string())),
                            ty: Type::Union(vec![Type::Int, Type::None]),
                        },
                        is_mutable: false,
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains(".get(\"key\")"), "should emit .get(\"key\") for string literal key");
        assert!(!rust_code.contains("&\"key\".to_string()"), "should NOT have &\"key\".to_string()");
    }

    #[test]
    fn test_string_concat_flattened() {
        // "a" + "b" + "c" should emit format!("{}{}{}", ...) not nested format!
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Let {
                    name: "s".to_string(),
                    ty: Type::Str,
                    value: HirExpr::BinOp {
                        left: Box::new(HirExpr::BinOp {
                            left: Box::new(HirExpr::StringLiteral("a".to_string())),
                            op: "+".to_string(),
                            right: Box::new(HirExpr::StringLiteral("b".to_string())),
                            ty: Type::Str,
                        }),
                        op: "+".to_string(),
                        right: Box::new(HirExpr::StringLiteral("c".to_string())),
                        ty: Type::Str,
                    },
                    is_mutable: false,
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("format!(\"{}{}{}\","), "should flatten into single format! with 3 placeholders");
        assert_eq!(rust_code.matches("format!").count(), 1, "should have exactly one format! call");
    }

    #[test]
    fn test_mut_on_mutating_method_call() {
        // Variable with .push() call should have `mut`
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Let {
                        name: "items".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                        value: HirExpr::ListLiteral {
                            elements: vec![HirExpr::IntLiteral(1)],
                            ty: Type::List(Box::new(Type::Int)),
                        },
                        is_mutable: true,
                    },
                    HirStmt::Expr {
                        expr: HirExpr::MethodCall {
                            object: Box::new(HirExpr::Name {
                                name: "items".to_string(),
                                ty: Type::List(Box::new(Type::Int)),
                            }),
                            method: "append".to_string(),
                            args: vec![HirExpr::IntLiteral(2)],
                            ty: Type::None,
                        },
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("let mut items"), "should emit `let mut items` for variable with .push()");
    }

    #[test]
    fn test_empty_print() {
        // print() should emit println!() not println!("{}", "")
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![],
                        ty: Type::None,
                    },
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
            }],
            classes: vec![],
            imports: vec![],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("println!()"), "should emit println!() for empty print");
        assert!(!rust_code.contains(r#"println!("{}", "")"#), "should NOT emit println with empty string arg");
    }
}
