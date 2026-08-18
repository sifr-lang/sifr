use super::*;

#[test]
fn interop_bridge_uses_package_structural_mapping_for_native_values() {
    let mapped_ty = Type::Class {
        identity: Some("values.Token".to_string()),
        type_args: Vec::new(),
        name: "Token".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    };
    let mut mapped_class = class("Token", HirClassKind::Regular, Vec::new());
    mapped_class.identity = Some("values.Token".to_string());
    let mut opaque = opaque_declaration("bridge.token.Token");
    opaque.arguments.push(RustInteropArgument {
        name: Some("structural".to_string()),
        value: RustInteropValue::TargetPath(RustTargetPath {
            segments: ["bridge", "token", "TokenMapping"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            span: Default::default(),
        }),
        span: Default::default(),
    });
    mapped_class.rust_interop = vec![opaque];
    let declarations = module_with(Vec::new(), vec![mapped_class]);
    let consumer = module_with(
        vec![HirFunction {
            name: "identity".to_string(),
            params: vec![HirParam {
                name: "value".to_string(),
                ty: mapped_ty.clone(),
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            }],
            return_type: mapped_ty,
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                "bridge.token.identity",
            )],
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }],
        Vec::new(),
    );

    let plan = interop_build_plan_for_named_modules([
        (Some("values"), &declarations),
        (Some("main"), &consumer),
    ]);
    let signature = &plan.rust.bridge_contracts.signatures[0];
    let mapped = "::sifr_runtime::interop::structural::MappedValue<bridge::token::Token, bridge::token::TokenMapping>";
    let borrowed_mapped = format!("&{mapped}");

    assert_eq!(
        signature.params[0].ty.rust_borrowed_type.as_deref(),
        Some(borrowed_mapped.as_str())
    );
    assert_eq!(
        signature.return_type.rust_return_type.as_deref(),
        Some(mapped)
    );
}
