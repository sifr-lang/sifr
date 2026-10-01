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
mod expanded;
mod identity;
mod typed;
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use serde_json::json;
#[derive(Default)]
struct Capture {
    ast: Vec<expanded::Declaration>,
}
impl Callbacks for Capture {
    fn after_expansion<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        self.ast = expanded::capture(tcx);
        Compilation::Continue
    }
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        let (result, catalog) = typed::capture(tcx, &self.ast);
        let mut cfg=tcx.sess.config.iter().map(|(key,value)|json!({"key":key.to_string(),"value":value.map(|value|value.to_string())})).collect::<Vec<_>>();
        cfg.sort_by_cached_key(|atom| atom.to_string());
        let other = typed::other_macros(tcx, &self.ast);
        let sources=tcx.sess.source_map().files().iter().filter_map(|file|file.src.as_ref().map(|src|json!({"file":file.name.prefer_local_unconditionally().to_string(),"text":src.as_str()}))).collect::<Vec<_>>();
        let path = std::env::var_os("SIFR_BUILTIN_CAPTURE").expect("explicit capture destination");
        let bytes = serde_json::to_vec_pretty(&json!({"schema":"sifr-maintainability-builtin-capability-v1", "cfg":cfg,"lowering_erased_bound":tcx.lang_items().pointee_sized_trait().map(|def|identity::path(tcx,def)),"callable_catalog":catalog,"declarations":result, "other_macro_declarations":other, "source_files":sources})).expect("JSON serialization");
        std::fs::write(path, bytes).expect("write owned evidence");
        Compilation::Stop
    }
}
fn main() {
    rustc_driver::run_compiler(
        &std::env::args().collect::<Vec<_>>(),
        &mut Capture::default(),
    );
}
