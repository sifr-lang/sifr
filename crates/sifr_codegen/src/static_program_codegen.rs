use sifr_ir::{StaticProgramValue, StaticSpecializationOutput};
use sifr_type_system::source_class_rust_name;
use std::collections::BTreeSet;
use std::fmt::Write as _;

/// Emits deterministic immutable bytes for every retained package specialization.
#[must_use]
pub fn emit_static_specialization_programs(
    outputs: &[StaticSpecializationOutput],
    structural_owners: &BTreeSet<String>,
) -> String {
    let mut ordered = outputs.to_vec();
    ordered.sort_by(|left, right| {
        (&left.owner, &left.package_module, &left.function).cmp(&(
            &right.owner,
            &right.package_module,
            &right.function,
        ))
    });
    let mut out = String::new();
    for output in &ordered {
        let suffix = rust_static_suffix(output);
        let _ = writeln!(
            out,
            "#[doc(hidden)]\npub(crate) static __SIFR_STATIC_PROGRAM_BYTES_{suffix}: &[u8] = &{:?};",
            output.canonical_value.as_bytes()
        );
        let _ = writeln!(
            out,
            "#[doc(hidden)]\npub(crate) const __SIFR_STATIC_PROGRAM_IDENTITY_{suffix}: [u8; 32] = {:?};",
            output.program_identity
        );
        if structural_owners.contains(&output.owner) {
            let owner = source_class_rust_name(&output.owner);
            out.push_str(
                &crate::static_program_slots_codegen::emit_method_slot_table(
                    output, &owner, &suffix,
                ),
            );
            let value = static_value_expression(&output.value);
            let _ = writeln!(
                out,
                "#[doc(hidden)]\npub(crate) static __SIFR_STATIC_PROGRAM_VALUE_{suffix}: ::sifr_runtime::interop::structural::StaticProgramValue = {value};"
            );
            let identity = output
                .program_identity
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            let slot_table =
                crate::static_program_slots_codegen::slot_table_header_expression(output, &suffix);
            let _ = writeln!(
                out,
                "#[doc(hidden)]\npub(crate) static __SIFR_STATIC_PROGRAM_{suffix}: ::std::sync::LazyLock<::sifr_runtime::interop::structural::StaticProgram<{owner}>> = ::std::sync::LazyLock::new(|| {{\n    let identity = ::sifr_runtime::interop::structural::StaticProgramIdentity::from_bytes([{identity}]);\n    let shape = <{owner} as ::sifr_runtime::interop::structural::StructuralType>::shape_identity();\n    let header = ::sifr_runtime::interop::structural::StaticProgramHeader::__from_compiler(::sifr_runtime::interop::structural::STATIC_PROGRAM_FORMAT_VERSION, {}, ::sifr_runtime::interop::structural::STRUCTURAL_BRIDGE_CONTRACT_VERSION, identity, shape, {slot_table}, ::sifr_runtime::interop::__generated_glue::token());\n    ::sifr_runtime::interop::structural::StaticProgram::__from_compiler(header, __SIFR_STATIC_PROGRAM_BYTES_{suffix}, __SIFR_STATIC_PROGRAM_VALUE_{suffix}, ::sifr_runtime::interop::__generated_glue::token())\n}});",
                output.structural_contract_version
            );
            let _ = writeln!(
                out,
                "impl ::sifr_runtime::interop::structural::StaticProgramType for {owner} {{\n    fn static_program() -> &'static ::sifr_runtime::interop::structural::StaticProgram<Self> {{\n        &__SIFR_STATIC_PROGRAM_{suffix}\n    }}\n}}"
            );
        }
    }
    out
}

fn static_value_expression(value: &StaticProgramValue) -> String {
    let path = "::sifr_runtime::interop::structural::StaticProgramValue";
    match value {
        StaticProgramValue::None => format!("{path}::None"),
        StaticProgramValue::Bool(value) => format!("{path}::Bool({value})"),
        StaticProgramValue::Integer(value) => format!("{path}::Integer({value:?})"),
        StaticProgramValue::FloatBits(value) => format!("{path}::FloatBits({value})"),
        StaticProgramValue::String(value) => format!("{path}::String({value:?})"),
        StaticProgramValue::Bytes(value) => format!("{path}::Bytes(&{value:?})"),
        StaticProgramValue::Tuple(values) => {
            format!("{path}::Tuple(&[{}])", static_values_expression(values))
        }
        StaticProgramValue::List(values) => {
            format!("{path}::List(&[{}])", static_values_expression(values))
        }
        StaticProgramValue::Record(fields) => format!(
            "{path}::Record(&[{}])",
            fields
                .iter()
                .map(|(name, value)| format!("({name:?}, {})", static_value_expression(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        StaticProgramValue::CallableIdentity(identity) => {
            let owner = identity
                .owner
                .as_ref()
                .map_or_else(|| "None".to_string(), |owner| format!("Some({owner:?})"));
            let generic_arguments = identity
                .generic_arguments
                .iter()
                .map(|argument| format!("{argument:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{path}::CallableIdentity {{ module: {:?}, owner: {owner}, symbol: {:?}, generic_arguments: &[{generic_arguments}], signature: {:?} }}",
                identity.module, identity.symbol, identity.signature
            )
        }
    }
}

fn static_values_expression(values: &[StaticProgramValue]) -> String {
    values
        .iter()
        .map(static_value_expression)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Returns only specialization owners that can receive all structural bridge implementations.
#[must_use]
pub fn structural_static_program_owners(module: &sifr_ir::HirModule) -> BTreeSet<String> {
    module
        .classes
        .iter()
        .filter(|class| crate::structural_impl_codegen::structural_record_supported(class, module))
        .map(|class| class.name.clone())
        .collect()
}

#[must_use]
pub fn static_program_cache_fragment(outputs: &[StaticSpecializationOutput]) -> String {
    let mut ordered = outputs.to_vec();
    ordered.sort_by(|left, right| {
        (
            &left.owner,
            &left.package_module,
            &left.function,
            left.program_identity,
        )
            .cmp(&(
                &right.owner,
                &right.package_module,
                &right.function,
                right.program_identity,
            ))
    });
    let mut out = String::new();
    for output in ordered {
        let _ = writeln!(
            out,
            "{}|{}|{}|{}",
            output.owner,
            output.package_module,
            output.function,
            hex(&output.program_identity)
        );
    }
    out
}

/// Returns a deterministic build-cache fragment for generated method-slot glue.
#[must_use]
pub fn method_slot_cache_fragment(outputs: &[StaticSpecializationOutput]) -> String {
    let mut ordered = outputs
        .iter()
        .filter(|output| !output.method_slots.is_empty())
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        (&left.owner, &left.package_module, &left.function).cmp(&(
            &right.owner,
            &right.package_module,
            &right.function,
        ))
    });
    let mut out = String::new();
    for output in ordered {
        let _ = writeln!(
            out,
            "{}|{}|{}|{}|method-slot-table-1",
            output.owner,
            output.package_module,
            output.function,
            hex(&output.program_identity),
        );
    }
    out
}

fn rust_static_suffix(output: &StaticSpecializationOutput) -> String {
    let readable = format!(
        "{}_{}_{}",
        output.owner, output.package_module, output.function
    )
    .chars()
    .map(|character| {
        if character.is_ascii_alphanumeric() {
            character.to_ascii_uppercase()
        } else {
            '_'
        }
    })
    .collect::<String>();
    format!(
        "{readable}_{}",
        hex(&output.program_identity).to_uppercase()
    )
}

fn hex(bytes: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_ir::{
        CallableIdentity, MethodKind, StaticMethodParam, StaticMethodSlot, StaticMethodSlotContext,
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
        let emitted = emit_static_specialization_programs(
            &[output(7)],
            &BTreeSet::from(["Record".to_string()]),
        );
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
        output.method_slots.push(StaticMethodSlot {
            owner_identity: "fixture.Record".to_string(),
            owner_type: Type::Class {
                identity: Some("fixture.Record".to_string()),
                type_args: Vec::new(),
                name: "Record".to_string(),
                fields: Vec::new(),
                methods: Vec::new(),
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
            input_type: Type::Str,
            output_type: Type::Str,
            context_type: None,
            context_mutable: false,
        });
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
}
