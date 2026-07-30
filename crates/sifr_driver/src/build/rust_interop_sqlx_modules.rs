use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::Token;
use syn::{Attribute, Expr, Item, ItemMod, Lit, Meta};

pub(super) fn reachable_rust_modules(backend_root: &Path) -> Vec<syn::File> {
    let Some(entry_path) = crate_entry_path(backend_root) else {
        return Vec::new();
    };
    let Some(canonical_root) = fs::canonicalize(backend_root).ok() else {
        return Vec::new();
    };
    let entry_module_dir = entry_path
        .parent()
        .map_or_else(|| backend_root.to_path_buf(), Path::to_path_buf);
    let mut pending = vec![PendingModule {
        source_path: entry_path,
        module_dir: entry_module_dir,
    }];
    let mut visited = BTreeSet::new();
    let mut modules = Vec::new();
    while let Some(module) = pending.pop() {
        let Some(canonical_source) = regular_source_path(&module.source_path, &canonical_root)
        else {
            continue;
        };
        if !visited.insert(canonical_source.clone()) {
            continue;
        }
        let Ok(source) = fs::read_to_string(&canonical_source) else {
            continue;
        };
        let Ok(syntax) = syn::parse_file(&source) else {
            continue;
        };
        if has_conditional_compilation_attribute(&syntax.attrs) {
            continue;
        }
        collect_declared_modules(&syntax.items, &module.module_dir, &mut pending);
        modules.push(syntax);
    }
    modules
}

struct PendingModule {
    source_path: PathBuf,
    module_dir: PathBuf,
}

fn crate_entry_path(backend_root: &Path) -> Option<PathBuf> {
    let manifest = fs::read_to_string(backend_root.join("Cargo.toml")).ok();
    if let Some(path) = manifest
        .and_then(|source| source.parse::<toml::Table>().ok())
        .and_then(|table| {
            table
                .get("lib")
                .and_then(toml::Value::as_table)
                .and_then(|lib| lib.get("path"))
                .and_then(toml::Value::as_str)
                .map(PathBuf::from)
        })
    {
        return Some(backend_root.join(path));
    }
    let lib = backend_root.join("src/lib.rs");
    if is_regular_file(&lib) {
        return Some(lib);
    }
    let main = backend_root.join("src/main.rs");
    is_regular_file(&main).then_some(main)
}

fn regular_source_path(path: &Path, canonical_root: &Path) -> Option<PathBuf> {
    if !is_regular_file(path) {
        return None;
    }
    let canonical = fs::canonicalize(path).ok()?;
    canonical.starts_with(canonical_root).then_some(canonical)
}

fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

fn collect_declared_modules(items: &[Item], module_dir: &Path, pending: &mut Vec<PendingModule>) {
    for item in items {
        let Item::Mod(module) = item else {
            continue;
        };
        if module_declaration_may_vary(&module.attrs) {
            continue;
        }
        if let Some((_, nested_items)) = &module.content {
            let nested_module_dir = declared_path(module).map_or_else(
                || module_dir.join(module.ident.to_string()),
                |path| module_dir.join(path),
            );
            collect_declared_modules(nested_items, &nested_module_dir, pending);
            continue;
        }
        if let Some(module) = resolve_declared_module(module, module_dir) {
            pending.push(module);
        }
    }
}

fn resolve_declared_module(module: &ItemMod, module_dir: &Path) -> Option<PendingModule> {
    if let Some(path) = declared_path(module) {
        let source_path = module_dir.join(path);
        let child_dir = module_dir_for_explicit_path(&source_path);
        return Some(PendingModule {
            source_path,
            module_dir: child_dir,
        });
    }
    let module_name = module.ident.to_string();
    let flat = module_dir.join(format!("{module_name}.rs"));
    let nested = module_dir.join(&module_name).join("mod.rs");
    let source_path = match (is_regular_file(&flat), is_regular_file(&nested)) {
        (true, false) => flat,
        (false, true) => nested,
        _ => return None,
    };
    Some(PendingModule {
        source_path,
        module_dir: module_dir.join(module_name),
    })
}

fn declared_path(module: &ItemMod) -> Option<PathBuf> {
    module.attrs.iter().find_map(|attribute| {
        if !attribute.path().is_ident("path") {
            return None;
        }
        let Meta::NameValue(value) = &attribute.meta else {
            return None;
        };
        let Expr::Lit(value) = &value.value else {
            return None;
        };
        let Lit::Str(path) = &value.lit else {
            return None;
        };
        Some(PathBuf::from(path.value()))
    })
}

fn module_dir_for_explicit_path(source_path: &Path) -> PathBuf {
    let parent = source_path.parent().unwrap_or_else(|| Path::new(""));
    if source_path.file_name().is_some_and(|name| name == "mod.rs") {
        return parent.to_path_buf();
    }
    let stem = source_path
        .file_stem()
        .map_or_else(PathBuf::new, PathBuf::from);
    parent.join(stem)
}

pub(super) fn has_conditional_compilation_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().any(attribute_may_disable)
}

fn module_declaration_may_vary(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
}

fn attribute_may_disable(attribute: &Attribute) -> bool {
    if attribute.path().is_ident("cfg") {
        return true;
    }
    if !attribute.path().is_ident("cfg_attr") {
        return false;
    }
    let Meta::List(list) = &attribute.meta else {
        return true;
    };
    cfg_attr_arguments_may_disable(list)
}

fn cfg_attr_arguments_may_disable(list: &syn::MetaList) -> bool {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let Ok(arguments) = parser.parse2(list.tokens.clone()) else {
        return true;
    };
    arguments.iter().skip(1).any(meta_may_disable)
}

fn meta_may_disable(meta: &Meta) -> bool {
    match meta {
        Meta::List(list) if list.path.is_ident("cfg") => true,
        Meta::List(list) if list.path.is_ident("cfg_attr") => cfg_attr_arguments_may_disable(list),
        _ => false,
    }
}
