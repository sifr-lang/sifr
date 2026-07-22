use super::arrow_certification::{
    is_dotted_target, validate_distributions, validate_fixture, DlpackCertification,
};
use std::path::Path;

pub(super) fn validate_dlpack_certifications(
    package_root: &Path,
    certifications: &[DlpackCertification],
) -> Result<(), String> {
    let mut previous_target: Option<&str> = None;
    for certification in certifications {
        if !is_dotted_target(&certification.target)
            || certification.producer_module.trim().is_empty()
            || certification.producer_type.trim().is_empty()
        {
            return Err("DLPack certification identities must be valid and non-empty".to_string());
        }
        if previous_target.is_some_and(|previous| previous >= certification.target.as_str()) {
            return Err("DLPack certifications must be sorted by unique target".to_string());
        }
        previous_target = Some(&certification.target);
        if certification.copy_performed
            || !certification.pointer_identity_verified
            || !certification.within_run_assertions
        {
            return Err(format!(
                "DLPack certification '{}' does not prove a within-run no-copy transfer",
                certification.target
            ));
        }
        if certification.exact_deleter_count != 1 {
            return Err(format!(
                "DLPack certification '{}' must prove exactly one deleter call",
                certification.target
            ));
        }
        validate_distributions(
            &certification.target,
            &certification.distributions,
            "DLPack",
        )?;
        validate_fixture(
            package_root,
            &certification.target,
            &certification.fixture,
            &certification.fixture_digest,
            "DLPack",
        )?;
    }
    Ok(())
}
