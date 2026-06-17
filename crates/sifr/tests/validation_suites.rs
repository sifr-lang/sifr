#![allow(clippy::expect_used, clippy::unwrap_used)]

mod validation_suite_support;

#[test]
#[ignore = "invoked explicitly by verification area runners"]
fn test_validation_suite_matrix() {
    if let Err(error) = validation_suite_support::run() {
        panic!("{error}");
    }
}
