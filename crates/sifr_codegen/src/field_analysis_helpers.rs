use crate::helpers::{type_references_any_class, type_references_class};
use crate::RustEmitter;
use sifr_ir::{HirExpr, HirModule};
use std::collections::{HashMap, HashSet};

impl RustEmitter {
    pub(crate) fn recursive_target_rust_type_for_field(
        &self,
        ty: &sifr_type_system::Type,
    ) -> crate::RustType {
        match ty {
            sifr_type_system::Type::Alias { body, .. } => {
                self.recursive_target_rust_type_for_field(body)
            }
            _ => self.rust_ir_type_with_generics(ty),
        }
    }

    pub(crate) fn recursive_field_storage_rust_type(
        &self,
        ty: &sifr_type_system::Type,
        same_scc_classes: &HashSet<String>,
    ) -> crate::RustType {
        match ty {
            sifr_type_system::Type::Union(_) => {
                if let Some(member) = ty.optional_member_type() {
                    if type_references_any_class(&member, same_scc_classes) {
                        crate::RustType::Option(Box::new(crate::RustType::Boxed(Box::new(
                            self.recursive_target_rust_type_for_field(&member),
                        ))))
                    } else {
                        self.rust_ir_type_with_generics(ty)
                    }
                } else {
                    crate::RustType::Boxed(Box::new(self.rust_ir_type_with_generics(ty)))
                }
            }
            sifr_type_system::Type::Class { .. } => {
                crate::RustType::Boxed(Box::new(self.recursive_target_rust_type_for_field(ty)))
            }
            _ => crate::RustType::Boxed(Box::new(self.rust_ir_type_with_generics(ty))),
        }
    }

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

    /// Detect recursive class fields that need Box<T> wrapping.
    /// A field is recursive if it references a class in the same recursive SCC.
    pub(crate) fn detect_recursive_fields(&mut self, module: &HirModule) {
        let class_names: HashSet<String> = module
            .classes
            .iter()
            .map(|class| class.name.clone())
            .collect();
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

        for class in &module.classes {
            let field_names: Vec<String> = class.fields.iter().map(|(n, _)| n.clone()).collect();
            self.class_field_order
                .insert(class.name.clone(), field_names);
            let mut deps = HashSet::new();
            for (field_name, field_ty) in &class.fields {
                self.class_field_types
                    .insert((class.name.clone(), field_name.clone()), field_ty.clone());
                for target in &class_names {
                    if type_references_class(field_ty, target) {
                        deps.insert(target.clone());
                    }
                }
                if type_references_class(field_ty, &class.name) {
                    self.recursive_fields
                        .insert((class.name.clone(), field_name.clone()));
                }
            }
            graph.insert(class.name.clone(), deps);
            if !class.type_params.is_empty() {
                self.generic_classes.insert(class.name.clone());
                self.generic_class_params
                    .insert(class.name.clone(), class.type_params.clone());
                self.generic_class_templates
                    .insert(class.name.clone(), class.clone());
            }
        }

        let mut reachability: HashMap<String, HashSet<String>> = HashMap::new();
        for class_name in &class_names {
            let mut seen = HashSet::new();
            let mut stack = vec![class_name.clone()];
            while let Some(current) = stack.pop() {
                let Some(neighbors) = graph.get(&current) else {
                    continue;
                };
                for neighbor in neighbors {
                    if seen.insert(neighbor.clone()) {
                        stack.push(neighbor.clone());
                    }
                }
            }
            reachability.insert(class_name.clone(), seen);
        }

        for class in &module.classes {
            let same_scc_classes: HashSet<String> = class_names
                .iter()
                .filter(|candidate| {
                    reachability
                        .get(&class.name)
                        .is_some_and(|reachable| reachable.contains(*candidate))
                        && reachability
                            .get(*candidate)
                            .is_some_and(|reachable| reachable.contains(&class.name))
                })
                .cloned()
                .collect();
            if same_scc_classes.is_empty() {
                continue;
            }

            for (field_name, field_ty) in &class.fields {
                if type_references_any_class(field_ty, &same_scc_classes) {
                    let key = (class.name.clone(), field_name.clone());
                    self.recursive_fields.insert(key.clone());
                    self.recursive_field_rust_types.insert(
                        key,
                        self.recursive_field_storage_rust_type(field_ty, &same_scc_classes),
                    );
                }
            }
        }
    }

    pub(crate) fn register_external_class_fields(
        &mut self,
        local_class_name: &str,
        source_class_name: &str,
        fields: &[(String, sifr_type_system::Type)],
    ) {
        self.class_field_order.insert(
            local_class_name.to_string(),
            fields.iter().map(|(name, _)| name.clone()).collect(),
        );

        // Imported recursive metadata currently models self-recursive classes.
        // Mutually recursive imported classes should graduate to the full SCC
        // analysis used by `detect_recursive_fields`.
        let same_class_names =
            HashSet::from([local_class_name.to_string(), source_class_name.to_string()]);
        for (field_name, field_ty) in fields {
            self.class_field_types.insert(
                (local_class_name.to_string(), field_name.clone()),
                field_ty.clone(),
            );
            if type_references_any_class(field_ty, &same_class_names) {
                let key = (local_class_name.to_string(), field_name.clone());
                self.recursive_fields.insert(key.clone());
                self.recursive_field_rust_types.insert(
                    key,
                    self.recursive_field_storage_rust_type(field_ty, &same_class_names),
                );
            }
        }
    }
}
