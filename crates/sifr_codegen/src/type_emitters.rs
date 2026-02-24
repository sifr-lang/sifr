use crate::{is_hashable_type_codegen, RustEmitter};
use sifr_hir::HirClass;
use sifr_type_system::Type;

impl RustEmitter {
    pub(super) fn emit_protocol_trait(&mut self, class: &HirClass, module_public: bool) {
        self.write_indent();
        if module_public {
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
    pub(super) fn emit_enum_class(&mut self, class: &HirClass, module_public: bool) {
        // #[repr(i64)]
        // #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        // enum Color { RED = 1, GREEN = 2, BLUE = 3 }
        self.write_indent();
        self.write("#[repr(i64)]\n");
        self.write_indent();
        self.write("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
        self.write_indent();
        if module_public {
            self.write("pub enum ");
        } else {
            self.write("enum ");
        }
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;
        let mut auto_value = 1i64;
        for (variant_name, value) in &class.enum_variants {
            self.write_indent();
            self.write(variant_name);
            let v = value.unwrap_or(auto_value);
            self.write(&format!(" = {v}"));
            self.write(",\n");
            auto_value = v + 1;
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");

        // impl Display for Color { fn fmt(...) { write!(f, "{:?}", self) } }
        self.write_indent();
        self.write("impl std::fmt::Display for ");
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;
        self.write_indent();
        self.write("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
        self.indent += 1;
        self.write_indent();
        self.write("write!(f, \"{:?}\", self)\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");

        // impl Color { fn name(&self) -> String { format!("{:?}", self) } fn value(&self) -> i64 { *self as i64 } }
        self.write_indent();
        self.write("impl ");
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;
        self.write_indent();
        self.write("fn name(&self) -> String { format!(\"{:?}\", self) }\n");
        self.write_indent();
        self.write("fn value(&self) -> i64 { *self as i64 }\n");
        // Emit user-defined methods
        let class_name = class.name.clone();
        let methods = class.methods.clone();
        for method in &methods {
            self.current_class_name = Some(class_name.clone());
            self.emit_class_method(method, class, module_public);
        }
        self.current_class_name = None;
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");
    }

    pub(super) fn emit_newtype(&mut self, class: &HirClass, inner: &Type, module_public: bool) {
        // Derive attributes
        self.write_indent();
        if is_hashable_type_codegen(inner) {
            self.write("#[derive(Debug, Clone, PartialEq, Eq, Hash)]\n");
        } else {
            self.write("#[derive(Debug, Clone, PartialEq)]\n");
        }

        self.write_indent();
        if module_public {
            self.write(&format!(
                "pub struct {}({});\n\n",
                class.name,
                inner.rust_type()
            ));
        } else {
            self.write(&format!(
                "struct {}({});\n\n",
                class.name,
                inner.rust_type()
            ));
        }

        // Impl block with constructor and value() accessor
        self.write_indent();
        self.write("impl ");
        self.write(&class.name);
        self.write(" {\n");
        self.indent += 1;

        // Constructor: fn new(value: InnerType) -> Self
        self.write_indent();
        let pub_prefix = if module_public { "pub " } else { "" };
        self.write(&format!(
            "{}fn new(value: {}) -> Self {{\n",
            pub_prefix,
            inner.rust_type()
        ));
        self.indent += 1;
        self.write_indent();
        self.write("Self(value)\n");
        self.indent -= 1;
        self.write_indent();
        self.write("}\n\n");

        // Accessor: fn value(&self) -> InnerType
        self.write_indent();
        self.write(&format!(
            "{}fn value(&self) -> {} {{\n",
            pub_prefix,
            inner.rust_type()
        ));
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
            self.emit_class_method(method, class, module_public);
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
}
