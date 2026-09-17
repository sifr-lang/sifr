use super::{
    AdapterFieldDefault, AdapterFieldPlan, AdapterHandlerPlan, AppliedAdapterMetadata,
    AttachedApiDeclaration, AttachedApiReceiver, AttachedApiSetDeclaration, AttachedApiSetIdentity,
    Binder, BindingId, BodyRole, CallableIdentity, ClassAdapterMarkerDeclaration,
    ClassAdapterProviderDeclaration, ClassAdapterSelection, CompilerIntrinsicId,
    ConstSpecializationRequest, Declaration, DeclarationDescriptorFunction,
    DeclarationDescriptorKind, DeclarationKind, DeclarationMetadataTargetKind, FieldIdentity,
    FixedIntType, FragmentBoundary, FragmentValidation, FunctionType, HirAsyncWithKind, HirClass,
    HirClassKind, HirCollectionMutation, HirExceptHandler, HirExpr, HirFStringPart, HirFunction,
    HirImport, HirIteratorOp, HirMatchArm, HirModule, HirParam, HirPattern, HirSqlBoundQuery,
    HirSqlCardinality, HirSqlEffectContract, HirSqlEffectKind, HirSqlExecution,
    HirSqlExecutionMethod, HirSqlMigrationGraph, HirSqlMigrationStep, HirSqlMigrationStepKind,
    HirSqlParameterSlot, HirSqlQueryAdapter, HirSqlQueryTemplate, HirStmt, HirTemplateFormatSpec,
    HirTemplateFormatSpecPart, HirTemplateInterpolation, HirTemplateOffsetMapping,
    HirTemplateSegment, HirTemplateStaticMapping, HirTemplateString, HirTupleTarget,
    HirTupleTargetBinding, HirWithItem, HirWithItemKind, InteropSummary,
    JsonIntegerBoundaryRequest, MetadataStore, MethodCallSource, MethodKind, Module,
    MutableArgumentTarget, MutableReceiverTarget, NominalView, Package, ParamConvention,
    ParamMutability, ParamOwnership, Place, PlaceProjection, PythonArrowDeclaration,
    PythonArrowKind, PythonArrowSchemaMode, PythonBufferAccess, PythonBufferDeclaration,
    PythonBufferLayout, PythonCallbackConcurrency, PythonCallbackDeclaration,
    PythonCallbackDispatch, PythonCallbackLifetime, PythonCleanupPolicy, PythonDlpackDeclaration,
    PythonDlpackDevice, PythonDlpackStreamMode, PythonInteropDeclaration,
    PythonInteropDecoratorKind, PythonInteropEffect, PythonInteropParameter, PythonParameterKind,
    PythonRecordExpansion, PythonTargetPath, ReceiverConvention, Record, RecordId, Result,
    RustCallbackBackpressure, RustCallbackOverflow, RustCallbackShutdown,
    RustInteropAbiRequirements, RustInteropArgument, RustInteropDeclaration,
    RustInteropDecoratorKind, RustInteropEffect, RustInteropValue, RustPayload, RustTargetPath,
    RustThreadsafeCallbackContract, SemanticBody, SemanticExports, SourceFile, SourceLocation,
    SourceOriginId, StaticMethodParam, StaticMethodSlot, StaticMethodSlotContext,
    StaticMethodSlotInputRole, StaticProgramValue, StaticSpecializationOutput,
    StructuralMethodExport, StructuralRecordField, StructuralRecordType, TemplatePayload,
    TemplateRole, Text, TextRange, Type, TypedDeclarationDescriptor, TypedDeclarationMetadata, err,
};
pub(crate) fn references(
    store: &MetadataStore,
    kind: u16,
    bytes: &[u8],
) -> Result<Vec<(RecordId, u16)>> {
    let mut refs = Vec::new();
    match kind {
        AdapterFieldDefault::KIND => decode_refs::<AdapterFieldDefault>(store, bytes, &mut refs)?,
        AdapterFieldPlan::KIND => decode_refs::<AdapterFieldPlan>(store, bytes, &mut refs)?,
        AdapterHandlerPlan::KIND => decode_refs::<AdapterHandlerPlan>(store, bytes, &mut refs)?,
        AppliedAdapterMetadata::KIND => {
            decode_refs::<AppliedAdapterMetadata>(store, bytes, &mut refs)?;
        }
        AttachedApiDeclaration::KIND => {
            decode_refs::<AttachedApiDeclaration>(store, bytes, &mut refs)?;
        }
        AttachedApiReceiver::KIND => decode_refs::<AttachedApiReceiver>(store, bytes, &mut refs)?,
        AttachedApiSetDeclaration::KIND => {
            decode_refs::<AttachedApiSetDeclaration>(store, bytes, &mut refs)?;
        }
        AttachedApiSetIdentity::KIND => {
            decode_refs::<AttachedApiSetIdentity>(store, bytes, &mut refs)?;
        }
        Binder::KIND => decode_refs::<Binder>(store, bytes, &mut refs)?,
        BindingId::KIND => decode_refs::<BindingId>(store, bytes, &mut refs)?,
        BodyRole::KIND => decode_refs::<BodyRole>(store, bytes, &mut refs)?,
        CallableIdentity::KIND => decode_refs::<CallableIdentity>(store, bytes, &mut refs)?,
        ClassAdapterMarkerDeclaration::KIND => {
            decode_refs::<ClassAdapterMarkerDeclaration>(store, bytes, &mut refs)?;
        }
        ClassAdapterProviderDeclaration::KIND => {
            decode_refs::<ClassAdapterProviderDeclaration>(store, bytes, &mut refs)?;
        }
        ClassAdapterSelection::KIND => {
            decode_refs::<ClassAdapterSelection>(store, bytes, &mut refs)?;
        }
        CompilerIntrinsicId::KIND => decode_refs::<CompilerIntrinsicId>(store, bytes, &mut refs)?,
        ConstSpecializationRequest::KIND => {
            decode_refs::<ConstSpecializationRequest>(store, bytes, &mut refs)?;
        }
        Declaration::KIND => decode_refs::<Declaration>(store, bytes, &mut refs)?,
        DeclarationDescriptorFunction::KIND => {
            decode_refs::<DeclarationDescriptorFunction>(store, bytes, &mut refs)?;
        }
        DeclarationDescriptorKind::KIND => {
            decode_refs::<DeclarationDescriptorKind>(store, bytes, &mut refs)?;
        }
        DeclarationKind::KIND => decode_refs::<DeclarationKind>(store, bytes, &mut refs)?,
        DeclarationMetadataTargetKind::KIND => {
            decode_refs::<DeclarationMetadataTargetKind>(store, bytes, &mut refs)?;
        }
        FieldIdentity::KIND => decode_refs::<FieldIdentity>(store, bytes, &mut refs)?,
        FixedIntType::KIND => decode_refs::<FixedIntType>(store, bytes, &mut refs)?,
        FragmentBoundary::KIND => decode_refs::<FragmentBoundary>(store, bytes, &mut refs)?,
        FragmentValidation::KIND => decode_refs::<FragmentValidation>(store, bytes, &mut refs)?,
        FunctionType::KIND => decode_refs::<FunctionType>(store, bytes, &mut refs)?,
        HirAsyncWithKind::KIND => decode_refs::<HirAsyncWithKind>(store, bytes, &mut refs)?,
        HirClass::KIND => decode_refs::<HirClass>(store, bytes, &mut refs)?,
        HirClassKind::KIND => decode_refs::<HirClassKind>(store, bytes, &mut refs)?,
        HirCollectionMutation::KIND => {
            decode_refs::<HirCollectionMutation>(store, bytes, &mut refs)?;
        }
        HirExceptHandler::KIND => decode_refs::<HirExceptHandler>(store, bytes, &mut refs)?,
        HirExpr::KIND => decode_refs::<HirExpr>(store, bytes, &mut refs)?,
        HirFStringPart::KIND => decode_refs::<HirFStringPart>(store, bytes, &mut refs)?,
        HirFunction::KIND => decode_refs::<HirFunction>(store, bytes, &mut refs)?,
        HirImport::KIND => decode_refs::<HirImport>(store, bytes, &mut refs)?,
        HirIteratorOp::KIND => decode_refs::<HirIteratorOp>(store, bytes, &mut refs)?,
        HirMatchArm::KIND => decode_refs::<HirMatchArm>(store, bytes, &mut refs)?,
        HirModule::KIND => decode_refs::<HirModule>(store, bytes, &mut refs)?,
        HirParam::KIND => decode_refs::<HirParam>(store, bytes, &mut refs)?,
        HirPattern::KIND => decode_refs::<HirPattern>(store, bytes, &mut refs)?,
        HirSqlBoundQuery::KIND => decode_refs::<HirSqlBoundQuery>(store, bytes, &mut refs)?,
        HirSqlCardinality::KIND => decode_refs::<HirSqlCardinality>(store, bytes, &mut refs)?,
        HirSqlEffectContract::KIND => decode_refs::<HirSqlEffectContract>(store, bytes, &mut refs)?,
        HirSqlEffectKind::KIND => decode_refs::<HirSqlEffectKind>(store, bytes, &mut refs)?,
        HirSqlExecution::KIND => decode_refs::<HirSqlExecution>(store, bytes, &mut refs)?,
        HirSqlExecutionMethod::KIND => {
            decode_refs::<HirSqlExecutionMethod>(store, bytes, &mut refs)?;
        }
        HirSqlMigrationGraph::KIND => decode_refs::<HirSqlMigrationGraph>(store, bytes, &mut refs)?,
        HirSqlMigrationStep::KIND => decode_refs::<HirSqlMigrationStep>(store, bytes, &mut refs)?,
        HirSqlMigrationStepKind::KIND => {
            decode_refs::<HirSqlMigrationStepKind>(store, bytes, &mut refs)?;
        }
        HirSqlParameterSlot::KIND => decode_refs::<HirSqlParameterSlot>(store, bytes, &mut refs)?,
        HirSqlQueryAdapter::KIND => decode_refs::<HirSqlQueryAdapter>(store, bytes, &mut refs)?,
        HirSqlQueryTemplate::KIND => decode_refs::<HirSqlQueryTemplate>(store, bytes, &mut refs)?,
        HirStmt::KIND => decode_refs::<HirStmt>(store, bytes, &mut refs)?,
        HirTemplateFormatSpec::KIND => {
            decode_refs::<HirTemplateFormatSpec>(store, bytes, &mut refs)?;
        }
        HirTemplateFormatSpecPart::KIND => {
            decode_refs::<HirTemplateFormatSpecPart>(store, bytes, &mut refs)?;
        }
        HirTemplateInterpolation::KIND => {
            decode_refs::<HirTemplateInterpolation>(store, bytes, &mut refs)?;
        }
        HirTemplateOffsetMapping::KIND => {
            decode_refs::<HirTemplateOffsetMapping>(store, bytes, &mut refs)?;
        }
        HirTemplateSegment::KIND => decode_refs::<HirTemplateSegment>(store, bytes, &mut refs)?,
        HirTemplateStaticMapping::KIND => {
            decode_refs::<HirTemplateStaticMapping>(store, bytes, &mut refs)?;
        }
        HirTemplateString::KIND => decode_refs::<HirTemplateString>(store, bytes, &mut refs)?,
        HirTupleTarget::KIND => decode_refs::<HirTupleTarget>(store, bytes, &mut refs)?,
        HirTupleTargetBinding::KIND => {
            decode_refs::<HirTupleTargetBinding>(store, bytes, &mut refs)?;
        }
        HirWithItem::KIND => decode_refs::<HirWithItem>(store, bytes, &mut refs)?,
        HirWithItemKind::KIND => decode_refs::<HirWithItemKind>(store, bytes, &mut refs)?,
        InteropSummary::KIND => decode_refs::<InteropSummary>(store, bytes, &mut refs)?,
        JsonIntegerBoundaryRequest::KIND => {
            decode_refs::<JsonIntegerBoundaryRequest>(store, bytes, &mut refs)?;
        }
        MethodCallSource::KIND => decode_refs::<MethodCallSource>(store, bytes, &mut refs)?,
        MethodKind::KIND => decode_refs::<MethodKind>(store, bytes, &mut refs)?,
        Module::KIND => decode_refs::<Module>(store, bytes, &mut refs)?,
        MutableArgumentTarget::KIND => {
            decode_refs::<MutableArgumentTarget>(store, bytes, &mut refs)?;
        }
        MutableReceiverTarget::KIND => {
            decode_refs::<MutableReceiverTarget>(store, bytes, &mut refs)?;
        }
        NominalView::KIND => decode_refs::<NominalView>(store, bytes, &mut refs)?,
        Package::KIND => decode_refs::<Package>(store, bytes, &mut refs)?,
        ParamConvention::KIND => decode_refs::<ParamConvention>(store, bytes, &mut refs)?,
        ParamMutability::KIND => decode_refs::<ParamMutability>(store, bytes, &mut refs)?,
        ParamOwnership::KIND => decode_refs::<ParamOwnership>(store, bytes, &mut refs)?,
        Place::KIND => decode_refs::<Place>(store, bytes, &mut refs)?,
        PlaceProjection::KIND => decode_refs::<PlaceProjection>(store, bytes, &mut refs)?,
        PythonArrowDeclaration::KIND => {
            decode_refs::<PythonArrowDeclaration>(store, bytes, &mut refs)?;
        }
        PythonArrowKind::KIND => decode_refs::<PythonArrowKind>(store, bytes, &mut refs)?,
        PythonArrowSchemaMode::KIND => {
            decode_refs::<PythonArrowSchemaMode>(store, bytes, &mut refs)?;
        }
        PythonBufferAccess::KIND => decode_refs::<PythonBufferAccess>(store, bytes, &mut refs)?,
        PythonBufferDeclaration::KIND => {
            decode_refs::<PythonBufferDeclaration>(store, bytes, &mut refs)?;
        }
        PythonBufferLayout::KIND => decode_refs::<PythonBufferLayout>(store, bytes, &mut refs)?,
        PythonCallbackConcurrency::KIND => {
            decode_refs::<PythonCallbackConcurrency>(store, bytes, &mut refs)?;
        }
        PythonCallbackDeclaration::KIND => {
            decode_refs::<PythonCallbackDeclaration>(store, bytes, &mut refs)?;
        }
        PythonCallbackDispatch::KIND => {
            decode_refs::<PythonCallbackDispatch>(store, bytes, &mut refs)?;
        }
        PythonCallbackLifetime::KIND => {
            decode_refs::<PythonCallbackLifetime>(store, bytes, &mut refs)?;
        }
        PythonCleanupPolicy::KIND => decode_refs::<PythonCleanupPolicy>(store, bytes, &mut refs)?,
        PythonDlpackDeclaration::KIND => {
            decode_refs::<PythonDlpackDeclaration>(store, bytes, &mut refs)?;
        }
        PythonDlpackDevice::KIND => decode_refs::<PythonDlpackDevice>(store, bytes, &mut refs)?,
        PythonDlpackStreamMode::KIND => {
            decode_refs::<PythonDlpackStreamMode>(store, bytes, &mut refs)?;
        }
        PythonInteropDeclaration::KIND => {
            decode_refs::<PythonInteropDeclaration>(store, bytes, &mut refs)?;
        }
        PythonInteropDecoratorKind::KIND => {
            decode_refs::<PythonInteropDecoratorKind>(store, bytes, &mut refs)?;
        }
        PythonInteropEffect::KIND => decode_refs::<PythonInteropEffect>(store, bytes, &mut refs)?,
        PythonInteropParameter::KIND => {
            decode_refs::<PythonInteropParameter>(store, bytes, &mut refs)?;
        }
        PythonParameterKind::KIND => decode_refs::<PythonParameterKind>(store, bytes, &mut refs)?,
        PythonRecordExpansion::KIND => {
            decode_refs::<PythonRecordExpansion>(store, bytes, &mut refs)?;
        }
        PythonTargetPath::KIND => decode_refs::<PythonTargetPath>(store, bytes, &mut refs)?,
        ReceiverConvention::KIND => decode_refs::<ReceiverConvention>(store, bytes, &mut refs)?,
        RustCallbackBackpressure::KIND => {
            decode_refs::<RustCallbackBackpressure>(store, bytes, &mut refs)?;
        }
        RustCallbackOverflow::KIND => decode_refs::<RustCallbackOverflow>(store, bytes, &mut refs)?,
        RustCallbackShutdown::KIND => decode_refs::<RustCallbackShutdown>(store, bytes, &mut refs)?,
        RustInteropAbiRequirements::KIND => {
            decode_refs::<RustInteropAbiRequirements>(store, bytes, &mut refs)?;
        }
        RustInteropArgument::KIND => decode_refs::<RustInteropArgument>(store, bytes, &mut refs)?,
        RustInteropDeclaration::KIND => {
            decode_refs::<RustInteropDeclaration>(store, bytes, &mut refs)?;
        }
        RustInteropDecoratorKind::KIND => {
            decode_refs::<RustInteropDecoratorKind>(store, bytes, &mut refs)?;
        }
        RustInteropEffect::KIND => decode_refs::<RustInteropEffect>(store, bytes, &mut refs)?,
        RustInteropValue::KIND => decode_refs::<RustInteropValue>(store, bytes, &mut refs)?,
        RustPayload::KIND => decode_refs::<RustPayload>(store, bytes, &mut refs)?,
        RustTargetPath::KIND => decode_refs::<RustTargetPath>(store, bytes, &mut refs)?,
        RustThreadsafeCallbackContract::KIND => {
            decode_refs::<RustThreadsafeCallbackContract>(store, bytes, &mut refs)?;
        }
        SemanticBody::KIND => decode_refs::<SemanticBody>(store, bytes, &mut refs)?,
        SemanticExports::KIND => decode_refs::<SemanticExports>(store, bytes, &mut refs)?,
        SourceFile::KIND => decode_refs::<SourceFile>(store, bytes, &mut refs)?,
        SourceLocation::KIND => decode_refs::<SourceLocation>(store, bytes, &mut refs)?,
        SourceOriginId::KIND => decode_refs::<SourceOriginId>(store, bytes, &mut refs)?,
        StaticMethodParam::KIND => decode_refs::<StaticMethodParam>(store, bytes, &mut refs)?,
        StaticMethodSlot::KIND => decode_refs::<StaticMethodSlot>(store, bytes, &mut refs)?,
        StaticMethodSlotContext::KIND => {
            decode_refs::<StaticMethodSlotContext>(store, bytes, &mut refs)?;
        }
        StaticMethodSlotInputRole::KIND => {
            decode_refs::<StaticMethodSlotInputRole>(store, bytes, &mut refs)?;
        }
        StaticProgramValue::KIND => decode_refs::<StaticProgramValue>(store, bytes, &mut refs)?,
        StaticSpecializationOutput::KIND => {
            decode_refs::<StaticSpecializationOutput>(store, bytes, &mut refs)?;
        }
        StructuralMethodExport::KIND => {
            decode_refs::<StructuralMethodExport>(store, bytes, &mut refs)?;
        }
        StructuralRecordField::KIND => {
            decode_refs::<StructuralRecordField>(store, bytes, &mut refs)?;
        }
        StructuralRecordType::KIND => decode_refs::<StructuralRecordType>(store, bytes, &mut refs)?,
        TemplatePayload::KIND => decode_refs::<TemplatePayload>(store, bytes, &mut refs)?,
        TemplateRole::KIND => decode_refs::<TemplateRole>(store, bytes, &mut refs)?,
        Text::KIND => decode_refs::<Text>(store, bytes, &mut refs)?,
        TextRange::KIND => decode_refs::<TextRange>(store, bytes, &mut refs)?,
        Type::KIND => decode_refs::<Type>(store, bytes, &mut refs)?,
        TypedDeclarationDescriptor::KIND => {
            decode_refs::<TypedDeclarationDescriptor>(store, bytes, &mut refs)?;
        }
        TypedDeclarationMetadata::KIND => {
            decode_refs::<TypedDeclarationMetadata>(store, bytes, &mut refs)?;
        }
        _ => return Err(err("unknown record kind")),
    }
    Ok(refs)
}

fn decode_refs<T: Record>(
    store: &MetadataStore,
    bytes: &[u8],
    refs: &mut Vec<(RecordId, u16)>,
) -> Result<()> {
    let value: T = serde_json::from_slice(bytes).map_err(|e| err(format!("record decode: {e}")))?;
    if serde_json::to_vec(&value).map_err(|e| err(e.to_string()))? != bytes {
        return Err(err("noncanonical record encoding"));
    }
    store.validate_fields(&value)?;
    value.references(refs);
    Ok(())
}
