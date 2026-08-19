use sifr_ir::HirClass;
use sifr_type_system::Type;

pub(crate) struct StructuralRecordField<'a> {
    pub(crate) name: &'a str,
    pub(crate) ty: &'a Type,
    pub(crate) inherited: bool,
}

pub(crate) fn structural_record_fields(class: &HirClass) -> Vec<StructuralRecordField<'_>> {
    let mut fields = class
        .parent_type
        .as_ref()
        .and_then(|parent| match parent.resolve_alias() {
            Type::Class { fields, .. } => Some(fields.as_slice()),
            _ => None,
        })
        .unwrap_or_default()
        .iter()
        .map(|(name, ty)| StructuralRecordField {
            name,
            ty,
            inherited: true,
        })
        .collect::<Vec<_>>();
    fields.extend(class.fields.iter().map(|(name, ty)| StructuralRecordField {
        name,
        ty,
        inherited: false,
    }));
    fields
}

pub(crate) fn concrete_record_fields(class: &HirClass) -> Vec<(String, Type)> {
    structural_record_fields(class)
        .into_iter()
        .map(|field| (field.name.to_string(), field.ty.clone()))
        .collect()
}
