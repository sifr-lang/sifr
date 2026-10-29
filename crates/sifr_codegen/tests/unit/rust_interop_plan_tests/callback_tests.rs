use super::*;

#[test]
fn interop_bridge_distinguishes_call_scoped_and_threadsafe_callbacks() {
    let mut function = HirFunction {
        name: "subscribe".to_string(),
        params: vec![HirParam {
            name: "callback".to_string(),
            ty: Type::Callable(
                vec![Type::Int],
                vec![ParamConvention::borrow()],
                Box::new(Type::None),
            ),
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::None,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            "bridge.events.subscribe",
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let module_without_callback = module_with(vec![function.clone()], Vec::new());
    let plan_without_callback =
        interop_build_plan_for_named_modules([(Some("main"), &module_without_callback)]);

    let call_scoped = &plan_without_callback.rust.bridge_contracts.signatures[0].params[0].ty;
    assert_eq!(call_scoped.kind, RustBridgeTypeKind::CallScopedCallback);
    assert_eq!(
        call_scoped.rust_borrowed_type.as_deref(),
        Some(
            "::sifr_runtime::interop::CallScopedCallbackBridge<'_, \
             (::sifr_runtime::interop::SifrIntBridge,), ()>"
        )
    );

    function.rust_interop.push(callback_declaration());
    let module_with_callback = module_with(vec![function], Vec::new());
    let plan_with_callback =
        interop_build_plan_for_named_modules([(Some("main"), &module_with_callback)]);
    let signature = &plan_with_callback.rust.bridge_contracts.signatures[0];

    assert_eq!(plan_with_callback.rust.bridge_contracts.signatures.len(), 1);
    assert_eq!(signature.params[0].ty.kind, RustBridgeTypeKind::Callback);
    assert_eq!(
        signature.params[0].ty.rust_borrowed_type.as_deref(),
        Some(
            "&::sifr_runtime::interop::ThreadsafeCallbackBridge<\
             (::sifr_runtime::interop::SifrIntBridge,), ()>"
        )
    );
    assert_eq!(
        signature.params[0].ty.rust_owned_type.as_deref(),
        Some(
            "::sifr_runtime::interop::ThreadsafeCallbackBridge<\
             (::sifr_runtime::interop::SifrIntBridge,), ()>"
        )
    );
}

fn callback_declaration() -> RustInteropDeclaration {
    RustInteropDeclaration {
        kind: RustInteropDecoratorKind::Callback,
        target: None,
        arguments: Vec::new(),
        span: Default::default(),
        effect: RustInteropEffect::Sync,
        abi_requirements: RustInteropAbiRequirements::default(),
        consumes_receiver: false,
    }
}
