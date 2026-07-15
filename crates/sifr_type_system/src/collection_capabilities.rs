use crate::Type;
use sifr_diagnostics::DiagnosticCode;

type CapabilityResult = Result<Type, (DiagnosticCode, String)>;

pub(crate) fn type_check_list_repetition(
    collection: &Type,
    count: &Type,
) -> Option<CapabilityResult> {
    let Type::List(element) = collection else {
        return None;
    };
    if count != &Type::Int {
        return None;
    }
    if matches!(element.resolve_alias(), Type::Any | Type::Unknown) {
        return Some(Err((
            DiagnosticCode::TYPE_MISMATCH,
            "cannot repeat a list whose element clone capability is not statically known"
                .to_string(),
        )));
    }
    if element.contains_affine_resource() {
        return Some(Err((
            DiagnosticCode::PYZC_INVALID_DECLARATION,
            "cannot repeat a list containing affine Python buffers because repetition duplicates its elements"
                .to_string(),
        )));
    }
    Some(Ok(collection.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_repetition_requires_known_cloneable_elements() {
        assert!(
            type_check_list_repetition(&Type::List(Box::new(Type::Int)), &Type::Int)
                .expect("list repetition")
                .is_ok()
        );
        for element in [Type::Any, Type::Unknown] {
            let error = type_check_list_repetition(&Type::List(Box::new(element)), &Type::Int)
                .expect("list repetition")
                .expect_err("dynamic elements must be rejected");
            assert_eq!(error.0, DiagnosticCode::TYPE_MISMATCH);
        }
        let buffer = Type::PythonBuffer(Box::new(Type::FixedInt(crate::FixedIntType::U8)));
        let error = type_check_list_repetition(&Type::List(Box::new(buffer)), &Type::Int)
            .expect("list repetition")
            .expect_err("affine elements must be rejected");
        assert_eq!(error.0, DiagnosticCode::PYZC_INVALID_DECLARATION);
    }
}
