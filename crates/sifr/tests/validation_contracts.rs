#![allow(clippy::expect_used, clippy::unwrap_used)]

mod validation_contract_support;

#[test]
#[ignore = "invoked explicitly by verification area runners"]
fn test_validation_contract_matrix() {
    if let Err(error) = validation_contract_support::run() {
        panic!("{error}");
    }
}
