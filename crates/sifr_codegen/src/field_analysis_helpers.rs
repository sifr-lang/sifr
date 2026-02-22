use crate::helpers::type_references_class;
use crate::RustEmitter;
use sifr_hir::{HirExpr, HirModule};

impl RustEmitter {
    /// Check if the object expression is `self._data` inside the `deque` class.
    pub(crate) fn is_deque_data_field(&self, object: &HirExpr) -> bool {
        if self.current_class_name.as_deref() != Some("deque") {
            return false;
        }
        if let HirExpr::FieldAccess {
            object: inner,
            field,
            ..
        } = object
        {
            if field == "_data" {
                if let HirExpr::Name { name, .. } = inner.as_ref() {
                    return name == "self";
                }
            }
        }
        false
    }

    /// Detect self-referential class fields that need Box<T> wrapping.
    /// A field is recursive if its type directly or indirectly references the class being defined.
    pub(crate) fn detect_recursive_fields(&mut self, module: &HirModule) {
        for class in &module.classes {
            let field_names: Vec<String> = class.fields.iter().map(|(n, _)| n.clone()).collect();
            self.class_field_order
                .insert(class.name.clone(), field_names);
            for (field_name, field_ty) in &class.fields {
                if type_references_class(field_ty, &class.name) {
                    self.recursive_fields
                        .insert((class.name.clone(), field_name.clone()));
                }
            }
            if !class.type_params.is_empty() {
                self.generic_classes.insert(class.name.clone());
                self.generic_class_params
                    .insert(class.name.clone(), class.type_params.clone());
            }
        }
    }
}
