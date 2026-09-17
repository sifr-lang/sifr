use super::{
    HirTemplateFormatSpec, HirTemplateFormatSpecPart, HirTemplateInterpolation,
    HirTemplateOffsetMapping, HirTemplateSegment, HirTemplateStaticMapping, HirTemplateString,
    Record, RecordId, References, sealed,
};
impl References for HirTemplateSegment {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.value.references(out);
        self.mappings.references(out);
        self.virtual_range.references(out);
    }
}
impl sealed::Sealed for HirTemplateSegment {}
impl Record for HirTemplateSegment {
    const KIND: u16 = 59;
}
impl References for HirTemplateStaticMapping {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.source_range.references(out);
        self.virtual_range.references(out);
        self.offsets.references(out);
    }
}
impl sealed::Sealed for HirTemplateStaticMapping {}
impl Record for HirTemplateStaticMapping {
    const KIND: u16 = 60;
}
impl References for HirTemplateOffsetMapping {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.source_range.references(out);
        self.virtual_range.references(out);
    }
}
impl sealed::Sealed for HirTemplateOffsetMapping {}
impl Record for HirTemplateOffsetMapping {
    const KIND: u16 = 58;
}
impl References for HirTemplateFormatSpec {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.range.references(out);
        self.parts.references(out);
    }
}
impl sealed::Sealed for HirTemplateFormatSpec {}
impl Record for HirTemplateFormatSpec {
    const KIND: u16 = 55;
}
impl References for HirTemplateFormatSpecPart {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Literal(v0) => {
                v0.references(out);
            }
            Self::Interpolation {
                value,
                clone_from_borrow,
                source_range,
                conversion,
                format_spec,
            } => {
                value.references(out);
                clone_from_borrow.references(out);
                source_range.references(out);
                conversion.references(out);
                format_spec.references(out);
            }
        }
    }
}
impl sealed::Sealed for HirTemplateFormatSpecPart {}
impl Record for HirTemplateFormatSpecPart {
    const KIND: u16 = 56;
}
impl References for HirTemplateInterpolation {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.value.references(out);
        self.clone_from_borrow.references(out);
        self.value_type.references(out);
        self.source_range.references(out);
        self.expression_range.references(out);
        self.expression_source.references(out);
        self.virtual_range.references(out);
        self.conversion.references(out);
        self.format_spec.references(out);
    }
}
impl sealed::Sealed for HirTemplateInterpolation {}
impl Record for HirTemplateInterpolation {
    const KIND: u16 = 57;
}
impl References for HirTemplateString {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.source_range.references(out);
        self.virtual_source.references(out);
        self.segments.references(out);
        self.interpolations.references(out);
        self.ty.references(out);
    }
}
impl sealed::Sealed for HirTemplateString {}
impl Record for HirTemplateString {
    const KIND: u16 = 61;
}
