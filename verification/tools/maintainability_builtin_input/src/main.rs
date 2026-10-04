#![feature(rustc_private)]
extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_interface;
extern crate rustc_lexer;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_type_ir;
mod dynamic;
mod expanded;
mod identity;
mod inventory;
mod semantic;
mod source_binder;
mod source_envelope;
mod typed;
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
#[derive(Default)]
struct Capture {
    ast: Vec<expanded::Declaration>,
    original_text: Option<expanded::storage::OriginalText>,
    stage_attributes: Vec<expanded::StageAttribute>,
    associations: Vec<(rustc_ast::NodeId, rustc_hir::def_id::LocalDefId)>,
}
impl Callbacks for Capture {
    fn after_expansion<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        if std::env::var_os("SIFR_BUILTIN_SOURCE_BINDER").is_none()
            || std::env::var_os("SIFR_BUILTIN_SOURCE_ENVELOPE").is_some()
        {
            let (ast, original_text) = expanded::capture(tcx);
            self.ast = ast;
            self.original_text = Some(original_text);
            if std::env::var_os("SIFR_BUILTIN_SOURCE_ENVELOPE").is_some() {
                self.associations = expanded::associations(tcx);
                self.stage_attributes = expanded::stage_attributes(tcx);
            }
        }
        Compilation::Continue
    }
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        if let Some(original_text) = self.original_text.take() {
            original_text.restore(&mut self.ast);
        }
        if let Some(path) = std::env::var_os("SIFR_BUILTIN_SOURCE_ENVELOPE") {
            write_original_json(
                path.into(),
                &source_envelope::capture(
                    tcx,
                    &self.ast,
                    &self.associations,
                    &self.stage_attributes,
                ),
            );
        }
        if let Some(path) = std::env::var_os("SIFR_BUILTIN_SOURCE_BINDER") {
            write_original_json(path.into(), &source_binder::capture(tcx));
            return Compilation::Stop;
        }
        let inventory = inventory::capture(tcx);
        // Bounded acceptance hook changes only the saved projection, after inventory.
        if let Ok(owner) = std::env::var("SIFR_BUILTIN_OMIT_AST_OWNER") {
            self.ast
                .retain(|decl| tcx.def_path_str(decl.def.to_def_id()) != owner);
        }
        inventory::reconcile(tcx, &inventory, &self.ast);
        let inventory_path = std::env::var_os("SIFR_BUILTIN_INVENTORY")
            .expect("explicit independent inventory destination");
        write_original_json(inventory_path.into(), &inventory);
        drop(inventory);
        let path = std::env::var_os("SIFR_BUILTIN_CAPTURE").expect("explicit capture destination");
        capture_output::write(tcx, &self.ast, path.into());
        Compilation::Stop
    }
}
/// Serialize every original diagnostic value without a second complete byte buffer.
fn write_original_json(path: std::path::PathBuf, value: &serde_json::Value) {
    use std::io::Write;
    let mut temporary = path.clone();
    temporary.set_extension(format!("original-json-{}.tmp", std::process::id()));
    let file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .expect("create owned original diagnostic output");
    let mut writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)
        .expect("serialize complete original diagnostic");
    writer.flush().expect("flush complete original diagnostic");
    writer
        .get_ref()
        .sync_all()
        .expect("durable complete original diagnostic");
    assert!(
        !path.exists(),
        "original diagnostic destination already exists"
    );
    std::fs::rename(temporary, path).expect("publish complete original diagnostic");
}

fn main() {
    rustc_driver::run_compiler(
        &std::env::args().collect::<Vec<_>>(),
        &mut Capture::default(),
    );
}

/// Complete capture output serialization; semantic producers stay unchanged.
mod capture_output {
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
}
