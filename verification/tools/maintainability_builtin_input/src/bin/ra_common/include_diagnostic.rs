//! Development-only public include-origin inventory. Publishes no accepted proof.
use hir::{Semantics, db::HirDatabase};
use serde_json::{Value, json};
use syntax::AstNode;
fn range(r: syntax::TextRange) -> Value {
    json!([u32::from(r.start()), u32::from(r.end())])
}
fn node<DB: HirDatabase>(
    inventory: &mut super::include_inventory::Inventory,
    sem: &Semantics<'_, DB>,
    db: &DB,
    files: &vfs::Vfs,
    file: hir::HirFileId,
    n: &syntax::SyntaxNode,
    owner: String,
    module: hir::Module,
    roundtrip: bool,
) -> anyhow::Result<Value> {
    let module_source = sem.module_definition_node(module);
    let module_roundtrip = if let Some(m) = syntax::ast::Module::cast(module_source.value.clone()) {
        sem.to_def(&m) == Some(module)
    } else if let Some(f) = syntax::ast::SourceFile::cast(module_source.value.clone()) {
        sem.to_def(&f) == Some(module)
    } else {
        false
    };
    let source = inventory.reference(sem, db, files, file, n)?;
    let context = super::include_context::capture(inventory, sem, db, files, file, n, module)?;
    let semantics = super::include_semantics::capture(inventory, sem, db, files, file, n)?;
    Ok(
        json!({"context":context,"semantics":semantics,"owner":owner,"module":format!("{module:?}"),"module_source":{"file":format!("{:?}",module_source.file_id),"kind":format!("{:?}",module_source.value.kind()),"range":range(module_source.value.text_range()),"roundtrip":module_roundtrip,"aggregate_original":sem.original_range_opt(&module_source.value).map(|r|json!({"file":files.file_path(r.file_id.file_id(db)).as_path().map(|p|p.as_str()),"range":range(r.range)}))},"is_include":super::include_inventory::is_include(file,db),"roundtrip":roundtrip,"native_file":format!("{file:?}"),"kind":format!("{:?}",n.kind()),"range":range(n.text_range()),"aggregate_original":sem.original_range_opt(n).map(|r|json!({"file":files.file_path(r.file_id.file_id(db)).as_path().map(|p|p.as_str()),"range":range(r.range)})),"syntax":source,"parent_context":sem.find_parent_file(file).map(|parent|json!({"file":format!("{:?}",parent.file_id),"kind":format!("{:?}",parent.value.kind()),"range":range(parent.value.text_range()),"aggregate_original":sem.original_range_opt(&parent.value).map(|r|json!({"file":files.file_path(r.file_id.file_id(db)).as_path().map(|p|p.as_str()),"range":range(r.range)}))}))}),
    )
}
pub fn capture<DB: HirDatabase>(
    sem: &Semantics<'_, DB>,
    db: &DB,
    files: &vfs::Vfs,
    krate: hir::Crate,
    suffix: &str,
) -> anyhow::Result<Value> {
    let mut inventory = super::include_inventory::Inventory::new()?;
    let mut owners = vec![];
    let mut declaration_inventory = vec![];
    macro_rules! add {
        ($d:expr,$module:expr) => {{
            let d=$d;
            if let Some(src)=sem.source(d) {
                owners.push(node(&mut inventory,sem,db,files,src.file_id,src.value.syntax(),format!("{d:?}"),$module,sem.to_def(&src.value)==Some(d))?);
            } else { owners.push(json!({"owner":format!("{d:?}"),"disposition":"missing-native-source"})); }
        }};
    }
    macro_rules! associated {
        ($d:expr,$module:expr) => {
            for item in $d.items(db) {
                match item {
                    hir::AssocItem::Function(f) => add!(f, $module),
                    hir::AssocItem::Const(c) => add!(c, $module),
                    hir::AssocItem::TypeAlias(t) => add!(t, $module),
                }
            }
        };
    }
    let mut modules = krate.modules(db);
    // Register roots through actual owner sources before discovering body-local modules.
    for module in modules.clone() {
        let source = sem.module_definition_node(module);
        inventory.reference(sem, db, files, source.file_id, &source.value)?;
    }
    let mut seen_files = std::collections::HashSet::new();
    let mut cursor = 0;
    loop {
        let roots = inventory.source_roots();
        if cursor >= roots.len() {
            break;
        }
        let (file, root) = roots[cursor].clone();
        cursor += 1;
        if !seen_files.insert(file) {
            continue;
        }
        for n in root.descendants() {
            if let Some(a) = syntax::ast::Adt::cast(n.clone()) {
                if let Some(d) = sem.to_def(&a) {
                    let m = d.module(db);
                    if m.krate(db) == krate && !modules.contains(&m) {
                        modules.push(m);
                    }
                }
            }
            if let Some(f) = syntax::ast::Fn::cast(n.clone()) {
                if let Some(d) = sem.to_def(&f) {
                    let m = d.module(db);
                    if m.krate(db) == krate && !modules.contains(&m) {
                        modules.push(m);
                    }
                }
            }
            if let Some(call) = syntax::ast::MacroCall::cast(n) {
                if let Some(expanded) = sem.expand_macro_call(&call) {
                    if super::include_inventory::is_include(expanded.file_id, db) {
                        inventory.reference(sem, db, files, expanded.file_id, &expanded.value)?;
                    }
                }
            }
        }
    }
    let mut seen_macros = std::collections::HashSet::new();
    for module in modules {
        let legacy_macros = module.legacy_macros(db);
        let module_members = module.declarations(db);
        let impls = module.impl_defs(db);
        declaration_inventory.push(json!({"module":format!("{module:?}"),"members":module_members.iter().map(|m|json!({"actual_module_def":format!("{m:?}"),"disposition":match m{hir::ModuleDef::Module(_)=>"module-source-enumerated-separately",hir::ModuleDef::EnumVariant(_)=>"imported-enum-variant-not-original-declaration",hir::ModuleDef::BuiltinType(_)=>"builtin-type-with-no-source-declaration",_=>"owned-source-enumerated-below"}})).collect::<Vec<_>>(),"legacy_macros":legacy_macros.iter().map(|m|json!({"owner":format!("{m:?}"),"actual_definition_module":format!("{:?}",m.module(db))})).collect::<Vec<_>>(),"impl_definitions":impls.iter().map(|d|format!("{d:?}")).collect::<Vec<_>>()}));
        if let Some(src) = module.declaration_source(db) {
            owners.push(node(
                &mut inventory,
                sem,
                db,
                files,
                src.file_id,
                src.value.syntax(),
                format!("{module:?}"),
                module,
                sem.to_def(&src.value) == Some(module),
            )?);
        }
        for d in module_members {
            match d {
                hir::ModuleDef::Function(f) => add!(f, module),
                hir::ModuleDef::Adt(d) => add!(d, module),
                hir::ModuleDef::Const(c) => add!(c, module),
                hir::ModuleDef::Static(c) => add!(c, module),
                hir::ModuleDef::TypeAlias(t) => add!(t, module),
                hir::ModuleDef::Macro(m) => {
                    if !seen_macros.insert(m) {
                        continue;
                    }
                    if let Some(src) = sem.source(m) {
                        let roundtrip = src
                            .value
                            .as_ref()
                            .left()
                            .is_some_and(|s| sem.to_def(s) == Some(m));
                        owners.push(node(
                            &mut inventory,
                            sem,
                            db,
                            files,
                            src.file_id,
                            src.value.syntax(),
                            format!("{m:?}"),
                            module,
                            roundtrip,
                        )?);
                    } else {
                        owners.push(
                            json!({"owner":format!("{m:?}"),"disposition":"missing-native-source"}),
                        );
                    }
                }
                hir::ModuleDef::Trait(t) => {
                    add!(t, module);
                    associated!(t, module);
                }
                _ => {}
            }
        }
        for m in legacy_macros {
            if !seen_macros.insert(m) {
                continue;
            }
            if let Some(src) = sem.source(m) {
                let roundtrip = src
                    .value
                    .as_ref()
                    .left()
                    .is_some_and(|n| sem.to_def(n) == Some(m));
                owners.push(node(
                    &mut inventory,
                    sem,
                    db,
                    files,
                    src.file_id,
                    src.value.syntax(),
                    format!("{m:?}"),
                    m.module(db),
                    roundtrip,
                )?);
            } else {
                owners
                    .push(json!({"owner":format!("{m:?}"),"disposition":"missing-native-source"}));
            }
        }
        for d in impls {
            add!(d, module);
            associated!(d, module);
        }
    }
    let mut include_contexts = vec![];
    for (file, root) in inventory.source_roots() {
        if !(super::include_inventory::is_include(file, db)) {
            continue;
        }
        let parent = sem
            .find_parent_file(file)
            .ok_or_else(|| anyhow::anyhow!("missing actual include invocation parent"))?;
        let scope = sem
            .scope(&parent.value)
            .ok_or_else(|| anyhow::anyhow!("missing actual include invocation scope"))?;
        let m = scope.module();
        include_contexts.push(json!({"root":inventory.reference(sem,db,files,file,&root)?,"module":format!("{m:?}"),"context":super::include_context::capture(&mut inventory,sem,db,files,file,&root,m)?}));
    }
    let mut selected = inventory.physical_files();
    for (file, path) in files.iter() {
        if path.as_path().is_some_and(|path| {
            if suffix.ends_with('/') {
                path.as_str().contains(suffix)
            } else {
                path.as_str().ends_with(suffix)
            }
        }) {
            let file = hir::EditionedFileId::new(db, file, krate.edition(db));
            if !selected.contains(&file) {
                selected.push(file);
            }
        }
    }
    selected.sort_by_key(|f| (files.file_path(f.file_id(db)).to_string(), format!("{f:?}")));
    let mut physical = vec![];
    for file in selected {
        let path = files.file_path(file.file_id(db));
        let path = path
            .as_path()
            .ok_or_else(|| anyhow::anyhow!("missing physical origin VFS path"))?;
        let parsed = sem.parse(file);
        physical.push(json!({"file":path.as_str(),"editioned_file":format!("{file:?}"),"syntax":inventory.reference(sem,db,files,file.into(),parsed.syntax())?}));
    }
    Ok(
        json!({"schema":"development-public-include-token-inventory-v1","semantic_export":false,"accepted_proof":false,"owners":owners,"declaration_inventory":declaration_inventory,"include_contexts":include_contexts,"physical":physical,"syntax_inventory":inventory.finish()}),
    )
}
