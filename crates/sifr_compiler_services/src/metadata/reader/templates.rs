//! Template roles select the exact canonical HIR inventory, never a source reconstruction.
use super::{Result, wire};
pub(super) fn validate(store: &wire::MetadataStore, module: &wire::Module) -> Result<()> {
    let hir = store.get(module.hir_inventory)?;
    let mut functions = std::collections::BTreeSet::new();
    let mut classes = std::collections::BTreeSet::new();
    for reference in &module.templates {
        let template = store.get(*reference)?;
        let owner = store.get(template.owner)?;
        let role = store.get(template.role)?;
        let valid = match role.as_ref() {
            wire::TemplateRole::GenericFunction => match (template.function, template.class) {
                (Some(function), None) if hir.functions.contains(&function) => {
                    let function = store.get(function)?;
                    functions.insert(function.name)
                        && function.name == owner.symbol
                        && !function.type_params.is_empty()
                }
                _ => false,
            },
            wire::TemplateRole::GenericClass | wire::TemplateRole::ProjectPolicyClass => {
                match (template.function, template.class) {
                    (None, Some(class)) if hir.classes.contains(&class) => {
                        let class = store.get(class)?;
                        classes.insert(class.name)
                            && class.name == owner.symbol
                            && class.type_params.is_empty()
                                == matches!(role.as_ref(), wire::TemplateRole::ProjectPolicyClass)
                    }
                    _ => false,
                }
            }
        };
        if !valid || store.get(owner.module)?.name != module.name {
            return Err(wire::MetadataError(
                "template role/owner differs from demanded HIR inventory".into(),
            ));
        }
    }
    let expected_functions = hir
        .functions
        .iter()
        .map(|reference| store.get(*reference))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|function| !function.type_params.is_empty())
        .map(|function| function.name)
        .collect();
    let expected_classes = hir
        .classes
        .iter()
        .map(|reference| store.get(*reference))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|class| class.name)
        .collect();
    if functions != expected_functions || classes != expected_classes {
        return Err(wire::MetadataError(
            "demanded template inventory is incomplete".into(),
        ));
    }
    Ok(())
}
