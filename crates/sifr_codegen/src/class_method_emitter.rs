use crate::{
    helpers::{
        body_contains_field_assign_codegen,
        collect_mutated_vars_with_sigs,
        recursive_field_rust_type,
    },
    RustEmitter,
};
use sifr_hir::{HirClass, HirExpr, HirFunction, HirStmt, MethodKind};
use sifr_type_system::{ParamConvention, Type};

impl RustEmitter {
    pub(super) fn emit_class_method(&mut self, method: &HirFunction, class: &HirClass, module_public: bool) {
        self.current_return_type = Some(method.return_type.clone());

        // Pre-scan: collect mutated variables so we know which need `mut`
        self.mutated_vars = collect_mutated_vars_with_sigs(&method.body, &self.func_signatures);

        self.write_indent();
        let pub_prefix = if module_public { "pub " } else { "" };

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
                    self.write(&format!("{pub_prefix}fn new("));
                    for (i, param) in method.params.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.write(&param.name);
                        self.write(": ");
                        // Check if this parameter corresponds to a recursive field
                        let is_recursive = self.recursive_fields.contains(&(class.name.clone(), param.name.clone()));
                        if is_recursive {
                            self.write(&recursive_field_rust_type(&param.ty, &class.name));
                        } else if matches!(&param.ty, Type::Callable(..)) {
                            // Callable params in constructors need 'static for Box::new()
                            self.write(&format!("{} + 'static", param.ty.rust_type()));
                        } else {
                            self.write(&param.ty.rust_type());
                        }
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

                    let inheritance_parent = if has_super { class.parent_class.as_ref() } else { None };
                    if let Some(parent_name) = inheritance_parent {
                        // Inheritance constructor: emit super call, then Self { parent: ..., own fields }
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

                        // Emit own field inits (recursive fields already have correct Box type from params)
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
                            // deque._data = [] → VecDeque::new() in constructor
                            if class.name == "deque" && *field_name == "_data" {
                                if let HirExpr::ListLiteral { elements, .. } = value {
                                    if elements.is_empty() {
                                        self.write("VecDeque::new()");
                                        self.write(",\n");
                                        continue;
                                    }
                                }
                            }
                            // Wrap Callable values in Box::new() for struct fields
                            let field_ty = class.fields.iter().find(|(n, _)| n == field_name).map(|(_, t)| t);
                            let needs_box = field_ty.is_some_and(|t| matches!(t, Type::Callable(..)));
                            if needs_box {
                                self.write("Box::new(");
                                self.emit_expr(value);
                                self.write(")");
                            } else {
                                self.emit_expr(value);
                            }
                            self.write(",\n");
                        }
                        // For any fields not explicitly assigned, check if param name matches
                        for (field_name, field_ty) in &class.fields {
                            if !field_inits.iter().any(|(f, _)| f == field_name) {
                                if method.params.iter().any(|p| &p.name == field_name) {
                                    self.write_indent();
                                    // Wrap Callable params in Box::new() for struct fields
                                    if matches!(field_ty, Type::Callable(..)) {
                                        self.write(&format!("{field_name}: Box::new({field_name})"));
                                    } else {
                                        self.write(field_name);
                                    }
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
                        self.write(&format!("{pub_prefix}fn "));
                        self.write(&method.name);
                        self.write("(&mut self");
                    } else {
                        self.write(&format!("{pub_prefix}fn "));
                        self.write(&method.name);
                        self.write("(&self");
                    }
                    for param in &method.params {
                        self.write(", ");
                        self.write(&param.name);
                        self.write(": ");
                        let rust_ty = self.rust_type_with_generics(&param.ty);
                        match param.convention {
                            ParamConvention::Borrow => {
                                if param.ty.ownership() == sifr_type_system::OwnershipKind::Copy {
                                    self.write(&rust_ty);
                                } else {
                                    self.write(&format!("&{rust_ty}"));
                                }
                            }
                            ParamConvention::MutBorrow => {
                                self.write(&format!("&mut {rust_ty}"));
                            }
                            ParamConvention::Own => {
                                self.write(&rust_ty);
                            }
                        }
                    }
                    self.write(")");

                    if method.return_type != Type::None {
                        self.write(" -> ");
                        // If return type is the same generic class, include type params
                        let ret_rust_type = if let Type::Class { name: ret_name, .. } = &method.return_type {
                            if !class.type_params.is_empty() && ret_name == &class.name {
                                format!("{}<{}>", ret_name, class.type_params.join(", "))
                            } else {
                                method.return_type.rust_type()
                            }
                        } else {
                            method.return_type.rust_type()
                        };
                        self.write(&ret_rust_type);
                    }

                    self.write(" {\n");
                    self.indent += 1;

                    // Track borrowed/mut-borrowed params for correct key-ref and borrow-prefix logic
                    self.borrowed_params.clear();
                    self.mut_borrowed_params.clear();
                    for param in &method.params {
                        if param.convention == ParamConvention::Borrow
                            && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        {
                            self.borrowed_params.insert(param.name.clone());
                        }
                        if param.convention == ParamConvention::MutBorrow
                            && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        {
                            self.mut_borrowed_params.insert(param.name.clone());
                        }
                    }

                    for stmt in &method.body {
                        self.emit_stmt(stmt);
                    }

                    self.borrowed_params.clear();
                    self.mut_borrowed_params.clear();

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                }
            }
        }

        self.current_return_type = None;
        self.mutated_vars.clear();
    }
}
