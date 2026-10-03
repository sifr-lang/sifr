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
mod capture_output;
mod dynamic;
mod expanded;
mod expanded_storage;
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
    original_text: Option<expanded_storage::OriginalText>,
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
            std::fs::write(
                path,
                serde_json::to_vec_pretty(&source_envelope::capture(
                    tcx,
                    &self.ast,
                    &self.associations,
                    &self.stage_attributes,
                ))
                .expect("envelope diagnostic serialization"),
            )
            .expect("write unaccepted envelope diagnostic");
        }
        if let Some(path) = std::env::var_os("SIFR_BUILTIN_SOURCE_BINDER") {
            std::fs::write(
                path,
                serde_json::to_vec_pretty(&source_binder::capture(tcx))
                    .expect("source binder serialization"),
            )
            .expect("write source binder diagnostic");
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
        std::fs::write(
            inventory_path,
            serde_json::to_vec_pretty(&inventory).expect("inventory serialization"),
        )
        .expect("write independent inventory");
        drop(inventory);
        let path = std::env::var_os("SIFR_BUILTIN_CAPTURE").expect("explicit capture destination");
        capture_output::write(tcx, &self.ast, path.into());
        Compilation::Stop
    }
}
fn main() {
    rustc_driver::run_compiler(
        &std::env::args().collect::<Vec<_>>(),
        &mut Capture::default(),
    );
}
