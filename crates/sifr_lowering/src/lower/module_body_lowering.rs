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
    let mut classes: Vec<HirClass> = stmts
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::ClassDef(class) => lower_class(class, ctx),
            _ => None,
        })
        .collect();
    generic_method_requirements::close_generic_method_requirements(ctx);
    super::method_receiver_analysis::validate_and_annotate_class_receivers(&mut classes, ctx);
    for class in &mut classes {
        for method in class
            .methods
            .iter_mut()
            .chain(class.operator_impls.iter_mut().map(|(_, method)| method))
        {
            super::method_receiver_places::validate_function_method_places(method, ctx);
        }
    }

    let mut functions: Vec<HirFunction> = stmts
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
    for function in &mut functions {
        super::method_receiver_analysis::annotate_and_verify_function_calls(function, ctx);
        super::method_receiver_places::validate_function_method_places(function, ctx);
    }
    (functions, classes)
}
