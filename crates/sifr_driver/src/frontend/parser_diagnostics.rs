use crate::diagnostics::RenderedDiagnostic;
use sifr_python_ast::ModModule;
use sifr_python_parser::Parsed;

pub(crate) fn parse_module_with_diagnostics(
    source: &str,
    context: Option<&str>,
) -> Result<Parsed<ModModule>, Vec<RenderedDiagnostic>> {
    sifr_syntax::parse_module_raw(source, context)
}
