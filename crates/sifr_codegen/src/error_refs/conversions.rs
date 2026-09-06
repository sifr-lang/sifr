use crate::{RustEmitter, RustExpr, RustItem, RustStmt, RustTypeParam};
use sifr_ir::HirModule;
use sifr_type_system::{Type, class_rust_name};
use std::collections::{BTreeMap, HashMap};

/// Conversion authority follows semantic ancestry, never an error's basename.
#[derive(Clone, Default)]
pub(crate) struct ErrorConversionDemand(BTreeMap<String, NominalError>);

#[derive(Clone)]
struct NominalError {
    rust_name: String,
    type_arguments: String,
    type_params: Vec<RustTypeParam>,
    ancestors: Vec<String>,
    declaration: bool,
    owns_message: bool,
}

impl ErrorConversionDemand {
    pub(super) fn record_classes(&mut self, module: &HirModule, module_name: Option<&str>) {
        for class in &module.classes {
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
            self.0.insert(
                class.identity.clone().unwrap_or_else(|| {
                    module_name.map_or_else(
                        || class.name.clone(),
                        |module| format!("{module}.{}", class.name),
                    )
                }),
                NominalError {
                    type_arguments: target[rust_name.len()..].to_string(),
                    rust_name,
                    type_params: RustEmitter::class_impl_type_params(class),
                    ancestors: chain.split('|').map(str::to_string).collect(),
                    declaration: true,
                    owns_message: class.fields.iter().any(|(name, _)| name == "message"),
                },
            );
        }
    }

    pub(super) fn record_type(&mut self, ty: &Type) {
        let Type::Class {
            identity,
            name,
            fields,
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
        self.0
            .entry(identity.clone().unwrap_or_else(|| name.clone()))
            .or_insert_with(|| NominalError {
                type_arguments: rendered[rust_name.len()..].to_string(),
                rust_name,
                type_params: Vec::new(),
                ancestors: chain.split('|').map(str::to_string).collect(),
                declaration: false,
                owns_message: fields.iter().any(|(name, _)| name == "message"),
            });
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        for (identity, error) in &other.0 {
            if error.declaration || !self.0.contains_key(identity) {
                self.0.insert(identity.clone(), error.clone());
            }
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn render(&self, paths: &HashMap<String, String>) -> Vec<RustItem> {
        self.0
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
                    *type_params = error.type_params.clone();
                    // A transitive error owns its parent value. Consume that value via
                    // the existing inheritance impl, rather than moving through Deref
                    // or cloning the inherited message.
                    if !error.owns_message
                        && error
                            .ancestors
                            .first()
                            .is_some_and(|ancestor| !is_root_error(ancestor))
                    {
                        let mut value = RustExpr::Ident("err".to_string());
                        for ancestor in &error.ancestors {
                            let target = if is_root_error(ancestor) {
                                "Error".to_string()
                            } else if let Some(path) = paths.get(ancestor) {
                                path.clone()
                            } else {
                                let name = ancestor.rsplit('.').next().unwrap_or(ancestor);
                                class_rust_name(
                                    ancestor.contains('.').then_some(ancestor.as_str()),
                                    name,
                                )
                            };
                            value = RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![format!(
                                    "::std::convert::Into::<{target}>::into"
                                )])),
                                args: vec![value],
                            };
                            if is_root_error(ancestor) {
                                break;
                            }
                        }
                        if let RustItem::Fn { body, .. } = &mut items[0] {
                            *body = vec![RustStmt::Return(Some(value))];
                        }
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
