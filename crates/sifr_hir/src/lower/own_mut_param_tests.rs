use crate::{lower_module, HirDiagnostic, HirModule};
use sifr_python_parser::parse_module;
use sifr_type_system::ParamConvention;

fn lower_source(source: &str) -> Result<HirModule, Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|result| result.module)
}

#[test]
fn test_hir_tracks_all_four_parameter_convention_shapes() {
    let module = lower_source(
        "def modes(items: list[int], mut borrowed: list[int], own taken: list[int], own mut transformed: list[int]) -> list[int]:\n    return transformed\n",
    )
    .unwrap();

    let params = &module.functions[0].params;
    assert_eq!(params[0].convention, ParamConvention::borrow());
    assert_eq!(params[1].convention, ParamConvention::mut_borrow());
    assert_eq!(params[2].convention, ParamConvention::own());
    assert_eq!(params[3].convention, ParamConvention::own_mut());
}

#[test]
fn test_hir_normalizes_mut_own_and_own_mut_to_same_convention() {
    let module = lower_source(
        "def take_a(own mut items: list[int]) -> list[int]:\n    return items\n\ndef take_b(mut own items: list[int]) -> list[int]:\n    return items\n",
    )
    .unwrap();

    assert_eq!(
        module.functions[0].params[0].convention,
        ParamConvention::own_mut()
    );
    assert_eq!(
        module.functions[1].params[0].convention,
        ParamConvention::own_mut()
    );
}
