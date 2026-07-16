use super::{
    generic_method_requirements, lower_class, lower_function,
    module_function_registry::ModuleFunctionRegistry, LowerCtx, Stmt,
};
use sifr_ir::{HirClass, HirFunction};

pub(super) fn lower_module_bodies(
    stmts: &[Stmt],
    function_names: &mut ModuleFunctionRegistry,
    ctx: &mut LowerCtx,
) -> (Vec<HirFunction>, Vec<HirClass>) {
    let classes = stmts
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::ClassDef(class) => lower_class(class, ctx),
            _ => None,
        })
        .collect();
    generic_method_requirements::close_generic_method_requirements(ctx);

    let functions = stmts
        .iter()
        .filter_map(|stmt| {
            let Stmt::FunctionDef(function) = stmt else {
                return None;
            };
            let name = function.name.to_string();
            if !function_names.note_lowering(&name) {
                return None;
            }
            let lowered = lower_function(function, ctx)?;
            if let Some(function_type) = ctx.functions.get_mut(&name) {
                *function_type.return_type = lowered.return_type.clone();
            }
            Some(lowered)
        })
        .collect();
    (functions, classes)
}
