use super::{LowerCtx, Type, resolve_object_field_type};
use crate::lower::binding_mutability::ensure_mutable_parameter_binding;
use crate::lower::statement_diagnostics;
use ruff_text_size::Ranged;
use sifr_python_ast::{Expr, ExprSubscript};

pub(super) fn resolve_nested_assignment_root(
    subscript: &ExprSubscript,
    ctx: &mut LowerCtx,
) -> Option<(String, Option<String>, Type)> {
    let (object, field, ty) = match subscript.value.as_ref() {
        Expr::Name(name) => {
            let ty = ctx
                .scope
                .lookup(&name.id)
                .map(|info| info.effective_type().clone())
                .unwrap_or(Type::Unknown);
            (name.id.to_string(), None, ty)
        }
        Expr::Attribute(attribute) => {
            let Expr::Name(name) = attribute.value.as_ref() else {
                statement_diagnostics::invalid_assignment_target(
                    ctx,
                    "nested subscript assignment target must be a simple name",
                    attribute.value.range(),
                );
                return None;
            };
            let object = name.id.to_string();
            let field = attribute.attr.to_string();
            let ty = resolve_object_field_type(ctx, &object, &field);
            (object, Some(field), ty)
        }
        _ => {
            statement_diagnostics::invalid_assignment_target(
                ctx,
                "nested subscript assignment target must be a simple name",
                subscript.value.range(),
            );
            return None;
        }
    };
    ensure_mutable_parameter_binding(ctx, &object, subscript.value.range())
        .then_some((object, field, ty))
}
