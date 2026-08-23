use sifr_codegen::RustInteropPlanDeclaration;
use sifr_ir::{RustInteropValue, RustTargetPath};

const MAPPER_ASSERTION: &str = "__sifr_assert_panic_mapper";

pub(super) struct PanicMapperProbe {
    pub(super) assertion: &'static str,
    pub(super) invocation: String,
}

pub(super) fn panic_mapper_probe(
    declaration: &RustInteropPlanDeclaration,
    path: &RustTargetPath,
) -> Option<PanicMapperProbe> {
    let target = map_error_target(declaration)?;
    if target.span != path.span || target.segments != path.segments {
        return None;
    }
    Some(PanicMapperProbe {
        assertion: "fn __sifr_assert_panic_mapper<__SifrMappedError: std::fmt::Display>(\n    _f: fn(sifr_runtime::interop::RustPanicErrorBridge) -> __SifrMappedError,\n) {}\n",
        invocation: format!("{MAPPER_ASSERTION}({});", target.segments.join("::")),
    })
}

pub(super) fn stderr_reports_invalid_panic_mapper(stderr: &str) -> bool {
    stderr.split("\nerror").any(|error| {
        if !error.contains(MAPPER_ASSERTION) {
            return false;
        }
        let function_shape_mismatch =
            error.contains("expected fn pointer") || error.contains("mismatched types");
        let display_bound_mismatch = error.contains("std::fmt::Display")
            && (error.contains("trait bound") || error.contains("not implemented"));
        function_shape_mismatch || display_bound_mismatch
    })
}

fn map_error_target(declaration: &RustInteropPlanDeclaration) -> Option<&RustTargetPath> {
    declaration
        .declaration
        .arguments
        .iter()
        .find(|argument| argument.name.as_deref() == Some("panic"))
        .and_then(|argument| match &argument.value {
            RustInteropValue::PolicyCall { name, argument, .. } if name == "map_error" => {
                match argument.as_ref() {
                    RustInteropValue::TargetPath(target) => Some(target),
                    _ => None,
                }
            }
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruff_text_size::TextRange;
    use sifr_codegen::RustInteropOwner;
    use sifr_ir::{
        RustInteropAbiRequirements, RustInteropArgument, RustInteropDeclaration,
        RustInteropDecoratorKind, RustInteropEffect,
    };

    #[test]
    fn probe_requires_redacted_panic_bridge_input_and_display_error() {
        let declaration = RustInteropPlanDeclaration {
            module_name: None,
            owner: RustInteropOwner::Function {
                name: "may_panic".to_string(),
            },
            declaration: RustInteropDeclaration {
                kind: RustInteropDecoratorKind::Function,
                target: None,
                arguments: vec![RustInteropArgument {
                    name: Some("panic".to_string()),
                    value: RustInteropValue::PolicyCall {
                        name: "map_error".to_string(),
                        argument: Box::new(RustInteropValue::TargetPath(RustTargetPath {
                            segments: vec![
                                "bridge".to_string(),
                                "wrapper".to_string(),
                                "map_panic".to_string(),
                            ],
                            span: TextRange::default(),
                        })),
                        span: TextRange::default(),
                    },
                    span: TextRange::default(),
                }],
                span: TextRange::default(),
                effect: RustInteropEffect::Sync,
                abi_requirements: RustInteropAbiRequirements::default(),
                consumes_receiver: false,
            },
        };

        let path = map_error_target(&declaration)
            .expect("mapper target")
            .clone();
        let probe = panic_mapper_probe(&declaration, &path).expect("mapper probe");

        assert!(
            probe
                .assertion
                .contains("fn(sifr_runtime::interop::RustPanicErrorBridge)")
        );
        assert!(probe.assertion.contains("std::fmt::Display"));
        assert_eq!(
            probe.invocation,
            "__sifr_assert_panic_mapper(bridge::wrapper::map_panic);"
        );
    }

    #[test]
    fn stderr_classifier_requires_a_mapper_shape_or_display_failure() {
        assert!(stderr_reports_invalid_panic_mapper(
            "error[E0308]: mismatched types\n__sifr_assert_panic_mapper(bridge::map_panic)\nexpected fn pointer"
        ));
        assert!(stderr_reports_invalid_panic_mapper(
            "error[E0277]: the trait bound `Mapped: std::fmt::Display` is not implemented\n__sifr_assert_panic_mapper(bridge::map_panic)"
        ));
        assert!(!stderr_reports_invalid_panic_mapper(
            "error[E0425]: cannot find value `missing` in module `bridge`\n__sifr_assert_panic_mapper(bridge::missing)"
        ));
        assert!(!stderr_reports_invalid_panic_mapper(
            "error: unrelated bridge source failure\n__sifr_assert_panic_mapper(bridge::map_panic)"
        ));
        assert!(!stderr_reports_invalid_panic_mapper(
            "error[E0425]: cannot find value `missing` in module `bridge`\n__sifr_assert_panic_mapper(bridge::missing)\n\nerror[E0308]: mismatched types\nunrelated source expression\nexpected fn pointer"
        ));
    }
}
