//! Serialize the unchanged semantic producer records in their original order.
//! No complete original inventory or selected semantic fact is omitted.
use crate::{expanded::Declaration, identity, typed};
use rustc_middle::ty::TyCtxt;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

fn record(writer: &mut impl Write, first: &mut bool, value: &Value) {
    if !*first {
        writer.write_all(b",").expect("write record separator");
    }
    *first = false;
    serde_json::to_writer(writer, value).expect("serialize complete semantic record");
}
pub fn write(tcx: TyCtxt<'_>, ast: &[Declaration], path: PathBuf) {
    let mut temporary = path.clone();
    temporary.set_extension(format!("capture-original-{}.tmp", std::process::id()));
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .expect("create exclusively owned original capture output");
    let mut writer = BufWriter::new(file);
    writer
        .write_all(b"{\"declarations\":[")
        .expect("write declaration array");
    let mut first = true;
    let mut catalog = BTreeMap::new();
    // typed::capture itself, its selection, facts and order are unchanged.
    // Its per-body catalogs are extended in the same declaration order as before.
    for decl in ast {
        let (records, body_catalog) = typed::capture(tcx, std::slice::from_ref(decl));
        catalog.extend(body_catalog);
        for declaration in records {
            record(&mut writer, &mut first, &declaration);
        }
    }
    writer
        .write_all(b"],\"callable_catalog\":")
        .expect("write catalog key");
    serde_json::to_writer(&mut writer, &catalog).expect("serialize complete callable catalog");
    drop(catalog);
    writer
        .write_all(b",\"expanded_owner_ledger\":[")
        .expect("write owner ledger key");
    first = true;
    for decl in ast.iter().filter(|decl| typed::selected(tcx, decl)) {
        record(
            &mut writer,
            &mut first,
            &json!({"owner":tcx.def_path_str(decl.def.to_def_id()),
            "token_sequence":identity::tokens(&decl.tokens),"ast_body":decl.body}),
        );
    }
    writer
        .write_all(b"],\"other_macro_declarations\":[")
        .expect("write other macro key");
    first = true;
    for decl in ast {
        for other in typed::other_macros(tcx, std::slice::from_ref(decl)) {
            record(&mut writer, &mut first, &other);
        }
    }
    writer
        .write_all(b"],\"source_files\":[")
        .expect("write source file key");
    first = true;
    for file in tcx.sess.source_map().files().iter() {
        if let Some(source) = &file.src {
            record(
                &mut writer,
                &mut first,
                &json!({
                "file":file.name.prefer_local_unconditionally().to_string(),"text":source.as_str()}),
            );
        }
    }
    writer.write_all(b"],\"cfg\":").expect("write cfg key");
    let mut cfg = tcx
        .sess
        .config
        .iter()
        .map(|(key, value)| {
            json!({
        "key":key.to_string(),"value":value.map(|value|value.to_string())})
        })
        .collect::<Vec<_>>();
    cfg.sort_by_cached_key(|atom| atom.to_string());
    serde_json::to_writer(&mut writer, &cfg).expect("serialize actual cfg");
    writer
        .write_all(b",\"lowering_erased_bound\":")
        .expect("write erased bound key");
    serde_json::to_writer(
        &mut writer,
        &tcx.lang_items()
            .pointee_sized_trait()
            .map(|def| identity::path(tcx, def)),
    )
    .expect("serialize unchanged erased bound");
    writer
        .write_all(b",\"schema\":\"sifr-maintainability-builtin-capability-v3\"}")
        .expect("finish complete original capture");
    writer.flush().expect("flush complete original capture");
    writer
        .get_ref()
        .sync_all()
        .expect("durable complete original capture");
    assert!(
        !path.exists(),
        "original capture destination already exists"
    );
    std::fs::rename(temporary, path).expect("publish complete original capture");
}
