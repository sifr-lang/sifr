use super::{
    Binder, BodyRole, Declaration, DeclarationKind, FragmentBoundary, FragmentValidation,
    InteropSummary, Module, NominalView, Package, Record, RecordId, References, RustPayload,
    SemanticBody, SourceFile, SourceLocation, TemplatePayload, TemplateRole, Text, TextRange,
    sealed,
};
impl References for Text {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.value.references(out);
    }
}
impl sealed::Sealed for Text {}
impl Record for Text {
    const KIND: u16 = 130;
}
impl References for Package {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.name.references(out);
        self.version.references(out);
        self.source_identity.references(out);
    }
}
impl sealed::Sealed for Package {}
impl Record for Package {
    const KIND: u16 = 74;
}
impl References for Module {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.package.references(out);
        self.name.references(out);
        self.private.references(out);
        self.declarations.references(out);
        self.exports.references(out);
        self.hir_inventory.references(out);
        self.source.references(out);
        self.dependencies.references(out);
        self.semantic.references(out);
        self.rust.references(out);
        self.templates.references(out);
        self.interop.references(out);
    }
}
impl sealed::Sealed for Module {}
impl Record for Module {
    const KIND: u16 = 70;
}
impl References for Declaration {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.module.references(out);
        self.symbol.references(out);
        self.kind.references(out);
        self.binder.references(out);
        self.full_view.references(out);
        self.location.references(out);
    }
}
impl sealed::Sealed for Declaration {}
impl Record for Declaration {
    const KIND: u16 = 18;
}
impl References for DeclarationKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Function => {}
            Self::Class => {}
            Self::Protocol => {}
            Self::Enum => {}
            Self::Newtype => {}
            Self::Alias => {}
            Self::Constant => {}
            Self::Metadata => {}
        }
    }
}
impl sealed::Sealed for DeclarationKind {}
impl Record for DeclarationKind {
    const KIND: u16 = 21;
}
impl References for Binder {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.declaration.references(out);
        self.nested_path.references(out);
        self.parameters.references(out);
    }
}
impl sealed::Sealed for Binder {}
impl Record for Binder {
    const KIND: u16 = 9;
}
impl References for NominalView {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.name.references(out);
        self.identity.references(out);
        self.fields.references(out);
        self.methods.references(out);
        self.parent_class.references(out);
        self.enum_variants.references(out);
        self.newtype_inner.references(out);
    }
}
impl sealed::Sealed for NominalView {}
impl Record for NominalView {
    const KIND: u16 = 73;
}
impl References for SourceFile {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.relative_path.references(out);
        self.content_digest.references(out);
        self.byte_length.references(out);
    }
}
impl sealed::Sealed for SourceFile {}
impl Record for SourceFile {
    const KIND: u16 = 116;
}
impl References for SourceLocation {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.source.references(out);
        self.start.references(out);
        self.end.references(out);
        self.documentation.references(out);
    }
}
impl sealed::Sealed for SourceLocation {}
impl Record for SourceLocation {
    const KIND: u16 = 117;
}
impl References for TextRange {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.source.references(out);
        self.start.references(out);
        self.end.references(out);
    }
}
impl sealed::Sealed for TextRange {}
impl Record for TextRange {
    const KIND: u16 = 131;
}
impl References for SemanticBody {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.owner.references(out);
        self.role.references(out);
        self.expressions.references(out);
        self.functions.references(out);
        self.field_indices.references(out);
    }
}
impl sealed::Sealed for SemanticBody {}
impl Record for SemanticBody {
    const KIND: u16 = 114;
}
impl References for BodyRole {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Constant => {}
            Self::ConstFunction => {}
            Self::ArgumentDefaults => {}
            Self::FieldDefaults => {}
            Self::Descriptor => {}
        }
    }
}
impl sealed::Sealed for BodyRole {}
impl Record for BodyRole {
    const KIND: u16 = 11;
}
impl References for RustPayload {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.module.references(out);
        self.source.references(out);
        self.names.references(out);
        self.mappings.references(out);
        self.validation.references(out);
        self.generators.references(out);
    }
}
impl sealed::Sealed for RustPayload {}
impl Record for RustPayload {
    const KIND: u16 = 111;
}
impl References for FragmentValidation {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.bytes_digest.references(out);
        self.compiler_identity.references(out);
        self.target_identity.references(out);
        self.grammar_identity.references(out);
        self.boundary.references(out);
    }
}
impl sealed::Sealed for FragmentValidation {}
impl Record for FragmentValidation {
    const KIND: u16 = 26;
}
impl References for FragmentBoundary {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::StdlibModule => {}
        }
    }
}
impl sealed::Sealed for FragmentBoundary {}
impl Record for FragmentBoundary {
    const KIND: u16 = 25;
}
impl References for TemplatePayload {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.owner.references(out);
        self.role.references(out);
        self.function.references(out);
        self.class.references(out);
    }
}
impl sealed::Sealed for TemplatePayload {}
impl Record for TemplatePayload {
    const KIND: u16 = 128;
}
impl References for TemplateRole {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::GenericFunction => {}
            Self::GenericClass => {}
            Self::ProjectPolicyClass => {}
        }
    }
}
impl sealed::Sealed for TemplateRole {}
impl Record for TemplateRole {
    const KIND: u16 = 129;
}
impl References for InteropSummary {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.module.references(out);
        self.semantic_dependencies.references(out);
        self.body_dependencies.references(out);
        self.codegen_dependencies.references(out);
        self.intrinsics.references(out);
        self.rust_declarations.references(out);
        self.python_declarations.references(out);
        self.required_features.references(out);
        self.required_support.references(out);
    }
}
impl sealed::Sealed for InteropSummary {}
impl Record for InteropSummary {
    const KIND: u16 = 66;
}
