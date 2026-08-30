use super::LowerCtx;
use super::expressions::{is_poisoned_binding_expr, lower_expr};
use super::type_bounds::supports_print_formatting;
use ruff_text_size::{Ranged, TextRange, TextSize};
use sifr_ir::{
    HirExpr, HirTemplateFormatSpec, HirTemplateFormatSpecPart, HirTemplateInterpolation,
    HirTemplateSegment, HirTemplateStaticMapping, HirTemplateString,
};
use sifr_python_ast::{ExprTString, InterpolatedStringElement, InterpolatedStringFormatSpec};
use sifr_type_system::Type;

const VIRTUAL_HOLE: char = '\u{fffc}';

pub(in crate::lower) fn lower_template_string_expr(
    template: &ExprTString,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let mut virtual_source = String::new();
    let mut segments = Vec::new();
    let mut interpolations = Vec::new();
    let mut current_text = String::new();
    let mut current_mappings = Vec::new();
    let mut segment_start = TextSize::ZERO;

    for string in &template.value {
        for element in &string.elements {
            match element {
                InterpolatedStringElement::Literal(literal) => {
                    let virtual_start = TextSize::of(&virtual_source);
                    current_text.push_str(&literal.value);
                    virtual_source.push_str(&literal.value);
                    current_mappings.push(HirTemplateStaticMapping {
                        source_range: literal.range,
                        virtual_range: TextRange::new(virtual_start, TextSize::of(&virtual_source)),
                    });
                }
                InterpolatedStringElement::Interpolation(interpolation) => {
                    let segment_end = TextSize::of(&virtual_source);
                    segments.push(HirTemplateSegment {
                        value: std::mem::take(&mut current_text),
                        mappings: std::mem::take(&mut current_mappings),
                        virtual_range: TextRange::new(segment_start, segment_end),
                    });

                    let virtual_start = segment_end;
                    virtual_source.push(VIRTUAL_HOLE);
                    let virtual_end = TextSize::of(&virtual_source);
                    let value = lower_expr(&interpolation.expression, ctx)?;
                    if is_poisoned_binding_expr(&value, ctx) {
                        return None;
                    }
                    let value_type = value.ty().clone();
                    let clone_from_borrow = borrowed_name_requires_clone(&value, ctx);
                    interpolations.push(HirTemplateInterpolation {
                        value: Box::new(value),
                        clone_from_borrow,
                        value_type,
                        source_range: interpolation.range,
                        expression_range: interpolation.expression.range(),
                        expression_source: interpolation_source(interpolation, ctx),
                        virtual_range: TextRange::new(virtual_start, virtual_end),
                        conversion: interpolation.conversion.to_char(),
                        format_spec: match interpolation.format_spec.as_deref() {
                            Some(spec) => Some(lower_format_spec(spec, ctx)?),
                            None => None,
                        },
                    });
                    segment_start = virtual_end;
                }
            }
        }
    }

    let segment_end = TextSize::of(&virtual_source);
    segments.push(HirTemplateSegment {
        value: current_text,
        mappings: current_mappings,
        virtual_range: TextRange::new(segment_start, segment_end),
    });
    debug_assert_eq!(segments.len(), interpolations.len() + 1);

    let hole_types = interpolations
        .iter()
        .map(|interpolation| interpolation.value_type.clone())
        .collect::<Vec<_>>();
    Some(HirExpr::TemplateString(HirTemplateString {
        source_range: template.range,
        virtual_source,
        segments,
        interpolations,
        ty: Type::Template(hole_types),
    }))
}

fn lower_format_spec(
    spec: &InterpolatedStringFormatSpec,
    ctx: &mut LowerCtx,
) -> Option<HirTemplateFormatSpec> {
    let mut parts = Vec::new();
    for element in &spec.elements {
        match element {
            InterpolatedStringElement::Literal(literal) => {
                parts.push(HirTemplateFormatSpecPart::Literal(
                    literal.value.to_string(),
                ));
            }
            InterpolatedStringElement::Interpolation(interpolation) => {
                let value = lower_expr(&interpolation.expression, ctx)?;
                if is_poisoned_binding_expr(&value, ctx) {
                    return None;
                }
                if !supports_print_formatting(value.ty()) {
                    ctx.error_with_code_at(
                        sifr_diagnostics::DiagnosticCode::TYPE_MISMATCH,
                        format!(
                            "template format-spec value type '{}' cannot be formatted",
                            value.ty().display_name()
                        ),
                        interpolation.expression.range(),
                    );
                    return None;
                }
                parts.push(HirTemplateFormatSpecPart::Interpolation {
                    clone_from_borrow: borrowed_name_requires_clone(&value, ctx),
                    value: Box::new(value),
                    source_range: interpolation.range,
                    conversion: interpolation.conversion.to_char(),
                    format_spec: match interpolation.format_spec.as_deref() {
                        Some(nested) => Some(Box::new(lower_format_spec(nested, ctx)?)),
                        None => None,
                    },
                });
            }
        }
    }
    Some(HirTemplateFormatSpec {
        range: spec.range,
        parts,
    })
}

fn borrowed_name_requires_clone(value: &HirExpr, ctx: &LowerCtx) -> bool {
    matches!(value, HirExpr::Name { name, ty, .. }
        if ctx.borrowed_params.contains(name)
            && ty.ownership() == sifr_type_system::OwnershipKind::Move)
}

fn interpolation_source(
    interpolation: &sifr_python_ast::InterpolatedElement,
    ctx: &LowerCtx,
) -> String {
    let range = interpolation.expression.range();
    if let Some(source) = ctx.source_text.as_deref()
        && let Some(value) = source.get(range.start().to_usize()..range.end().to_usize())
    {
        return value.to_string();
    }
    interpolation
        .debug_text
        .as_ref()
        .map_or_else(String::new, |debug| debug.expression().to_string())
}
