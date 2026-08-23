use super::*;
use sifr_lowering::{
    AdapterHandlerPlan, CallableIdentity, ClassAdapterSelection, SourceOriginId,
    StaticProgramValue, lower_module,
};
use sifr_syntax::parse_module_suite;
use sifr_type_system::FunctionType;

fn class_type(class: &sifr_lowering::HirClass) -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
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
fn unresolved_handler_ancestry_uses_an_opaque_checked_contract() {
    let source = r#"
class Parent[T]:
    @classmethod
    def normalize(cls, own value: T) -> T:
        return value

class Child(Parent[int]):
    pass
"#;
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    let mut lowered = lower_module(&parsed).expect("fixture lowers");
    let Type::Class { type_args, .. } = lowered.module.classes[1]
        .parent_type
        .as_mut()
        .expect("child parent type exists")
    else {
        panic!("child parent must be a class")
    };
    type_args.clear();
    let callable = CallableIdentity {
        module: "fixture.unresolved".to_string(),
        owner: Some("fixture.unresolved.Parent".to_string()),
        symbol: "normalize".to_string(),
        generic_arguments: Vec::new(),
        signature: "checked".to_string(),
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
            field_plans: Vec::new(),
            handler_plans: vec![AdapterHandlerPlan {
                callable: callable.clone(),
                descriptor_type: Type::Str,
                descriptor_value: StaticProgramValue::String("after".to_string()),
                descriptor_origin: SourceOriginId::new([9; 32], 1),
                descriptor_range: ruff_text_size::TextRange::default(),
                declaration_order: 0,
            }],
            attached_api_set: None,
            adapter_invocation_identity: [0; 32],
            post_adapter_identity: [0; 32],
            range: ruff_text_size::TextRange::default(),
        });
    let child = &lowered.module.classes[1];
    let shape = describe_type_with_externals(
        "fixture.unresolved",
        &class_type(child),
        &lowered,
        &ExternalDefs::default(),
    );
    let ShapeNode::Nominal { methods, .. } = shape.root else {
        panic!("child must have a nominal shape")
    };

    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].target.as_ref(), Some(&callable));
    assert!(methods[0].params.is_empty());
    assert_eq!(
        *methods[0].output,
        ShapeNode::Other("checked_handler".to_string())
    );
}
