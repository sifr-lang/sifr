use super::*;
use sifr_lowering::lower_module;
use sifr_syntax::parse_module_suite;
use sifr_type_system::FunctionType;

fn describe_type(module_name: &str, ty: &Type, lowering: &LoweringResult) -> StructuralShape {
    describe_type_with_externals(module_name, ty, lowering, &ExternalDefs::default())
}

fn class_type(class: &sifr_lowering::HirClass, type_args: Vec<Type>) -> Type {
    Type::Class {
        identity: None,
        type_args,
        name: class.name.clone(),
        fields: class.fields.clone(),
        methods: class
            .methods
            .iter()
            .chain(class.operator_impls.iter().map(|(_, method)| method))
            .map(|method| {
                (
                    method.name.clone(),
                    FunctionType {
                        receiver: method.receiver,
                        params: method
                            .params
                            .iter()
                            .map(|param| (param.name.clone(), param.ty.clone(), param.convention))
                            .collect(),
                        return_type: Box::new(method.return_type.clone()),
                    },
                )
            })
            .collect(),
        parent_class: None,
    }
}

#[test]
fn class_shape_preserves_defaults_generics_and_recursive_identity() {
    let source = "class Node[T]:\n    value: T\n    label: str = \"root\"\n    next: Node[T] | None = None\n";
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    let lowered = lower_module(&parsed).expect("fixture lowers");
    let class = lowered.module.classes.first().expect("class exists");
    let shape = describe_type(
        "fixture.shapes",
        &class_type(class, vec![Type::Int]),
        &lowered,
    );
    assert!(shape.canonical_identity.contains("fixture.shapes.Node"));
    assert!(shape.canonical_identity.contains("label:str:default"));
    assert!(shape.canonical_identity.contains("ref:fixture.shapes.Node"));
}

#[test]
fn union_description_is_deterministic() {
    let parsed = parse_module_suite("", None).expect("fixture parses");
    let lowered = lower_module(&parsed).expect("fixture lowers");
    let ty = Type::Union(vec![Type::Int, Type::Str]);
    assert_eq!(
        describe_type("fixture", &ty, &lowered),
        describe_type("fixture", &ty, &lowered)
    );
}

#[test]
fn explicit_initializer_does_not_erase_field_default_metadata() {
    let source = "class Config:\n    retries: int = 3\n\n    def __init__(self, retries: int) -> None:\n        self.retries = retries\n";
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    let lowered = lower_module(&parsed).expect("fixture lowers");
    let class = lowered.module.classes.first().expect("class exists");
    let shape = describe_type("fixture.defaults", &class_type(class, Vec::new()), &lowered);
    assert!(shape
        .canonical_identity
        .contains("retries:int:default=int:3"));
}

#[test]
fn adapted_factory_default_is_not_reported_as_required() {
    let source = r#"
class Model:
    tags: list[str]
"#;
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    let mut lowered = lower_module(&parsed).expect("fixture lowers");
    let factory = sifr_lowering::CallableIdentity {
        module: "fixture.runtime_defaults".to_string(),
        owner: None,
        symbol: "make_tags".to_string(),
        generic_arguments: Vec::new(),
        signature: "()->list[str]".to_string(),
    };
    lowered
        .class_adapter_selections
        .push(sifr_lowering::ClassAdapterSelection {
            owner: "Model".to_string(),
            provider_module: "fixture.adapter".to_string(),
            provider_function: "adapt".to_string(),
            descriptor_type: Type::None,
            marker_identities: Vec::new(),
            data_parent: None,
            field_plans: vec![sifr_lowering::AdapterFieldPlan {
                identity: "fixture.runtime_defaults.Model.tags".to_string(),
                name: "tags".to_string(),
                declared_type: Type::List(Box::new(Type::Str)),
                default: sifr_lowering::AdapterFieldDefault::Factory(factory.clone()),
                validation_policy: None,
            }],
            handler_plans: Vec::new(),
            attached_api_set: None,
            adapter_invocation_identity: [0; 32],
            post_adapter_identity: [0; 32],
            range: ruff_text_size::TextRange::default(),
        });
    let class = lowered.module.classes.first().expect("class exists");
    let shape = describe_type(
        "fixture.runtime_defaults",
        &class_type(class, Vec::new()),
        &lowered,
    );
    let ShapeNode::Nominal { fields, .. } = &shape.root else {
        panic!("model shape should be nominal");
    };
    assert!(matches!(
        &fields[0].default,
        ShapeFieldDefault::Factory(value) if value == &factory
    ));
    let ConstValue::Record(shape) = shape.to_const_value() else {
        panic!("shape input should be a record");
    };
    let Some(ConstValue::Record(root)) = shape.get("root") else {
        panic!("shape root should be a record");
    };
    let Some(ConstValue::List(fields)) = root.get("fields") else {
        panic!("shape fields should be a list");
    };
    let ConstValue::Record(field) = &fields[0] else {
        panic!("shape field should be a record");
    };
    assert_eq!(field.get("required"), Some(&ConstValue::Bool(false)));
    assert_eq!(
        field.get("default_kind"),
        Some(&ConstValue::String("factory".to_string()))
    );
    assert!(matches!(
        field.get("default_factory"),
        Some(ConstValue::CallableIdentity(value)) if value == &factory
    ));
}

#[test]
fn enum_and_newtype_shapes_preserve_metadata_and_nominal_identity() {
    let source = r#"
from enum import Enum

@metadata("fixture.kind", "color")
@metadata("enum_variant", "RED", "fixture.label", "red")
class Color(Enum):
    RED = 1
    BLUE = 2

@metadata("fixture.kind", "port")
class Port(int):
    pass
"#;
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    let lowered = lower_module(&parsed).expect("fixture lowers");
    let color = &lowered.module.classes[0];
    let color_shape = describe_type("fixture.nominal", &class_type(color, Vec::new()), &lowered);
    assert!(color_shape
        .canonical_identity
        .contains("enum:fixture.nominal.Color"));
    assert!(color_shape.canonical_identity.contains("fixture.label"));

    let port = &lowered.module.classes[1];
    let port_shape = describe_type("fixture.nominal", &class_type(port, Vec::new()), &lowered);
    assert!(port_shape
        .canonical_identity
        .contains("newtype:fixture.nominal.Port"));
    assert!(port_shape.canonical_identity.contains("fixture.kind"));
}

#[test]
fn annotated_methods_and_parameters_are_exposed_in_source_order() {
    let source = r#"
class Model:
    value: int

    @metadata("fixture.callback", "compare")
    def __eq__(self, other: Model) -> bool:
        return True

    @staticmethod
    @metadata("fixture.callback", "first")
    @metadata("parameter", "value", "fixture.role", "input")
    def first(value: int) -> int:
        return value

    @staticmethod
    def helper(value: int) -> int:
        return value

    @classmethod
    @metadata("fixture.callback", "middle")
    def middle(cls, value: int) -> int:
        return value

    @staticmethod
    @metadata("fixture.callback", "second")
    async def second(*, mut value: str) -> str:
        return ""
"#;
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    let lowered = lower_module(&parsed).expect("fixture lowers");
    let class = lowered.module.classes.first().expect("class exists");
    let shape = describe_type(
        "fixture.callbacks",
        &class_type(class, Vec::new()),
        &lowered,
    );
    let const_shape = shape.to_const_value();
    let ConstValue::Record(shape_record) = const_shape else {
        panic!("shape must serialize as a record");
    };
    let Some(ConstValue::Record(root_record)) = shape_record.get("root") else {
        panic!("shape root must serialize as a record");
    };
    let Some(ConstValue::List(serialized_methods)) = root_record.get("methods") else {
        panic!("nominal methods must serialize as a list");
    };
    let Some(ConstValue::Record(first_method)) = serialized_methods.get(1) else {
        panic!("first callback method must serialize as a record");
    };
    let Some(ConstValue::List(first_params)) = first_method.get("params") else {
        panic!("method parameters must serialize as a list");
    };
    let Some(ConstValue::Record(first_param)) = first_params.first() else {
        panic!("first parameter must serialize as a record");
    };
    assert_eq!(
        first_method.get("name"),
        Some(&ConstValue::String("first".to_string()))
    );
    assert_eq!(
        first_param.get("convention"),
        Some(&ConstValue::String("borrow".to_string()))
    );
    assert!(matches!(
        first_param.get("metadata"),
        Some(ConstValue::List(metadata)) if metadata.len() == 1
    ));
    let ShapeNode::Nominal { methods, .. } = shape.root else {
        panic!("model must have a nominal shape");
    };

    assert_eq!(
        methods
            .iter()
            .map(|method| method.name.as_str())
            .collect::<Vec<_>>(),
        ["__eq__", "first", "middle", "second"]
    );
    assert_eq!(methods[0].kind, "regular");
    assert_eq!(methods[0].receiver.as_deref(), Some("borrow"));
    assert!(matches!(
        methods[0].params[0].declared_type,
        ShapeNode::RecursiveReference(_)
    ));
    assert_eq!(methods[1].kind, "static");
    assert_eq!(methods[1].params[0].convention, "borrow");
    assert_eq!(methods[1].params[0].metadata[0].key, "fixture.role");
    assert_eq!(methods[2].kind, "class");
    assert_eq!(methods[2].receiver, None);
    assert!(methods[3].is_async);
    assert!(methods[3].params[0].keyword_only);
    assert_eq!(methods[3].params[0].convention, "mut_borrow");
}

#[test]
fn described_method_exposes_successful_result_output() {
    let source = r#"
class HandlerFailure(Error):
    message: str

class Model:
    @staticmethod
    @metadata("fixture.callback", "checked")
    def checked(own value: str) -> Result[int, HandlerFailure]:
        return 1
"#;
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    let lowered = lower_module(&parsed).expect("fixture lowers");
    let model = &lowered.module.classes[1];
    let shape = describe_type(
        "fixture.successful_output",
        &class_type(model, Vec::new()),
        &lowered,
    );
    let ShapeNode::Nominal { methods, .. } = shape.root else {
        panic!("model must have a nominal shape");
    };
    assert_eq!(methods.len(), 1);
    assert!(matches!(*methods[0].result, ShapeNode::Other(_)));
    assert_eq!(
        methods[0].output.as_ref(),
        &ShapeNode::Primitive("int".to_string())
    );
    assert!(methods[0].fallible);
}

#[test]
fn generic_method_contract_uses_concrete_class_arguments() {
    let source = r#"
class Box[T]:
    value: T

    @staticmethod
    @metadata("fixture.callback", "after")
    def normalize(value: T) -> T:
        return value

class Container:
    item: Box[int]
"#;
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    let lowered = lower_module(&parsed).expect("fixture lowers");
    let container = &lowered.module.classes[1];
    let shape = describe_type(
        "fixture.generics",
        &class_type(container, Vec::new()),
        &lowered,
    );
    let ShapeNode::Nominal { fields, .. } = shape.root else {
        panic!("container must have a nominal shape");
    };
    let ShapeNode::Nominal { methods, .. } = &fields[0].declared_type else {
        panic!("container item must preserve its concrete box shape");
    };
    assert_eq!(
        methods[0].params[0].declared_type,
        ShapeNode::Primitive("int".to_string())
    );
    assert_eq!(*methods[0].result, ShapeNode::Primitive("int".to_string()));
    assert!(!shape.canonical_identity.contains("param:T"));
}

#[test]
fn inherited_handler_preserves_checked_signature_and_uses_child_diagnostic_origin() {
    use sifr_lowering::{
        AdapterFieldDefault, AdapterFieldPlan, AdapterHandlerPlan, CallableIdentity,
        ClassAdapterSelection, SourceOriginId, StaticProgramValue,
    };

    let source = r#"
class Parent[T]:
    value: T

    @classmethod
    def normalize(cls, own value: T) -> T:
        return value

class Child(Parent[int]):
    @classmethod
    def finish(cls, own value: int) -> int:
        return value
"#;
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    let mut lowered = lower_module(&parsed).expect("fixture lowers");
    let callable = CallableIdentity {
        module: "fixture.inherited".to_string(),
        owner: Some("fixture.inherited.Parent".to_string()),
        symbol: "normalize".to_string(),
        generic_arguments: Vec::new(),
        signature: "checked".to_string(),
    };
    let child_callable = CallableIdentity {
        module: "fixture.inherited".to_string(),
        owner: Some("fixture.inherited.Child".to_string()),
        symbol: "finish".to_string(),
        generic_arguments: Vec::new(),
        signature: "checked-child".to_string(),
    };
    lowered
        .class_adapter_selections
        .push(ClassAdapterSelection {
            owner: "Child".to_string(),
            provider_module: "fixture.adapter".to_string(),
            provider_function: "adapt".to_string(),
            descriptor_type: Type::Str,
            marker_identities: Vec::new(),
            data_parent: Some("Parent".to_string()),
            field_plans: vec![AdapterFieldPlan {
                identity: "fixture.inherited.Child.value".to_string(),
                name: "value".to_string(),
                declared_type: Type::Int,
                default: AdapterFieldDefault::Required,
                validation_policy: None,
            }],
            handler_plans: vec![
                AdapterHandlerPlan {
                    callable: callable.clone(),
                    descriptor_type: Type::Str,
                    descriptor_value: StaticProgramValue::String("after".to_string()),
                    descriptor_origin: SourceOriginId::new([7; 32], 3),
                    descriptor_range: ruff_text_size::TextRange::default(),
                    declaration_order: 0,
                },
                AdapterHandlerPlan {
                    callable: child_callable.clone(),
                    descriptor_type: Type::Str,
                    descriptor_value: StaticProgramValue::String("after".to_string()),
                    descriptor_origin: SourceOriginId::new([7; 32], 4),
                    descriptor_range: ruff_text_size::TextRange::default(),
                    declaration_order: 1,
                },
            ],
            attached_api_set: None,
            adapter_invocation_identity: [0; 32],
            post_adapter_identity: [0; 32],
            range: ruff_text_size::TextRange::default(),
        });
    let child = &lowered.module.classes[1];
    let shape = describe_type(
        "fixture.inherited",
        &class_type(child, Vec::new()),
        &lowered,
    );
    let ShapeNode::Nominal {
        fields, methods, ..
    } = shape.root
    else {
        panic!("child must have a nominal shape");
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name, "value");
    assert_eq!(
        fields[0].declared_type,
        ShapeNode::Primitive("int".to_string())
    );
    assert_eq!(methods.len(), 2);
    assert_eq!(methods[0].name, "normalize");
    assert_eq!(methods[1].name, "finish");
    assert_eq!(methods[0].target.as_ref(), Some(&callable));
    assert_eq!(methods[0].kind, "class");
    assert_eq!(methods[0].receiver, None);
    assert_eq!(
        methods[0].params[0].declared_type,
        ShapeNode::Primitive("int".to_string())
    );
    assert_eq!(*methods[0].result, ShapeNode::Primitive("int".to_string()));
    assert_eq!(*methods[0].output, ShapeNode::Primitive("int".to_string()));
    assert!(!methods[0].fallible);
    assert_eq!(methods[0].origin, None);
    assert_eq!(methods[1].target.as_ref(), Some(&child_callable));
    assert_eq!(methods[1].origin, Some(SourceOriginId::new([7; 32], 4)));
}

#[test]
fn async_method_contract_is_identical_at_root_and_nested_positions() {
    let source = r#"
class Inner:
    value: int

    @metadata("fixture.callback", "run")
    async def run(self) -> str:
        return ""

class Outer:
    inner: Inner
"#;
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    let lowered = lower_module(&parsed).expect("fixture lowers");
    let inner = &lowered.module.classes[0];
    let outer = &lowered.module.classes[1];

    let inner_shape = describe_type(
        "fixture.async_contract",
        &class_type(inner, Vec::new()),
        &lowered,
    );
    let outer_shape = describe_type(
        "fixture.async_contract",
        &class_type(outer, Vec::new()),
        &lowered,
    );
    let ShapeNode::Nominal {
        methods: root_methods,
        ..
    } = inner_shape.root
    else {
        panic!("inner must have a nominal shape");
    };
    let ShapeNode::Nominal { fields, .. } = outer_shape.root else {
        panic!("outer must have a nominal shape");
    };
    let ShapeNode::Nominal {
        methods: nested_methods,
        ..
    } = &fields[0].declared_type
    else {
        panic!("outer field must preserve the nested inner shape");
    };

    assert_eq!(&root_methods, nested_methods);
    assert_eq!(
        *root_methods[0].result,
        ShapeNode::Primitive("str".to_string())
    );
}

#[test]
fn annotated_constructor_uses_source_name_and_moves_identity() {
    let source = |parameter_type: &str| {
        format!(
            "class Model:\n    @metadata(\"fixture.callback\", \"post_init\")\n    def __init__(self, value: {parameter_type}) -> None:\n        pass\n"
        )
    };
    let describe = |source: &str| {
        let parsed = parse_module_suite(source, None).expect("fixture parses");
        let lowered = lower_module(&parsed).expect("fixture lowers");
        let class = lowered.module.classes.first().expect("class exists");
        describe_type(
            "fixture.constructor",
            &class_type(class, Vec::new()),
            &lowered,
        )
    };

    let integer = describe(&source("int"));
    let string = describe(&source("str"));
    let ShapeNode::Nominal { methods, .. } = &integer.root else {
        panic!("model must have a nominal shape");
    };
    assert_eq!(methods[0].name, "__init__");
    assert_eq!(methods[0].kind, "regular");
    assert_eq!(methods[0].receiver, None);
    assert_ne!(integer.canonical_identity, string.canonical_identity);
}

#[test]
fn only_annotated_method_contracts_change_canonical_identity() {
    let plain_source = r#"
class Model:
    value: int

    @staticmethod
    def helper(value: int) -> int:
        return 0
"#;
    let annotated_source = plain_source.replace(
        "    @staticmethod\n    def helper",
        "    @staticmethod\n    @metadata(\"fixture.callback\", \"after\")\n    def helper",
    );
    let changed_source = annotated_source.replace("value: int) -> int", "value: str) -> int");

    let describe = |source: &str| {
        let parsed = parse_module_suite(source, None).expect("fixture parses");
        let lowered = lower_module(&parsed).expect("fixture lowers");
        let class = lowered.module.classes.first().expect("class exists");
        describe_type(
            "fixture.callbacks",
            &class_type(class, Vec::new()),
            &lowered,
        )
        .canonical_identity
    };

    let without_helper = describe("class Model:\n    value: int\n");
    assert_eq!(describe(plain_source), without_helper);
    assert_ne!(describe(&annotated_source), without_helper);
    assert_ne!(describe(&changed_source), describe(&annotated_source));
}

#[test]
fn canonical_values_bind_record_keys_and_bytes_without_collisions() {
    let ambiguous_key = ConstValue::Record(BTreeMap::from([(
        "a=str:1:x,b".to_string(),
        ConstValue::None,
    )]));
    let two_fields = ConstValue::Record(BTreeMap::from([
        ("a".to_string(), ConstValue::String("x".to_string())),
        ("b".to_string(), ConstValue::None),
    ]));
    assert_ne!(
        canonical_value(&ambiguous_key),
        canonical_value(&two_fields)
    );
    assert_eq!(
        canonical_value(&ConstValue::Bytes(vec![0, 255])),
        "bytes:2:00ff"
    );
    assert_ne!(
        canonical_value(&ConstValue::Bytes(Vec::new())),
        canonical_value(&ConstValue::String(String::new()))
    );
}
