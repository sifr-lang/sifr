use hir::{HasVisibility, HirDisplay, Semantics};
use std::{env, path::Path};
use syntax::{
    AstNode,
    ast::{HasAttrs, HasName},
};
#[path = "ra_common/declarations.rs"]
mod declarations;
#[path = "ra_common/include_sources.rs"]
mod include_sources;
#[path = "ra_common/local_sources.rs"]
mod local_sources;
#[path = "ra_common/ra_types.rs"]
mod ra_types;
#[path = "ra_common/source_binder.rs"]
mod source_binder;
use load_cargo::{LoadCargoConfig, ProcMacroServerChoice};
use project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustLibSource};
use serde_json::json;
fn main() -> anyhow::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.get(1).is_some_and(|s| s == "--source-binder-syntax") {
        println!(
            "{}",
            serde_json::to_string(&source_binder::syntax_fixture(&args[2])?)?
        );
        return Ok(());
    }
    let root = Path::new(&args[1]);
    let package = &args[2];
    let mut config = CargoConfig::default();
    config.sysroot = Some(RustLibSource::Discover);
    config.set_test = args.get(5).is_some_and(|s| s == "test");
    config.cfg_overrides.global =
        cfg::CfgDiff::new(vec![], vec![cfg::CfgAtom::Flag(hir::sym::rust_analyzer)]);
    config.target = Some("x86_64-unknown-linux-gnu".into());
    config.run_build_script_command = Some(vec![
        "cargo".into(),
        "check".into(),
        "--locked".into(),
        if config.set_test {
            "--tests".into()
        } else {
            "--lib".into()
        },
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
    let project_model::ProjectWorkspaceKind::Cargo { cargo, .. } = &workspace.kind else {
        anyhow::bail!("missing selected Cargo workspace authority");
    };
    let selected_targets = cargo
        .packages()
        .filter(|p| cargo[*p].is_member && cargo[*p].name == *package)
        .flat_map(|p| cargo[p].targets.iter().copied())
        .filter(|t| {
            matches!(cargo[*t].kind, project_model::TargetKind::Lib { .. })
                && Some(cargo[*t].name.as_str()) == capture["context"]["crate"].as_str()
        })
        .map(|t| cargo[t].root.clone())
        .collect::<Vec<_>>();
    anyhow::ensure!(
        selected_targets.len() == 1,
        "ambiguous selected Cargo library target root"
    );
    let selected_root = selected_targets[0].clone();
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
    if env::var_os("SIFR_BUILTIN_SOURCE_BINDER_RA").is_some() {
        let dependency_cfg = capture["dependency_cfg"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("missing original dependency cfg authority"))?
            .iter()
            .map(|a| {
                let key = hir::Symbol::intern(a["key"].as_str().expect("authenticated cfg key"));
                if let Some(value) = a["value"].as_str() {
                    cfg::CfgAtom::KeyValue {
                        key,
                        value: hir::Symbol::intern(value),
                    }
                } else {
                    cfg::CfgAtom::Flag(key)
                }
            })
            .collect::<Vec<_>>();
        let mut disabled = workspace
            .rustc_cfg
            .iter()
            .filter(|a| !dependency_cfg.contains(a))
            .cloned()
            .collect::<Vec<_>>();
        disabled.push(cfg::CfgAtom::Flag(hir::sym::rust_analyzer));
        config
            .cfg_overrides
            .selective
            .insert("syn".into(), cfg::CfgDiff::new(dependency_cfg, disabled));
    }
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
        let mut trait_methods = std::collections::BTreeMap::new();
        let root_files = files
            .iter()
            .filter_map(|(file, path)| {
                (path.as_path() == Some(selected_root.as_path())).then_some(file)
            })
            .collect::<Vec<_>>();
        anyhow::ensure!(
            root_files.len() == 1,
            "missing/ambiguous selected root file identity"
        );
        let crates = hir::Crate::all(&db)
            .into_iter()
            .filter(|c| c.root_file(&db) == root_files[0])
            .collect::<Vec<_>>();
        anyhow::ensure!(
            crates.len() == 1 && crates[0].origin(&db).is_local(),
            "missing/ambiguous selected semantic crate authority"
        );
        let krate = crates[0];
        let mut selected_cfg = krate
            .cfg(&db)
            .into_iter()
            .map(|atom| match atom {
                cfg::CfgAtom::Flag(key) => json!({"key":key.as_str(),"value":null}),
                cfg::CfgAtom::KeyValue { key, value } => {
                    json!({"key":key.as_str(),"value":value.as_str()})
                }
            })
            .collect::<Vec<_>>();
        selected_cfg.sort_by_cached_key(|atom| atom.to_string());
        let context = json!({"kind":"ra-semantic-selected-cargo-target-root","root_file":selected_root.as_str(),"package":package,"crate":capture["context"]["crate"]});
        if env::var_os("SIFR_BUILTIN_SOURCE_BINDER_RA").is_some() {
            let result = source_binder::capture(&semantics, &db, &files, krate, &args[3])?;
            println!("{}", serde_json::to_string(&result)?);
            return Ok(());
        }
        for (file, path) in files.iter() {
            let Some(path) = path.as_path() else { continue };
            if !(if args[3].ends_with('/') {
                path.as_str().contains(&args[3])
            } else {
                path.as_str().ends_with(&args[3])
            }) {
                continue;
            }
            let file_text = std::fs::read_to_string(path.as_str())?;
            for module in local_sources::modules(
                &semantics,
                &db,
                hir::EditionedFileId::new(&db, file, krate.edition(&db)),
                krate,
            ) {
                for definition in module.declarations(&db) {
                    let hir::ModuleDef::Adt(adt) = definition else {
                        continue;
                    };
                    let Some(source) = semantics.source(adt) else {
                        anyhow::bail!("missing common ADT source");
                    };
                    let included = source.file_id.original_file_respecting_includes(&db);
                    let is_include =
                        source.file_id.is_macro() && included != source.file_id.original_file(&db);
                    let source_file = if is_include {
                        included
                    } else {
                        semantics.original_range(source.value.syntax()).file_id
                    };
                    if source_file.file_id(&db) != file {
                        continue;
                    }
                    let mut physical = None;
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
                                if is_include && physical.is_none() {
                                    let node = include_sources::physical_adt(
                                        &semantics,
                                        adt,
                                        source.file_id,
                                        included,
                                    )?;
                                    include_sources::token_correspondence(
                                        node.syntax(),
                                        source.value.syntax(),
                                    )?;
                                    physical = Some(node);
                                }
                                let (site, range, text, mapping) = if let Some(node) = &physical {
                                    let original_attribute =
                                        node.attrs().nth(attribute_ordinal).ok_or_else(|| {
                                            anyhow::anyhow!(
                                                "missing original include attribute ordinal"
                                            )
                                        })?;
                                    let original_meta =
                                        original_attribute.meta().ok_or_else(|| {
                                            anyhow::anyhow!(
                                                "missing original include attribute meta"
                                            )
                                        })?;
                                    anyhow::ensure!(
                                        declarations::source_tokens(original_meta.syntax())
                                            == declarations::source_tokens(meta.syntax()),
                                        "included original derive attribute correspondence conflict"
                                    );
                                    let range = format!("{:?}", node.syntax().text_range());
                                    let mut mapping = include_sources::token_correspondence(
                                        node.syntax(),
                                        source.value.syntax(),
                                    )?;
                                    mapping["kind"] =
                                        json!("ra-public-include-token-descent-and-to-def");
                                    mapping["receiver"] =
                                        ra_types::canonical(&db, &adt.ty(&db), &[])?;
                                    mapping["file"] = json!(path.as_str());
                                    mapping["source_range"] = json!(range);
                                    (
                                        Some(declarations::invocation_site(
                                            &original_meta,
                                            derive_ordinal,
                                            &file_text,
                                            path.as_str(),
                                        )?),
                                        range,
                                        node.syntax().text().to_string(),
                                        mapping,
                                    )
                                } else {
                                    (
                                        if source.file_id.original_file(&db).file_id(&db) == file
                                            && !source.file_id.is_macro()
                                        {
                                            Some(declarations::invocation_site(
                                                &meta,
                                                derive_ordinal,
                                                &file_text,
                                                path.as_str(),
                                            )?)
                                        } else {
                                            None
                                        },
                                        format!(
                                            "{:?}",
                                            semantics.original_range(source.value.syntax()).range
                                        ),
                                        source.value.syntax().text().to_string(),
                                        serde_json::Value::Null,
                                    )
                                };
                                invocations.push(json!({"include_source_mapping":mapping,"invocation_site":site,"module":ra_types::module(&db,module),"source_range":range,"attribute_ordinal":attribute_ordinal,"derive_ordinal":derive_ordinal,"receiver":ra_types::canonical(&db,&adt.ty(&db),&[])?,"macro":format!("{}::{}",ra_types::module(&db,origin.module(&db)),origin.name(&db).as_str()),"declaration_facts":declarations::facts(&db,&semantics,hir::GenericDef::Adt(adt),source.value.syntax(),&local_sources::canonical_adt(&db,adt)?)?,"declaration_source":text,"source_name":source.value.name().map(|n|n.text().to_string())}));
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
                    let included = source.file_id.original_file_respecting_includes(&db);
                    let source_file = if source.file_id.is_macro()
                        && included != source.file_id.original_file(&db)
                    {
                        included
                    } else {
                        semantics.original_range(source.value.syntax()).file_id
                    };
                    if source_file.file_id(&db) != file {
                        continue;
                    }
                    let Some(trait_) = implementation.trait_(&db) else {
                        continue;
                    };
                    let receiver_identity = ra_types::canonical(&db, &receiver, &[])?;
                    let trait_identity = format!(
                        "{}::{}",
                        ra_types::module(&db, trait_.module(&db)),
                        trait_.name(&db).as_str()
                    );
                    let selected = invocations.iter().any(|i| {
                        i["receiver"] == receiver_identity
                            && ra_types::builtin_trait(i["macro"].as_str().unwrap_or(""))
                                == Some(trait_identity.as_str())
                    });
                    if !selected {
                        continue;
                    }
                    for item in trait_.items(&db) {
                        if let hir::AssocItem::Function(original) = item {
                            let source = semantics.source(original).ok_or_else(|| {
                                anyhow::anyhow!("missing actual trait declaration source")
                            })?;
                            let identity = format!(
                                "{}::{}::{}",
                                ra_types::module(&db, trait_.module(&db)),
                                trait_.name(&db).as_str(),
                                original.name(&db).as_str()
                            );
                            trait_methods.insert(
                                identity.clone(),
                                declarations::facts(
                                    &db,
                                    &semantics,
                                    hir::GenericDef::Function(original),
                                    source.value.syntax(),
                                    &identity,
                                )?,
                            );
                        }
                    }
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
                        let trait_source = semantics.source(function).ok_or_else(|| {
                            anyhow::anyhow!("missing original trait method source")
                        })?;
                        let original_function =
                            semantics.to_def(&trait_source.value).ok_or_else(|| {
                                anyhow::anyhow!("unresolved original trait method declaration")
                            })?;
                        let trait_identity = format!(
                            "{}::{}",
                            ra_types::module(&db, trait_.module(&db)),
                            trait_.name(&db).as_str()
                        );
                        let original_facts = declarations::facts(
                            &db,
                            &semantics,
                            hir::GenericDef::Function(original_function),
                            trait_source.value.syntax(),
                            &format!("{}::{}", trait_identity, function.name(&db).as_str()),
                        )?;
                        records.push(json!({ "trait_declaration_facts":original_facts, "canonical_signature":signature,"generic_bounds":generic_bounds,"visibility":format!("{:?}",function.visibility(&db)),"module":ra_types::module(&db,module),"trait_identity":format!("{}::{}",ra_types::module(&db,trait_.module(&db)),trait_.name(&db).as_str()),"receiver_identity":ra_types::canonical(&db,&receiver,&[])?,"receiver": receiver.display(&db, krate.to_display_target(&db)).to_string(), "trait": trait_.name(&db).as_str(), "method": function.name(&db).as_str(), "signature": callable.display(&db, krate.to_display_target(&db)).to_string() }));
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
                        Some("core::cmp::Eq") => "core::cmp::Eq",
                        Some("core::cmp::Ord") => "core::cmp::Ord",
                        Some("core::cmp::PartialOrd") => "core::cmp::PartialOrd",
                        Some("core::default::Default") => "core::default::Default",
                        Some("core::hash::macros::Hash") => "core::hash::Hash",
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
            json!({ "producer":"03fcb77246f2568adb0e9b2fa60d19c6cc1686f4", "context":context,"cfg":selected_cfg,"common_members":records, "invocations":invocations,"trait_methods":trait_methods })
        );
        Ok::<_, anyhow::Error>(())
    })
}
