pub(in crate::lower) use sifr_ir::{
    HirDiagnostic, LoweringWarningDiagnostic, RevealTypeDiagnostic,
};
use sifr_type_system::Type;

pub(in crate::lower) fn fallback_error_type(name: &str) -> Type {
    Type::Class {
        identity: None,
        name: name.to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: vec![],
        parent_class: None,
    }
}
