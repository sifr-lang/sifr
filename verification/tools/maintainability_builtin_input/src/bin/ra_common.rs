use hir::{HasVisibility, HirDisplay, Semantics};
use std::{env, path::Path};
use syntax::{
    AstNode,
    ast::{HasAttrs, HasName},
};
#[path = "ra_common/ra_types.rs"]
mod ra_types;
use load_cargo::{LoadCargoConfig, ProcMacroServerChoice};
use project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustLibSource};
use serde_json::json;
fn main() -> anyhow::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let root = Path::new(&args[1]);
    let package = &args[2];
    let mut config = CargoConfig::default();
    config.sysroot = Some(RustLibSource::Discover);
    config.set_test = false;
    config.cfg_overrides.global =
        cfg::CfgDiff::new(vec![], vec![cfg::CfgAtom::Flag(hir::sym::rust_analyzer)]);
    config.target = Some("x86_64-unknown-linux-gnu".into());
    config.run_build_script_command = Some(vec![
        "cargo".into(),
        "check".into(),
        "--locked".into(),
        "--lib".into(),
        "--message-format=json".into(),
        "-p".into(),
        package.clone(),
        "--target".into(),
        config.target.clone().expect("selected target"),
    ]);
    let manifest =
        ProjectManifest::discover_single(&vfs::AbsPathBuf::assert_utf8(root.to_path_buf()))?;
    let mut workspace = ProjectWorkspace::load(manifest, &config, &|p| eprintln!("{p}"))?;
    let capture: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[4])?)?;
    let mut actual_cfg = vec![];
    for atom in capture["cfg"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("missing actual compiler cfg"))?
    {
        let key = hir::Symbol::intern(
            atom["key"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("missing cfg key"))?,
        );
        actual_cfg.push(if let Some(value) = atom["value"].as_str() {
            cfg::CfgAtom::KeyValue {
                key,
                value: hir::Symbol::intern(value),
            }
        } else {
            cfg::CfgAtom::Flag(key)
        });
    }
    let mut disabled = workspace
        .rustc_cfg
        .iter()
        .filter(|atom| !actual_cfg.contains(atom))
        .cloned()
        .collect::<Vec<_>>();
    disabled.push(cfg::CfgAtom::Flag(hir::sym::rust_analyzer));
    let selected_diff = cfg::CfgDiff::new(actual_cfg, disabled);
    config
        .cfg_overrides
        .selective
        .insert(package.clone(), selected_diff);
    workspace.cfg_overrides = config.cfg_overrides.clone();
    let builds = workspace.run_build_scripts(&config, &|p| eprintln!("{p}"))?;
    if let Some(error) = builds.error() {
        anyhow::bail!("selected preparation failed: {error}");
    }
    workspace.set_build_scripts(builds);
    let load = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::Sysroot,
        prefill_caches: false,
        num_worker_threads: 1,
        proc_macro_processes: 1,
    };
    let (db, files, server) = load_cargo::load_workspace(workspace, &config.extra_env, &load)?;
    anyhow::ensure!(server.is_some(), "missing selected proc-macro server");
    hir::attach_db(&db, || {
        let semantics = Semantics::new(&db);
        let mut records = vec![];
        let mut invocations = vec![];
        let mut selected_cfg = None;
        for (file, path) in files.iter() {
            let Some(path) = path.as_path() else { continue };
            if !path.as_str().ends_with(&args[3]) {
                continue;
            }
            let Some(krate) = semantics.first_crate(file) else {
                continue;
            };
            let mut cfg = krate
                .cfg(&db)
                .into_iter()
                .map(|atom| match atom {
                    cfg::CfgAtom::Flag(key) => json!({"key":key.as_str(),"value":null}),
                    cfg::CfgAtom::KeyValue { key, value } => {
                        json!({"key":key.as_str(),"value":value.as_str()})
                    }
                })
                .collect::<Vec<_>>();
            cfg.sort_by_cached_key(|atom| atom.to_string());
            selected_cfg = Some(cfg);
            for module in krate.modules(&db) {
                for definition in module.declarations(&db) {
                    let hir::ModuleDef::Adt(adt) = definition else {
                        continue;
                    };
                    let Some(source) = semantics.source(adt) else {
                        anyhow::bail!("missing common ADT source");
                    };
                    if semantics
                        .original_range(source.value.syntax())
                        .file_id
                        .file_id(&db)
                        != file
                    {
                        continue;
                    }
                    for (attribute_ordinal, attribute) in source.value.attrs().enumerate() {
                        let Some(meta) = attribute.meta() else {
                            continue;
                        };
                        let Some(macros) = semantics.resolve_derive_macro(&meta) else {
                            continue;
                        };
                        for (derive_ordinal, origin) in macros.into_iter().enumerate() {
                            let origin =
                                origin.ok_or_else(|| anyhow::anyhow!("unresolved common macro"))?;
                            if origin.builtin_derive_kind(&db).is_some() {
                                invocations.push(json!({"module":ra_types::module(&db,module),"source_range":format!("{:?}",semantics.original_range(source.value.syntax()).range),"attribute_ordinal":attribute_ordinal,"derive_ordinal":derive_ordinal,"receiver":ra_types::canonical(&db,&adt.ty(&db),&[])?,"macro":format!("{}::{}",ra_types::module(&db,origin.module(&db)),origin.name(&db).as_str()),"declaration_source":source.value.syntax().text().to_string(),"source_name":source.value.name().map(|n|n.text().to_string())}));
                            }
                        }
                    }
                }
                for implementation in module.impl_defs(&db) {
                    let receiver = implementation.self_ty(&db);
                    let Some(adt) = receiver.as_adt() else {
                        continue;
                    };
                    let Some(source) = semantics.source(adt) else {
                        anyhow::bail!("missing impl ADT source");
                    };
                    if semantics
                        .original_range(source.value.syntax())
                        .file_id
                        .file_id(&db)
                        != file
                    {
                        continue;
                    }
                    let Some(trait_) = implementation.trait_(&db) else {
                        continue;
                    };
                    anyhow::ensure!(!receiver.contains_unknown(), "unknown common receiver");
                    for item in implementation.items(&db) {
                        let hir::AssocItem::Function(function) = item else {
                            continue;
                        };
                        let callable = function.fn_ptr_type(&db);
                        anyhow::ensure!(!callable.contains_unknown(), "unknown common signature");
                        let substitution = ra_types::substitution(&db, implementation)?;
                        let generic_bounds = ra_types::bounds(&db, &receiver)?;
                        let signature = json!({"parameters":function.assoc_fn_params(&db).iter().map(|param|ra_types::canonical(&db,param.ty(),&substitution)).collect::<anyhow::Result<Vec<_>>>()?,"return":ra_types::canonical(&db,&function.ret_type(&db),&substitution)?,"variadic":function.is_varargs(&db),"unsafe":function.is_unsafe(&db)});
                        records.push(json!({ "canonical_signature":signature,"generic_bounds":generic_bounds,"visibility":format!("{:?}",function.visibility(&db)),"module":ra_types::module(&db,module),"trait_identity":format!("{}::{}",ra_types::module(&db,trait_.module(&db)),trait_.name(&db).as_str()),"receiver_identity":ra_types::canonical(&db,&receiver,&[])?,"receiver": receiver.display(&db, krate.to_display_target(&db)).to_string(), "trait": trait_.name(&db).as_str(), "method": function.name(&db).as_str(), "signature": callable.display(&db, krate.to_display_target(&db)).to_string() }));
                    }
                }
            }
        }
        for invocation in &mut invocations {
            invocation["identity"] = json!([
                invocation["receiver"],
                invocation["macro"],
                invocation["module"],
                invocation["source_range"],
                invocation["attribute_ordinal"],
                invocation["derive_ordinal"]
            ]);
        }
        for member in &mut records {
            let matching = invocations
                .iter()
                .filter(|invocation| {
                    let trait_ = match invocation["macro"].as_str() {
                        Some("core::fmt::macros::Debug") => "core::fmt::Debug",
                        Some("core::clone::Clone") => "core::clone::Clone",
                        Some("core::cmp::PartialEq") => "core::cmp::PartialEq",
                        Some("core::marker::Copy") => "core::marker::Copy",
                        _ => return false,
                    };
                    invocation["receiver"] == member["receiver_identity"]
                        && member["trait_identity"] == trait_
                })
                .collect::<Vec<_>>();
            anyhow::ensure!(
                matching.len() <= 1,
                "ambiguous invocation-owned common member"
            );
            member["invocation"] = matching
                .first()
                .map(|invocation| invocation["identity"].clone())
                .unwrap_or(serde_json::Value::Null);
        }
        records.sort_by_cached_key(|r| r.to_string());
        println!(
            "{}",
            json!({ "producer":"03fcb77246f2568adb0e9b2fa60d19c6cc1686f4", "cfg":selected_cfg,"common_members":records, "invocations":invocations })
        );
        Ok::<_, anyhow::Error>(())
    })
}
