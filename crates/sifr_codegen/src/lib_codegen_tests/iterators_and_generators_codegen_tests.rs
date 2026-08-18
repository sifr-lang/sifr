use super::*;

fn test_error_type(name: &str) -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: name.to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}
use sifr_ir::CompilerIntrinsicId;
#[test]
fn test_generate_rust_while_else_with_borrowed_condition_uses_broke_marker() {
    let list_ty = Type::List(Box::new(Type::Int));
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "iterate".to_string(),
                params: vec![HirParam {
                    name: "xs".to_string(),
                    ty: list_ty.clone(),
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                }],
                return_type: Type::None,
                body: vec![HirStmt::While {
                    condition: HirExpr::Name {
                        name: "xs".to_string(),
                        binding_id: None,
                        ty: list_ty.clone(),
                    },
                    body: vec![HirStmt::Break],
                    else_body: Some(vec![HirStmt::Expr {
                        expr: HirExpr::Call {
                            mutable_arg_places: Vec::new(),
                            func: "print".to_string(),
                            args: vec![HirExpr::StringLiteral("empty".to_string())],
                            ty: Type::None,
                        },
                    }]),
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
            HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        mutable_arg_places: Vec::new(),
                        func: "iterate".to_string(),
                        args: vec![HirExpr::ListLiteral {
                            elements: vec![],
                            ty: list_ty.clone(),
                        }],
                        ty: Type::None,
                    },
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
        ],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("fn iterate(xs: &Vec<i64>)"));
    assert!(rust_code.contains("let mut _broke: bool = false;"));
    assert!(rust_code.contains("_broke = true;"));
    assert!(rust_code.contains("if !(_broke) {"));
}

#[test]
fn test_generate_rust_generator_try_except_materializes_without_shape_panic() {
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "gen".to_string(),
                params: vec![],
                return_type: Type::Iterator(Box::new(Type::Int)),
                body: vec![HirStmt::TryExcept {
                    body: vec![HirStmt::Yield {
                        value: HirExpr::IntLiteral(1),
                    }],
                    handlers: vec![HirExceptHandler {
                        error_type: Some("Error".to_string()),
                        error_resolved_type: None,
                        name: Some("e".to_string()),
                        body: vec![HirStmt::Yield {
                            value: HirExpr::IntLiteral(2),
                        }],
                    }],
                    body_error_types: vec![test_error_type("Error")],
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
            HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::For {
                    target: "v".to_string(),
                    target_ty: Type::Int,
                    iter: HirExpr::Call {
                        mutable_arg_places: Vec::new(),
                        func: "gen".to_string(),
                        args: vec![],
                        ty: Type::Iterator(Box::new(Type::Int)),
                    },
                    body: vec![HirStmt::Expr {
                        expr: HirExpr::Call {
                            mutable_arg_places: Vec::new(),
                            func: "print".to_string(),
                            args: vec![HirExpr::Name {
                                name: "v".to_string(),
                                binding_id: None,
                                ty: Type::Int,
                            }],
                            ty: Type::None,
                        },
                    }],
                    else_body: None,
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
        ],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("if !__sifr_generator_initialized {"));
    assert!(rust_code.contains("_yields.push(1_i64);"));
    assert!(rust_code.contains("_yields.push(2_i64);"));
    assert!(rust_code.contains("__sifr_generator_iter.next()"));
}

#[test]
fn test_generate_rust_generator_conditional_yield_preserves_else_branch() {
    let rust_code = generate_rust_from_source(
        "def gen() -> Iterator[int]:\n    i: int = 0\n    while i < 4:\n        if i < 3:\n            yield i\n            i = i + 1\n        else:\n            i = i + 1\n\ndef main():\n    g: Iterator[int] = gen()\n    print(next(g))\n",
    );

    let cond_idx = rust_code
        .find("if i < (3_i64) {")
        .expect("generator conditional branch should be emitted");
    let cond_region = &rust_code[cond_idx..];

    assert!(cond_region.contains("_yields.push(i);"));
    assert!(
        cond_region.contains("} else {"),
        "generator else branch should be preserved"
    );
    assert!(
        cond_region.contains("i += 1_i64;"),
        "generator else branch body should be preserved"
    );
}

#[test]
fn test_generate_rust_generator_expression_without_filter_lowers_to_map_chain() {
    let rust_code = generate_rust_from_source(
        "def main():\n    xs: list[int] = [1, 2, 3]\n    squares: Iterator[int] = (x * x for x in xs)\n    print(list(squares))\n",
    );

    assert!(rust_code.contains("let squares: Box<dyn Iterator<Item = i64>>"));
    assert!(rust_code.contains("iter().copied().map(|x| x * x)"));
}

#[test]
fn test_generate_rust_filter_over_list_lowers_to_lazy_boxed_iterator() {
    let rust_code = generate_rust_from_source(
        "def main():\n    nums: list[int] = [1, 2, 3, 4]\n    evens: Iterator[int] = filter(lambda x: x % 2 == 0, nums)\n    print(list(evens))\n",
    );

    assert!(rust_code.contains("let evens: Box<dyn Iterator<Item = i64>>"));
    assert!(rust_code.contains("Box::new("));
    assert!(rust_code.contains(".iter().copied().filter("));
}

#[test]
fn test_generate_rust_iterable_binding_from_iterator_materializes_once() {
    let rust_code = generate_rust_from_source(
        "def main():\n    base: list[int] = [1, 2, 3]\n    it: Iterator[int] = iter(base)\n    xs: Iterable[int] = it\n    print(list(xs))\n",
    );

    assert!(rust_code.contains("let xs: Vec<i64> = it.collect::<Vec<_>>();"));
}

#[test]
fn test_generate_rust_iterable_return_from_iterator_materializes_for_signature() {
    let rust_code = generate_rust_from_source(
        "def adapt(own it: Iterator[int]) -> Iterable[int]:\n    return it\n\ndef main():\n    base: list[int] = [1, 2]\n    it: Iterator[int] = iter(base)\n    xs: Iterable[int] = adapt(it)\n    print(list(xs))\n",
    );

    assert!(rust_code.contains("fn adapt(it: Box<dyn Iterator<Item = i64>>) -> Vec<i64> {"));
    assert!(rust_code.contains("it.collect::<Vec<_>>()"));
}

#[test]
fn test_generate_rust_iterator_return_consumes_local_list_binding() {
    let rust_code = generate_rust_from_source(
        "def build() -> Iterator[int]:\n    result: list[int] = [1, 2, 3]\n    return iter(result)\n\ndef main():\n    print(list(build()))\n",
    );

    assert!(rust_code.contains("fn build() -> Box<dyn Iterator<Item = i64>> {"));
    assert!(rust_code.contains("Box::new(result.into_iter())"));
}

#[test]
fn test_generate_rust_iterator_return_consumes_owned_param_binding() {
    let rust_code = generate_rust_from_source(
        "def adapt(own items: list[int]) -> Iterator[int]:\n    return iter(items)\n\ndef main():\n    print(list(adapt([1, 2, 3])))\n",
    );

    assert!(rust_code.contains("fn adapt(items: Vec<i64>) -> Box<dyn Iterator<Item = i64>> {"));
    assert!(rust_code.contains("Box::new(items.into_iter())"));
}

#[test]
fn test_generate_rust_open_uses_canonical_filehandle_constructor() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::IntrinsicCall {
                    intrinsic: CompilerIntrinsicId::OpenText,
                    args: vec![
                        HirExpr::StringLiteral("/tmp/sifr_codegen_open.txt".to_string()),
                        HirExpr::StringLiteral("w".to_string()),
                        HirExpr::StringLiteral("utf-8".to_string()),
                        HirExpr::StringLiteral("strict".to_string()),
                    ],
                    ty: Type::Unknown,
                    call_range: Default::default(),
                    arg_ranges: vec![Default::default(); 4],
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };
    let rust_code = generate_rust_with_metadata(&module).rust_source;

    assert!(rust_code.contains("::sifr_stdlib::fs::open_file"));
    assert!(rust_code.contains("TextFileHandle::new("));
    assert!(rust_code.contains("BinaryFileHandle::new("));
    for (class_name, argument) in [
        ("Encoding", "__encoding"),
        ("DecodeErrorHandler", "__errors.clone()"),
        ("EncodeErrorHandler", "__errors"),
    ] {
        let rust_name = sifr_type_system::stdlib_class_rust_name("sifr.encoding", class_name);
        assert!(rust_code.contains(&format!("{rust_name}::new({argument})")));
    }
    assert!(!rust_code
        .contains("return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });"));
}

#[test]
fn test_generate_rust_generator_clones_borrowed_params_into_owned_locals_before_calls() {
    let rust_code = generate_rust_from_source(
        "def glob(directory: str, pattern: str) -> list[str]:\n    return []\n\ndef iglob(directory: str, pattern: str) -> Iterator[str]:\n    matches: list[str] = glob(directory, pattern)\n    i: int = 0\n    while i < len(matches):\n        yield matches[i]\n        i = i + 1\n",
    );

    assert!(rust_code.contains("let directory = directory.clone();"));
    assert!(rust_code.contains("let pattern = pattern.clone();"));
    assert!(rust_code.contains("let matches: Vec<String> = glob(&directory, &pattern);"));
}

#[test]
fn test_generate_rust_recursive_constructor_argument_wraps_optional_box_field() {
    let rust_code = generate_rust_from_source(
        "class Entry:\n    value: int\n    next: Entry | None\n\n    def __init__(self, value: int = 0, next: Entry | None = None):\n        self.value = value\n        self.next = next\n\ndef main():\n    long = Entry(4, Entry(5, Entry(6)))\n    print(long.value)\n",
    );

    assert!(rust_code.contains(
        "let long: Entry = Entry::new(4_i64, Some(Box::new(Entry::new(5_i64, Some(Box::new(Entry::new(6_i64, None)))))));"
    ));
}

#[test]
fn test_generate_rust_defaultdict_int_augassign_uses_entry_default() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef main():\n    counts = defaultdict(int)\n    counts[\"steps\"] += 1\n    counts[\"steps\"] += 2\n    assert counts[\"steps\"] == 3\n",
    );

    assert!(rust_code.contains("let __elem = counts.entry(\"steps\".to_string()).or_insert(0);"));
    assert!(rust_code.contains("*__elem += 1_i64;"));
    assert!(rust_code.contains("*__elem += 2_i64;"));
}

#[test]
fn test_generate_rust_defaultdict_list_len_borrows_string_literal_key() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef main():\n    groups = defaultdict(list)\n    groups[\"hit\"].append(\"hot\")\n    assert len(groups[\"hit\"]) == 1\n",
    );

    assert!(
        rust_code.contains("groups.get(\"hit\").map_or"),
        "{rust_code}"
    );
    assert!(
        !rust_code.contains("groups.get(\"hit\".to_string())"),
        "{rust_code}"
    );
    assert!(
        !rust_code.contains("groups.get(&\"hit\".to_string())"),
        "{rust_code}"
    );
}

#[test]
fn test_generate_rust_nested_string_function_closure_params_are_typed() {
    let rust_code = generate_rust_from_source(
        "def main():\n    def greet(name: str) -> str:\n        return \"Hello, \" + name\n    print(greet(\"Sifr\"))\n",
    );

    assert!(
        rust_code.contains("let greet = |name: &String|"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("greet(&\"Sifr\".to_string())"),
        "{rust_code}"
    );
}

#[test]
fn test_generate_rust_tuple_field_assignment_emits_mutable_self_receiver() {
    let rust_code = generate_rust_from_source(
        "class RunningBounds:\n    left: int\n    right: int\n\n    def __init__(self, left: int, right: int):\n        self.left = left\n        self.right = right\n\n    def rotate(mut self, next_value: int) -> None:\n        self.left, self.right = self.right, next_value\n",
    );

    assert!(rust_code.contains("fn rotate(&mut self, next_value: i64)"));
}

#[test]
fn test_generate_rust_test_uses_explicit_test_mode_context() {
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Pass],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
            HirFunction {
                name: "test_sample".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Pass],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
            HirFunction {
                name: "helper".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Pass],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
        ],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust_test(&module, "main").rust_source;
    assert!(!rust_code.contains("fn main("));
    assert!(rust_code.contains("#[test]\nfn test_sample()"));
    assert!(rust_code.contains("fn helper()"));
    assert!(!rust_code.contains("#[test]\nfn helper()"));
}

#[test]
fn test_generate_rust_test_collects_imports_from_emitted_code() {
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "test_collections".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Expr {
                        expr: HirExpr::DictLiteral {
                            keys: vec![HirExpr::StringLiteral("k".to_string())],
                            values: vec![HirExpr::IntLiteral(1)],
                            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                        },
                    },
                    HirStmt::Expr {
                        expr: HirExpr::SetLiteral {
                            elements: vec![HirExpr::IntLiteral(1)],
                            ty: Type::Set(Box::new(Type::Int)),
                        },
                    },
                ],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
            HirFunction {
                name: "helper_int".to_string(),
                params: vec![HirParam {
                    name: "x".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
                }],
                return_type: Type::Int,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::Name {
                        name: "x".to_string(),
                        binding_id: None,
                        ty: Type::Int,
                    }),
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            },
        ],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let result = generate_rust_test(&module, "main");
    assert!(result
        .rust_source
        .contains("use ::std::collections::HashMap;"));
    assert!(result
        .rust_source
        .contains("use ::std::collections::HashSet;"));
    assert!(!result.rust_source.contains("use ::sifr_runtime::SifrInt;"));
    assert!(!result
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::NumBigint));
    assert!(!result
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::NumTraits));
}

#[test]
fn test_generate_rust_test_emits_local_module_import_uses() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "test_import".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    mutable_arg_places: Vec::new(),
                    func: "helper".to_string(),
                    args: vec![],
                    ty: Type::Int,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "support".to_string(),
            names: vec!["helper".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_test(&module, "main");
    assert!(
        generated
            .rust_source
            .contains("use crate::support::helper;"),
        "test codegen should emit local module uses for imported names"
    );
}

#[test]
fn test_checked_field_mutation_is_explicit_and_non_sticky() {
    let items_ty = Type::List(Box::new(Type::Int));
    let table_ty = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
    let label_ty = Type::Str;
    let class_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Bucket".to_string(),
        fields: vec![
            ("items".to_string(), items_ty.clone()),
            ("table".to_string(), table_ty.clone()),
            ("label".to_string(), label_ty.clone()),
        ],
        methods: vec![],
        parent_class: None,
    };

    let module = HirModule {
        functions: vec![],
        classes: vec![HirClass {
            name: "Bucket".to_string(),
            identity: None,
            fields: vec![
                ("items".to_string(), items_ty.clone()),
                ("table".to_string(), table_ty.clone()),
                ("label".to_string(), label_ty.clone()),
            ],
            field_defaults: Vec::new(),
            field_default_identities: Vec::new(),
            declaration_metadata: Vec::new(),
            methods: vec![
                HirFunction {
                    name: "append_item".to_string(),
                    params: vec![HirParam {
                        name: "x".to_string(),
                        ty: Type::Int,
                        default: None,
                        keyword_only: false,
                        convention: ParamConvention::own(),
                    }],
                    return_type: Type::None,
                    body: vec![HirStmt::Expr {
                        expr: HirExpr::MethodCall {
                            object: Box::new(HirExpr::FieldAccess {
                                object: Box::new(HirExpr::Name {
                                    name: "self".to_string(),
                                    binding_id: Some(sifr_ir::BindingId(1)),
                                    ty: class_ty.clone(),
                                }),
                                field: "items".to_string(),
                                ty: items_ty.clone(),
                            }),
                            method: "append".to_string(),
                            args: vec![HirExpr::Name {
                                name: "x".to_string(),
                                binding_id: None,
                                ty: Type::Int,
                            }],
                            receiver_convention: Some(
                                sifr_type_system::ReceiverConvention::MutableBorrow,
                            ),
                            receiver_target: Some(sifr_ir::MutableReceiverTarget::Place(
                                sifr_ir::Place {
                                    root: sifr_ir::BindingId(1),
                                    projections: vec![sifr_ir::PlaceProjection::Field(
                                        sifr_ir::FieldIdentity {
                                            declaring_class: "Bucket".to_string(),
                                            field: "items".to_string(),
                                        },
                                    )],
                                },
                            )),
                            mutable_arg_places: vec![None],
                            source: None,
                            ty: Type::None,
                        },
                    }],
                    is_async: false,
                    method_kind: MethodKind::Regular,
                    receiver: Some(sifr_type_system::ReceiverConvention::MutableBorrow),
                    decorators: vec![],
                    rust_interop: Vec::new(),
                    python_interop: Vec::new(),
                    compiler_intrinsic: None,
                    type_params: vec![],
                },
                HirFunction {
                    name: "read_table".to_string(),
                    params: vec![],
                    return_type: Type::Union(vec![Type::Int, Type::None]),
                    body: vec![HirStmt::Return {
                        value: Some(HirExpr::Index {
                            object: Box::new(HirExpr::FieldAccess {
                                object: Box::new(HirExpr::Name {
                                    name: "self".to_string(),
                                    binding_id: None,
                                    ty: class_ty.clone(),
                                }),
                                field: "table".to_string(),
                                ty: table_ty.clone(),
                            }),
                            index: Box::new(HirExpr::StringLiteral("k".to_string())),
                            ty: Type::Union(vec![Type::Int, Type::None]),
                        }),
                    }],
                    is_async: false,
                    method_kind: MethodKind::Regular,
                    receiver: Some(sifr_type_system::ReceiverConvention::SharedBorrow),
                    decorators: vec![],
                    rust_interop: Vec::new(),
                    python_interop: Vec::new(),
                    compiler_intrinsic: None,
                    type_params: vec![],
                },
                HirFunction {
                    name: "leak_guard".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![
                        HirStmt::Let {
                            name: "d".to_string(),
                            ty: table_ty.clone(),
                            value: HirExpr::DictLiteral {
                                keys: vec![HirExpr::StringLiteral("k".to_string())],
                                values: vec![HirExpr::IntLiteral(1)],
                                ty: table_ty.clone(),
                            },
                            is_mutable: false,
                        },
                        HirStmt::Expr {
                            expr: HirExpr::Index {
                                object: Box::new(HirExpr::Name {
                                    name: "d".to_string(),
                                    binding_id: None,
                                    ty: table_ty.clone(),
                                }),
                                index: Box::new(HirExpr::StringLiteral("k".to_string())),
                                ty: Type::Union(vec![Type::Int, Type::None]),
                            },
                        },
                        HirStmt::Return {
                            value: Some(HirExpr::FieldAccess {
                                object: Box::new(HirExpr::Name {
                                    name: "self".to_string(),
                                    binding_id: None,
                                    ty: class_ty,
                                }),
                                field: "label".to_string(),
                                ty: label_ty,
                            }),
                        },
                    ],
                    is_async: false,
                    method_kind: MethodKind::Regular,
                    receiver: Some(sifr_type_system::ReceiverConvention::SharedBorrow),
                    decorators: vec![],
                    rust_interop: Vec::new(),
                    python_interop: Vec::new(),
                    compiler_intrinsic: None,
                    type_params: vec![],
                },
            ],
            is_hashable: false,
            is_error_type: false,
            kind: HirClassKind::Regular,
            operator_impls: vec![],
            newtype_inner: None,
            implements_protocols: vec![],
            parent_class: None,
            parent_type: None,
            type_params: vec![],
            enum_variants: vec![],
            rust_interop: Vec::new(),
        }],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(
        rust_code.contains("self.items.push(x);")
            || rust_code.contains("(self.items).push(x);")
            || rust_code.contains("self.items.extend(")
            || rust_code.contains("(self.items).extend("),
        "{rust_code}"
    );
    assert!(!rust_code.contains("self.items.clone().push(x)"));
    assert!(rust_code.contains("self.table.get(\"k\").copied()"));
    assert!(!rust_code.contains("self.table.clone().get(\"k\")"));
    assert!(rust_code.contains("self.label.clone()"), "{rust_code}");
}

#[test]
fn test_codegen_structured_lowering_applies_to_simple_stmt() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Let {
                name: "x".to_string(),
                ty: Type::Int,
                value: HirExpr::IntLiteral(1),
                is_mutable: false,
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);

    assert!(generated.rust_source.contains("1_i64"));
    assert!(generated.lowering_stats.stmt_structured > 0);
}

#[test]
fn test_structured_aug_assign_uses_string_and_list_methods() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "s".to_string(),
                    ty: Type::Str,
                    value: HirExpr::StringLiteral("Hello".to_string()),
                    is_mutable: true,
                },
                HirStmt::AugAssign {
                    name: "s".to_string(),
                    op: "+=".to_string(),
                    value: HirExpr::StringLiteral("World".to_string()),
                },
                HirStmt::Let {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                    value: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(1)],
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    is_mutable: true,
                },
                HirStmt::AugAssign {
                    name: "items".to_string(),
                    op: "+=".to_string(),
                    value: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(2)],
                        ty: Type::List(Box::new(Type::Int)),
                    },
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("s.push_str("));
    assert!(generated.rust_source.contains("items.extend(vec![2_i64])"));
    assert!(!generated.rust_source.contains("s += "));
    assert!(!generated.rust_source.contains("items += "));
}
