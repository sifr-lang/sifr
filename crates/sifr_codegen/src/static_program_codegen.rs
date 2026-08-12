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
            let _ = writeln!(
                out,
                "#[doc(hidden)]\npub(crate) static __SIFR_STATIC_PROGRAM_{suffix}: ::std::sync::LazyLock<::sifr_runtime::interop::structural::StaticProgram<{owner}>> = ::std::sync::LazyLock::new(|| {{\n    let identity = ::sifr_runtime::interop::structural::StaticProgramIdentity::from_bytes([{identity}]);\n    let shape = <{owner} as ::sifr_runtime::interop::structural::StructuralType>::shape_identity();\n    let header = ::sifr_runtime::interop::structural::StaticProgramHeader::__from_compiler(::sifr_runtime::interop::structural::STATIC_PROGRAM_FORMAT_VERSION, {}, ::sifr_runtime::interop::structural::STRUCTURAL_BRIDGE_CONTRACT_VERSION, identity, shape, ::sifr_runtime::interop::__generated_glue::token());\n    ::sifr_runtime::interop::structural::StaticProgram::__from_compiler(header, __SIFR_STATIC_PROGRAM_BYTES_{suffix}, __SIFR_STATIC_PROGRAM_VALUE_{suffix}, ::sifr_runtime::interop::__generated_glue::token())\n}});",
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
}
