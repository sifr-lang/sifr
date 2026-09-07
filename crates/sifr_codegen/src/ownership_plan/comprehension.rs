use crate::{HirExpr, RustExpr};

/// A comprehension result executes once per innermost iteration. Only that
/// generator's binding is freshly owned on every execution; outer bindings
/// must remain available for subsequent iterations.
pub(crate) fn materialize_comprehension_value(
    source: &HirExpr,
    lowered: RustExpr,
    generators: &[(String, HirExpr, Option<HirExpr>)],
) -> RustExpr {
    let HirExpr::Name { name, .. } = source else {
        return lowered;
    };
    let fresh_innermost = generators
        .last()
        .is_some_and(|(binding, _, _)| binding == name);
    if !fresh_innermost
        && !source.ty().contains_affine_resource()
        && !crate::helpers::is_copy_type_for_codegen(source.ty())
        && crate::RustEmitter::rust_expr_is_reusable_place_for_ir(&lowered)
    {
        super::materialize_owned_value(source.ty(), lowered)
    } else {
        lowered
    }
}
