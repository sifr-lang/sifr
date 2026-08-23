use super::rust_interop_digest::relative_path_string;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use syn::visit::{self, Visit};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AsyncRuntimeBridgeViolation {
    pub(super) file: String,
    pub(super) construct: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AsyncRuntimeBindings {
    runtime_types: BTreeSet<String>,
    runtime_modules: BTreeSet<String>,
    task_modules: BTreeSet<String>,
    tokio_roots: BTreeSet<String>,
    blocking_functions: BTreeSet<String>,
}

impl AsyncRuntimeBindings {
    fn merge_from(&mut self, source: &Self) -> bool {
        let before = self.clone();
        self.runtime_types
            .extend(source.runtime_types.iter().cloned());
        self.runtime_modules
            .extend(source.runtime_modules.iter().cloned());
        self.task_modules
            .extend(source.task_modules.iter().cloned());
        self.tokio_roots.extend(source.tokio_roots.iter().cloned());
        self.blocking_functions
            .extend(source.blocking_functions.iter().cloned());
        *self != before
    }

    fn copy_symbol_from(&mut self, source: &Self, symbol: &str, local_name: &str) -> bool {
        let before = self.clone();
        if source.runtime_types.contains(symbol) {
            self.runtime_types.insert(local_name.to_string());
        }
        if source.runtime_modules.contains(symbol) {
            self.runtime_modules.insert(local_name.to_string());
        }
        if source.task_modules.contains(symbol) {
            self.task_modules.insert(local_name.to_string());
        }
        if source.tokio_roots.contains(symbol) {
            self.tokio_roots.insert(local_name.to_string());
        }
        if source.blocking_functions.contains(symbol) {
            self.blocking_functions.insert(local_name.to_string());
        }
        *self != before
    }
}

pub(super) fn unsafe_bridge_files(package: &sifr_package::SifrPackageMetadata) -> Vec<String> {
    let mut files = Vec::new();
    for bridge_root in &package.manifest.rust.bridges {
        let root = package.package_root.join(bridge_root);
        collect_unsafe_bridge_files(&package.package_root, &root, &mut files);
    }
    files.sort();
    files.dedup();
    files
}

pub(super) fn async_runtime_bridge_violations(
    package: &sifr_package::SifrPackageMetadata,
) -> Vec<AsyncRuntimeBridgeViolation> {
    let mut violations = Vec::new();
    let mut roots = BTreeSet::from([package.package_root.join("src")]);
    roots.extend(
        package
            .manifest
            .rust
            .bridges
            .iter()
            .map(|root| package.package_root.join(root)),
    );
    for root in roots {
        collect_async_runtime_bridge_violations(&package.package_root, &root, &mut violations);
    }
    violations
        .sort_by(|left, right| (&left.file, left.construct).cmp(&(&right.file, right.construct)));
    violations.dedup();
    violations
}

fn collect_unsafe_bridge_files(package_root: &Path, path: &Path, files: &mut Vec<String>) {
    if path.is_file() {
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            return;
        }
        let Ok(source) = fs::read_to_string(path) else {
            return;
        };
        if source.contains("unsafe") {
            files.push(relative_path_string(package_root, path));
        }
        return;
    }
    let Ok(read_dir) = fs::read_dir(path) else {
        return;
    };
    for entry in read_dir.flatten() {
        collect_unsafe_bridge_files(package_root, &entry.path(), files);
    }
}

fn collect_async_runtime_bridge_violations(
    package_root: &Path,
    path: &Path,
    violations: &mut Vec<AsyncRuntimeBridgeViolation>,
) {
    if path.is_file() {
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            return;
        }
        let relative_file = relative_path_string(package_root, path);
        let Ok(source) = fs::read_to_string(path) else {
            violations.push(AsyncRuntimeBridgeViolation {
                file: relative_file,
                construct: "unauditable bridge source",
            });
            return;
        };
        let Ok(audit) = audit_source(&source, rust_module_path(package_root, path)) else {
            violations.push(AsyncRuntimeBridgeViolation {
                file: relative_file,
                construct: "unauditable bridge source",
            });
            return;
        };
        for construct in audit.violations {
            violations.push(AsyncRuntimeBridgeViolation {
                file: relative_file.clone(),
                construct,
            });
        }
        return;
    }
    let Ok(read_dir) = fs::read_dir(path) else {
        return;
    };
    for entry in read_dir.flatten() {
        collect_async_runtime_bridge_violations(package_root, &entry.path(), violations);
    }
}

struct AsyncRuntimeAudit {
    local_functions: BTreeMap<Vec<String>, BTreeSet<String>>,
    bindings: BTreeMap<Vec<String>, AsyncRuntimeBindings>,
    module_path: Vec<String>,
    violations: BTreeSet<&'static str>,
}

#[cfg(test)]
fn audit_file(file: &syn::File) -> AsyncRuntimeAudit {
    audit_file_with_module_path(file, Vec::new())
}

fn audit_file_with_module_path(file: &syn::File, module_path: Vec<String>) -> AsyncRuntimeAudit {
    let mut audit = AsyncRuntimeAudit {
        local_functions: collect_local_functions(&file.items, &module_path),
        bindings: collect_import_bindings(file, &module_path),
        module_path,
        violations: BTreeSet::new(),
    };
    audit.visit_file(file);
    audit
}

fn audit_source(source: &str, module_path: Vec<String>) -> Result<AsyncRuntimeAudit, syn::Error> {
    syn::parse_file(source).map(|file| audit_file_with_module_path(&file, module_path))
}

fn collect_local_functions(
    items: &[syn::Item],
    initial_module_path: &[String],
) -> BTreeMap<Vec<String>, BTreeSet<String>> {
    fn collect(
        items: &[syn::Item],
        module_path: &mut Vec<String>,
        functions: &mut BTreeMap<Vec<String>, BTreeSet<String>>,
    ) {
        for item in items {
            match item {
                syn::Item::Fn(function) => {
                    functions
                        .entry(module_path.clone())
                        .or_default()
                        .insert(function.sig.ident.to_string());
                }
                syn::Item::Mod(module) => {
                    let Some((_, items)) = &module.content else {
                        continue;
                    };
                    module_path.push(module.ident.to_string());
                    collect(items, module_path, functions);
                    module_path.pop();
                }
                _ => {}
            }
        }
    }

    let mut functions = BTreeMap::new();
    collect(items, &mut initial_module_path.to_vec(), &mut functions);
    functions
}

fn collect_import_bindings(
    file: &syn::File,
    initial_module_path: &[String],
) -> BTreeMap<Vec<String>, AsyncRuntimeBindings> {
    struct Collector {
        module_path: Vec<String>,
        bindings: BTreeMap<Vec<String>, AsyncRuntimeBindings>,
        globs: Vec<(Vec<String>, Vec<String>)>,
        named_imports: Vec<(Vec<String>, Vec<String>, String)>,
    }

    struct TypeAliasCollector {
        module_path: Vec<String>,
        aliases: Vec<(Vec<String>, String, Vec<String>)>,
    }

    impl<'ast> Visit<'ast> for Collector {
        fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
            self.module_path.push(module.ident.to_string());
            if module.content.is_none() {
                self.module_path.pop();
                return;
            }
            self.bindings.entry(self.module_path.clone()).or_default();
            visit::visit_item_mod(self, module);
            self.module_path.pop();
        }

        fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
            collect_use_aliases(
                &item.tree,
                &mut Vec::new(),
                self.bindings.entry(self.module_path.clone()).or_default(),
            );
            let mut prefixes = Vec::new();
            collect_glob_prefixes(&item.tree, &mut Vec::new(), &mut prefixes);
            self.globs.extend(
                prefixes
                    .into_iter()
                    .filter(|prefix| !is_known_tokio_glob(prefix))
                    .map(|prefix| (self.module_path.clone(), prefix)),
            );
            let mut named = Vec::new();
            collect_named_imports(&item.tree, &mut Vec::new(), &mut named);
            self.named_imports.extend(
                named
                    .into_iter()
                    .map(|(source, local_name)| (self.module_path.clone(), source, local_name)),
            );
        }
    }

    impl<'ast> Visit<'ast> for TypeAliasCollector {
        fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
            if module.content.is_none() {
                return;
            }
            self.module_path.push(module.ident.to_string());
            visit::visit_item_mod(self, module);
            self.module_path.pop();
        }

        fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
            if let syn::Type::Path(path) = item.ty.as_ref() {
                self.aliases.push((
                    self.module_path.clone(),
                    item.ident.to_string(),
                    path_segments(&path.path),
                ));
            }
        }
    }

    let mut collector = Collector {
        module_path: initial_module_path.to_vec(),
        bindings: BTreeMap::from([(
            initial_module_path.to_vec(),
            AsyncRuntimeBindings::default(),
        )]),
        globs: Vec::new(),
        named_imports: Vec::new(),
    };
    collector.visit_file(file);
    let mut type_aliases = TypeAliasCollector {
        module_path: initial_module_path.to_vec(),
        aliases: Vec::new(),
    };
    type_aliases.visit_file(file);
    for (module_path, _, _) in &type_aliases.aliases {
        collector.bindings.entry(module_path.clone()).or_default();
    }
    loop {
        let mut changed = false;
        let additions = type_aliases
            .aliases
            .iter()
            .filter_map(|(module_path, alias, target)| {
                let bindings = collector.bindings.get(module_path)?;
                (is_runtime_type_path(target, bindings) && !bindings.runtime_types.contains(alias))
                    .then(|| (module_path.clone(), alias.clone()))
            })
            .collect::<Vec<_>>();
        changed |= !additions.is_empty();
        for (module_path, alias) in additions {
            collector
                .bindings
                .entry(module_path)
                .or_default()
                .runtime_types
                .insert(alias);
        }
        for (destination, prefix) in &collector.globs {
            let Some(source_module) = resolve_import_module(destination, prefix) else {
                continue;
            };
            let Some(source) = collector.bindings.get(&source_module).cloned() else {
                continue;
            };
            changed |= collector
                .bindings
                .entry(destination.clone())
                .or_default()
                .merge_from(&source);
        }
        for (destination, source_path, local_name) in &collector.named_imports {
            let Some((symbol, module_prefix)) = source_path.split_last() else {
                continue;
            };
            let Some(source_module) = resolve_import_module(destination, module_prefix) else {
                continue;
            };
            let Some(source) = collector.bindings.get(&source_module).cloned() else {
                continue;
            };
            changed |= collector
                .bindings
                .entry(destination.clone())
                .or_default()
                .copy_symbol_from(&source, symbol, local_name);
        }
        if !changed {
            break;
        }
    }
    collector.bindings
}

fn collect_glob_prefixes(
    tree: &syn::UseTree,
    prefix: &mut Vec<String>,
    globs: &mut Vec<Vec<String>>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_glob_prefixes(&path.tree, prefix, globs);
            prefix.pop();
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_glob_prefixes(item, prefix, globs);
            }
        }
        syn::UseTree::Glob(_) => globs.push(prefix.clone()),
        syn::UseTree::Name(_) | syn::UseTree::Rename(_) => {}
    }
}

fn collect_named_imports(
    tree: &syn::UseTree,
    prefix: &mut Vec<String>,
    imports: &mut Vec<(Vec<String>, String)>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_named_imports(&path.tree, prefix, imports);
            prefix.pop();
        }
        syn::UseTree::Name(name) => {
            let name = name.ident.to_string();
            if name == "self" {
                if let Some(local_name) = prefix.last().cloned() {
                    imports.push((prefix.clone(), local_name));
                }
            } else {
                let mut source = prefix.clone();
                source.push(name.clone());
                imports.push((source, name));
            }
        }
        syn::UseTree::Rename(rename) => {
            let mut source = prefix.clone();
            if rename.ident != "self" {
                source.push(rename.ident.to_string());
            }
            imports.push((source, rename.rename.to_string()));
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_named_imports(item, prefix, imports);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

fn is_known_tokio_glob(prefix: &[String]) -> bool {
    matches!(prefix, [tokio] if tokio == "tokio")
        || matches!(
            prefix,
            [tokio, module]
                if tokio == "tokio" && matches!(module.as_str(), "runtime" | "task")
        )
}

fn resolve_import_module(destination: &[String], prefix: &[String]) -> Option<Vec<String>> {
    let mut source = destination.to_vec();
    let mut remaining = prefix;
    match remaining.first().map(String::as_str) {
        Some("crate") => {
            source.clear();
            remaining = &remaining[1..];
        }
        Some("self") => remaining = &remaining[1..],
        Some("super") => {
            while remaining.first().is_some_and(|part| part == "super") {
                source.pop()?;
                remaining = &remaining[1..];
            }
        }
        Some(_) => {}
        None => return None,
    }
    source.extend(remaining.iter().cloned());
    Some(source)
}

fn rust_module_path(package_root: &Path, path: &Path) -> Vec<String> {
    let Ok(relative) = path.strip_prefix(package_root) else {
        return Vec::new();
    };
    let mut components = relative
        .parent()
        .into_iter()
        .flat_map(Path::components)
        .filter_map(|component| component.as_os_str().to_str())
        .filter(|component| *component != "src")
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return components;
    };
    if !matches!(stem, "lib" | "main" | "mod") {
        components.push(stem.to_string());
    }
    components
}

impl<'ast> Visit<'ast> for AsyncRuntimeAudit {
    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        if module.content.is_none() {
            return;
        }
        self.module_path.push(module.ident.to_string());
        visit::visit_item_mod(self, module);
        self.module_path.pop();
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(function) = call.func.as_ref() {
            let segments = expression_path_segments(function);
            let bindings = self
                .bindings
                .get(&self.module_path)
                .cloned()
                .unwrap_or_default();
            if is_runtime_constructor(&segments, &bindings) {
                self.violations.insert("Tokio runtime construction");
            }
            if is_blocking_function_call(
                &segments,
                &self.local_functions,
                &self.module_path,
                &bindings,
            ) {
                self.violations.insert("blocking runtime operation");
            }
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if call.method == "block_on" {
            self.violations.insert("block_on");
        }
        visit::visit_expr_method_call(self, call);
    }
}

fn collect_use_aliases(
    tree: &syn::UseTree,
    prefix: &mut Vec<String>,
    bindings: &mut AsyncRuntimeBindings,
) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_aliases(&path.tree, prefix, bindings);
            prefix.pop();
        }
        syn::UseTree::Name(name) => {
            let name = name.ident.to_string();
            if name == "self" {
                if let Some(local_name) = prefix.last().cloned() {
                    record_import(prefix, &local_name, bindings);
                }
            } else {
                let mut full_path = prefix.clone();
                full_path.push(name.clone());
                record_import(&full_path, &name, bindings);
            }
        }
        syn::UseTree::Rename(rename) => {
            let mut full_path = prefix.clone();
            if rename.ident != "self" {
                full_path.push(rename.ident.to_string());
            }
            record_import(&full_path, &rename.rename.to_string(), bindings);
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_use_aliases(item, prefix, bindings);
            }
        }
        syn::UseTree::Glob(_) => record_glob_import(prefix, bindings),
    }
}

fn record_import(full_path: &[String], local_name: &str, bindings: &mut AsyncRuntimeBindings) {
    if matches!(full_path, [tokio] if tokio == "tokio") {
        bindings.tokio_roots.insert(local_name.to_string());
    }
    if matches!(
        full_path,
        [tokio, runtime] if tokio == "tokio" && runtime == "runtime"
    ) {
        bindings.runtime_modules.insert(local_name.to_string());
    }
    if matches!(
        full_path,
        [tokio, task] if tokio == "tokio" && task == "task"
    ) {
        bindings.task_modules.insert(local_name.to_string());
    }
    if matches!(
        full_path,
        [tokio, runtime, ty]
            if tokio == "tokio"
                && runtime == "runtime"
                && matches!(ty.as_str(), "Builder" | "Runtime")
    ) {
        bindings.runtime_types.insert(local_name.to_string());
    }
    if matches!(
        full_path,
        [tokio, task, function]
            if tokio == "tokio"
                && task == "task"
                && function == "block_in_place"
    ) || full_path.last().is_some_and(|name| name == "block_on")
    {
        bindings.blocking_functions.insert(local_name.to_string());
    }
}

fn record_glob_import(prefix: &[String], bindings: &mut AsyncRuntimeBindings) {
    match prefix {
        [tokio] if tokio == "tokio" => {
            bindings.runtime_modules.insert("runtime".to_string());
            bindings.task_modules.insert("task".to_string());
        }
        [tokio, runtime] if tokio == "tokio" && runtime == "runtime" => {
            bindings.runtime_types.insert("Builder".to_string());
            bindings.runtime_types.insert("Runtime".to_string());
        }
        [tokio, task] if tokio == "tokio" && task == "task" => {
            bindings
                .blocking_functions
                .insert("block_in_place".to_string());
        }
        _ => {}
    }
}

fn is_runtime_type_path(segments: &[String], bindings: &AsyncRuntimeBindings) -> bool {
    matches!(segments, [ty] if bindings.runtime_types.contains(ty))
        || matches!(
            segments,
            [tokio, runtime, ty]
                if (tokio == "tokio" || bindings.tokio_roots.contains(tokio))
                    && runtime == "runtime"
                    && matches!(ty.as_str(), "Builder" | "Runtime")
        )
        || matches!(
            segments,
            [runtime, ty]
                if bindings.runtime_modules.contains(runtime)
                    && matches!(ty.as_str(), "Builder" | "Runtime")
        )
}

fn is_runtime_constructor(segments: &[String], bindings: &AsyncRuntimeBindings) -> bool {
    let Some(method) = segments.last() else {
        return false;
    };
    if !matches!(
        method.as_str(),
        "new" | "new_current_thread" | "new_multi_thread"
    ) {
        return false;
    }
    let owner_is_alias = matches!(segments, [owner, _] if bindings.runtime_types.contains(owner));
    owner_is_alias
        || matches!(
            segments,
            [tokio, runtime, owner, _]
                if (tokio == "tokio" || bindings.tokio_roots.contains(tokio))
                    && runtime == "runtime"
                    && matches!(owner.as_str(), "Builder" | "Runtime")
        )
        || matches!(
            segments,
            [runtime, owner, _]
                if bindings.runtime_modules.contains(runtime)
                    && matches!(owner.as_str(), "Builder" | "Runtime")
        )
}

fn is_blocking_function_call(
    segments: &[String],
    local_functions: &BTreeMap<Vec<String>, BTreeSet<String>>,
    module_path: &[String],
    bindings: &AsyncRuntimeBindings,
) -> bool {
    let Some(function) = segments.last() else {
        return false;
    };
    if function == "block_on" && is_local_function_call(segments, module_path, local_functions) {
        return false;
    }
    if bindings.blocking_functions.contains(function) || function == "block_on" {
        return true;
    }
    matches!(
        segments,
        [tokio, task, function]
            if (tokio == "tokio" || bindings.tokio_roots.contains(tokio))
                && task == "task"
                && function == "block_in_place"
    ) || matches!(
        segments,
        [task, function]
            if bindings.task_modules.contains(task) && function == "block_in_place"
    )
}

fn is_local_function_call(
    segments: &[String],
    module_path: &[String],
    local_functions: &BTreeMap<Vec<String>, BTreeSet<String>>,
) -> bool {
    let Some((function, qualifier)) = segments.split_last() else {
        return false;
    };
    let mut target_module = module_path.to_vec();
    let mut remaining = qualifier;
    if let Some(first) = remaining.first() {
        match first.as_str() {
            "crate" => {
                target_module.clear();
                remaining = &remaining[1..];
            }
            "self" => remaining = &remaining[1..],
            "super" => {
                while remaining.first().is_some_and(|part| part == "super") {
                    target_module.pop();
                    remaining = &remaining[1..];
                }
            }
            _ => {}
        }
    }
    target_module.extend(remaining.iter().cloned());
    local_functions
        .get(&target_module)
        .is_some_and(|functions| functions.contains(function))
}

fn path_segments(path: &syn::Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
}

fn expression_path_segments(path: &syn::ExprPath) -> Vec<String> {
    let Some(qself) = &path.qself else {
        return path_segments(&path.path);
    };
    let syn::Type::Path(owner) = qself.ty.as_ref() else {
        return path_segments(&path.path);
    };
    let mut segments = path_segments(&owner.path);
    segments.extend(
        path.path
            .segments
            .iter()
            .skip(qself.position)
            .map(|segment| segment.ident.to_string()),
    );
    segments
}

#[cfg(test)]
#[path = "rust_interop_bridge_audit_test_cases.rs"]
mod test_cases;

#[cfg(test)]
mod tests {
    use super::{
        AsyncRuntimeAudit, audit_file, audit_file_with_module_path,
        collect_async_runtime_bridge_violations, test_cases,
    };
    use std::collections::BTreeSet;

    fn audit(source: &str) -> AsyncRuntimeAudit {
        let file = syn::parse_file(source).expect("test source should parse");
        audit_file(&file)
    }

    fn audit_module(source: &str, module_path: &[&str]) -> AsyncRuntimeAudit {
        let file = syn::parse_file(source).expect("test source should parse");
        audit_file_with_module_path(&file, module_path.iter().map(ToString::to_string).collect())
    }

    fn assert_only_violation(case: &str, source: &str, construct: &'static str) {
        let actual = audit(source).violations;
        let expected = BTreeSet::from([construct]);
        assert_eq!(actual, expected, "{case}");
    }

    #[test]
    fn async_runtime_audit_ignores_literals_local_names_and_unrelated_builders() {
        let audit = audit(
            r###"
            use tokio::runtime::Builder;
            fn safe() {
                let text = r#"runtime.block_on() and Builder::new()"#;
                let _thread = std::thread::Builder::new();
                block_on();
                let _ = text;
            }
            fn block_on() {}
            mod helpers {
                pub fn go() {
                    block_on();
                }
                pub fn block_on() {}
            }
            fn qualified() {
                helpers::block_on();
                self::block_on();
            }
            mod alias_scope {
                use tokio::runtime::Builder;
            }
            mod unrelated_scope {
                struct Builder;
                impl Builder {
                    fn new() -> Self {
                        Self
                    }
                }
                fn safe() {
                    let _builder = Builder::new();
                }
            }
            "###,
        );

        assert!(audit.violations.is_empty());
    }

    #[test]
    fn async_runtime_audit_checks_each_intra_crate_import_shape() {
        for (case, source) in test_cases::CONSTRUCTION_CASES {
            assert_only_violation(case, source, "Tokio runtime construction");
        }
        for (case, source) in test_cases::BLOCKING_CASES {
            assert_only_violation(case, source, "blocking runtime operation");
        }
    }

    #[test]
    fn async_runtime_audit_allows_unrelated_glob_constructors() {
        let ordinary = audit(
            r#"
            use std::collections::*;
            use std::thread::*;
            fn safe() {
                let _map: HashMap<String, String> = HashMap::new();
                let _text = String::new();
                let _items: Vec<String> = Vec::new();
                let _thread = Builder::new();
            }
            "#,
        );

        assert!(ordinary.violations.is_empty());
        for (case, source) in test_cases::NO_VIOLATION_CASES {
            assert!(audit(source).violations.is_empty(), "{case}");
        }
    }

    #[test]
    fn async_runtime_audit_fails_closed_on_unparseable_source() {
        let root =
            std::env::temp_dir().join(format!("sifr-rust-audit-parse-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("temporary audit root should be created");
        let source = root.join("broken.rs");
        std::fs::write(&source, "fn broken(").expect("malformed source should be written");
        let mut violations = Vec::new();
        collect_async_runtime_bridge_violations(&root, &source, &mut violations);
        assert_eq!(
            violations,
            vec![super::AsyncRuntimeBridgeViolation {
                file: "broken.rs".to_string(),
                construct: "unauditable bridge source",
            }]
        );
        std::fs::remove_dir_all(root).expect("temporary audit root should be removed");
    }

    #[test]
    fn async_runtime_audit_resolves_crate_qualified_local_block_on() {
        let audit = audit_module(
            r#"
            pub fn block_on() {}
            pub fn safe() {
                crate::bridges::local_helper::block_on();
                self::block_on();
            }
            "#,
            &["bridges", "local_helper"],
        );

        assert!(audit.violations.is_empty());
    }

    #[test]
    fn async_runtime_audit_finds_aliases_and_blocking_runtime_operations() {
        let audit = audit(
            r#"
            use futures::executor::block_on as wait;
            use tokio::{runtime::Builder as Rt, task::block_in_place};
            use tokio::runtime as rt;
            use tokio::task as task;
            use tokio as t;
            fn forbidden() {
                let runtime = Rt::new_current_thread();
                runtime.block_on(async {});
                let _second = rt::Builder::new_current_thread();
                let _third = t::runtime::Runtime::new();
                wait(async {});
                block_in_place(|| {});
                task::block_in_place(|| {});
                t::task::block_in_place(|| {});
            }
            "#,
        );

        assert!(audit.violations.contains("Tokio runtime construction"));
        assert!(audit.violations.contains("block_on"));
        assert!(audit.violations.contains("blocking runtime operation"));
    }
}
