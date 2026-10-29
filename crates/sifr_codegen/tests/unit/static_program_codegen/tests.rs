use super::*;
use sifr_ir::{
    CallableIdentity, MethodKind, StaticMethodParam, StaticMethodSlot, StaticMethodSlotContext,
    StaticMethodSlotInputRole,
};
use sifr_type_system::{ParamConvention, Type};

fn output(identity: u8) -> StaticSpecializationOutput {
    StaticSpecializationOutput {
        owner: "Record".to_string(),
        package_module: "schema.package".to_string(),
        function: "derive".to_string(),
        canonical_value: "record:1:value=int:7".to_string(),
        value: StaticProgramValue::Record(vec![(
            "value".to_string(),
            StaticProgramValue::Integer("7".to_string()),
        )]),
        program_identity: [identity; 32],
        structural_contract_version: 1,
        method_slots: Vec::new(),
        method_slot_context: None,
    }
}

#[test]
fn emission_contains_static_bytes_identity_and_typed_envelope() {
    let emitted =
        emit_static_specialization_programs(&[output(7)], &BTreeSet::from(["Record".to_string()]));
    assert!(emitted.contains("static __SIFR_STATIC_PROGRAM_BYTES_RECORD_SCHEMA_PACKAGE_DERIVE"));
    assert!(emitted.contains("StaticProgram<Record>"));
    assert!(emitted.contains("StaticProgramIdentity::from_bytes([7, 7"));
    assert!(emitted.contains("StaticProgramValue::Record"));
    assert!(emitted.contains("StaticProgramValue::Integer(\"7\")"));
}

#[test]
fn callable_identity_emission_preserves_the_exact_checked_target() {
    let emitted =
        static_value_expression(&StaticProgramValue::CallableIdentity(CallableIdentity {
            module: "fixture.callbacks".to_string(),
            owner: Some("fixture.callbacks.Handler".to_string()),
            symbol: "accept".to_string(),
            generic_arguments: vec!["str".to_string()],
            signature: "FunctionType([str], bool)".to_string(),
        }));
    assert!(emitted.contains("StaticProgramValue::CallableIdentity"));
    assert!(emitted.contains("module: \"fixture.callbacks\""));
    assert!(emitted.contains("owner: Some(\"fixture.callbacks.Handler\")"));
    assert!(emitted.contains("generic_arguments: &[\"str\"]"));
    assert!(emitted.contains("signature: \"FunctionType([str], bool)\""));
}

#[test]
fn ineligible_owner_keeps_static_bytes_without_typed_structural_impl() {
    let emitted = emit_static_specialization_programs(&[output(7)], &BTreeSet::new());
    assert!(emitted.contains("__SIFR_STATIC_PROGRAM_BYTES_"));
    assert!(!emitted.contains("impl ::sifr_runtime::interop::structural::StaticProgramType"));
    assert!(!emitted.contains("sifr_runtime::"));
    assert!(!emitted.contains("__SIFR_STATIC_PROGRAM_VALUE_"));
}

#[test]
fn cache_fragment_changes_only_with_program_identity() {
    assert_eq!(
        static_program_cache_fragment(&[output(7)]),
        static_program_cache_fragment(&[output(7)])
    );
    assert_ne!(
        static_program_cache_fragment(&[output(7)]),
        static_program_cache_fragment(&[output(8)])
    );
    assert_eq!(
        static_program_cache_fragment(&[output(7), output(8)]),
        static_program_cache_fragment(&[output(8), output(7)])
    );
}

#[test]
fn empty_slot_tables_do_not_add_a_cache_fragment() {
    assert!(method_slot_cache_fragment(&[output(7)]).is_empty());
}

fn test_method_slot() -> StaticMethodSlot {
    StaticMethodSlot {
        owner_identity: "fixture.Record".to_string(),
        owner_type: Type::Class {
            identity: Some("fixture.Record".to_string()),
            type_args: Vec::new(),
            name: "Record".to_string(),
            fields: Vec::new().into(),
            methods: Vec::new().into(),
            parent_class: None,
        },
        name: "normalize".to_string(),
        hir_name: "normalize".to_string(),
        method_kind: MethodKind::StaticMethod,
        receiver: None,
        params: vec![StaticMethodParam {
            name: "value".to_string(),
            ty: Type::Str,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::Result(Box::new(Type::Str), Box::new(Type::Str)),
        is_async: false,
        input_role: StaticMethodSlotInputRole::Value,
        input_type: Type::Str,
        output_type: Type::Str,
        context_type: None,
        context_mutable: false,
        descriptor_type: None,
        descriptor_value: None,
        descriptor_origin: None,
        descriptor_range: None,
        declaration_order: None,
        is_fallible: true,
    }
}

#[test]
fn emitted_symbols_cannot_collide_after_readable_name_normalization() {
    let mut dotted = output(7);
    dotted.package_module = "schema.package".to_string();
    let mut underscored = output(8);
    underscored.package_module = "schema_package".to_string();

    assert_ne!(
        rust_static_suffix(&dotted),
        rust_static_suffix(&underscored)
    );
}

#[test]
fn emits_monomorphic_method_slot_dispatch_and_header_identity() {
    let mut output = output(9);
    output.method_slots.push(test_method_slot());
    output.method_slot_context = Some(StaticMethodSlotContext::None);

    let cache_fragment = method_slot_cache_fragment(&[output.clone()]);
    assert!(cache_fragment.contains("method-slot-table-1"));
    assert!(cache_fragment.contains(&hex(&output.program_identity)));

    let emitted =
        emit_static_specialization_programs(&[output], &BTreeSet::from(["Record".to_string()]));
    assert!(emitted.contains("impl ::sifr_runtime::interop::structural::MethodSlotTable<::sifr_runtime::interop::structural::NoContext> for Record"));
    assert!(emitted.contains("match index"));
    assert!(emitted.contains("Record::normalize(value)"));
    assert!(emitted.contains("Some(*__SIFR_METHOD_SLOT_IDENTITY_"));
}

#[test]
fn emits_receiver_and_value_dispatch_from_the_recorded_input_role() {
    let mut output = output(10);
    let mut slot = test_method_slot();
    slot.name = "serialize".to_string();
    slot.hir_name = "serialize".to_string();
    slot.method_kind = MethodKind::Regular;
    slot.receiver = Some(sifr_type_system::ReceiverConvention::SharedBorrow);
    slot.input_role = StaticMethodSlotInputRole::ReceiverAndValue;
    slot.input_type = Type::Tuple(vec![slot.owner_type.clone(), Type::Str]);
    slot.return_type = Type::Str;
    slot.is_fallible = false;
    output.method_slots.push(slot);
    output.method_slot_context = Some(StaticMethodSlotContext::None);

    let emitted =
        emit_static_specialization_programs(&[output], &BTreeSet::from(["Record".to_string()]));

    assert!(
        emitted.contains("let (receiver, value): (Record, String) ="),
        "{emitted}"
    );
    assert!(emitted.contains("receiver.serialize(value)"), "{emitted}");
}
