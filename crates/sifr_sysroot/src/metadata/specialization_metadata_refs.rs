use super::{
    AdapterFieldDefault, AdapterFieldPlan, AdapterHandlerPlan, AppliedAdapterMetadata,
    AttachedApiDeclaration, AttachedApiReceiver, AttachedApiSetDeclaration, AttachedApiSetIdentity,
    CallableIdentity, ClassAdapterMarkerDeclaration, ClassAdapterProviderDeclaration,
    ClassAdapterSelection, ConstSpecializationRequest, DeclarationDescriptorFunction,
    DeclarationDescriptorKind, DeclarationMetadataTargetKind, JsonIntegerBoundaryRequest, Record,
    RecordId, References, SourceOriginId, StaticMethodParam, StaticMethodSlot,
    StaticMethodSlotContext, StaticMethodSlotInputRole, StaticProgramValue,
    StaticSpecializationOutput, TypedDeclarationDescriptor, TypedDeclarationMetadata, sealed,
};
impl References for SourceOriginId {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.location.references(out);
        self.local_ordinal.references(out);
    }
}
impl sealed::Sealed for SourceOriginId {}
impl Record for SourceOriginId {
    const KIND: u16 = 118;
}
impl References for DeclarationMetadataTargetKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Type => {}
            Self::Field => {}
            Self::EnumVariant => {}
            Self::Function => {}
            Self::Method => {}
            Self::Parameter => {}
        }
    }
}
impl sealed::Sealed for DeclarationMetadataTargetKind {}
impl Record for DeclarationMetadataTargetKind {
    const KIND: u16 = 22;
}
impl References for TypedDeclarationMetadata {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.owner.references(out);
        self.target_kind.references(out);
        self.target_name.references(out);
        self.key.references(out);
        self.value_type.references(out);
        self.value.references(out);
        self.range.references(out);
    }
}
impl sealed::Sealed for TypedDeclarationMetadata {}
impl Record for TypedDeclarationMetadata {
    const KIND: u16 = 134;
}
impl References for ConstSpecializationRequest {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.owner.references(out);
        self.package_module.references(out);
        self.function.references(out);
        self.range.references(out);
    }
}
impl sealed::Sealed for ConstSpecializationRequest {}
impl Record for ConstSpecializationRequest {
    const KIND: u16 = 17;
}
impl References for DeclarationDescriptorKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Field => {}
            Self::Class => {}
            Self::Method => {}
            Self::Type => {}
        }
    }
}
impl sealed::Sealed for DeclarationDescriptorKind {}
impl Record for DeclarationDescriptorKind {
    const KIND: u16 = 20;
}
impl References for ClassAdapterProviderDeclaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.module.references(out);
        self.function.references(out);
        self.descriptor_module.references(out);
        self.descriptor_symbol.references(out);
        self.descriptor_type.references(out);
        self.range.references(out);
    }
}
impl sealed::Sealed for ClassAdapterProviderDeclaration {}
impl Record for ClassAdapterProviderDeclaration {
    const KIND: u16 = 14;
}
impl References for ClassAdapterMarkerDeclaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.module.references(out);
        self.symbol.references(out);
        self.provider_module.references(out);
        self.provider_function.references(out);
        self.descriptor_type.references(out);
        self.range.references(out);
    }
}
impl sealed::Sealed for ClassAdapterMarkerDeclaration {}
impl Record for ClassAdapterMarkerDeclaration {
    const KIND: u16 = 13;
}
impl References for AttachedApiSetIdentity {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.module.references(out);
        self.symbol.references(out);
    }
}
impl sealed::Sealed for AttachedApiSetIdentity {}
impl Record for AttachedApiSetIdentity {
    const KIND: u16 = 8;
}
impl References for AttachedApiSetDeclaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.identity.references(out);
        self.range.references(out);
    }
}
impl sealed::Sealed for AttachedApiSetDeclaration {}
impl Record for AttachedApiSetDeclaration {
    const KIND: u16 = 7;
}
impl References for AttachedApiReceiver {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Type => {}
            Self::Immutable => {}
            Self::Mutable => {}
            Self::Owned => {}
        }
    }
}
impl sealed::Sealed for AttachedApiReceiver {}
impl Record for AttachedApiReceiver {
    const KIND: u16 = 6;
}
impl References for AttachedApiDeclaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.module.references(out);
        self.function.references(out);
        self.set.references(out);
        self.public_name.references(out);
        self.receiver.references(out);
        self.owner_type_param.references(out);
        self.type_params.references(out);
        self.type_param_bounds.references(out);
        self.function_type.references(out);
        self.defaults.references(out);
        self.range.references(out);
    }
}
impl sealed::Sealed for AttachedApiDeclaration {}
impl Record for AttachedApiDeclaration {
    const KIND: u16 = 5;
}
impl References for ClassAdapterSelection {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.owner.references(out);
        self.provider_module.references(out);
        self.provider_function.references(out);
        self.descriptor_type.references(out);
        self.marker_identities.references(out);
        self.data_parent.references(out);
        self.field_plans.references(out);
        self.handler_plans.references(out);
        self.attached_api_set.references(out);
        self.adapter_invocation_identity.references(out);
        self.post_adapter_identity.references(out);
        self.range.references(out);
    }
}
impl sealed::Sealed for ClassAdapterSelection {}
impl Record for ClassAdapterSelection {
    const KIND: u16 = 15;
}
impl References for AdapterHandlerPlan {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.callable.references(out);
        self.descriptor_type.references(out);
        self.descriptor_value.references(out);
        self.descriptor_origin.references(out);
        self.descriptor_range.references(out);
        self.declaration_order.references(out);
    }
}
impl sealed::Sealed for AdapterHandlerPlan {}
impl Record for AdapterHandlerPlan {
    const KIND: u16 = 3;
}
impl References for AdapterFieldPlan {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.identity.references(out);
        self.name.references(out);
        self.declared_type.references(out);
        self.default.references(out);
        self.validation_policy.references(out);
    }
}
impl sealed::Sealed for AdapterFieldPlan {}
impl Record for AdapterFieldPlan {
    const KIND: u16 = 2;
}
impl References for AdapterFieldDefault {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Required => {}
            Self::Const(v0) => {
                v0.references(out);
            }
            Self::Factory(v0) => {
                v0.references(out);
            }
        }
    }
}
impl sealed::Sealed for AdapterFieldDefault {}
impl Record for AdapterFieldDefault {
    const KIND: u16 = 1;
}
impl References for DeclarationDescriptorFunction {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.module.references(out);
        self.function.references(out);
        self.provider_module.references(out);
        self.provider_function.references(out);
        self.descriptor_type.references(out);
        self.return_type.references(out);
        self.kind.references(out);
        self.range.references(out);
    }
}
impl sealed::Sealed for DeclarationDescriptorFunction {}
impl Record for DeclarationDescriptorFunction {
    const KIND: u16 = 19;
}
impl References for CallableIdentity {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.module.references(out);
        self.owner.references(out);
        self.symbol.references(out);
        self.generic_arguments.references(out);
        self.signature.references(out);
    }
}
impl sealed::Sealed for CallableIdentity {}
impl Record for CallableIdentity {
    const KIND: u16 = 12;
}
impl References for TypedDeclarationDescriptor {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.owner.references(out);
        self.target_kind.references(out);
        self.target_identity.references(out);
        self.target_callable.references(out);
        self.provider_module.references(out);
        self.provider_function.references(out);
        self.value_type.references(out);
        self.value.references(out);
        self.range.references(out);
    }
}
impl sealed::Sealed for TypedDeclarationDescriptor {}
impl Record for TypedDeclarationDescriptor {
    const KIND: u16 = 133;
}
impl References for AppliedAdapterMetadata {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.owner.references(out);
        self.target_kind.references(out);
        self.target_name.references(out);
        self.key.references(out);
        self.value_type.references(out);
        self.value.references(out);
    }
}
impl sealed::Sealed for AppliedAdapterMetadata {}
impl Record for AppliedAdapterMetadata {
    const KIND: u16 = 4;
}
impl References for StaticProgramValue {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::None => {}
            Self::Bool(v0) => {
                v0.references(out);
            }
            Self::Integer(v0) => {
                v0.references(out);
            }
            Self::FloatBits(v0) => {
                v0.references(out);
            }
            Self::String(v0) => {
                v0.references(out);
            }
            Self::Bytes(v0) => {
                v0.references(out);
            }
            Self::Tuple(v0) => {
                v0.references(out);
            }
            Self::List(v0) => {
                v0.references(out);
            }
            Self::Record(v0) => {
                v0.references(out);
            }
            Self::CallableIdentity(v0) => {
                v0.references(out);
            }
        }
    }
}
impl sealed::Sealed for StaticProgramValue {}
impl Record for StaticProgramValue {
    const KIND: u16 = 123;
}
impl References for StaticSpecializationOutput {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.owner.references(out);
        self.package_module.references(out);
        self.function.references(out);
        self.canonical_value.references(out);
        self.value.references(out);
        self.program_identity.references(out);
        self.structural_contract_version.references(out);
        self.method_slots.references(out);
        self.method_slot_context.references(out);
    }
}
impl sealed::Sealed for StaticSpecializationOutput {}
impl Record for StaticSpecializationOutput {
    const KIND: u16 = 124;
}
impl References for StaticMethodSlotContext {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::None => {}
            Self::Shared(v0) => {
                v0.references(out);
            }
            Self::Mutable(v0) => {
                v0.references(out);
            }
        }
    }
}
impl sealed::Sealed for StaticMethodSlotContext {}
impl Record for StaticMethodSlotContext {
    const KIND: u16 = 121;
}
impl References for StaticMethodSlotInputRole {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Value => {}
            Self::Receiver => {}
            Self::ReceiverAndValue => {}
        }
    }
}
impl sealed::Sealed for StaticMethodSlotInputRole {}
impl Record for StaticMethodSlotInputRole {
    const KIND: u16 = 122;
}
impl References for StaticMethodSlot {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.owner_identity.references(out);
        self.owner_type.references(out);
        self.name.references(out);
        self.hir_name.references(out);
        self.method_kind.references(out);
        self.receiver.references(out);
        self.params.references(out);
        self.return_type.references(out);
        self.is_async.references(out);
        self.input_role.references(out);
        self.input_type.references(out);
        self.output_type.references(out);
        self.context_type.references(out);
        self.context_mutable.references(out);
        self.descriptor_type.references(out);
        self.descriptor_value.references(out);
        self.descriptor_origin.references(out);
        self.descriptor_range.references(out);
        self.declaration_order.references(out);
        self.is_fallible.references(out);
    }
}
impl sealed::Sealed for StaticMethodSlot {}
impl Record for StaticMethodSlot {
    const KIND: u16 = 120;
}
impl References for StaticMethodParam {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.name.references(out);
        self.ty.references(out);
        self.keyword_only.references(out);
        self.convention.references(out);
    }
}
impl sealed::Sealed for StaticMethodParam {}
impl Record for StaticMethodParam {
    const KIND: u16 = 119;
}
impl References for JsonIntegerBoundaryRequest {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.owner.references(out);
        self.field.references(out);
        self.profile.references(out);
        self.representation.references(out);
        self.static_minimum.references(out);
        self.static_maximum.references(out);
        self.range.references(out);
    }
}
impl sealed::Sealed for JsonIntegerBoundaryRequest {}
impl Record for JsonIntegerBoundaryRequest {
    const KIND: u16 = 67;
}
