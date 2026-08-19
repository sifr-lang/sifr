//! Concrete generic bindings from an adapted class to a checked handler owner.

use sifr_lowering::{substitute_type_vars, ExternalDefs, HirClass, LoweringResult};
use sifr_type_system::Type;
use std::collections::{BTreeSet, HashMap};

pub(crate) enum HandlerAncestry {
    Owner(HashMap<String, Type>),
    ImportedBoundary {
        module: String,
        name: String,
        bindings: HashMap<String, Type>,
    },
}

pub(crate) fn resolve(
    module_name: &str,
    root: &HirClass,
    root_type_args: &[Type],
    owner_identity: &str,
    lowering: &LoweringResult,
    external_defs: &ExternalDefs,
) -> Option<HandlerAncestry> {
    let bindings = bind_exact(&root.type_params, root_type_args.iter().cloned())?;
    walk_local(
        module_name,
        root,
        bindings,
        owner_identity,
        lowering,
        external_defs,
        &mut BTreeSet::new(),
    )
}

#[allow(clippy::too_many_arguments)]
fn walk_local(
    module_name: &str,
    current: &HirClass,
    bindings: HashMap<String, Type>,
    owner_identity: &str,
    lowering: &LoweringResult,
    external_defs: &ExternalDefs,
    visiting: &mut BTreeSet<String>,
) -> Option<HandlerAncestry> {
    let current_identity = current
        .identity
        .clone()
        .unwrap_or_else(|| format!("{module_name}.{}", current.name));
    if current_identity == owner_identity {
        return Some(HandlerAncestry::Owner(bindings));
    }
    if !visiting.insert(current_identity) {
        return None;
    }
    let Type::Class {
        identity,
        name,
        type_args,
        ..
    } = current.parent_type.as_ref()?.resolve_alias()
    else {
        return None;
    };
    let parent_identity = identity
        .clone()
        .unwrap_or_else(|| format!("{module_name}.{name}"));
    let parent_args = type_args
        .iter()
        .map(|argument| substitute_type_vars(argument, &bindings))
        .collect::<Vec<_>>();
    let local_parent = lowering.module.classes.iter().find(|candidate| {
        candidate.identity.as_deref() == Some(parent_identity.as_str())
            || (candidate.name == *name
                && parent_identity == format!("{module_name}.{}", candidate.name))
    });
    if let Some(parent) = local_parent {
        let parent_bindings = bind_exact(&parent.type_params, parent_args)?;
        return walk_local(
            module_name,
            parent,
            parent_bindings,
            owner_identity,
            lowering,
            external_defs,
            visiting,
        );
    }
    let (source_module, source_name) = parent_identity.rsplit_once('.')?;
    let type_params = external_defs
        .class_type_params
        .get(source_module)
        .and_then(|classes| classes.get(source_name))?;
    let parent_bindings = bind_exact(type_params, parent_args)?;
    if parent_identity == owner_identity {
        Some(HandlerAncestry::Owner(parent_bindings))
    } else {
        Some(HandlerAncestry::ImportedBoundary {
            module: source_module.to_string(),
            name: source_name.to_string(),
            bindings: parent_bindings,
        })
    }
}

fn bind_exact(
    type_params: &[String],
    type_args: impl IntoIterator<Item = Type>,
) -> Option<HashMap<String, Type>> {
    let type_args = type_args.into_iter().collect::<Vec<_>>();
    (type_params.len() == type_args.len())
        .then(|| type_params.iter().cloned().zip(type_args).collect())
}

#[cfg(test)]
mod tests {
    use super::{resolve, HandlerAncestry};
    use sifr_lowering::{lower_module, ExternalDefs};
    use sifr_syntax::parse_module_suite;
    use sifr_type_system::Type;

    fn generic_chain() -> sifr_lowering::LoweringResult {
        let parsed = parse_module_suite(
            "class Base[T]:\n    value: T\n\nclass Child[U](Base[U]):\n    pass\n",
            None,
        )
        .expect("fixture parses");
        lower_module(&parsed).expect("fixture lowers")
    }

    #[test]
    fn unspecialized_generic_ancestor_keeps_its_symbolic_argument() {
        let lowered = generic_chain();
        let child = &lowered.module.classes[1];
        let ancestry = resolve(
            "fixture",
            child,
            &[Type::TypeVar("U".to_string())],
            "fixture.Base",
            &lowered,
            &ExternalDefs::default(),
        );

        let Some(HandlerAncestry::Owner(bindings)) = ancestry else {
            panic!("local base must resolve")
        };
        assert_eq!(bindings.get("T"), Some(&Type::TypeVar("U".to_string())));
    }

    #[test]
    fn unspecialized_generic_ancestor_rejects_missing_root_arguments() {
        let lowered = generic_chain();
        let child = &lowered.module.classes[1];

        assert!(resolve(
            "fixture",
            child,
            &[],
            "fixture.Base",
            &lowered,
            &ExternalDefs::default(),
        )
        .is_none());
    }
}
