use sifr_lowering::{
    ExternalDefs, LoweringOptions, LoweringResult, lower_module_with_externals_name_and_options,
};
use sifr_python_ast::Stmt;

pub(super) fn lower_with_prepared_methods(
    module_name: &str,
    stmts: &[Stmt],
    external_defs: &ExternalDefs,
    options: &LoweringOptions,
) -> Result<LoweringResult, Vec<sifr_lowering::HirDiagnostic>> {
    loop {
        let result = lower_module_with_externals_name_and_options(
            module_name,
            stmts,
            external_defs,
            options.clone(),
        );
        // Provisional lowering only records normally typed requests. Actual host
        // transport runs here, before inference and export collection finalize.
        if !external_defs
            .typed_method_processor
            .as_ref()
            .is_some_and(|processor| processor.prepare_pending())
        {
            return result;
        }
    }
}
