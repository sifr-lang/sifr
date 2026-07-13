use sifr_ir::{HirModule, PythonInteropDeclaration, PythonInteropEffect};

pub(crate) fn python_omit_parameter_indices(
    declaration: &PythonInteropDeclaration,
) -> impl Iterator<Item = usize> + '_ {
    declaration
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| parameter.omit_when_absent.then_some(index))
}

pub(crate) fn module_uses_async_python_declaration(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .chain(module.classes.iter().flat_map(|class| class.methods.iter()))
        .any(|function| {
            function
                .python_interop
                .iter()
                .any(|declaration| declaration.effect == PythonInteropEffect::Async)
        })
}
