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
    let entry_dir_path = entry_path
        .parent()
        .map_or_else(|| backend_root.to_path_buf(), Path::to_path_buf);
    let mut pending = vec![PendingModule {
        source_path: entry_path,
        directory: ModuleDirectory {
            dir_path: entry_dir_path,
            relative: None,
        },
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
        collect_declared_modules(&syntax.items, &module.directory, &mut pending);
        modules.push(syntax);
    }
    modules
}

struct PendingModule {
    source_path: PathBuf,
    directory: ModuleDirectory,
}

struct ModuleDirectory {
    dir_path: PathBuf,
    relative: Option<PathBuf>,
}

impl ModuleDirectory {
    fn plain_base(&self) -> PathBuf {
        self.relative.as_ref().map_or_else(
            || self.dir_path.clone(),
            |relative| self.dir_path.join(relative),
        )
    }
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

fn collect_declared_modules(
    items: &[Item],
    directory: &ModuleDirectory,
    pending: &mut Vec<PendingModule>,
) {
    for item in items {
        let Item::Mod(module) = item else {
            continue;
        };
        if module_declaration_may_vary(&module.attrs) {
            continue;
        }
        if let Some((_, nested_items)) = &module.content {
            let dir_path = declared_path(module).map_or_else(
                || directory.plain_base().join(module.ident.to_string()),
                |path| directory.dir_path.join(path),
            );
            collect_declared_modules(
                nested_items,
                &ModuleDirectory {
                    dir_path,
                    relative: None,
                },
                pending,
            );
            continue;
        }
        if let Some(module) = resolve_declared_module(module, directory) {
            pending.push(module);
        }
    }
}

fn resolve_declared_module(module: &ItemMod, directory: &ModuleDirectory) -> Option<PendingModule> {
    if let Some(path) = declared_path(module) {
        let source_path = directory.dir_path.join(path);
        let dir_path = source_path
            .parent()
            .map_or_else(PathBuf::new, Path::to_path_buf);
        return Some(PendingModule {
            source_path,
            directory: ModuleDirectory {
                dir_path,
                relative: None,
            },
        });
    }
    let module_name = module.ident.to_string();
    let base = directory.plain_base();
    let flat = base.join(format!("{module_name}.rs"));
    let nested = base.join(&module_name).join("mod.rs");
    let (source_path, directory) = match (is_regular_file(&flat), is_regular_file(&nested)) {
        (true, false) => (
            flat,
            ModuleDirectory {
                dir_path: base,
                relative: Some(PathBuf::from(module_name)),
            },
        ),
        (false, true) => (
            nested,
            ModuleDirectory {
                dir_path: base.join(module_name),
                relative: None,
            },
        ),
        _ => return None,
    };
    Some(PendingModule {
        source_path,
        directory,
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
