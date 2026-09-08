use crate::{RustEmitter, RustExpr, RustType};
use sifr_ir::{HirClass, HirModule};
use sifr_type_system::{Type, source_class_rust_name};

const CANONICAL_PYTHON_ERROR_IDENTITY: &str = "_sifr.python.PythonError";
impl RustEmitter {
    pub(crate) fn class_uses_python_error_bridge(class: &HirClass, module: &HirModule) -> bool {
        if class.identity.as_deref() == Some(CANONICAL_PYTHON_ERROR_IDENTITY) {
            return true;
        }
        if !crate::python_interop_common::module_uses_python_declaration(module) {
            return false;
        }
        Type::Class {
            identity: class.identity.clone(),
            type_args: class
                .type_params
                .iter()
                .cloned()
                .map(Type::TypeVar)
                .collect(),
            name: class.name.clone(),
            fields: class.fields.clone(),
            methods: Vec::new(),
            parent_class: class.semantic_parent_chain(),
        }
        .is_python_error_contract()
    }

    pub(crate) fn class_struct_fields(
        &mut self,
        class: &HirClass,
        module_public: bool,
        uses_python_error_bridge: bool,
    ) -> Vec<(String, RustType)> {
        if class.python_opaque_declaration().is_some() {
            let mut fields = vec![
                (
                    "__sifr_python_object".to_string(),
                    RustType::Named("::sifr_runtime::python::ObjectHandle".to_string()),
                ),
                (
                    "__sifr_python_callbacks".to_string(),
                    RustType::Named("::sifr_runtime::python::CallbackOwnerSlot".to_string()),
                ),
                (
                    "__sifr_python_not_send_sync".to_string(),
                    RustType::Named("std::marker::PhantomData<std::rc::Rc<()>>".to_string()),
                ),
            ];
            if let Some(errors) = self.python_retained_callback_errors.get(&class.name) {
                fields.extend(errors.iter().enumerate().map(|(index, error)| {
                    (
                        format!("__sifr_python_callback_failure_{index}"),
                        RustType::Generic {
                            base: "::sifr_runtime::python::CallbackFailureSlot".to_string(),
                            params: vec![self.rust_ir_type_with_generics(error)],
                        },
                    )
                }));
            }
            return fields;
        }
        let mut fields = Vec::new();
        if let Some(parent) = &class.parent_class {
            if parent != "NonSend" {
                let field_name = if module_public {
                    format!("pub {}", parent.to_lowercase())
                } else {
                    parent.to_lowercase()
                };
                let parent_rust_type = class.parent_type.as_ref().map_or_else(
                    || RustType::Named(source_class_rust_name(parent)),
                    crate::sifr_type_to_rust_type,
                );
                fields.push((field_name, parent_rust_type));
            }
        }

        for (field_name, field_ty) in &class.fields {
            let name = if module_public {
                format!("pub {field_name}")
            } else {
                field_name.clone()
            };
            let ty = self.class_struct_field_rust_type(class, field_name, field_ty);
            fields.push((name, ty));
        }
        if Self::class_needs_phantom_marker(class) {
            fields.push((
                "__sifr_type_marker".to_string(),
                RustType::Named(format!(
                    "std::marker::PhantomData<fn() -> {}>",
                    Self::class_phantom_tuple(class)
                )),
            ));
        }
        if uses_python_error_bridge {
            let name = if module_public {
                "pub __sifr_python_error".to_string()
            } else {
                "__sifr_python_error".to_string()
            };
            fields.push((
                name,
                RustType::Option(Box::new(RustType::Named(
                    "::sifr_runtime::python::PythonError".to_string(),
                ))),
            ));
        }
        fields
    }

    pub(crate) fn class_struct_field_rust_type(
        &mut self,
        class: &HirClass,
        field_name: &str,
        field_ty: &Type,
    ) -> RustType {
        if self
            .recursive_fields
            .contains(&(class.name.clone(), field_name.to_string()))
        {
            return self
                .recursive_field_rust_types
                .get(&(class.name.clone(), field_name.to_string()))
                .cloned()
                .unwrap_or_else(|| self.rust_ir_type_with_generics(field_ty));
        }
        if class.name == "deque" && field_name == "_data" {
            self.collection_needs.needs_vecdeque = true;
            if let Type::List(elem) = field_ty {
                return RustType::VecDeque(Box::new(self.rust_ir_type_with_generics(elem)));
            }
        }
        self.rust_ir_struct_field_type_with_generics(field_ty)
    }

    fn class_phantom_tuple(class: &HirClass) -> String {
        if class.type_params.len() == 1 {
            format!("({},)", class.type_params[0])
        } else {
            format!("({})", class.type_params.join(", "))
        }
    }

    pub(crate) fn append_class_phantom_initializer(
        class: &HirClass,
        fields: &mut Vec<(String, RustExpr)>,
    ) {
        if Self::class_needs_phantom_marker(class) {
            fields.push((
                "__sifr_type_marker".to_string(),
                RustExpr::Path(vec![
                    "std".to_string(),
                    "marker".to_string(),
                    "PhantomData".to_string(),
                ]),
            ));
        }
    }
}
