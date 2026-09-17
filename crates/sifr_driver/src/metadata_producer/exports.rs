use super::{Encode, Encoder, Result, wire};
use std::collections::{BTreeMap, BTreeSet};

fn origin(
    module: &str,
    name: &str,
    compiled: &crate::stdlib::StdlibCompiled,
    cx: &Encoder,
    seen: &mut BTreeSet<(String, String)>,
) -> Result<(String, String)> {
    if !seen.insert((module.to_owned(), name.to_owned())) {
        return Err(wire::MetadataError(format!(
            "cyclic exported declaration origin: {module}.{name}"
        )));
    }
    let scope = cx.scopes.get(module).ok_or_else(|| {
        wire::MetadataError(format!(
            "export references a module outside the canonical inventory: {module}"
        ))
    })?;
    let hir = compiled.code.hir_modules.get(module).ok_or_else(|| {
        wire::MetadataError(format!("missing checked module for export: {module}"))
    })?;
    if scope.locations.contains_key(name)
        || hir.functions.iter().any(|f| f.name == name)
        || hir.classes.iter().any(|c| c.name == name)
        || hir.constants.iter().any(|(n, _, _)| n == name)
    {
        return Ok((module.to_owned(), name.to_owned()));
    }
    for import in &hir.imports {
        for imported in &import.names {
            let local = import
                .aliases
                .iter()
                .find(|(original, _)| original == imported)
                .map(|(_, alias)| alias)
                .unwrap_or(imported);
            if local == name && cx.scopes.contains_key(&import.module) {
                return origin(&import.module, imported, compiled, cx, seen);
            }
        }
    }
    Err(wire::MetadataError(format!(
        "cannot bind exported declaration {module}.{name} to its checked source"
    )))
}
pub(super) fn project(
    compiled: &crate::stdlib::StdlibCompiled,
    module: &str,
    declarations: &mut BTreeMap<wire::Ref<wire::Text>, wire::Ref<wire::Declaration>>,
    cx: &mut Encoder,
) -> Result<BTreeMap<wire::Ref<wire::Text>, wire::Ref<wire::Declaration>>> {
    let defs = &compiled.defs;
    let mut kinds = BTreeMap::new();
    macro_rules! functions {($($field:ident),*)=>{$(if let Some(values)=defs.$field.get(module){for name in values.keys(){kinds.insert(name.clone(),wire::DeclarationKind::Function);}})*};}
    functions!(
        functions,
        class_adapter_providers,
        class_adapter_markers,
        attached_api_sets,
        attached_apis,
        descriptor_functions
    );
    if let Some(values) = defs.constants.get(module) {
        for name in values.keys() {
            kinds.insert(name.clone(), wire::DeclarationKind::Constant);
        }
    }
    if let Some(values) = defs.classes.get(module) {
        for (name, ty) in values {
            kinds.insert(name.clone(), super::type_kind(ty));
        }
    }
    if let Some(values) = defs.generic_type_aliases.get(module) {
        for name in values.keys() {
            kinds.insert(name.clone(), wire::DeclarationKind::Alias);
        }
    }
    let mut exports = BTreeMap::new();
    for (name, kind) in kinds {
        let (owner, symbol) = origin(module, &name, compiled, cx, &mut BTreeSet::new())?;
        let declaration = cx.declaration(&format!("{owner}.{symbol}"), kind)?;
        let name = name.encode(cx)?;
        exports.insert(name, declaration);
        if owner == module {
            declarations.insert(name, declaration);
        }
    }
    Ok(exports)
}

pub(super) fn bind_reference(
    compiled: &crate::stdlib::StdlibCompiled,
    module: &str,
    name: &str,
    kind: wire::DeclarationKind,
    cx: &mut Encoder,
) -> Result<wire::Ref<wire::Declaration>> {
    let (owner, symbol) = origin(module, name, compiled, cx, &mut BTreeSet::new())?;
    cx.declaration(&format!("{owner}.{symbol}"), kind)
}
