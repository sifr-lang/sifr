use crate::{RustEmitter, RustExpr, RustItem, RustStmt, RustTypeParam};
use sifr_ir::HirModule;
use sifr_type_system::{Type, class_rust_name};
use std::collections::{BTreeMap, HashMap};

/// Conversion authority follows semantic ancestry, never an error's basename.
#[derive(Clone, Default)]
pub(crate) struct ErrorConversionDemand {
    errors: BTreeMap<String, NominalError>,
    storage: BTreeMap<String, MessageStorage>,
}

#[derive(Clone)]
struct MessageStorage {
    owns_message: bool,
    parent: Option<(String, String)>,
}

#[derive(Clone)]
struct NominalError {
    rust_name: String,
    type_arguments: String,
    type_params: Vec<RustTypeParam>,
    declaration: bool,
}

impl ErrorConversionDemand {
    pub(super) fn record_classes(&mut self, module: &HirModule, module_name: Option<&str>) {
        for class in &module.classes {
            let identity = class.identity.clone().unwrap_or_else(|| {
                module_name.map_or_else(
                    || class.name.clone(),
                    |module| format!("{module}.{}", class.name),
                )
            });
            let parent = class.parent_type.as_ref().and_then(|ty| {
                let Type::Class { identity, name, .. } = ty.resolve_alias() else {
                    return None;
                };
                Some((
                    identity.clone().unwrap_or_else(|| name.clone()),
                    class.parent_class.as_ref()?.to_lowercase(),
                ))
            });
            self.storage.insert(
                identity.clone(),
                MessageStorage {
                    owns_message: class.fields.iter().any(|(name, ty)| {
                        name == "message" && matches!(ty.resolve_alias(), Type::Str)
                    }),
                    parent,
                },
            );
            let Some(chain) = class
                .semantic_parent_chain()
                .filter(|_| class.is_error_type)
            else {
                continue;
            };
            if !chain.split('|').any(is_root_error) {
                continue;
            }
            let rust_name = sifr_type_system::source_class_rust_name(&class.name);
            let target = RustEmitter::class_impl_target(class);
            self.errors.insert(
                identity,
                NominalError {
                    type_arguments: target[rust_name.len()..].to_string(),
                    rust_name,
                    type_params: RustEmitter::class_impl_type_params(class),
                    declaration: true,
                },
            );
        }
    }

    pub(super) fn record_type(&mut self, ty: &Type) {
        let Type::Class {
            identity,
            name,
            parent_class: Some(chain),
            ..
        } = ty.resolve_alias()
        else {
            return;
        };
        if !chain.split('|').any(is_root_error)
            || (crate::BUILTIN_ERROR_CLASSES.contains(&name.as_str())
                && identity.as_deref().is_none_or(|id| {
                    id.starts_with("sifr.builtin.")
                        || sifr_type_system::is_global_rust_nominal_identity(id)
                }))
        {
            return;
        }
        let rust_name = class_rust_name(identity.as_deref(), name);
        let rendered = crate::render_type(&crate::sifr_type_to_rust_type(ty));
        self.errors
            .entry(identity.clone().unwrap_or_else(|| name.clone()))
            .or_insert_with(|| NominalError {
                type_arguments: rendered[rust_name.len()..].to_string(),
                rust_name,
                type_params: Vec::new(),
                declaration: false,
            });
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        self.storage.extend(other.storage.clone());
        for (identity, error) in &other.errors {
            if error.declaration || !self.errors.contains_key(identity) {
                self.errors.insert(identity.clone(), error.clone());
            }
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub(crate) fn render(&self, paths: &HashMap<String, String>) -> Vec<RustItem> {
        self.errors
            .iter()
            .map(|(identity, error)| {
                let source = format!(
                    "{}{}",
                    paths.get(identity).unwrap_or(&error.rust_name),
                    error.type_arguments
                );
                let mut item = crate::build_error_into_error_impl(&source);
                if let RustItem::Impl {
                    type_params, items, ..
                } = &mut item
                {
                    type_params.clone_from(&error.type_params);
                    // Project through owned embedded fields, never through Deref.
                    // Data parents can themselves be non-errors: semantic ancestry
                    // is not authority for where the message is physically stored.
                    let mut value = RustExpr::Ident("err".to_string());
                    let mut owner = identity.as_str();
                    let mut visited = std::collections::HashSet::new();
                    while let Some(storage) = self.storage.get(owner) {
                        if storage.owns_message {
                            break;
                        }
                        assert!(
                            visited.insert(owner),
                            "checked class storage must be acyclic"
                        );
                        let Some((parent, field)) = &storage.parent else {
                            unreachable!(
                                "checked error must have owned or inherited string storage"
                            );
                        };
                        value = RustExpr::Field {
                            expr: Box::new(value),
                            field: field.clone(),
                        };
                        owner = parent;
                    }
                    value = RustExpr::Field {
                        expr: Box::new(value),
                        field: "message".to_string(),
                    };
                    if let RustItem::Fn { body, .. } = &mut items[0] {
                        *body = vec![RustStmt::Return(Some(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "Self".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![value],
                        }))];
                    }
                }
                item
            })
            .collect()
    }
}

fn is_root_error(identity: &str) -> bool {
    matches!(identity, "Error" | "sifr.builtin.Error")
}
