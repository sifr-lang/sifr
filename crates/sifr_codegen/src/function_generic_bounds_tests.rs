use super::*;
use sifr_ir::{HirParam, MethodKind};
use sifr_type_system::ParamConvention;

fn parameter(name: &str, ty: Type) -> HirParam {
    HirParam {
        name: name.to_string(),
        ty,
        default: None,
        keyword_only: false,
        convention: ParamConvention::own(),
    }
}

fn function(
    name: &str,
    type_param: &str,
    params: Vec<HirParam>,
    return_type: Type,
    body: Vec<HirStmt>,
) -> HirFunction {
    function_with_type_params(
        name,
        vec![type_param.to_string()],
        params,
        return_type,
        body,
    )
}

fn function_with_type_params(
    name: &str,
    type_params: Vec<String>,
    params: Vec<HirParam>,
    return_type: Type,
    body: Vec<HirStmt>,
) -> HirFunction {
    HirFunction {
        name: name.to_string(),
        params,
        return_type,
        body,
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params,
    }
}

fn name(name: &str, ty: Type) -> HirExpr {
    HirExpr::Name {
        name: name.to_string(),
        binding_id: None,
        ty,
    }
}

fn call(function: &str, args: Vec<HirExpr>, ty: Type) -> HirStmt {
    HirStmt::Expr {
        expr: HirExpr::Call {
            func: function.to_string(),
            args,
            mutable_arg_places: Vec::new(),
            ty,
        },
    }
}

fn module(
    functions: Vec<HirFunction>,
    type_param_bounds: HashMap<String, HashMap<String, Vec<String>>>,
) -> HirModule {
    HirModule {
        functions,
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds,
    }
}

#[test]
fn source_protocol_bounds_survive_without_local_operator_use() {
    let passthrough = function(
        "passthrough",
        "T",
        vec![parameter("value", Type::TypeVar("T".to_string()))],
        Type::TypeVar("T".to_string()),
        vec![HirStmt::Return {
            value: Some(name("value", Type::TypeVar("T".to_string()))),
        }],
    );
    let bounds = HashMap::from([(
        "passthrough".to_string(),
        HashMap::from([("T".to_string(), vec!["Comparable".to_string()])]),
    )]);

    let closed = RustEmitter::closed_function_type_param_bounds(&module(vec![passthrough], bounds));

    assert!(
        closed["passthrough"]["T"]
            .contains(&FunctionTypeParamBound::Trait("PartialOrd".to_string()))
    );
}

#[test]
fn generic_call_graph_closes_operator_display_and_hash_requirements() {
    let compared = Type::TypeVar("Compared".to_string());
    let compare = function(
        "compare",
        "Compared",
        vec![
            parameter("left", compared.clone()),
            parameter("right", compared.clone()),
        ],
        Type::Bool,
        vec![HirStmt::Return {
            value: Some(HirExpr::Compare {
                left: Box::new(name("left", compared.clone())),
                ops: vec!["<".to_string()],
                comparators: vec![name("right", compared.clone())],
                ty: Type::Bool,
            }),
        }],
    );
    let displayed = Type::TypeVar("Displayed".to_string());
    let display = function(
        "display",
        "Displayed",
        vec![parameter("value", displayed.clone())],
        Type::None,
        vec![call("print", vec![name("value", displayed)], Type::None)],
    );
    let keyed = Type::TypeVar("Key".to_string());
    let key_map = Type::Dict(Box::new(keyed), Box::new(Type::Int));
    let hash = function(
        "hash",
        "Key",
        vec![parameter("values", key_map)],
        Type::None,
        vec![HirStmt::Pass],
    );
    let forwarded = Type::TypeVar("Forwarded".to_string());
    let forwarded_map = Type::Dict(Box::new(forwarded.clone()), Box::new(Type::Int));
    let forward = function(
        "forward",
        "Forwarded",
        vec![
            parameter("value", forwarded.clone()),
            parameter("values", forwarded_map.clone()),
        ],
        Type::None,
        vec![
            call(
                "compare",
                vec![
                    name("value", forwarded.clone()),
                    name("value", forwarded.clone()),
                ],
                Type::Bool,
            ),
            call(
                "display",
                vec![name("value", forwarded.clone())],
                Type::None,
            ),
            call("hash", vec![name("values", forwarded_map)], Type::None),
        ],
    );
    let outer = Type::TypeVar("Outer".to_string());
    let outer_map = Type::Dict(Box::new(outer.clone()), Box::new(Type::Int));
    let outer_forward = function(
        "outer_forward",
        "Outer",
        vec![
            parameter("value", outer.clone()),
            parameter("values", outer_map.clone()),
        ],
        Type::None,
        vec![call(
            "forward",
            vec![name("value", outer), name("values", outer_map)],
            Type::None,
        )],
    );

    let closed = RustEmitter::closed_function_type_param_bounds(&module(
        vec![compare, display, hash, forward, outer_forward],
        HashMap::new(),
    ));
    let outer_bounds = closed["outer_forward"]["Outer"]
        .iter()
        .map(|bound| bound.render_for("Outer"))
        .collect::<HashSet<_>>();

    for expected in ["PartialOrd", "std::fmt::Display", "std::hash::Hash", "Eq"] {
        assert!(outer_bounds.contains(expected), "missing {expected}");
    }
}

#[test]
fn generic_call_graph_renders_output_traits_for_the_receiving_parameter() {
    let callee_type = Type::TypeVar("T".to_string());
    let add_same = function(
        "add_same",
        "T",
        vec![
            parameter("left", callee_type.clone()),
            parameter("right", callee_type.clone()),
        ],
        callee_type.clone(),
        vec![HirStmt::Return {
            value: Some(HirExpr::BinOp {
                left: Box::new(name("left", callee_type.clone())),
                op: "+".to_string(),
                right: Box::new(name("right", callee_type.clone())),
                ty: callee_type,
            }),
        }],
    );
    let caller_type = Type::TypeVar("U".to_string());
    let relay_add = function(
        "relay_add",
        "U",
        vec![
            parameter("left", caller_type.clone()),
            parameter("right", caller_type.clone()),
        ],
        caller_type.clone(),
        vec![HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "add_same".to_string(),
                args: vec![
                    name("left", caller_type.clone()),
                    name("right", caller_type.clone()),
                ],
                mutable_arg_places: Vec::new(),
                ty: caller_type,
            }),
        }],
    );
    let bounds = HashMap::from([
        (
            "add_same".to_string(),
            HashMap::from([("T".to_string(), vec!["Addable".to_string()])]),
        ),
        (
            "relay_add".to_string(),
            HashMap::from([("U".to_string(), vec!["Addable".to_string()])]),
        ),
    ]);

    let closed =
        RustEmitter::closed_function_type_param_bounds(&module(vec![add_same, relay_add], bounds));
    let rendered = closed["relay_add"]["U"]
        .iter()
        .map(|bound| bound.render_for("U"))
        .collect::<HashSet<_>>();

    assert!(rendered.contains("__SifrAdd"));
    assert!(rendered.iter().all(|bound| !bound.contains("Output = T")));
}

#[test]
fn called_nested_function_propagates_captured_generic_demands() {
    let displayed = Type::TypeVar("Displayed".to_string());
    let display = function(
        "display",
        "Displayed",
        vec![parameter("value", displayed.clone())],
        Type::None,
        vec![call("print", vec![name("value", displayed)], Type::None)],
    );
    let outer_type = Type::TypeVar("Outer".to_string());
    let nested = function_with_type_params(
        "nested",
        Vec::new(),
        Vec::new(),
        Type::None,
        vec![call(
            "display",
            vec![name("value", outer_type.clone())],
            Type::None,
        )],
    );
    let outer = function(
        "outer",
        "Outer",
        vec![parameter("value", outer_type)],
        Type::None,
        vec![
            HirStmt::NestedFunction {
                func: nested,
                move_captures: false,
                capture_clones: Vec::new(),
            },
            call("nested", Vec::new(), Type::None),
        ],
    );

    let closed = RustEmitter::closed_function_type_param_bounds(&module(
        vec![display, outer],
        HashMap::new(),
    ));

    assert!(
        closed["outer"]["Outer"].contains(&FunctionTypeParamBound::Trait(
            "std::fmt::Display".to_string()
        ))
    );
}

#[test]
fn uncalled_or_shadowing_nested_functions_do_not_leak_module_demands() {
    let displayed = Type::TypeVar("Displayed".to_string());
    let display = function(
        "display",
        "Displayed",
        vec![parameter("value", displayed.clone())],
        Type::None,
        vec![call("print", vec![name("value", displayed)], Type::None)],
    );
    let outer_type = Type::TypeVar("Outer".to_string());
    let uncalled = function_with_type_params(
        "uncalled",
        Vec::new(),
        Vec::new(),
        Type::None,
        vec![call(
            "display",
            vec![name("value", outer_type.clone())],
            Type::None,
        )],
    );
    let shadowing_display = function_with_type_params(
        "display",
        Vec::new(),
        vec![parameter("value", outer_type.clone())],
        Type::None,
        vec![HirStmt::Pass],
    );
    let outer = function(
        "outer",
        "Outer",
        vec![parameter("value", outer_type.clone())],
        Type::None,
        vec![
            HirStmt::NestedFunction {
                func: uncalled,
                move_captures: false,
                capture_clones: Vec::new(),
            },
            HirStmt::NestedFunction {
                func: shadowing_display,
                move_captures: false,
                capture_clones: Vec::new(),
            },
            call("display", vec![name("value", outer_type)], Type::None),
        ],
    );

    let closed = RustEmitter::closed_function_type_param_bounds(&module(
        vec![display, outer],
        HashMap::new(),
    ));

    assert!(closed["outer"]["Outer"].is_empty());
}

#[test]
fn structural_correspondence_does_not_overconstrain_sibling_parameters() {
    let callee_type = Type::TypeVar("T".to_string());
    let relay = function(
        "relay",
        "T",
        vec![parameter(
            "value",
            Type::Tuple(vec![callee_type, Type::Int]),
        )],
        Type::None,
        vec![HirStmt::Pass],
    );
    let left = Type::TypeVar("Left".to_string());
    let right = Type::TypeVar("Right".to_string());
    let caller = function_with_type_params(
        "caller",
        vec!["Left".to_string(), "Right".to_string()],
        vec![parameter(
            "value",
            Type::Tuple(vec![left.clone(), right.clone()]),
        )],
        Type::None,
        vec![call(
            "relay",
            vec![name("value", Type::Tuple(vec![left, right]))],
            Type::None,
        )],
    );
    let bounds = HashMap::from([(
        "relay".to_string(),
        HashMap::from([("T".to_string(), vec!["Comparable".to_string()])]),
    )]);

    let closed =
        RustEmitter::closed_function_type_param_bounds(&module(vec![relay, caller], bounds));

    assert!(
        closed["caller"]["Left"].contains(&FunctionTypeParamBound::Trait("PartialOrd".to_string()))
    );
    assert!(closed["caller"]["Right"].is_empty());
}

#[test]
fn same_basename_functions_in_distinct_modules_have_distinct_closure_identity() {
    let left_type = Type::TypeVar("Left".to_string());
    let left = function(
        "transform",
        "Left",
        vec![parameter("value", left_type.clone())],
        left_type.clone(),
        vec![HirStmt::Return {
            value: Some(HirExpr::BinOp {
                left: Box::new(name("value", left_type.clone())),
                op: "+".to_string(),
                right: Box::new(name("value", left_type.clone())),
                ty: left_type,
            }),
        }],
    );
    let right_type = Type::TypeVar("Right".to_string());
    let right = function(
        "transform",
        "Right",
        vec![parameter("value", right_type.clone())],
        right_type.clone(),
        vec![HirStmt::Return {
            value: Some(name("value", right_type)),
        }],
    );

    let left_closed =
        RustEmitter::closed_function_type_param_bounds(&module(vec![left], HashMap::new()));
    let right_closed =
        RustEmitter::closed_function_type_param_bounds(&module(vec![right], HashMap::new()));

    assert!(
        left_closed["transform"]["Left"]
            .contains(&FunctionTypeParamBound::Trait("__SifrAdd".to_string()))
    );
    assert!(right_closed["transform"]["Right"].is_empty());
}
