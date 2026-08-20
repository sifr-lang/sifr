use sifr_ir::{
    MethodKind, StaticMethodSlot, StaticMethodSlotContext, StaticMethodSlotInputRole,
    StaticSpecializationOutput,
};
use sifr_type_system::{ParamConvention, ReceiverConvention, Type};
use std::fmt::Write as _;

const STRUCTURAL: &str = "::sifr_runtime::interop::structural";

pub(super) fn emit_method_slot_table(
    output: &StaticSpecializationOutput,
    owner: &str,
    suffix: &str,
) -> String {
    let Some(context) = output.method_slot_context.as_ref() else {
        return String::new();
    };
    if output.method_slots.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    let signatures = output
        .method_slots
        .iter()
        .map(slot_signature_expression)
        .collect::<Vec<_>>()
        .join(",\n        ");
    let _ = writeln!(
        out,
        "#[doc(hidden)]\npub(crate) static __SIFR_METHOD_SLOT_SIGNATURES_{suffix}: ::std::sync::LazyLock<Vec<{STRUCTURAL}::SlotSignature>> = ::std::sync::LazyLock::new(|| vec![\n        {signatures}\n]);"
    );

    let identity_signatures = output
        .method_slots
        .iter()
        .map(slot_identity_expression)
        .collect::<Vec<_>>()
        .join(",\n        ");
    let context_identity = context_identity_expression(context);
    let context_mode = context_mode_expression(context);
    let program_identity = output
        .program_identity
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(
        out,
        "#[doc(hidden)]\npub(crate) static __SIFR_METHOD_SLOT_IDENTITY_{suffix}: ::std::sync::LazyLock<{STRUCTURAL}::SlotTableIdentity> = ::std::sync::LazyLock::new(|| {STRUCTURAL}::slot_table_identity(\n    {STRUCTURAL}::StaticProgramIdentity::from_bytes([{program_identity}]),\n    {context_identity},\n    {context_mode},\n    &[\n        {identity_signatures}\n    ],\n));"
    );

    let context_type = context_impl_type(context);
    let impl_generics = if matches!(context, StaticMethodSlotContext::Shared(_)) {
        "<'__sifr_context>"
    } else {
        ""
    };
    let arms = output
        .method_slots
        .iter()
        .enumerate()
        .map(|(index, slot)| slot_arm(index, slot, context))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = writeln!(
        out,
        "impl{impl_generics} {STRUCTURAL}::MethodSlotTable<{context_type}> for {owner} {{\n    fn slot_table_identity() -> {STRUCTURAL}::SlotTableIdentity {{\n        *__SIFR_METHOD_SLOT_IDENTITY_{suffix}\n    }}\n\n    fn slot_signatures() -> &'static [{STRUCTURAL}::SlotSignature] {{\n        &__SIFR_METHOD_SLOT_SIGNATURES_{suffix}\n    }}\n\n    fn invoke_slot(\n        index: usize,\n        input: {STRUCTURAL}::StructuralArena,\n        context: &mut {context_type},\n        handler: Option<&{STRUCTURAL}::SlotHandler<'_>>,\n        sink: &mut dyn {STRUCTURAL}::SlotSink,\n    ) -> Result<(), {STRUCTURAL}::SlotError> {{\n        let _ = handler;\n        match index {{\n{arms}\n            _ => Err({STRUCTURAL}::SlotError::UnknownSlot),\n        }}\n    }}\n}}"
    );
    out
}

pub(super) fn slot_table_header_expression(
    output: &StaticSpecializationOutput,
    suffix: &str,
) -> String {
    if output.method_slots.is_empty() {
        "None".to_string()
    } else {
        format!("Some(*__SIFR_METHOD_SLOT_IDENTITY_{suffix})")
    }
}

fn slot_signature_expression(slot: &StaticMethodSlot) -> String {
    let input = render_slot_type(&slot.input_type);
    let output = render_slot_type(&slot.output_type);
    let receiver = receiver_expression(slot.receiver);
    format!(
        "{STRUCTURAL}::SlotSignature::__from_compiler({:?}, <{input} as {STRUCTURAL}::StructuralType>::shape_identity(), <{output} as {STRUCTURAL}::StructuralType>::shape_identity(), {receiver}, None, ::sifr_runtime::interop::__generated_glue::token())",
        slot.name
    )
}

fn slot_identity_expression(slot: &StaticMethodSlot) -> String {
    let input = render_slot_type(&slot.input_type);
    let output = render_slot_type(&slot.output_type);
    let receiver = receiver_identity_expression(slot.receiver);
    format!(
        "{STRUCTURAL}::SlotIdentitySignature {{ name: {:?}, input: <{input} as {STRUCTURAL}::StructuralType>::shape_identity(), output: <{output} as {STRUCTURAL}::StructuralType>::shape_identity(), receiver: {receiver}, handler: None }}",
        slot.name
    )
}

fn slot_arm(index: usize, slot: &StaticMethodSlot, context: &StaticMethodSlotContext) -> String {
    let input = render_slot_type(&slot.input_type);
    let receiver_binding = if slot.receiver == Some(ReceiverConvention::MutableBorrow) {
        "mut "
    } else {
        ""
    };
    let value_binding = if slot
        .params
        .first()
        .is_some_and(|param| param.convention.is_mut_borrow())
    {
        "mut "
    } else {
        ""
    };
    let binding = match slot.input_role {
        StaticMethodSlotInputRole::ReceiverAndValue => {
            format!("({receiver_binding}receiver, {value_binding}value)")
        }
        StaticMethodSlotInputRole::Receiver => format!("{receiver_binding}value"),
        StaticMethodSlotInputRole::Value => format!("{value_binding}value"),
    };
    let call = slot_call_expression(slot, context);
    let dispatch = if slot.is_fallible {
        format!(
            "match {call} {{\n                    Ok(output) => {STRUCTURAL}::StructuralProject::structural_project(&output, &mut {STRUCTURAL}::SlotSinkVisitor::new(sink)).map_err({STRUCTURAL}::SlotError::Contract),\n                    Err(error) => Err({STRUCTURAL}::SlotError::Slot(error.to_string())),\n                }}"
        )
    } else {
        format!(
            "let output = {call};\n                {STRUCTURAL}::StructuralProject::structural_project(&output, &mut {STRUCTURAL}::SlotSinkVisitor::new(sink)).map_err({STRUCTURAL}::SlotError::Contract)"
        )
    };
    format!(
        "            {index}usize => {{\n                let {binding} = {STRUCTURAL}::structural_construct::<{input}, _>(input).map_err({STRUCTURAL}::SlotError::Contract)?;\n                {dispatch}\n            }}"
    )
}

fn slot_call_expression(slot: &StaticMethodSlot, context: &StaticMethodSlotContext) -> String {
    let owner = render_slot_type(&slot.owner_type);
    let method = crate::Renderer::render_identifier(&slot.hir_name);
    let context_arg = slot.context_type.as_ref().map(|_| match context {
        StaticMethodSlotContext::None => "context".to_string(),
        StaticMethodSlotContext::Shared(_) => "context.get()".to_string(),
        StaticMethodSlotContext::Mutable(_) => "context".to_string(),
    });
    if slot.receiver.is_some() {
        let mut args = Vec::new();
        let receiver = if slot.input_role == StaticMethodSlotInputRole::ReceiverAndValue {
            let value = slot.params.first().map_or_else(
                || "value".to_string(),
                |param| argument_for_convention("value", param.convention),
            );
            args.push(value);
            "receiver"
        } else {
            "value"
        };
        args.extend(context_arg);
        return format!("{receiver}.{method}({})", args.join(", "));
    }
    let value = slot.params.first().map_or_else(
        || "value".to_string(),
        |param| argument_for_convention("value", param.convention),
    );
    let mut args = vec![value];
    args.extend(context_arg);
    match slot.method_kind {
        MethodKind::Regular | MethodKind::StaticMethod => {
            format!("{owner}::{method}({})", args.join(", "))
        }
        MethodKind::ClassMethod => {
            format!("{owner}::{method}({})", args.join(", "))
        }
    }
}

fn argument_for_convention(name: &str, convention: ParamConvention) -> String {
    if convention.is_mut_borrow() {
        format!("&mut {name}")
    } else if convention.is_shared_borrow() {
        format!("&{name}")
    } else {
        name.to_string()
    }
}

fn context_impl_type(context: &StaticMethodSlotContext) -> String {
    match context {
        StaticMethodSlotContext::None => format!("{STRUCTURAL}::NoContext"),
        StaticMethodSlotContext::Shared(ty) => format!(
            "{STRUCTURAL}::SharedContext<'__sifr_context, {}>",
            render_slot_type(ty)
        ),
        StaticMethodSlotContext::Mutable(ty) => render_slot_type(ty),
    }
}

fn context_identity_expression(context: &StaticMethodSlotContext) -> String {
    let ty = match context {
        StaticMethodSlotContext::None => format!("{STRUCTURAL}::NoContext"),
        StaticMethodSlotContext::Shared(ty) | StaticMethodSlotContext::Mutable(ty) => {
            render_slot_type(ty)
        }
    };
    format!("<{ty} as {STRUCTURAL}::StructuralType>::shape_identity()")
}

fn context_mode_expression(context: &StaticMethodSlotContext) -> &'static str {
    match context {
        StaticMethodSlotContext::None => {
            "::sifr_runtime::interop::structural::SlotContextModeIdentity::None"
        }
        StaticMethodSlotContext::Shared(_) => {
            "::sifr_runtime::interop::structural::SlotContextModeIdentity::Shared"
        }
        StaticMethodSlotContext::Mutable(_) => {
            "::sifr_runtime::interop::structural::SlotContextModeIdentity::Mutable"
        }
    }
}

fn receiver_expression(receiver: Option<ReceiverConvention>) -> &'static str {
    match receiver {
        None => "::sifr_runtime::interop::structural::SlotReceiver::None",
        Some(ReceiverConvention::SharedBorrow) => {
            "::sifr_runtime::interop::structural::SlotReceiver::Shared"
        }
        Some(ReceiverConvention::MutableBorrow) => {
            "::sifr_runtime::interop::structural::SlotReceiver::Exclusive"
        }
        Some(ReceiverConvention::Owned | ReceiverConvention::OwnedMutable) => {
            "::sifr_runtime::interop::structural::SlotReceiver::Owned"
        }
    }
}

fn receiver_identity_expression(receiver: Option<ReceiverConvention>) -> &'static str {
    match receiver {
        None => "::sifr_runtime::interop::structural::SlotReceiverIdentity::None",
        Some(ReceiverConvention::SharedBorrow) => {
            "::sifr_runtime::interop::structural::SlotReceiverIdentity::Shared"
        }
        Some(ReceiverConvention::MutableBorrow) => {
            "::sifr_runtime::interop::structural::SlotReceiverIdentity::Exclusive"
        }
        Some(ReceiverConvention::Owned | ReceiverConvention::OwnedMutable) => {
            "::sifr_runtime::interop::structural::SlotReceiverIdentity::Owned"
        }
    }
}

fn render_slot_type(ty: &Type) -> String {
    crate::render_type(&crate::sifr_type_to_rust_type(ty))
}
